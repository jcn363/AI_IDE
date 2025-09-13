//! # Quality Intelligence Dashboard Errors
//!
//! This module defines all error types used in the quality intelligence dashboard.
//! Errors are designed to provide clear, actionable feedback for debugging and user communication.

use std::fmt;

/// Main dashboard error type
#[derive(Debug)]
pub enum DashboardError {
    /// Configuration errors
    Configuration(ConfigurationError),

    /// Engine initialization errors
    Engine(EngineError),

    /// Metric collection errors
    MetricCollection(MetricCollectionError),

    /// Trend analysis errors
    TrendAnalysis(TrendAnalysisError),

    /// Visualization errors
    Visualization(VisualizationError),

    /// Collaboration errors
    Collaboration(CollaborationError),

    /// UI integration errors
    UiIntegration(UiIntegrationError),

    /// Security validation errors
    Security(SecurityError),

    /// External service errors
    ExternalService(ExternalServiceError),

    /// Database errors
    Database(DatabaseError),

    /// Performance errors
    Performance(PerformanceError),

    /// Generic system errors
    System(SystemError),
}

/// Type alias for DashboardResult
pub type DashboardResult<T> = Result<T, DashboardError>;

/// Configuration validation errors
#[derive(Debug)]
pub enum ConfigurationError {
    /// Invalid configuration parameter
    InvalidParameter {
        parameter: String,
        reason:    String,
    },

    /// Missing required configuration
    MissingConfig(String),

    /// Configuration file access error
    FileAccess(String),

    /// Configuration parsing error
    ParseError(String),
}

/// Engine initialization and operation errors
#[derive(Debug)]
pub enum EngineError {
    /// Engine initialization failed
    InitializationFailed(String),

    /// Engine shutdown error
    ShutdownError(String),

    /// Engine state corruption
    StateCorruption(String),

    /// Resource allocation failed
    ResourceAllocation(String),
}

/// Metric collection system errors
#[derive(Debug)]
pub enum MetricCollectionError {
    /// Metric source unavailable
    SourceUnavailable(String),

    /// Metric validation failed
    ValidationFailed(String),

    /// Metric aggregation error
    AggregationFailed(String),

    /// Rate limiting exceeded
    RateLimitExceeded { attempted: u32, limit: u32 },
}

/// Trend analysis and forecasting errors
#[derive(Debug)]
pub enum TrendAnalysisError {
    /// Insufficient data for analysis
    InsufficientData(String),

    /// Statistical analysis failed
    StatisticalError(String),

    /// Forecasting model error
    ForecastingError(String),

    /// Time series data corruption
    DataCorruption(String),
}

/// Visualization and scoring errors
#[derive(Debug)]
pub enum VisualizationError {
    /// Chart generation failed
    ChartGenerationFailed(String),

    /// Data transformation error
    DataTransformation(String),

    /// Rendering engine error
    RenderingError(String),

    /// Widget configuration error
    WidgetConfigError(String),
}

/// Collaboration system errors
#[derive(Debug)]
pub enum CollaborationError {
    /// Session creation failed
    SessionCreationFailed(String),

    /// User permission denied
    PermissionDenied(String),

    /// Real-time sync failed
    SyncError(String),

    /// Team size exceeded
    TeamSizeExceeded { current: usize, maximum: usize },
}

/// UI integration errors
#[derive(Debug)]
pub enum UiIntegrationError {
    /// Update transmission failed
    UpdateFailed(String),

    /// Layout initialization error
    LayoutError(String),

    /// Export operation failed
    ExportError(String),

    /// Accessibility check failed
    AccessibilityError(String),
}

/// Security validation errors
#[derive(Debug)]
pub enum SecurityError {
    /// Path traversal attempt detected
    PathTraversal {
        attempted_path:      String,
        sanitization_result: String,
    },

    /// Unauthorized access attempt
    UnauthorizedAccess(String),

    /// Encryption/decryption error
    EncryptionError(String),

    /// Audit logging failed
    AuditFailure(String),
}

/// External service communication errors
#[derive(Debug)]
pub enum ExternalServiceError {
    /// Service connection failed
    ConnectionFailed(String),

    /// API request failed
    ApiRequestError(String),

    /// Service timeout
    Timeout {
        service:    String,
        timeout_ms: u64,
        operation:  String,
    },

    /// Service returned invalid response
    InvalidResponse(String),
}

/// Database operation errors
#[derive(Debug)]
pub enum DatabaseError {
    /// Connection establishment failed
    ConnectionFailed(String),

    /// Query execution failed
    QueryFailed(String),

    /// Migration error
    MigrationError(String),

    /// Data integrity violation
    IntegrityViolation(String),
}

/// Performance and resource errors
#[derive(Debug)]
pub enum PerformanceError {
    /// Memory usage exceeded
    MemoryLimitExceeded { used: usize, limit: usize },

    /// CPU usage threshold exceeded
    CpuThresholdExceeded { usage: f64, threshold: f64 },

    /// Slow operation detected
    SlowOperation {
        operation:    String,
        duration_ms:  u64,
        threshold_ms: u64,
    },

    /// Resource contention detected
    ResourceContention(String),
}

/// Generic system errors
#[derive(Debug)]
pub enum SystemError {
    /// I/O operation failed
    IoError(String),

    /// Serialization/deserialization error
    SerializationError(String),

    /// Unspecified internal error
    InternalError(String),

    /// Unhandled error condition
    UnknownError(String),
}

impl fmt::Display for DashboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DashboardError::Configuration(e) => write!(f, "Configuration error: {}", e),
            DashboardError::Engine(e) => write!(f, "Engine error: {}", e),
            DashboardError::MetricCollection(e) => write!(f, "Metric collection error: {}", e),
            DashboardError::TrendAnalysis(e) => write!(f, "Trend analysis error: {}", e),
            DashboardError::Visualization(e) => write!(f, "Visualization error: {}", e),
            DashboardError::Collaboration(e) => write!(f, "Collaboration error: {}", e),
            DashboardError::UiIntegration(e) => write!(f, "UI integration error: {}", e),
            DashboardError::Security(e) => write!(f, "Security error: {}", e),
            DashboardError::ExternalService(e) => write!(f, "External service error: {}", e),
            DashboardError::Database(e) => write!(f, "Database error: {}", e),
            DashboardError::Performance(e) => write!(f, "Performance error: {}", e),
            DashboardError::System(e) => write!(f, "System error: {}", e),
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::InvalidParameter { parameter, reason } => {
                write!(f, "Invalid parameter '{}': {}", parameter, reason)
            }
            ConfigurationError::MissingConfig(config) => {
                write!(f, "Missing required configuration: {}", config)
            }
            ConfigurationError::FileAccess(path) => {
                write!(f, "Configuration file access failed: {}", path)
            }
            ConfigurationError::ParseError(reason) => {
                write!(f, "Configuration parsing failed: {}", reason)
            }
        }
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::InitializationFailed(reason) => {
                write!(f, "Engine initialization failed: {}", reason)
            }
            EngineError::ShutdownError(reason) => write!(f, "Engine shutdown error: {}", reason),
            EngineError::StateCorruption(reason) => {
                write!(f, "Engine state corruption: {}", reason)
            }
            EngineError::ResourceAllocation(reason) => {
                write!(f, "Resource allocation failed: {}", reason)
            }
        }
    }
}

impl fmt::Display for MetricCollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricCollectionError::SourceUnavailable(source) => {
                write!(f, "Metric source unavailable: {}", source)
            }
            MetricCollectionError::ValidationFailed(reason) => {
                write!(f, "Metric validation failed: {}", reason)
            }
            MetricCollectionError::AggregationFailed(reason) => {
                write!(f, "Metric aggregation failed: {}", reason)
            }
            MetricCollectionError::RateLimitExceeded { attempted, limit } => write!(
                f,
                "Rate limit exceeded: {} requested, {} allowed per second",
                attempted, limit
            ),
        }
    }
}

impl fmt::Display for TrendAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrendAnalysisError::InsufficientData(reason) => {
                write!(f, "Insufficient data for trend analysis: {}", reason)
            }
            TrendAnalysisError::StatisticalError(reason) => {
                write!(f, "Statistical analysis error: {}", reason)
            }
            TrendAnalysisError::ForecastingError(reason) => {
                write!(f, "Forecasting model error: {}", reason)
            }
            TrendAnalysisError::DataCorruption(reason) => {
                write!(f, "Time series data corruption: {}", reason)
            }
        }
    }
}

impl fmt::Display for VisualizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisualizationError::ChartGenerationFailed(reason) => {
                write!(f, "Chart generation failed: {}", reason)
            }
            VisualizationError::DataTransformation(reason) => {
                write!(f, "Data transformation error: {}", reason)
            }
            VisualizationError::RenderingError(reason) => {
                write!(f, "Rendering engine error: {}", reason)
            }
            VisualizationError::WidgetConfigError(reason) => {
                write!(f, "Widget configuration error: {}", reason)
            }
        }
    }
}

impl fmt::Display for CollaborationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollaborationError::SessionCreationFailed(reason) => {
                write!(f, "Session creation failed: {}", reason)
            }
            CollaborationError::PermissionDenied(reason) => {
                write!(f, "Permission denied: {}", reason)
            }
            CollaborationError::SyncError(reason) => write!(f, "Real-time sync failed: {}", reason),
            CollaborationError::TeamSizeExceeded { current, maximum } => write!(
                f,
                "Team size limit exceeded: {} current, {} maximum",
                current, maximum
            ),
        }
    }
}

impl fmt::Display for UiIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiIntegrationError::UpdateFailed(reason) => write!(f, "UI update failed: {}", reason),
            UiIntegrationError::LayoutError(reason) => {
                write!(f, "Layout initialization error: {}", reason)
            }
            UiIntegrationError::ExportError(reason) => {
                write!(f, "Export operation failed: {}", reason)
            }
            UiIntegrationError::AccessibilityError(reason) => {
                write!(f, "Accessibility check failed: {}", reason)
            }
        }
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::PathTraversal {
                attempted_path,
                sanitization_result,
            } => write!(
                f,
                "Path traversal detected: {} sanitized to {}",
                attempted_path, sanitization_result
            ),
            SecurityError::UnauthorizedAccess(reason) => {
                write!(f, "Unauthorized access attempt: {}", reason)
            }
            SecurityError::EncryptionError(reason) => {
                write!(f, "Encryption/decryption error: {}", reason)
            }
            SecurityError::AuditFailure(reason) => write!(f, "Audit logging failed: {}", reason),
        }
    }
}

impl fmt::Display for ExternalServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExternalServiceError::ConnectionFailed(reason) => {
                write!(f, "Service connection failed: {}", reason)
            }
            ExternalServiceError::ApiRequestError(reason) => {
                write!(f, "API request error: {}", reason)
            }
            ExternalServiceError::Timeout {
                service,
                timeout_ms,
                operation,
            } => write!(
                f,
                "{} operation on {} timed out after {}ms",
                operation, service, timeout_ms
            ),
            ExternalServiceError::InvalidResponse(reason) => {
                write!(f, "Service returned invalid response: {}", reason)
            }
        }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed(reason) => {
                write!(f, "Database connection failed: {}", reason)
            }
            DatabaseError::QueryFailed(reason) => write!(f, "Database query failed: {}", reason),
            DatabaseError::MigrationError(reason) => {
                write!(f, "Database migration error: {}", reason)
            }
            DatabaseError::IntegrityViolation(reason) => {
                write!(f, "Database integrity violation: {}", reason)
            }
        }
    }
}

impl fmt::Display for PerformanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PerformanceError::MemoryLimitExceeded { used, limit } => write!(
                f,
                "Memory limit exceeded: {}MB used, {}MB limit",
                used, limit
            ),
            PerformanceError::CpuThresholdExceeded { usage, threshold } => write!(
                f,
                "CPU threshold exceeded: {:.2}% usage, {:.2}% threshold",
                usage, threshold
            ),
            PerformanceError::SlowOperation {
                operation,
                duration_ms,
                threshold_ms,
            } => write!(
                f,
                "Slow operation detected: {} took {}ms (threshold: {}ms)",
                operation, duration_ms, threshold_ms
            ),
            PerformanceError::ResourceContention(reason) => {
                write!(f, "Resource contention detected: {}", reason)
            }
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::IoError(reason) => write!(f, "I/O error: {}", reason),
            SystemError::SerializationError(reason) => write!(f, "Serialization error: {}", reason),
            SystemError::InternalError(reason) => write!(f, "Internal error: {}", reason),
            SystemError::UnknownError(reason) => write!(f, "Unknown error: {}", reason),
        }
    }
}

// Error conversion implementations
impl std::error::Error for DashboardError {}
impl std::error::Error for ConfigurationError {}
impl std::error::Error for EngineError {}
impl std::error::Error for MetricCollectionError {}
impl std::error::Error for TrendAnalysisError {}
impl std::error::Error for VisualizationError {}
impl std::error::Error for CollaborationError {}
impl std::error::Error for UiIntegrationError {}
impl std::error::Error for SecurityError {}
impl std::error::Error for ExternalServiceError {}
impl std::error::Error for DatabaseError {}
impl std::error::Error for PerformanceError {}
impl std::error::Error for SystemError {}

// From implementations for convenience
impl From<ConfigurationError> for DashboardError {
    fn from(error: ConfigurationError) -> Self {
        DashboardError::Configuration(error)
    }
}

impl From<EngineError> for DashboardError {
    fn from(error: EngineError) -> Self {
        DashboardError::Engine(error)
    }
}

impl From<MetricCollectionError> for DashboardError {
    fn from(error: MetricCollectionError) -> Self {
        DashboardError::MetricCollection(error)
    }
}

impl From<TrendAnalysisError> for DashboardError {
    fn from(error: TrendAnalysisError) -> Self {
        DashboardError::TrendAnalysis(error)
    }
}

impl From<VisualizationError> for DashboardError {
    fn from(error: VisualizationError) -> Self {
        DashboardError::Visualization(error)
    }
}

impl From<CollaborationError> for DashboardError {
    fn from(error: CollaborationError) -> Self {
        DashboardError::Collaboration(error)
    }
}

impl From<UiIntegrationError> for DashboardError {
    fn from(error: UiIntegrationError) -> Self {
        DashboardError::UiIntegration(error)
    }
}

impl From<SecurityError> for DashboardError {
    fn from(error: SecurityError) -> Self {
        DashboardError::Security(error)
    }
}

impl From<ExternalServiceError> for DashboardError {
    fn from(error: ExternalServiceError) -> Self {
        DashboardError::ExternalService(error)
    }
}

impl From<DatabaseError> for DashboardError {
    fn from(error: DatabaseError) -> Self {
        DashboardError::Database(error)
    }
}

impl From<PerformanceError> for DashboardError {
    fn from(error: PerformanceError) -> Self {
        DashboardError::Performance(error)
    }
}

impl From<SystemError> for DashboardError {
    fn from(error: SystemError) -> Self {
        DashboardError::System(error)
    }
}
