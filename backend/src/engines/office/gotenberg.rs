use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const MAX_INPUT_SIZE_BYTES: usize = 100 * 1024 * 1024;
const MAX_OUTPUT_SIZE_BYTES: usize = 200 * 1024 * 1024;
const CONVERSION_TIMEOUT: Duration = Duration::from_secs(120);

fn unique_temp_dir() -> Result<tempfile::TempDir, String> {
    tempfile::Builder::new()
        .prefix("pdfin-office-")
        .tempdir()
        .map_err(|error| format!("Gagal membuat temporary directory: {error}"))
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
    let temp_path = temp_dir.path();
    let input_path = temp_path.join(format!("input.{input_extension}"));

    fs::write(&input_path, document_bytes)
        .map_err(|error| format!("Gagal menulis file {document_type}: {error}"))?;

    let user_installation = format!(
        "-env:UserInstallation=file://{}",
        temp_path.display()
    );

    let mut process = Command::new("libreoffice")
        .arg("--headless")
        .arg("--norestore")
        .arg(user_installation)
        .arg("--convert-to")
        .arg("pdf")
        .arg("--outdir")
        .arg(temp_path)
        .arg(&input_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Gagal menjalankan LibreOffice: {error}"))?;

    let deadline = Instant::now() + CONVERSION_TIMEOUT;

    loop {
        match process.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let output = process
                        .wait_with_output()
                        .map_err(|error| format!("Gagal membaca output LibreOffice: {error}"))?;
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    return Err(format!(
                        "LibreOffice gagal mengonversi {document_type}: {}",
                        stderr.trim()
                    ));
                }

                break;
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = process.kill();
                let _ = process.wait();

                return Err(format!(
                    "LibreOffice melebihi batas waktu {} detik",
                    CONVERSION_TIMEOUT.as_secs()
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                let _ = process.kill();
                let _ = process.wait();

                return Err(format!("Gagal memantau LibreOffice: {error}"));
            }
        }
    }

    let output_path = temp_path.join("input.pdf");
    let pdf_bytes = fs::read(&output_path).map_err(|error| {
        format!("LibreOffice tidak menghasilkan file PDF untuk {document_type}: {error}")
    })?;

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
