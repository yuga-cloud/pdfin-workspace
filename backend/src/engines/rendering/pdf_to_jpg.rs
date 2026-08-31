use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::engines::{common::validate_input, pdf::common::load_pdf_document};

const MAX_RENDER_PAGES: usize = 100;
const MAX_INPUT_SIZE_BYTES: usize = 100 * 1024 * 1024;
const MAX_OUTPUT_FILE_SIZE_BYTES: usize = 25 * 1024 * 1024;

pub fn pdf_to_jpg(pdf_bytes: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    validate_input(pdf_bytes, "PDF")?;

    if pdf_bytes.len() > MAX_INPUT_SIZE_BYTES {
        return Err("Ukuran PDF melebihi batas render JPG".to_owned());
    }

    let document = load_pdf_document(pdf_bytes)
        .map_err(|error| format!("Gagal membaca PDF sebelum render JPG: {error}"))?;
    let page_count = document.get_pages().len();

    if page_count == 0 {
        return Err("PDF tidak memiliki halaman".to_owned());
    }

    if page_count > MAX_RENDER_PAGES {
        return Err(format!(
            "PDF melebihi batas render {} halaman",
            MAX_RENDER_PAGES
        ));
    }

    let temp_dir = tempfile::tempdir()
        .map_err(|error| format!("Gagal membuat temporary directory: {error}"))?;

    let input_path = temp_dir.path().join("input.pdf");
    let output_prefix = temp_dir.path().join("page");

    fs::write(&input_path, pdf_bytes)
        .map_err(|error| format!("Gagal menulis temporary PDF: {error}"))?;

    let output = Command::new("pdftocairo")
        .arg("-jpeg")
        .arg("-r")
        .arg("150")
        .arg(&input_path)
        .arg(&output_prefix)
        .output()
        .map_err(|error| format!("Gagal menjalankan pdftocairo: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "pdftocairo gagal: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let mut page_files = collect_page_files(temp_dir.path())?;
    page_files.sort_by_key(|path| page_number(path));

    let mut pages = Vec::with_capacity(page_files.len());

    for path in page_files {
        let bytes = fs::read(&path).map_err(|error| format!("Gagal membaca hasil JPG: {error}"))?;

        if bytes.len() > MAX_OUTPUT_FILE_SIZE_BYTES {
            return Err("Ukuran hasil JPG melebihi batas maksimum".to_owned());
        }

        if !bytes.is_empty() {
            pages.push(bytes);
        }
    }

    if pages.len() != page_count {
        return Err(format!(
            "Jumlah gambar hasil render tidak sesuai: diharapkan {page_count}, didapat {}",
            pages.len()
        ));
    }

    Ok(pages)
}

fn collect_page_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for entry in
        fs::read_dir(dir).map_err(|error| format!("Gagal membaca temporary directory: {error}"))?
    {
        let path = entry
            .map_err(|error| format!("Gagal membaca entry temporary directory: {error}"))?
            .path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("jpg") {
            files.push(path);
        }
    }

    Ok(files)
}

fn page_number(path: &Path) -> u32 {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| {
            stem.rsplit_once('-')
                .and_then(|(_, number)| number.parse().ok())
        })
        .unwrap_or(u32::MAX)
}
