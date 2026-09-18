use axum::{
    extract::{Multipart, State},
    response::Response,
};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, task};
use tracing::error;

use crate::{
    engines::pdf::{common::validate_pdf_path, split::split_pdf_from_path as split_pdf_engine},
    error::AppError,
    features::convert::response::attachment_response,
    state::AppState,
    zip::{ZipEntry, create_stored_zip},
};

const PDF_CONTENT_TYPE: &str = "application/pdf";
const ZIP_CONTENT_TYPE: &str = "application/zip";
const MAX_SPLIT_RANGES: usize = 64;
const MAX_RANGE_INPUT_LENGTH: usize = 4 * 1024;
const MAX_INPUT_BYTES: usize = 500 * 1024 * 1024;
const MAX_MULTIPART_FIELDS: usize = 64;

struct TempPdfUpload {
    file: NamedTempFile,
    size: usize,
}

pub async fn split_pdf(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let (file, ranges) = read_split_request(multipart).await?;

    validate_total_input_size(file.size)?;
    validate_pdf_file(file.file.path())?;

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
        split_pdf_engine(file.file.path(), &ranges)
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

fn validate_total_input_size(size: usize) -> Result<(), AppError> {
    if size > MAX_INPUT_BYTES {
        return Err(AppError::bad_request(
            "split_input_too_large",
            "Ukuran PDF melebihi batas maksimum (500 MB)",
        ));
    }

    Ok(())
}

fn validate_pdf_file(path: &std::path::Path) -> Result<(), AppError> {
    validate_pdf_path(path).map_err(|message| AppError::bad_request("invalid_input", message))
}

async fn read_split_request(
    mut multipart: Multipart,
) -> Result<(TempPdfUpload, Vec<(u32, u32)>), AppError> {
    let mut file = None;
    let mut ranges = None;
    let mut field_count = 0usize;

    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                field_count += 1;
                if field_count > MAX_MULTIPART_FIELDS {
                    return Err(AppError::bad_request(
                        "too_many_fields",
                        "Jumlah field multipart dalam satu request terlalu banyak",
                    ));
                }

                match field.name().unwrap_or_default() {
                "file" => {
                    if file.is_some() {
                        return Err(AppError::bad_request(
                            "duplicate_file",
                            "Field file hanya boleh dikirim sekali",
                        ));
                    }

                    let temp = NamedTempFile::new().map_err(|error| {
                        error!(%error, "Gagal membuat temporary file PDF");
                        AppError::internal(
                            "tempfile_failed",
                            "Gagal menyiapkan penyimpanan sementara",
                        )
                    })?;
                    let mut output = tokio::fs::File::from_std(temp.reopen().map_err(|error| {
                        error!(%error, "Gagal membuka temporary file PDF");
                        AppError::internal("tempfile_failed", "Gagal membuka penyimpanan sementara")
                    })?);

                    let mut size = 0usize;
                    while let Some(chunk) = field.chunk().await.map_err(|error| {
                        error!(%error, "Gagal membaca file PDF");
                        AppError::bad_request(
                            "invalid_upload",
                            "Gagal membaca file PDF yang diunggah",
                        )
                    })? {
                        size = size.checked_add(chunk.len()).ok_or_else(|| {
                            AppError::bad_request(
                                "split_input_too_large",
                                "Ukuran PDF melebihi kapasitas yang didukung",
                            )
                        })?;

                        if size > MAX_INPUT_BYTES {
                            return Err(AppError::bad_request(
                                "split_input_too_large",
                                "Ukuran PDF melebihi batas maksimum (500 MB)",
                            ));
                        }

                        output.write_all(&chunk).await.map_err(|error| {
                            error!(%error, "Gagal menulis temporary PDF");
                            AppError::internal(
                                "tempfile_write_failed",
                                "Gagal menyimpan PDF sementara",
                            )
                        })?;
                    }

                    output.flush().await.map_err(|error| {
                        error!(%error, "Gagal flush temporary PDF");
                        AppError::internal("tempfile_write_failed", "Gagal menyimpan PDF sementara")
                    })?;

                    if size == 0 {
                        return Err(AppError::bad_request("empty_file", "File PDF kosong"));
                    }

                    file = Some(TempPdfUpload { file: temp, size });
                }
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
                }
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

    let file =
        file.ok_or_else(|| AppError::bad_request("file_missing", "Field file tidak ditemukan"))?;
    let ranges = ranges
        .ok_or_else(|| AppError::bad_request("ranges_missing", "Field ranges tidak ditemukan"))?;
    Ok((file, ranges))
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
