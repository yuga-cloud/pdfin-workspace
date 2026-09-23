use axum::{extract::Path, Json, Router, routing::{get, post}};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
struct FileResponse {
    id: Uuid,
    filename: String,
    size: u64,
}

async fn upload_files() -> Json<FileResponse> {
    Json(FileResponse {
        id: Uuid::new_v4(),
        filename: "placeholder.pdf".to_string(),
        size: 0,
    })
}

async fn get_file(Path(id): Path<Uuid>) -> Json<FileResponse> {
    Json(FileResponse {
        id,
        filename: "placeholder.pdf".to_string(),
        size: 0,
    })
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/files", post(upload_files))
        .route("/files/:id", get(get_file))
}
