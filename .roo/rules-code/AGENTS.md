# Project Coding Rules (Non-Obvious Only)

## Async/Concurrency Patterns
- Always wrap async state access in Arc<Mutex<T>> or RwLock<T> to prevent race conditions (seen in AI service initialization)
- Use tokio::sync::mpsc or oneshot for one-off async communication between components
- EventBus pattern (in infra.rs) is the standard for pub-sub communication - don't implement custom event systems
- Background tasks must use spawn_background_task! macro from command_templates.rs for proper cleanup

## Command Handling (Tauri)
- Always use tauri_command_template! and acquire_service_and_execute! macros for new commands (standardizes error handling and state acquisition)
- Input validation required - use TauriInputSanitizer from rust-ai-ide-common for all user inputs
- Placeholder command implementations return dummy data like serde_json::json!({"status": "ok"}) - check if real implementation exists
- State access follows double-locking pattern for lazy initialization - implement your own classifiers don't reuse existing Arc references

## Security Programming
- No plain text secrets - must use secure storage via security crate
- Path traversal attacks - validate all file paths through validate_secure_path() from common validation
- Command injection protection - use sanitized command args from TauriInputSanitizer
- Audit logging required for sensitive operations - use audit_logger from security crate

## Database Patterns
- SQLite version conflicts prevented by workspace-level enforce_same_version=true for libsqlite3-sys and rusqlite
- Raw SQL queries forbidden - use cargo_metadata or dependency analysis functions instead
- Migration scripts must be forward-only (no rollbacks) by architectural constraint
- Connection pooling mandatory through ConnectionPool<T> from infra.rs

## Error Handling
- Ok return type favored over ? - errors aggregated and returned at function boundaries
- IDEError enum covers all error cases - extend if needed but don't create new error types
- Context printing disabled by default - errors handled silently unless explicitly logged
- Retry logic required for external API calls - use execute_with_retry() from command_templates.rs

## Memory Management
- Large workspaces (>1M LOC) require virtual memory management - don't load entire projects into RAM
- File watching coalesces changes via debouncing (configurable in FileWatcher struct)
- Caching layers use Moka LRU cache with TTL - eviction policy is TTL-based not LRU
- Memory profiling tools in utils/performance_testing.rs but primarily for debugging

## AI/ML Integration
- Model loading/unloading happens through LSP service - direct model access forbidden
- Hyperparameter tuning restricted to specific tuning pipelines in ai-learning crate
- Offline mode supported but requires pre-downloaded models - cloud backing forbidden
- Federated learning prohibited - all AI/ML processing happens locally

## Cargo Integration
- Dependency graph visualization uses custom graph_export_utf8_bytes() function
- Vulnerability scanning uses rustsec integration - custom scanning forbidden
- License compliance checked through spdx crate - custom parsers not allowed

## Web/Webview Patterns
- React hooks forbidden for external state libraries due to webview isolation (tokenizers, LSP, etc.)
- Type generation runs cargo bin - don't manually write TS interfaces
- Suite-WebView isolation prevents use of localStorage, Cookies, most web APIs