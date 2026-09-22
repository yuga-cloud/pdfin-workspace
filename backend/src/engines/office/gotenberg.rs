use std::{
    fs::{self, File},
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const MAX_INPUT_SIZE_BYTES: usize = 100 * 1024 * 1024;
const MAX_OUTPUT_SIZE_BYTES: usize = 200 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const CONVERSION_TIMEOUT: Duration = Duration::from_secs(120);
const OUTPUT_POLL_INTERVAL: Duration = Duration::from_millis(50);

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
    let output_path = temp_path.join("input.pdf");
    let stderr_path = temp_path.join("libreoffice.stderr");

    fs::write(&input_path, document_bytes)
        .map_err(|error| format!("Gagal menulis file {document_type}: {error}"))?;

    let user_installation = format!("-env:UserInstallation=file://{}", temp_path.display());
    let stderr_file = File::create(&stderr_path)
        .map_err(|error| format!("Gagal membuat log LibreOffice: {error}"))?;

    let mut process = crate::engines::sandbox::command("libreoffice", temp_path)?
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
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|error| format!("Gagal menjalankan LibreOffice: {error}"))?;

    let deadline = Instant::now() + CONVERSION_TIMEOUT;

    loop {
        if output_size_exceeds(&output_path, MAX_OUTPUT_SIZE_BYTES)? {
            let _ = process.kill();
            let _ = process.wait();

            return Err(format!(
                "Hasil PDF melebihi batas ukuran {} MB dan proses dihentikan",
                MAX_OUTPUT_SIZE_BYTES / 1024 / 1024
            ));
        }

        match process.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let stderr = read_limited_stderr(&stderr_path).unwrap_or_default();
                    return Err(format!(
                        "LibreOffice gagal mengonversi {document_type} (exit code {:?}): {stderr}",
                        status.code()
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
            Ok(None) => thread::sleep(OUTPUT_POLL_INTERVAL),
            Err(error) => {
                let _ = process.kill();
                let _ = process.wait();

                return Err(format!("Gagal memantau LibreOffice: {error}"));
            }
        }
    }

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

fn output_size_exceeds(path: &std::path::Path, max_size: usize) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() {
                return Err("Hasil PDF bukan regular file".to_owned());
            }
            Ok(metadata.len() > max_size as u64)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("Gagal memeriksa ukuran hasil PDF: {error}")),
    }
}

fn read_limited_stderr(path: &std::path::Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::with_capacity(MAX_STDERR_BYTES + 1);
    file.take((MAX_STDERR_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;

    let truncated = bytes.len() > MAX_STDERR_BYTES;
    bytes.truncate(MAX_STDERR_BYTES);

    let mut stderr = String::from_utf8_lossy(&bytes).trim().to_owned();
    if truncated {
        stderr.push_str(" [stderr truncated]");
    }
    Ok(stderr)
}
