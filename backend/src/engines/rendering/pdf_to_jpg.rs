use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::engines::common::validate_input;

pub fn pdf_to_jpg(pdf_bytes: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    validate_input(pdf_bytes, "PDF")?;

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
        .map_err(|error| {
            format!(
                "Gagal menjalankan pdftocairo. \
                 Pastikan Poppler terpasang: {error}"
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!(
            "pdftocairo gagal dengan status {}: {}",
            output
                .status
                .code()
                .map_or_else(|| "unknown".to_owned(), |code| code.to_string()),
            stderr.trim()
        ));
    }

    let mut pages = Vec::new();

    let mut page_files = collect_page_files(temp_dir.path())?;

    page_files.sort_by_key(|path| page_number(path));

    for path in page_files {
        let bytes = fs::read(&path).map_err(|error| format!("Gagal membaca hasil JPG: {error}"))?;

        if !bytes.is_empty() {
            pages.push(bytes);
        }
    }

    if pages.is_empty() {
        return Err("Konversi PDF ke JPG tidak menghasilkan gambar".to_owned());
    }

    Ok(pages)
}

fn collect_page_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries =
        fs::read_dir(dir).map_err(|error| format!("Gagal membaca temporary directory: {error}"))?;

    let mut files = Vec::new();

    for entry in entries {
        let entry =
            entry.map_err(|error| format!("Gagal membaca entry temporary directory: {error}"))?;

        let path = entry.path();

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
                .and_then(|(_, number)| number.parse::<u32>().ok())
        })
        .unwrap_or(u32::MAX)
}
