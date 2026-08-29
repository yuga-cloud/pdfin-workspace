use crate::engines::common::validate_input;

pub fn pdf_to_powerpoint(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    Err("Engine PDF ke PowerPoint belum diimplementasikan".to_owned())
}
