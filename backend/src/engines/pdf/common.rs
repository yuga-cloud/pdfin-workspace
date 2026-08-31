const MAX_PDF_INPUT_BYTES: usize = 500 * 1024 * 1024;
const PDF_HEADER_SCAN_BYTES: usize = 1024;

pub fn validate_pdf(pdf_bytes: &[u8]) -> Result<(), String> {
    if pdf_bytes.is_empty() {
        return Err("File PDF kosong".to_owned());
    }

    if pdf_bytes.len() > MAX_PDF_INPUT_BYTES {
        return Err(format!(
            "Ukuran PDF melebihi batas maksimum ({} MiB)",
            MAX_PDF_INPUT_BYTES / 1024 / 1024
        ));
    }

    validate_pdf_signature(pdf_bytes)
}

fn validate_pdf_signature(bytes: &[u8]) -> Result<(), String> {
    let header_window = bytes.len().min(PDF_HEADER_SCAN_BYTES);

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
        assert!(validate_pdf(&[]).is_err());
    }

    #[test]
    fn accepts_pdf_signature_with_leading_bytes() {
        assert!(validate_pdf(b"%PDF-1.7\n...").is_ok());
        assert!(validate_pdf(b"junk\n%PDF-1.7\n...").is_ok());
    }

    #[test]
    fn rejects_non_pdf_input() {
        assert!(validate_pdf(b"PK\x03\x04").is_err());
        assert!(validate_pdf(b"not a pdf").is_err());
    }

    #[test]
    fn rejects_pdf_signature_outside_scan_window() {
        let mut bytes = vec![b'x'; PDF_HEADER_SCAN_BYTES + b"%PDF-".len()];
        bytes.extend_from_slice(b"%PDF-1.7");
        assert!(validate_pdf(&bytes).is_err());
    }

    #[test]
    fn rejects_oversized_pdf_input() {
        let mut bytes = vec![b'x'; MAX_PDF_INPUT_BYTES + 1];
        bytes[..5].copy_from_slice(b"%PDF-");
        assert!(validate_pdf(&bytes).is_err());
    }
}
