# Backend - Rust/Axum PDF Processing API

**High-performance REST API for PDF and Office document processing.**

## Features

- 🚀 **Fast** - Built with Rust and Axum async runtime
- 📦 **Concurrent** - CPU-core aware concurrency limiting
- 📊 **Observable** - Structured tracing and detailed logging
- 🔒 **Robust** - Graceful error handling and shutdown
- 📄 **Versatile** - 14 PDF and Office operations

## Quick Start

### Prerequisites

- Rust 1.70+
- Cargo

### Build

```bash
cargo build
```

### Run

```bash
cargo run
# Server listening on http://localhost:3000
```

## Architecture

### Project Structure

```
src/
├── main.rs        # Server setup, middleware, listeners
├── routes.rs      # Route definitions
├── handlers.rs    # HTTP request handlers
├── engines.rs     # PDF/Office processing logic
├── error.rs       # Error types and response conversion
├── state.rs       # Application state
└── features.rs    # Feature flags
```

### Request Flow

1. **Client** sends multipart file upload
2. **routes.rs** matches endpoint
3. **handlers.rs** extracts and validates request
4. **Semaphore** limits concurrent PDF operations
5. **engines.rs** processes file
6. **error.rs** converts result to JSON response
7. **Response** returned to client

### Concurrency Control

```rust
// Automatically limits PDF operations to CPU core count
let pdf_concurrency = std::thread::available_parallelism()
    .map(usize::from)
    .unwrap_or(1);

let state = AppState {
    pdf_semaphore: Arc::new(Semaphore::new(pdf_concurrency)),
};
```

## API Endpoints

### Health

```
GET /health
Response: "ok"
```

### PDF Operations

```
POST /api/pdf/compress       - Compress PDF
POST /api/pdf/merge          - Merge multiple PDFs
POST /api/pdf/split          - Split PDF by pages
POST /api/pdf/rotate         - Rotate pages
POST /api/pdf/watermark      - Add watermark
POST /api/pdf/page-numbers   - Add page numbers
```

### Format Conversions

```
POST /api/pdf/to-word        - PDF → Word (.docx)
POST /api/pdf/to-excel       - PDF → Excel (.xlsx)
POST /api/pdf/to-powerpoint  - PDF → PowerPoint (.pptx)
POST /api/pdf/to-jpg         - PDF → JPG images

POST /api/pdf/word-to-pdf    - Word → PDF
POST /api/pdf/excel-to-pdf   - Excel → PDF
POST /api/pdf/powerpoint-to-pdf - PowerPoint → PDF
POST /api/pdf/jpg-to-pdf     - JPG → PDF
```

### Request Format

```
Content-Type: multipart/form-data

File field: "file" (binary)
Optional fields depend on operation
```

### Response Format

**Success (200 OK):**
```json
{
  "success": true,
  "message": "PDF compressed successfully",
  "file_size": 1024000
}
```

**Error (400/500):**
```json
{
  "error": {
    "code": "COMPRESS_FAILED",
    "message": "Failed to compress PDF: invalid format"
  }
}
```

## Environment Variables

```bash
# Server binding
SERVER_HOST=0.0.0.0          # Default: 127.0.0.1
SERVER_PORT=3000             # Default: 3000

# Request limits
MAX_REQUEST_BODY_SIZE=52428800  # Default: 50MB
REQUEST_TIMEOUT_SECS=120       # Default: 120

# CORS
CORS_ORIGIN=http://localhost:8080  # Default: Allow all

# Logging
RUST_LOG=info                 # Default: info
```

## Configuration

### Server

- **Host**: `127.0.0.1` (localhost)
- **Port**: `3000`
- **Body Limit**: 50MB
- **Request Timeout**: 120 seconds
- **Concurrency**: CPU core count

### Middleware Stack

1. **TraceLayer** - HTTP request/response logging
2. **TimeoutLayer** - Request timeout enforcement
3. **RequestBodyLimitLayer** - Request body size limit
4. **CorsLayer** - Cross-origin resource sharing

## Logging

### Structured Logging

All logs include:
- Thread ID and name
- Request method and URI
- Response status and latency
- Error details

### Log Format

```
2026-08-29T21:49:39Z INFO method=POST uri=/api/pdf/compress "HTTP request masuk"
2026-08-29T21:49:40Z INFO status=200 latency_ms=1234 "HTTP response selesai"
```

### Set Log Level

```bash
RUST_LOG=debug cargo run
RUST_LOG=trace cargo run
```

## Development

### Run with Logging

```bash
RUST_LOG=debug cargo run
```

### Run Tests

```bash
cargo test
```

### Check Code Quality

```bash
# Lint
cargo clippy

# Format
cargo fmt --check

# Apply formatting
cargo fmt
```

## Performance Characteristics

- **Single PDF Operation**: Limited by semaphore to CPU cores
- **Multiple Concurrent Requests**: Other requests wait for semaphore
- **Large Files**: Streamed via multipart, not fully buffered
- **Memory**: Reasonable for files up to 50MB
- **CPU**: Fully utilized for compress/convert operations

## Error Handling

All errors are converted to JSON responses with:
- **code**: Machine-readable error identifier
- **message**: Human-readable error description
- **HTTP Status**: Appropriate status code (400, 500, etc)

### Common Error Codes

```
NO_FILE              - File not provided
FILE_READ_ERROR      - Failed to read uploaded file
MULTIPART_ERROR      - Multipart parsing failed
SEMAPHORE_ERROR      - Concurrency limit exceeded
COMPRESS_FAILED      - PDF compression failed
INTERNAL_ERROR       - Unexpected server error
```

## Dependencies

- **axum** 0.8.9 - Web framework
- **tokio** 1.53 - Async runtime
- **tower-http** 0.7 - HTTP middleware
- **tracing** 0.1.44 - Structured logging
- **serde** 1.0.229 - Serialization
- **lopdf** 0.44 - PDF manipulation
- **pdfium-render** 0.9.3 - PDF rendering
- **rust_xlsxwriter** 0.99 - Excel generation

## Production Deployment

### Build Release Binary

```bash
cargo build --release
# Binary at target/release/backend
```

### Docker

```dockerfile
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/backend"]
```

### Configuration for Production

```bash
RUST_LOG=warn
CORS_ORIGIN=https://yourdomain.com
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

## Monitoring

### Health Check

```bash
curl http://localhost:3000/health
# Response: "ok"
```

### Metrics

Check logs for:
- Request latency (latency_ms)
- Error rates (status != 200)
- Concurrency (concurrent operations via semaphore)

## TODO & Improvements

- [ ] Implement actual PDF operations in `engines.rs`
- [ ] Add rate limiting per IP/user
- [ ] Add request ID tracking
- [ ] Add metrics/prometheus export
- [ ] Add database persistence
- [ ] Add authentication/authorization
- [ ] Add async processing with job queue
- [ ] Add progress tracking
- [ ] Add webhook callbacks
- [ ] Add comprehensive test suite

## Contributing

1. Write tests for new features
2. Run `cargo clippy` and `cargo fmt`
3. Update documentation
4. Submit PR

## License

MIT
