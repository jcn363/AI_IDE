//! Core type definitions for the supervisor system

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Service identifier type
pub type ServiceId = String;

/// Channel identifier for IPC operations
pub type ChannelId = String;

/// Checkpoint identifier
pub type CheckpointId = Uuid;

/// Service state enum representing current status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceState {
    /// Service is starting up
    Starting,
    /// Service is running and healthy
    Running,
    /// Service is being restarted
    Restarting,
    /// Service is stopping normally
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service has failed
    Failed(String),
    /// Service is in recovery state
    Recovering,
}

/// Restart policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Never automatically restart
    Never,
    /// Always restart immediately
    Always,
    /// Restart with exponential backoff
    ExponentialBackoff {
        /// Base delay before first retry
        base_delay: std::time::Duration,
        /// Maximum delay between retries
        max_delay: std::time::Duration,
        /// Maximum number of restart attempts
        max_attempts: usize,
    },
    /// Restart with fixed delay
    FixedDelay {
        /// Delay before restart
        delay: std::time::Duration,
        /// Maximum number of restart attempts
        max_attempts: usize,
    },
}

/// Service configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Unique service identifier
    pub id: ServiceId,
    /// Human-readable service name
    pub name: String,
    /// Command to execute for the service
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
    /// Working directory for the service
    pub working_dir: Option<String>,
    /// Environment variables for the service
    pub environment: HashMap<String, String>,
    /// Health check timeout duration
    pub health_check_timeout: std::time::Duration,
    /// Restart policy for this service
    pub restart_policy: RestartPolicy,
    /// Maximum time to wait for graceful shutdown
    pub shutdown_timeout: std::time::Duration,
    /// Critical service flag - system can shut down if this fails
    pub critical: bool,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Whether the service is healthy
    pub healthy: bool,
    /// Timestamp of the health check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Health check duration
    pub duration: std::time::Duration,
    /// Optional error message if unhealthy
    pub error_message: Option<String>,
    /// Health score (0.0 to 1.0)
    pub score: f64,
}

/// Service monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Number of successful health checks
    pub successful_checks: u64,
    /// Number of failed health checks
    pub failed_checks: u64,
    /// Total number of restarts
    pub restart_count: u32,
    /// Last successful health check timestamp
    pub last_successful_check: Option<chrono::DateTime<chrono::Utc>>,
    /// Last failed health check timestamp
    pub last_failed_check: Option<chrono::DateTime<chrono::Utc>>,
    /// Current uptime duration
    pub uptime: Option<std::time::Duration>,
    /// Memory usage in bytes (if available)
    pub memory_usage: Option<u64>,
}

/// IPC Channel health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelHealth {
    /// Channel identifier
    pub id: ChannelId,
    /// Whether the channel is currently healthy
    pub healthy: bool,
    /// Last successful message timestamp
    pub last_message_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Last failure timestamp
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of buffered messages waiting to be sent
    pub buffered_message_count: usize,
    /// Number of reconnection attempts
    pub reconnection_attempts: u32,
}

/// IPC message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Unique message identifier
    pub id: Uuid,
    /// Message type for routing
    pub message_type: String,
    /// Message payload
    pub payload: serde_json::Value,
    /// Timestamp when message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Number of retry attempts
    pub retry_count: u32,
}

/// Supervisor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    /// Maximum number of concurrent services
    pub max_concurrent_services: usize,
    /// Global health check interval
    pub health_check_interval: std::time::Duration,
    /// Maximum time to wait for all services to shutdown
    pub global_shutdown_timeout: std::time::Duration,
    /// Database path for state persistence
    pub database_path: String,
    /// Checkpoint directory path
    pub checkpoint_dir: String,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
    /// Enable automatic backups
    pub enable_auto_backup: bool,
    /// Backup interval
    pub backup_interval: std::time::Duration,
}

/// State snapshot for checkpoint recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Unique snapshot identifier
    pub id: CheckpointId,
    /// Timestamp when snapshot was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// All service states at snapshot time
    pub service_states: HashMap<ServiceId, ServiceSnapshot>,
    /// Active IPC channels
    pub ipc_channels: Vec<ChannelHealth>,
    /// Pending operations queue
    pub pending_operations: Vec<PendingOperation>,
}

/// Individual service snapshot within StateSnapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSnapshot {
    /// Service identifier
    pub service_id: ServiceId,
    /// Service state at snapshot time
    pub state: ServiceState,
    /// Service metrics
    pub metrics: ServiceMetrics,
    /// Last start timestamp
    pub last_start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Process ID if running
    pub process_id: Option<u32>,
}

/// Pending operation that needs to be recovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    /// Operation identifier
    pub id: Uuid,
    /// Operation type
    pub operation_type: String,
    /// Service this operation is for
    pub service_id: ServiceId,
    /// Operation parameters
    pub parameters: serde_json::Value,
    /// Timestamp when operation was queued
    pub queued_time: chrono::DateTime<chrono::Utc>,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Current retry count
    pub retry_count: u32,
}

/// Supervisor statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorStats {
    /// Total number of services managed
    pub total_services: usize,
    /// Number of services currently running
    pub running_services: usize,
    /// Number of services currently restarting
    pub restarting_services: usize,
    /// Number of services in failed state
    pub failed_services: usize,
    /// Total service restarts across all services
    pub total_restarts: u64,
    /// Total successful health checks
    pub total_successful_checks: u64,
    /// Total failed health checks
    pub total_failed_checks: u64,
    /// Supervisor uptime
    pub uptime: std::time::Duration,
    /// Last checkpoint timestamp
    pub last_checkpoint: Option<chrono::DateTime<chrono::Utc>>,
}

/// Event types sent by the supervisor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisorEvent {
    /// Service state changed
    ServiceStateChanged {
        service_id: ServiceId,
        old_state: ServiceState,
        new_state: ServiceState,
    },
    /// Service health check result
    HealthCheckCompleted {
        service_id: ServiceId,
        result: HealthCheckResult,
    },
    /// Service restarted
    ServiceRestarted {
        service_id: ServiceId,
        reason: String,
    },
    /// IPC channel state changed
    IpcChannelStateChanged {
        channel_id: ChannelId,
        healthy: bool,
        reason: Option<String>,
    },
    /// Checkpoint created or loaded
    CheckpointEvent {
        checkpoint_id: CheckpointId,
        event_type: CheckpointEventType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointEventType {
    Created,
    Loaded,
    Failed(String),
}

/// Thread-safe wrapper for shared state
pub type SharedSupervisorState = Arc<Mutex<SupervisorState>>;

/// Internal supervisor state
#[derive(Debug)]
pub struct SupervisorState {
    /// Configuration
    pub config: SupervisorConfig,
    /// Service registry
    pub services: HashMap<ServiceId, ServiceInfo>,
    /// Statistics
    pub stats: SupervisorStats,
    /// Running recovery operations
    pub recovery_tasks: HashMap<ServiceId, tokio::task::JoinHandle<()>>,
}

#[derive(Debug)]
pub struct ServiceInfo {
    pub config: ServiceConfig,
    pub state: ServiceState,
    pub process_handler: Option<tokio::process::Child>,
    pub last_health_check: Option<HealthCheckResult>,
    pub metrics: ServiceMetrics,
    pub monitor_task: Option<tokio::task::JoinHandle<()>>,
}

// Implementation helpers

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_services: 10,
            health_check_interval: std::time::Duration::from_secs(5),
            global_shutdown_timeout: std::time::Duration::from_secs(30),
            database_path: "supervisor.db".to_string(),
            checkpoint_dir: "checkpoints".to_string(),
            enable_detailed_logging: false,
            enable_auto_backup: true,
            backup_interval: std::time::Duration::from_secs(3600), // 1 hour
        }
    }
}

impl ServiceState {
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceState::Running)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, ServiceState::Failed(_))
    }

    pub fn is_transient(&self) -> bool {
        matches!(self, ServiceState::Starting | ServiceState::Restarting | ServiceState::Stopping | ServiceState::Recovering)
    }
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            successful_checks: 0,
            failed_checks: 0,
            restart_count: 0,
            last_successful_check: None,
            last_failed_check: None,
            uptime: None,
            memory_usage: None,
        }
    }
}

impl HealthCheckResult {
    pub fn success(duration: std::time::Duration, score: f64) -> Self {
        Self {
            healthy: true,
            timestamp: chrono::Utc::now(),
            duration,
            error_message: None,
            score,
        }
    }

    pub fn failure(duration: std::time::Duration, error_message: String) -> Self {
        Self {
            healthy: false,
            timestamp: chrono::Utc::now(),
            duration,
            error_message: Some(error_message),
            score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_state_is_running() {
        assert!(ServiceState::Running.is_running());
        assert!(!ServiceState::Failed("test".to_string()).is_running());
    }

    #[test]
    fn test_service_state_is_failed() {
        assert!(ServiceState::Failed("test".to_string()).is_failed());
        assert!(!ServiceState::Running.is_failed());
    }

    #[test]
    fn test_health_check_result_creation() {
        let success_result = HealthCheckResult::success(
            std::time::Duration::from_millis(100),
            1.0
        );
        assert!(success_result.healthy);
        assert_eq!(success_result.score, 1.0);

        let failure_result = HealthCheckResult::failure(
            std::time::Duration::from_millis(50),
            "Connection failed".to_string()
        );
        assert!(!failure_result.healthy);
        assert_eq!(failure_result.error_message.unwrap(), "Connection failed");
    }
}