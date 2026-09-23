use axum::{Json, Router, extract::Path, routing::get};
use shared::{ApiResponse, JobResponse, JobStatus};
use uuid::Uuid;

use crate::state::AppState;

async fn get_job(Path(id): Path<Uuid>) -> Json<ApiResponse<JobResponse>> {
    Json(ApiResponse::success(
        JobResponse {
            job_id: id,
            status: JobStatus::Queued,
            progress: 0,
        },
        Uuid::new_v4(),
    ))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/jobs/:id", get(get_job))
}
