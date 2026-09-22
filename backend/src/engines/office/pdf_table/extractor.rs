use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use super::model::PdfWord;

const WORD_GAP_FACTOR: f32 = 0.55;
const MIN_WORD_GAP: f32 = 1.5;
const MAX_WORD_GAP: f32 = 12.0;
const ROW_TOLERANCE_FACTOR: f32 = 0.45;
const MIN_ROW_TOLERANCE: f32 = 2.0;
const MAX_ROW_TOLERANCE: f32 = 8.0;

const MAX_PDFIUM_INPUT_BYTES: usize = 100 * 1024 * 1024;
const MAX_EXTRACTION_PAGES: usize = 1_000;
const MAX_WORDS_PER_PAGE: usize = 50_000;
const MAX_WORDS_PER_DOCUMENT: usize = 250_000;
const MAX_WORD_TEXT_BYTES: usize = 64 * 1024;
const MAX_TOTAL_WORD_TEXT_BYTES: usize = 64 * 1024 * 1024;
const MAX_PDFIUM_OBJECTS: usize = 750_000;

const PDFIUM_WORKER_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_PDFIUM_WORKER_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_PDFIUM_WORKER_STDERR_BYTES: usize = 16 * 1024;

fn append_word_char(
    text: &mut String,
    value: char,
    total_text_bytes: &mut usize,
) -> Result<(), String> {
    let value_len = value.len_utf8();
    let next_len = text.len().saturating_add(value_len);
    if next_len > MAX_WORD_TEXT_BYTES {
        return Err(format!(
            "Text word PDF melebihi batas maksimum ({} KiB)",
            MAX_WORD_TEXT_BYTES / 1024
        ));
    }

    *total_text_bytes = total_text_bytes
        .checked_add(value_len)
        .ok_or_else(|| "Total text PDF terlalu besar".to_owned())?;
    if *total_text_bytes > MAX_TOTAL_WORD_TEXT_BYTES {
        return Err(format!(
            "Total text hasil ekstraksi PDF melebihi batas maksimum ({} MiB)",
            MAX_TOTAL_WORD_TEXT_BYTES / 1024 / 1024
        ));
    }

    text.push(value);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct PdfWordWire {
    text: String,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

pub fn extract_pdf_words(pdf_bytes: &[u8]) -> Result<Vec<Vec<PdfWord>>, String> {
    #[cfg(test)]
    {
        return extract_pdf_words_in_process(pdf_bytes);
    }

    #[cfg(not(test))]
    {
        extract_pdf_words_via_worker(pdf_bytes)
    }
}

#[cfg(not(test))]
fn extract_pdf_words_via_worker(pdf_bytes: &[u8]) -> Result<Vec<Vec<PdfWord>>, String> {
    let expected_pages = validate_pdfium_input(pdf_bytes)?;

    let temp_dir = tempfile::tempdir()
        .map_err(|error| format!("Gagal membuat temporary directory PDFium: {error}"))?;
    let input_path = temp_dir.path().join("input.pdf");
    let output_path = temp_dir.path().join("output.json");
    let stderr_path = temp_dir.path().join("worker.stderr");

    fs::write(&input_path, pdf_bytes)
        .map_err(|error| format!("Gagal menulis input worker PDFium: {error}"))?;

    let executable = std::env::current_exe()
        .map_err(|error| format!("Gagal menemukan executable worker PDFium: {error}"))?;
    let executable_str = executable
        .to_str()
        .ok_or_else(|| "Path executable worker PDFium bukan UTF-8 yang valid.".to_owned())?;
    let executable_dir = executable
        .parent()
        .ok_or_else(|| "Direktori executable worker PDFium tidak valid.".to_owned())?;

    let pdfium_path = pdfium_bundled::ensure_pdfium_library(None)
        .map_err(|error| format!("Gagal menyiapkan library PDFium: {error}"))?;

    let stderr_file = File::create(&stderr_path)
        .map_err(|error| format!("Gagal membuat log worker PDFium: {error}"))?;

    let mut child = crate::engines::sandbox::command_with_read_only_paths_and_env(
        executable_str,
        temp_dir.path(),
        &[executable_dir, pdfium_path.as_path()],
        &[("PDFIUM_LIB_PATH", pdfium_path.as_os_str())],
    )?
    .arg("--pdfium-worker")
    .arg(&input_path)
    .arg(&output_path)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::from(stderr_file))
    .spawn()
    .map_err(|error| format!("Gagal menjalankan worker PDFium: {error}"))?;

    let deadline = Instant::now() + PDFIUM_WORKER_TIMEOUT;
    loop {
        if file_size_exceeds(&output_path, MAX_PDFIUM_WORKER_OUTPUT_BYTES) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "Output worker PDFium melebihi batas maksimum ({} MiB)",
                MAX_PDFIUM_WORKER_OUTPUT_BYTES / 1024 / 1024
            ));
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let stderr = read_limited_file(&stderr_path, MAX_PDFIUM_WORKER_STDERR_BYTES)
                        .unwrap_or_default();
                    return Err(format!(
                        "Worker PDFium gagal dengan status {status}: {}",
                        String::from_utf8_lossy(&stderr).trim()
                    ));
                }
                break;
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "Worker PDFium melebihi batas waktu {} detik",
                    PDFIUM_WORKER_TIMEOUT.as_secs()
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Gagal memantau worker PDFium: {error}"));
            }
        }
    }

    let output = read_limited_file(&output_path, MAX_PDFIUM_WORKER_OUTPUT_BYTES)?;
    let pages: Vec<Vec<PdfWordWire>> = serde_json::from_slice(&output)
        .map_err(|error| format!("Output worker PDFium tidak valid: {error}"))?;

    if pages.len() != expected_pages {
        return Err(format!(
            "Worker PDFium mengembalikan jumlah halaman {} dari yang diharapkan {}",
            pages.len(),
            expected_pages
        ));
    }

    let total_words: usize = pages.iter().map(Vec::len).sum();
    if total_words > MAX_WORDS_PER_DOCUMENT {
        return Err(format!(
            "Jumlah total word PDFium melebihi batas maksimum ({MAX_WORDS_PER_DOCUMENT})"
        ));
    }

    Ok(pages
        .into_iter()
        .map(|page| {
            page.into_iter()
                .map(|word| PdfWord::new(word.text, word.left, word.right, word.top, word.bottom))
                .collect()
        })
        .collect())
}

pub fn run_pdfium_worker(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let pdf_bytes = read_limited_file(input_path, MAX_PDFIUM_INPUT_BYTES)?;
    validate_pdfium_input(&pdf_bytes)?;

    let pages = extract_pdf_words_in_process(&pdf_bytes)?;
    if let Ok(metadata) = fs::symlink_metadata(output_path) {
        if !metadata.file_type().is_file() {
            return Err("Output worker PDFium bukan regular file".to_owned());
        }
    }

    let output_file = File::create(output_path)
        .map_err(|error| format!("Gagal membuat output worker PDFium: {error}"))?;
    let mut output_file = std::io::BufWriter::new(output_file);

    let wire_pages: Vec<Vec<PdfWordWire>> = pages
        .into_iter()
        .map(|page| {
            page.into_iter()
                .map(|word| PdfWordWire {
                    text: word.text,
                    left: word.left,
                    right: word.right,
                    top: word.top,
                    bottom: word.bottom,
                })
                .collect()
        })
        .collect();

    serde_json::to_writer(&mut output_file, &wire_pages)
        .map_err(|error| format!("Gagal menulis output worker PDFium: {error}"))?;
    output_file
        .flush()
        .map_err(|error| format!("Gagal flush output worker PDFium: {error}"))?;

    if file_size_exceeds(output_path, MAX_PDFIUM_WORKER_OUTPUT_BYTES) {
        return Err(format!(
            "Output worker PDFium melebihi batas maksimum ({} MiB)",
            MAX_PDFIUM_WORKER_OUTPUT_BYTES / 1024 / 1024
        ));
    }

    Ok(())
}

fn validate_pdfium_input(pdf_bytes: &[u8]) -> Result<usize, String> {
    if pdf_bytes.len() > MAX_PDFIUM_INPUT_BYTES {
        return Err(format!(
            "PDF terlalu besar untuk ekstraksi PDFium (maksimum {} MiB)",
            MAX_PDFIUM_INPUT_BYTES / 1024 / 1024
        ));
    }

    let preflight = crate::engines::pdf::common::load_pdf_document(pdf_bytes)
        .map_err(|error| format!("Gagal membaca PDF sebelum ekstraksi PDFium: {error}"))?;

    let page_count = preflight.get_pages().len();
    if page_count > MAX_EXTRACTION_PAGES {
        return Err(format!(
            "Jumlah halaman PDF melebihi batas ekstraksi ({MAX_EXTRACTION_PAGES})"
        ));
    }

    if preflight.objects.len() > MAX_PDFIUM_OBJECTS {
        return Err(format!(
            "Jumlah object PDF melebihi batas ekstraksi PDFium ({MAX_PDFIUM_OBJECTS})"
        ));
    }

    crate::engines::pdf::common::validate_pdf_page_dimensions(&preflight)
        .map_err(|error| format!("PDF ditolak sebelum ekstraksi PDFium: {error}"))?;

    Ok(page_count)
}

fn file_size_exceeds(path: &Path, limit: usize) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.len() > limit as u64)
        .unwrap_or(false)
}

fn read_limited_file(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Gagal membaca metadata file worker {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "File worker {} bukan regular file",
            path.display()
        ));
    }

    let mut file = File::open(path)
        .map_err(|error| format!("Gagal membaca file worker {}: {error}", path.display()))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("Gagal seek file worker: {error}"))?;

    let mut bytes = Vec::new();
    let limit_u64 = limit as u64;
    let mut limited = (&mut file).take(limit_u64.saturating_add(1));
    limited
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Gagal membaca file worker: {error}"))?;

    if bytes.len() > limit {
        return Err(format!(
            "File worker melebihi batas maksimum ({} MiB)",
            limit / 1024 / 1024
        ));
    }

    Ok(bytes)
}

fn extract_pdf_words_in_process(pdf_bytes: &[u8]) -> Result<Vec<Vec<PdfWord>>, String> {
    if pdf_bytes.len() > MAX_PDFIUM_INPUT_BYTES {
        return Err(format!(
            "PDF terlalu besar untuk ekstraksi PDFium (maksimum {} MiB)",
            MAX_PDFIUM_INPUT_BYTES / 1024 / 1024
        ));
    }

    let preflight = crate::engines::pdf::common::load_pdf_document(pdf_bytes)
        .map_err(|error| format!("Gagal membaca PDF sebelum ekstraksi PDFium: {error}"))?;

    let page_count = preflight.get_pages().len();
    if page_count > MAX_EXTRACTION_PAGES {
        return Err(format!(
            "Jumlah halaman PDF melebihi batas ekstraksi ({MAX_EXTRACTION_PAGES})"
        ));
    }

    if preflight.objects.len() > MAX_PDFIUM_OBJECTS {
        return Err(format!(
            "Jumlah object PDF melebihi batas ekstraksi PDFium ({MAX_PDFIUM_OBJECTS})"
        ));
    }

    crate::engines::pdf::common::validate_pdf_page_dimensions(&preflight)
        .map_err(|error| format!("PDF ditolak sebelum ekstraksi PDFium: {error}"))?;

    drop(preflight);

    let pdfium = pdfium_bundled::bind_pdfium_silent()
        .map_err(|error| format!("Gagal memuat PDFium: {error}"))?;

    let document = pdfium
        .load_pdf_from_byte_slice(pdf_bytes, None)
        .map_err(|error| format!("Gagal membuka PDF dengan PDFium: {error}"))?;

    let document_page_count = document.pages().len() as usize;

    if document_page_count > MAX_EXTRACTION_PAGES {
        return Err(format!(
            "Jumlah halaman PDF melebihi batas ekstraksi ({MAX_EXTRACTION_PAGES})"
        ));
    }

    let mut pages = Vec::with_capacity(document_page_count);
    let mut total_words = 0usize;

    for (page_index, page) in document.pages().iter().enumerate() {
        let text = page.text().map_err(|error| {
            format!(
                "Gagal membaca text PDF pada halaman {}: {error}",
                page_index + 1
            )
        })?;

        if text.is_empty() {
            pages.push(Vec::new());
            continue;
        }

        /*
         * PENTING: koordinat native PDFium memakai origin kiri-bawah
         * dengan sumbu Y mengarah ke ATAS halaman. Artinya
         * `bounds.top()` justru bernilai LEBIH BESAR daripada
         * `bounds.bottom()` (kebalikan dari koordinat layar/screen
         * yang dipakai di seluruh kode ini, termasuk PdfWord::height()
         * di model.rs dan pengurutan baris berdasarkan `top` di
         * detector.rs).
         *
         * Tanpa konversi ini:
         * - PdfWord::height() selalu clamp ke 0.0 (bottom - top < 0),
         *   sehingga semua toleransi adaptif (row/word gap) collapse
         *   ke nilai minimum, tidak peduli ukuran font sebenarnya.
         * - Baris/kata diurutkan dari BAWAH ke ATAS halaman, sehingga
         *   output tabel/baris jadi terbalik.
         *
         * Kita konversi ke koordinat top-down (Y=0 di atas halaman)
         * di sini, sekali per halaman, supaya seluruh kode di bawahnya
         * (extractor + detector + model) bisa konsisten menganggap
         * top < bottom seperti koordinat layar biasa.
         */
        // NOTE: sesuaikan pemanggilan ini dengan API pdfium-render versi
        // yang lo pakai kalau nama methodnya beda — intinya kita cuma
        // butuh tinggi halaman dalam PdfPoints. Di versi pdfium-render
        // yang umum dipakai sekarang ini tersedia lewat `page.page_size()`.
        let page_height = page.page_size().height().value;

        let chars = text.chars();

        let mut words = Vec::new();
        let mut total_text_bytes = 0usize;

        let mut current_text = String::new();

        let mut current_left = 0.0_f32;
        let mut current_right = 0.0_f32;
        let mut current_top = 0.0_f32;
        let mut current_bottom = 0.0_f32;

        let mut previous_right: Option<f32> = None;
        let mut previous_top: Option<f32> = None;
        let mut previous_height: Option<f32> = None;

        for character in chars.iter() {
            let Some(value) = character.unicode_char() else {
                continue;
            };

            let bounds = match character.tight_bounds() {
                Ok(bounds) => bounds,
                Err(_) => {
                    /*
                     * Bounds gagal diambil (mis. karakter kontrol/
                     * invisible). Sebelumnya karakter ini di-skip total
                     * dengan `continue`, artinya HILANG dari teks yang
                     * diekstrak tanpa jejak. Kita tetap masukkan
                     * karakternya ke word yang sedang berjalan supaya
                     * teksnya tidak diam-diam corrupt, walau posisinya
                     * tidak ikut memperluas bounding box.
                     */
                    if !current_text.is_empty() {
                        append_word_char(&mut current_text, value, &mut total_text_bytes)?;
                    }

                    continue;
                }
            };

            let left = bounds.left().value;
            let right = bounds.right().value;

            // Konversi dari koordinat native PDFium (Y ke atas, origin
            // kiri-bawah) ke koordinat top-down (Y ke bawah, origin
            // kiri-atas) supaya `top < bottom` konsisten dengan asumsi
            // di model.rs dan detector.rs.
            let top = page_height - bounds.top().value;
            let bottom = page_height - bounds.bottom().value;

            let height = (bottom - top).abs().max(0.1);

            /*
             * PDFium bisa mengembalikan whitespace dan newline
             * sebagai character.
             */
            if value.is_whitespace() {
                flush_word(
                    &mut words,
                    &mut current_text,
                    &mut current_left,
                    &mut current_right,
                    &mut current_top,
                    &mut current_bottom,
                );

                previous_right = None;
                previous_top = None;
                previous_height = None;

                continue;
            }

            /*
             * Kalau karakter sebelumnya tidak ada,
             * langsung mulai word baru.
             */
            let Some(previous_right_value) = previous_right else {
                append_word_char(&mut current_text, value, &mut total_text_bytes)?;

                current_left = left;
                current_right = right;
                current_top = top;
                current_bottom = bottom;

                previous_right = Some(right);
                previous_top = Some(top);
                previous_height = Some(height);

                continue;
            };

            let previous_height_value = previous_height.unwrap_or(height);

            /*
             * Tolerance vertikal dibuat relatif terhadap ukuran font.
             */
            let row_tolerance = (previous_height_value * ROW_TOLERANCE_FACTOR)
                .clamp(MIN_ROW_TOLERANCE, MAX_ROW_TOLERANCE);

            let same_line = previous_top
                .map(|previous| (previous - top).abs() <= row_tolerance)
                .unwrap_or(true);

            let horizontal_gap = left - previous_right_value;

            /*
             * Jangan memakai angka fixed 5 px.
             *
             * Ukuran font PDF bisa berbeda jauh.
             */
            let word_gap =
                (previous_height_value * WORD_GAP_FACTOR).clamp(MIN_WORD_GAP, MAX_WORD_GAP);

            let new_word = !same_line || horizontal_gap > word_gap;

            if new_word {
                flush_word(
                    &mut words,
                    &mut current_text,
                    &mut current_left,
                    &mut current_right,
                    &mut current_top,
                    &mut current_bottom,
                );

                append_word_char(&mut current_text, value, &mut total_text_bytes)?;

                current_left = left;
                current_right = right;
                current_top = top;
                current_bottom = bottom;
            } else {
                append_word_char(&mut current_text, value, &mut total_text_bytes)?;

                current_right = current_right.max(right);
                current_top = current_top.min(top);
                current_bottom = current_bottom.max(bottom);
            }

            if words.len() >= MAX_WORDS_PER_PAGE {
                return Err(format!(
                    "Jumlah word pada halaman {} melebihi batas maksimum ({MAX_WORDS_PER_PAGE})",
                    page_index + 1
                ));
            }

            previous_right = Some(right);
            previous_top = Some(top);
            previous_height = Some(height);
        }

        flush_word(
            &mut words,
            &mut current_text,
            &mut current_left,
            &mut current_right,
            &mut current_top,
            &mut current_bottom,
        );

        if words.len() > MAX_WORDS_PER_PAGE {
            return Err(format!(
                "Jumlah word pada halaman {} melebihi batas maksimum ({MAX_WORDS_PER_PAGE})",
                page_index + 1
            ));
        }

        total_words = total_words
            .checked_add(words.len())
            .ok_or_else(|| "Jumlah word PDF terlalu besar".to_owned())?;
        if total_words > MAX_WORDS_PER_DOCUMENT {
            return Err(format!(
                "Jumlah total word PDF melebihi batas maksimum ({MAX_WORDS_PER_DOCUMENT})"
            ));
        }

        tracing::debug!(
            page = page_index + 1,
            words = words.len(),
            "PDFium text extraction selesai"
        );

        pages.push(words);
    }

    Ok(pages)
}

fn flush_word(
    words: &mut Vec<PdfWord>,
    text: &mut String,
    left: &mut f32,
    right: &mut f32,
    top: &mut f32,
    bottom: &mut f32,
) {
    let value = text.trim();

    if !value.is_empty() {
        words.push(PdfWord::new(value.to_owned(), *left, *right, *top, *bottom));
    }

    text.clear();

    *left = 0.0;
    *right = 0.0;
    *top = 0.0;
    *bottom = 0.0;
}
