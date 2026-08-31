use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::header,
    response::IntoResponse,
};
use tokio::task;
use tracing::error;

use crate::{
    engines::{
        common::validate_input,
        optimize::{
            compress::{CompressionQuality, compress_pdf as compress_pdf_engine},
            page_numbers::add_page_numbers as add_page_numbers_engine,
            watermark::add_watermark as add_watermark_engine,
        },
    },
    error::AppError,
    state::AppState,
};

const PDF_CONTENT_TYPE: &str = "application/pdf";

fn validate_pdf_input(bytes: &[u8]) -> Result<(), AppError> {
    validate_input(bytes, "PDF")
        .map_err(|message| AppError::bad_request("invalid_input", message))
}

pub async fn compress_pdf(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (data, quality) = read_compress_request(&mut multipart).await?;
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
        compress_pdf_engine(&data, quality)
    })
    .await
    .map_err(|error| {
        error!(%error, "Compression worker mengalami panic");
        AppError::internal(
            "compress_worker_failed",
            "Worker kompresi PDF mengalami kegagalan",
        )
    })?;

    let bytes = result.map_err(|error| {
        error!(%error, ?quality, "Kompresi PDF gagal");
        AppError::internal("compress_failed", "Gagal melakukan kompresi PDF")
    })?;

    Ok((
        [(header::CONTENT_TYPE, PDF_CONTENT_TYPE)],
        Bytes::from(bytes),
    ))
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
    let (data, text) = read_watermark_request(&mut multipart).await?;
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
        add_watermark_engine(&data, &text)
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

async fn read_compress_request(
    multipart: &mut Multipart,
) -> Result<(Bytes, CompressionQuality), AppError> {
    let mut file_bytes = None;
    let mut quality = None;

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
                "quality" => {
                    let value = field.text().await.map_err(|error| {
                        error!(%error, "Gagal membaca parameter quality");
                        AppError::bad_request(
                            "invalid_quality",
                            "Parameter kualitas kompresi tidak valid",
                        )
                    })?;
                    quality =
                        Some(CompressionQuality::from_str(value.trim()).ok_or_else(|| {
                            AppError::bad_request(
                                "invalid_quality",
                                "Kualitas kompresi harus high, medium, atau low",
                            )
                        })?);
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
    Ok((data, quality.unwrap_or(CompressionQuality::Medium)))
}

async fn read_single_file(mut multipart: Multipart) -> Result<Bytes, AppError> {
    loop {
        match multipart.next_field().await {
            Ok(Some(field)) if field.name() == Some("file") => {
                return match field.bytes().await {
                    Ok(bytes) if !bytes.is_empty() => Ok(bytes),
                    Ok(_) => Err(AppError::bad_request("empty_file", "File PDF kosong")),
                    Err(error) => {
                        error!(%error, "Gagal membaca file PDF");
                        Err(AppError::bad_request(
                            "invalid_upload",
                            "Gagal membaca file PDF yang diunggah",
                        ))
                    }
                };
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

async fn read_watermark_request(multipart: &mut Multipart) -> Result<(Bytes, String), AppError> {
    let mut file_bytes = None;
    let mut text = None;

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
                "text" => match field.text().await {
                    Ok(value) if !value.trim().is_empty() => text = Some(value),
                    Ok(_) => {
                        return Err(AppError::bad_request(
                            "invalid_watermark",
                            "Teks watermark tidak boleh kosong",
                        ));
                    }
                    Err(error) => {
                        error!(%error, "Gagal membaca teks watermark");
                        return Err(AppError::bad_request(
                            "invalid_watermark",
                            "Gagal membaca teks watermark",
                        ));
                    }
                },
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
    let text = text
        .ok_or_else(|| AppError::bad_request("watermark_missing", "Field text tidak ditemukan"))?;
    Ok((data, text))
}

async fn run_pdf_operation<F>(
    state: AppState,
    multipart: Multipart,
    engine: F,
    operation: &'static str,
) -> Result<impl IntoResponse, AppError>
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>, String> + Send + 'static,
{
    let data = read_single_file(multipart).await?;
    validate_pdf_input(&data)?;

    let permit = state
        .pdf_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| {
            error!(%error, operation, "PDF semaphore tidak tersedia");
            AppError::service_unavailable("pdf_busy", "Server sedang terlalu sibuk memproses PDF")
        })?;

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        engine(&data)
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
