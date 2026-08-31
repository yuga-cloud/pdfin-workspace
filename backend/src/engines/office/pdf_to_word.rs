use std::io::Cursor;

use docx_rs::{BreakType, Docx, Paragraph, Run};

use crate::engines::{common::validate_input, office::pdf_table::extract_pdf_words};

const MAX_DOCX_OUTPUT_BYTES: usize = 128 * 1024 * 1024;
const LINE_BREAK_TOLERANCE: f32 = 10.0;

pub fn pdf_to_word(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    let pages = extract_pdf_words(pdf_bytes)?;

    if pages.is_empty() || pages.iter().all(Vec::is_empty) {
        return Err("PDF tidak memiliki text yang dapat dikonversi ke Word".to_owned());
    }

    let mut document = Docx::new();

    for (page_index, words) in pages.iter().enumerate() {
        let mut ordered = words.clone();
        ordered.sort_by(|left, right| {
            left.top
                .partial_cmp(&right.top)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    left.left
                        .partial_cmp(&right.left)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut paragraph = Paragraph::new();
        let mut previous_top: Option<f32> = None;
        let mut has_text = false;

        for word in ordered {
            if let Some(top) = previous_top
                && (word.top - top).abs() > LINE_BREAK_TOLERANCE
                && has_text
            {
                document = document.add_paragraph(paragraph);
                paragraph = Paragraph::new();
                has_text = false;
            }

            if has_text {
                paragraph = paragraph.add_run(Run::new().add_text(" "));
            }

            paragraph = paragraph.add_run(Run::new().add_text(word.text));
            previous_top = Some(word.top);
            has_text = true;
        }

        if has_text {
            document = document.add_paragraph(paragraph);
        }

        if page_index + 1 < pages.len() {
            document = document.add_paragraph(
                Paragraph::new().add_run(Run::new().add_break(BreakType::Page)),
            );
        }
    }

    let mut output = Cursor::new(Vec::new());
    document
        .build()
        .pack(&mut output)
        .map_err(|error| format!("Gagal membuat DOCX: {error}"))?;

    let bytes = output.into_inner();

    if bytes.is_empty() {
        return Err("DOCX hasil konversi kosong".to_owned());
    }

    if bytes.len() > MAX_DOCX_OUTPUT_BYTES {
        return Err(format!(
            "DOCX hasil konversi melebihi batas maksimum ({} MiB)",
            MAX_DOCX_OUTPUT_BYTES / 1024 / 1024
        ));
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_pdf() {
        assert!(pdf_to_word(&[]).is_err());
    }

    #[test]
    fn rejects_non_pdf_input() {
        assert!(pdf_to_word(b"not a pdf").is_err());
    }
}
