use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::image::jpg_to_pdf::jpgs_to_pdf as jpgs_to_pdf_engine, error::AppError, state::AppState,
};

use super::super::{
    response::binary_response,
    service::{read_multiple_files, run_conversion_many},
};

const PDF_CONTENT_TYPE: &str = "application/pdf";

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_multiple_files(multipart).await?;

    let output =
        run_conversion_many(state, data, jpgs_to_pdf_engine, "JPG → PDF").await?;

    Ok(binary_response(PDF_CONTENT_TYPE, output))
}
