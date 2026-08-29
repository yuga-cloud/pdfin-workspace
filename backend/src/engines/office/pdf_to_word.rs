use docx_rs::{Docx, Paragraph, Run};
use std::io::Cursor;

use crate::engines::{
    common::validate_input,
    office::pdf_table::extractor::extract_pdf_words,
};

pub fn pdf_to_word(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    let pages = extract_pdf_words(pdf_bytes)?;

    if pages.is_empty() || pages.iter().all(Vec::is_empty) {
        return Err("PDF tidak memiliki text yang dapat dikonversi ke Word".to_owned());
    }

    let mut document = Docx::new();

    for (page_index, words) in pages.iter().enumerate() {
        let mut paragraph = Paragraph::new();

        let mut ordered = words.clone();
        ordered.sort_by(|a, b| {
            a.top
                .partial_cmp(&b.top)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.left
                        .partial_cmp(&b.left)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut previous_top: Option<f32> = None;

        for word in ordered {
            if let Some(top) = previous_top {
                let line_gap = (word.top - top).abs();
                if line_gap > 10.0 {
                    paragraph = paragraph.add_run(Run::new().add_break(docx_rs::BreakType::Line));
                }
            }

            if !paragraph.children.is_empty() {
                paragraph = paragraph.add_run(Run::new().add_text(" "));
            }

            paragraph = paragraph.add_run(Run::new().add_text(word.text));
            previous_top = Some(word.top);
        }

        if !paragraph.children.is_empty() {
            document = document.add_paragraph(paragraph);
        }

        if page_index + 1 < pages.len() {
            document = document.add_paragraph(
                Paragraph::new().add_run(Run::new().add_break(docx_rs::BreakType::Page)),
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

    Ok(bytes)
}
