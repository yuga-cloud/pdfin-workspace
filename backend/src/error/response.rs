use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

pub fn json_error_response(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: ErrorBody {
                code: code.into(),
                message: message.into(),
            },
        }),
    )
        .into_response()
}
