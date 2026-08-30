use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    error::{error_code, AppError},
    state::AppState,
};

use super::super::service::read_single_file;

pub async fn handler(
    State(_state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let _ = read_single_file(multipart).await?;

    Err(AppError::not_implemented(
        error_code::CONVERSION_NOT_IMPLEMENTED,
        "Konversi PDF ke PowerPoint belum tersedia.",
    ))
}
