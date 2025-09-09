# Rust AI IDE Supervisor

## Core High Availability Framework for Rust AI IDE

The Supervisor crate provides comprehensive high availability (HA) capabilities for the Rust AI IDE project, ensuring system reliability through process monitoring, state persistence, and IPC recovery mechanisms.

## Features

### 🏥 Service Supervisor
- **Process Monitoring**: Continuous health monitoring of critical services (AI LSP, model manager, webhooks)
- **Automated Restarts**: Configurable restart policies with exponential backoff
- **Graceful Shutdowns**: Clean service termination with configurable timeouts
- **Critical Service Support**: System can safely shut down if critical services fail

### 💾 State Persistence Layer
- **SQLite Backend**: Robust, transactional state storage with ACID properties
- **Checkpoint/Savepoint System**: Crash recovery with validated system snapshots
- **Migration Support**: Forward-only schema evolution for data integrity
- **File-based Backup**: Additional persistence layer with checksum verification

### 🔄 IPC Recovery System
- **Channel Health Monitoring**: Automatic detection of communication failures
- **Message Buffering**: Queue failed messages for retry during recovery
- **Auto-reconnection**: Configurable reconnection strategies with backoff
- **State Synchronization**: Ensure consistent state post-recovery

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 Supervisor State Management                  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Service Supervisor                         │  │
│  │  • Process monitoring & health checks                   │  │
│  │  • Restart policies & exponential backoff              │  │
│  │  • Graceful shutdown handling                          │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │            State Persistence Layer                     │  │
│  │  • SQLite database storage                             │  │
│  │  • Checkpoints & savepoints                            │  │
│  │  • Migration system                                    │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │            IPC Recovery System                         │  │
│  │  • Channel health monitoring                           │  │
│  │  • Message buffering & retry                           │  │
│  │  • Auto-reconnection logic                             │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
           │                       │
           └─────────────┬─────────┘
                         │
            ┌────────────────────┐
            │    Shared State    │
            │  Arc<Mutex<...>>   │
            └────────────────────┘
                         │
       ┌─────────────────┴─────────────────┐
       │                                   │
┌──────▼─────┐                     ┌──────▼─────┐
│   Tauri     │                     │  EventBus  │
│  Commands  │                     │Integration │
└────────────┘                     └────────────┘
```

## Usage

### Basic Service Supervision

```rust,no_run
use rust_ai_ide_supervisor::{Supervisor, ServiceConfig, RestartPolicy};

// Create supervisor instance
let supervisor = Supervisor::new()?;

// Configure critical service
let lsp_config = ServiceConfig {
    id: "ai_lsp".to_string(),
    name: "AI Language Server".to_string(),
    command: "/path/to/lsp".to_string(),
    args: vec!["--config".to_string(), "lsp-config.json".to_string()],
    working_dir: None,
    environment: std::env::vars().collect(),
    health_check_timeout: std::time::Duration::from_secs(30),
    restart_policy: RestartPolicy::ExponentialBackoff {
        base_delay: std::time::Duration::from_secs(1),
        max_delay: std::time::Duration::from_secs(60),
        max_attempts: 5,
    },
    shutdown_timeout: std::time::Duration::from_secs(10),
    critical: true, // System shutdown if this fails
};

// Register and start monitoring
supervisor.register_service(lsp_config).await?;
supervisor.start_monitoring().await?;
```

### State Persistence and Checkpoints

```rust,no_run
use rust_ai_ide_supervisor::StatePersistence;

// Initialize persistence layer
let persistence = StatePersistence::new(
    "/data/supervisor.db",
    "/data/checkpoints"
).await?;

// Create state checkpoint (would be called during shutdown or regularly)
let checkpoint_id = persistence.create_checkpoint(&services, &pending_operations).await?;

// Load checkpoint after crash recovery
let snapshot = persistence.load_latest_checkpoint().await?;
println!("Recovered from checkpoint: {}", snapshot.id);
```

### IPC Monitoring with Recovery

```rust,no_run
use rust_ai_ide_supervisor::{IpcMonitor, IpcMessage, RecoveryConfig};

// Create monitor with recovery configuration
let config = RecoveryConfig::default();
let monitor = IpcMonitor::with_config(config);

// Register communication channel
monitor.register_channel("lsp_channel".to_string()).await?;

// Send message with automatic retry on failure
let message = IpcMessage {
    id: uuid::Uuid::new_v4(),
    message_type: "completion_request".to_string(),
    payload: serde_json::json!({"code": "fn main() {}"}),
    timestamp: chrono::Utc::now(),
    retry_count: 0,
};

monitor.send_message("lsp_channel", message).await.expect("Failed to send");
```

## Configuration

### Supervisor Configuration
```rust
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub max_concurrent_services: usize,           // Default: 10
    pub health_check_interval: Duration,          // Default: 5s
    pub global_shutdown_timeout: Duration,        // Default: 30s
    pub database_path: String,                    // Default: "supervisor.db"
    pub checkpoint_dir: String,                   // Default: "checkpoints"
    pub enable_detailed_logging: bool,            // Default: false
    pub enable_auto_backup: bool,                 // Default: true
    pub backup_interval: Duration,                // Default: 1 hour
}
```

### Restart Policies
- **Never**: No automatic restart
- **Always**: Immediate restart on failure
- **ExponentialBackoff**: `delay = base_delay * 2^attempt` with max delay cap
- **FixedDelay**: Restart after fixed delay interval

## Integration Points

### Tauri Commands
The supervisor integrates with Tauri for frontend control:
- `init_supervisor` - Initialize the supervisor system
- `register_service` - Register a new service for monitoring
- `start_supervisor_monitoring` - Begin monitoring all services
- `get_supervisor_health` - Get current system health status
- `create_checkpoint` - Create system state checkpoint
- `load_checkpoint` - Load latest checkpoint

### EventBus Integration
Supervisor events are published to the existing EventBus:
- `SupervisorEvent::ServiceStateChanged`
- `SupervisorEvent::HealthCheckCompleted`
- `SupervisorEvent::ServiceRestarted`
- `SupervisorEvent::IpcChannelStateChanged`

## Security Features

### Path Validation
- Database and checkpoint paths are validated through `validate_secure_path`
- Input sanitization using `TauriInputSanitizer` for all user inputs
- Command injection protection for service arguments

### Audit Logging
All sensitive operations are logged through the security audit system:
- Service state changes
- Checkpoint operations
- Recovery actions

## Error Handling

The supervisor uses comprehensive error types covering:
- Process monitoring errors
- Database operations failures
- IPC communication issues
- Security validation errors
- Migration failures

Error handling follows the project's patterns:
- Errors aggregated at function boundaries
- IDEError enum extension support
- Context-aware error messages

## Testing

### Unit Tests
- Service health check logic
- Restart policy calculations
- Error aggregation mechanisms
- Security validation functions

### Integration Tests
- End-to-end service lifecycle management
- Database persistence across restarts
- IPC recovery after simulated failures
- Performance under concurrent load

## Performance Considerations

- **Async-First Design**: All operations are async for optimal concurrency
- **Connection Pooling**: SQLite connections efficiently managed
- **Message Buffering**: Configurable buffer sizes prevent memory issues
- **Exponential Backoff**: Intelligent retry logic prevents resource exhaustion
- **Health Check Optimization**: Overlapping health checks for monitoring efficiency

## Development

### Building
```bash
cargo build -p rust-ai-ide-supervisor
```

### Testing
```bash
cargo test -p rust-ai-ide-supervisor
```

### Linting
```bash
cargo +nightly clippy -p rust-ai-ide-supervisor
```

## Dependencies

Core dependencies include:
- `tokio` - Async runtime and utilities
- `rusqlite` - SQLite database interface
- `serde` - Serialization for state persistence
- `chrono` - Date/time handling
- `uuid` - Unique identifier generation
- Project internal crates for security, error handling, and common utilities

## Implementation Notes

- **Nightly Rust Support**: Uses Rust nightly 2025-09-03 with unstable features
- **Workspace Integration**: Fully integrated with 30+ crate workspace
- **Tauri Compatibility**: Maintains compatibility with existing Tauri integration patterns
- **Memory Efficient**: Designed for large workspaces (>1M LOC)

## Contributing

Follow the project's contribution guidelines:
1. Ensure all tests pass
2. Update documentation for public APIs
3. Follow existing code patterns for error handling and async operations
4. Add comprehensive test coverage for new features

## License

Licensed under MIT OR Apache-2.0 in compliance with project security policies.