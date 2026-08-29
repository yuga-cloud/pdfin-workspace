pub fn validate_input(bytes: &[u8], format: &str) -> Result<(), String> {
    if bytes.is_empty() {
        return Err(format!("File {format} kosong"));
    }

    if format.eq_ignore_ascii_case("PDF") {
        validate_pdf_signature(bytes)?;
    }

    Ok(())
}

fn validate_pdf_signature(bytes: &[u8]) -> Result<(), String> {
    let header_window = bytes.len().min(1024);

    if bytes[..header_window]
        .windows(b"%PDF-".len())
        .any(|window| window == b"%PDF-")
    {
        Ok(())
    } else {
        Err("File bukan PDF yang valid".to_owned())
    }
}
