# Security Policy

## Reporting Security Issues

If you discover a security vulnerability in PDFin Workspace, please email security@example.com instead of using the issue tracker.

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge your report within 48 hours and provide status updates as we work on a fix.

## Security Best Practices

### For Users

1. **Keep dependencies updated** - Regularly update Rust and npm packages
2. **Validate uploads** - Verify file types before processing
3. **Use HTTPS** - Always use encrypted connections in production
4. **Enable CORS** - Configure CORS to only allow trusted origins
5. **Monitor logs** - Review server logs for suspicious activity

### For Developers

1. **Input validation** - Always validate file uploads and request data
2. **Error handling** - Don't expose sensitive information in error messages
3. **Dependencies** - Keep dependencies up to date and audit regularly
4. **Code review** - All changes should be peer reviewed
5. **Testing** - Write security tests for sensitive operations

## Known Security Considerations

### Backend

- File upload size limit: 50MB (configurable)
- Request timeout: 120 seconds
- No authentication implemented (can be added)
- CORS: Allow All by default (change for production)
- File type validation: Minimal (improve in production)

### Frontend

- Client-side validation only (server should validate too)
- No authentication implemented
- File type check via extension only
- No content security policy (add in production)

## Deployment Security Checklist

- [ ] Set `CORS_ORIGIN` to specific domain
- [ ] Set `RUST_LOG=warn` (reduce logging verbosity)
- [ ] Use HTTPS/TLS
- [ ] Enable rate limiting
- [ ] Implement authentication
- [ ] Add file type validation on server
- [ ] Set appropriate file size limits
- [ ] Enable security headers
- [ ] Regular security audits
- [ ] Monitor for CVEs in dependencies

## Supported Versions

| Version | Supported |
|---------|----------|
| 0.1.x   | ✅ Yes   |
| < 0.1   | ❌ No    |

## Dependencies

We use the following key dependencies. Check their security advisories:

- Rust: Ecosystem via `cargo audit`
- Node.js: Ecosystem via `npm audit`

## Security Updates

We will issue security updates for:
- Critical vulnerabilities (CVSS 9-10)
- High vulnerabilities (CVSS 7-8.9) affecting common use cases
- Medium vulnerabilities (CVSS 4-6.9) with significant impact

Updates will be released as soon as possible after discovery.

## Contact

- Security Issues: security@example.com
- General Questions: issues@example.com

Thank you for helping keep PDFin Workspace secure! 🔒
