use std::{collections::{BTreeMap, HashMap, HashSet}, path::Path};

use lopdf::{Document, Object, ObjectId};

use super::common::{load_pdf_document, load_pdf_document_from_path, validate_pdf, validate_pdf_path};

const MAX_MERGE_INPUTS: usize = 32;
const MAX_TOTAL_INPUT_BYTES: usize = 500 * 1024 * 1024;
const MAX_MERGED_PAGES: usize = 10_000;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024 * 1024;

/// Menggabungkan beberapa PDF menjadi satu dokumen dari memory buffer.
pub fn merge_pdfs(pdfs: &[&[u8]]) -> Result<Vec<u8>, String> {
    validate_merge_limits(pdfs.len())?;

    let total_input_bytes = pdfs
        .iter()
        .try_fold(0usize, |total, pdf| total.checked_add(pdf.len()))
        .ok_or_else(|| "Total ukuran PDF melebihi batas numerik yang didukung".to_owned())?;

    if total_input_bytes > MAX_TOTAL_INPUT_BYTES {
        return Err(format!(
            "Total ukuran PDF melebihi batas maksimum ({} MB)",
            MAX_TOTAL_INPUT_BYTES / 1024 / 1024
        ));
    }

    for (index, pdf) in pdfs.iter().enumerate() {
        validate_pdf(pdf).map_err(|error| format!("PDF ke-{} tidak valid: {error}", index + 1))?;
    }

    merge_documents(pdfs.iter().enumerate().map(|(index, pdf)| {
        load_pdf_document(pdf).map(|document| (index, document))
    }))
}

/// Menggabungkan PDF langsung dari file agar upload besar tidak perlu disalin
/// kembali ke heap sebelum `lopdf` membangun object graph.
pub fn merge_pdfs_from_paths(paths: &[&Path]) -> Result<Vec<u8>, String> {
    validate_merge_limits(paths.len())?;

    let mut total_input_bytes = 0usize;
    for (index, path) in paths.iter().enumerate() {
        validate_pdf_path(path).map_err(|error| format!("PDF ke-{} tidak valid: {error}", index + 1))?;
        let size = std::fs::metadata(path)
            .map_err(|error| format!("Gagal membaca metadata PDF ke-{}: {error}", index + 1))?
            .len();
        let size = usize::try_from(size)
            .map_err(|_| "Total ukuran PDF melebihi kapasitas yang didukung".to_owned())?;
        total_input_bytes = total_input_bytes
            .checked_add(size)
            .ok_or_else(|| "Total ukuran PDF melebihi batas numerik yang didukung".to_owned())?;
    }

    if total_input_bytes > MAX_TOTAL_INPUT_BYTES {
        return Err(format!(
            "Total ukuran PDF melebihi batas maksimum ({} MB)",
            MAX_TOTAL_INPUT_BYTES / 1024 / 1024
        ));
    }

    merge_documents(paths.iter().enumerate().map(|(index, path)| {
        load_pdf_document_from_path(path).map(|document| (index, document))
    }))
}

fn validate_merge_limits(input_count: usize) -> Result<(), String> {
    if input_count == 0 {
        return Err("Tidak ada file PDF yang akan digabungkan".to_owned());
    }

    if input_count > MAX_MERGE_INPUTS {
        return Err(format!(
            "Jumlah PDF melebihi batas maksimum ({MAX_MERGE_INPUTS})"
        ));
    }

    Ok(())
}

fn merge_documents<I>(documents: I) -> Result<Vec<u8>, String>
where
    I: IntoIterator<Item = Result<(usize, Document), String>>,
{
    let mut next_object_id: u32 = 1;
    let mut pages_in_order: Vec<(ObjectId, Object)> = Vec::new();
    let mut document_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut output = Document::with_version("1.5");

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for document_result in documents {
        let (index, mut document) = document_result
            .map_err(|error| format!("Gagal membaca PDF input: {error}"))?;

        document.renumber_objects_with(next_object_id);
        next_object_id = document.max_id.saturating_add(1);

        let pages = document.get_pages();

        if pages_in_order.len().saturating_add(pages.len()) > MAX_MERGED_PAGES {
            return Err(format!(
                "Jumlah halaman hasil merge melebihi batas maksimum ({MAX_MERGED_PAGES})"
            ));
        }

        let page_ids: Vec<ObjectId> = pages.values().copied().collect();
        let page_id_set: HashSet<ObjectId> = page_ids.iter().copied().collect();
        let mut page_objects = HashMap::with_capacity(page_ids.len());

        for (object_id, object) in document.objects {
            if page_id_set.contains(&object_id) {
                page_objects.insert(object_id, object);
                continue;
            }

            match object.type_name().unwrap_or_default() {
                b"Catalog" => {
                    if catalog_object.is_none() {
                        catalog_object = Some((object_id, object));
                    }
                }
                b"Pages" => {
                    if pages_object.is_none() {
                        pages_object = Some((object_id, object));
                    }
                }
                b"Outlines" | b"Outline" => {}
                _ => {
                    document_objects.insert(object_id, object);
                }
            }
        }

        for object_id in page_ids {
            let object = page_objects
                .remove(&object_id)
                .ok_or_else(|| format!("Object halaman {:?} tidak ditemukan pada PDF ke-{}.", object_id, index + 1))?;
            pages_in_order.push((object_id, object));
        }
    }

    let (pages_id, pages_source_object) =
        pages_object.ok_or_else(|| "Object Pages tidak ditemukan pada PDF.".to_owned())?;

    let (catalog_id, catalog_source_object) =
        catalog_object.ok_or_else(|| "Object Catalog tidak ditemukan pada PDF.".to_owned())?;

    output.objects.extend(document_objects);

    for (object_id, object) in &pages_in_order {
        let dictionary = object
            .as_dict()
            .map_err(|error| format!("Object halaman PDF tidak valid: {error}"))?;

        let mut dictionary = dictionary.clone();
        dictionary.set("Parent", pages_id);

        output
            .objects
            .insert(*object_id, Object::Dictionary(dictionary));
    }

    let mut pages_dictionary = pages_source_object
        .as_dict()
        .map_err(|error| format!("Object Pages tidak valid: {error}"))?
        .clone();

    pages_dictionary.remove(b"Parent");
    pages_dictionary.set("Type", "Pages");
    pages_dictionary.set("Count", pages_in_order.len() as u32);

    let kids = pages_in_order
        .iter()
        .map(|(object_id, _)| Object::Reference(*object_id))
        .collect::<Vec<_>>();
    pages_dictionary.set("Kids", kids);

    output
        .objects
        .insert(pages_id, Object::Dictionary(pages_dictionary));

    let mut catalog_dictionary = catalog_source_object
        .as_dict()
        .map_err(|error| format!("Object Catalog tidak valid: {error}"))?
        .clone();

    catalog_dictionary.set("Type", "Catalog");
    catalog_dictionary.set("Pages", pages_id);
    catalog_dictionary.remove(b"Outlines");

    output
        .objects
        .insert(catalog_id, Object::Dictionary(catalog_dictionary));

    output.trailer.set("Root", catalog_id);
    output.max_id = output
        .objects
        .keys()
        .map(|(id, _generation)| *id)
        .max()
        .unwrap_or(0);

    output.renumber_objects();
    output.adjust_zero_pages();
    output.compress();

    let mut result = Vec::new();
    output
        .save_to(&mut result)
        .map_err(|error| format!("Gagal menyimpan PDF hasil merge: {error}"))?;

    if result.is_empty() {
        return Err("PDF hasil merge kosong.".to_owned());
    }

    if result.len() > MAX_OUTPUT_BYTES {
        return Err(format!(
            "PDF hasil merge melebihi batas maksimum ({} MB)",
            MAX_OUTPUT_BYTES / 1024 / 1024
        ));
    }

    Ok(result)
}
