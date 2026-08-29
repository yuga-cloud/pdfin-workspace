use crate::engines::common::validate_input;

pub fn pdf_to_word(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    Err("Engine PDF ke Word belum diimplementasikan".to_owned())
}
