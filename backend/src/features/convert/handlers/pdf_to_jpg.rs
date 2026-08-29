use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::rendering::pdf_to_jpg::pdf_to_jpg as pdf_to_jpg_engine, error::AppError,
    state::AppState,
};

use super::super::{
    response::binary_response,
    service::{read_single_file, run_conversion},
};

const JPEG_CONTENT_TYPE: &str = "image/jpeg";

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_single_file(multipart).await?;

    let images = run_conversion(state, data, pdf_to_jpg_engine, "PDF → JPG").await?;

    let first_image = images.into_iter().next().ok_or_else(|| {
        AppError::internal(
            "empty_conversion_result",
            "Konversi PDF ke JPG tidak menghasilkan gambar",
        )
    })?;

    Ok(binary_response(JPEG_CONTENT_TYPE, first_image))
}
