use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::office::excel_to_pdf::excel_to_pdf as excel_to_pdf_engine, error::AppError,
    state::AppState,
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

    let bytes = run_conversion(state, data, excel_to_pdf_engine, "Excel → PDF").await?;

    Ok(binary_response(PDF_CONTENT_TYPE, bytes))
}
