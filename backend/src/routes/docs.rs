use axum::{Json, Router, routing::get};
use utoipa::OpenApi;

use crate::{openapi::ApiDoc, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/openapi.json", get(openapi_json))
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
