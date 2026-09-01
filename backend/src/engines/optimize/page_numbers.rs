use super::common::{add_text_to_pages, validate_input};

const PAGE_NUMBER_FONT_SIZE: f64 = 10.0;

pub fn add_page_numbers(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    add_text_to_pages(
        pdf_bytes,
        |page_number| page_number.to_string(),
        PAGE_NUMBER_FONT_SIZE,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_pdf() {
        assert!(add_page_numbers(&[]).is_err());
    }
}
