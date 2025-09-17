# Contributing to Rust AI IDE

## Welcome Contributors! 🎉

Thank you for your interest in contributing to the Rust AI IDE project! This document provides comprehensive guidelines for contributors, covering development setup, coding standards, cross-platform development, plugin development, and performance optimization.

## Table of Contents

- [Development Setup](#development-setup)
- [Cross-Platform Development](#cross-platform-development)
- [Coding Standards](#coding-standards)
- [Plugin Development](#plugin-development)
- [Performance Optimization](#performance-optimization)
- [Testing Guidelines](#testing-guidelines)
- [Security Guidelines](#security-guidelines)
- [Code Review Process](#code-review-process)

## Development Setup

### Prerequisites

#### System Requirements
- **Operating System**: Windows 10+, macOS 11+, Ubuntu 20.04+
- **Memory**: 16GB RAM minimum (32GB recommended for large workspaces)
- **Storage**: 50GB free space (for build artifacts and test data)
- **Network**: High-speed internet for dependency downloads

#### Toolchain Requirements
```bash
# Install Rust nightly toolchain
rustup toolchain install nightly-2025-09-03
rustup component add rust-src rustfmt clippy --toolchain nightly-2025-09-03
rustup default nightly-2025-09-03

# Verify installation
rustc --version  # Should show nightly-2025-09-03
cargo --version

# Install Node.js (18+)
# Download from https://nodejs.org/ or use nvm
node --version  # Should be 18+
npm --version

# Install SQLite development libraries
# Ubuntu/Debian:
sudo apt-get install libsqlite3-dev sqlite3
# macOS:
brew install sqlite3
# Windows: SQLite is bundled via libsqlite3-sys
```

### Project Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/rust-ai-ide/rust-ai-ide.git
   cd rust-ai-ide
   ```

2. **Initialize submodules (if any)**
   ```bash
   git submodule update --init --recursive
   ```

3. **Install dependencies**
   ```bash
   # Rust dependencies (automatically handled by Cargo)
   # Web frontend dependencies
   cd web && npm install && cd ..
   ```

4. **Verify setup**
   ```bash
   # Check Rust toolchain
   cargo check --workspace

   # Check web frontend
   cd web && npm run type-check && cd ..
   ```

### Development Workflow

#### Branching Strategy
```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Create bugfix branch
git checkout -b bugfix/issue-description

# Create hotfix branch (from main)
git checkout -b hotfix/critical-fix
```

#### Commit Guidelines
```bash
# Use conventional commits
git commit -m "feat: add new AI completion feature"
git commit -m "fix: resolve memory leak in cache layer"
git commit -m "perf: optimize LSP response time by 25%"
git commit -m "docs: update architecture documentation"
git commit -m "refactor: simplify async state management"
```

#### Pre-commit Hooks
```bash
# Install pre-commit hooks (if configured)
# These run automatically on commit and include:
# - Code formatting (cargo fmt)
# - Linting (cargo clippy)
# - Testing (cargo test)
# - Security scanning (cargo audit)
```

## Cross-Platform Development

### Platform-Specific Code Organization

#### Conditional Compilation
```rust
// Platform-specific implementations
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

// Cross-platform abstraction
pub trait PlatformFileSystem {
    fn create_temp_file(&self) -> Result<PathBuf, PlatformError>;
    fn get_app_data_dir(&self) -> Result<PathBuf, PlatformError>;
}

// Platform-specific implementations
#[cfg(target_os = "windows")]
impl PlatformFileSystem for WindowsFileSystem {
    fn create_temp_file(&self) -> Result<PathBuf, PlatformError> {
        // Windows-specific implementation
        use std::env;
        let temp_dir = env::temp_dir();
        // ... implementation
    }

    fn get_app_data_dir(&self) -> Result<PathBuf, PlatformError> {
        // Use APPDATA environment variable
        // ... implementation
    }
}
```

#### Build Configuration
```toml
# Cargo.toml - Platform-specific dependencies
[target.'cfg(target_os = "windows")'.dependencies]
winapi = "0.3"
windows = "0.48"

[target.'cfg(target_os = "macos")'.dependencies]
objc = "0.2"
cocoa = "0.24"

[target.'cfg(target_os = "linux")'.dependencies]
dbus = "0.9"
```

### Platform Testing Strategy

#### Automated Testing
```bash
# Run tests on all platforms via CI/CD
# GitHub Actions matrix configuration:
# - ubuntu-latest (x86_64)
# - windows-latest (x86_64)
# - macos-latest (x86_64, arm64)

# Run platform-specific tests
cargo test --features windows-tests  # Windows only
cargo test --features macos-tests    # macOS only
cargo test --features linux-tests    # Linux only
```

#### Manual Testing Checklist
- [ ] File system operations work correctly
- [ ] Native dialogs display properly
- [ ] Keyboard shortcuts match platform conventions
- [ ] Memory usage within acceptable limits
- [ ] Startup time meets performance targets
- [ ] Integration with platform-specific services

### Cross-Platform Best Practices

#### Path Handling
```rust
use std::path::{Path, PathBuf};

// Always use platform-independent path operations
pub fn normalize_path(path: &Path) -> PathBuf {
    // Use dunce crate for path normalization
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

// Avoid hard-coded path separators
pub fn join_paths(base: &Path, relative: &str) -> PathBuf {
    base.join(relative)  // Automatically uses correct separator
}
```

#### Environment Variables
```rust
// Use platform-appropriate environment variables
pub fn get_config_dir() -> Result<PathBuf, ConfigError> {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support"))
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.config", h)))
            .map(PathBuf::from)
    }
}
```

## Coding Standards

### Rust Guidelines Compliance

#### Official Rust Standards
- Follow the [Official Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for consistent code formatting
- Address all `clippy` warnings and suggestions
- Maintain 97%+ test coverage

#### Project-Specific Patterns

##### Async/Concurrency Patterns
```rust
// ✅ Correct: Always wrap async state in Arc<Mutex<T>>
pub struct AsyncService {
    state: Arc<Mutex<ServiceState>>,
}

impl AsyncService {
    pub async fn process(&self, data: Data) -> Result<Output, Error> {
        let mut state = self.state.lock().await;
        // Process data...
    }
}

// ✅ Correct: Use tokio::sync primitives
use tokio::sync::{mpsc, oneshot};

pub async fn handle_request(&self, request: Request) -> Result<Response, Error> {
    let (tx, rx) = oneshot::channel();

    // Spawn background task
    tokio::spawn(async move {
        let result = self.process_request(request).await;
        let _ = tx.send(result);
    });

    // Wait for result with timeout
    match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(result) => result?,
        Err(_) => Err(Error::Timeout),
    }
}

// ❌ Incorrect: Don't use std::sync for async contexts
// pub struct BadService {
//     state: Mutex<ServiceState>,  // No Arc - race conditions!
// }
```

##### Command Handling (Tauri)
```rust
// ✅ Correct: Use command templates
#[tauri_command_template]
pub async fn analyze_code(
    context: CommandContext,
    request: AnalyzeCodeRequest,
) -> Result<AnalyzeCodeResponse, IDEError> {
    // Input validation
    let sanitized_request = TauriInputSanitizer::sanitize(request)?;

    // Service acquisition
    let service = acquire_service_and_execute!(context, code_analysis_service)?;

    // Execute with retry logic
    execute_with_retry!(
        service.analyze_code(sanitized_request),
        max_attempts = 3,
        backoff_strategy = ExponentialBackoff
    )
}

// ✅ Correct: Validate inputs
pub struct AnalyzeCodeRequest {
    pub file_path: String,
    pub content: String,
}

impl Validate for AnalyzeCodeRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_secure_path(&self.file_path)?;
        if self.content.is_empty() {
            return Err(ValidationError::EmptyContent);
        }
        Ok(())
    }
}
```

### Error Handling Standards

```rust
// ✅ Correct: Use IDEError enum
use rust_ai_ide_common::error::IDEError;

pub async fn process_file(path: &Path) -> Result<FileData, IDEError> {
    // Validate input
    validate_secure_path(path)?;

    // Attempt operation
    match tokio::fs::read(path).await {
        Ok(content) => {
            // Process content
            let processed = process_content(&content)?;
            Ok(processed)
        }
        Err(e) => {
            // Map to IDEError
            Err(IDEError::Io {
                source: e,
                context: format!("Failed to read file: {}", path.display()),
            })
        }
    }
}

// ✅ Correct: Aggregate errors at boundaries
pub async fn batch_process_files(paths: &[PathBuf]) -> Result<BatchResult, IDEError> {
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for path in paths {
        match process_file(path).await {
            Ok(data) => results.push(data),
            Err(e) => errors.push(e),
        }
    }

    if errors.is_empty() {
        Ok(BatchResult::Success(results))
    } else if results.is_empty() {
        Err(IDEError::BatchFailed { errors })
    } else {
        Ok(BatchResult::Partial { results, errors })
    }
}
```

### Memory Management Standards

```rust
// ✅ Correct: Use memory pooling for large allocations
use rust_ai_ide_memory::pool::MemoryPool;

pub struct LargeDataProcessor {
    pool: MemoryPool<Vec<u8>>,
}

impl LargeDataProcessor {
    pub fn process(&self, data: &[u8]) -> Result<ProcessedData, Error> {
        // Get buffer from pool
        let mut buffer = self.pool.allocate(data.len())?;

        // Process data
        self.process_into_buffer(data, &mut buffer)?;

        // Return processed data (buffer will be returned to pool)
        Ok(ProcessedData::from_buffer(buffer))
    }
}

// ✅ Correct: Virtual memory for large workspaces
use rust_ai_ide_memory::virtual_memory::VirtualMemoryManager;

pub struct WorkspaceManager {
    vm_manager: VirtualMemoryManager,
}

impl WorkspaceManager {
    pub async fn load_workspace(&self, config: WorkspaceConfig) -> Result<Workspace, Error> {
        // Check if workspace fits in memory
        let estimated_size = self.estimate_workspace_size(&config);

        if estimated_size > self.vm_manager.available_memory() {
            // Use virtual memory mapping
            self.vm_manager.map_workspace_to_disk(&config.path).await
        } else {
            // Load directly into memory
            self.load_workspace_into_memory(&config).await
        }
    }
}
```

## Plugin Development

### Plugin Architecture Overview

#### WebAssembly Runtime
```rust
// Plugin interface definition
#[wasm_bindgen]
pub struct PluginHost {
    runtime: wasmtime::Engine,
    store: wasmtime::Store<PluginContext>,
}

#[wasm_bindgen]
impl PluginHost {
    pub fn load_plugin(&mut self, wasm_bytes: &[u8]) -> Result<(), PluginError> {
        // Validate plugin
        self.validate_plugin(wasm_bytes)?;

        // Instantiate plugin
        let module = Module::from_binary(&self.runtime, wasm_bytes)?;
        let instance = Instance::new(&mut self.store, &module, &[])?;

        // Register plugin
        self.register_plugin_instance(instance)?;

        Ok(())
    }
}
```

#### Plugin API Definition
```typescript
// TypeScript definitions for plugins
export interface PluginAPI {
  // File system access
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, content: Uint8Array): Promise<void>;

  // LSP integration
  registerLanguageServer(server: LanguageServerConfig): Promise<void>;
  sendLSPRequest(request: LSPRequest): Promise<LSPResponse>;

  // UI integration
  registerCommand(command: CommandConfig): Promise<void>;
  showNotification(notification: NotificationConfig): Promise<void>;

  // Configuration
  getConfiguration(section: string): Promise<any>;
  updateConfiguration(section: string, value: any): Promise<void>;
}
```

### Plugin Development Workflow

#### 1. Create Plugin Structure
```rust
// lib.rs
use rust_ai_ide_plugin_api::*;

#[derive(Default)]
pub struct MyPlugin {
    config: PluginConfig,
}

impl Plugin for MyPlugin {
    fn initialize(&mut self, api: &PluginAPI) -> Result<(), PluginError> {
        // Register commands
        api.register_command(CommandConfig {
            id: "my-plugin.analyze",
            title: "Analyze with My Plugin",
            handler: Self::handle_analyze,
        })?;

        // Register language server
        api.register_language_server(LanguageServerConfig {
            language: "rust",
            command: vec!["my-language-server".to_string()],
            args: vec![],
        })?;

        Ok(())
    }

    fn dispose(&mut self) -> Result<(), PluginError> {
        // Cleanup resources
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(MyPlugin::default())
}
```

#### 2. Build Plugin
```toml
# Cargo.toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # WebAssembly compatible

[dependencies]
rust-ai-ide-plugin-api = "3.4"
serde = { version = "1.0", features = ["derive"] }
wasm-bindgen = "0.2"
```

#### 3. Test Plugin
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_ai_ide_plugin_api::test_utils::*;

    #[test]
    fn test_plugin_initialization() {
        let mut plugin = MyPlugin::default();
        let api = MockPluginAPI::new();

        assert!(plugin.initialize(&api).is_ok());
        assert!(api.has_command("my-plugin.analyze"));
    }

    #[test]
    fn test_plugin_functionality() {
        // Test plugin commands and integrations
    }
}
```

### Plugin Security Guidelines

#### Sandboxing Requirements
```rust
// Plugin must run in isolated environment
pub struct PluginSandbox {
    memory_limit: usize,
    cpu_limit: u64,
    allowed_syscalls: HashSet<String>,
}

impl PluginSandbox {
    pub fn execute_plugin(&self, plugin: &dyn Plugin, input: PluginInput) -> Result<PluginOutput, SecurityError> {
        // Set resource limits
        self.set_memory_limit(plugin, self.memory_limit)?;
        self.set_cpu_limit(plugin, self.cpu_limit)?;

        // Execute in sandbox
        self.execute_in_sandbox(plugin, input)
    }
}
```

#### Input Validation
```rust
impl PluginInput {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate file paths
        for path in &self.file_paths {
            validate_secure_path(path)?;
        }

        // Validate command arguments
        for arg in &self.args {
            if arg.contains("..") || arg.contains("/") {
                return Err(ValidationError::InvalidPath);
            }
        }

        Ok(())
    }
}
```

## Performance Optimization Guidelines

### Memory Optimization

#### Object Pooling Pattern
```rust
use rust_ai_ide_memory::pool::ObjectPool;

pub struct PerformanceOptimizer {
    string_pool: ObjectPool<String>,
    vec_pool: ObjectPool<Vec<u8>>,
}

impl PerformanceOptimizer {
    pub fn process_data(&self, input: &[u8]) -> Result<String, Error> {
        // Get string from pool
        let mut result = self.string_pool.acquire()?;

        // Process data
        self.process_into_string(input, &mut result)?;

        Ok(result)
    }
}
```

#### Cache Optimization
```rust
use moka::future::Cache;
use std::time::Duration;

pub struct IntelligentCache<K, V> {
    l1_cache: Cache<K, V>,
    l2_cache: Option<RedisCache<K, V>>,
    ttl_predictor: MLTtlPredictor,
}

impl<K, V> IntelligentCache<K, V> {
    pub async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        // Try L1 cache first
        if let Some(value) = self.l1_cache.get(key).await {
            return Ok(Some(value));
        }

        // Try L2 cache
        if let Some(ref l2) = self.l2_cache {
            if let Some(value) = l2.get(key).await? {
                // Warm L1 cache
                self.l1_cache.insert(key.clone(), value.clone()).await;
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    pub async fn insert(&self, key: K, value: V) -> Result<(), CacheError> {
        // Predict optimal TTL
        let predicted_ttl = self.ttl_predictor.predict_ttl(&key, &value);

        // Insert into L1 with predicted TTL
        self.l1_cache.insert_with_ttl(key.clone(), value.clone(), predicted_ttl).await;

        // Insert into L2 if available
        if let Some(ref l2) = self.l2_cache {
            l2.insert_with_ttl(key, value, predicted_ttl * 2).await?;
        }

        Ok(())
    }
}
```

### Async Optimization

#### Work Stealing Implementation
```rust
use tokio::runtime::Handle;
use std::sync::Arc;

pub struct WorkStealingExecutor {
    workers: Vec<Worker>,
    global_queue: Arc<Mutex<VecDeque<Task>>>,
}

impl WorkStealingExecutor {
    pub fn spawn_task<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // Select optimal worker
        let worker = self.select_worker();

        // Spawn on selected worker
        worker.spawn_task(future)
    }

    fn select_worker(&self) -> &Worker {
        // Use work-stealing algorithm to select least-loaded worker
        self.workers
            .iter()
            .min_by_key(|w| w.queue_size())
            .unwrap()
    }
}
```

#### SIMD and Vectorization
```rust
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn vectorized_sum(values: &[f32]) -> f32 {
    use std::arch::x86_64::*;

    let mut sum = 0.0f32;
    let chunks = values.chunks_exact(8);

    // Process 8 values at once using SIMD
    for chunk in chunks {
        unsafe {
            let vector = _mm256_loadu_ps(chunk.as_ptr());
            let vector_sum = _mm256_hadd_ps(vector, vector);
            sum += _mm256_cvtss_f32(_mm256_castps256_ps128(vector_sum));
        }
    }

    // Handle remaining values
    for &value in chunks.remainder() {
        sum += value;
    }

    sum
}
```

## Testing Guidelines

### Unit Testing Standards
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_async_operation() {
        let service = TestService::new();
        let result = service.perform_operation().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }

    #[test]
    fn test_error_handling() {
        let service = TestService::new();

        // Test error conditions
        let result = service.handle_invalid_input();
        assert!(matches!(result, Err(IDEError::InvalidInput { .. })));
    }

    #[test]
    fn test_memory_usage() {
        let service = TestService::new();

        // Measure memory usage
        let initial_memory = get_current_memory_usage();
        let _result = service.process_large_data();
        let final_memory = get_current_memory_usage();

        // Assert memory usage is within limits
        assert!(final_memory - initial_memory < MAX_MEMORY_DELTA);
    }
}
```

### Integration Testing
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn test_database_integration() {
        let docker = Cli::default();
        let database = docker.run(Postgres::default());

        // Test database operations
        let repo = DatabaseRepository::new(database.url().await?);
        let result = repo.save_test_data().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_external_api_integration() {
        // Mock external API for testing
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/data")
            .with_status(200)
            .with_body(r#"{"data": "test"}"#)
            .create();

        // Test API integration
        let client = APIClient::new(server.url());
        let result = client.fetch_data().await;

        assert!(result.is_ok());
        mock.assert();
    }
}
```

### Performance Testing
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn benchmark_code_analysis(c: &mut Criterion) {
        let analyzer = CodeAnalyzer::new();
        let test_code = generate_large_test_file();

        c.bench_function("analyze_large_file", |b| {
            b.iter(|| {
                black_box(analyzer.analyze(black_box(&test_code)))
            })
        });
    }

    fn benchmark_memory_usage(c: &mut Criterion) {
        c.bench_function("memory_allocation", |b| {
            b.iter(|| {
                let data = black_box(vec![0u8; 1024 * 1024]); // 1MB
                black_box(process_data(&data));
            })
        });
    }

    criterion_group!(benches, benchmark_code_analysis, benchmark_memory_usage);
    criterion_main!(benches);
}
```

## Security Guidelines

### Input Validation
```rust
use rust_ai_ide_common::validation::TauriInputSanitizer;

pub async fn process_user_input(input: UserInput) -> Result<ProcessedData, Error> {
    // Sanitize all user inputs
    let sanitized_input = TauriInputSanitizer::sanitize(input)?;

    // Validate file paths
    validate_secure_path(&sanitized_input.file_path)?;

    // Validate command arguments
    for arg in &sanitized_input.args {
        if contains_shell_metacharacters(arg) {
            return Err(Error::InvalidArgument);
        }
    }

    Ok(process_sanitized_input(sanitized_input))
}
```

### Secure Storage
```rust
use rust_ai_ide_security::storage::SecureStorage;

pub struct CredentialManager {
    storage: SecureStorage,
}

impl CredentialManager {
    pub async fn store_credentials(&self, service: &str, credentials: Credentials) -> Result<(), SecurityError> {
        // Encrypt credentials
        let encrypted = self.storage.encrypt(&credentials)?;

        // Store securely
        self.storage.store(service, &encrypted).await?;

        // Audit log
        audit_logger::log_security_event(
            SecurityEvent::CredentialStored { service: service.to_string() }
        ).await;

        Ok(())
    }
}
```

## Code Review Process

### Review Checklist
- [ ] **Functionality**: Code works as intended
- [ ] **Performance**: No performance regressions
- [ ] **Security**: Security best practices followed
- [ ] **Testing**: Adequate test coverage
- [ ] **Documentation**: Code is well-documented
- [ ] **Style**: Follows coding standards
- [ ] **Architecture**: Follows architectural patterns

### Automated Checks
```yaml
# GitHub Actions workflow
name: Code Review
on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Check formatting
        run: cargo fmt --check

      - name: Run linter
        run: cargo clippy -- -D warnings

      - name: Run tests
        run: cargo test --workspace

      - name: Check security
        run: cargo audit

      - name: Check coverage
        run: cargo tarpaulin --workspace --out Lcov

      - name: Performance regression test
        run: cargo bench --workspace
```

### Review Guidelines for Reviewers
1. **Be Constructive**: Focus on improvement, not criticism
2. **Explain Reasoning**: Provide context for suggestions
3. **Prioritize Issues**: Address critical issues first
4. **Mentor Contributors**: Help contributors learn and improve
5. **Follow Up**: Ensure issues are resolved before merging

---

Thank you for contributing to the Rust AI IDE! Your contributions help make this project better for everyone. If you have any questions about these guidelines, please don't hesitate to ask in our [community discussions](https://github.com/rust-ai-ide/rust-ai-ide/discussions).