use std::collections::HashSet;

use lopdf::{Document, Object, ObjectId};

use super::common::{load_pdf_document, validate_pdf};

const MAX_OUTPUT_BYTES: usize = 1024 * 1024 * 1024;

/// Memutar seluruh halaman PDF.
///
/// `extra_deg` harus berupa kelipatan 90 derajat.
///
/// Nilai yang umum:
/// - 90
/// - 180
/// - 270
///
/// Nilai negatif seperti -90 juga diperbolehkan
/// dan akan dinormalisasi menjadi 270 derajat.
pub fn rotate_pdf(pdf_bytes: &[u8], extra_deg: i64) -> Result<Vec<u8>, String> {
    validate_pdf(pdf_bytes)?;

    if extra_deg % 90 != 0 {
        return Err("Sudut rotasi PDF harus merupakan kelipatan 90 derajat.".to_owned());
    }

    let mut document = load_pdf_document(pdf_bytes)?;

    let page_ids = document.get_pages().values().copied().collect::<Vec<_>>();

    if page_ids.is_empty() {
        return Err("PDF tidak memiliki halaman.".to_owned());
    }

    for page_id in page_ids {
        let current_rotation = inherited_rotation(&document, page_id)?;
        let rotation = (current_rotation + extra_deg).rem_euclid(360);

        let object = document
            .get_object_mut(page_id)
            .map_err(|error| format!("Gagal mengakses halaman PDF: {error}"))?;

        let dictionary = object
            .as_dict_mut()
            .map_err(|error| format!("Objek halaman PDF tidak valid: {error}"))?;

        dictionary.set("Rotate", Object::Integer(rotation));
    }

    let mut output = Vec::with_capacity(pdf_bytes.len());

    document
        .save_to(&mut output)
        .map_err(|error| format!("Gagal menyimpan PDF hasil rotasi: {error}"))?;

    if output.is_empty() {
        return Err("PDF hasil rotasi kosong.".to_owned());
    }

    if output.len() > MAX_OUTPUT_BYTES {
        return Err(format!(
            "PDF hasil rotasi melebihi batas maksimum ({} MB)",
            MAX_OUTPUT_BYTES / 1024 / 1024
        ));
    }

    Ok(output)
}

fn inherited_rotation(document: &Document, page_id: ObjectId) -> Result<i64, String> {
    let mut current = page_id;
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(current) {
            return Err("Page tree mengandung reference cycle".to_owned());
        }

        let dictionary = document
            .get_dictionary(current)
            .map_err(|error| format!("Gagal membaca dictionary halaman PDF: {error}"))?;

        if let Ok(rotation) = dictionary.get(b"Rotate") {
            return rotation
                .as_i64()
                .map_err(|error| format!("Nilai Rotate halaman PDF tidak valid: {error}"));
        }

        current = dictionary
            .get(b"Parent")
            .and_then(Object::as_reference)
            .map_err(|_| "Page tree halaman tidak memiliki parent yang valid".to_owned())?;
    }
}

#[cfg(test)]
mod tests {
    use lopdf::{Document, Object, dictionary};

    use super::*;

    fn page_tree_with_rotation(page_rotation: Option<i64>, parent_rotation: Option<i64>) -> Document {
        let mut document = Document::with_version("1.7");
        let pages_id = document.new_object_id();
        let page_id = document.new_object_id();

        let mut pages = dictionary! {
            "Type" => "Pages",
            "Count" => 1,
            "Kids" => vec![Object::Reference(page_id)],
        };
        if let Some(rotation) = parent_rotation {
            pages.set("Rotate", rotation);
        }
        document.objects.insert(pages_id, pages.into());

        let mut page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 100.into(), 100.into()],
        };
        if let Some(rotation) = page_rotation {
            page.set("Rotate", rotation);
        }
        document.objects.insert(page_id, page.into());

        let catalog_id = document.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        document.trailer.set("Root", catalog_id);
        document
    }

    #[test]
    fn rejects_invalid_rotation() {
        let error = rotate_pdf(b"%PDF-1.7", 45).unwrap_err();
        assert!(error.contains("kelipatan 90"));
    }

    #[test]
    fn uses_inherited_rotation_when_page_has_none() {
        let document = page_tree_with_rotation(None, Some(90));
        let page_id = document
            .get_pages()
            .values()
            .copied()
            .next()
            .expect("test page");

        assert_eq!(inherited_rotation(&document, page_id).unwrap(), 90);
    }

    #[test]
    fn local_rotation_overrides_inherited_rotation() {
        let document = page_tree_with_rotation(Some(180), Some(90));
        let page_id = document
            .get_pages()
            .values()
            .copied()
            .next()
            .expect("test page");

        assert_eq!(inherited_rotation(&document, page_id).unwrap(), 180);
    }

    #[test]
    fn detects_page_tree_cycles() {
        let mut document = Document::with_version("1.7");
        let page_id = document.new_object_id();
        let pages_id = document.new_object_id();

        document.objects.insert(
            page_id,
            dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
            }
            .into(),
        );
        document.objects.insert(
            pages_id,
            dictionary! {
                "Type" => "Pages",
                "Parent" => page_id,
                "Count" => 1,
                "Kids" => vec![Object::Reference(page_id)],
            }
            .into(),
        );

        let error = inherited_rotation(&document, page_id).unwrap_err();
        assert!(error.contains("reference cycle"));
    }
}
