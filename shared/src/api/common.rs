use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub request_id: Uuid,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, request_id: Uuid) -> Self {
        Self {
            data: Some(data),
            error: None,
            request_id,
        }
    }

    pub fn failure(error: ApiError, request_id: Uuid) -> Self {
        Self {
            data: None,
            error: Some(error),
            request_id,
        }
    }
}
