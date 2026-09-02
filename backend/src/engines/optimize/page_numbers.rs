use std::path::Path;

use super::common::add_text_to_pages_from_path;

const PAGE_NUMBER_FONT_SIZE: f64 = 10.0;

pub fn add_page_numbers_from_path(pdf_path: &Path) -> Result<Vec<u8>, String> {
    add_text_to_pages_from_path(
        pdf_path,
        |page_number| page_number.to_string(),
        PAGE_NUMBER_FONT_SIZE,
        true,
    )
}
