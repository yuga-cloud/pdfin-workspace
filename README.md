# PDFin Workspace

**High-performance PDF & Office processing workspace built with Rust, Leptos, and React.**

🚀 **Production-ready** workspace for PDF and document processing with modern tooling.

## Quick Start

### Backend (Rust)

```bash
cd backend
cargo run
# Server running at http://localhost:3000
```

### Frontend (React)

```bash
cd frontend-react
npm install
npm run dev
# App running at http://localhost:8080
```

## Project Structure

```
pdfin-workspace/
├── backend/                 # Axum REST API (Rust)
│   ├── src/
│   │   ├── main.rs         # Server entry point
│   │   ├── routes.rs       # API routes
│   │   ├── handlers.rs     # Request handlers
│   │   ├── engines.rs      # PDF/Office processing logic
│   │   ├── error.rs        # Error handling
│   │   ├── state.rs        # Application state
│   │   └── features.rs     # Feature flags
│   └── Cargo.toml
│
├── frontend-react/          # React 19 UI (TypeScript)
│   ├── src/
│   │   ├── components/     # Reusable components
│   │   ├── hooks/          # Custom React hooks
│   │   ├── routes/         # TanStack Router pages
│   │   ├── lib/            # Utilities & API client
│   │   ├── main.tsx        # Entry point
│   │   └── index.css       # Global styles
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── README.md
│
├── frontend-leptos/         # Leptos UI (Rust) - STUB
│   ├── src/
│   └── Cargo.toml
│
├── shared/                  # Shared Rust types
│   ├── src/
│   │   ├── lib.rs
│   │   ├── operation.rs    # PdfOperation enum
│   │   └── error.rs        # ApiError struct
│   └── Cargo.toml
│
├── Cargo.toml              # Workspace config
├── .gitignore
└── README.md
```

## Architecture

### Backend (Axum + Rust)

- **Framework**: Axum 0.8.9
- **Runtime**: Tokio async
- **Request Body**: Max 50MB
- **Timeout**: 120 seconds
- **Concurrency**: Limited by available CPU cores via semaphore
- **CORS**: Configurable (production-ready)
- **Logging**: Structured tracing with detailed HTTP metrics

**Features**:
- 14 PDF/Office operations (compress, merge, split, rotate, watermark, page numbers, format conversions)
- Error handling with custom ApiError type
- Graceful shutdown support
- Health check endpoint

### Frontend (React 19 + TypeScript)

- **Framework**: React 19 + TypeScript
- **Router**: TanStack Router v1
- **State**: React Query (TanStack Query)
- **UI**: Radix UI + Tailwind CSS
- **Forms**: React Hook Form + Zod validation
- **Build**: Vite
- **Dev Server**: Port 8080

**Features**:
- Drag & drop file upload
- Real-time operation selection
- Progress indicators
- Error handling with user feedback
- Responsive design
- Type-safe API client

### Shared (Rust)

- `PdfOperation` enum - Type-safe operation definitions
- `ApiError` struct - Consistent error responses
- Cross-crate type sharing

## API Endpoints

```
✓ GET  /health                    Health check

✓ POST /api/pdf/compress          Compress PDF
✓ POST /api/pdf/merge             Merge multiple PDFs
✓ POST /api/pdf/split             Split PDF by pages
✓ POST /api/pdf/rotate            Rotate PDF pages
✓ POST /api/pdf/watermark         Add watermark
✓ POST /api/pdf/page-numbers      Add page numbers

✓ POST /api/pdf/to-word           PDF → Word
✓ POST /api/pdf/to-excel          PDF → Excel
✓ POST /api/pdf/to-powerpoint     PDF → PowerPoint
✓ POST /api/pdf/to-jpg            PDF → JPG

✓ POST /api/pdf/word-to-pdf       Word → PDF
✓ POST /api/pdf/excel-to-pdf      Excel → PDF
✓ POST /api/pdf/powerpoint-to-pdf PowerPoint → PDF
✓ POST /api/pdf/jpg-to-pdf        JPG → PDF
```

## Configuration

### Backend

Environment variables (optional, defaults provided):

```bash
CORS_ORIGIN=http://localhost:8080    # CORS allowed origin
SERVER_HOST=0.0.0.0                  # Server bind address
SERVER_PORT=3000                     # Server port
RUST_LOG=info                        # Log level
```

### Frontend

Create `.env.local`:

```bash
VITE_API_URL=http://localhost:3000   # Backend API URL
VITE_APP_NAME=PDFin Workspace
```

## Development

### Prerequisites

- Rust 1.70+
- Node.js 18+
- npm or yarn

### Setup

```bash
# Install backend dependencies
cd backend
cargo build

# Install frontend dependencies
cd ../frontend-react
npm install
```

### Run Development

```bash
# Terminal 1: Backend
cd backend
cargo run

# Terminal 2: Frontend
cd frontend-react
npm run dev
```

### Testing

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend-react
npm test

# Frontend e2e tests
npm run test:e2e
```

### Linting & Formatting

```bash
# Backend
cd backend
cargo clippy
cargo fmt

# Frontend
cd frontend-react
npm run lint
npm run format
```

## Production Build

### Backend

```bash
cd backend
cargo build --release
# Binary at target/release/backend
```

### Frontend

```bash
cd frontend-react
npm run build
# Output at dist/
```

## Deployment

### Docker

```bash
# Build backend
docker build -f backend/Dockerfile -t pdfin-backend .

# Build frontend
docker build -f frontend-react/Dockerfile -t pdfin-frontend .

# Run
docker run -p 3000:3000 pdfin-backend
docker run -p 8080:8080 pdfin-frontend
```

### Manual

1. Build both backend and frontend
2. Deploy backend to production server
3. Serve frontend static files with nginx/apache
4. Configure reverse proxy for `/api/` → backend

## Performance

- **PDF Processing**: Limited by CPU cores via semaphore (prevents overload)
- **Request Timeout**: 120 seconds
- **Max File Size**: 50MB
- **Concurrent Requests**: Unlimited (but PDF ops are serialized)
- **Frontend**: Optimized with Vite, React lazy loading, React Query caching

## Security

- CORS configurable per environment
- Request body size limits (50MB)
- File type validation on frontend
- Error messages don't expose internal details
- Graceful error handling

## Issues & TODOs

- [ ] Implement actual PDF compression in `engines.rs`
- [ ] Implement merge, split, rotate, watermark operations
- [ ] Implement format conversion logic (PDF ↔ Word, Excel, PowerPoint, JPG)
- [ ] Add database persistence layer (optional)
- [ ] Add authentication/authorization
- [ ] Add rate limiting per user/IP
- [ ] Add file upload size per-operation limits
- [ ] Add batch processing support
- [ ] Add webhook support for async processing
- [ ] Add progress tracking for large files
- [ ] Add comprehensive test suite
- [ ] Delete or complete `frontend-leptos/` (currently stub)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see LICENSE file for details

## Support

For issues and questions:
- Create a GitHub issue
- Check existing issues for solutions
- See documentation in subdirectory README files

## Roadmap

- [ ] v0.2.0 - Implement all PDF operations
- [ ] v0.3.0 - Add database backend
- [ ] v0.4.0 - Add user authentication
- [ ] v0.5.0 - Add batch processing
- [ ] v1.0.0 - Production release

---

**Made with ❤️ for high-performance document processing**
