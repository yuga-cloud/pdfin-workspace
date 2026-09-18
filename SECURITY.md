# Security Policy

pdfin processes untrusted PDF, image, and Office documents. Treat every uploaded
document as hostile input.

## Supported security posture

The backend applies request size, timeout, concurrency, upload, PDF parsing, and
output limits. Uploaded files are written to temporary storage and are not
intended to become publicly retrievable files.

Production deployments should keep the Rust backend bound to loopback and put
TLS termination, rate limiting, and edge filtering in front of it. The example
systemd unit in `deploy/pdfin.service.example` includes additional
least-privilege and network-isolation controls.

## Reporting a vulnerability

Please do not disclose an unpatched vulnerability in a public issue.

When private security reporting is available for this repository, use GitHub's
private vulnerability reporting/security advisory flow. Otherwise, contact the
repository owner privately and include:

- affected endpoint, component, or dependency;
- reproducible steps or a minimal proof of concept;
- expected versus observed behavior;
- impact and any required attacker privileges;
- the affected commit, release, or environment.

Do not include real user documents or personal data in a report.

## Public-release requirements

Before making the repository public, verify all of the following:

1. No secrets, credentials, private keys, tokens, production configuration, or
   personal data exist in Git history.
2. `frontend-react/package-lock.json` is committed and CI uses `npm ci`, not
   a floating dependency installation.
3. Rust dependencies are locked and CI uses `--locked`.
4. GitHub Actions used on trusted runners are pinned to immutable commit SHAs.
5. Self-hosted runner jobs cannot execute code from untrusted fork or Dependabot
   pull requests.
6. The production backend is loopback-only and the host firewall blocks direct
   public access to port 3000.
7. PDF/Office/image processing runs under the unprivileged service account and
   the production systemd restrictions are enabled.
8. LibreOffice, Tesseract, Poppler/pdftoppm, PDFium, and Rust/Node dependencies
   are patched to supported security releases before launch.
9. Public traffic is protected at the edge with TLS and appropriate
   rate-limiting/abuse controls.
10. Branch protection/rulesets require review and all required security checks
    before changes reach `main`.

## Scope

A vulnerability in a third-party parser or system package used by pdfin should
be reported as security-sensitive because hostile documents are an attacker
controlled input to those components.
