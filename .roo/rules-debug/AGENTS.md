# Project Debug Rules (Non-Obvious Only)

## Webview Debugging
- Webview dev tools accessed via Command Palette > "Developer: Open Webview Developer Tools" (not F12)
- Webview isolation prevents use of localStorage, Cookies, most browser APIs - debugging must account for this
- React hooks will fail for external state libraries due to webview isolation

## IPC Communication
- IPC messages fail silently if not wrapped in try/catch in packages/ipc/src/
- Webview and extension communicate through specific IPC channel patterns only - manual IPC calls will fail
- Tauri commands have extensive placeholder implementations - debugging shows dummy data in tests

## Production Debugging
- Production builds require NODE_ENV=production or features break without error
- AI LSP service initialization required before features work - debugging without this shows stubs
- Cloud integrations require webhook system on port 3000 - missing causes silent failures

## Database Debugging
- Database migrations must be run from specific directories, not root (storage/init)
- SQLite connections must use bundled library - system SQLite causes version conflicts
- Raw SQL forbidden - database issues mask as driver problems if using raw queries

## Network Debugging
- Only crates.io registry and approved GitHub orgs allowed
- All HTTP requests must use reqwest with rustls-tls - other TLS libraries banned
- Webhook debugging requires separate webhook system on port 3000

## Extension Protocol Debugging
- LSP messages logged to "Extension Host" output channel, not Debug Console
- AI/ML operations through LSP service only - direct model access will panic
- Federated learning prohibited - local AI/ML processing only

## Security Debugging
- Path traversal vulnerabilities prevented by validate_secure_path() - failing paths aren't logged
- Command injection protection uses TauriInputSanitizer - sanitized failures are silent
- Audit logging in security crate - security events don't appear in standard logs

## Dependency Debugging
- Cargo dependency graph uses custom graph_export_utf8_bytes() - standard cargo graph commands fail
- License compliance checked through spdx crate - license violations don't cause build errors
- Vulnerability scanning uses rustsec integration - custom scanners fail silently