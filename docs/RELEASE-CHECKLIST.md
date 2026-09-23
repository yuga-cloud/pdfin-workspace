# Public Release Checklist

Use this checklist against the exact commit intended for public release.

## Automated gates

- [ ] Rust formatting passes.
- [ ] Rust check, Clippy, and tests pass with `--locked`.
- [ ] Frontend installs with `npm ci --ignore-scripts`.
- [ ] Frontend typecheck, lint, tests, and production build pass.
- [ ] Cargo audit passes.
- [ ] npm audit has no unreviewed high-severity findings.
- [ ] Dependency Review passes.
- [ ] CodeQL passes for Rust and TypeScript/JavaScript.
- [ ] Gitleaks full-history scan passes.

## Runtime gates

- [ ] Linux production host has bubblewrap and `PDFIN_PARSER_SANDBOX=required`.
- [ ] Backend binds only to loopback.
- [ ] Reverse proxy provides TLS and edge abuse controls.
- [ ] Direct public access to port 3000 is blocked.
- [ ] LibreOffice, Poppler, Tesseract, qpdf, Ghostscript, Python/PyMuPDF, system libraries, and the host kernel are patched.
- [ ] Resource limits and temporary-file restrictions in `deploy/pdfin.service.example` are reviewed against the production host.

## Repository gates

- [ ] No secrets, private credentials, production configuration, or private data exist in the repository or Git history.
- [ ] Internal platform/template instructions have been removed.
- [ ] Root README and deployment/security documentation describe the actual PDFin architecture.
- [ ] Every user-facing tool maps to an implemented backend or client path.
- [ ] PDF → PowerPoint remains hidden until a production implementation exists.
- [ ] A project license has been selected and added before external reuse/contributions are invited.
- [ ] Main branch protection requires review and the required CI/security checks.

## Final release

Record the release commit SHA here:

```text
<release-commit-sha>
```

Do not publish the repository until the checks above have been reviewed against that exact commit.
