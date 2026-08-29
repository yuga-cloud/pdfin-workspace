pub fn validate_pdf(pdf_bytes: &[u8]) -> Result<(), String> {
    if pdf_bytes.is_empty() {
        return Err("File PDF kosong".to_owned());
    }

    let header_window = pdf_bytes.len().min(1024);

    if pdf_bytes[..header_window]
        .windows(b"%PDF-".len())
        .any(|window| window == b"%PDF-")
    {
        Ok(())
    } else {
        Err("File bukan PDF yang valid".to_owned())
    }
}
