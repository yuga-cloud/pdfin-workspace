# App Builder Workspace

Clean project workspace with a tidy folder structure.

## Structure

```
├── migrations/     # Database migrations
├── public/         # Static assets
│   └── __app/      # App install / PWA assets
├── scripts/        # Build & utility scripts
├── server/         # Server middleware
├── src/            # Application source
│   ├── components/
│   ├── lib/
│   └── routes/
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Getting started

```bash
npm install
npm run dev
```

## Scripts

- `npm run dev` – start development server
- `npm run build` – production build
- `npm run typecheck` – TypeScript check
- `npm run lint` – ESLint
