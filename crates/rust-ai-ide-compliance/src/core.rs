//! Core compliance types and error handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Compliance error types
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ComplianceError {
    /// GDPR-related compliance failures
    #[error("GDPR compliance violation: {details}")]
    GdprViolation {
        details: String,
        code: Option<String>,
    },

    /// HIPAA-related compliance failures
    #[error("HIPAA compliance violation: {details}")]
    HipaaViolation {
        details: String,
        code: Option<String>,
    },

    /// Data protection and privacy errors
    #[error("Data protection error: {details}")]
    DataProtectionError {
        details: String,
        context: Option<String>,
    },

    /// Audit and logging failures
    #[error("Audit error: {details}")]
    AuditError {
        details: String,
        source: Option<String>,
    },

    /// Policy enforcement failures
    #[error("Policy enforcement error: {details}")]
    PolicyError {
        details: String,
        policy_id: Option<String>,
    },

    /// Report generation failures
    #[error("Report generation error: {details}")]
    ReportError {
        details: String,
        report_type: Option<String>,
    },

    /// External integration errors
    #[error("Integration error: {details}")]
    IntegrationError {
        details: String,
        provider: Option<String>,
    },

    /// Configuration errors
    #[error("Configuration error: {details}")]
    ConfigurationError {
        details: String,
        section: Option<String>,
    },

    /// Validation errors
    #[error("Validation error: {details}")]
    ValidationError {
        details: String,
        field: Option<String>,
    },
}

impl ComplianceError {
    /// Create a GDPR violation error
    pub fn gdpr_violation(details: impl Into<String>, code: Option<String>) -> Self {
        Self::GdprViolation {
            details: details.into(),
            code,
        }
    }

    /// Create a HIPAA violation error
    pub fn hipaa_violation(details: impl Into<String>, code: Option<String>) -> Self {
        Self::HipaaViolation {
            details: details.into(),
            code,
        }
    }

    /// Create a data protection error
    pub fn data_protection_error(details: impl Into<String>, context: Option<String>) -> Self {
        Self::DataProtectionError {
            details: details.into(),
            context,
        }
    }

    /// Check if error is critical (requires immediate attention)
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::GdprViolation { .. } | Self::HipaaViolation { .. })
    }

    /// Get error category for grouping
    pub fn category(&self) -> &'static str {
        match self {
            Self::GdprViolation { .. } => "gdpr",
            Self::HipaaViolation { .. } => "hipaa",
            Self::DataProtectionError { .. } => "data_protection",
            Self::AuditError { .. } => "audit",
            Self::PolicyError { .. } => "policy",
            Self::ReportError { .. } => "report",
            Self::IntegrationError { .. } => "integration",
            Self::ConfigurationError { .. } => "configuration",
            Self::ValidationError { .. } => "validation",
        }
    }
}

/// Compliance result type alias
pub type ComplianceResult<T> = std::result::Result<T, ComplianceError>;

/// Audit severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Informational - normal operations
    Info,
    /// Warning - potential issues
    Warning,
    /// Error - compliance violations
    Error,
    /// Critical - immediate action required
    Critical,
}

impl fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditSeverity::Info => write!(f, "INFO"),
            AuditSeverity::Warning => write!(f, "WARNING"),
            AuditSeverity::Error => write!(f, "ERROR"),
            AuditSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl From<AuditSeverity> for log::Level {
    fn from(severity: AuditSeverity) -> Self {
        match severity {
            AuditSeverity::Info => log::Level::Info,
            AuditSeverity::Warning => log::Level::Warn,
            AuditSeverity::Error => log::Level::Error,
            AuditSeverity::Critical => log::Level::Error,
        }
    }
}

/// Audit entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier
    pub id: uuid::Uuid,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event severity
    pub severity: AuditSeverity,
    /// Event category (gdpr, hipaa, policy, etc.)
    pub category: String,
    /// User ID if applicable
    pub user_id: Option<String>,
    /// Session ID for tracking
    pub session_id: Option<String>,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: String,
    /// Event details
    pub details: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Source of the event
    pub source: String,
}

impl Default for AuditEntry {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            severity: AuditSeverity::Info,
            category: String::new(),
            user_id: None,
            session_id: None,
            action: String::new(),
            resource: String::new(),
            details: String::new(),
            metadata: HashMap::new(),
            source: "compliance_engine".to_string(),
        }
    }
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(severity: AuditSeverity, category: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            severity,
            category: category.into(),
            action: action.into(),
            ..Default::default()
        }
    }

    /// Add metadata to the audit entry
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = resource.into();
        self
    }

    /// Set details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = details.into();
        self
    }

    /// Set source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

/// Compliance configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable GDPR compliance
    pub gdpr_enabled: bool,
    /// Enable HIPAA compliance
    pub hipaa_enabled: bool,
    /// Compliance frameworks to enforce
    pub frameworks: Vec<String>,
    /// Audit configuration
    pub audit: AuditConfig,
    /// Policy configuration
    pub policies: PolicyConfig,
    /// Notification configuration
    pub notifications: NotificationConfig,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            gdpr_enabled: true,
            hipaa_enabled: false,
            frameworks: vec!["gdpr".to_string()],
            audit: AuditConfig::default(),
            policies: PolicyConfig::default(),
            notifications: NotificationConfig::default(),
        }
    }
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Audit log retention days
    pub retention_days: u32,
    /// Maximum audit entries to keep
    pub max_entries: usize,
    /// Audit path (if file-based)
    pub path: Option<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 2555, // 7 years
            max_entries: 1_000_000,
            path: None,
        }
    }
}

/// Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Enable automatic policy enforcement
    pub enforcement_enabled: bool,
    /// Policy validation mode
    pub validation_mode: PolicyValidationMode,
    /// Policy conflict resolution strategy
    pub conflict_resolution: PolicyConflictResolution,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            enforcement_enabled: true,
            validation_mode: PolicyValidationMode::Strict,
            conflict_resolution: PolicyConflictResolution::DenyAll,
        }
    }
}

/// Policy validation modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyValidationMode {
    /// Strict validation - all policies must pass
    Strict,
    /// Permissive validation - allow if majority pass
    Permissive,
    /// Advisory validation - warn but allow
    Advisory,
}

/// Policy conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyConflictResolution {
    /// Deny all conflicting operations
    DenyAll,
    /// Allow only the most restrictive policy
    MostRestrictive,
    /// Require explicit consent for conflicts
    RequireConsent,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable notifications
    pub enabled: bool,
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
    /// Critical notification recipients
    pub critical_recipients: Vec<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![],
            critical_recipients: vec![],
        }
    }
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// Email notifications
    Email,
    /// Webhook notifications
    Webhook,
    /// System log notifications
    SystemLog,
    /// In-app notifications
    InApp,
}

impl fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationChannel::Email => write!(f, "email"),
            NotificationChannel::Webhook => write!(f, "webhook"),
            NotificationChannel::SystemLog => write!(f, "system_log"),
            NotificationChannel::InApp => write!(f, "in_app"),
        }
    }
}

/// Utility functions for compliance operations
pub mod utils {
    use super::*;

    /// Generate compliance report ID
    pub fn generate_report_id() -> String {
        format!("cmp-{}", uuid::Uuid::new_v4().simple())
    }

    /// Hash sensitive data for audit logging
    pub fn hash_for_audit(data: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("sha256:{}", base16ct::encode_lower(&hasher.finalize()))
    }

    /// Validate email address for notifications
    pub fn validate_email(email: &str) -> bool {
        email.contains('@') && email.len() >= 5 && email.len() <= 254
    }

    /// Check if a string contains personally identifiable information
    pub fn contains_pii(data: &str) -> bool {
        // Simple PII detection patterns
        let pii_patterns = [
            r"\b\d{3}-\d{2}-\d{4}\b",  // SSN
            r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b",  // Credit card
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",  // Email
        ];

        for pattern in &pii_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(data) {
                return true;
            }
        }
        false
    }
}