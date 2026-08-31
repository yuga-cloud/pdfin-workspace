/// Re-export the centralized bounded PDF loader so optimization engines use
/// the same decompression budget as the core PDF engines.
pub use crate::engines::pdf::common::load_pdf_document;

pub fn validate_input(bytes: &[u8], format: &str) -> Result<(), String> {
    if bytes.is_empty() {
        return Err(format!("File {format} kosong"));
    }

    Ok(())
}
