use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::header,
    response::IntoResponse,
};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, task};
use tracing::error;

use crate::{
    engines::pdf::{
        common::validate_pdf_path,
        merge::merge_pdfs_from_paths as merge_pdf_engine,
        rotate::rotate_pdf_from_path as rotate_pdf_engine,
    },
    error::AppError,
    state::AppState,
};

const DEFAULT_ROTATION_DEGREES: i64 = 90;
const PDF_CONTENT_TYPE: &str = "application/pdf";
const MAX_MERGE_FILES: usize = 32;
const MAX_TOTAL_MERGE_INPUT_BYTES: usize = 500 * 1024 * 1024;

struct TempPdfUpload {
    file: NamedTempFile,
    size: usize,
}

pub async fn merge_pdfs(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let files = read_multiple_files_to_tempfiles(multipart).await?;

    if files.len() < 2 {
        return Err(AppError::bad_request(
            "insufficient_files",
            "Minimal dua file PDF diperlukan untuk digabungkan",
        ));
    }

    let total_input_bytes = files
        .iter()
        .try_fold(0usize, |total, file| total.checked_add(file.size))
        .ok_or_else(|| {
            AppError::bad_request("merge_input_too_large", "Total ukuran PDF terlalu besar")
        })?;

    if total_input_bytes > MAX_TOTAL_MERGE_INPUT_BYTES {
        return Err(AppError::bad_request(
            "merge_input_too_large",
            "Total ukuran PDF melebihi batas maksimum (500 MB)",
        ));
    }

    for (index, file) in files.iter().enumerate() {
        validate_pdf_path(file.file.path()).map_err(|message| {
            AppError::bad_request(
                "invalid_input",
                format!("PDF ke-{} tidak valid: {message}", index + 1),
            )
        })?;
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
        let paths: Vec<_> = files.iter().map(|file| file.file.path()).collect();
        merge_pdf_engine(&paths)
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

pub async fn rotate(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (file, degrees) = read_rotate_request(multipart).await?;
    validate_rotation_degrees(degrees)?;
    validate_pdf_path(file.file.path())
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
        rotate_pdf_engine(file.file.path(), degrees)
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

async fn read_multiple_files_to_tempfiles(
    mut multipart: Multipart,
) -> Result<Vec<TempPdfUpload>, AppError> {
    let mut files = Vec::new();

    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) if field.name() == Some("file") => {
                if files.len() >= MAX_MERGE_FILES {
                    return Err(AppError::bad_request(
                        "too_many_files",
                        "Jumlah file PDF dalam satu request terlalu banyak",
                    ));
                }

                let temp = NamedTempFile::new().map_err(|error| {
                    error!(%error, "Gagal membuat temporary file PDF");
                    AppError::internal("tempfile_failed", "Gagal menyiapkan penyimpanan sementara")
                })?;

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
                            "merge_input_too_large",
                            "Ukuran PDF melebihi kapasitas yang didukung",
                        )
                    })?;

                    if size > MAX_TOTAL_MERGE_INPUT_BYTES {
                        return Err(AppError::bad_request(
                            "merge_input_too_large",
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
                    return Err(AppError::bad_request(
                        "empty_file",
                        "Salah satu file PDF kosong",
                    ));
                }

                files.push(TempPdfUpload { file: temp, size });
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

async fn read_rotate_request(mut multipart: Multipart) -> Result<(TempPdfUpload, i64), AppError> {
    let mut file = None;
    let mut degrees = DEFAULT_ROTATION_DEGREES;

    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => match field.name().unwrap_or_default() {
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
                                "rotate_input_too_large",
                                "Ukuran PDF melebihi kapasitas yang didukung",
                            )
                        })?;

                        if size > MAX_TOTAL_MERGE_INPUT_BYTES {
                            return Err(AppError::bad_request(
                                "rotate_input_too_large",
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

    let file =
        file.ok_or_else(|| AppError::bad_request("file_missing", "Field file tidak ditemukan"))?;
    Ok((file, degrees))
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
