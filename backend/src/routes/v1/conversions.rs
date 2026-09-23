use axum::{Json, Router, routing::post};
use serde::Deserialize;
use shared::{ApiResponse, JobResponse, JobStatus};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
struct CreateConversionRequest {
    file_id: Uuid,
    operation: String,
}

async fn create_conversion(
    Json(payload): Json<CreateConversionRequest>,
) -> Json<ApiResponse<JobResponse>> {
    let _ = payload;

    Json(ApiResponse::success(
        JobResponse {
            job_id: Uuid::new_v4(),
            status: JobStatus::Queued,
            progress: 0,
        },
        Uuid::new_v4(),
    ))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/conversions", post(create_conversion))
}
