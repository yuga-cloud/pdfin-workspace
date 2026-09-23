use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::response::json_error_response;

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn bad_request(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn too_many_requests(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn internal(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn service_unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: code.into(),
            message: message.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        json_error_response(self.status, self.code, self.message)
    }
}
