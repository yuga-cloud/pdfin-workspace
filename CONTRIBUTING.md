# Contributing to PDFin Workspace

Thank you for your interest in contributing! This document provides guidelines and instructions.

## Code of Conduct

- Be respectful and inclusive
- No harassment or discrimination
- Constructive feedback only
- Help others learn and grow

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch
4. Make your changes
5. Test thoroughly
6. Submit a pull request

## Development Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed setup instructions.

## Coding Standards

### Rust
- Format with `cargo fmt`
- Lint with `cargo clippy`
- Write tests for new functionality
- Document public APIs
- Use meaningful variable names
- Keep functions small and focused

### TypeScript/React
- Format with Prettier
- Lint with ESLint
- Write tests for components
- Use type safety (no `any`)
- Follow React best practices
- Use meaningful component names

## Commit Messages

Follow conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `docs`: Documentation
- `style`: Code style changes
- `test`: Add/modify tests
- `chore`: Build, dependencies, etc.

Example:
```
feat(backend): add PDF compression operation

Implement compress_pdf function using lopdf library.
Adds compression endpoint at POST /api/pdf/compress.

Closes #123
```

## Pull Request Process

1. **Branch naming**: `feature/description` or `fix/description`
2. **Title**: Clear, descriptive PR title
3. **Description**: Explain what and why
4. **Tests**: Add tests for your changes
5. **Linting**: Ensure code passes linting
6. **Documentation**: Update docs if needed
7. **Review**: Address feedback constructively

## Testing

### Backend
```bash
cd backend
cargo test
cargo clippy
cargo fmt --check
```

### Frontend
```bash
cd frontend-react
npm test
npm run lint
npm run typecheck
```

## Documentation

- Update README.md if changing features
- Update DEVELOPMENT.md if changing setup
- Add inline comments for complex logic
- Document public APIs
- Update CHANGELOG.md

## Areas for Contribution

### High Priority
- Implement PDF operations (compress, merge, split, etc.)
- Add comprehensive tests
- Add error handling improvements
- Add performance optimizations

### Medium Priority
- Add database integration
- Add user authentication
- Add rate limiting
- Add caching

### Low Priority
- UI improvements
- Documentation improvements
- Code cleanup
- Dependency updates

## Reporting Issues

### Bug Reports
1. Check existing issues first
2. Provide clear title and description
3. Include reproduction steps
4. Share error messages and logs
5. Specify OS and versions

### Feature Requests
1. Check existing issues first
2. Describe the use case
3. Explain the benefit
4. Suggest implementation approach

## Questions?

- Check existing issues and discussions
- Review DEVELOPMENT.md
- Look at code examples
- Ask in GitHub Discussions

## License

By contributing, you agree your code will be licensed under MIT.

Thank you for contributing to PDFin Workspace! 🎉
