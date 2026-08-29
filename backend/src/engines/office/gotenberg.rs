use std::{
    fs,
    path::Path,
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

use tempfile::tempdir;

const LIBREOFFICE_TIMEOUT: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn convert_to_pdf(
    document_bytes: &[u8],
    input_extension: &str,
    document_type: &str,
) -> Result<Vec<u8>, String> {
    if document_bytes.is_empty() {
        return Err(format!("{document_type} kosong"));
    }

    let temp_dir = tempdir().map_err(|error| {
        format!("Gagal membuat temporary directory: {error}")
    })?;

    let input_path = temp_dir
        .path()
        .join(format!("input.{input_extension}"));

    fs::write(&input_path, document_bytes).map_err(|error| {
        format!("Gagal menulis file {document_type}: {error}")
    })?;

    let mut command = Command::new("libreoffice");
    command
        .arg("--headless")
        .arg("--convert-to")
        .arg("pdf")
        .arg("--outdir")
        .arg(temp_dir.path())
        .arg(&input_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = run_with_timeout(command, LIBREOFFICE_TIMEOUT)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "LibreOffice gagal mengonversi {document_type}: {}",
            stderr.trim()
        ));
    }

    let output_path = temp_dir.path().join("input.pdf");
    let pdf_bytes = fs::read(&output_path).map_err(|error| {
        format!(
            "LibreOffice tidak menghasilkan file PDF untuk {document_type}: {error}"
        )
    })?;

    if pdf_bytes.is_empty() {
        return Err(format!("LibreOffice menghasilkan PDF kosong untuk {document_type}"));
    }

    Ok(pdf_bytes)
}

fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<Output, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("Gagal menjalankan LibreOffice: {error}"))?;

    wait_with_timeout(&mut child, timeout)?;

    child
        .wait_with_output()
        .map_err(|error| format!("Gagal mengambil output LibreOffice: {error}"))
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }

                return Ok(());
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();

                    return Err(format!(
                        "LibreOffice melebihi batas waktu {} detik dan dihentikan",
                        timeout.as_secs()
                    ));
                }

                thread::sleep(POLL_INTERVAL);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();

                return Err(format!("Gagal menunggu LibreOffice: {error}"));
            }
        }
    }
}

#[allow(dead_code)]
fn _keep_path_type_used(_: &Path) {}
