# 🏗️ Implementation Patterns & Standards

**Date:** September 10, 2025  
**Version:** 3.2.0 | **Status:** Active Standards  

## 📋 Standards Overview

This document establishes the unified implementation patterns and coding standards that have been validated across the 67-crate workspace. These patterns ensure consistency, maintainability, and performance across all development teams.

## 🎯 Core Implementation Principles

### 1. **Shared Crates First**
Always prioritize shared crates before implementing custom functionality.

**Pattern Structure:**
```rust
// ✅ CORRECT: Start with shared imports
use rust_ai_ide_common::{
    // Core types and errors
    ProgrammingLanguage, Position, Range, Location,
    IdeError, IdeResult, Caching,

    // Performance monitoring
    PerformanceMetrics, time_operation,

    // File operations
    fs_utils::{read_file_to_string, write_string_to_file},
};

// Then add crate-specific imports
use rust_ai_ide_shared_codegen::CodeGenerator;
use rust_ai_ide_shared_services::{WorkspaceManager, LspClient};

// Custom implementation follows using shared patterns
```

### 2. **Error Handling Standardization**
Use `IdeError` consistently across all crates.

**Correct Pattern:**
```rust
#[derive(thiserror::Error, Debug)]
pub enum IDEError {
    #[error("I/O error: {source}")]
    Io { #[from] source: std::io::Error },

    #[error("Workspace operation error: {message}")]
    Workspace { message: String },

    #[error("AI service error: {source}")]
    AI { #[from] source: AIServiceError },
}

pub type IDEResult<T> = Result<T, IDEError>;
```

### 3. **Async Thread Safety Pattern**
All shared resources must be wrapped in `Arc<Mutex<T>>`.

**Pattern:**
```rust
#[derive(Clone)]
pub struct SharedResource {
    inner: Arc<RwLock<InnerResource>>,
}

#[derive(Default)]
struct InnerResource {
    data: HashMap<String, Vec<String>>,
    connections: Vec<Connection>,
}

impl SharedResource {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Default::default())),
        }
    }

    pub async fn safe_operation(&self, key: &str) -> IDEResult<String> {
        let mut inner = self.inner.write().await;
        inner.perform_operation(key).await
    }
}
```

## 🛠️ Command Implementation Patterns

### Tauri Command Template
```rust:src-tauri/src/command_templates.rs
tauri_command_template! {
    #[tauri::command]
    pub async fn implement_feature(
        state: tauri::State<'_, AppState>,
        input: FeatureInput
    ) -> CommandResult<FeatureOutput> {
        acquire_service_and_execute!(state, |services| {
            validate_commands! {
                input.feature_name = InputSanitization::sanitize_string(input.feature_name)?;
                input.feature_description = InputSanitization::sanitize_string(input.feature_description)?;
            }

            services.feature_service.create_feature(input).await
        })
    }
}
```

### Service Layer Implementation
```rust:src-tauri/src/handlers/project.rs
#[async_trait]
pub trait ProjectService: Send + Sync {
    async fn analyze_project(
        &self,
        project_path: PathBuf,
        context: ProjectContext,
    ) -> IDEResult<ProjectAnalysis> {

        // Performance monitoring built-in
        let result = time_operation!("project_analysis", async {
            validate_secure_path(&project_path)?;

            // Implement specific project analysis logic
            let structure = self.analyze_structure(&project_path).await?;
            let dependencies = self.analyze_dependencies(&project_path).await?;

            Ok(ProjectAnalysis { structure, dependencies })
        }).await?;

        Ok(result)
    }
}
```

## 🔧 Core Implementation Patterns

### 1. **Performance-Aware Implementation**
```rust:src-tauri/src/utils/cache.rs
#[async_trait]
pub trait Cachable<T>: Send + Sync {
    async fn get_or_compute<F>(
        &self,
        key: &str,
        compute_fn: F
    ) -> IDEResult<T>
    where
        F: Fn() -> pin_project_lite::Pin<Box<dyn Future<Output = IDEResult<T>> + Send>> + Send;

    async fn invalidate(&self, key: &str) -> IDEResult<()>;
}

#[derive(Clone)]
pub struct IntelligentCache {
    cache: Arc<RwLock<HashMap<String, CachedValue >>>,
    ttl_seconds: u64,
}

#[async_trait]
impl Cachable<Vec<ProjectAnalysis>> for IntelligentCache {
    async fn get_or_compute<F>(&self, key: &str, compute_fn: F) -> IDEResult<Vec<ProjectAnalysis>>
    where
        F: Fn() -> pin_project_lite::Pin<Box<dyn Future<Output = IDEResult<Vec<ProjectAnalysis>>> + Send>> + Send,
    {
        // Implement LRU cache with TTL policy
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();

        let maybe_cached = {
            let cache = self.cache.read().await;
            cache.get(key).cloned()
        };

        if let Some(cached) = maybe_cached {
            if now < cached.expiry {
                return Ok(cached.data);
            }
        }

        // Compute new value
        let fresh_data = compute_fn().await?;
        let expiry = now + self.ttl_seconds;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), CachedValue {
                data: fresh_data.clone(),
                expiry,
                last_accessed: now,
                access_count: 1,
            });
        }

        Ok(fresh_data)
    }
}
```

### 2. **Background Task Implementation**
```rust:src-tauri/src/lifecycle/lifecycle.rs
spawn_background_task! {
    project_indexing_task: async {
        let progress = ProgressReporter::new("project_indexing");
        let indexer = ProjectIndexer::new();

        loop {
            let projects = discover_projects_requiring_indexing().await?;

            for project in projects {
                progress.report_step(&format!("Indexing {}", project.name), 10.0)?;
                indexer.index_project(&project).await?;
            }

            indexer.optimize_index().await?;
            tokio::time::sleep(Duration::from_secs(300)).await;
        }
    }
}
```

### 3. **Event-Driven Architecture**
```rust:src-tauri/src/infra.rs
#[async_trait]
pub trait EventHandler<E>: Send + Sync {
    async fn handle_event(&mut self, event: &E, context: &EventContext) -> EventResult;
}

pub struct EventBus {
    channels: Arc<DashMap<EventType, Vec<ChannelSender<EventData>>>>,
    registry: Arc<DashMap<String, Box<dyn EventHandler<Event>>>>,
}

impl EventBus {
    pub async fn post_event(&self, event_type: EventType, data: EventData) -> IDEResult<()> {
        if let Some(receivers) = self.channels.get(&event_type) {
            for receiver in receivers.iter() {
                receiver.send(data.clone()).await
                    .map_err(|_| IDEError::EventPostingFailed {
                        event_type: format!("{:?}", event_type)
                    })?;
            }
        }

        Ok(())
    }

    pub async fn subscribe<H>(
        &self,
        event_types: &[EventType],
        handler: H,
        priority: HandlerPriority
    ) -> SubscriberId
    where
        H: EventHandler<Event> + Send + Sync + 'static,
    {
        let subscriber_id = Uuid::new_v4().to_string();

        for event_type in event_types {
            self.channels
                .entry(*event_type)
                .or_insert_with(Vec::new)
                .push(ChannelSender {
                    sender: tokio::sync::mpsc::unbounded_channel(),
                    priority,
                    id: subscriber_id.clone(),
                });
        }

        self.registry.insert(subscriber_id.clone(), Box::new(handler));
        subscriber_id
    }
}
```

## 🔒 Security Implementation Standards

### Input Validation Pattern
```rust:src-tauri/src/validation.rs
#[derive(Debug)]
pub struct TauriInputSanitizer;

impl TauriInputSanitizer {
    pub fn sanitize_string(input: String) -> IDEResult<String> {
        // Basic sanitization rules
        let cleaned = input.trim();

        if cleaned.len() > MAX_STRING_LENGTH {
            return Err(IDEError::InputValidationError {
                field: "string",
                reason: format!("length {} exceeds maximum {}", cleaned.len(), MAX_STRING_LENGTH)
            });
        }

        // Prevent injection attacks
        if Self::detect_injection_patterns(cleaned) {
            return Err(IDEError::SecurityError {
                context: "input_validation",
                details: "injection pattern detected"
            });
        }

        Ok(cleaned.to_string())
    }

    pub fn validate_path(path: &str) -> IDEResult<PathBuf> {
        validate_secure_path(path).await
            .map_err(|_| IDEError::PathValidationError {
                path: path.to_string(),
                reason: "insecure path detected"
            })
    }
}
```

### Audit Logging Pattern
```rust:src-tauri/src/security/audit.rs
#[derive(Clone)]
pub struct AuditLogger {
    writer: Arc<RwLock<AuditWriter>>,
    formatter: AuditFormatter,
}

impl AuditLogger {
    pub async fn log_security_event(
        &self,
        event: SecurityEvent,
        user_context: &UserContext
    ) -> IDEResult<()> {
        let audit_entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            event_type: event.event_type.clone(),
            user_id: user_context.user_id.clone(),
            session_id: user_context.session_id.clone(),
            ip_address: user_context.ip_address,
            user_agent: user_context.user_agent.clone(),
            resource: event.resource.clone(),
            action: event.action.clone(),
            result: event.result,
            metadata: event.metadata.clone(),
        };

        {
            let mut writer = self.writer.write().await;
            writer.write_entry(&audit_entry).await?;
        }

        // Alert on high-severity events
        if event.severity >= SecuritySeverity::High {
            self.generate_security_alert(&audit_entry).await?;
        }

        Ok(())
    }
}
```

## 📊 Performance Monitoring Standards

### Metrics Collection Pattern
```rust:src-tauri/src/metrics/performance.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    operation_name: String,
    duration: Duration,
    start_time: SystemTime,
    memory_usage: u64,
    cpu_usage: f64,
    success: bool,
    metadata: HashMap<String, String>,
}

#[async_trait]
pub trait PerformanceMonitor {
    async fn time_operation<F, Fut, T>(
        &self,
        operation_name: &str,
        operation: F
    ) -> IDEResult<(T, Duration)>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = IDEResult<T>>;

    async fn record_metric(&self, metrics: PerformanceMetrics) -> IDEResult<()>;

    async fn get_metrics(&self, filter: MetricsFilter) -> IDEResult<Vec<PerformanceMetrics>>;
}
```

## 🔄 State Management Patterns

### Double-Locking Pattern
```rust:src-tauri/src/state/mod.rs
#[derive(Clone)]
pub struct ServicePool<T> {
    service: Arc<RwLock<Option<T>>>,
    init_fn: fn() -> T,
}

impl<T> ServicePool<T> {
    pub fn new(init_fn: fn() -> T) -> Self {
        Self {
            service: Arc::new(RwLock::new(None)),
            init_fn,
        }
    }

    pub async fn get(&self) -> IDEResult<Arc<RwLock<T>>> {
        // First lock - check if service exists
        let read_guard = self.service.read().await;
        if read_guard.is_some() {
            drop(read_guard);
            // Use Arc clone to avoid holding read lock during write
            return Ok(Arc::clone(&self.service));
        }
        drop(read_guard);

        // Second lock - initialize if not initialized
        let mut write_guard = self.service.write().await;
        if write_guard.is_none() {
            *write_guard = Some((self.init_fn)());
        }

        Ok(Arc::clone(&self.service))
    }
}
```

## ⚡ Async Patterns

### Command Execution Pattern
```rust:src-tauri/src/command_patterns.rs
pub struct CommandExecutor {
    max_concurrency: usize,
    retry_policy: RetryPolicy,
    rate_limiter: Arc<RateLimiter>,
}

impl CommandExecutor {
    pub async fn execute_with_retry<T, F>(
        &self,
        operation: F,
        description: &str
    ) -> IDEResult<T>
    where
        F: Fn() -> pin_project_lite::Pin<Box<dyn Future<Output = IDEResult<T>> + Send>>,
        T: Send,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.retry_policy.max_attempts {
            // Rate limiting
            self.rate_limiter.wait_until_allowed().await?;

            let permit = self.get_concurrency_permit().await?;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if !self.should_retry_error(&last_error.as_ref().unwrap()) {
                        break;
                    }

                    if attempts < self.retry_policy.max_attempts {
                        let delay = self.retry_policy.backoff_strategy.calculate_delay(attempts);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| IDEError::MaxRetriesExceeded {
            operation: description.to_string(),
        }))
    }
}
```

## 🧪 Testing Pattern Standards

### Unit Test Pattern
```rust:src-tauri/src/commands/project.rs/test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use rust_ai_ide_common::test_utils::*;
    use tokio::test;

    #[test]
    async fn test_project_analysis() {
        // Setup test environment
        let mock_workspace = TestWorkspaceBuilder::new()
            .add_file("Cargo.toml", SAMPLE_CARGO_TOML)
            .add_file("src/lib.rs", SAMPLE_LIB_RS)
            .build()
            .await;

        let service = ProjectService::new(mock_workspace.path());
        let context = ProjectContext {
            project_name: "test_project".to_string(),
            target_languages: vec![ProgrammingLanguage::Rust],
            analysis_config: Default::default(),
        };

        // Execute test
        let result = service.analyze_project(mock_workspace.path(), context).await;

        // Verify results
        assert!(result.is_ok());
        let analysis = result.unwrap();

        assert!(analysis.dependencies.contains(&"rustc".to_string()));
        assert_eq!(analysis.project_name, "test_project");
        assert!(analysis.file_count > 0);
    }

    #[test]
    async fn test_project_analysis_error_handling() {
        let service = ProjectService::new();
        let context = ProjectContext::default();

        // Test non-existent project
        let result = service
            .analyze_project("non_existent_path", context)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), IDEError::WorkspaceNotFound { .. }));
    }
}
```

## 📋 Implementation Checklist

### ✅ Code Submission Requirements
- [ ] Used shared crates (`rust-ai-ide-common`, `shared-codegen`, `shared-services`)
- [ ] Implemented with `IdeError` consistent error handling
- [ ] Used `time_operation!` for performance monitoring
- [ ] Applied `Arc<Mutex<T>>` for shared state access
- [ ] Followed async safety patterns
- [ ] Added comprehensive unit tests
- [ ] Used `tauri_command_template!` for external APIs
- [ ] Applied input sanitization via `TauriInputSanitizer`
- [ ] Included audit logging for sensitive operations

### 🔸 Code Quality Standards
- [ ] No duplicate patterns (use shared crates)
- [ ] Functions follow single responsibility principle
- [ ] Error handling uses early returns without `?`
- [ ] All public APIs are thread-safe with `Arc<Mutex<T>>`
- [ ] Performance monitoring included for operations >100ms
- [ ] Memory usage tracked for large data structures
- [ ] Logging implemented appropriately

### 🐛 Common Pattern Violations to Avoid
- ❌ Custom error types (use `IdeError`)
- ❌ Direct `tokio::spawn()` (use `spawn_background_task!`)
- ❌ Unwrapped async access (use lock patterns)
- ❌ Missing performance monitoring
- ❌ Not using shared type definitions
- ❌ Direct file system access without validation
- ❌ Missing audit logging for security operations

---

**Pattern Status:** Standardized across 67 crates  
**Adherence:** Required for all new development  
**Migration:** Legacy code should be updated to follow these patterns