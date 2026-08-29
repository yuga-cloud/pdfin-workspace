use super::gotenberg::convert_to_pdf;
use crate::engines::common::validate_input;

pub fn word_to_pdf(document_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(document_bytes, "Word")?;

    tracing::info!("Memulai konversi Word → PDF");

    convert_to_pdf(document_bytes, "docx", "Word")
}
