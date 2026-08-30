use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_INPUT_SIZE_BYTES: usize = 100 * 1024 * 1024;
const MAX_OUTPUT_SIZE_BYTES: usize = 200 * 1024 * 1024;

fn unique_temp_dir() -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Gagal mendapatkan timestamp: {e}"))?
        .as_nanos();

    let dir = std::env::temp_dir().join(format!("pdfin-office-{timestamp}"));

    fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat temporary directory: {e}"))?;

    Ok(dir)
}

pub fn convert_to_pdf(
    document_bytes: &[u8],
    input_extension: &str,
    document_type: &str,
) -> Result<Vec<u8>, String> {
    if document_bytes.is_empty() {
        return Err(format!("{document_type} kosong"));
    }

    if document_bytes.len() > MAX_INPUT_SIZE_BYTES {
        return Err(format!(
            "{document_type} melebihi batas ukuran {} MB",
            MAX_INPUT_SIZE_BYTES / 1024 / 1024
        ));
    }

    let temp_dir = unique_temp_dir()?;

    let input_path = temp_dir.join(format!("input.{input_extension}"));

    fs::write(&input_path, document_bytes)
        .map_err(|e| format!("Gagal menulis file {document_type}: {e}"))?;

    let output = Command::new("libreoffice")
        .arg("--headless")
        .arg("--norestore")
        .arg("--convert-to")
        .arg("pdf")
        .arg("--outdir")
        .arg(&temp_dir)
        .arg(&input_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Gagal menjalankan LibreOffice: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_dir_all(&temp_dir);

        return Err(format!(
            "LibreOffice gagal mengonversi {document_type}: {}",
            stderr.trim()
        ));
    }

    let output_path = temp_dir.join("input.pdf");

    let pdf_bytes = fs::read(&output_path).map_err(|e| {
        format!("LibreOffice tidak menghasilkan file PDF untuk {document_type}: {e}")
    })?;

    let _ = fs::remove_dir_all(&temp_dir);

    if pdf_bytes.is_empty() {
        return Err("LibreOffice menghasilkan PDF kosong".to_owned());
    }

    if pdf_bytes.len() > MAX_OUTPUT_SIZE_BYTES {
        return Err(format!(
            "Hasil PDF melebihi batas ukuran {} MB",
            MAX_OUTPUT_SIZE_BYTES / 1024 / 1024
        ));
    }

    Ok(pdf_bytes)
}
