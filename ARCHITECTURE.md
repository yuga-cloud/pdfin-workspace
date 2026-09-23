# PDFin Workspace Architecture

## Repository ownership boundary

This repository is the source of truth for the full PDFin application.

```
pdfin-workspace
├── backend        # Axum services and processing logic
├── shared         # API contract, DTOs, shared domain types
└── frontend-react # React UI and frontend integration
```

## Frontend boundary

Frontend contributors should work inside:

```
frontend-react/
```

Frontend must consume backend capabilities through API contracts. Do not duplicate backend business rules in the frontend.

## Backend boundary

Backend owns:

- PDF processing
- file lifecycle
- validation enforcement
- conversion execution
- API implementation

## Shared boundary

Shared is the contract layer between frontend and backend.

It should contain:

- request DTOs
- response DTOs
- operation enums
- API error schemas

It should not contain application-specific processing logic.

## Integration rule

Changes that modify API shape must update shared contracts first, then backend and frontend clients.
