# Troubleshooting Guide for Modern Rust AI IDE Architecture

This comprehensive troubleshooting guide covers common issues encountered when working with the modern architecture implemented during the refactoring, including unified error handling, async patterns, security measures, and performance optimizations.

## Table of Contents

- [Unified Error Handling Issues](#unified-error-handling-issues)
- [Async/Batch Processing Problems](#asyncbatch-processing-problems)
- [Security Validation Failures](#security-validation-failures)
- [Performance Degradation](#performance-degradation)
- [Memory and Resource Issues](#memory-and-resource-issues)
- [Compilation and Build Issues](#compilation-and-build-issues)
- [Testing Infrastructure Problems](#testing-infrastructure-problems)

## Unified Error Handling Issues

### Compilation Errors with IDEResult

**Issue**: `"the trait bound 'std::io::Error: Into<IDEError>' is not satisfied"`

**Solution**: Use explicit error conversion for third-party errors:

```rust
use rust_ai_ide_errors::{IDEError, IDEResult};

// ❌ Wrong
pub fn read_file(path: &str) -> IDEResult<String> {
    std::fs::read_to_string(path)
}

// ✅ Correct
pub fn read_file(path: &str) -> IDEResult<String> {
    std::fs::read_to_string(path)
        .map_err(|e| IDEError::Io(format!("Failed to read '{}': {}", path, e)))
}
```

**Issue**: Mixed error types in the same function

**Solution**: Standardize all errors to IDEResult:

```rust
// ✅ Consistent error handling
pub fn complex_operation() -> IDEResult<String> {
    let data = load_data().await?;
    let processed = validate_data(&data)?;
    let result = transform_data(processed)?;

    Ok(result)
}
```

### Error Context Issues

**Issue**: Unclear error messages making debugging difficult

**Solution**: Add context to errors using helper methods:

```rust
use rust_ai_ide_errors::{IDEError, IDEResult, ContextualError};

pub fn process_user_file(user_id: &str, filename: &str) -> IDEResult<String> {
    validate_filename(filename)
        .with_context(format!("Validating filename '{}' for user {}", filename, user_id))?;

    let content = load_file_content(filename)
        .with_context(format!("Loading file '{}' for user {}", filename, user_id))?;

    Ok(content)
}
```

## Async/Batch Processing Problems

### "Future cannot be sent between threads safely"

**Issue**: Attempting to send non-Send types across await points

**Solution**: Ensure all captured variables implement Send:

```rust
// ❌ Problem: Rc<T> is not Send
use std::rc::Rc;

async fn problematic_future() {
    let data = Rc::new(String::from("test"));
    some_async_operation().await;
    println!("{}", data);
}

// ✅ Solution: Use Arc<T> for shared ownership
use std::sync::Arc;

async fn working_future() {
    let data = Arc::new(String::from("test"));
    some_async_operation().await;
    println!("{}", data);
}
```

### Deadlocks in Async Code

**Issue**: Holding locks across await points causes deadlocks

**Solution**: Minimize lock scope and avoid holding locks during async operations:

```rust
// ❌ Dangerous: Holding lock across await
async fn deadlock_prone(shared_data: Arc<RwLock<Vec<String>>>) {
    let mut data = shared_data.write().await;
    data.push("item".to_string());

    some_async_operation().await; // Deadlock risk!
    // Lock still held here
}

// ✅ Safe: Release lock before await
async fn deadlock_free(shared_data: Arc<RwLock<Vec<String>>>) {
    {
        let mut data = shared_data.write().await;
        data.push("item".to_string());
        // Lock released at end of scope
    }

    some_async_operation().await; // Safe to await
}
```

### Task Cancellation Issues

**Issue**: Long-running tasks don't respond to cancellation signals

**Solution**: Use tokio's select! macro for cooperative cancellation:

```rust
use tokio::select;
use tokio_util::sync::CancellationToken;

pub async fn cancellable_operation(
    token: &CancellationToken
) -> IDEResult<String> {
    select! {
        result = perform_work() => {
            Ok(result?)
        }
        _ = token.cancelled() => {
            Err(IDEError::Concurrency("Operation was cancelled".into()))
        }
    }
}
```

## Security Validation Failures

### Path Traversal Vulnerabilities

**Issue**: Security validation rejecting legitimate file paths

**Solution**: Ensure proper path normalization before validation:

```rust
use rust_ai_ide_security::validation::ValidatedFilePath;
use std::path::Path;

// ❌ Wrong: Raw path validation
let path = "../../../etc/passwd";
let validated = ValidatedFilePath::new(path, "file_read"); // Security error!

// ✅ Correct: Normalize path first
let normalized = Path::new(path).canonicalize()
    .map_err(|e| IDEError::Io(format!("Path normalization failed: {}", e)))?;

let validated = ValidatedFilePath::new(normalized.to_string_lossy().as_ref(), "file_read");
```

### Input Sanitization Issues

**Issue**: Overly aggressive sanitization removing valid content

**Solution**: Use appropriate sanitization levels for different contexts:

```rust
use rust_ai_ide_security::validation::SanitizedString;

// For file paths: minimal sanitization
let path_input = SanitizedString::new(user_path, 4096)?;

// For code content: balanced sanitization
let code_input = SanitizedString::new(user_code, 10000)?;

// For HTML display: strict sanitization
let display_text = SanitizedString::new(user_text, 1000)?;
```

### Command Injection Prevention

**Issue**: Legitimate commands being rejected as dangerous

**Solution**: Properly escape arguments without over-sanitization:

```rust
use rust_ai_ide_security::command::{SecureCommand, SecureArg};

// ✅ Correct: Use SecureArg for automatic escaping
let cmd = SecureCommand::new("grep", vec![
    SecureArg::new("-r"),
    SecureArg::new(pattern), // Automatically escaped
    SecureArg::validated(path)?, // Path validated
])?;
```

## Performance Degradation

### Cache Performance Issues

**Issue**: Cache hit ratio dropping causing performance regression

**Solution**: Implement proper cache key generation and sizing:

```rust
use rust_ai_ide_cache::{CacheConfig, InMemoryCache, EvictionPolicy};

// ✅ Optimized cache configuration
let config = CacheConfig {
    max_entries: Some(10000),
    eviction_policy: EvictionPolicy::Lfu, // Least Frequently Used
    default_ttl: Some(Duration::from_secs(1800)),
    max_memory_mb: Some(512),
    ..Default::default()
};

let cache = InMemoryCache::new(&config);
```

**Issue**: Cache key collisions causing incorrect results

**Solution**: Use structured key generation:

```rust
use rust_ai_ide_cache::key_utils;

// ✅ Structured keys prevent collisions
let key = key_utils::structured_key(&[
    &file_path.to_string(),
    &analysis_type.to_string(),
    &config_hash.to_string(),
]);
```

### Memory Usage Spikes

**Issue**: Application memory usage growing continuously

**Solution**: Implement proper cleanup and monitor memory usage:

```rust
use rust_ai_ide_common::memory::{MemoryMonitor, CleanupPolicy};

// Regular cleanup of temporary resources
let cleanup_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    loop {
        interval.tick().await;
        perform_memory_cleanup().await?;
    }
});

// Monitor memory usage
let monitor = MemoryMonitor::new(1024 * 1024 * 1024); // 1GB limit
if monitor.is_near_limit() {
    warn!("Memory usage critical, triggering cleanup");
    force_cleanup().await?;
}
```

### Slow Async Operations

**Issue**: Async functions taking unexpectedly long to complete

**Solution**: Add proper timeout and monitoring:

```rust
use tokio::time::{timeout, Duration};
use rust_ai_ide_common::time_operation;

pub async fn monitored_operation() -> IDEResult<String> {
    let (result, timing) = time_operation!("monitored_operation", async {
        timeout(Duration::from_secs(30), actual_work()).await
    })?;

    if timing > Duration::from_millis(1000) {
        warn!("Operation took {}ms, consider optimization", timing.as_millis());
    }

    result.map_err(|_| IDEError::Timeout("Operation timed out".into()))
}
```

## Memory and Resource Issues

### Connection Pool Exhaustion

**Issue**: All connections in pool become unavailable

**Solution**: Implement connection health checks and recovery:

```rust
pub async fn with_connection_health_check<F, Fut>(
    pool: &ConnectionPool,
    operation: F
) -> IDEResult<String>
where
    F: FnOnce(&Connection) -> Fut,
    Fut: Future<Output = IDEResult<String>>,
{
    let mut attempts = 0;
    loop {
        match pool.acquire().await {
            Ok(conn) => {
                match conn.health_check().await {
                    Ok(_) => {
                        let result = operation(&conn).await?;
                        pool.release(conn).await?;
                        return Ok(result);
                    }
                    Err(_) => {
                        pool.remove_broken_connection(conn).await;
                    }
                }
            }
            Err(_) if attempts < 3 => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### File Handle Leaks

**Issue**: File handles not being properly closed

**Solution**: Use RAII patterns and explicit cleanup:

```rust
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;

pub async fn safe_file_processing(path: &str) -> IDEResult<Vec<String>> {
    let file = File::open(path).await
        .map_err(|e| IDEError::Io(format!("Failed to open '{}': {}", path, e)))?;

    // File automatically closed when `_file` goes out of scope
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    let mut results = Vec::new();
    while let Some(line) = lines.next_line().await? {
        results.push(line);
    }

    Ok(results)
}
```

## Compilation and Build Issues

### Async Trait Conflicts

**Issue**: Compilation errors with async trait implementations

**Solution**: Ensure proper trait bounds and macro usage:

```rust
use async_trait::async_trait;

// ✅ Correct async trait definition
#[async_trait]
pub trait AsyncService: Send + Sync + 'static {
    async fn process(&self, data: String) -> IDEResult<String>;
    async fn health_check(&self) -> IDEResult<HealthStatus>;
}

// ✅ Correct implementation
pub struct MyService;

#[async_trait]
impl AsyncService for MyService {
    async fn process(&self, data: String) -> IDEResult<String> {
        // Implementation
        Ok(data.to_uppercase())
    }

    async fn health_check(&self) -> IDEResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }
}
```

### Dependency Resolution Issues

**Issue**: Cargo build failing due to version conflicts in workspace

**Solution**: Use workspace inheritance and check Cargo.lock:

```toml
# In workspace Cargo.toml
[workspace.dependencies]
tokio = "1.32"
async-trait = "0.1"
rust_ai_ide_errors = { path = "crates/rust-ai-ide-errors", version = "0.1" }

# In crate Cargo.toml
[dependencies]
tokio = { workspace = true }
async-trait = { workspace = true }
rust_ai_ide_errors = { workspace = true }
```

### Build Performance Issues

**Issue**: Slow compilation times after architectural changes

**Solution**: Optimize build configuration and use incremental compilation:

```toml
# In Cargo.toml
[profile.dev]
incremental = true
opt-level = 0
debug = true

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

## Testing Infrastructure Problems

### Async Test Timeouts

**Issue**: Async tests timing out unexpectedly

**Solution**: Use proper test attributes and timeouts:

```rust
use tokio::test;

#[tokio::test]
async fn test_async_operation() {
    let result = timeout(Duration::from_secs(5), async_operation())
        .await
        .expect("Test timed out");

    assert!(result.is_ok());
}
```

### Resource Cleanup in Tests

**Issue**: Test isolation problems due to shared state

**Solution**: Implement proper test setup and teardown:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref TEST_DATABASE: Arc<Mutex<TestDatabase>> = Arc::new(Mutex::new(TestDatabase::new()));
}

#[tokio::test]
async fn test_database_operation() {
    let db = TEST_DATABASE.lock().await;
    db.reset().await; // Clean state for each test

    // Test implementation
    let result = db.perform_operation().await;
    assert!(result.is_ok());
}
```

### Mock Service Configuration

**Issue**: Difficulty mocking async services for unit tests

**Solution**: Use dependency injection with mock implementations:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Cache: Send + Sync + 'static {
    async fn get(&self, key: &str) -> IDEResult<Option<String>>;
    async fn set(&self, key: String, value: String) -> IDEResult<()>;
}

// Test implementation
pub struct MockCache {
    data: Arc<Mutex<HashMap<String, String>>>,
}

#[async_trait]
impl Cache for MockCache {
    async fn get(&self, key: &str) -> IDEResult<Option<String>> {
        let data = self.data.lock().await;
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: String, value: String) -> IDEResult<()> {
        let mut data = self.data.lock().await;
        data.insert(key, value);
        Ok(())
    }
}
```

## Monitoring and Diagnostics

### Runtime Performance Monitoring

**Issue**: Difficulty identifying performance bottlenecks in production

**Solution**: Implement comprehensive monitoring:

```rust
use rust_ai_ide_common::{PerformanceMetrics, time_operation};
use tokio::time::Duration;

pub async fn monitored_service_operation() -> IDEResult<String> {
    let (result, timing) = time_operation!("service_operation", async {
        // Actual work
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok("completed".to_string())
    })?;

    if timing > Duration::from_millis(500) {
        warn!("Slow operation detected: {}ms", timing.as_millis());
        // Could send metrics to monitoring system
    }

    Ok(result)
}
```

### Health Check Implementation

**Issue**: Lack of visibility into system health

**Solution**: Implement comprehensive health checks:

```rust
#[async_trait]
pub trait HealthCheck: Send + Sync + 'static {
    async fn check_health(&self) -> HealthStatus;
    fn name(&self) -> &'static str;
}

pub async fn system_health_check(checks: &[Box<dyn HealthCheck>]) -> SystemHealth {
    let mut all_healthy = true;
    let mut failed_checks = Vec::new();

    for check in checks {
        match check.check_health().await {
            HealthStatus::Healthy => {}
            HealthStatus::Degraded => { all_healthy = false; }
            HealthStatus::Unhealthy => {
                all_healthy = false;
                failed_checks.push(check.name());
            }
        }
    }

    if all_healthy {
        SystemHealth::Healthy
    } else if failed_checks.is_empty() {
        SystemHealth::Degraded
    } else {
        SystemHealth::Unhealthy { failed_checks }
    }
}
```

## Common Error Patterns and Solutions

| Error Pattern | Likely Cause | Solution |
|---------------|--------------|----------|
| `IDEError::Concurrency` | Lock ordering issues | Review lock acquisition order |
| `IDEError::Timeout` | Slow operations | Add timeouts and optimize performance |
| `IDEError::Security` | Input validation failure | Review validation rules |
| `IDEError::Resource` | Resource exhaustion | Implement resource pools and limits |
| Memory allocation errors | Memory leaks | Add monitoring and cleanup |
| Compilation errors with async | Trait bound issues | Add proper Send + Sync bounds |

## Getting Help

When encountering issues not covered in this guide:

1. **Check the codebase**: Look for similar patterns in existing code
2. **Review documentation**: Consult the Shared Architecture Guide and API docs
3. **Run tests**: Execute the relevant test suite to isolate issues
4. **Collect diagnostics**: Gather performance metrics and error details
5. **Search existing issues**: Check if the problem has been resolved before

## Prevention Best Practices

### Code Review Checklist

- [ ] All public functions use IDEResult<T>
- [ ] Async functions have proper Send + Sync bounds
- [ ] Input validation occurs before processing
- [ ] Resources are properly cleaned up
- [ ] Error messages provide useful context
- [ ] Performance monitoring is in place
- [ ] Tests cover error scenarios

### Architectural Review Points

- [ ] Async boundaries are clearly defined
- [ ] Resource ownership is clearly documented
- [ ] Security validation is comprehensive
- [ ] Performance metrics are collected
- [ ] Error propagation follows consistent patterns
- [ ] Memory usage is bounded and monitored

---

This troubleshooting guide is continuously updated. For the latest information, refer to the project's documentation repository.