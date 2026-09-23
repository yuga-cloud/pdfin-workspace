# pdfin-workspace

High-performance PDF and Office processing workspace built with Rust.

## Overview

`pdfin-workspace` is a backend-focused document processing platform designed around safe, predictable, and resource-conscious document handling.

## Features

- PDF processing engine
- PDF merge and split operations
- PDF optimization workflows
- Image to PDF conversion
- Office container validation
- ZIP safety validation
- Rate limiting and trusted proxy handling

## Architecture

- Backend: Rust + Axum
- Frontend: React + TypeScript
- Shared Rust crates for reusable components

Workspace layout:

```
.
├── backend
├── frontend-react
├── shared
├── deploy
└── .github
```

## Development

Requirements:

- Rust stable toolchain
- Node.js for frontend development

Backend checks:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

## Security

Security documentation is available in `SECURITY.md`.

Please report security issues responsibly.

## License

Licensed under the Apache License 2.0.
