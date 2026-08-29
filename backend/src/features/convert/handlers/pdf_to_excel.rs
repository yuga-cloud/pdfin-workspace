use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::office::pdf_to_excel::pdf_to_excel as pdf_to_excel_engine, error::AppError,
    state::AppState,
};

use super::super::{
    response::binary_response,
    service::{read_single_file, run_conversion},
};

const EXCEL_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_single_file(multipart).await?;

    let bytes = run_conversion(state, data, pdf_to_excel_engine, "PDF → Excel").await?;

    Ok(binary_response(EXCEL_CONTENT_TYPE, bytes))
}
