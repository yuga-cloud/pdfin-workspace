use std::{collections::HashSet, fs::File, io::Read, path::Path};

use lopdf::{Document, LoadOptions, Object};

const MAX_PDF_INPUT_BYTES: usize = 500 * 1024 * 1024;
const PDF_HEADER_SCAN_BYTES: usize = 1024;
const MAX_PDF_DECOMPRESSED_STREAM_BYTES: usize = 128 * 1024 * 1024;
const MAX_PDF_PAGE_DIMENSION_POINTS: f64 = 20_000.0;
const MAX_PDF_RENDER_WIDTH_PIXELS: f64 = 16_384.0;
const MAX_PDF_RENDER_HEIGHT_PIXELS: f64 = 16_384.0;

/// Validate page geometry before handing an untrusted PDF to a raster renderer.
///
/// Renderers may allocate from page dimensions before they produce an output
/// file, so output-size limits alone are not sufficient for resource safety.
pub fn validate_pdf_page_dimensions(document: &Document) -> Result<(), String> {
    for (page_number, page_id) in document.get_pages() {
        let page = document
            .get_dictionary(page_id)
            .map_err(|error| format!("Gagal membaca halaman PDF {page_number}: {error}"))?;

        let media_box = inherited_page_box(document, page_id, b"MediaBox", page_number)?;
        let crop_box = inherited_page_box_optional(document, page_id, b"CropBox", page_number)?
            .unwrap_or(media_box);

        let user_unit = if page.has(b"UserUnit") {
            page.get_deref(b"UserUnit", document)
                .map_err(|error| {
                    format!("UserUnit halaman PDF {page_number} tidak valid: {error}")
                })?
                .as_float()
                .map_err(|error| {
                    format!("UserUnit halaman PDF {page_number} tidak valid: {error}")
                })?
        } else {
            1.0
        };

        if !user_unit.is_finite() || user_unit <= 0.0 {
            return Err(format!("UserUnit halaman PDF {page_number} tidak valid"));
        }

        let width_points = (crop_box[2] - crop_box[0]).abs() * f64::from(user_unit);
        let height_points = (crop_box[3] - crop_box[1]).abs() * f64::from(user_unit);

        if !width_points.is_finite()
            || !height_points.is_finite()
            || width_points <= 0.0
            || height_points <= 0.0
        {
            return Err(format!("Ukuran halaman PDF {page_number} tidak valid"));
        }

        if width_points > MAX_PDF_PAGE_DIMENSION_POINTS
            || height_points > MAX_PDF_PAGE_DIMENSION_POINTS
        {
            return Err(format!(
                "Dimensi halaman PDF {page_number} melebihi batas maksimum ({MAX_PDF_PAGE_DIMENSION_POINTS:.0} pt)"
            ));
        }
    }

    Ok(())
}

pub fn validate_pdf_render_dimensions(
    document: &Document,
    dpi: u32,
    max_render_pixels: u64,
) -> Result<(), String> {
    validate_pdf_page_dimensions(document)?;

    let dpi = f64::from(dpi);
    let max_render_pixels = max_render_pixels as f64;

    for (page_number, page_id) in document.get_pages() {
        let page = document
            .get_dictionary(page_id)
            .map_err(|error| format!("Gagal membaca halaman PDF {page_number}: {error}"))?;

        let media_box = inherited_page_box(document, page_id, b"MediaBox", page_number)?;
        let crop_box = inherited_page_box_optional(document, page_id, b"CropBox", page_number)?
            .unwrap_or(media_box);

        let user_unit = if page.has(b"UserUnit") {
            page.get_deref(b"UserUnit", document)
                .map_err(|error| {
                    format!("UserUnit halaman PDF {page_number} tidak valid: {error}")
                })?
                .as_float()
                .map_err(|error| {
                    format!("UserUnit halaman PDF {page_number} tidak valid: {error}")
                })?
        } else {
            1.0
        };

        let width_points = (crop_box[2] - crop_box[0]).abs() * f64::from(user_unit);
        let height_points = (crop_box[3] - crop_box[1]).abs() * f64::from(user_unit);

        let width_pixels = width_points * dpi / 72.0;
        let height_pixels = height_points * dpi / 72.0;

        if width_pixels > MAX_PDF_RENDER_WIDTH_PIXELS
            || height_pixels > MAX_PDF_RENDER_HEIGHT_PIXELS
            || width_pixels * height_pixels > max_render_pixels
        {
            return Err(format!(
                "Dimensi halaman PDF {page_number} terlalu besar untuk render pada {dpi} DPI"
            ));
        }
    }

    Ok(())
}

fn inherited_page_box_optional(
    document: &Document,
    page_id: (u32, u16),
    key: &[u8],
    page_number: u32,
) -> Result<Option<[f64; 4]>, String> {
    let mut current = page_id;
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(current) {
            return Err(format!(
                "Page tree halaman {page_number} mengandung reference cycle"
            ));
        }

        let dictionary = document
            .get_dictionary(current)
            .map_err(|error| format!("Gagal membaca page tree halaman {page_number}: {error}"))?;

        if let Ok(value) = dictionary.get_deref(key, document) {
            return parse_page_box(value, key, page_number).map(Some);
        }

        let Ok(parent) = dictionary.get(b"Parent").and_then(Object::as_reference) else {
            return Ok(None);
        };
        current = parent;
    }
}

fn inherited_page_box(
    document: &Document,
    page_id: (u32, u16),
    key: &[u8],
    page_number: u32,
) -> Result<[f64; 4], String> {
    inherited_page_box_optional(document, page_id, key, page_number)?.ok_or_else(|| {
        format!(
            "Halaman PDF {page_number} tidak memiliki {}",
            String::from_utf8_lossy(key)
        )
    })
}

fn parse_page_box(value: &Object, key: &[u8], page_number: u32) -> Result<[f64; 4], String> {
    let values = value.as_array().map_err(|error| {
        format!(
            "{} halaman PDF {page_number} tidak valid: {error}",
            String::from_utf8_lossy(key)
        )
    })?;

    if values.len() != 4 {
        return Err(format!(
            "{} halaman PDF {page_number} harus memiliki 4 nilai",
            String::from_utf8_lossy(key)
        ));
    }

    let mut result = [0.0; 4];
    for (index, item) in values.iter().enumerate() {
        let value = item.as_float().map_err(|error| {
            format!(
                "{} halaman PDF {page_number} tidak valid: {error}",
                String::from_utf8_lossy(key)
            )
        })?;
        if !value.is_finite() {
            return Err(format!(
                "{} halaman PDF {page_number} mengandung nilai non-finite",
                String::from_utf8_lossy(key)
            ));
        }
        result[index] = f64::from(value);
    }

    Ok(result)
}

pub fn validate_pdf_path(path: &Path) -> Result<(), String> {
    let metadata =
        std::fs::metadata(path).map_err(|error| format!("Gagal membaca metadata PDF: {error}"))?;
    let size = usize::try_from(metadata.len())
        .map_err(|_| "Ukuran PDF melebihi kapasitas yang didukung".to_owned())?;

    if size == 0 {
        return Err("File PDF kosong".to_owned());
    }

    if size > MAX_PDF_INPUT_BYTES {
        return Err(format!(
            "Ukuran PDF melebihi batas maksimum ({} MiB)",
            MAX_PDF_INPUT_BYTES / 1024 / 1024
        ));
    }

    let mut file = File::open(path).map_err(|error| format!("Gagal membuka PDF: {error}"))?;
    let mut header = [0_u8; PDF_HEADER_SCAN_BYTES];
    let bytes_read = file
        .read(&mut header)
        .map_err(|error| format!("Gagal membaca header PDF: {error}"))?;

    validate_pdf_signature(&header[..bytes_read])
}

/// Load a PDF with the explicit decompressed-stream budget used by the backend.
pub fn load_pdf_document(pdf_bytes: &[u8]) -> Result<Document, String> {
    Document::load_mem_with_options(
        pdf_bytes,
        LoadOptions::with_max_decompressed_size(MAX_PDF_DECOMPRESSED_STREAM_BYTES),
    )
    .map_err(|error| format!("Gagal membaca PDF: {error}"))
}

/// Load a PDF directly from disk while retaining the decompressed-stream budget.
pub fn load_pdf_document_from_path(path: &Path) -> Result<Document, String> {
    Document::load_with_options(
        path,
        LoadOptions::with_max_decompressed_size(MAX_PDF_DECOMPRESSED_STREAM_BYTES),
    )
    .map_err(|error| format!("Gagal membaca PDF: {error}"))
}

fn validate_pdf_signature(bytes: &[u8]) -> Result<(), String> {
    let header_window = bytes.len().min(PDF_HEADER_SCAN_BYTES);

    if bytes[..header_window]
        .windows(b"%PDF-".len())
        .any(|window| window == b"%PDF-")
    {
        Ok(())
    } else {
        Err("File bukan PDF yang valid".to_owned())
    }
}
#[cfg(test)]
mod tests {
    use lopdf::{Dictionary, Document, Object};

    use super::{validate_pdf_page_dimensions, validate_pdf_render_dimensions};

    #[test]
    fn accepts_large_page_for_non_raster_processing() {
        let document = test_document([0, 0, 20_000, 20_000]);
        assert!(validate_pdf_page_dimensions(&document).is_ok());
    }

    #[test]
    fn rejects_oversized_render_page() {
        let document = test_document([0, 0, 100_000, 100]);
        assert!(validate_pdf_render_dimensions(&document, 150, 50_000_000).is_err());
        assert!(validate_pdf_page_dimensions(&document).is_err());
    }

    fn test_document(media_box: [i64; 4]) -> Document {
        let mut document = Document::with_version("1.7");
        let catalog_id = document.new_object_id();
        let pages_id = document.new_object_id();
        let page_id = document.new_object_id();

        document.objects.insert(
            pages_id,
            Dictionary::from_iter([
                (b"Type".to_vec(), Object::Name(b"Pages".to_vec())),
                (
                    b"Kids".to_vec(),
                    Object::Array(vec![Object::Reference(page_id)]),
                ),
                (b"Count".to_vec(), Object::Integer(1)),
                (
                    b"MediaBox".to_vec(),
                    Object::Array(
                        media_box
                            .iter()
                            .copied()
                            .map(Object::Integer)
                            .collect::<Vec<_>>(),
                    ),
                ),
            ])
            .into(),
        );

        document.objects.insert(
            page_id,
            Dictionary::from_iter([
                (b"Type".to_vec(), Object::Name(b"Page".to_vec())),
                (b"Parent".to_vec(), Object::Reference(pages_id)),
            ])
            .into(),
        );

        document.objects.insert(
            catalog_id,
            Dictionary::from_iter([
                (b"Type".to_vec(), Object::Name(b"Catalog".to_vec())),
                (b"Pages".to_vec(), Object::Reference(pages_id)),
            ])
            .into(),
        );

        document.trailer.set("Root", catalog_id);
        document
    }
}
