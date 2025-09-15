use std::collections::HashMap;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

/// Service identifier
pub type ServiceId = String;

/// Version information for services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ServiceVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: Option<String>,
}

impl ServiceVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
        }
    }

    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }
}

impl std::fmt::Display for ServiceVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(build) = &self.build {
            write!(f, "-{}", build)?;
        }
        Ok(())
    }
}

/// Service status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    /// Service is initializing
    Initializing,
    /// Service is healthy and running
    Ready,
    /// Service is busy processing requests
    Busy,
    /// Service is in maintenance mode
    Maintenance,
    /// Service has encountered errors but is recovering
    Degraded,
    /// Service has failed and is not available
    Failed,
    /// Service is being shut down
    ShuttingDown,
    /// Service has been terminated
    Terminated,
}

impl ServiceStatus {
    /// Check if service is available for requests
    pub fn is_available(&self) -> bool {
        matches!(self, ServiceStatus::Ready | ServiceStatus::Busy)
    }

    /// Check if service is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, ServiceStatus::Failed | ServiceStatus::Terminated)
    }
}

/// Health check result for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub service_id: ServiceId,
    pub status: ServiceStatus,
    pub message: Option<String>,
    pub response_time_ms: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub next_check: chrono::DateTime<chrono::Utc>,
}

/// Service capabilities declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    pub supported_operations: Vec<String>,
    pub max_concurrent_requests: Option<u32>,
    pub rate_limits: Option<HashMap<String, u32>>,
    pub dependencies: Vec<ServiceId>,
    pub provides: Vec<String>,
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub id: ServiceId,
    pub name: String,
    pub description: String,
    pub version: ServiceVersion,
    pub status: ServiceStatus,
    pub capabilities: ServiceCapabilities,
    pub health_check_endpoint: Option<String>,
    pub priority: ServicePriority,
    pub tags: Vec<String>,
}

/// Service priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServicePriority {
    /// Lowest priority - clean up tasks, analytics
    Low = 0,
    /// Normal priority - standard operations
    Normal = 1,
    /// High priority - critical operations
    High = 2,
    /// Critical priority - must never fail
    Critical = 3,
}

/// Message types for inter-service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Request for service operation
    Request,
    /// Response to a request
    Response,
    /// Broadcast message to all services
    Broadcast,
    /// Targeted message to specific services
    Direct,
    /// Health status update
    HealthUpdate,
    /// Service discovery message
    ServiceDiscovery,
}

/// Message envelope for inter-service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMessage {
    pub message_id: String,
    pub message_type: MessageType,
    pub source_service: ServiceId,
    pub target_service: Option<ServiceId>,
    pub command: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<String>,
    pub priority: ServicePriority,
    pub immediate: bool,
}

/// Service configuration properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfiguration {
    pub properties: HashMap<String, serde_json::Value>,
    pub environment: HashMap<String, String>,
    pub secrets: HashMap<String, String>,
}

/// Lifecycle event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    /// Service is starting up
    Starting,
    /// Service is ready to accept requests
    Started,
    /// Service is stopping
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service has failed
    Failed,
    /// Service health has changed
    HealthChanged,
}

/// Lifecycle event notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleNotification {
    pub service_id: ServiceId,
    pub event: LifecycleEvent,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
