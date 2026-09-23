use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PdfOperation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertRequest {
    pub file_id: Uuid,
    pub operation: PdfOperation,
}
