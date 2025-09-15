use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the status of a security component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentStatus {
    /// The component is operating normally
    Operational,
    /// The component is healthy and functioning as expected
    Healthy,
    /// The component is experiencing degraded performance
    Degraded,
    /// The component is not functioning correctly
    Failed,
    /// The component is starting up
    Starting,
    /// The component is shutting down
    ShuttingDown,
}

/// Represents the result of a security operation
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Represents a security error
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization failed
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Already exists
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// WebAuthn error
    #[error("WebAuthn error: {0}")]
    WebAuthnError(String),

    /// Rate limited
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// Timeout
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// WebAuthn error
    #[error("WebAuthn error: {0}")]
    WebAuthn(String),
}

/// Represents a user context for security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// The user's unique identifier
    pub user_id: String,
    /// The user's username
    pub username: String,
    /// The user's email
    pub email: String,
    /// The user's roles
    pub roles: Vec<String>,
    /// When the context was created
    pub created_at: DateTime<Utc>,
    /// When the context expires
    pub expires_at: Option<DateTime<Utc>>,
}

/// Represents an audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Authentication event
    Authentication,
    /// Registration event
    Registration,
    /// Authorization event
    Authorization,
    /// Data access event
    DataAccess,
    /// Configuration change
    ConfigurationChange,
    /// Security event
    SecurityEvent,
    /// Security alert
    SecurityAlert,
    /// Security rate limit exceeded
    SecurityRateLimitExceeded,
    /// Credential management event
    CredentialManagement,
    /// Credential deletion
    CredentialDeletion,
    /// System event
    System,
}

/// Represents the severity of an audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventSeverity {
    /// Informational event
    Info,
    /// Low severity event
    Low,
    /// Medium severity event
    Medium,
    /// High severity event
    High,
    /// Warning event
    Warning,
    /// Error event
    Error,
    /// Critical event
    Critical,
}

/// Represents the context of an audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventContext {
    /// The event type
    pub event_type: AuditEventType,
    /// The event severity
    pub severity: AuditEventSeverity,
    /// The event message
    pub message: String,
    /// Additional event data
    pub data: serde_json::Value,
    /// The source IP address
    pub source_ip: Option<String>,
    /// The user agent
    pub user_agent: Option<String>,
    /// The timestamp of the event
    pub timestamp: DateTime<Utc>,
}

impl AuditEventContext {
    /// Creates a new AuditEventContext with the given parameters
    pub fn new(
        event_type: AuditEventType,
        severity: AuditEventSeverity,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            event_type,
            severity,
            message: message.into(),
            data,
            source_ip: None,
            user_agent: None,
            timestamp: Utc::now(),
        }
    }

    /// Sets the source IP address
    pub fn with_source_ip(mut self, source_ip: Option<String>) -> Self {
        self.source_ip = source_ip;
        self
    }

    /// Sets the user agent
    pub fn with_user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// Sets the timestamp
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the severity
    pub fn with_severity(mut self, severity: AuditEventSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Adds metadata to the event
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        if let Some(obj) = self.data.as_object_mut() {
            obj.insert(key.into(), serde_json::to_value(value).unwrap_or_default());
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key.into(), serde_json::to_value(value).unwrap_or_default());
            self.data = serde_json::Value::Object(map);
        }
        self
    }
}

/// Trait for audit logging
#[async_trait]
pub trait AuditLogger: Send + Sync + 'static {
    /// Log an audit event
    async fn log(&self, event: &AuditEventContext) -> Result<(), SecurityError>;
}

/// Trait for cryptographic operations
#[async_trait]
pub trait CryptoOps: Send + Sync + 'static {
    /// Encrypt data
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Decrypt data
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Generate a random key
    async fn generate_key(&self, key_size: usize) -> Result<Vec<u8>, SecurityError>;
}

/// Trait for master key management
#[async_trait]
pub trait MasterKeyManager: Send + Sync + 'static {
    /// Get the current master key
    async fn get_current_key(&self) -> Result<Vec<u8>, SecurityError>;

    /// Rotate the master key
    async fn rotate_key(&self) -> Result<(), SecurityError>;

    /// Re-encrypt data with a new key
    async fn reencrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;
}

/// Trait for data key management
#[async_trait]
pub trait DataKeyManager: Send + Sync + 'static {
    /// Generate a new data key
    async fn generate_data_key(&self) -> Result<Vec<u8>, SecurityError>;

    /// Decrypt a data key
    async fn decrypt_data_key(&self, encrypted_key: &[u8]) -> Result<Vec<u8>, SecurityError>;
}

/// Represents a network context
#[derive(Debug, Clone, Default)]
pub struct NetworkContext {
    /// Source IP address
    pub source_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Protocol used
    pub protocol: Option<String>,
}

/// Represents a resource context
#[derive(Debug, Clone, Default)]
pub struct ResourceContext {
    /// Resource type
    pub resource_type: String,
    /// Resource ID
    pub resource_id: String,
    /// Resource action
    pub action: String,
}

/// Represents the type of operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Authentication operation
    Authentication,
    /// Authorization operation
    Authorization,
    /// Data access operation
    DataAccess,
    /// Configuration change
    ConfigurationChange,
    /// System operation
    System,
}

/// Represents an operation context
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// Unique operation ID
    pub operation_id: String,
    /// Request ID for correlation
    pub request_id: String,
    /// Operation start time
    pub start_time: DateTime<Utc>,
    /// Network context
    pub network_context: Option<NetworkContext>,
    /// Resource context
    pub resource_context: Option<ResourceContext>,
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,
    /// Operation type
    pub operation_type: OperationType,
}

/// Represents encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// The encryption algorithm to use
    pub algorithm: String,
    /// The key size in bits
    pub key_size: usize,
    /// The initialization vector size in bytes
    pub iv_size: usize,
    /// Whether to use authenticated encryption
    pub authenticated: bool,
}
