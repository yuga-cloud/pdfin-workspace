use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn render_pdf_pages(pdf_bytes: &[u8]) -> Result<Vec<PathBuf>, String> {
    let temp_dir = create_temp_dir()?;

    let pdf_path = temp_dir.join("input.pdf");

    fs::write(&pdf_path, pdf_bytes)
        .map_err(|error| format!("Gagal menulis PDF sementara: {error}"))?;

    let output_prefix = temp_dir.join("page");

    let executable = pdf_renderer_executable();

    let output = Command::new(executable)
        .arg("-jpeg")
        .arg("-r")
        .arg("200")
        .arg(&pdf_path)
        .arg(&output_prefix)
        .output()
        .map_err(|error| format!("Gagal menjalankan PDF renderer: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!(
            "PDF renderer gagal dengan status {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    let mut pages = fs::read_dir(&temp_dir)
        .map_err(|error| format!("Gagal membaca direktori OCR sementara: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg"))
        })
        .collect::<Vec<_>>();

    pages.sort();

    if pages.is_empty() {
        return Err("PDF renderer tidak menghasilkan halaman gambar.".to_owned());
    }

    Ok(pages)
}

fn pdf_renderer_executable() -> &'static str {
    if cfg!(target_os = "windows") {
        "pdftoppm.exe"
    } else {
        "pdftoppm"
    }
}

fn create_temp_dir() -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Gagal mendapatkan waktu sistem: {error}"))?
        .as_nanos();

    let dir = std::env::temp_dir().join(format!("pdfin-ocr-{timestamp}"));

    fs::create_dir_all(&dir)
        .map_err(|error| format!("Gagal membuat direktori OCR sementara: {error}"))?;

    Ok(dir)
}

#[allow(dead_code)]
fn _is_existing_file(path: &Path) -> bool {
    path.is_file()
}
