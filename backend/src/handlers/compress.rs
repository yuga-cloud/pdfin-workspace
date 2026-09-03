use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::header,
    response::IntoResponse,
};
use tempfile::tempdir;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    task,
};
use tracing::error;

use crate::{
    engines::{
        common::validate_input,
        optimize::{compress::CompressionQuality, compress_disk::compress_pdf_from_path},
    },
    error::AppError,
    state::AppState,
};

const PDF_CONTENT_TYPE: &str = "application/pdf";
const MAX_PDF_UPLOAD_BYTES: usize = 500 * 1024 * 1024;
const MAX_QUALITY_FIELD_BYTES: usize = 32;
const PDF_HEADER_SCAN_BYTES: usize = 1024;

pub async fn compress_pdf(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let temp_dir = tempdir().map_err(|error| {
        error!(%error, "Gagal membuat temporary directory untuk kompresi");
        AppError::internal(
            "compress_tempdir_failed",
            "Gagal menyiapkan ruang sementara untuk kompresi PDF",
        )
    })?;

    let input_path = temp_dir.path().join("input.pdf");
    let mut file = false;
    let mut quality = CompressionQuality::Medium;

    while let Some(mut field) = multipart.next_field().await.map_err(|error| {
        error!(%error, "Gagal membaca multipart request untuk kompresi");
        AppError::bad_request("invalid_multipart", "Request multipart tidak valid")
    })? {
        match field.name().unwrap_or_default() {
            "file" => {
                if file {
                    return Err(AppError::bad_request(
                        "duplicate_file",
                        "Hanya satu file PDF yang boleh dikirim",
                    ));
                }

                let mut output = File::create(&input_path).await.map_err(|error| {
                    error!(%error, "Gagal membuat file PDF sementara");
                    AppError::internal(
                        "compress_tempfile_failed",
                        "Gagal menyiapkan file sementara untuk kompresi PDF",
                    )
                })?;

                let mut total_written = 0usize;
                while let Some(chunk) = field.chunk().await.map_err(|error| {
                    error!(%error, "Gagal membaca chunk PDF");
                    AppError::bad_request("invalid_upload", "Gagal membaca file PDF yang diunggah")
                })? {
                    total_written = total_written.checked_add(chunk.len()).ok_or_else(|| {
                        AppError::bad_request(
                            "upload_too_large",
                            "Ukuran file PDF melebihi batas maksimum (500 MB)",
                        )
                    })?;

                    if total_written > MAX_PDF_UPLOAD_BYTES {
                        return Err(AppError::bad_request(
                            "upload_too_large",
                            "Ukuran file PDF melebihi batas maksimum (500 MB)",
                        ));
                    }

                    output.write_all(&chunk).await.map_err(|error| {
                        error!(%error, "Gagal menulis chunk PDF sementara");
                        AppError::internal(
                            "compress_temp_write_failed",
                            "Gagal menyimpan file PDF sementara",
                        )
                    })?;
                }

                output.flush().await.map_err(|error| {
                    error!(%error, "Gagal flush file PDF sementara");
                    AppError::internal(
                        "compress_temp_flush_failed",
                        "Gagal menyimpan file PDF sementara",
                    )
                })?;

                if total_written == 0 {
                    return Err(AppError::bad_request("empty_file", "File PDF kosong"));
                }

                file = true;
            }
            "quality" => {
                let value = field.text().await.map_err(|error| {
                    error!(%error, "Gagal membaca parameter quality");
                    AppError::bad_request(
                        "invalid_quality",
                        "Parameter kualitas kompresi tidak valid",
                    )
                })?;

                if value.len() > MAX_QUALITY_FIELD_BYTES {
                    return Err(AppError::bad_request(
                        "invalid_quality",
                        "Parameter kualitas kompresi terlalu panjang",
                    ));
                }

                quality = CompressionQuality::from_str(value.trim()).ok_or_else(|| {
                    AppError::bad_request(
                        "invalid_quality",
                        "Kualitas kompresi harus high, medium, atau low",
                    )
                })?;
            }
            _ => {}
        }
    }

    if !file {
        return Err(AppError::bad_request(
            "file_missing",
            "Field file tidak ditemukan",
        ));
    }

    let mut header = [0_u8; PDF_HEADER_SCAN_BYTES];
    let mut input = File::open(&input_path).await.map_err(|error| {
        error!(%error, "Gagal membuka PDF sementara untuk validasi");
        AppError::internal(
            "compress_input_read_failed",
            "Gagal membaca file PDF sementara",
        )
    })?;
    let header_len = input.read(&mut header).await.map_err(|error| {
        error!(%error, "Gagal membaca header PDF sementara");
        AppError::internal(
            "compress_input_read_failed",
            "Gagal membaca file PDF sementara",
        )
    })?;

    validate_input(&header[..header_len], "PDF")
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

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        compress_pdf_from_path(&input_path, quality)
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
