# PDFin

![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

PDFin is a high-performance PDF and Office document processing workspace with a React + Vite frontend and a Rust + Axum backend.

The service is designed around a simple rule: uploaded documents are untrusted input. PDF, image, and Office processing therefore uses bounded input sizes, output sizes, timeouts, concurrency, rate limits, parser-specific guards, and process isolation where available.

## Architecture

```text
React + Vite + TypeScript
          │
          │ /rust-api
          ▼
     Rust + Axum
          │
          ├── PDF structural operations (lopdf)
          ├── PDF rendering / text extraction
          ├── Office conversion (LibreOffice)
          ├── OCR (Poppler + Tesseract)
          └── optimization helpers (PyMuPDF / qpdf / Ghostscript)
```

The repository is intentionally a small Cargo workspace:

- `backend/` — Axum HTTP service and document-processing engines.
- `shared/` — types shared by backend features.
- `frontend-react/` — React/Vite application.
- `deploy/` — production systemd example.

## Supported operations

Current user-facing conversion and PDF tooling includes PDF merge/split/rotation/page management, PDF optimization, JPG ↔ PDF, Word → PDF, Excel → PDF, PowerPoint → PDF, PDF → Word, and PDF → Excel.

PDF → PowerPoint is intentionally not exposed until its implementation is production-ready.

## Development

### Backend

```bash
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

### Frontend

```bash
cd frontend-react
npm ci
npm run typecheck
npm run lint
npm test
npm run build
```

The Vite development server uses port 8080 and proxies `/rust-api` to the Axum backend on port 3000.

## Production model

Keep the Axum listener on loopback and place a TLS reverse proxy or equivalent edge service in front of it. Enable the example systemd restrictions in `deploy/pdfin.service.example`, including the required parser sandbox on Linux.

See:

- [Deployment](docs/DEPLOYMENT.md)
- [Security hardening](docs/SECURITY-HARDENING.md)
- [Security policy](SECURITY.md)

## Security model

PDFin treats document contents, filenames, ZIP metadata, page geometry, rendered output, and Office relationships as attacker-controlled input.

Important protections include:

- bounded HTTP request bodies and multipart parsing;
- per-IP rate limiting and request/conversion concurrency limits;
- bounded PDF decompression;
- PDF page-dimension validation before expensive raster work;
- bounded ZIP entry counts and uncompressed sizes;
- ZIP path-traversal, overlap, encryption, macro, and external-relationship checks;
- output-size and timeout controls around external document processors;
- least-privilege systemd deployment and a bubblewrap parser sandbox on Linux;
- GitHub Actions dependency and secret scanning.

These controls reduce attack impact but do not make third-party parsers bug-free. Keep system packages and parser dependencies patched.

## Contributing

Before opening a pull request, run the same formatting, typecheck, lint, test, and build commands used by CI. Avoid adding new document-processing paths without an explicit input/output/resource budget.

## License

Licensed under the Apache License 2.0. See [LICENSE](LICENSE).
