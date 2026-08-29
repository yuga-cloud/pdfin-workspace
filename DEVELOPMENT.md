# Development Documentation

## Setup Instructions

### 1. Clone Repository

```bash
git clone https://github.com/yuga-cloud/pdfin-workspace.git
cd pdfin-workspace
```

### 2. Backend Setup (Rust)

```bash
cd backend

# Install dependencies
cargo build

# Run development server
cargo run

# Server will start at http://localhost:3000
```

### 3. Frontend Setup (React)

```bash
cd frontend-react

# Install dependencies
npm install

# Create environment file
cp .env.example .env.local

# Start development server
npm run dev

# App will open at http://localhost:8080
```

## Project Layout

### Backend (`backend/`)

**Key Files:**
- `src/main.rs` - Server initialization
- `src/routes.rs` - API endpoints
- `src/handlers.rs` - Request handlers
- `src/engines.rs` - Processing logic
- `src/error.rs` - Error handling
- `src/state.rs` - App state

**Port:** 3000
**Framework:** Axum (async web framework)

### Frontend (`frontend-react/`)

**Key Files:**
- `src/main.tsx` - Entry point
- `src/routes/` - Page components
- `src/components/` - Reusable components
- `src/lib/api.ts` - API client
- `src/hooks/` - Custom hooks

**Port:** 8080
**Framework:** React 19 + TypeScript

### Shared (`shared/`)

**Key Files:**
- `src/operation.rs` - Operation enum
- `src/error.rs` - Error type

Shared types between backend and frontend.

## Development Workflow

### Working on Backend

```bash
cd backend

# Check syntax
cargo check

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt

# Run with logging
RUST_LOG=debug cargo run
```

### Working on Frontend

```bash
cd frontend-react

# Type check
npm run typecheck

# Lint
npm run lint

# Format
npm run format

# Run tests
npm test

# Build for production
npm run build
```

## API Testing

### Using curl

```bash
# Health check
curl http://localhost:3000/health

# Compress PDF
curl -X POST \
  -F "file=@document.pdf" \
  http://localhost:3000/api/pdf/compress
```

### Using Postman

1. Import postman collection (add to repo)
2. Select environment: Development
3. Run requests

## Troubleshooting

### Backend won't start

```bash
# Check if port 3000 is in use
lsof -i :3000

# Kill process using port
kill -9 <PID>

# Try custom port
SERVER_PORT=3001 cargo run
```

### Frontend won't start

```bash
# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Check Node version
node --version  # Should be 18+

# Try custom port
npm run dev -- --port 3001
```

### CORS errors

```bash
# Backend must allow frontend origin
# Set in backend .env:
CORS_ORIGIN=http://localhost:8080

# Or allow all (dev only):
# In main.rs, CorsLayer::new().allow_origin(Any)
```

### API connection failed

```bash
# Check backend is running
curl http://localhost:3000/health

# Check frontend .env.local has correct API URL
cat frontend-react/.env.local
# Should have: VITE_API_URL=http://localhost:3000
```

## Code Style

### Rust

- 4-space indentation
- Max line length: 100 chars
- Run `cargo fmt` before commit

### TypeScript/React

- 2-space indentation
- Max line length: 100 chars
- Use ESLint + Prettier
- Run `npm run format` before commit

## Git Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: describe what you did"

# Push and create PR
git push origin feature/my-feature
```

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: feat, fix, refactor, docs, style, test, chore

## Testing

### Backend Tests

```bash
cd backend
cargo test
cargo test -- --nocapture  # With output
```

### Frontend Tests

```bash
cd frontend-react
npm test
npm test -- --watch
```

### E2E Tests

```bash
cd frontend-react
npm run test:e2e
```

## Performance Profiling

### Backend

```bash
# Build with debug info
cargo build

# Profile with perf (Linux)
perf record ./target/debug/backend
perf report
```

### Frontend

```bash
# Chrome DevTools
# Open http://localhost:8080
# Press F12 → Performance
```

## Database Setup (Future)

```bash
# When database is added:
cd backend
rustup update
cargo install sqlx-cli
sqlx database create
sqlx migrate run
```

## Deployment Checklist

- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Environment variables documented
- [ ] README updated
- [ ] Version bumped
- [ ] CHANGELOG updated
- [ ] Security review completed
- [ ] Performance tested
- [ ] Deployment tested in staging

## Useful Resources

- [Axum Documentation](https://docs.rs/axum/)
- [React Documentation](https://react.dev)
- [TanStack Router](https://tanstack.com/router/latest)
- [Tailwind CSS](https://tailwindcss.com)
- [Radix UI](https://www.radix-ui.com)

## Getting Help

1. Check existing issues and discussions
2. Review error logs and stack traces
3. Search documentation
4. Ask in discussions or create an issue

## Next Steps

1. ✅ Implement backend routes and handlers
2. ✅ Build frontend components
3. ⏳ Add actual PDF operation implementations
4. ⏳ Add database persistence
5. ⏳ Add authentication
6. ⏳ Add comprehensive tests
7. ⏳ Deploy to production
