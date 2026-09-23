use axum::{Router, http::header, response::IntoResponse, routing::get};

use crate::state::AppState;

/// Spesifikasi OpenAPI resmi untuk API v1, di-embed dari `docs/openapi.yaml`
/// agar selalu sinkron dengan kode yang di-deploy.
const OPENAPI_YAML: &str = include_str!("../../../../docs/openapi.yaml");

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/health", get(health))
        .route("/openapi.yaml", get(openapi_yaml))
}

async fn health() -> &'static str {
    "ok"
}

async fn openapi_yaml() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        OPENAPI_YAML,
    )
}
