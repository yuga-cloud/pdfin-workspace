use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use tempfile::tempdir;

use super::compress::CompressionQuality;
use crate::engines::pdf::common::{
    load_pdf_document, load_pdf_document_from_path, validate_pdf_path,
};

const PYTHON_SCRIPT_RELATIVE: &str = "scripts/compress_pdf/compressor.py";
const MAX_OUTPUT_BYTES: usize = 1024 * 1024 * 1024;
const MAX_INPUT_PAGES: usize = 10_000;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const MIN_PRIMARY_REDUCTION_PERCENT: usize = 5;
const PYMUPDF_TIMEOUT: Duration = Duration::from_secs(120);
const QPDF_TIMEOUT: Duration = Duration::from_secs(180);
const GHOSTSCRIPT_TIMEOUT: Duration = Duration::from_secs(300);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(25);

static PYTHON_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static QPDF_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static GHOSTSCRIPT_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn compress_pdf_from_path(
    input_path: &Path,
    quality: CompressionQuality,
) -> Result<Vec<u8>, String> {
    validate_pdf_path(input_path)?;

    let input_size = usize::try_from(
        fs::metadata(input_path)
            .map_err(|error| format!("Gagal membaca metadata PDF: {error}"))?
            .len(),
    )
    .map_err(|_| "Ukuran PDF melebihi kapasitas yang didukung".to_owned())?;

    let input_document = load_pdf_document_from_path(input_path)
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

    if let Some(python) = resolve_python()
        && let Some(script) = resolve_python_script()
    {
        let quality_name = quality_name(quality);
        let output_path = temp_dir.path().join(format!("pymupdf-{quality_name}.pdf"));
        let stderr_path = temp_dir
            .path()
            .join(format!("pymupdf-{quality_name}.stderr"));

        if run_pymupdf(
            &python,
            &script,
            quality,
            input_path,
            &output_path,
            &stderr_path,
        )
        .is_ok()
            && let Ok(output) = fs::read(&output_path)
            && is_valid_pdf(&output, input_pages)
        {
            if has_meaningful_reduction(output.len(), input_size) {
                return Ok(output);
            }

            if output.len() < input_size
                && let Some(fallback) = run_fallback(
                    quality,
                    input_path,
                    temp_dir.path(),
                    input_pages,
                    input_size,
                )
            {
                return Ok(choose_smaller(output, fallback));
            }
        }
    }

    if let Some(fallback) = run_fallback(
        quality,
        input_path,
        temp_dir.path(),
        input_pages,
        input_size,
    ) {
        return Ok(fallback);
    }

    fs::read(input_path).map_err(|error| format!("Gagal membaca PDF input: {error}"))
}

fn quality_name(quality: CompressionQuality) -> &'static str {
    match quality {
        CompressionQuality::High => "high",
        CompressionQuality::Medium => "medium",
        CompressionQuality::Low => "low",
    }
}

fn choose_smaller(first: Vec<u8>, second: Vec<u8>) -> Vec<u8> {
    if first.len() <= second.len() {
        first
    } else {
        second
    }
}

fn run_fallback(
    quality: CompressionQuality,
    input_path: &Path,
    temp_dir: &Path,
    expected_pages: usize,
    input_size: usize,
) -> Option<Vec<u8>> {
    let output = match quality {
        CompressionQuality::High => {
            let qpdf = resolve_qpdf()?;
            run_qpdf(&qpdf, input_path, temp_dir).ok()?
        }
        CompressionQuality::Medium => {
            let gs = resolve_ghostscript()?;
            run_ghostscript(&gs, input_path, temp_dir, GhostscriptProfile::Medium).ok()?
        }
        CompressionQuality::Low => {
            let gs = resolve_ghostscript()?;
            run_ghostscript(&gs, input_path, temp_dir, GhostscriptProfile::Low).ok()?
        }
    };

    if output.len() > MAX_OUTPUT_BYTES
        || output.len() >= input_size
        || !is_valid_pdf(&output, expected_pages)
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
        self.color_dpi()
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
) -> Result<(), String> {
    run_external_command(
        &mut Command::new(python),
        PYMUPDF_TIMEOUT,
        stderr_path,
        "PyMuPDF",
        output_path,
        &[
            script.as_os_str(),
            quality_name(quality).as_ref(),
            input_path.as_os_str(),
            output_path.as_os_str(),
        ],
    )
}

fn run_qpdf(qpdf: &Path, input_path: &Path, temp_dir: &Path) -> Result<Vec<u8>, String> {
    let output_path = temp_dir.join("qpdf-output.pdf");
    let stderr_path = temp_dir.join("qpdf.stderr");

    run_external_command(
        &mut Command::new(qpdf),
        QPDF_TIMEOUT,
        &stderr_path,
        "qpdf",
        &output_path,
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
        &output_path,
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
    output_path: &Path,
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
                if output_size_exceeds(output_path)? {
                    return Err(format!(
                        "Hasil {tool_name} melebihi batas ukuran {} MiB",
                        MAX_OUTPUT_BYTES / 1024 / 1024
                    ));
                }

                if status.success() {
                    return Ok(());
                }

                let stderr = read_limited_stderr(stderr_path).unwrap_or_default();
                return Err(format!(
                    "{tool_name} gagal (exit code {:?}): {stderr}",
                    status.code()
                ));
            }
            Ok(None) => {
                if output_size_exceeds(output_path)? {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "Hasil {tool_name} melebihi batas ukuran {} MiB dan proses dihentikan.",
                        MAX_OUTPUT_BYTES / 1024 / 1024
                    ));
                }

                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "{tool_name} melebihi batas waktu {} detik",
                        timeout.as_secs()
                    ));
                }
                thread::sleep(PROCESS_POLL_INTERVAL);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Gagal memantau {tool_name}: {error}"));
            }
        }
    }
}

fn output_size_exceeds(path: &Path) -> Result<bool, String> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.len() > MAX_OUTPUT_BYTES as u64),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("Gagal memeriksa ukuran hasil PDF: {error}")),
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
    PYTHON_PATH
        .get_or_init(|| {
            let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
            let venv_python = manifest_dir.join(".venv/bin/python");

            if venv_python.is_file() && python_has_pymupdf(&venv_python) {
                return Some(venv_python);
            }

            for candidate in ["python3", "python", "/usr/bin/python3"] {
                let path = PathBuf::from(candidate);

                if path.is_absolute() && !path.is_file() {
                    continue;
                }

                if python_has_pymupdf(&path) {
                    return Some(path);
                }
            }

            None
        })
        .clone()
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

fn resolve_qpdf() -> Option<PathBuf> {
    QPDF_PATH
        .get_or_init(|| resolve_program(&["qpdf", "/usr/bin/qpdf"]))
        .clone()
}

fn resolve_ghostscript() -> Option<PathBuf> {
    GHOSTSCRIPT_PATH
        .get_or_init(|| resolve_program(&["gs", "ghostscript", "/usr/bin/gs"]))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_input_path() {
        let result = compress_pdf_from_path(
            Path::new("/definitely/not/a/real/pdfin-file.pdf"),
            CompressionQuality::Medium,
        );

        assert!(result.is_err());
    }

    #[test]
    fn chooses_smaller_output() {
        let result = choose_smaller(vec![0; 3], vec![0; 5]);
        assert_eq!(result.len(), 3);
    }
}
