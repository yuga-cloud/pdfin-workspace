use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PdfOperation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertRequest {
    pub file_ids: Vec<Uuid>,
    pub operation: PdfOperation,
}
