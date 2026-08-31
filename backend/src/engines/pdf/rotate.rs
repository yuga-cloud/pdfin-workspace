use lopdf::{Document, Object};

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
        let object = document
            .get_object_mut(page_id)
            .map_err(|error| format!("Gagal mengakses halaman PDF: {error}"))?;

        let dictionary = object
            .as_dict_mut()
            .map_err(|error| format!("Objek halaman PDF tidak valid: {error}"))?;

        let current_rotation = dictionary
            .get(b"Rotate")
            .ok()
            .and_then(|value| value.as_i64().ok())
            .unwrap_or(0);

        let rotation = (current_rotation + extra_deg).rem_euclid(360);

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
