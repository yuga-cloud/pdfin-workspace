use axum::{Json, Router, routing::post};
use serde_json::json;

use crate::state::AppState;

async fn create_conversion() -> Json<serde_json::Value> {
    Json(json!({
        "message": "conversion endpoint ready",
        "status": "not_implemented"
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/conversions", post(create_conversion))
}
