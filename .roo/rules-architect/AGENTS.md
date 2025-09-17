# Project Architecture Rules (Non-Obvious Only)

## Module Coupling
- Providers MUST be stateless - hidden caching layer assumes this (infrastructure/infra.rs)
- Circular dependencies permitted in types packages (intentional for shared types across crates)
- EventBus is the only pub-sub mechanism - no custom event systems allowed
- AI/ML processing always happens locally - no cloud federation or distributed learning

## Communication Patterns
- Webview and extension communicate through strict IPC channel patterns only - no custom IPC
- Tauri commands use double-locking for async state initialization - pattern mandatory
- Background tasks require spawn_background_task! macro for proper cleanup
- Connection pooling used for all external connections - ConnectionPool<T> mandatory

## State Management Constraints
- All async state wrapped in Arc<Mutex<T>> or RwLock<T> to prevent race conditions
- Double-locking patterns for lazy service initialization are required
- Rate limiter and connection pooling architecturally enforced via infra module
- State access follows lazy initialization - don't reuse existing Arc references

## Database Architecture
- SQLite migrations forward-only by design - no rollbacks allowed
- Version conflicts prevented by workspace-level enforce_same_version for SQLite libs
- Raw SQL forbidden - all queries through cargo_metadata or dependency analysis functions
- Connection pooling mandatory for all database connections

## Security Architecture
- Path validation through validate_secure_path() - all user paths validated
- Command injection protected by TauriInputSanitizer - input sanitization required
- Forbidden crates (openssl, md5, ring, quick-js) banned for security reasons
- Audit logging required for all sensitive operations via security crate

## Performance Patterns
- Large workspaces (>1M LOC) require virtual memory management - don't load entire codebases
- File watching uses debouncing for change coalescing (configurable in FileWatcher)
- Caching layers use Moka LRU with TTL - TTL eviction policy mandatory
- Memory profiling in utils/performance_testing.rs but only for debugging

## Platform Constraints
- React hooks forbidden for external state libraries due to webview isolation
- Webview ES modules isolated from extension - no unified state patterns
- Type generation automated through Cargo binary - manual TS interface writing forbidden
- No localStorage, Cookies, or most web APIs due to webview restrictions

## AI/ML Architecture
- Model loading/unloading through LSP service only - direct model access forbidden
- Hyperparameter tuning through specific ai-learning crates only
- Offline mode requires pre-downloaded models - cloud backing forbidden
- Federated learning prohibited - all AI/ML processing happens locally

## Lifecycle Management
- Lifecycle manager enforces correct initialization order for services
- Async initialization mandatory for AI LSP, webhooks on port 3000
- Services start in specific phases - lifecycle management is architecture-level
- Cleanup integrated into lifecycle stages - explicit cleanup required