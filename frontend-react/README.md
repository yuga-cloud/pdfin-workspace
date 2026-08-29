# PDFin Workspace - Frontend (React)

High-performance PDF & Office document processing web interface built with React 19, TypeScript, and modern tooling.

## Features

- 📁 **File Upload** - Drag & drop or click to upload
- 🔄 **14+ Operations** - PDF compression, merge, split, conversion, watermarking, and more
- ⚡ **Real-time Processing** - Powered by Rust backend via Axum
- 🎨 **Modern UI** - Built with Radix UI and Tailwind CSS
- 📊 **TypeScript** - Full type safety
- ♿ **Accessible** - WCAG compliant components

## Tech Stack

- **Framework**: React 19 + TypeScript
- **Router**: TanStack Router v1
- **State**: TanStack Query (React Query)
- **UI Components**: Radix UI
- **Styling**: Tailwind CSS + CVA
- **Forms**: React Hook Form + Zod
- **Build**: Vite
- **Icons**: Lucide React

## Quick Start

### Prerequisites
- Node.js 18+
- npm or yarn

### Installation

```bash
npm install
```

### Development

```bash
# Start development server (http://localhost:8080)
npm run dev

# Type check
npm run typecheck

# Lint
npm run lint

# Format
npm run format
```

### Production

```bash
# Build
npm run build

# Preview
npm run preview
```

## Configuration

Create `.env.local` based on `.env.example`:

```bash
cp .env.example .env.local
```

Key variables:
- `VITE_API_URL` - Backend API URL (default: http://localhost:3000)

## Project Structure

```
src/
├── components/       # Reusable React components
│   ├── PdfUpload.tsx
│   └── OperationSelector.tsx
├── hooks/           # Custom React hooks
│   └── useFileUpload.ts
├── lib/             # Utilities
│   └── api.ts       # API client with Zod validation
├── routes/          # TanStack Router pages
│   ├── __root.tsx
│   └── index.lazy.tsx
├── main.tsx         # Entry point
└── index.css        # Global styles
```

## API Integration

The frontend communicates with the backend via REST API:

```typescript
// Example usage
import { processPdf } from '@/lib/api'

const response = await processPdf(file, 'compress')
// response: { success: boolean, message: string, file_size?: number }
```

## Components

### PdfUpload
File upload component with drag & drop support.

```tsx
<PdfUpload operation="compress" />
```

### OperationSelector
Dropdown to select PDF operation.

```tsx
<OperationSelector selected={op} onSelect={setOp} />
```

## Styling

- **Tailwind CSS** for utility-first styling
- **CVA (Class Variance Authority)** for component variants
- **Radix UI** for unstyled, accessible components

## Testing

```bash
# Run tests
npm test

# Run tests in watch mode
npm test -- --watch

# Run e2e tests
npm run test:e2e
```

## Performance Tips

- Use `React.lazy()` for route splitting
- Memoize expensive computations with `useMemo`
- Optimize re-renders with `useCallback`
- Use React Query for efficient data fetching

## Troubleshooting

### Port Already in Use
```bash
npm run dev -- --port 3001
```

### CORS Errors
Ensure backend CORS is configured:
```bash
# Backend should allow frontend origin
CORS_ORIGIN=http://localhost:8080
```

## Contributing

1. Create a feature branch
2. Make your changes
3. Run `npm run lint` and `npm run format`
4. Submit a pull request

## License

MIT
