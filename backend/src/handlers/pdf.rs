use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::header,
    response::IntoResponse,
};
use tokio::task;
use tracing::error;

use crate::{
    engines::pdf::{
        merge::merge_pdfs as merge_pdf_engine, pages::manage_pages as manage_pages_engine,
        rotate::rotate_pdf as rotate_pdf_engine,
    },
    error::AppError,
    state::AppState,
};

const DEFAULT_ROTATION_DEGREES: i64 = 90;
const PDF_CONTENT_TYPE: &str = "application/pdf";
const MAX_MERGE_FILES: usize = 50;
const MAX_PAGE_ORDER_LENGTH: usize = 16 * 1024;

pub async fn merge_pdfs(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let files = read_multiple_files(multipart).await?;

    if files.len() < 2 {
        return Err(AppError::bad_request(
            "insufficient_files",
            "Minimal dua file PDF diperlukan untuk digabungkan",
        ));
    }

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
        let file_refs: Vec<&[u8]> = files.iter().map(Bytes::as_ref).collect();
        merge_pdf_engine(&file_refs)
    })
    .await
    .map_err(|error| {
        error!(%error, "Merge worker mengalami panic");
        AppError::internal(
            "merge_worker_failed",
            "Worker penggabungan PDF mengalami kegagalan",
        )
    })?;

    let pdf_bytes = result.map_err(|error| {
        error!(%error, "Gagal menggabungkan PDF: {error}");
        AppError::internal("merge_failed", "Gagal menggabungkan file PDF")
    })?;

    Ok((
        [(header::CONTENT_TYPE, PDF_CONTENT_TYPE)],
        Bytes::from(pdf_bytes),
    ))
}

pub async fn manage_pages(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (data, page_order) = read_manage_pages_request(multipart).await?;

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
        manage_pages_engine(&data, &page_order)
    })
    .await
    .map_err(|error| {
        error!(%error, "Page management worker mengalami panic");
        AppError::internal(
            "page_management_worker_failed",
            "Worker pengaturan halaman mengalami kegagalan",
        )
    })?;

    let pdf_bytes = result.map_err(|error| {
        error!(%error, "Gagal mengatur halaman PDF: {error}");
        AppError::internal("page_management_failed", "Gagal mengatur halaman PDF")
    })?;

    Ok((
        [(header::CONTENT_TYPE, PDF_CONTENT_TYPE)],
        Bytes::from(pdf_bytes),
    ))
}

pub async fn rotate(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (data, degrees) = read_rotate_request(multipart).await?;
    validate_rotation_degrees(degrees)?;

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
        rotate_pdf_engine(&data, degrees)
    })
    .await
    .map_err(|error| {
        error!(%error, "Rotate worker mengalami panic");
        AppError::internal(
            "rotate_worker_failed",
            "Worker rotasi PDF mengalami kegagalan",
        )
    })?;

    let pdf_bytes = result.map_err(|error| {
        error!(%error, "Gagal memutar PDF: {error}");
        AppError::internal("rotate_failed", "Gagal memutar file PDF")
    })?;

    Ok((
        [(header::CONTENT_TYPE, PDF_CONTENT_TYPE)],
        Bytes::from(pdf_bytes),
    ))
}

async fn read_multiple_files(mut multipart: Multipart) -> Result<Vec<Bytes>, AppError> {
    let mut files = Vec::new();

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) if field.name() == Some("file") => {
                if files.len() >= MAX_MERGE_FILES {
                    return Err(AppError::bad_request(
                        "too_many_files",
                        "Jumlah file PDF dalam satu request terlalu banyak",
                    ));
                }

                match field.bytes().await {
                    Ok(bytes) if !bytes.is_empty() => files.push(bytes),
                    Ok(_) => {
                        return Err(AppError::bad_request(
                            "empty_file",
                            "Salah satu file PDF kosong",
                        ));
                    }
                    Err(error) => {
                        error!(%error, "Gagal membaca file PDF");
                        return Err(AppError::bad_request(
                            "invalid_upload",
                            "Gagal membaca file PDF yang diunggah",
                        ));
                    }
                }
            }
            Ok(Some(_)) => continue,
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

    if files.is_empty() {
        return Err(AppError::bad_request(
            "file_missing",
            "Field file tidak ditemukan",
        ));
    }

    Ok(files)
}

async fn read_manage_pages_request(
    mut multipart: Multipart,
) -> Result<(Bytes, Vec<u32>), AppError> {
    let mut file_bytes = None;
    let mut page_order = None;

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
                "pages" => {
                    let text = field.text().await.map_err(|error| {
                        error!(%error, "Gagal membaca pages");
                        AppError::bad_request("invalid_pages", "Gagal membaca urutan halaman")
                    })?;

                    if text.len() > MAX_PAGE_ORDER_LENGTH {
                        return Err(AppError::bad_request(
                            "pages_too_long",
                            "Urutan halaman terlalu panjang",
                        ));
                    }

                    page_order = Some(parse_page_order(&text)?);
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
    let page_order = page_order
        .ok_or_else(|| AppError::bad_request("pages_missing", "Field pages tidak ditemukan"))?;
    Ok((data, page_order))
}

async fn read_rotate_request(mut multipart: Multipart) -> Result<(Bytes, i64), AppError> {
    let mut file_bytes = None;
    let mut degrees = DEFAULT_ROTATION_DEGREES;

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
                "degrees" => {
                    let text = field.text().await.map_err(|error| {
                        error!(%error, "Gagal membaca degrees");
                        AppError::bad_request("invalid_degrees", "Gagal membaca nilai rotasi")
                    })?;
                    degrees = text.trim().parse::<i64>().map_err(|error| {
                        error!(%error, "Nilai degrees tidak valid: {error}");
                        AppError::bad_request("invalid_degrees", "Nilai derajat rotasi tidak valid")
                    })?;
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
    Ok((data, degrees))
}

fn validate_rotation_degrees(degrees: i64) -> Result<(), AppError> {
    if degrees % 90 != 0 {
        return Err(AppError::bad_request(
            "invalid_degrees",
            "Sudut rotasi harus merupakan kelipatan 90 derajat",
        ));
    }

    Ok(())
}

fn parse_page_order(input: &str) -> Result<Vec<u32>, AppError> {
    let pages = input
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<u32>().map_err(|_| {
                AppError::bad_request(
                    "invalid_pages",
                    format!("Nomor halaman tidak valid: {part}"),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if pages.is_empty() {
        return Err(AppError::bad_request(
            "invalid_pages",
            "Urutan halaman tidak boleh kosong",
        ));
    }

    if pages.len() > MAX_PAGE_ORDER_LENGTH / 2 || pages.contains(&0) {
        return Err(AppError::bad_request(
            "invalid_pages",
            "Urutan halaman tidak valid atau terlalu panjang",
        ));
    }

    Ok(pages)
}
