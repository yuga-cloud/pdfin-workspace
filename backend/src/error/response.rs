use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use shared::ApiError;

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: ApiError,
}

pub fn json_error_response(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: ApiError::new(code, message),
        }),
    )
        .into_response()
}
