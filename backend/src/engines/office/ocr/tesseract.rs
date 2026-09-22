use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use tempfile::tempdir;

const MAX_TSV_BYTES: usize = 16 * 1024 * 1024;
const MAX_OCR_WORDS: usize = 50_000;
const MAX_OCR_TEXT_LENGTH: usize = 10_000;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const OCR_TIMEOUT: Duration = Duration::from_secs(120);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug, Clone)]
pub struct OcrTextItem {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub width: f64,
    pub height: f64,
}

/// Jalankan Tesseract menggunakan output TSV.
///
/// TSV memberikan koordinat setiap word:
///
/// level page block paragraph line word
/// left top width height confidence text
///
/// Koordinat ini kemudian dapat digunakan untuk
/// membangun kembali baris dan kolom Excel.
pub fn ocr_image(image_path: &Path) -> Result<Vec<OcrTextItem>, String> {
    let executable = tesseract_executable();
    let temp_dir = tempdir()
        .map_err(|error| format!("Gagal membuat temporary directory untuk Tesseract: {error}"))?;
    let stdout_path = temp_dir.path().join("output.tsv");
    let stderr_path = temp_dir.path().join("stderr.log");

    let stdout_file = File::create(&stdout_path)
        .map_err(|error| format!("Gagal membuat output sementara Tesseract: {error}"))?;
    let stderr_file = File::create(&stderr_path)
        .map_err(|error| format!("Gagal membuat log Tesseract: {error}"))?;

    let mut child = crate::engines::sandbox::command_with_read_only_paths(
        executable,
        temp_dir.path(),
        &[image_path],
    )?
    .arg(image_path)
        .arg("stdout")
        .arg("--psm")
        .arg("6")
        .arg("-l")
        .arg("eng")
        .arg("tsv")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|error| format!("Gagal menjalankan Tesseract: {error}"))?;

    let deadline = Instant::now() + OCR_TIMEOUT;
    loop {
        if file_size_exceeds(&stdout_path, MAX_TSV_BYTES) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "Output OCR terlalu besar (maksimum {} MiB)",
                MAX_TSV_BYTES / 1024 / 1024
            ));
        }

        if file_size_exceeds(&stderr_path, MAX_STDERR_BYTES) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Log error Tesseract melebihi batas maksimum".to_owned());
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let stderr =
                        read_limited_file(&stderr_path, MAX_STDERR_BYTES).unwrap_or_default();
                    return Err(format!(
                        "Tesseract gagal dengan status {status}: {}",
                        String::from_utf8_lossy(&stderr).trim()
                    ));
                }
                break;
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "Tesseract melebihi batas waktu {} detik",
                    OCR_TIMEOUT.as_secs()
                ));
            }
            Ok(None) => thread::sleep(PROCESS_POLL_INTERVAL),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Gagal memantau Tesseract: {error}"));
            }
        }
    }

    let tsv_bytes = read_limited_file(&stdout_path, MAX_TSV_BYTES)?;
    let tsv = String::from_utf8_lossy(&tsv_bytes);

    parse_tesseract_tsv(&tsv)
}

fn file_size_exceeds(path: &Path, limit: usize) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.len() > limit as u64)
        .unwrap_or(false)
}

fn read_limited_file(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;

    let mut bytes = Vec::with_capacity(limit.saturating_add(1));
    file.take(limit.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;

    if bytes.len() > limit {
        return Err(format!("File melebihi batas maksimum {limit} byte"));
    }

    Ok(bytes)
}

/* -------------------------------------------------------------------------- */
/* TSV PARSER                                                                 */
/* -------------------------------------------------------------------------- */

fn parse_tesseract_tsv(tsv: &str) -> Result<Vec<OcrTextItem>, String> {
    if tsv.len() > MAX_TSV_BYTES {
        return Err(format!(
            "Input TSV terlalu besar (maksimum {} MiB)",
            MAX_TSV_BYTES / 1024 / 1024
        ));
    }

    let mut items = Vec::new();

    for (line_number, line) in tsv.lines().enumerate() {
        /*
         * Baris pertama adalah header:
         *
         * level page_num block_num par_num line_num word_num
         * left top width height conf text
         */
        if line_number == 0 {
            continue;
        }

        let columns = split_tsv_line(line);

        if columns.len() < 12 {
            continue;
        }

        let level = columns[0].parse::<u32>().unwrap_or(0);

        /*
         * Level 5 = word.
         *
         * Kita hanya membutuhkan word agar koordinat
         * bisa digunakan untuk membangun cell.
         */
        if level != 5 {
            continue;
        }

        if items.len() >= MAX_OCR_WORDS {
            return Err(format!(
                "Jumlah word OCR melebihi batas maksimum ({MAX_OCR_WORDS})"
            ));
        }

        let left = match columns[6].parse::<f64>() {
            Ok(value) if value.is_finite() => value,
            _ => continue,
        };

        let top = match columns[7].parse::<f64>() {
            Ok(value) if value.is_finite() => value,
            _ => continue,
        };

        let width = match columns[8].parse::<f64>() {
            Ok(value) if value.is_finite() && value >= 0.0 => value,
            _ => continue,
        };

        let height = match columns[9].parse::<f64>() {
            Ok(value) if value.is_finite() && value >= 0.0 => value,
            _ => continue,
        };

        let confidence = columns[10].parse::<f64>().unwrap_or(-1.0);

        /*
         * Confidence terlalu rendah biasanya merupakan
         * noise OCR.
         *
         * -1 berarti Tesseract tidak memberikan confidence.
         */
        if !confidence.is_finite() || (0.0..20.0).contains(&confidence) {
            continue;
        }

        let text = columns[11].trim();

        if text.is_empty() {
            continue;
        }

        if text.len() > MAX_OCR_TEXT_LENGTH {
            return Err(format!(
                "Text OCR pada word melebihi batas maksimum ({MAX_OCR_TEXT_LENGTH} byte)"
            ));
        }

        items.push(OcrTextItem {
            x: left,
            y: top,
            text: text.to_owned(),
            width,
            height,
        });
    }

    Ok(items)
}

/// Split TSV sambil tetap menangani field kosong.
fn split_tsv_line(line: &str) -> Vec<&str> {
    line.split('\t').collect()
}

/* -------------------------------------------------------------------------- */
/* EXECUTABLE                                                                 */
/* -------------------------------------------------------------------------- */

fn tesseract_executable() -> &'static str {
    if cfg!(target_os = "windows") {
        "tesseract.exe"
    } else {
        "tesseract"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tesseract_should_be_available() {
        let output = Command::new(tesseract_executable())
            .arg("--version")
            .output()
            .expect("Tesseract tidak ditemukan");

        assert!(output.status.success());
    }

    #[test]
    fn parse_tsv_should_extract_words() {
        let tsv = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t50\t100\t80\t20\t95.2\tNama
5\t1\t1\t1\t1\t2\t150\t100\t60\t20\t94.1\tBarang
";

        let items = parse_tesseract_tsv(tsv).unwrap();

        assert_eq!(items.len(), 2);

        assert_eq!(items[0].text, "Nama");
        assert_eq!(items[1].text, "Barang");

        assert_eq!(items[0].x, 50.0);
        assert_eq!(items[1].x, 150.0);

        assert_eq!(items[0].y, 100.0);
        assert_eq!(items[1].y, 100.0);
    }
}
