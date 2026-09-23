# PDFin Frontend

React + Vite + TypeScript frontend for PDFin.

## Development

Install dependencies from the repository root:

```bash
cd frontend-react
npm ci
npm run dev
```

The frontend uses Vite on port 8080 and proxies `/api/v1` requests (plus the deprecated `/rust-api` alias) to the local Axum backend on port 3000.

## Validation

```bash
npm run typecheck
npm run lint
npm test
npm run build
```

CI installs dependencies with `npm ci --ignore-scripts` and keeps the committed `package-lock.json` as the reproducible dependency source.

## Structure

- `src/` — application code and feature modules.
- `scripts/` — build, PWA, and browser smoke-test helpers.
- `server/` — deployment middleware used by the Vercel/Nitro build.
- `public/` — static assets.

Document processing should remain bounded on the client. Server-side conversions are subject to the same resource and abuse controls enforced by the Rust backend.
