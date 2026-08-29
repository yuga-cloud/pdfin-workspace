use super::common::validate_input;

pub fn add_page_numbers(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    Err("Engine nomor halaman PDF belum diimplementasikan".to_owned())
}
