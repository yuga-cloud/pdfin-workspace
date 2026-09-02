use std::path::Path;

use super::common::{add_text_to_pages_from_path, validate_watermark_text};

const WATERMARK_FONT_SIZE: f64 = 36.0;

pub fn add_watermark_from_path(pdf_path: &Path, text: &str) -> Result<Vec<u8>, String> {
    validate_watermark_text(text)?;

    let watermark = text.trim().to_owned();
    add_text_to_pages_from_path(pdf_path, |_| watermark.clone(), WATERMARK_FONT_SIZE, true)
}
