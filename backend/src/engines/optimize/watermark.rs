use super::common::validate_input;

pub fn add_watermark(pdf_bytes: &[u8], text: &str) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    if text.trim().is_empty() {
        return Err("Teks watermark tidak boleh kosong".to_owned());
    }

    Err("Engine watermark PDF belum diimplementasikan".to_owned())
}
