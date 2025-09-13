# ADR-006: Async Concurrency Patterns and State Management

## Status

- **Date**: 2025-01-13
- **Status**: Accepted

## Context

The Rust AI IDE project requires:

1. **Async State Management**: Safe concurrent access to shared application state
2. **Performance Optimization**: Efficient handling of concurrent operations
3. **Race Condition Prevention**: Protection against concurrent data access issues
4. **Resource Management**: Proper cleanup and lifecycle management of async tasks
5. **Scalability**: Support for multiple concurrent users and operations
6. **Error Propagation**: Reliable error handling in async contexts

### Forces Considered

- **Safety vs. Performance**: Memory safety guarantees vs. execution speed
- **Complexity vs. Maintainability**: Advanced patterns vs. code readability
- **Resource Usage vs. Responsiveness**: Memory overhead vs. user experience
- **Flexibility vs. Consistency**: Custom solutions vs. standardized patterns
- **Debugging vs. Production**: Development visibility vs. runtime efficiency

## Decision

**Adopt Tokio-based async concurrency patterns** with the following architectural choices:

1. **Arc<Mutex<T>> for Shared State**: Thread-safe access to application state
2. **Double-Locking Pattern**: Lazy initialization of async services
3. **Tokio Primitives**: `mpsc`, `oneshot`, and `RwLock` for inter-task communication
4. **EventBus Pattern**: Standardized pub-sub communication between components
5. **Spawn Background Task Macro**: Consistent background task lifecycle management
6. **Structured Concurrency**: Clear task hierarchies and cleanup patterns

### Async Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Application State                         │
│                    Arc<Mutex<AppState>>                      │
├─────────────────────┬─────────────────────┬─────────────────┤
│ Service Acquisition │ State Updates       │ Background Tasks │
├─────────────────────┼─────────────────────┼─────────────────┤
│ • Double-locking    │ • RwLock patterns   │ • Spawn macros    │
│ • Lazy init         │ • Atomic operations │ • Cleanup         │
│ • Error handling    │ • Event publishing  │ • Resource mgmt   │
└─────────────────────┴─────────────────────┴─────────────────┘
                                 │
                    ┌────────────┴────────────┐
                    ▼                         ▼
        ┌─────────────────────┐   ┌─────────────────────┐
        │  Inter-Task         │   │  Event Bus         │
        │  Communication      │   │  System            │
        │                     │   │                     │
        │ • MPSC Channels     │   │ • Pub-Sub Pattern   │
        │ • Oneshot Responses │   │ • Async Broadcasting │
        │ • Stream Processing │   │ • Event Filtering   │
        └─────────────────────┘   └─────────────────────┘
```

## Consequences

### Positive

- **Memory Safety**: Compile-time guarantees prevent data races and memory corruption
- **Performance**: Efficient async execution with minimal overhead
- **Scalability**: Support for high-concurrency workloads and multiple users
- **Maintainability**: Clear patterns reduce complexity and improve code consistency
- **Debugging**: Structured concurrency enables better error tracking and debugging
- **Resource Efficiency**: Automatic cleanup and proper resource lifecycle management

### Negative

- **Learning Curve**: Complex async patterns require developer expertise
- **Debugging Difficulty**: Async stack traces can be harder to follow
- **Performance Overhead**: Safety guarantees add runtime overhead
- **Code Complexity**: Async code is inherently more complex than synchronous
- **Testing Challenges**: Async testing requires specialized approaches

### Risks

- **Deadlocks**: Improper lock ordering can cause deadlocks
- **Race Conditions**: Subtle concurrency bugs difficult to reproduce
- **Resource Leaks**: Improper cleanup of async tasks and resources
- **Performance Degradation**: Lock contention under high concurrency
- **Complexity Management**: Growing complexity as codebase scales

#### Mitigation Strategies

- **Code Reviews**: Mandatory review of async concurrency patterns
- **Testing**: Comprehensive async testing with race condition detection
- **Monitoring**: Runtime monitoring of locks and task lifecycles
- **Documentation**: Clear guidelines and examples for async patterns
- **Tools**: Utilize Rust's async tooling (tokio-console, tracing)

## Alternatives Considered

### Alternative 1: Synchronous Architecture
- **Reason Not Chosen**: Would violate performance requirements and user experience expectations
- **Impact**: Poor responsiveness, blocked UI, inability to handle concurrent operations

### Alternative 2: Actor Model Framework
- **Reason Not Chosen**: Would introduce heavy dependencies and complexity not justified by requirements
- **Impact**: Increased resource usage, steeper learning curve, maintenance overhead

### Alternative 3: Custom Threading Solution
- **Reason Not Chosen**: Would compromise safety guarantees and increase bug potential
- **Impact**: Race conditions, deadlocks, memory safety issues, debugging difficulties

### Alternative 4: Reactive Programming Framework
- **Reason Not Chosen**: Would add unnecessary abstraction layers and performance overhead
- **Impact**: Performance degradation, increased complexity, debugging challenges

## Implementation Notes

### Double-Locking Pattern for Service Initialization

```rust
// crates/rust-ai-ide-infra/src/service_manager.rs
pub struct ServiceManager<T> {
    service: Arc<Mutex<Option<Arc<T>>>>,
    service_initializer: Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Arc<T>, Error>> + Send>> + Send + Sync>,
}

impl<T> ServiceManager<T> {
    pub async fn get_service(&self) -> Result<Arc<T>, Error> {
        // First lock check - fast path
        {
            let service_guard = self.service.lock().await;
            if let Some(service) = service_guard.as_ref() {
                return Ok(service.clone());
            }
        }

        // Service not initialized - second lock for initialization
        let mut service_guard = self.service.lock().await;

        // Double-check pattern
        if let Some(service) = service_guard.as_ref() {
            return Ok(service.clone());
        }

        // Initialize service
        let service = (self.service_initializer)().await?;
        *service_guard = Some(service.clone());

        Ok(service)
    }
}
```

### EventBus Implementation

```rust
// crates/rust-ai-ide-infra/src/event_bus.rs
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventType, Vec<Arc<dyn EventSubscriber>>>>>,
    sender: mpsc::UnboundedSender<Event>,
    receiver_task: JoinHandle<()>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let subscribers = Arc::new(RwLock::new(HashMap::new()));

        let subscribers_clone = subscribers.clone();
        let receiver_task = tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                Self::process_event(&subscribers_clone, event).await;
            }
        });

        Self {
            subscribers,
            sender,
            receiver_task,
        }
    }

    pub async fn publish(&self, event: Event) -> Result<(), EventError> {
        self.sender.send(event).map_err(|_| EventError::ChannelClosed)?;
        Ok(())
    }

    async fn process_event(subscribers: &Arc<RwLock<HashMap<EventType, Vec<Arc<dyn EventSubscriber>>>>>, event: Event) {
        let event_type = event.event_type();
        let subs = subscribers.read().await;

        if let Some(subscribers_list) = subs.get(&event_type) {
            for subscriber in subscribers_list {
                let subscriber = subscriber.clone();
                let event = event.clone();

                // Spawn task to avoid blocking event processing
                tokio::spawn(async move {
                    if let Err(e) = subscriber.handle_event(event).await {
                        log::error!("Event handler error: {}", e);
                    }
                });
            }
        }
    }
}
```

### Background Task Management

```rust
// crates/rust-ai-ide-infra/src/command_templates.rs
#[macro_export]
macro_rules! spawn_background_task {
    ($task_name:expr, $task:expr) => {{
        use std::sync::atomic::{AtomicBool, Ordering};
        use tokio::task::JoinHandle;

        static TASK_RUNNING: AtomicBool = AtomicBool::new(false);

        if TASK_RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            let task_name = $task_name;
            let handle: JoinHandle<()> = tokio::spawn(async move {
                log::info!("Starting background task: {}", task_name);

                match $task.await {
                    Ok(_) => log::info!("Background task completed: {}", task_name),
                    Err(e) => log::error!("Background task failed {}: {}", task_name, e),
                }

                TASK_RUNNING.store(false, Ordering::SeqCst);
            });

            // Store handle for cleanup if needed
            Some(handle)
        } else {
            log::warn!("Background task {} already running", $task_name);
            None
        }
    }};
}
```

### Inter-Task Communication Patterns

```rust
// crates/rust-ai-ide-ai/src/model_manager.rs
pub struct ModelManager {
    command_sender: mpsc::Sender<ModelCommand>,
    result_receiver: Arc<Mutex<Option<mpsc::Receiver<ModelResult>>>>,
    worker_task: JoinHandle<()>,
}

impl ModelManager {
    pub async fn execute_inference(&self, request: InferenceRequest) -> Result<InferenceResult, Error> {
        // Create oneshot channel for response
        let (response_sender, response_receiver) = oneshot::channel();

        // Send command to worker
        self.command_sender.send(ModelCommand::Inference {
            request,
            response_sender,
        }).await?;

        // Wait for response with timeout
        match tokio::time::timeout(Duration::from_secs(30), response_receiver).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(_)) => Err(Error::WorkerChannelClosed),
            Err(_) => Err(Error::InferenceTimeout),
        }
    }
}
```

### State Update Patterns

```rust
// crates/rust-ai-ide-core/src/state_management.rs
pub struct AppState {
    pub ai_service: Arc<Mutex<Option<Arc<AIService>>>>,
    pub lsp_service: Arc<Mutex<Option<Arc<LSPService>>>>,
    pub project_state: Arc<RwLock<ProjectState>>,
    pub user_preferences: Arc<RwLock<UserPreferences>>,
}

impl AppState {
    pub async fn update_project_state<F, R>(&self, updater: F) -> Result<R, Error>
    where
        F: FnOnce(&mut ProjectState) -> Result<R, Error>,
    {
        let mut project_state = self.project_state.write().await;
        let result = updater(&mut project_state)?;

        // Publish state change event
        self.event_bus.publish(Event::ProjectStateChanged).await?;

        Ok(result)
    }

    pub async fn get_user_preference<T: Clone>(&self, key: &str) -> Option<T> {
        let preferences = self.user_preferences.read().await;
        preferences.get(key)
    }

    pub async fn set_user_preference<T: Serialize>(&self, key: String, value: T) -> Result<(), Error> {
        let mut preferences = self.user_preferences.write().await;
        preferences.set(key, value)?;

        // Persist to disk asynchronously
        let preferences_clone = preferences.clone();
        tokio::spawn(async move {
            if let Err(e) = preferences_clone.persist().await {
                log::error!("Failed to persist user preferences: {}", e);
            }
        });

        Ok(())
    }
}
```

### Error Handling in Async Contexts

```rust
// crates/rust-ai-ide-core/src/error_handling.rs
pub async fn execute_with_retry<F, Fut, T>(
    operation: F,
    max_retries: usize,
    base_delay: Duration,
) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
{
    let mut delay = base_delay;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_retries {
                    return Err(e);
                }

                log::warn!("Operation failed (attempt {}): {}", attempt + 1, e);
                tokio::time::sleep(delay).await;

                // Exponential backoff
                delay = delay.saturating_mul(2);
            }
        }
    }

    unreachable!()
}
```

### Configuration

```toml
# .rust-ai-ide.toml - Async Configuration
[async]
# Concurrency settings
max_concurrent_ai_requests = 4
max_concurrent_lsp_requests = 8

# Timeout settings
ai_request_timeout_seconds = 30
lsp_request_timeout_seconds = 10

# Resource limits
max_background_tasks = 10
background_task_cleanup_interval_seconds = 300

# Event bus settings
event_bus_buffer_size = 1000
event_processing_concurrency = 4
```

## Related ADRs

- [ADR-001: Multi-Crate Workspace Architecture](adr-001-multi-crate-workspace-architecture.md)
- [ADR-002: Nightly Rust Usage](adr-002-nightly-rust-usage.md)
- [ADR-003: Tauri Integration Patterns](adr-003-tauri-integration-patterns.md)
- [ADR-004: AI/ML Service Architecture](adr-004-ai-ml-service-architecture.md)
- [ADR-005: Security Framework](adr-005-security-framework.md)