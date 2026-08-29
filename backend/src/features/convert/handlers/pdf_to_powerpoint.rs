use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::office::pdf_to_powerpoint::pdf_to_powerpoint as pdf_to_powerpoint_engine,
    error::AppError, state::AppState,
};

use super::super::{
    response::binary_response,
    service::{read_single_file, run_conversion},
};

const POWERPOINT_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presentation";

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_single_file(multipart).await?;

    let output = run_conversion(state, data, pdf_to_powerpoint_engine, "PDF → PowerPoint").await?;

    Ok(binary_response(POWERPOINT_CONTENT_TYPE, output))
}
