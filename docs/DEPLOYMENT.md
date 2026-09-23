# PDFin Deployment

This document describes the production deployment model for the Rust backend.

## Host requirements

A Linux host should provide:

- Rust toolchain used to build the backend;
- bubblewrap (`bwrap`) for the required parser sandbox;
- LibreOffice for Office conversion;
- Poppler utilities, including `pdftoppm` / `pdftocairo`;
- Tesseract for OCR;
- Python + PyMuPDF when the optimization path that uses PyMuPDF is enabled;
- qpdf and Ghostscript when the corresponding optimization fallbacks are enabled;
- systemd.

Package names vary by distribution. Install the distribution's supported security releases rather than copying package versions from this document.

## Build

Build the workspace with the committed lockfile:

```bash
cargo build --release --workspace --locked
```

Build the frontend with the committed npm lockfile:

```bash
cd frontend-react
npm ci --ignore-scripts
npm run typecheck
npm run lint
npm test
npm run build
```

Do not use a floating `npm install` in production CI.

## Runtime account

Run the backend as an unprivileged dedicated account such as `pdfin:pdfin`. Do not run the service as root.

For production PDFium, preinstall the reviewed PDFium shared library at the path used by the service. The example uses:

```text
/opt/pdfin/lib/libpdfium.so
```

and sets:

```text
PDFIUM_LIB_PATH=/opt/pdfin/lib/libpdfium.so
PDFIUM_NO_AUTO_DOWNLOAD=1
```

This is deliberate: parser sandboxes have no network access, and runtime auto-downloads would make the PDFium supply-chain boundary less explicit. The parent process resolves the library path once and passes only that file into the PDFium worker as a read-only bind.

The example service file sets:

- `PDFIN_HOST=127.0.0.1`;
- `PDFIN_PORT=3000`;
- request, timeout, concurrency, and rate limits;
- `PDFIN_PARSER_SANDBOX=required`;
- systemd memory, task, file-descriptor, namespace, capability, filesystem, and network restrictions.

Review every restriction against the actual host before enabling the service.

## Reverse proxy

Expose only the reverse proxy publicly. The Axum process should remain loopback-only.

At the edge:

1. terminate TLS;
2. apply request and connection abuse controls;
3. forward only the headers your deployment explicitly trusts;
4. preserve the client IP correctly for rate limiting;
5. reject direct access to the backend port from the public network.

Set `PDFIN_TRUSTED_PROXY_IPS` only to proxy addresses that you fully control.

## Parser sandbox

On Linux, PDFin can run untrusted parser subprocesses inside bubblewrap. Production should keep:

```text
PDFIN_PARSER_SANDBOX=required
```

The service must refuse startup when the required sandbox dependency is unavailable.

This is defense in depth. The host should still run the service under the dedicated unprivileged account and systemd restrictions from `deploy/pdfin.service.example`.

## Temporary files

Document inputs and intermediate results are stored in temporary locations. Do not expose the system temporary directory through the web server, reverse proxy, or static asset configuration.

Use the service's systemd filesystem restrictions and a restrictive `UMask`.

## Operational checks

After deployment, verify:

```bash
systemctl status pdfin
journalctl -u pdfin
curl -fsS http://127.0.0.1:3000/health
```

The public health endpoint should be exposed through the reverse proxy only when that is consistent with your monitoring policy.

## Updating

Keep these classes of components patched:

- Rust crates in `Cargo.lock`;
- Node dependencies in `frontend-react/package-lock.json`;
- LibreOffice;
- Poppler;
- Tesseract;
- Ghostscript;
- qpdf;
- Python/PyMuPDF;
- bubblewrap;
- the host kernel and system libraries.

Re-run the full CI/security pipeline after every dependency change.

## Rollback

Keep a known-good backend binary and frontend build artifact. Roll back the whole application release together rather than mixing a frontend from one release with an incompatible backend contract.
