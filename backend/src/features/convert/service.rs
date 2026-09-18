use std::fs;

use axum::{extract::Multipart, extract::multipart::Field};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, task};
use tracing::error;

use crate::{
    error::{AppError, error_code},
    state::AppState,
};

const MAX_MULTIPART_FILES: usize = 50;
const MAX_MULTIPART_FIELDS: usize = 64;
const MAX_CONVERSION_UPLOAD_BYTES: usize = 500 * 1024 * 1024;
const MAX_MULTIPART_TOTAL_BYTES: usize = 1024 * 1024 * 1024;

pub struct TempUpload {
    file: NamedTempFile,
}

impl TempUpload {
    fn path(&self) -> &std::path::Path {
        self.file.path()
    }
}

pub async fn read_single_file(mut multipart: Multipart) -> Result<TempUpload, AppError> {
    let mut field_count = 0usize;

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                field_count += 1;

                if field_count > MAX_MULTIPART_FIELDS {
                    return Err(AppError::bad_request(
                        "too_many_fields",
                        "Jumlah field multipart dalam satu request terlalu banyak",
                    ));
                }

                if field.name() == Some("file") {
                    return stream_field(field, None).await;
                }
            }

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

pub async fn read_multiple_files(mut multipart: Multipart) -> Result<Vec<TempUpload>, AppError> {
    let mut files = Vec::new();
    let mut total_size = 0usize;
    let mut field_count = 0usize;

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                field_count += 1;

                if field_count > MAX_MULTIPART_FIELDS {
                    return Err(AppError::bad_request(
                        "too_many_fields",
                        "Jumlah field multipart dalam satu request terlalu banyak",
                    ));
                }

                if field.name() != Some("file") {
                    continue;
                }

                if files.len() >= MAX_MULTIPART_FILES {
                    return Err(AppError::bad_request(
                        "too_many_files",
                        "Jumlah file dalam satu request terlalu banyak",
                    ));
                }

                let file = stream_field(field, Some(&mut total_size)).await?;
                files.push(file);
            }

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

async fn stream_field(
    mut field: Field<'_>,
    mut total_size: Option<&mut usize>,
) -> Result<TempUpload, AppError> {
    let temp = NamedTempFile::new().map_err(|error| {
        error!(%error, "Gagal membuat temporary file conversion");
        AppError::internal("tempfile_failed", "Gagal menyiapkan penyimpanan sementara")
    })?;

    let std_file = temp.reopen().map_err(|error| {
        error!(%error, "Gagal membuka temporary file conversion");
        AppError::internal("tempfile_failed", "Gagal membuka penyimpanan sementara")
    })?;
    let mut output = tokio::fs::File::from_std(std_file);
    let mut file_size = 0usize;

    while let Some(chunk) = field.chunk().await.map_err(|error| {
        error!(%error, "Gagal membaca uploaded file");
        AppError::bad_request(
            error_code::INVALID_UPLOAD,
            "Gagal membaca file yang diunggah",
        )
    })? {
        file_size = file_size.checked_add(chunk.len()).ok_or_else(|| {
            AppError::bad_request(
                "upload_too_large",
                "Ukuran file melebihi batas maksimum (500 MB)",
            )
        })?;

        if file_size > MAX_CONVERSION_UPLOAD_BYTES {
            return Err(AppError::bad_request(
                "upload_too_large",
                "Ukuran file melebihi batas maksimum (500 MB)",
            ));
        }

        if let Some(total) = total_size.as_deref_mut() {
            *total = total.checked_add(chunk.len()).ok_or_else(|| {
                AppError::bad_request(
                    "request_too_large",
                    "Ukuran total upload melebihi batas maksimum",
                )
            })?;

            if *total > MAX_MULTIPART_TOTAL_BYTES {
                return Err(AppError::bad_request(
                    "request_too_large",
                    "Ukuran total upload melebihi batas maksimum (1 GiB)",
                ));
            }
        }

        output.write_all(&chunk).await.map_err(|error| {
            error!(%error, "Gagal menulis temporary conversion file");
            AppError::internal("tempfile_write_failed", "Gagal menyimpan file sementara")
        })?;
    }

    output.flush().await.map_err(|error| {
        error!(%error, "Gagal flush temporary conversion file");
        AppError::internal("tempfile_write_failed", "Gagal menyimpan file sementara")
    })?;

    if file_size == 0 {
        return Err(AppError::bad_request(
            error_code::EMPTY_FILE,
            "File yang dikirim kosong",
        ));
    }

    Ok(TempUpload { file: temp })
}

fn read_temp_file(upload: TempUpload) -> Result<Vec<u8>, String> {
    fs::read(upload.path()).map_err(|error| format!("Gagal membaca file sementara: {error}"))
}

async fn acquire_conversion_permit(
    state: &AppState,
    operation: &'static str,
) -> Result<tokio::sync::OwnedSemaphorePermit, AppError> {
    state
        .conversion_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| {
            error!(%error, operation, "Conversion semaphore tidak tersedia");

            AppError::service_unavailable(
                error_code::PDF_BUSY,
                "Server sedang terlalu sibuk memproses konversi",
            )
        })
}

pub async fn run_conversion_validated<V, F, T>(
    state: AppState,
    data: TempUpload,
    validate: V,
    engine: F,
    operation: &'static str,
) -> Result<T, AppError>
where
    V: FnOnce(&[u8]) -> Result<(), AppError> + Send + 'static,
    F: FnOnce(&[u8]) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let permit = acquire_conversion_permit(&state, operation).await?;

    task::spawn_blocking(move || {
        let _permit = permit;
        let data = read_temp_file(data).map_err(|error| {
            error!(%error, operation, "Gagal membaca file sementara");
            AppError::internal(error_code::INVALID_UPLOAD, "Gagal membaca file sementara")
        })?;

        validate(&data)?;

        engine(&data).map_err(|error| {
            error!(%error, operation, "Konversi gagal");
            AppError::internal(
                error_code::CONVERSION_FAILED,
                "Gagal memproses file. Silakan coba lagi.",
            )
        })
    })
    .await
    .map_err(|error| {
        error!(%error, operation, "Conversion worker mengalami panic");

        AppError::internal(
            error_code::CONVERSION_WORKER_FAILED,
            "Worker konversi mengalami kegagalan",
        )
    })?
}

pub async fn run_conversion_many<F, T>(
    state: AppState,
    data: Vec<TempUpload>,
    engine: F,
    operation: &'static str,
) -> Result<T, AppError>
where
    F: FnOnce(&[&[u8]]) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let permit = acquire_conversion_permit(&state, operation).await?;

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        let buffers = data
            .into_iter()
            .map(read_temp_file)
            .collect::<Result<Vec<_>, _>>()?;
        let refs: Vec<&[u8]> = buffers.iter().map(Vec::as_slice).collect();
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
    data: TempUpload,
    engine: F,
    operation: &'static str,
) -> Result<T, AppError>
where
    F: FnOnce(&[u8]) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let permit = acquire_conversion_permit(&state, operation).await?;

    let result = task::spawn_blocking(move || {
        let _permit = permit;
        let data = read_temp_file(data)?;
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
