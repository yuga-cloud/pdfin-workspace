use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::header,
    response::IntoResponse,
};
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tracing::error;

use crate::{
    engines::pdf::{
        common::validate_pdf_path, pages_disk::manage_pages_from_path as manage_pages_engine,
    },
    error::AppError,
    state::AppState,
};

const PDF_CONTENT_TYPE: &str = "application/pdf";
const MAX_PDF_UPLOAD_BYTES: usize = 500 * 1024 * 1024;
const MAX_MULTIPART_FIELDS: usize = 64;
const MAX_PAGE_ORDER_LENGTH: usize = 16 * 1024;

pub async fn manage_pages(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let temp = NamedTempFile::new().map_err(|error| {
        error!(%error, "Gagal membuat temporary file PDF");
        AppError::internal("tempfile_failed", "Gagal menyiapkan penyimpanan sementara")
    })?;

    let input_path = temp.path().to_owned();
    let mut file_present = false;
    let mut page_order = None;
    let mut field_count = 0usize;

    while let Some(mut field) = multipart.next_field().await.map_err(|error| {
        error!(%error, "Gagal membaca multipart request");
        AppError::bad_request("invalid_multipart", "Request multipart tidak valid")
    })? {
        field_count += 1;
        if field_count > MAX_MULTIPART_FIELDS {
            return Err(AppError::bad_request(
                "too_many_fields",
                "Jumlah field multipart dalam satu request terlalu banyak",
            ));
        }

        match field.name().unwrap_or_default() {
            "file" => {
                if file_present {
                    return Err(AppError::bad_request(
                        "duplicate_file",
                        "Hanya satu file PDF yang boleh dikirim",
                    ));
                }

                let mut output = tokio::fs::File::from_std(temp.reopen().map_err(|error| {
                    error!(%error, "Gagal membuka temporary file PDF");
                    AppError::internal("tempfile_failed", "Gagal membuka penyimpanan sementara")
                })?);

                let mut size = 0usize;
                while let Some(chunk) = field.chunk().await.map_err(|error| {
                    error!(%error, "Gagal membaca file PDF");
                    AppError::bad_request("invalid_upload", "Gagal membaca file PDF yang diunggah")
                })? {
                    size = size.checked_add(chunk.len()).ok_or_else(|| {
                        AppError::bad_request(
                            "pages_input_too_large",
                            "Ukuran PDF melebihi batas maksimum (500 MB)",
                        )
                    })?;

                    if size > MAX_PDF_UPLOAD_BYTES {
                        return Err(AppError::bad_request(
                            "pages_input_too_large",
                            "Ukuran PDF melebihi batas maksimum (500 MB)",
                        ));
                    }

                    output.write_all(&chunk).await.map_err(|error| {
                        error!(%error, "Gagal menulis temporary PDF");
                        AppError::internal("tempfile_write_failed", "Gagal menyimpan PDF sementara")
                    })?;
                }

                output.flush().await.map_err(|error| {
                    error!(%error, "Gagal flush temporary PDF");
                    AppError::internal("tempfile_write_failed", "Gagal menyimpan PDF sementara")
                })?;

                if size == 0 {
                    return Err(AppError::bad_request("empty_file", "File PDF kosong"));
                }

                file_present = true;
            }
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
        }
    }

    if !file_present {
        return Err(AppError::bad_request(
            "file_missing",
            "Field file tidak ditemukan",
        ));
    }

    let page_order = page_order
        .ok_or_else(|| AppError::bad_request("pages_missing", "Field pages tidak ditemukan"))?;

    validate_pdf_path(&input_path)
        .map_err(|message| AppError::bad_request("invalid_input", message))?;

    let permit = state
        .pdf_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| {
            error!(%error, "PDF semaphore tidak tersedia");
            AppError::service_unavailable("pdf_busy", "Server sedang terlalu sibuk memproses PDF")
        })?;

    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        manage_pages_engine(&input_path, &page_order)
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

    let mut seen = std::collections::HashSet::with_capacity(pages.len());
    if pages.iter().any(|page| !seen.insert(*page)) {
        return Err(AppError::bad_request(
            "duplicate_pages",
            "Urutan halaman tidak boleh mengandung halaman yang sama lebih dari sekali",
        ));
    }

    Ok(pages)
}
