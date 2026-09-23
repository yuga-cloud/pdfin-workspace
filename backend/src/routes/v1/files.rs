use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use shared::{ApiResponse, FileMetadata, UploadResponse};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

async fn upload_files() -> Result<Json<ApiResponse<UploadResponse>>, AppError> {
    Ok(Json(ApiResponse::success(
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
    )))
}

async fn get_file(Path(id): Path<Uuid>) -> Result<Json<ApiResponse<FileMetadata>>, AppError> {
    if id.is_nil() {
        return Err(AppError::bad_request(
            "INVALID_FILE_ID",
            "File id tidak valid",
        ));
    }

    Ok(Json(ApiResponse::success(
        FileMetadata {
            id,
            filename: "placeholder.pdf".to_string(),
            size: 0,
            mime: "application/pdf".to_string(),
            status: shared::FileStatus::Ready,
        },
        Uuid::new_v4(),
    )))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/files", post(upload_files))
        .route("/files/:id", get(get_file))
}
