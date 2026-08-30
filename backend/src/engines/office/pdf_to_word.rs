use std::io::Cursor;

use docx_rs::{BreakType, Docx, Paragraph, Run};

use crate::engines::{common::validate_input, office::pdf_table::extract_pdf_words};

pub fn pdf_to_word(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    let pages = extract_pdf_words(pdf_bytes)?;

    if pages.is_empty() || pages.iter().all(Vec::is_empty) {
        return Err("PDF tidak memiliki text yang dapat dikonversi ke Word".to_owned());
    }

    let mut document = Docx::new();

    for (page_index, words) in pages.iter().enumerate() {
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

        let mut paragraph = Paragraph::new();
        let mut previous_top: Option<f32> = None;
        let mut has_text = false;

        for word in ordered {
            if let Some(top) = previous_top {
                if (word.top - top).abs() > 10.0 && has_text {
                    document = document.add_paragraph(paragraph);
                    paragraph = Paragraph::new();
                    has_text = false;
                }
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
            document = document
                .add_paragraph(Paragraph::new().add_run(Run::new().add_break(BreakType::Page)));
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
