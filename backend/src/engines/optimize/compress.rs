use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use lopdf::Document;
use tempfile::tempdir;

use super::common::validate_input;

const PYTHON_SCRIPT_RELATIVE: &str = "scripts/compress_pdf/compressor.py";
const QPDF_CANDIDATES: [&str; 2] = ["qpdf", "/usr/bin/qpdf"];
const GHOSTSCRIPT_CANDIDATES: [&str; 3] = ["gs", "ghostscript", "/usr/bin/gs"];
const MIN_PRIMARY_REDUCTION_PERCENT: usize = 5;
const MAX_INPUT_BYTES: usize = 500 * 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024 * 1024;
const MAX_INPUT_PAGES: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionQuality {
    High,
    Medium,
    Low,
}

impl CompressionQuality {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

pub fn compress_pdf(pdf_bytes: &[u8], quality: CompressionQuality) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    if pdf_bytes.len() > MAX_INPUT_BYTES {
        return Err(format!(
            "Ukuran PDF melebihi batas maksimum ({} MiB)",
            MAX_INPUT_BYTES / 1024 / 1024
        ));
    }

    let input_document = Document::load_mem(pdf_bytes)
        .map_err(|error| format!("Gagal membaca PDF sebelum kompresi: {error}"))?;
    let input_pages = input_document.get_pages().len();

    if input_pages == 0 {
        return Err("PDF tidak memiliki halaman.".to_owned());
    }

    if input_pages > MAX_INPUT_PAGES {
        return Err(format!(
            "Jumlah halaman PDF melebihi batas maksimum ({MAX_INPUT_PAGES})"
        ));
    }

    let temp_dir =
        tempdir().map_err(|error| format!("Gagal membuat direktori sementara: {error}"))?;
    let input_path = temp_dir.path().join("input.pdf");

    fs::write(&input_path, pdf_bytes)
        .map_err(|error| format!("Gagal menulis PDF sementara: {error}"))?;

    if let Some(python) = resolve_python()
        && let Some(script) = resolve_python_script()
    {
        let output_path = temp_dir
            .path()
            .join(format!("pymupdf-{}.pdf", quality.as_str()));

        match run_pymupdf(&python, &script, quality, &input_path, &output_path) {
            Ok(output) if is_valid_pdf(&output, input_pages) => {
                if has_meaningful_reduction(output.len(), pdf_bytes.len()) {
                    return Ok(output);
                }

                if output.len() < pdf_bytes.len() {
                    match run_fallback(
                        pdf_bytes,
                        quality,
                        &input_path,
                        temp_dir.path(),
                        input_pages,
                    ) {
                        Some(fallback) if fallback.len() < output.len() => return Ok(fallback),
                        _ => return Ok(output),
                    }
                }
            }
            Ok(_) | Err(_) => {}
        }
    }

    if let Some(fallback) = run_fallback(
        pdf_bytes,
        quality,
        &input_path,
        temp_dir.path(),
        input_pages,
    ) {
        return Ok(fallback);
    }

    Ok(pdf_bytes.to_vec())
}

fn run_fallback(
    original_bytes: &[u8],
    quality: CompressionQuality,
    input_path: &Path,
    temp_dir: &Path,
    expected_pages: usize,
) -> Option<Vec<u8>> {
    let output = match quality {
        CompressionQuality::High => {
            let qpdf = resolve_program(&QPDF_CANDIDATES)?;
            run_qpdf(&qpdf, input_path, temp_dir).ok()?
        }
        CompressionQuality::Medium => {
            let gs = resolve_program(&GHOSTSCRIPT_CANDIDATES)?;
            run_ghostscript(&gs, input_path, temp_dir, GhostscriptProfile::Medium).ok()?
        }
        CompressionQuality::Low => {
            let gs = resolve_program(&GHOSTSCRIPT_CANDIDATES)?;
            run_ghostscript(&gs, input_path, temp_dir, GhostscriptProfile::Low).ok()?
        }
    };

    if output.len() > MAX_OUTPUT_BYTES
        || !is_valid_pdf(&output, expected_pages)
        || output.len() >= original_bytes.len()
    {
        return None;
    }

    Some(output)
}

#[derive(Debug, Clone, Copy)]
enum GhostscriptProfile {
    Medium,
    Low,
}

impl GhostscriptProfile {
    fn output_name(self) -> &'static str {
        match self {
            Self::Medium => "ghostscript-medium.pdf",
            Self::Low => "ghostscript-low.pdf",
        }
    }

    fn color_dpi(self) -> u32 {
        match self {
            Self::Medium => 150,
            Self::Low => 110,
        }
    }

    fn gray_dpi(self) -> u32 {
        match self {
            Self::Medium => 150,
            Self::Low => 110,
        }
    }

    fn mono_dpi(self) -> u32 {
        match self {
            Self::Medium => 300,
            Self::Low => 240,
        }
    }

    fn jpeg_quality(self) -> u32 {
        match self {
            Self::Medium => 80,
            Self::Low => 60,
        }
    }
}

fn run_pymupdf(
    python: &Path,
    script: &Path,
    quality: CompressionQuality,
    input_path: &Path,
    output_path: &Path,
) -> Result<Vec<u8>, String> {
    let result = Command::new(python)
        .arg(script)
        .arg(quality.as_str())
        .arg(input_path)
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("Gagal menjalankan PyMuPDF: {error}"))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!(
            "PyMuPDF gagal pada mode `{}`:\n{}",
            quality.as_str(),
            stderr
        ));
    }

    fs::read(output_path).map_err(|error| format!("Gagal membaca output PyMuPDF: {error}"))
}

fn run_qpdf(qpdf: &Path, input_path: &Path, temp_dir: &Path) -> Result<Vec<u8>, String> {
    let output_path = temp_dir.join("qpdf-output.pdf");
    let result = Command::new(qpdf)
        .arg("--object-streams=generate")
        .arg("--compress-streams=y")
        .arg("--recompress-flate")
        .arg("--compression-level=9")
        .arg("--optimize-images")
        .arg("--jpeg-quality=75")
        .arg(input_path)
        .arg(&output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("Gagal menjalankan qpdf: {error}"))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("qpdf gagal:\n{stderr}"));
    }

    fs::read(&output_path).map_err(|error| format!("Gagal membaca hasil qpdf: {error}"))
}

fn run_ghostscript(
    gs: &Path,
    input_path: &Path,
    temp_dir: &Path,
    profile: GhostscriptProfile,
) -> Result<Vec<u8>, String> {
    let output_path = temp_dir.join(profile.output_name());
    let qfactor = profile.jpeg_quality() as f32 / 100.0;
    let image_dict = format!(
        "<< /QFactor {qfactor} \
           /Blend 1 \
           /HSamples [2 1 1 2] \
           /VSamples [2 1 1 2] >>"
    );

    let result = Command::new(gs)
        .arg("-dSAFER")
        .arg("-dBATCH")
        .arg("-dNOPAUSE")
        .arg("-sDEVICE=pdfwrite")
        .arg("-dCompatibilityLevel=1.7")
        .arg("-dDetectDuplicateImages=true")
        .arg("-dCompressPages=true")
        .arg("-dWriteXRefStm=true")
        .arg("-dWriteObjStms=true")
        .arg("-dDownsampleColorImages=true")
        .arg("-dDownsampleGrayImages=true")
        .arg("-dDownsampleMonoImages=true")
        .arg("-dColorImageDownsampleType=/Bicubic")
        .arg("-dGrayImageDownsampleType=/Bicubic")
        .arg("-dMonoImageDownsampleType=/Subsample")
        .arg(format!("-dColorImageResolution={}", profile.color_dpi()))
        .arg(format!("-dGrayImageResolution={}", profile.gray_dpi()))
        .arg(format!("-dMonoImageResolution={}", profile.mono_dpi()))
        .arg(format!("-dColorImageDict={image_dict}"))
        .arg(format!("-dGrayImageDict={image_dict}"))
        .arg(format!("-dMonoImageDict={image_dict}"))
        .arg("-dAutoFilterColorImages=true")
        .arg("-dAutoFilterGrayImages=true")
        .arg("-dEncodeColorImages=true")
        .arg("-dEncodeGrayImages=true")
        .arg("-dEncodeMonoImages=true")
        .arg("-dPreserveAnnots=true")
        .arg("-dPreserveOverprintSettings=true")
        .arg("-dPreserveEPSInfo=true")
        .arg("-dMaxInlineImageSize=0")
        .arg(format!("-sOutputFile={}", output_path.display()))
        .arg(input_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("Gagal menjalankan Ghostscript: {error}"))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Ghostscript gagal:\n{stderr}"));
    }

    fs::read(&output_path).map_err(|error| format!("Gagal membaca hasil Ghostscript: {error}"))
}

fn has_meaningful_reduction(output_size: usize, input_size: usize) -> bool {
    if output_size >= input_size {
        return false;
    }

    let reduction = input_size.saturating_sub(output_size);
    reduction.saturating_mul(100) >= input_size.saturating_mul(MIN_PRIMARY_REDUCTION_PERCENT)
}

fn is_valid_pdf(bytes: &[u8], expected_pages: usize) -> bool {
    if bytes.is_empty() || bytes.len() > MAX_OUTPUT_BYTES {
        return false;
    }

    let document = match Document::load_mem(bytes) {
        Ok(document) => document,
        Err(_) => return false,
    };

    let pages = document.get_pages();
    !pages.is_empty() && pages.len() == expected_pages
}

fn resolve_python_script() -> Option<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir.join(PYTHON_SCRIPT_RELATIVE);
    script.is_file().then_some(script)
}

fn resolve_python() -> Option<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let venv_python = manifest_dir.join(".venv/bin/python");

    if venv_python.is_file() && python_has_pymupdf(&venv_python) {
        return Some(venv_python);
    }

    let candidates = ["python3", "python", "/usr/bin/python3"];

    for candidate in candidates {
        let path = PathBuf::from(candidate);

        if path.is_absolute() && !path.is_file() {
            continue;
        }

        if python_has_pymupdf(&path) {
            return Some(path);
        }
    }

    None
}

fn python_has_pymupdf(python: &Path) -> bool {
    Command::new(python)
        .arg("-c")
        .arg("import pymupdf")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn resolve_program(candidates: &[&str]) -> Option<PathBuf> {
    for candidate in candidates {
        let path = PathBuf::from(candidate);

        if path.is_absolute() {
            if path.is_file() {
                return Some(path);
            }
            continue;
        }

        let status = Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if let Ok(status) = status
            && status.success()
        {
            return Some(path);
        }
    }

    None
}
