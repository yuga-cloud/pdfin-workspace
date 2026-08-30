use axum::{body::Bytes, http::header, response::Response};

pub fn binary_response(content_type: &'static str, bytes: Vec<u8>) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Bytes::from(bytes).into())
        .expect("binary response harus valid")
}

pub fn attachment_response(
    content_type: &'static str,
    filename: &str,
    bytes: Vec<u8>,
) -> Response {
    let disposition = format!("attachment; filename=\"{filename}\"");

    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(Bytes::from(bytes).into())
        .expect("attachment response harus valid")
}
