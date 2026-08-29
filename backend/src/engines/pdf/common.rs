pub fn validate_pdf(pdf_bytes: &[u8]) -> Result<(), String> {
    if pdf_bytes.is_empty() {
        return Err("File PDF kosong".to_owned());
    }

    Ok(())
}
