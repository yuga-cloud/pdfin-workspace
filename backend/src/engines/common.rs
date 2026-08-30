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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_input() {
        assert!(validate_input(&[], "PDF").is_err());
    }

    #[test]
    fn accepts_pdf_signature_with_leading_bytes() {
        assert!(validate_input(b"%PDF-1.7\n...", "PDF").is_ok());
        assert!(validate_input(b"junk\n%PDF-1.7\n...", "PDF").is_ok());
    }

    #[test]
    fn rejects_non_pdf_with_pdf_format() {
        assert!(validate_input(b"PK\x03\x04", "PDF").is_err());
    }

    #[test]
    fn non_pdf_formats_only_require_non_empty_input() {
        assert!(validate_input(b"anything", "Excel").is_ok());
    }
}
