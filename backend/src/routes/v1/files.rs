use axum::{Json, Router, routing::{get, post}};
use serde_json::json;

use crate::state::AppState;

async fn upload_files() -> Json<serde_json::Value> {
    Json(json!({
        "message": "upload endpoint ready",
        "status": "not_implemented"
    }))
}

async fn get_file() -> Json<serde_json::Value> {
    Json(json!({
        "message": "file lookup endpoint ready",
        "status": "not_implemented"
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/files", post(upload_files))
        .route("/files/:id", get(get_file))
}
