use super::common::{add_text_to_pages, validate_input, validate_watermark_text};

const WATERMARK_FONT_SIZE: f64 = 36.0;

pub fn add_watermark(pdf_bytes: &[u8], text: &str) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;
    validate_watermark_text(text)?;

    let watermark = text.trim().to_owned();
    add_text_to_pages(
        pdf_bytes,
        |_| watermark.clone(),
        WATERMARK_FONT_SIZE,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_watermark() {
        assert!(add_watermark(b"%PDF-1.7", "   ").is_err());
    }

    #[test]
    fn rejects_non_ascii_watermark() {
        assert!(add_watermark(b"%PDF-1.7", "é").is_err());
    }
}
