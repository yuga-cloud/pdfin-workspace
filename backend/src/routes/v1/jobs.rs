use axum::{extract::Path, Json, Router, routing::get};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
struct JobResponse {
    job_id: Uuid,
    status: &'static str,
}

async fn get_job(Path(id): Path<Uuid>) -> Json<JobResponse> {
    Json(JobResponse {
        job_id: id,
        status: "queued",
    })
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/jobs/:id", get(get_job))
}
