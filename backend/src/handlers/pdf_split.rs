use axum::{
    body::Bytes,
    extract::{Multipart, State},
    response::Response,
};
use tokio::task;
use tracing::error;

use crate::{
    engines::{common::validate_input, pdf::split::split_pdf as split_pdf_engine},
    error::AppError,
    features::convert::response::attachment_response,
    state::AppState,
    zip::{ZipEntry, create_stored_zip},
};

const PDF_CONTENT_TYPE: &str = "application/pdf";
const ZIP_CONTENT_TYPE: &str = "application/zip";
const MAX_SPLIT_RANGES: usize = 100;
const MAX_RANGE_INPUT_LENGTH: usize = 4 * 1024;

fn validate_pdf_input(bytes: &[u8]) -> Result<(), AppError> {
    validate_input(bytes, "PDF").map_err(|message| AppError::bad_request("invalid_input", message))
}

pub async fn split_pdf(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let (data, ranges) = read_split_request(multipart).await?;
    validate_pdf_input(&data)?;

    let permit = state
        .pdf_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| {
            error!(%error, "PDF semaphore tidak tersedia");
            AppError::service_unavailable("pdf_busy", "Server sedang terlalu sibuk memproses PDF")
        })?;

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        split_pdf_engine(&data, &ranges)
    })
    .await
    .map_err(|error| {
        error!(%error, "Split worker mengalami panic");
        AppError::internal(
            "split_worker_failed",
            "Worker pemisahan PDF mengalami kegagalan",
        )
    })?;

    let pdf_files = result.map_err(|error| {
        error!(%error, "Gagal memisahkan PDF: {error}");
        AppError::internal("split_failed", "Gagal memisahkan file PDF")
    })?;

    if pdf_files.is_empty() {
        return Err(AppError::internal(
            "empty_split_result",
            "Pemisahan PDF tidak menghasilkan file",
        ));
    }

    if pdf_files.len() == 1 {
        let pdf = pdf_files
            .into_iter()
            .next()
            .expect("single split result harus tersedia");
        return Ok(attachment_response(PDF_CONTENT_TYPE, "split.pdf", pdf));
    }

    let names = (1..=pdf_files.len())
        .map(|number| format!("split-{number:03}.pdf"))
        .collect::<Vec<_>>();
    let entries = pdf_files
        .iter()
        .zip(names.iter())
        .map(|(data, name)| ZipEntry {
            name: name.as_str(),
            data: data.as_slice(),
        })
        .collect::<Vec<_>>();

    let archive = create_stored_zip(&entries).map_err(|error| {
        error!(%error, "Gagal membuat ZIP hasil split PDF");
        AppError::internal(
            "split_zip_failed",
            "Gagal membuat arsip hasil pemisahan PDF",
        )
    })?;

    Ok(attachment_response(
        ZIP_CONTENT_TYPE,
        "split-results.zip",
        archive,
    ))
}

async fn read_split_request(
    mut multipart: Multipart,
) -> Result<(Bytes, Vec<(u32, u32)>), AppError> {
    let mut file_bytes = None;
    let mut ranges = None;

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => match field.name().unwrap_or_default() {
                "file" => match field.bytes().await {
                    Ok(bytes) if !bytes.is_empty() => file_bytes = Some(bytes),
                    Ok(_) => return Err(AppError::bad_request("empty_file", "File PDF kosong")),
                    Err(error) => {
                        error!(%error, "Gagal membaca file PDF");
                        return Err(AppError::bad_request(
                            "invalid_upload",
                            "Gagal membaca file PDF yang diunggah",
                        ));
                    }
                },
                "ranges" => {
                    let text = field.text().await.map_err(|error| {
                        error!(%error, "Gagal membaca ranges");
                        AppError::bad_request("invalid_ranges", "Gagal membaca rentang halaman")
                    })?;

                    if text.len() > MAX_RANGE_INPUT_LENGTH {
                        return Err(AppError::bad_request(
                            "ranges_too_long",
                            "Parameter rentang halaman terlalu panjang",
                        ));
                    }

                    ranges = Some(parse_ranges(&text)?);
                }
                _ => {}
            },
            Ok(None) => break,
            Err(error) => {
                error!(%error, "Gagal membaca multipart request");
                return Err(AppError::bad_request(
                    "invalid_multipart",
                    "Request multipart tidak valid",
                ));
            }
        }
    }

    let data = file_bytes
        .ok_or_else(|| AppError::bad_request("file_missing", "Field file tidak ditemukan"))?;
    let ranges = ranges
        .ok_or_else(|| AppError::bad_request("ranges_missing", "Field ranges tidak ditemukan"))?;
    Ok((data, ranges))
}

fn parse_ranges(input: &str) -> Result<Vec<(u32, u32)>, AppError> {
    let mut ranges = Vec::new();

    for part in input
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if ranges.len() >= MAX_SPLIT_RANGES {
            return Err(AppError::bad_request(
                "too_many_ranges",
                "Jumlah rentang halaman terlalu banyak",
            ));
        }

        let mut values = part.splitn(2, '-');
        let start = values
            .next()
            .ok_or_else(|| AppError::bad_request("invalid_ranges", "Rentang halaman tidak valid"))?
            .parse::<u32>()
            .map_err(|_| {
                AppError::bad_request(
                    "invalid_ranges",
                    format!("Nomor halaman tidak valid: {part}"),
                )
            })?;
        let end = values.next().unwrap_or(part).parse::<u32>().map_err(|_| {
            AppError::bad_request(
                "invalid_ranges",
                format!("Nomor halaman tidak valid: {part}"),
            )
        })?;

        if start == 0 || end == 0 || start > end {
            return Err(AppError::bad_request(
                "invalid_ranges",
                format!("Rentang halaman tidak valid: {part}"),
            ));
        }

        ranges.push((start, end));
    }

    if ranges.is_empty() {
        return Err(AppError::bad_request(
            "invalid_ranges",
            "Pilih setidaknya satu rentang halaman",
        ));
    }

    Ok(ranges)
}
