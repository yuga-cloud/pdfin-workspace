use std::collections::HashSet;

use lopdf::{
    content::{Content, Operation},
    Dictionary, Document, Object, ObjectId,
};

use crate::engines::pdf::common::{load_pdf_document, validate_pdf};

const OVERLAY_FONT_NAME: &[u8] = b"PDFinOverlayFont";
const PAGE_MARGIN: f64 = 24.0;
const MAX_WATERMARK_TEXT_BYTES: usize = 1024;

pub fn validate_input(bytes: &[u8], format: &str) -> Result<(), String> {
    if bytes.is_empty() {
        return Err(format!("File {format} kosong"));
    }

    Ok(())
}

pub fn validate_watermark_text(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Teks watermark tidak boleh kosong".to_owned());
    }

    if text.len() > MAX_WATERMARK_TEXT_BYTES {
        return Err(format!(
            "Teks watermark melebihi batas maksimum ({} byte)",
            MAX_WATERMARK_TEXT_BYTES
        ));
    }

    if !text.is_ascii() {
        return Err("Teks watermark saat ini hanya mendukung karakter ASCII".to_owned());
    }

    Ok(())
}

pub fn add_text_to_pages(
    pdf_bytes: &[u8],
    text_for_page: impl Fn(u32) -> String,
    font_size: f64,
    center: bool,
) -> Result<Vec<u8>, String> {
    validate_pdf(pdf_bytes)?;

    let mut document = load_pdf_document(pdf_bytes)?;
    let pages = document.get_pages();

    if pages.is_empty() {
        return Err("PDF tidak memiliki halaman".to_owned());
    }

    let font_id = {
        let mut font = Dictionary::new();
        font.set("Type", "Font");
        font.set("Subtype", "Type1");
        font.set("BaseFont", "Helvetica");
        font.set("Encoding", "WinAnsiEncoding");
        document.add_object(font)
    };

    let page_count = pages.len();
    for (page_number, page_id) in pages {
        let text = text_for_page(page_number);
        if text.is_empty() {
            continue;
        }

        let (min_x, min_y, width, height) = page_bounds(&document, page_id)?;
        let x = if center {
            centered_x(min_x, width, &text, font_size)
        } else {
            min_x + PAGE_MARGIN
        };
        let y = if center {
            min_y + (height - font_size) / 2.0
        } else {
            min_y + PAGE_MARGIN
        };

        let font_name = install_font(&mut document, page_id, font_id)?;
        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(font_name), font_size.into()]),
                Operation::new("Td", vec![x.into(), y.into()]),
                Operation::new("Tj", vec![Object::string_literal(text)]),
                Operation::new("ET", vec![]),
                Operation::new("Q", vec![]),
            ],
        };

        document
            .add_to_page_content(page_id, content)
            .map_err(|error| format!("Gagal menambahkan overlay ke halaman {page_number}: {error}"))?;
    }

    let mut output = Vec::with_capacity(
        pdf_bytes
            .len()
            .saturating_add(page_count.saturating_mul(128)),
    );
    document
        .save_to(&mut output)
        .map_err(|error| format!("Gagal menyimpan PDF hasil overlay: {error}"))?;

    Ok(output)
}

fn install_font(
    doc: &mut Document,
    page_id: ObjectId,
    font_id: ObjectId,
) -> Result<Vec<u8>, String> {
    let mut resources = inherited_resources(doc, page_id)?.unwrap_or_default();

    let mut fonts = match resources.get(b"Font") {
        Some(Object::Reference(fonts_id)) => doc
            .get_dictionary(*fonts_id)
            .map_err(|error| format!("Gagal membuka dictionary font: {error}"))?
            .clone(),
        Some(Object::Dictionary(fonts)) => fonts.clone(),
        Some(other) => {
            return Err(format!(
                "Dictionary Font halaman tidak valid: {}",
                other.enum_variant()
            ));
        }
        None => Dictionary::new(),
    };

    let name = unique_font_name(&fonts);
    fonts.set(name.clone(), font_id);
    resources.set("Font", fonts);

    doc.get_dictionary_mut(page_id)
        .map_err(|error| format!("Gagal membuka dictionary halaman: {error}"))?
        .set("Resources", resources);

    Ok(name)
}

fn inherited_resources(doc: &Document, page_id: ObjectId) -> Result<Option<Dictionary>, String> {
    let mut current = page_id;
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(current) {
            return Err("Page tree mengandung reference cycle".to_owned());
        }

        let page = doc
            .get_dictionary(current)
            .map_err(|error| format!("Gagal membaca dictionary halaman: {error}"))?;

        if let Ok(resources) = page.get_deref(b"Resources", doc) {
            return resources
                .as_dict()
                .map(|resources| Some(resources.clone()))
                .map_err(|error| format!("Resources halaman bukan dictionary: {error}"));
        }

        current = match page.get(b"Parent").and_then(Object::as_reference) {
            Ok(parent) => parent,
            Err(_) => return Ok(None),
        };
    }
}

fn unique_font_name(fonts: &Dictionary) -> Vec<u8> {
    if !fonts.has(OVERLAY_FONT_NAME) {
        return OVERLAY_FONT_NAME.to_vec();
    }

    for index in 2..=1024 {
        let candidate = format!("PDFinOverlayFont{index}").into_bytes();
        if !fonts.has(&candidate) {
            return candidate;
        }
    }

    format!("PDFinOverlayFont{}", fonts.len() + 1).into_bytes()
}

fn page_bounds(doc: &Document, page_id: ObjectId) -> Result<(f64, f64, f64, f64), String> {
    let mut current = page_id;
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(current) {
            return Err("Page tree mengandung reference cycle".to_owned());
        }

        let page = doc
            .get_dictionary(current)
            .map_err(|error| format!("Gagal membaca dictionary halaman: {error}"))?;

        if let Ok(media_box) = page.get_deref(b"MediaBox", doc) {
            let values = media_box
                .as_array()
                .map_err(|error| format!("MediaBox bukan array: {error}"))?;
            if values.len() != 4 {
                return Err("MediaBox PDF harus memiliki 4 nilai".to_owned());
            }

            let x0 = f64::from(
                values[0]
                    .as_float()
                    .map_err(|error| format!("MediaBox X0 tidak valid: {error}"))?,
            );
            let y0 = f64::from(
                values[1]
                    .as_float()
                    .map_err(|error| format!("MediaBox Y0 tidak valid: {error}"))?,
            );
            let x1 = f64::from(
                values[2]
                    .as_float()
                    .map_err(|error| format!("MediaBox X1 tidak valid: {error}"))?,
            );
            let y1 = f64::from(
                values[3]
                    .as_float()
                    .map_err(|error| format!("MediaBox Y1 tidak valid: {error}"))?,
            );

            let min_x = x0.min(x1);
            let min_y = y0.min(y1);
            let width = (x1 - x0).abs();
            let height = (y1 - y0).abs();

            if width <= 0.0 || height <= 0.0 {
                return Err("MediaBox PDF memiliki ukuran tidak valid".to_owned());
            }

            return Ok((min_x, min_y, width, height));
        }

        current = page
            .get(b"Parent")
            .and_then(Object::as_reference)
            .map_err(|_| "PDF halaman tidak memiliki MediaBox".to_owned())?;
    }
}

fn centered_x(min_x: f64, width: f64, text: &str, font_size: f64) -> f64 {
    let estimated_width = text.chars().count() as f64 * font_size * 0.5;
    min_x + ((width - estimated_width) / 2.0).max(PAGE_MARGIN)
}
