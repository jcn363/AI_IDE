/*! # Rust AI IDE Supervisor

Core high availability framework providing:
- Service Supervisor: Process monitoring and restart logic for critical services
- State Persistence: Checkpoint/savepoint mechanisms for crash recovery
- IPC Recovery: Channel health monitoring and automatic reconnection

## Architecture

This crate implements three main components:

1. **Service Supervisor** - Monitors critical services (AI LSP, model manager, webhooks)
   and manages restart policies with exponential backoff.

2. **State Persistence Layer** - Stores service states and pending operations for
   crash recovery using SQLite and file-based checkpoints.

3. **IPC Recovery System** - Monitors communication channels, buffers messages,
   and handles reconnection logic post-failure.

## Key Patterns Used

- `Arc<Mutex<T>>` for async state access to prevent race conditions
- `tokio::sync` primitives for component communication
- EventBus integration for supervisor notifications
- Double-locking for lazy initialization
- Standardized error handling with `IDEResult<T>`

## Usage Example

```rust,no_run
use rust_ai_ide_supervisor::{Supervisor, ServiceConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

let supervisor = Supervisor::new()?;
let service_config = ServiceConfig {
    name: "ai_lsp".to_string(),
    command: "/path/to/lsp".to_string(),
    args: vec![],
    health_check_timeout: std::time::Duration::from_secs(30),
    restart_policy: RestartPolicy::ExponentialBackoff {
        base_delay: std::time::Duration::from_secs(1),
        max_delay: std::time::Duration::from_secs(60),
        max_attempts: 5,
    },
};

supervisor.register_service(service_config).await?;
supervisor.start_monitoring().await?;
# Ok(())
```

*/

// Re-exports for public API
pub mod error;
pub mod service_supervisor;
pub mod state_persistence;
pub mod ipc_recovery;
pub mod types;

// Utility modules
mod utils;
mod migration;
mod commands;

pub use error::{SupervisorError, SupervisorResult};
pub use service_supervisor::{Supervisor, ServiceConfig, RestartPolicy, ServiceMonitor};
pub use state_persistence::{StatePersistence, Checkpoint, StateSnapshot};
pub use ipc_recovery::{IpcMonitor, ChannelHealth, RecoveryQueue};
pub use types::*;

use std::sync::Arc;
use tokio::sync::Mutex;

/// Main Supervisor type providing high availability monitoring and state management
pub type Supervisor = service_supervisor::Supervisor;

/// Service supervision result type
pub type SupervisorResult<T> = Result<T, SupervisorError>;

/// Initialize the supervisor with default configuration
pub fn init() -> SupervisorResult<()> {
    service_supervisor::init()
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_supervisor_creation() {
        let supervisor = Supervisor::new().expect("Failed to create supervisor");
        assert!(supervisor.is_ready().await);
    }

    #[test]
    async fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }
}