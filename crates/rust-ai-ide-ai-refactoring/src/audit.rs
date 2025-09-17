//! Audit logging and comprehensive error handling for refactoring operations

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// Audit log entry for refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp:          std::time::SystemTime,
    pub operation_id:       String,
    pub operation_type:     String,
    pub user_id:            Option<String>,
    pub file_path:          String,
    pub success:            bool,
    pub error_message:      Option<String>,
    pub safety_score:       f64,
    pub execution_time_ms:  u64,
    pub backup_created:     bool,
    pub rollback_performed: bool,
    pub metadata:           HashMap<String, serde_json::Value>,
}

/// Comprehensive error types for refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringError {
    SafetyViolation {
        message:    String,
        risk_level: String,
    },
    DependencyConflict {
        symbol:    String,
        conflicts: Vec<String>,
    },
    SyntaxError {
        line:    usize,
        message: String,
    },
    SemanticError {
        message:          String,
        affected_symbols: Vec<String>,
    },
    BackupFailure {
        reason: String,
    },
    RollbackFailure {
        reason: String,
    },
    PerformanceTimeout {
        operation:   String,
        duration_ms: u64,
    },
    ValidationFailure {
        checks:   Vec<String>,
        failures: Vec<String>,
    },
    AccessDenied {
        resource: String,
        reason:   String,
    },
    ConfigurationError {
        setting:  String,
        expected: String,
        actual:   String,
    },
}

/// Enhanced error with context and recovery suggestions
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error:                RefactoringError,
    pub context:              ErrorContext,
    pub recovery_suggestions: Vec<String>,
    pub related_errors:       Vec<RefactoringError>,
}

/// Error context information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation_id:   String,
    pub file_path:      String,
    pub line_number:    Option<usize>,
    pub symbol_name:    Option<String>,
    pub user_id:        Option<String>,
    pub workspace_root: String,
    pub stack_trace:    Option<String>,
}

/// Audit logger for refactoring operations
pub struct AuditLogger {
    log_entries:        Mutex<Vec<AuditLogEntry>>,
    max_entries:        usize,
    security_validator: SecurityValidator,
}

/// Security validator for audit logging
struct SecurityValidator;

impl SecurityValidator {
    fn validate_log_entry(&self, entry: &AuditLogEntry) -> Result<(), String> {
        // Validate that sensitive information is not logged in plain text
        if let Some(error_msg) = &entry.error_message {
            if error_msg.contains("password") || error_msg.contains("token") || error_msg.contains("key") {
                return Err("Sensitive information detected in error message".to_string());
            }
        }

        // Validate file paths don't contain sensitive information
        if entry.file_path.contains("..") || entry.file_path.contains("/") == false {
            return Err("Invalid file path in audit log".to_string());
        }

        Ok(())
    }
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_entries: usize) -> Self {
        AuditLogger {
            log_entries: Mutex::new(Vec::new()),
            max_entries,
            security_validator: SecurityValidator,
        }
    }

    /// Log a refactoring operation
    pub async fn log_operation(&self, entry: AuditLogEntry) -> Result<(), String> {
        // Security validation
        self.security_validator.validate_log_entry(&entry)?;

        let mut entries = self.log_entries.lock().await;

        // Add new entry
        entries.push(entry);

        // Maintain maximum entries limit
        if entries.len() > self.max_entries {
            let excess = entries.len() - self.max_entries;
            entries.drain(0..excess);
        }

        Ok(())
    }

    /// Get audit entries for a specific operation
    pub async fn get_operation_logs(&self, operation_id: &str) -> Vec<AuditLogEntry> {
        let entries = self.log_entries.lock().await;
        entries
            .iter()
            .filter(|entry| entry.operation_id == operation_id)
            .cloned()
            .collect()
    }

    /// Get audit entries for a specific file
    pub async fn get_file_logs(&self, file_path: &str) -> Vec<AuditLogEntry> {
        let entries = self.log_entries.lock().await;
        entries
            .iter()
            .filter(|entry| entry.file_path == file_path)
            .cloned()
            .collect()
    }

    /// Get audit entries for a specific user
    pub async fn get_user_logs(&self, user_id: &str) -> Vec<AuditLogEntry> {
        let entries = self.log_entries.lock().await;
        entries
            .iter()
            .filter(|entry| entry.user_id.as_ref() == Some(&user_id.to_string()))
            .cloned()
            .collect()
    }

    /// Generate audit report
    pub async fn generate_report(
        &self,
        start_time: std::time::SystemTime,
        end_time: std::time::SystemTime,
    ) -> AuditReport {
        let entries = self.log_entries.lock().await;

        let relevant_entries: Vec<_> = entries
            .iter()
            .filter(|entry| entry.timestamp >= start_time && entry.timestamp <= end_time)
            .collect();

        let total_operations = relevant_entries.len();
        let successful_operations = relevant_entries.iter().filter(|e| e.success).count();
        let failed_operations = total_operations - successful_operations;
        let average_execution_time = if total_operations > 0 {
            relevant_entries
                .iter()
                .map(|e| e.execution_time_ms)
                .sum::<u64>()
                / total_operations as u64
        } else {
            0
        };

        let rollback_count = relevant_entries
            .iter()
            .filter(|e| e.rollback_performed)
            .count();
        let safety_score_avg = if total_operations > 0 {
            relevant_entries.iter().map(|e| e.safety_score).sum::<f64>() / total_operations as f64
        } else {
            0.0
        };

        AuditReport {
            period_start: start_time,
            period_end: end_time,
            total_operations,
            successful_operations,
            failed_operations,
            average_execution_time_ms: average_execution_time,
            rollback_operations: rollback_count,
            average_safety_score: safety_score_avg,
            top_error_types: self.analyze_error_patterns(&relevant_entries),
        }
    }

    /// Analyze error patterns in audit entries
    fn analyze_error_patterns(&self, entries: &[&AuditLogEntry]) -> HashMap<String, usize> {
        let mut error_counts = HashMap::new();

        for entry in entries {
            if !entry.success {
                if let Some(error_msg) = &entry.error_message {
                    // Categorize errors (simplified)
                    let error_type = if error_msg.contains("safety") {
                        "Safety Violation"
                    } else if error_msg.contains("syntax") {
                        "Syntax Error"
                    } else if error_msg.contains("semantic") {
                        "Semantic Error"
                    } else if error_msg.contains("backup") {
                        "Backup Failure"
                    } else {
                        "Other Error"
                    };

                    *error_counts.entry(error_type.to_string()).or_insert(0) += 1;
                }
            }
        }

        error_counts
    }
}

/// Audit report for a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub period_start:              std::time::SystemTime,
    pub period_end:                std::time::SystemTime,
    pub total_operations:          usize,
    pub successful_operations:     usize,
    pub failed_operations:         usize,
    pub average_execution_time_ms: u64,
    pub rollback_operations:       usize,
    pub average_safety_score:      f64,
    pub top_error_types:           HashMap<String, usize>,
}

/// Error handler for refactoring operations
pub struct ErrorHandler {
    audit_logger: Arc<AuditLogger>,
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        ErrorHandler { audit_logger }
    }

    /// Handle a refactoring error with context
    pub async fn handle_error(&self, error: RefactoringError, context: ErrorContext) -> ContextualError {
        // Log the error
        let audit_entry = AuditLogEntry {
            timestamp:          std::time::SystemTime::now(),
            operation_id:       context.operation_id.clone(),
            operation_type:     "error_handling".to_string(),
            user_id:            context.user_id.clone(),
            file_path:          context.file_path.clone(),
            success:            false,
            error_message:      Some(format!("{:?}", error)),
            safety_score:       0.0,
            execution_time_ms:  0,
            backup_created:     false,
            rollback_performed: false,
            metadata:           HashMap::new(),
        };

        let _ = self.audit_logger.log_operation(audit_entry).await;

        // Generate recovery suggestions
        let recovery_suggestions = self.generate_recovery_suggestions(&error, &context);

        // Find related errors
        let related_errors = self.find_related_errors(&error, &context).await;

        ContextualError {
            error,
            context,
            recovery_suggestions,
            related_errors,
        }
    }

    /// Generate recovery suggestions for an error
    fn generate_recovery_suggestions(&self, error: &RefactoringError, _context: &ErrorContext) -> Vec<String> {
        match error {
            RefactoringError::SafetyViolation { risk_level, .. } => {
                vec![
                    format!("Review {} risk factors before proceeding", risk_level),
                    "Consider creating additional backups".to_string(),
                    "Run static analysis tools to validate safety".to_string(),
                ]
            }
            RefactoringError::DependencyConflict { symbol, conflicts } => {
                vec![
                    format!(
                        "Resolve conflicts for symbol '{}' with {} other symbols",
                        symbol,
                        conflicts.len()
                    ),
                    "Consider renaming symbols to avoid conflicts".to_string(),
                    "Review import statements for ambiguity".to_string(),
                ]
            }
            RefactoringError::SyntaxError { line, .. } => {
                vec![
                    format!("Fix syntax error at line {}", line),
                    "Run cargo check to validate syntax".to_string(),
                    "Consider using rustfmt to fix formatting issues".to_string(),
                ]
            }
            RefactoringError::BackupFailure { .. } => {
                vec![
                    "Ensure write permissions to backup directory".to_string(),
                    "Check available disk space".to_string(),
                    "Try creating backup in alternative location".to_string(),
                ]
            }
            _ => vec![
                "Review operation parameters".to_string(),
                "Check system resources and permissions".to_string(),
                "Consider contacting system administrator".to_string(),
            ],
        }
    }

    /// Find related errors in the audit log
    async fn find_related_errors(&self, error: &RefactoringError, context: &ErrorContext) -> Vec<RefactoringError> {
        // This would typically search the audit log for similar errors
        // For now, return some mock related errors based on the error type
        match error {
            RefactoringError::SafetyViolation { .. } => {
                vec![RefactoringError::ValidationFailure {
                    checks:   vec!["safety".to_string()],
                    failures: vec!["high_risk".to_string()],
                }]
            }
            RefactoringError::SyntaxError { .. } => {
                vec![RefactoringError::SemanticError {
                    message:          "Related semantic issue".to_string(),
                    affected_symbols: vec!["unknown".to_string()],
                }]
            }
            _ => vec![],
        }
    }
}

/// Utility functions for error handling
pub mod error_utils {
    use super::*;

    /// Convert standard error to RefactoringError
    pub fn convert_std_error(error: Box<dyn std::error::Error + Send + Sync>, operation: &str) -> RefactoringError {
        let message = error.to_string();

        if message.contains("permission") || message.contains("access") {
            RefactoringError::AccessDenied {
                resource: operation.to_string(),
                reason:   message,
            }
        } else if message.contains("syntax") || message.contains("parse") {
            RefactoringError::SyntaxError {
                line: 0, // Would need to extract from error
                message,
            }
        } else if message.contains("backup") {
            RefactoringError::BackupFailure { reason: message }
        } else {
            RefactoringError::SemanticError {
                message:          format!("Operation '{}' failed: {}", operation, message),
                affected_symbols: vec![],
            }
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(error: &RefactoringError) -> bool {
        match error {
            RefactoringError::SafetyViolation { .. } => false, // Safety violations should not be auto-recovered
            RefactoringError::BackupFailure { .. } => true,    // Backup failures can often be recovered
            RefactoringError::ValidationFailure { .. } => true, // Validation can be retried
            RefactoringError::AccessDenied { .. } => false,    // Access issues typically not recoverable
            RefactoringError::PerformanceTimeout { .. } => true, // Timeouts can be retried
            _ => true,                                         // Most other errors are recoverable
        }
    }
}
