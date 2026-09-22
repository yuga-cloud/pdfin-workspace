use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxMode {
    Required,
    Auto,
    Disabled,
}

impl SandboxMode {
    fn parse(value: Option<&str>) -> Self {
        match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("disabled") | Some("off") | Some("false") => Self::Disabled,
            Some("auto") => Self::Auto,
            Some("required") | Some("on") | Some("true") => Self::Required,
            None if cfg!(target_os = "linux") => Self::Required,
            _ if cfg!(target_os = "linux") => Self::Required,
            _ => Self::Disabled,
        }
    }
}

pub fn mode() -> SandboxMode {
    SandboxMode::parse(
        std::env::var("PDFIN_PARSER_SANDBOX")
            .ok()
            .as_deref(),
    )
}

pub fn ensure_ready() -> Result<(), String> {
    match SandboxMode::parse(
        std::env::var("PDFIN_PARSER_SANDBOX")
            .ok()
            .as_deref(),
    ) {
        SandboxMode::Disabled => {
            tracing::warn!("Parser sandbox dinonaktifkan; service tidak terisolasi dari parser host");
            Ok(())
        }
        SandboxMode::Auto => {
            #[cfg(target_os = "linux")]
            {
                if find_bwrap().is_none() {
                    tracing::warn!(
                        "Bubblewrap tidak ditemukan; mode sandbox auto menggunakan host process"
                    );
                } else if let Err(error) = verify_runtime() {
                    tracing::warn!(
                        %error,
                        "Bubblewrap tersedia tetapi sandbox runtime tidak dapat dipakai; mode auto menggunakan host process"
                    );
                }
            }
            Ok(())
        }
        SandboxMode::Required => {
            #[cfg(target_os = "linux")]
            {
                if find_bwrap().is_none() {
                    return Err(
                        "Parser sandbox required tetapi bwrap tidak ditemukan".to_owned()
                    );
                }

                verify_runtime()
            }

            #[cfg(not(target_os = "linux"))]
            {
                Err("Parser sandbox required hanya didukung pada Linux + bubblewrap".to_owned())
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn verify_runtime() -> Result<(), String> {
    let writable_dir =
        tempfile::tempdir().map_err(|error| format!("Gagal membuat probe sandbox: {error}"))?;
    let command = command("/bin/true", writable_dir.path())?;
    let status = command
        .status()
        .map_err(|error| format!("Gagal menjalankan probe sandbox: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Probe bubblewrap gagal dengan status {status}; periksa user namespaces dan konfigurasi host"
        ))
    }
}

pub fn command(program: &str, writable_dir: &Path) -> Result<Command, String> {
    command_with_read_only_paths(program, writable_dir, &[])
}

pub fn command_with_read_only_paths(
    program: &str,
    writable_dir: &Path,
    read_only_paths: &[&Path],
) -> Result<Command, String> {
    command_with_read_only_paths_and_env(program, writable_dir, read_only_paths, &[])
}

pub fn command_with_read_only_paths_and_env(
    program: &str,
    writable_dir: &Path,
    read_only_paths: &[&Path],
    extra_env: &[(&str, &std::ffi::OsStr)],
) -> Result<Command, String> {
    let mode = SandboxMode::parse(
        std::env::var("PDFIN_PARSER_SANDBOX")
            .ok()
            .as_deref(),
    );

    if !writable_dir.is_absolute() {
        return Err(format!(
            "Direktori sandbox harus berupa absolute path: {}",
            writable_dir.display()
        ));
    }

    for path in read_only_paths {
        if !path.is_absolute() {
            return Err(format!(
                "Path read-only sandbox harus berupa absolute path: {}",
                path.display()
            ));
        }

        if !path.exists() {
            return Err(format!(
                "Path read-only sandbox tidak ditemukan: {}",
                path.display()
            ));
        }
    }

    match mode {
        SandboxMode::Disabled => {
            tracing::warn!(
                program,
                "Parser sandbox dinonaktifkan; hanya gunakan untuk development"
            );
            let mut command = Command::new(program);
            for (name, value) in extra_env {
                command.env(name, value);
            }
            Ok(command)
        }
        SandboxMode::Auto | SandboxMode::Required => {
            #[cfg(not(target_os = "linux"))]
            {
                if mode == SandboxMode::Required {
                    return Err(
                        "Parser sandbox required membutuhkan Linux + bubblewrap."
                            .to_owned(),
                    );
                }

                tracing::warn!(
                    program,
                    "Bubblewrap tidak tersedia pada platform ini; parser berjalan tanpa sandbox"
                );
                let mut command = Command::new(program);
                for (name, value) in extra_env {
                    command.env(name, value);
                }
                Ok(command)
            }

            #[cfg(target_os = "linux")]
            {
                let Some(bwrap) = find_bwrap() else {
                    if mode == SandboxMode::Required {
                        return Err("Parser sandbox required tetapi bwrap tidak ditemukan. Install bubblewrap atau set PDFIN_PARSER_SANDBOX=disabled hanya untuk development.".to_owned());
                    }

                    tracing::warn!(
                        program,
                        "Bubblewrap tidak ditemukan; parser berjalan tanpa sandbox"
                    );
                    let mut command = Command::new(program);
                    for (name, value) in extra_env {
                        command.env(name, value);
                    }
                    return Ok(command);
                };

                let mut command = Command::new(bwrap);
                command
                    .arg("--die-with-parent")
                    .arg("--new-session")
                    .arg("--unshare-all");

                for root in ["/usr", "/bin", "/sbin", "/lib", "/lib64"] {
                    let path = Path::new(root);
                    if path.exists() {
                        command.arg("--ro-bind").arg(path).arg(path);
                    }
                }

                // Start /etc empty and expose only non-secret runtime configuration
                // required by the dynamic loader/font stack. Do not bind the host
                // /etc tree wholesale because it can contain application credentials.
                command.arg("--tmpfs").arg("/etc");

                for path in [
                    "/etc/ld.so.cache",
                    "/etc/ld.so.conf",
                    "/etc/ld.so.conf.d",
                    "/etc/fonts",
                    "/etc/nsswitch.conf",
                    "/etc/passwd",
                    "/etc/group",
                ] {
                    let path = Path::new(path);
                    if path.exists() {
                        command.arg("--ro-bind").arg(path).arg(path);
                    }
                }

                command
                    .arg("--dev")
                    .arg("/dev")
                    .arg("--proc")
                    .arg("/proc")
                    .arg("--tmpfs")
                    .arg("/home")
                    .arg("--tmpfs")
                    .arg("/root")
                    .arg("--tmpfs")
                    .arg("/tmp")
                    .arg("--bind")
                    .arg(writable_dir)
                    .arg(writable_dir);

                for path in read_only_paths {
                    append_read_only_bind(&mut command, path)?;
                }

                command.arg("--clearenv");

                for (name, value) in extra_env {
                    command.arg("--setenv").arg(name).arg(value);
                }

                command
                    .arg("--setenv")
                    .arg("PATH")
                    .arg("/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                    .arg("--setenv")
                    .arg("HOME")
                    .arg(writable_dir)
                    .arg("--setenv")
                    .arg("TMPDIR")
                    .arg(writable_dir)
                    .arg("--setenv")
                    .arg("LANG")
                    .arg("C.UTF-8")
                    .arg("--setenv")
                    .arg("LC_ALL")
                    .arg("C.UTF-8")
                    .arg("--cap-drop")
                    .arg("ALL")
                    .arg("--chdir")
                    .arg(writable_dir)
                    .arg("--")
                    .arg(program);

                tracing::debug!(
                    program,
                    writable_dir = %writable_dir.display(),
                    "Menjalankan parser di bubblewrap sandbox"
                );

                Ok(command)
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn append_read_only_bind(command: &mut Command, path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Path sandbox tidak memiliki parent: {}", path.display()))?;

    const PREBOUND_ROOTS: [&str; 8] = [
        "/usr", "/bin", "/sbin", "/lib", "/lib64", "/dev", "/proc", "/tmp",
    ];

    let mut destination = PathBuf::from("/");
    for component in parent.components() {
        let component = component.as_os_str();

        if component == "/" {
            continue;
        }

        destination.push(component);

        let already_present = PREBOUND_ROOTS
            .iter()
            .any(|root| destination == Path::new(root) || destination.starts_with(root));

        if !already_present {
            command.arg("--dir").arg(&destination);
        }
    }

    if path.is_dir()
        && !PREBOUND_ROOTS
            .iter()
            .any(|root| path == Path::new(root) || path.starts_with(root))
    {
        command.arg("--dir").arg(path);
    }

    command.arg("--ro-bind").arg(path).arg(path);
    Ok(())
}

#[cfg(target_os = "linux")]
fn find_bwrap() -> Option<std::path::PathBuf> {
    const CANDIDATES: [&str; 2] = ["/usr/bin/bwrap", "/bin/bwrap"];

    CANDIDATES
        .into_iter()
        .map(std::path::PathBuf::from)
        .find(|path| path.is_file())
        .or_else(|| {
            let path = std::path::PathBuf::from("bwrap");

            Command::new(&path)
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .ok()
                .filter(|status| status.success())
                .map(|_| path)
        })
}

#[cfg(test)]
mod tests {
    use super::{command, SandboxMode};

    #[test]
    fn parses_modes() {
        assert_eq!(SandboxMode::parse(Some("disabled")), SandboxMode::Disabled);
        assert_eq!(SandboxMode::parse(Some("auto")), SandboxMode::Auto);
        assert_eq!(SandboxMode::parse(Some("required")), SandboxMode::Required);
        assert_eq!(SandboxMode::parse(Some("true")), SandboxMode::Required);
    }

    #[test]
    fn unknown_mode_is_fail_closed() {
        assert_eq!(
            SandboxMode::parse(Some("unexpected")),
            if cfg!(target_os = "linux") {
                SandboxMode::Required
            } else {
                SandboxMode::Disabled
            }
        );
        assert_eq!(
            SandboxMode::parse(None),
            if cfg!(target_os = "linux") {
                SandboxMode::Required
            } else {
                SandboxMode::Disabled
            }
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn required_sandbox_clears_environment_and_isolates_tmp() {
        let writable_dir = tempfile::tempdir().expect("tempdir");
        let host_tmp_dir = tempfile::tempdir().expect("host tempdir");
        let host_only = host_tmp_dir.path().join("host-only.txt");
        std::fs::write(&host_only, b"secret").expect("host probe");

        let mut command = command("/bin/sh", writable_dir.path()).expect("sandbox command");
        let script = format!(
            "test -z \"$PDFIN_SANDBOX_TEST_SECRET\" && test ! -e \"{}\" && test ! -e /etc/shadow && test -w \"{}\"",
            host_only.display(),
            writable_dir.path().display()
        );
        let output = command
            .env("PDFIN_SANDBOX_TEST_SECRET", "must-not-leak")
            .arg("-c")
            .arg(script)
            .output()
            .expect("sandbox process");

        assert!(
            output.status.success(),
            "sandbox probe failed: status={}, stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
