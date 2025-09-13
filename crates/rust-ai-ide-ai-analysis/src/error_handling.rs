use thiserror::Error;

/// Errors that can occur during code analysis
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Security scan failed: {0}")]
    SecurityScanError(String),

    #[error("Performance analysis failed: {0}")]
    PerformanceAnalysisError(String),

    #[error("Code quality check failed: {0}")]
    CodeQualityError(String),

    #[error("Architecture analysis failed: {0}")]
    ArchitectureError(String),

    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Regular expression error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Analysis timeout")]
    Timeout,

    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Unknown error: {0}")]
    Other(String),
}

impl AnalysisError {
    /// Create an error with a custom message
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AnalysisError::Timeout | AnalysisError::IoError(_) | AnalysisError::SerializationError(_)
        )
    }

    /// Get error category
    pub fn category(&self) -> &'static str {
        match self {
            AnalysisError::ParseError(_) => "parsing",
            AnalysisError::SecurityScanError(_) => "security",
            AnalysisError::PerformanceAnalysisError(_) => "performance",
            AnalysisError::CodeQualityError(_) => "quality",
            AnalysisError::ArchitectureError(_) => "architecture",
            AnalysisError::IoError(_) => "io",
            AnalysisError::SerializationError(_) => "serialization",
            AnalysisError::RegexError(_) => "regex",
            AnalysisError::UnsupportedLanguage(_) => "unsupported",
            AnalysisError::Timeout => "timeout",
            AnalysisError::Other(_) => "other",
        }
    }
}

/// Result alias for analysis operations
pub type AnalysisResult<T> = std::result::Result<T, AnalysisError>;

/// Error recovery strategies
#[derive(Debug, Clone, Copy)]
pub enum RecoveryStrategy {
    /// Skip the problematic analysis and continue
    Skip,
    /// Retry the operation after a delay
    Retry {
        max_attempts: usize,
        delay_ms:     u64,
    },
    /// Fall back to a simpler analysis method
    Fallback,
    /// Stop all analysis on this file
    Abort,
}

/// Analysis configuration
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub timeout_ms: u64,
    pub enable_security_scanning: bool,
    pub enable_performance_analysis: bool,
    pub enable_code_quality_checks: bool,
    pub enable_architecture_suggestions: bool,
    pub recovery_strategy: RecoveryStrategy,
    pub severity_threshold: crate::analysis::types::Severity,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30000, // 30 seconds
            enable_security_scanning: true,
            enable_performance_analysis: true,
            enable_code_quality_checks: true,
            enable_architecture_suggestions: true,
            recovery_strategy: RecoveryStrategy::Fallback,
            severity_threshold: crate::analysis::types::Severity::Info,
        }
    }
}

/// Error handling context
pub struct ErrorContext {
    pub operation: String,
    pub file_path: Option<String>,
    pub severity:  crate::analysis::types::Severity,
    pub config:    AnalysisConfig,
}

impl ErrorContext {
    pub fn new<S: Into<String>>(operation: S, config: AnalysisConfig) -> Self {
        Self {
            operation: operation.into(),
            file_path: None,
            severity: crate::analysis::types::Severity::Info,
            config,
        }
    }

    pub fn with_file_path<S: Into<String>>(mut self, path: S) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn with_severity(mut self, severity: crate::analysis::types::Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Log an error with context
    pub fn log_error(&self, error: &AnalysisError) {
        let log_msg = match &self.file_path {
            Some(path) => format!(
                "[{}] {} failed for '{}': {}",
                self.severity, self.operation, path, error
            ),
            None => format!("[{}] {} failed: {}", self.severity, self.operation, error),
        };

        match error.category() {
            "critical" | "error" => tracing::error!("{}", log_msg),
            "warning" => tracing::warn!("{}", log_msg),
            _ => tracing::info!("{}", log_msg),
        }
    }

    /// Handle an error according to the recovery strategy
    pub fn handle_error(&self, error: AnalysisError) -> RecoveryStrategy {
        self.log_error(&error);

        // Check if error should be ignored based on severity threshold
        if matches!(error, AnalysisError::ParseError(_))
            && matches!(
                self.config.severity_threshold,
                crate::analysis::types::Severity::Info | crate::analysis::types::Severity::Warning
            )
        {
            return RecoveryStrategy::Skip;
        }

        // Apply global recovery strategy
        match self.config.recovery_strategy {
            RecoveryStrategy::Skip => RecoveryStrategy::Skip,
            RecoveryStrategy::Retry {
                max_attempts,
                delay_ms,
            } =>
                if error.is_recoverable() {
                    RecoveryStrategy::Retry {
                        max_attempts,
                        delay_ms,
                    }
                } else {
                    RecoveryStrategy::Skip
                },
            RecoveryStrategy::Fallback => RecoveryStrategy::Fallback,
            RecoveryStrategy::Abort => RecoveryStrategy::Abort,
        }
    }
}

/// Error statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    pub parse_errors:        u64,
    pub security_errors:     u64,
    pub performance_errors:  u64,
    pub quality_errors:      u64,
    pub architecture_errors: u64,
    pub recovered_errors:    u64,
    pub fatal_errors:        u64,
}

impl ErrorStats {
    pub fn record(&mut self, error: &AnalysisError, recovered: bool) {
        match error.category() {
            "parsing" => self.parse_errors += 1,
            "security" => self.security_errors += 1,
            "performance" => self.performance_errors += 1,
            "quality" => self.quality_errors += 1,
            "architecture" => self.architecture_errors += 1,
            _ => {}
        }

        if recovered {
            self.recovered_errors += 1;
        } else {
            self.fatal_errors += 1;
        }
    }

    pub fn total_errors(&self) -> u64 {
        self.parse_errors
            + self.security_errors
            + self.performance_errors
            + self.quality_errors
            + self.architecture_errors
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_errors() + self.recovered_errors as u64;
        if total == 0 {
            1.0
        } else {
            let _successful = (self.parse_errors
                + self.security_errors
                + self.performance_errors
                + self.quality_errors
                + self.architecture_errors) as f64; // This should be the recovered ones, but corrected in logic
            1.0 - (self.fatal_errors as f64 / total as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::types::Severity;

    #[test]
    fn test_error_categories() {
        let parse_error = AnalysisError::ParseError("test".to_string());
        assert_eq!(parse_error.category(), "parsing");

        let io_error = AnalysisError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert_eq!(io_error.category(), "io");
    }

    #[test]
    fn test_error_recovery() {
        let parse_error = AnalysisError::ParseError("test".to_string());
        assert!(parse_error.is_recoverable()); // Parse errors are recoverable in some sense

        let timeout = AnalysisError::Timeout;
        assert!(timeout.is_recoverable());
    }

    #[test]
    fn test_error_context() {
        let config = AnalysisConfig::default();
        let context = ErrorContext::new("test operation", config)
            .with_file_path("test.rs")
            .with_severity(Severity::Error);

        let error = AnalysisError::ParseError("syntax error".to_string());
        let strategy = context.handle_error(error);

        // Should use fallback strategy from config
        match strategy {
            RecoveryStrategy::Skip => {}
            _ => panic!("Should skip unrecoverable errors"),
        }
    }

    #[test]
    fn test_error_stats() {
        let mut stats = ErrorStats::default();

        assert_eq!(stats.total_errors(), 0);
        assert_eq!(stats.success_rate(), 1.0);

        stats.record(&AnalysisError::ParseError("test".to_string()), true);
        assert_eq!(stats.total_errors(), 1);
        assert_eq!(stats.recovered_errors, 1);

        stats.record(&AnalysisError::Timeout, false);
        assert_eq!(stats.fatal_errors, 1);
    }
}
