# PDFin self-hosted GitHub Actions runner

PDFin's `CI` and `Security` workflows target a repository-level self-hosted Linux x64 runner with the default labels:

- `self-hosted`
- `linux`
- `x64`

The workflow uses `actions/cache@v5` and Node 24-based GitHub Actions. Keep the self-hosted runner application at least version **2.327.1** so the current Node 24 action runtime and cache service are supported.

GitHub routes a job to a self-hosted runner when all labels in `runs-on` match. The runner application itself is lightweight; the CPU/RAM load comes from the CI job that is actually running.

## 1. Create the runner

In GitHub, open:

`pdfin-workspace → Settings → Actions → Runners → New self-hosted runner`

Choose the Linux / x64 instructions. GitHub will show the current runner package and a short-lived registration token. Do not commit that token or any generated credentials to the repository.

Register it with the labels used by the workflows:

```bash
./config.sh --url https://github.com/yuga-cloud/pdfin-workspace --token <REGISTRATION_TOKEN> --labels linux,x64
```

The repository-level runner automatically receives the `self-hosted` label.

## 2. Install the PDFin toolchain on the runner machine

The current workflow expects the runner to already provide the build tools and system dependencies. On Arch Linux, install the following packages (adjust package names if your system uses a different package source):

```bash
sudo pacman -S --needed \
  base-devel \
  git \
  pkgconf \
  openssl \
  rustup \
  nodejs \
  npm \
  tesseract \
  libreoffice-fresh \
  poppler \
  bubblewrap
```

Then make sure the stable Rust toolchain and components required by CI are installed:

```bash
rustup default stable
rustup component add rustfmt clippy
```

Verify the core tools before starting the runner:

```bash
cargo --version
rustc --version
rustfmt --version
cargo clippy --version
node --version
npm --version
git --version
pkg-config --version
tesseract --version
libreoffice --version
pdftoppm -v
bwrap --version
```

The current workflow also expects a C compiler available as `cc` for native Rust dependencies.

## 3. Keep the runner online

From the runner directory:

```bash
./run.sh
```

For a machine that should run CI unattended, install the runner as a system service using the service instructions printed by GitHub during runner setup.

## 4. Validate the registration

Return to:

`pdfin-workspace → Settings → Actions → Runners`

The runner should be shown as **Idle** and should have `self-hosted`, `linux`, and `x64` labels.

Once the runner is online, new pushes to `main` and pull requests can execute both `CI` and `Security` without consuming GitHub-hosted runner minutes. Self-hosted runner compute is not charged as GitHub-hosted Actions minutes; you are responsible for the runner machine itself.

## 5. Resource expectations

Keep only one runner instance active on the development machine. GitHub sends one job at a time to a single runner, so the machine will normally be idle between jobs. Rust compilation, tests, npm installation, and PDF system integration tests are the resource-intensive portions.

For a development laptop with limited RAM, avoid starting multiple self-hosted runner instances for the same repository.