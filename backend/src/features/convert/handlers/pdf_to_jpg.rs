use axum::{
    extract::{Multipart, State},
    response::Response,
};

use crate::{
    engines::{common::validate_input, rendering::pdf_to_jpg::pdf_to_jpg as pdf_to_jpg_engine},
    error::{AppError, error_code},
    state::AppState,
    zip::{ZipEntry, create_stored_zip},
};

use super::super::{
    response::attachment_response,
    service::{read_single_file, run_conversion_validated},
};

const JPEG_CONTENT_TYPE: &str = "image/jpeg";
const ZIP_CONTENT_TYPE: &str = "application/zip";

fn validate_pdf_input(bytes: &[u8]) -> Result<(), AppError> {
    validate_input(bytes, "PDF")
        .map_err(|message| AppError::bad_request(error_code::INVALID_INPUT, message))
}

pub async fn handler(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = read_single_file(multipart).await?;
    let images = run_conversion_validated(
        state,
        data,
        validate_pdf_input,
        pdf_to_jpg_engine,
        "PDF → JPG",
    )
    .await?;

    if images.is_empty() {
        return Err(AppError::internal(
            "empty_conversion_result",
            "Konversi PDF ke JPG tidak menghasilkan gambar",
        ));
    }

    if images.len() == 1 {
        let image = images
            .into_iter()
            .next()
            .expect("single JPG result harus tersedia");
        return Ok(attachment_response(
            JPEG_CONTENT_TYPE,
            "page-001.jpg",
            image,
        ));
    }

    let names = (1..=images.len())
        .map(|number| format!("page-{number:03}.jpg"))
        .collect::<Vec<_>>();
    let entries = images
        .iter()
        .zip(names.iter())
        .map(|(data, name)| ZipEntry {
            name: name.as_str(),
            data: data.as_slice(),
        })
        .collect::<Vec<_>>();

    let archive = create_stored_zip(&entries).map_err(|error| {
        AppError::internal(
            "jpg_zip_failed",
            format!("Gagal membuat arsip JPG: {error}"),
        )
    })?;

    Ok(attachment_response(
        ZIP_CONTENT_TYPE,
        "pdf-pages.zip",
        archive,
    ))
}
