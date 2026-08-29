use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::office::pdf_to_word::pdf_to_word as pdf_to_word_engine, error::AppError,
    state::AppState,
};

use super::super::{
    response::binary_response,
    service::{read_single_file, run_conversion},
};

const WORD_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_single_file(multipart).await?;

    let bytes = run_conversion(state, data, pdf_to_word_engine, "PDF → Word").await?;

    Ok(binary_response(WORD_CONTENT_TYPE, bytes))
}
