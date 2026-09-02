use std::path::Path;

use axum::{
    body::Bytes,
    extract::{multipart::Field, Multipart, State},
    http::header,
    response::IntoResponse,
};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, task};
use tracing::error;

use crate::{
    engines::optimize::{
        page_numbers::add_page_numbers_from_path as add_page_numbers_engine,
        watermark::add_watermark_from_path as add_watermark_engine,
    },
    error::AppError,
    state::AppState,
};

const PDF_CONTENT_TYPE: &str = "application/pdf";
const MAX_PDF_UPLOAD_BYTES: usize = 500 * 1024 * 1024;
const MAX_WATERMARK_TEXT_BYTES: usize = 1024;

struct TempPdfUpload {
    file: NamedTempFile,
}

pub async fn add_page_numbers(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    run_pdf_operation(
        state,
        multipart,
        add_page_numbers_engine,
        "Nomor halaman PDF",
    )
    .await
}

pub async fn add_watermark(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (file, text) = read_watermark_request(&mut multipart).await?;

    let permit = acquire_pdf_permit(&state, "Watermark PDF").await?;
    let input_path = file.file.path().to_owned();

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        let _file = file;
        add_watermark_engine(&input_path, &text)
    })
    .await
    .map_err(|error| {
        error!(%error, "Watermark worker mengalami panic");
        AppError::internal(
            "watermark_worker_failed",
            "Worker watermark mengalami kegagalan",
        )
    })?;

    let bytes = result.map_err(|error| {
        error!(%error, "Watermark gagal");
        AppError::internal("watermark_failed", "Gagal menambahkan watermark ke PDF")
    })?;

    Ok((
        [(header::CONTENT_TYPE, PDF_CONTENT_TYPE)],
        Bytes::from(bytes),
    ))
}

async fn stream_pdf_field(mut field: Field<'_>) -> Result<TempPdfUpload, AppError> {
    let temp = NamedTempFile::new().map_err(|error| {
        error!(%error, "Gagal membuat temporary file PDF");
        AppError::internal(
            "tempfile_failed",
            "Gagal menyiapkan penyimpanan sementara",
        )
    })?;

    let std_file = temp.reopen().map_err(|error| {
        error!(%error, "Gagal membuka temporary file PDF");
        AppError::internal(
            "tempfile_failed",
            "Gagal membuka penyimpanan sementara",
        )
    })?;
    let mut output = tokio::fs::File::from_std(std_file);
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
                "pdf_upload_too_large",
                "Ukuran PDF melebihi batas maksimum (500 MB)",
            )
        })?;

        if size > MAX_PDF_UPLOAD_BYTES {
            return Err(AppError::bad_request(
                "pdf_upload_too_large",
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

    Ok(TempPdfUpload { file: temp })
}

async fn read_single_file(mut multipart: Multipart) -> Result<TempPdfUpload, AppError> {
    loop {
        match multipart.next_field().await {
            Ok(Some(field)) if field.name() == Some("file") => {
                return stream_pdf_field(field).await;
            }
            Ok(Some(_)) => continue,
            Ok(None) => {
                return Err(AppError::bad_request(
                    "file_missing",
                    "Field file tidak ditemukan",
                ));
            }
            Err(error) => {
                error!(%error, "Gagal membaca multipart request");
                return Err(AppError::bad_request(
                    "invalid_multipart",
                    "Request multipart tidak valid",
                ));
            }
        }
    }
}

async fn read_watermark_request(
    multipart: &mut Multipart,
) -> Result<(TempPdfUpload, String), AppError> {
    let mut file = None;
    let mut text = None;

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => match field.name().unwrap_or_default() {
                "file" => {
                    if file.is_some() {
                        return Err(AppError::bad_request(
                            "duplicate_file",
                            "Field file hanya boleh dikirim sekali",
                        ));
                    }
                    file = Some(stream_pdf_field(field).await?);
                }
                "text" => {
                    if text.is_some() {
                        return Err(AppError::bad_request(
                            "duplicate_watermark",
                            "Field text hanya boleh dikirim sekali",
                        ));
                    }

                    let value = field.text().await.map_err(|error| {
                        error!(%error, "Gagal membaca teks watermark");
                        AppError::bad_request(
                            "invalid_watermark",
                            "Gagal membaca teks watermark",
                        )
                    })?;

                    if value.trim().is_empty() {
                        return Err(AppError::bad_request(
                            "invalid_watermark",
                            "Teks watermark tidak boleh kosong",
                        ));
                    }

                    if value.len() > MAX_WATERMARK_TEXT_BYTES {
                        return Err(AppError::bad_request(
                            "invalid_watermark",
                            "Teks watermark melebihi batas maksimum (1024 byte)",
                        ));
                    }

                    if !value.is_ascii() {
                        return Err(AppError::bad_request(
                            "invalid_watermark",
                            "Teks watermark saat ini hanya mendukung karakter ASCII",
                        ));
                    }

                    text = Some(value);
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

    let file =
        file.ok_or_else(|| AppError::bad_request("file_missing", "Field file tidak ditemukan"))?;
    let text = text
        .ok_or_else(|| AppError::bad_request("watermark_missing", "Field text tidak ditemukan"))?;
    Ok((file, text))
}

async fn acquire_pdf_permit(
    state: &AppState,
    operation: &'static str,
) -> Result<tokio::sync::OwnedSemaphorePermit, AppError> {
    state
        .pdf_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| {
            error!(%error, operation, "PDF semaphore tidak tersedia");
            AppError::service_unavailable("pdf_busy", "Server sedang terlalu sibuk memproses PDF")
        })
}

async fn run_pdf_operation<F>(
    state: AppState,
    multipart: Multipart,
    engine: F,
    operation: &'static str,
) -> Result<impl IntoResponse, AppError>
where
    F: FnOnce(&Path) -> Result<Vec<u8>, String> + Send + 'static,
{
    let file = read_single_file(multipart).await?;
    let input_path = file.file.path().to_owned();
    let permit = acquire_pdf_permit(&state, operation).await?;

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        let _file = file;
        engine(&input_path)
    })
    .await
    .map_err(|error| {
        error!(%error, operation, "Optimize worker mengalami panic");
        AppError::internal(
            "optimize_worker_failed",
            "Worker optimasi PDF mengalami kegagalan",
        )
    })?;

    let bytes = result.map_err(|error| {
        error!(%error, operation, "Operasi optimasi gagal");
        AppError::internal("optimize_failed", "Gagal melakukan operasi optimasi PDF")
    })?;

    Ok((
        [(header::CONTENT_TYPE, PDF_CONTENT_TYPE)],
        Bytes::from(bytes),
    ))
}
