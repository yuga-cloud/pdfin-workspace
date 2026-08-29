use super::gotenberg::convert_to_pdf;
use crate::engines::common::validate_input;

pub fn powerpoint_to_pdf(document_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(document_bytes, "PowerPoint")?;

    tracing::info!("Memulai konversi PowerPoint → PDF");

    convert_to_pdf(document_bytes, "pptx", "PowerPoint")
}
