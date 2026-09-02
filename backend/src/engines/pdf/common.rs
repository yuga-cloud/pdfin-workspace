use std::{fs::File, io::Read, path::Path};

use lopdf::{Document, LoadOptions};

const MAX_PDF_INPUT_BYTES: usize = 500 * 1024 * 1024;
const PDF_HEADER_SCAN_BYTES: usize = 1024;
const MAX_PDF_DECOMPRESSED_STREAM_BYTES: usize = 128 * 1024 * 1024;

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
