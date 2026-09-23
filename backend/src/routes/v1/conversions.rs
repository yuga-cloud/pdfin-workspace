use axum::{Json, Router, routing::post};
use serde::Deserialize;
use shared::{ApiResponse, JobResponse, JobStatus};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
struct CreateConversionRequest {
    file_id: Uuid,
    operation: String,
}

async fn create_conversion(
    Json(payload): Json<CreateConversionRequest>,
) -> Result<Json<ApiResponse<JobResponse>>, AppError> {
    if payload.operation.trim().is_empty() {
        return Err(AppError::bad_request(
            "INVALID_OPERATION",
            "Operation conversion wajib diisi",
        ));
    }

    let _file_id = payload.file_id;

    Ok(Json(ApiResponse::success(
        JobResponse {
            job_id: Uuid::new_v4(),
            status: JobStatus::Queued,
            progress: 0,
        },
        Uuid::new_v4(),
    )))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/conversions", post(create_conversion))
}
