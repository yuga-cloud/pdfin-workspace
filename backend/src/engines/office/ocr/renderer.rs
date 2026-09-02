use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_INPUT_SIZE_BYTES: usize = 100 * 1024 * 1024;
const MAX_RENDER_PAGES: usize = 100;
const MAX_RENDERED_PAGE_BYTES: u64 = 25 * 1024 * 1024;
const MAX_TOTAL_RENDERED_BYTES: u64 = 512 * 1024 * 1024;
const RENDER_TIMEOUT: Duration = Duration::from_secs(120);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn render_pdf_pages(pdf_bytes: &[u8]) -> Result<Vec<PathBuf>, String> {
    if pdf_bytes.is_empty() {
        return Err("PDF kosong tidak dapat dirender untuk OCR".to_owned());
    }

    if pdf_bytes.len() > MAX_INPUT_SIZE_BYTES {
        return Err(format!(
            "Ukuran PDF melebihi batas render OCR ({} MiB)",
            MAX_INPUT_SIZE_BYTES / 1024 / 1024
        ));
    }

    let temp_dir = create_temp_dir()?;
    let pdf_path = temp_dir.join("input.pdf");

    if let Err(error) = fs::write(&pdf_path, pdf_bytes) {
        cleanup_temp_dir(&temp_dir);
        return Err(format!("Gagal menulis PDF sementara: {error}"));
    }

    let output_prefix = temp_dir.join("page");
    let executable = pdf_renderer_executable();

    let mut process = Command::new(executable)
        .arg("-jpeg")
        .arg("-r")
        .arg("200")
        .arg(&pdf_path)
        .arg(&output_prefix)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            cleanup_temp_dir(&temp_dir);
            format!("Gagal menjalankan PDF renderer: {error}")
        })?;

    let deadline = std::time::Instant::now() + RENDER_TIMEOUT;

    loop {
        match rendered_output_usage(&temp_dir) {
            Ok((page_count, total_bytes)) => {
                if page_count > MAX_RENDER_PAGES {
                    let _ = process.kill();
                    let _ = process.wait();
                    cleanup_temp_dir(&temp_dir);
                    return Err(format!(
                        "Jumlah halaman hasil render melebihi batas maksimum ({MAX_RENDER_PAGES})"
                    ));
                }

                if total_bytes > MAX_TOTAL_RENDERED_BYTES {
                    let _ = process.kill();
                    let _ = process.wait();
                    cleanup_temp_dir(&temp_dir);
                    return Err(format!(
                        "Ukuran total hasil render melebihi batas maksimum ({} MiB)",
                        MAX_TOTAL_RENDERED_BYTES / 1024 / 1024
                    ));
                }
            }
            Err(error) => {
                let _ = process.kill();
                let _ = process.wait();
                cleanup_temp_dir(&temp_dir);
                return Err(error);
            }
        }

        match process.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    cleanup_temp_dir(&temp_dir);
                    return Err(format!("PDF renderer gagal dengan status {status}"));
                }

                break;
            }
            Ok(None) if std::time::Instant::now() >= deadline => {
                let _ = process.kill();
                let _ = process.wait();
                cleanup_temp_dir(&temp_dir);

                return Err(format!(
                    "PDF renderer melebihi batas waktu {} detik",
                    RENDER_TIMEOUT.as_secs()
                ));
            }
            Ok(None) => thread::sleep(PROCESS_POLL_INTERVAL),
            Err(error) => {
                let _ = process.kill();
                let _ = process.wait();
                cleanup_temp_dir(&temp_dir);

                return Err(format!("Gagal memantau PDF renderer: {error}"));
            }
        }
    }

    let mut pages = collect_rendered_pages(&temp_dir)?;

    if pages.len() > MAX_RENDER_PAGES {
        cleanup_temp_dir(&temp_dir);
        return Err(format!(
            "PDF melebihi batas render OCR (maksimum {MAX_RENDER_PAGES} halaman)"
        ));
    }

    pages.sort_by_key(|path| page_number(path));

    if pages.is_empty() {
        cleanup_temp_dir(&temp_dir);
        return Err("PDF renderer tidak menghasilkan halaman gambar.".to_owned());
    }

    let (_, total_bytes) = rendered_output_usage(&temp_dir).inspect_err(|_| {
        cleanup_temp_dir(&temp_dir);
    })?;

    if total_bytes > MAX_TOTAL_RENDERED_BYTES {
        cleanup_temp_dir(&temp_dir);
        return Err(format!(
            "Ukuran total hasil render melebihi batas maksimum ({} MiB)",
            MAX_TOTAL_RENDERED_BYTES / 1024 / 1024
        ));
    }

    for path in &pages {
        let size = fs::metadata(path)
            .map_err(|error| format!("Gagal membaca metadata hasil render JPG: {error}"))?
            .len();

        if size == 0 {
            cleanup_temp_dir(&temp_dir);
            return Err("PDF renderer menghasilkan gambar kosong.".to_owned());
        }

        if size > MAX_RENDERED_PAGE_BYTES {
            cleanup_temp_dir(&temp_dir);
            return Err(format!(
                "Ukuran halaman hasil render melebihi batas maksimum ({} MiB)",
                MAX_RENDERED_PAGE_BYTES / 1024 / 1024
            ));
        }
    }

    Ok(pages)
}

fn rendered_output_usage(dir: &Path) -> Result<(usize, u64), String> {
    let mut page_count = 0usize;
    let mut total_bytes = 0u64;

    for entry in fs::read_dir(dir)
        .map_err(|error| format!("Gagal membaca direktori OCR sementara: {error}"))?
    {
        let path = entry
            .map_err(|error| format!("Gagal membaca entry temporary directory: {error}"))?
            .path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("jpg") {
            continue;
        }

        page_count = page_count
            .checked_add(1)
            .ok_or_else(|| "Jumlah hasil render melebihi kapasitas numerik".to_owned())?;
        let size = fs::metadata(&path)
            .map_err(|error| format!("Gagal membaca ukuran hasil render JPG: {error}"))?
            .len();

        if size > MAX_RENDERED_PAGE_BYTES {
            return Err(format!(
                "Ukuran satu hasil JPG melebihi batas maksimum ({} MiB)",
                MAX_RENDERED_PAGE_BYTES / 1024 / 1024
            ));
        }

        total_bytes = total_bytes
            .checked_add(size)
            .ok_or_else(|| "Ukuran total hasil render melebihi kapasitas numerik".to_owned())?;
    }

    Ok((page_count, total_bytes))
}

fn collect_rendered_pages(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pages = Vec::new();

    for entry in fs::read_dir(dir)
        .map_err(|error| format!("Gagal membaca entry temporary directory: {error}"))?
    {
        let path = entry
            .map_err(|error| format!("Gagal membaca entry temporary directory: {error}"))?
            .path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("jpg") {
            pages.push(path);
        }
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
        .map_err(|error| format!("Gagal mendapatkan timestamp: {error}"))?
        .as_nanos();

    let dir = std::env::temp_dir().join(format!("pdfin-ocr-{timestamp}"));

    fs::create_dir_all(&dir)
        .map_err(|error| format!("Gagal membuat direktori OCR sementara: {error}"))?;

    Ok(dir)
}

fn cleanup_temp_dir(path: &Path) {
    if let Err(error) = fs::remove_dir_all(path) {
        tracing::debug!(path = %path.display(), %error, "Gagal membersihkan direktori OCR sementara");
    }
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
