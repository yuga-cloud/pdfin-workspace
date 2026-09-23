use axum::{Json, Router, routing::get};
use serde_json::json;

use crate::state::AppState;

async fn get_job() -> Json<serde_json::Value> {
    Json(json!({
        "message": "job endpoint ready",
        "status": "not_implemented"
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/jobs/:id", get(get_job))
}
