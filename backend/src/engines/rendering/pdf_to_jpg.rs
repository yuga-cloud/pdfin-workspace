use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

use crate::engines::{
    common::validate_input,
    pdf::common::{load_pdf_document, validate_pdf_render_dimensions},
};

const MAX_RENDER_PAGES: usize = 100;
const MAX_INPUT_SIZE_BYTES: usize = 100 * 1024 * 1024;
const MAX_OUTPUT_FILE_SIZE_BYTES: usize = 25 * 1024 * 1024;
const MAX_TOTAL_OUTPUT_BYTES: usize = 512 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const RENDER_TIMEOUT: Duration = Duration::from_secs(120);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(25);

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

    validate_pdf_render_dimensions(&document, 150, 50_000_000)
        .map_err(|error| format!("PDF ditolak sebelum render JPG: {error}"))?;

    drop(document);

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
    let stderr_path = temp_dir.path().join("pdftocairo.stderr");

    fs::write(&input_path, pdf_bytes)
        .map_err(|error| format!("Gagal menulis temporary PDF: {error}"))?;

    run_pdftocairo(&input_path, &output_prefix, &stderr_path, temp_dir.path())?;

    let mut page_files = collect_page_files(temp_dir.path())?;
    page_files.sort_by_key(|path| page_number(path));

    let mut pages = Vec::with_capacity(page_files.len());
    let mut total_output_bytes = 0usize;

    for path in page_files {
        let bytes = fs::read(&path).map_err(|error| format!("Gagal membaca hasil JPG: {error}"))?;

        if bytes.len() > MAX_OUTPUT_FILE_SIZE_BYTES {
            return Err("Ukuran hasil JPG melebihi batas maksimum".to_owned());
        }

        total_output_bytes = total_output_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| "Ukuran total hasil JPG terlalu besar".to_owned())?;

        if total_output_bytes > MAX_TOTAL_OUTPUT_BYTES {
            return Err("Ukuran total hasil JPG melebihi batas maksimum".to_owned());
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

fn run_pdftocairo(
    input_path: &Path,
    output_prefix: &Path,
    stderr_path: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    let stderr_file = File::create(stderr_path)
        .map_err(|error| format!("Gagal membuat log pdftocairo: {error}"))?;

    let mut child = crate::engines::sandbox::command("pdftocairo", output_dir)?
        .arg("-jpeg")
        .arg("-r")
        .arg("150")
        .arg(input_path)
        .arg(output_prefix)
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|error| format!("Gagal menjalankan pdftocairo: {error}"))?;

    let deadline = Instant::now() + RENDER_TIMEOUT;

    loop {
        match rendered_output_usage(output_dir) {
            Ok((page_count, total_bytes)) => {
                if page_count > MAX_RENDER_PAGES {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "Jumlah halaman hasil render melebihi batas maksimum ({MAX_RENDER_PAGES})"
                    ));
                }

                if total_bytes > MAX_TOTAL_OUTPUT_BYTES as u64 {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "Ukuran total hasil render melebihi batas maksimum ({} MiB)",
                        MAX_TOTAL_OUTPUT_BYTES / 1024 / 1024
                    ));
                }
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }

                let stderr = read_limited_stderr(stderr_path).unwrap_or_default();
                return Err(format!(
                    "pdftocairo gagal (exit code {:?}): {stderr}",
                    status.code()
                ));
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "pdftocairo melebihi batas waktu {} detik",
                    RENDER_TIMEOUT.as_secs()
                ));
            }
            Ok(None) => thread::sleep(PROCESS_POLL_INTERVAL),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Gagal memantau pdftocairo: {error}"));
            }
        }
    }
}

fn rendered_output_usage(dir: &Path) -> Result<(usize, u64), String> {
    let mut page_count = 0usize;
    let mut total_bytes = 0u64;

    for entry in
        fs::read_dir(dir).map_err(|error| format!("Gagal membaca temporary directory: {error}"))?
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
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Gagal membaca metadata hasil render JPG: {error}"))?;
        if !metadata.file_type().is_file() {
            return Err("Hasil render JPG bukan regular file".to_owned());
        }
        let size = metadata.len();

        if size > MAX_OUTPUT_FILE_SIZE_BYTES as u64 {
            return Err(format!(
                "Ukuran satu hasil JPG melebihi batas maksimum ({} MiB)",
                MAX_OUTPUT_FILE_SIZE_BYTES / 1024 / 1024
            ));
        }

        total_bytes = total_bytes
            .checked_add(size)
            .ok_or_else(|| "Ukuran total hasil render melebihi kapasitas numerik".to_owned())?;
    }

    Ok((page_count, total_bytes))
}

fn read_limited_stderr(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::with_capacity(MAX_STDERR_BYTES + 1);
    file.seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
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
