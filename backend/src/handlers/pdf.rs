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
        pages::manage_pages as manage_pages_engine,
        rotate::rotate_pdf as rotate_pdf_engine,
    },
    error::AppError,
    state::AppState,
};

const DEFAULT_ROTATION_DEGREES: i64 = 90;
const PDF_CONTENT_TYPE: &str = "application/pdf";
const MAX_MERGE_FILES: usize = 32;
const MAX_TOTAL_MERGE_INPUT_BYTES: usize = 500 * 1024 * 1024;
const MAX_PAGE_ORDER_LENGTH: usize = 16 * 1024;

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