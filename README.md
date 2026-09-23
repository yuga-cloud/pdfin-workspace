# pdfin-workspace

High-performance PDF processing platform built with Rust and React.

## Architecture

- Backend: Rust + Axum
- Frontend: React + Vite + TypeScript
- Shared contracts: Rust workspace crates

## Features

- PDF processing pipeline
- API v1 foundation
- OpenAPI contract generation
- Secure document handling

## Development

### Backend

```bash
cargo check --workspace --locked
cargo run -p backend
```

### Frontend

```bash
cd frontend-react
npm ci
npm run dev
```

## Security

Uploaded documents are treated as untrusted input. See `SECURITY.md` for security practices and reporting guidance.

## License

See `LICENSE`.
