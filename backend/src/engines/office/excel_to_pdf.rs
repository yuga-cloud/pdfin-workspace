use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use tempfile::tempdir;

use crate::engines::common::validate_input;

// Menyedot script Python langsung ke dalam binary Rust saat kompilasi!
const PYTHON_SCRIPT: &str = include_str!("../../../scripts/excel_to_pdf.py");

const OUTPUT_FILE_NAME: &str = "output.pdf";
const PYTHON_TIMEOUT: Duration = Duration::from_secs(120);
const TIMEOUT_POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_STDERR_BYTES: usize = 16 * 1024;

const SYSTEM_PYTHON_CANDIDATES: [&str; 4] = [
    "/usr/bin/python3",
    "/usr/bin/python3.13",
    "python3",
    "python",
];

pub fn excel_to_pdf(document_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(document_bytes, "Excel")?;

    let temp_dir =
        tempdir().map_err(|error| format!("Gagal membuat direktori sementara: {error}"))?;

    let temp_path = temp_dir.path();
    let input_path = temp_path.join("input.xlsx");
    let output_path = temp_path.join(OUTPUT_FILE_NAME);
    let script_path = temp_path.join("excel_to_pdf.py");
    let profile_dir = temp_path.join("libreoffice-profile");

    fs::create_dir_all(&profile_dir).map_err(|error| {
        format!(
            "Gagal membuat profile LibreOffice `{}`: {error}",
            profile_dir.display()
        )
    })?;

    fs::write(&input_path, document_bytes).map_err(|error| {
        format!(
            "Gagal menyimpan file Excel sementara `{}`: {error}",
            input_path.display()
        )
    })?;

    // Menulis skrip yang ada di dalam binary ke file temporary untuk dieksekusi
    fs::write(&script_path, PYTHON_SCRIPT).map_err(|error| {
        format!(
            "Gagal menulis helper UNO `{}`: {error}",
            script_path.display()
        )
    })?;

    let python_program = resolve_uno_python()?;
    let pipe_name = unique_pipe_name();
    let profile_uri = format!("file://{}", profile_dir.display());
    let accept_argument = format!("--accept=pipe,name={pipe_name};urp;StarOffice.ComponentContext");
    let stderr_path = temp_path.join("uno.stderr");
    let mut last_error: Option<String> = None;

    for program in ["libreoffice", "soffice"] {
        let mut office = match Command::new(program)
            .arg("--headless")
            .arg("--nologo")
            .arg("--nodefault")
            .arg("--nolockcheck")
            .arg("--norestore")
            .arg(format!("-env:UserInstallation={profile_uri}"))
            .arg(&accept_argument)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                last_error = Some(format!("Tidak bisa menjalankan {program}: {error}"));
                continue;
            }
        };

        let mut python_command = Command::new(python_program);
        python_command
            .arg(&script_path)
            .arg(&input_path)
            .arg(&output_path)
            .arg(&pipe_name);

        let python_output = match run_with_timeout(python_command, PYTHON_TIMEOUT, &stderr_path) {
            Ok(output) => output,
            Err(error) => {
                stop_office(&mut office);
                last_error = Some(format!("Helper UNO gagal: {error}"));
                continue;
            }
        };

        stop_office(&mut office);

        if !python_output.status.success() {
            last_error = Some(format!(
                "Konversi Excel → PDF via LibreOffice UNO gagal (status {}): {}",
                python_output
                    .status
                    .code()
                    .map_or_else(|| "unknown".to_owned(), |code| code.to_string()),
                python_output.stderr.trim()
            ));
            continue;
        }

        if !output_path.is_file() {
            last_error = Some("LibreOffice selesai tetapi PDF hasil tidak ditemukan.".to_owned());
            continue;
        }

        return fs::read(&output_path)
            .map_err(|error| format!("Gagal membaca PDF hasil konversi: {error}"));
    }

    Err(last_error.unwrap_or_else(|| {
        "LibreOffice tidak ditemukan. Pastikan libreoffice/soffice dan paket python-uno tersedia."
            .to_owned()
    }))
}

struct CommandOutput {
    status: std::process::ExitStatus,
    stderr: String,
}

fn run_with_timeout(
    mut command: Command,
    timeout: Duration,
    stderr_path: &Path,
) -> Result<CommandOutput, String> {
    if timeout.is_zero() {
        return Err("Timeout helper UNO harus lebih besar dari 0.".to_owned());
    }

    let stderr_file = File::create(stderr_path)
        .map_err(|error| format!("Gagal membuat log helper UNO: {error}"))?;

    command
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));

    let mut child = command
        .spawn()
        .map_err(|error| format!("Gagal menjalankan helper UNO: {error}"))?;

    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = read_limited_stderr(stderr_path)?;
                return Ok(CommandOutput { status, stderr });
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();

                    return Err(format!(
                        "Helper UNO melebihi batas waktu {} detik dan dihentikan.",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(TIMEOUT_POLL_INTERVAL);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();

                return Err(format!("Gagal menunggu helper UNO: {error}"));
            }
        }
    }
}

fn read_limited_stderr(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| format!("Gagal membaca log helper UNO: {error}"))?;
    let mut bytes = Vec::with_capacity(MAX_STDERR_BYTES + 1);

    file.take((MAX_STDERR_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Gagal membaca log helper UNO: {error}"))?;

    let truncated = bytes.len() > MAX_STDERR_BYTES;
    bytes.truncate(MAX_STDERR_BYTES);

    let mut stderr = String::from_utf8_lossy(&bytes).trim().to_owned();
    if truncated {
        stderr.push_str(" [stderr truncated]");
    }

    Ok(stderr)
}

fn stop_office(office: &mut std::process::Child) {
    let _ = office.kill();
    let _ = office.wait();
}

fn resolve_uno_python() -> Result<&'static Path, String> {
    static UNO_PYTHON: OnceLock<Result<PathBuf, String>> = OnceLock::new();

    UNO_PYTHON
        .get_or_init(|| {
            for candidate in SYSTEM_PYTHON_CANDIDATES {
                let path = PathBuf::from(candidate);

                if path.is_absolute() && !path.is_file() {
                    continue;
                }

                let status = Command::new(candidate)
                    .arg("-c")
                    .arg("import uno")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                if status.is_ok_and(|status| status.success()) {
                    return Ok(path);
                }
            }

            Err(
                "Tidak ditemukan Python yang memiliki modul LibreOffice UNO. \
                 Pastikan paket Python UNO (misal: libreoffice-script-provider-python) terinstal."
                    .to_owned(),
            )
        })
        .as_ref()
        .map(PathBuf::as_path)
        .map_err(Clone::clone)
}

fn unique_pipe_name() -> String {
    static PIPE_COUNTER: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    let counter = PIPE_COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("pdfin-excel-{}-{nanos}-{counter}", std::process::id())
}
