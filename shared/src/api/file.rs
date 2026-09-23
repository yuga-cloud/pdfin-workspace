use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Uuid,
    pub filename: String,
    pub size: u64,
    pub mime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub file: FileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResponse {
    pub file_id: Uuid,
    pub filename: String,
}
