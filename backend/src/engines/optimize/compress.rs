use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use lopdf::Document;
use tempfile::tempdir;

use super::common::validate_input;

const PYTHON_SCRIPT_RELATIVE: &str = "scripts/compress_pdf/compressor.py";

const QPDF_CANDIDATES: [&str; 2] = ["qpdf", "/usr/bin/qpdf"];

const GHOSTSCRIPT_CANDIDATES: [&str; 3] = ["gs", "ghostscript", "/usr/bin/gs"];

const MIN_PRIMARY_REDUCTION_PERCENT: usize = 5;
const SUBPROCESS_TIMEOUT: Duration = Duration::from_secs(90);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(20);

static PYTHON_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static QPDF_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static GHOSTSCRIPT_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

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

/// Mengompresi PDF menggunakan pipeline yang dipilih
/// berdasarkan kualitas.
pub fn compress_pdf(pdf_bytes: &[u8], quality: CompressionQuality) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    if pdf_bytes.is_empty() {
        return Err("PDF kosong tidak dapat dikompresi.".to_owned());
    }

    let input_document = Document::load_mem(pdf_bytes)
        .map_err(|error| format!("Gagal membaca PDF sebelum kompresi: {error}"))?;

    let input_pages = input_document.get_pages().len();

    if input_pages == 0 {
        return Err("PDF tidak memiliki halaman.".to_owned());
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
                        Some(fallback) if fallback.len() < output.len() => {
                            return Ok(fallback);
                        }

                        _ => {
                            return Ok(output);
                        }
                    }
                }
            }

            Ok(_) => {}

            Err(_) => {}
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
    match quality {
        CompressionQuality::High => {
            let qpdf = resolve_qpdf()?;
            let output = run_qpdf(&qpdf, input_path, temp_dir).ok()?;

            if !is_valid_pdf(&output, expected_pages) || output.len() >= original_bytes.len() {
                return None;
            }

            Some(output)
        }

        CompressionQuality::Medium => {
            let gs = resolve_ghostscript()?;
            let output =
                run_ghostscript(&gs, input_path, temp_dir, GhostscriptProfile::Medium).ok()?;

            if !is_valid_pdf(&output, expected_pages) || output.len() >= original_bytes.len() {
                return None;
            }

            Some(output)
        }

        CompressionQuality::Low => {
            let gs = resolve_ghostscript()?;
            let output =
                run_ghostscript(&gs, input_path, temp_dir, GhostscriptProfile::Low).ok()?;

            if !is_valid_pdf(&output, expected_pages) || output.len() >= original_bytes.len() {
                return None;
            }

            Some(output)
        }
    }
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
    let mut command = Command::new(python);

    command
        .arg(script)
        .arg(quality.as_str())
        .arg(input_path)
        .arg(output_path);

    let result = run_command_with_timeout(&mut command, "PyMuPDF")?;

    if !result.success {
        return Err(format!("PyMuPDF gagal pada mode `{}`", quality.as_str()));
    }

    fs::read(output_path).map_err(|error| format!("Gagal membaca output PyMuPDF: {error}"))
}

fn run_qpdf(qpdf: &Path, input_path: &Path, temp_dir: &Path) -> Result<Vec<u8>, String> {
    let output_path = temp_dir.join("qpdf-output.pdf");
    let mut command = Command::new(qpdf);

    command
        .arg("--object-streams=generate")
        .arg("--compress-streams=y")
        .arg("--recompress-flate")
        .arg("--compression-level=9")
        .arg("--optimize-images")
        .arg("--jpeg-quality=75")
        .arg(input_path)
        .arg(&output_path);

    let result = run_command_with_timeout(&mut command, "qpdf")?;

    if !result.success {
        return Err("qpdf gagal".to_owned());
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

    let mut command = Command::new(gs);

    command
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
        .arg(input_path);

    let result = run_command_with_timeout(&mut command, "Ghostscript")?;

    if !result.success {
        return Err("Ghostscript gagal".to_owned());
    }

    fs::read(&output_path).map_err(|error| format!("Gagal membaca hasil Ghostscript: {error}"))
}

struct CommandResult {
    success: bool,
}

fn run_command_with_timeout(
    command: &mut Command,
    name: &str,
) -> Result<CommandResult, String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Gagal menjalankan {name}: {error}"))?;

    wait_for_child(&mut child, name)
}

fn wait_for_child(child: &mut Child, name: &str) -> Result<CommandResult, String> {
    let deadline = Instant::now() + SUBPROCESS_TIMEOUT;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok(CommandResult {
                    success: status.success(),
                });
            }

            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();

                return Err(format!("{name} melebihi batas waktu pemrosesan"));
            }

            Ok(None) => thread::sleep(PROCESS_POLL_INTERVAL),

            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();

                return Err(format!("Gagal memeriksa proses {name}: {error}"));
            }
        }
    }
}

fn has_meaningful_reduction(output_size: usize, input_size: usize) -> bool {
    if output_size >= input_size {
        return false;
    }

    let reduction = input_size.saturating_sub(output_size);

    reduction.saturating_mul(100) >= input_size.saturating_mul(MIN_PRIMARY_REDUCTION_PERCENT)
}

fn is_valid_pdf(bytes: &[u8], expected_pages: usize) -> bool {
    if bytes.is_empty() {
        return false;
    }

    let document = match Document::load_mem(bytes) {
        Ok(document) => document,
        Err(_) => return false,
    };

    document.get_pages().len() == expected_pages
}

fn resolve_python_script() -> Option<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir.join(PYTHON_SCRIPT_RELATIVE);

    script.is_file().then_some(script)
}

fn resolve_python() -> Option<PathBuf> {
    PYTHON_PATH
        .get_or_init(resolve_python_uncached)
        .clone()
}

fn resolve_python_uncached() -> Option<PathBuf> {
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
    let mut command = Command::new(python);

    command
        .arg("-c")
        .arg("import pymupdf")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command.status().is_ok_and(|status| status.success())
}

fn resolve_qpdf() -> Option<PathBuf> {
    QPDF_PATH.get_or_init(|| resolve_program(&QPDF_CANDIDATES)).clone()
}

fn resolve_ghostscript() -> Option<PathBuf> {
    GHOSTSCRIPT_PATH
        .get_or_init(|| resolve_program(&GHOSTSCRIPT_CANDIDATES))
        .clone()
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

        let mut command = Command::new(candidate);
        command
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if let Ok(status) = command.status()
            && status.success()
        {
            return Some(path);
        }
    }

    None
}
