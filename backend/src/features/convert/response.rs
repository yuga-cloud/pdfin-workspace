use axum::{body::Bytes, http::header, response::Response};

pub fn binary_response(content_type: &'static str, bytes: Vec<u8>) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Bytes::from(bytes).into())
        .expect("binary response harus valid")
}
