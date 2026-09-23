use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
struct CreateConversionRequest {
    file_id: Uuid,
    operation: String,
}

#[derive(Debug, Serialize)]
struct ConversionResponse {
    job_id: Uuid,
    status: &'static str,
}

async fn create_conversion(
    Json(payload): Json<CreateConversionRequest>,
) -> Json<ConversionResponse> {
    let _ = payload;

    Json(ConversionResponse {
        job_id: Uuid::new_v4(),
        status: "queued",
    })
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/conversions", post(create_conversion))
}
