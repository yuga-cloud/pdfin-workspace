# PDFin Security Hardening

PDFin accepts attacker-controlled documents, so security boundaries are part of the application architecture rather than an optional deployment feature.

## Trust boundaries

```text
Internet
  │
  ▼
TLS / edge filtering / abuse controls
  │
  ▼
Axum HTTP boundary
  │
  ├── request size / timeout / concurrency / rate limit
  │
  ▼
temporary input file
  │
  ├── bounded lopdf parsing
  ├── bounded PDF geometry
  ├── Office ZIP validation
  │
  └── isolated external parser/converter
        ├── bubblewrap on Linux
        ├── unprivileged service account
        └── systemd resource + network restrictions
```

The browser has a separate trust boundary. Client-side PDF processing is also bounded because a malicious file can exhaust browser memory even when the server is never contacted.

## PDF protections

Structural PDF paths use an explicit decompressed-stream budget.

Raster-oriented paths validate page dimensions before invoking expensive rendering. A valid PDF signature alone is not enough: a document can be syntactically valid while requesting extreme page geometry.

PDFium extraction is isolated behind a worker process on supported Linux deployments, with bounded worker input, output, stderr, and execution time.

## Office ZIP protections

OOXML validation checks:

- ZIP structure and central-directory consistency;
- entry count;
- total uncompressed size;
- filename length;
- path traversal;
- encryption;
- unsupported compression modes;
- ZIP64 usage;
- overlapping entries;
- required OOXML files;
- macro payloads;
- external-link payloads;
- external relationships.

These checks are deliberately performed before handing the document to higher-level Office libraries.

## External processes

LibreOffice, Tesseract, Poppler, qpdf, Ghostscript, and Python helpers are treated as separate attack surfaces.

For each processor, PDFin applies a combination of:

- temporary working directories;
- bounded stdout/stderr capture;
- timeouts;
- output-size limits;
- process cleanup on failure;
- production OS sandboxing.

Do not add a new subprocess without documenting its input paths, output paths, timeout, maximum output, network needs, and sandbox behavior.

## Browser protections

The frontend limits:

- PDF size and page count;
- page geometry;
- rendered pixel budgets;
- image dimensions and pixel counts;
- ZIP/Office entry counts and expanded sizes;
- preview rendering to visible/virtualized pages.

PDF.js scripting is disabled for document processing.

## CI and supply chain

The repository keeps both Rust and npm lockfiles committed. CI uses `--locked` and `npm ci`.

GitHub Actions used by trusted workflows are pinned to immutable commit SHAs. Public pull requests are validated on GitHub-hosted runners rather than the privileged self-hosted environment.

Security automation includes:

- Cargo dependency auditing;
- npm dependency auditing;
- dependency review;
- CodeQL for Rust and TypeScript/JavaScript;
- full-history secret scanning.

## Release gates

Before a public release:

1. all required CI checks pass on the exact release commit;
2. Cargo and npm lockfiles are committed;
3. dependency audits report no unreviewed high-severity findings;
4. the required parser sandbox is available on production Linux hosts;
5. external document-processing dependencies are patched;
6. no internal platform instructions or credentials remain in source/history;
7. the production service listens only on loopback;
8. the reverse proxy and firewall prevent direct backend access;
9. branch protection and required checks are configured;
10. a release artifact can be reproduced from the repository state.

No parser-hardening claim should be interpreted as a guarantee against vulnerabilities in third-party native or document-processing libraries.
