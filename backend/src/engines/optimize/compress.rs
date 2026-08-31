use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use lopdf::Document;
use tempfile::tempdir;

use super::common::{load_pdf_document, validate_input};

const PYTHON_SCRIPT_RELATIVE: &str = "scripts/compress_pdf/compressor.py";
const QPDF_CANDIDATES: [&str; 2] = ["qpdf", "/usr/bin/qpdf"];
const GHOSTSCRIPT_CANDIDATES: [&str; 3] = ["gs", "ghostscript", "/usr/bin/gs"];
const MIN_PRIMARY_REDUCTION_PERCENT: usize = 5;
const MAX_INPUT_BYTES: usize = 500 * 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024 * 1024;
const MAX_INPUT_PAGES: usize = 10_000;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const PYMUPDF_TIMEOUT: Duration = Duration::from_secs(120);
const QPDF_TIMEOUT: Duration = Duration::from_secs(180);
const GHOSTSCRIPT_TIMEOUT: Duration = Duration::from_secs(300);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(25);

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

    let input_document = load_pdf_document(pdf_bytes)
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
        let stderr_path = temp_dir
            .path()
            .join(format!("pymupdf-{}.stderr", quality.as_str()));

        match run_pymupdf(
            &python,
            &script,
            quality,
            &input_path,
            &output_path,
            &stderr_path,
        ) {
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
    stderr_path: &Path,
) -> Result<Vec<u8>, String> {
    run_external_command(
        &mut Command::new(python),
        PYMUPDF_TIMEOUT,
        stderr_path,
        "PyMuPDF",
        &[
            script.as_os_str(),
            quality.as_str().as_ref(),
            input_path.as_os_str(),
            output_path.as_os_str(),
        ],
    )?;

    fs::read(output_path).map_err(|error| format!("Gagal membaca output PyMuPDF: {error}"))
}

fn run_qpdf(qpdf: &Path, input_path: &Path, temp_dir: &Path) -> Result<Vec<u8>, String> {
    let output_path = temp_dir.join("qpdf-output.pdf");
    let stderr_path = temp_dir.join("qpdf.stderr");

    run_external_command(
        &mut Command::new(qpdf),
        QPDF_TIMEOUT,
        &stderr_path,
        "qpdf",
        &[
            "--object-streams=generate".as_ref(),
            "--compress-streams=y".as_ref(),
            "--recompress-flate".as_ref(),
            "--compression-level=9".as_ref(),
            "--optimize-images".as_ref(),
            "--jpeg-quality=75".as_ref(),
            input_path.as_os_str(),
            output_path.as_os_str(),
        ],
    )?;

    fs::read(&output_path).map_err(|error| format!("Gagal membaca hasil qpdf: {error}"))
}

fn run_ghostscript(
    gs: &Path,
    input_path: &Path,
    temp_dir: &Path,
    profile: GhostscriptProfile,
) -> Result<Vec<u8>, String> {
    let output_path = temp_dir.join(profile.output_name());
    let stderr_path = temp_dir.join(format!("{}.stderr", profile.output_name()));
    let qfactor = profile.jpeg_quality() as f32 / 100.0;
    let image_dict = format!(
        "<< /QFactor {qfactor} \\
           /Blend 1 \\
           /HSamples [2 1 1 2] \\
           /VSamples [2 1 1 2] >>"
    );

    run_external_command(
        &mut Command::new(gs),
        GHOSTSCRIPT_TIMEOUT,
        &stderr_path,
        "Ghostscript",
        &[
            "-dSAFER".as_ref(),
            "-dBATCH".as_ref(),
            "-dNOPAUSE".as_ref(),
            "-sDEVICE=pdfwrite".as_ref(),
            "-dCompatibilityLevel=1.7".as_ref(),
            "-dDetectDuplicateImages=true".as_ref(),
            "-dCompressPages=true".as_ref(),
            "-dWriteXRefStm=true".as_ref(),
            "-dWriteObjStms=true".as_ref(),
            "-dDownsampleColorImages=true".as_ref(),
            "-dDownsampleGrayImages=true".as_ref(),
            "-dDownsampleMonoImages=true".as_ref(),
            "-dColorImageDownsampleType=/Bicubic".as_ref(),
            "-dGrayImageDownsampleType=/Bicubic".as_ref(),
            "-dMonoImageDownsampleType=/Subsample".as_ref(),
            format!("-dColorImageResolution={}", profile.color_dpi()).as_ref(),
            format!("-dGrayImageResolution={}", profile.gray_dpi()).as_ref(),
            format!("-dMonoImageResolution={}", profile.mono_dpi()).as_ref(),
            format!("-dColorImageDict={image_dict}").as_ref(),
            format!("-dGrayImageDict={image_dict}").as_ref(),
            format!("-dMonoImageDict={image_dict}").as_ref(),
            "-dAutoFilterColorImages=true".as_ref(),
            "-dAutoFilterGrayImages=true".as_ref(),
            "-dEncodeColorImages=true".as_ref(),
            "-dEncodeGrayImages=true".as_ref(),
            "-dEncodeMonoImages=true".as_ref(),
            "-dPreserveAnnots=true".as_ref(),
            "-dPreserveOverprintSettings=true".as_ref(),
            "-dPreserveEPSInfo=true".as_ref(),
            "-dMaxInlineImageSize=0".as_ref(),
            format!("-sOutputFile={}", output_path.display()).as_ref(),
            input_path.as_os_str(),
        ],
    )?;

    fs::read(&output_path).map_err(|error| format!("Gagal membaca hasil Ghostscript: {error}"))
}

fn run_external_command(
    command: &mut Command,
    timeout: Duration,
    stderr_path: &Path,
    tool_name: &str,
    args: &[&std::ffi::OsStr],
) -> Result<(), String> {
    for arg in args {
        command.arg(arg);
    }

    command.stdout(Stdio::null());
    let stderr_file = File::create(stderr_path)
        .map_err(|error| format!("Gagal membuat log {tool_name}: {error}"))?;
    command.stderr(Stdio::from(stderr_file));

    let mut child = command
        .spawn()
        .map_err(|error| format!("Gagal menjalankan {tool_name}: {error}"))?;
    let deadline = Instant::now() + timeout;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }

                let stderr = read_limited_stderr(stderr_path).unwrap_or_default();
                return Err(format!(
                    "{tool_name} gagal (exit code {:?}): {stderr}",
                    status.code()
                ));
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{tool_name} melebihi batas waktu {} detik",
                    timeout.as_secs()
                ));
            }
            Ok(None) => thread::sleep(PROCESS_POLL_INTERVAL),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Gagal memantau {tool_name}: {error}"));
            }
        }
    }
}

fn read_limited_stderr(path: &Path) -> Result<String, String> {
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

    let document = match load_pdf_document(bytes) {
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn external_command_times_out() {
        let temp_dir = tempdir().expect("tempdir harus tersedia");
        let stderr_path = temp_dir.path().join("timeout.stderr");
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 1"]);

        let result = run_external_command(
            &mut command,
            Duration::from_millis(100),
            &stderr_path,
            "test-command",
            &[],
        );

        let error = result.expect_err("command harus timeout");
        assert!(error.contains("melebihi batas waktu"));
    }

    #[test]
    fn external_command_captures_stderr() {
        let temp_dir = tempdir().expect("tempdir harus tersedia");
        let stderr_path = temp_dir.path().join("failure.stderr");
        let mut command = Command::new("sh");
        command.args(["-c", "printf failure >&2; exit 7"]);

        let result = run_external_command(
            &mut command,
            Duration::from_secs(1),
            &stderr_path,
            "test-command",
            &[],
        );

        let error = result.expect_err("command harus gagal");
        assert!(error.contains("exit code Some(7)"));
        assert!(error.contains("failure"));
    }
}
