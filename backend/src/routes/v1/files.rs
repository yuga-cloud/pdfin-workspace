use axum::{extract::Path, Json, Router, routing::{get, post}};
use shared::{ApiResponse, FileMetadata, UploadResponse};
use uuid::Uuid;

use crate::state::AppState;

async fn upload_files() -> Json<ApiResponse<UploadResponse>> {
    Json(ApiResponse::success(
        UploadResponse {
            files: vec![FileMetadata {
                id: Uuid::new_v4(),
                filename: "placeholder.pdf".to_string(),
                size: 0,
                mime: "application/pdf".to_string(),
                status: shared::FileStatus::Uploaded,
            }],
        },
        Uuid::new_v4(),
    ))
}

async fn get_file(Path(id): Path<Uuid>) -> Json<ApiResponse<FileMetadata>> {
    Json(ApiResponse::success(
        FileMetadata {
            id,
            filename: "placeholder.pdf".to_string(),
            size: 0,
            mime: "application/pdf".to_string(),
            status: shared::FileStatus::Ready,
        },
        Uuid::new_v4(),
    ))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/files", post(upload_files))
        .route("/files/:id", get(get_file))
}
