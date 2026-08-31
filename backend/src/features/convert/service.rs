use axum::{body::Bytes, extract::Multipart};
use tokio::task;
use tracing::error;

use crate::{
    error::{AppError, error_code},
    state::AppState,
};

const MAX_MULTIPART_FILES: usize = 50;

pub async fn read_single_file(mut multipart: Multipart) -> Result<Bytes, AppError> {
    loop {
        match multipart.next_field().await {
            Ok(Some(field)) if field.name() == Some("file") => {
                return match field.bytes().await {
                    Ok(bytes) if !bytes.is_empty() => Ok(bytes),

                    Ok(_) => Err(AppError::bad_request(
                        error_code::EMPTY_FILE,
                        "File yang dikirim kosong",
                    )),

                    Err(error) => {
                        error!(%error, "Gagal membaca uploaded file");

                        Err(AppError::bad_request(
                            error_code::INVALID_UPLOAD,
                            "Gagal membaca file yang diunggah",
                        ))
                    }
                };
            }

            Ok(Some(_)) => continue,

            Ok(None) => {
                return Err(AppError::bad_request(
                    error_code::FILE_MISSING,
                    "Field file tidak ditemukan",
                ));
            }

            Err(error) => {
                error!(%error, "Gagal membaca multipart request");

                return Err(AppError::bad_request(
                    error_code::INVALID_MULTIPART,
                    "Request multipart tidak valid",
                ));
            }
        }
    }
}

pub async fn read_multiple_files(mut multipart: Multipart) -> Result<Vec<Bytes>, AppError> {
    let mut files = Vec::new();

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) if field.name() == Some("file") => {
                if files.len() >= MAX_MULTIPART_FILES {
                    return Err(AppError::bad_request(
                        "too_many_files",
                        "Jumlah file dalam satu request terlalu banyak",
                    ));
                }

                match field.bytes().await {
                    Ok(bytes) if !bytes.is_empty() => files.push(bytes),

                    Ok(_) => {
                        return Err(AppError::bad_request(
                            error_code::EMPTY_FILE,
                            "Salah satu file yang dikirim kosong",
                        ));
                    }

                    Err(error) => {
                        error!(%error, "Gagal membaca uploaded file");

                        return Err(AppError::bad_request(
                            error_code::INVALID_UPLOAD,
                            "Gagal membaca file yang diunggah",
                        ));
                    }
                }
            }

            Ok(Some(_)) => continue,

            Ok(None) => break,

            Err(error) => {
                error!(%error, "Gagal membaca multipart request");

                return Err(AppError::bad_request(
                    error_code::INVALID_MULTIPART,
                    "Request multipart tidak valid",
                ));
            }
        }
    }

    if files.is_empty() {
        return Err(AppError::bad_request(
            error_code::FILE_MISSING,
            "Field file tidak ditemukan",
        ));
    }

    Ok(files)
}

pub async fn run_conversion_validated<V, F, T>(
    state: AppState,
    data: Bytes,
    validate: V,
    engine: F,
    operation: &'static str,
) -> Result<T, AppError>
where
    V: FnOnce(&[u8]) -> Result<(), AppError> + Send + 'static,
    F: FnOnce(&[u8]) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    validate(&data)?;
    run_conversion(state, data, engine, operation).await
}

pub async fn run_conversion_many<F, T>(
    state: AppState,
    data: Vec<Bytes>,
    engine: F,
    operation: &'static str,
) -> Result<T, AppError>
where
    F: FnOnce(&[&[u8]]) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let permit = state
        .pdf_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| {
            error!(%error, operation, "PDF semaphore tidak tersedia");

            AppError::service_unavailable(
                error_code::PDF_BUSY,
                "Server sedang terlalu sibuk memproses PDF",
            )
        })?;

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        let refs: Vec<&[u8]> = data.iter().map(Bytes::as_ref).collect();
        engine(&refs)
    })
    .await
    .map_err(|error| {
        error!(%error, operation, "Conversion worker mengalami panic");

        AppError::internal(
            error_code::CONVERSION_WORKER_FAILED,
            "Worker konversi mengalami kegagalan",
        )
    })?;

    result.map_err(|error| {
        error!(%error, operation, "Konversi gagal");

        AppError::internal(
            error_code::CONVERSION_FAILED,
            "Gagal memproses file. Silakan coba lagi.",
        )
    })
}

pub async fn run_conversion<F, T>(
    state: AppState,
    data: Bytes,
    engine: F,
    operation: &'static str,
) -> Result<T, AppError>
where
    F: FnOnce(&[u8]) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let permit = state
        .pdf_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| {
            error!(%error, operation, "PDF semaphore tidak tersedia");

            AppError::service_unavailable(
                error_code::PDF_BUSY,
                "Server sedang terlalu sibuk memproses PDF",
            )
        })?;

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        engine(&data)
    })
    .await
    .map_err(|error| {
        error!(%error, operation, "Conversion worker mengalami panic");

        AppError::internal(
            error_code::CONVERSION_WORKER_FAILED,
            "Worker konversi mengalami kegagalan",
        )
    })?;

    result.map_err(|error| {
        error!(%error, operation, "Konversi gagal");

        AppError::internal(
            error_code::CONVERSION_FAILED,
            "Gagal memproses file. Silakan coba lagi.",
        )
    })
}
