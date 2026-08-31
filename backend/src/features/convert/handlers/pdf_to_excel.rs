use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::{common::validate_input, office::pdf_to_excel::pdf_to_excel as pdf_to_excel_engine},
    error::{AppError, error_code},
    state::AppState,
};

use super::super::{
    response::binary_response,
    service::{read_single_file, run_conversion_validated},
};

const EXCEL_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

fn validate_pdf_input(bytes: &[u8]) -> Result<(), AppError> {
    validate_input(bytes, "PDF")
        .map_err(|message| AppError::bad_request(error_code::INVALID_INPUT, message))
}

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_single_file(multipart).await?;

    let bytes = run_conversion_validated(
        state,
        data,
        validate_pdf_input,
        pdf_to_excel_engine,
        "PDF → Excel",
    )
    .await?;

    Ok(binary_response(EXCEL_CONTENT_TYPE, bytes))
}
