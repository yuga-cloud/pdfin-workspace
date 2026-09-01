use std::path::Path;

use lopdf::{Document, Object};

use super::common::{load_pdf_document_from_path, validate_pdf_path};

const MAX_PAGE_ORDER_ITEMS: usize = 5_000;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024 * 1024;

/// Reorders PDF pages from a disk-backed input.
pub fn manage_pages_from_path(pdf_path: &Path, page_order: &[u32]) -> Result<Vec<u8>, String> {
    validate_pdf_path(pdf_path)?;

    if page_order.is_empty() {
        return Err("Urutan halaman tidak boleh kosong".to_owned());
    }

    if page_order.len() > MAX_PAGE_ORDER_ITEMS {
        return Err(format!(
            "Jumlah halaman hasil melebihi batas maksimum ({MAX_PAGE_ORDER_ITEMS})"
        ));
    }

    if page_order.contains(&0) {
        return Err("Nomor halaman harus dimulai dari 1".to_owned());
    }

    let mut document = load_pdf_document_from_path(pdf_path)?;
    if document.get_pages().is_empty() {
        return Err("PDF tidak memiliki halaman.".to_owned());
    }

    let mut output = Document::with_version(document.version.clone());
    document.renumber_objects_with(1);
    let pages = document.get_pages();

    let selected_page_ids = page_order
        .iter()
        .map(|&page_number| {
            pages
                .get(&page_number)
                .copied()
                .ok_or_else(|| format!("Halaman {page_number} tidak ditemukan."))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let catalog_id = document
        .trailer
        .get(b"Root")
        .map_err(|error| format!("Catalog PDF tidak ditemukan: {error}"))?
        .as_reference()
        .map_err(|error| format!("Reference Catalog PDF tidak valid: {error}"))?;

    let catalog_dictionary = document
        .get_object(catalog_id)
        .map_err(|error| format!("Gagal membaca Catalog PDF: {error}"))?
        .as_dict()
        .map_err(|error| format!("Catalog PDF tidak valid: {error}"))?
        .clone();

    let original_pages_id = catalog_dictionary
        .get(b"Pages")
        .map_err(|error| format!("Pages root tidak ditemukan: {error}"))?
        .as_reference()
        .map_err(|error| format!("Reference Pages PDF tidak valid: {error}"))?;

    let original_pages_dictionary = document
        .get_object(original_pages_id)
        .map_err(|error| format!("Gagal membaca Pages root: {error}"))?
        .as_dict()
        .map_err(|error| format!("Object Pages PDF tidak valid: {error}"))?
        .clone();

    let output_pages_id = (document.max_id.saturating_add(1), 0);
    let output_catalog_id = (output_pages_id.0.saturating_add(1), 0);

    output.objects = std::mem::take(&mut document.objects);

    let mut kids = Vec::with_capacity(selected_page_ids.len());
    for page_id in selected_page_ids {
        let page_object = output
            .objects
            .get_mut(&page_id)
            .ok_or_else(|| format!("Object halaman {:?} tidak ditemukan.", page_id))?;

        let page_dictionary = page_object
            .as_dict_mut()
            .map_err(|error| format!("Object halaman PDF tidak valid: {error}"))?;

        page_dictionary.set("Parent", output_pages_id);
        kids.push(Object::Reference(page_id));
    }

    let mut pages_dictionary = original_pages_dictionary;
    pages_dictionary.remove(b"Parent");
    pages_dictionary.set("Type", "Pages");
    pages_dictionary.set("Count", u32::try_from(page_order.len()).unwrap_or(u32::MAX));
    pages_dictionary.set("Kids", kids);
    output
        .objects
        .insert(output_pages_id, Object::Dictionary(pages_dictionary));

    let mut new_catalog = catalog_dictionary;
    new_catalog.set("Type", "Catalog");
    new_catalog.set("Pages", output_pages_id);
    new_catalog.remove(b"Outlines");
    output
        .objects
        .insert(output_catalog_id, Object::Dictionary(new_catalog));

    output.trailer.set("Root", output_catalog_id);
    output.objects.remove(&catalog_id);
    output.objects.remove(&original_pages_id);
    output.renumber_objects();
    output.adjust_zero_pages();
    output.compress();

    let mut result = Vec::new();
    output
        .save_to(&mut result)
        .map_err(|error| format!("Gagal menyimpan PDF hasil pengaturan halaman: {error}"))?;

    if result.is_empty() {
        return Err("PDF hasil pengaturan halaman kosong.".to_owned());
    }

    if result.len() > MAX_OUTPUT_BYTES {
        return Err(format!(
            "PDF hasil pengaturan halaman melebihi batas maksimum ({} MB)",
            MAX_OUTPUT_BYTES / 1024 / 1024
        ));
    }

    Ok(result)
}
