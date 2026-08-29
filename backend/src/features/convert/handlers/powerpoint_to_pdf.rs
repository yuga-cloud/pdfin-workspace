use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::office::powerpoint_to_pdf::powerpoint_to_pdf as powerpoint_to_pdf_engine,
    error::AppError, state::AppState,
};

use super::super::{
    response::binary_response,
    service::{read_single_file, run_conversion},
};

const PDF_CONTENT_TYPE: &str = "application/pdf";

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_single_file(multipart).await?;

    let output = run_conversion(state, data, powerpoint_to_pdf_engine, "PowerPoint → PDF").await?;

    Ok(binary_response(PDF_CONTENT_TYPE, output))
}
