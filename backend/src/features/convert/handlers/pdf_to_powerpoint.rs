use axum::response::Response;

use crate::error::{AppError, error_code};

pub async fn handler() -> Result<Response, AppError> {
    Err(AppError::not_implemented(
        error_code::CONVERSION_NOT_IMPLEMENTED,
        "Konversi PDF ke PowerPoint belum tersedia.",
    ))
}
