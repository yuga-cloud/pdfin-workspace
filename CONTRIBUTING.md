# Contributing to pdfin-workspace

Thank you for your interest in contributing.

## Development setup

Requirements:

- Rust stable toolchain
- Cargo
- Git
- Required system dependencies for PDF/document processing engines

Clone the repository:

```bash
git clone https://github.com/yuga-cloud/pdfin-workspace.git
cd pdfin-workspace
```

Run validation:

```bash
cargo fmt --all
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

## Branch conventions

Use descriptive branches:

- `feat/<name>` for new features
- `fix/<name>` for bug fixes
- `docs/<name>` for documentation changes
- `chore/<name>` for maintenance work

## Pull requests

Before opening a pull request:

- Keep changes focused
- Add tests for behavior changes when possible
- Run formatting and validation commands locally
- Explain the motivation and implementation details

## Code style

- Follow Rust formatting with `rustfmt`
- Keep security boundaries explicit
- Avoid exposing secrets or production configuration
- Prefer small, reviewable changes

## Reporting security issues

Do not open public issues for security vulnerabilities. Follow the instructions in `SECURITY.md` instead.
