# PDFin Workspace Changelog

## [0.1.0] - 2026-08-29

### Added
- Initial project setup with Rust backend (Axum) and React frontend
- 14 PDF and Office document operations
- Comprehensive backend with routes, handlers, error handling
- Modern React 19 frontend with TypeScript
- API client with Zod validation
- Drag & drop file upload UI
- Operation selector component
- Structured logging and error handling
- Development documentation
- Production-ready configuration

### Backend Features
- Axum 0.8.9 REST API framework
- Tokio async runtime with multi-threading
- Request body limit (50MB) and timeout (120s)
- Concurrency control via semaphore (CPU core aware)
- CORS support with configurable origins
- Structured tracing and detailed HTTP logging
- Graceful shutdown handling
- Health check endpoint

### Frontend Features
- React 19 with TypeScript
- TanStack Router for client-side routing
- React Query for data fetching
- Radix UI for accessible components
- Tailwind CSS for styling
- React Hook Form + Zod for form validation
- Drag & drop file upload
- Real-time operation selection
- Error and success state handling
- Responsive design

### Operations
- PDF compression
- PDF merge, split, rotate
- Watermarking, page numbers
- Format conversions (PDF ↔ Word, Excel, PowerPoint, JPG)

### TODO
- Implement actual PDF processing operations
- Add database persistence layer
- Add user authentication
- Add rate limiting
- Add comprehensive test suite
- Add e2e tests
- Delete or complete Leptos frontend

## Future Versions

### [0.2.0] - PDF Operations
- Implement all PDF operations in engines.rs
- Add batch processing support
- Add progress tracking
- Add file compression for uploads

### [0.3.0] - Database
- Add PostgreSQL integration
- Add job persistence
- Add operation history
- Add user sessions

### [0.4.0] - Authentication
- Add user registration
- Add JWT authentication
- Add API keys
- Add rate limiting per user

### [0.5.0] - Advanced Features
- Add async processing with job queue
- Add webhook callbacks
- Add webhooks API
- Add S3 integration
- Add multi-language support

### [1.0.0] - Production Release
- Stable API
- Comprehensive documentation
- Full test coverage
- Performance optimizations
- Docker/Kubernetes ready
