//! Utility functions for supervisor operations

use std::path::Path;
use std::time::Duration;
use chrono::{DateTime, Utc};
use rust_ai_ide_common::validation::{validate_secure_path, TauriInputSanitizer};

use crate::error::{SupervisorError, SupervisorResult};

/// Security validation for database and checkpoint paths
pub struct PathValidator;

impl PathValidator {
    /// Validate and sanitize a database path
    pub fn validate_database_path(path: &str) -> SupervisorResult<String> {
        validate_secure_path(Path::new(path), "supervisor database")
            .map_err(|e| SupervisorError::security_error("database_path", &e.to_string()))?;

        // Ensure path doesn't contain dangerous characters or patterns
        let sanitized = TauriInputSanitizer::sanitize_path(path)
            .map_err(|e| SupervisorError::security_error("database_path", &format!("Path sanitization failed: {:?}", e)))?;

        // Additional validation: must end with .db extension for safety
        if !sanitized.ends_with(".db") {
            return Err(SupervisorError::security_error("database_path", "Database path must end with .db extension"));
        }

        Ok(sanitized)
    }

    /// Validate and sanitize a checkpoint directory path
    pub fn validate_checkpoint_dir(path: &str) -> SupervisorResult<String> {
        validate_secure_path(Path::new(path), "checkpoint directory")
            .map_err(|e| SupervisorError::security_error("checkpoint_dir", &e.to_string()))?;

        let sanitized = TauriInputSanitizer::sanitize_path(path)
            .map_err(|e| SupervisorError::security_error("checkpoint_dir", &format!("Path sanitization failed: {:?}", e)))?;

        // Ensure it's a directory path (doesn't end with file extensions)
        if sanitized.ends_with(".db") || sanitized.ends_with(".json") || sanitized.contains(".") {
            return Err(SupervisorError::security_error("checkpoint_dir", "Checkpoint directory should not have file extensions"));
        }

        Ok(sanitized)
    }
}

/// Time and duration utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Calculate exponential backoff delay
    pub fn calculate_exponential_backoff(attempt: u32, base_delay: Duration, max_delay: Duration) -> Duration {
        let delay_ms = base_delay.as_millis() as u64;
        let multiplier = 2_u64.pow(attempt.saturating_sub(1).min(20)); // Cap at 2^20 to prevent overflow
        let calculated_delay_ms = delay_ms.saturating_mul(multiplier);

        Duration::from_millis(std::cmp::min(calculated_delay_ms, max_delay.as_millis() as u64))
    }

    /// Check if a timestamp is within a time window
    pub fn is_within_time_window(timestamp: DateTime<Utc>, window: Duration) -> bool {
        let now = Utc::now();
        let diff = if timestamp <= now {
            now.signed_duration_since(timestamp)
        } else {
            timestamp.signed_duration_since(now)
        };

        diff <= chrono::Duration::from_std(window).unwrap_or_else(|_| chrono::Duration::hours(1))
    }

    /// Calculate time until next health check
    pub fn time_until_next_check(last_check: Option<DateTime<Utc>>, interval: Duration) -> Duration {
        if let Some(last) = last_check {
            let next_check_due = last + chrono::Duration::from_std(interval).unwrap_or_else(|_| chrono::Duration::minutes(5));
            let now = Utc::now();

            if next_check_due > now {
                (next_check_due - now).to_std().unwrap_or(Duration::from_secs(0))
            } else {
                Duration::from_secs(0) // Overdue
            }
        } else {
            Duration::from_secs(0) // Never checked, due immediately
        }
    }

    /// Format duration for logging
    pub fn format_duration(duration: Duration) -> String {
        if duration >= Duration::from_secs(3600) {
            format!("{:.2}h", duration.as_secs_f64() / 3600.0)
        } else if duration >= Duration::from_secs(60) {
            format!("{:.2}m", duration.as_secs_f64() / 60.0)
        } else {
            format!("{:.2}s", duration.as_secs_f64())
        }
    }
}

/// Service name validation utilities
pub struct ServiceValidator;

impl ServiceValidator {
    /// Validate service name format
    pub fn validate_service_name(name: &str) -> SupervisorResult<()> {
        if name.is_empty() {
            return Err(SupervisorError::validation_error("service_name", "Service name cannot be empty"));
        }

        if name.len() > 100 {
            return Err(SupervisorError::validation_error("service_name", "Service name too long (max 100 characters)"));
        }

        // Allow only alphanumeric characters, hyphens, and underscores
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(SupervisorError::validation_error("service_name", "Service name contains invalid characters"));
        }

        // Check for reserved names
        let reserved_names = ["system", "internal", "supervisor", "local"];
        if reserved_names.contains(&name.to_lowercase().as_str()) {
            return Err(SupervisorError::validation_error("service_name", "Service name is reserved"));
        }

        Ok(())
    }

    /// Validate command line arguments for security
    pub fn validate_command_args(args: &[String]) -> SupervisorResult<()> {
        for (index, arg) in args.iter().enumerate() {
            if arg.contains("&&") || arg.contains("||") || arg.contains("|") || arg.contains(";") {
                return Err(SupervisorError::security_error(
                    "command_args",
                    &format!("Potentially dangerous command argument at index {}: {}", index, arg)
                ));
            }
        }
        Ok(())
    }
}

/// Resource monitoring utilities
pub struct ResourceMonitor;

impl ResourceMonitor {
    /// Get process memory usage (placeholder - would integrate with sysinfo)
    pub async fn get_process_memory(pid: u32) -> SupervisorResult<u64> {
        // Placeholder implementation - in real usage, would use sysinfo or similar
        log::warn!("Process memory monitoring not implemented (using placeholder)");
        Ok(0) // Placeholder - return 0 to indicate unavailable
    }

    /// Check if system resource limits are exceeded
    pub fn check_resource_limits(current: u64, limit: u64, resource_name: &str) -> SupervisorResult<()> {
        if current >= limit {
            return Err(SupervisorError::resource_limit_exceeded(resource_name, current, limit));
        }
        Ok(())
    }

    /// Calculate resource usage percentage
    pub fn calculate_usage_percentage(used: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (used as f64 / total as f64) * 100.0
        }
    }
}

/// Logging utilities specific to supervisor operations
pub struct SupervisorLogger;

impl SupervisorLogger {
    /// Log service state change
    pub fn log_service_state_change(service_id: &str, old_state: &str, new_state: &str) {
        log::info!(
            "Service '{}' state changed: {} -> {}",
            service_id,
            old_state,
            new_state
        );

        // Could write to audit log here
        // crate::audit::log_service_change(service_id, old_state, new_state);
    }

    /// Log checkpoint operation
    pub fn log_checkpoint_operation(operation: &str, checkpoint_id: &str, success: bool) {
        if success {
            log::info!("Checkpoint operation '{}' succeeded for checkpoint {}", operation, checkpoint_id);
        } else {
            log::error!("Checkpoint operation '{}' failed for checkpoint {}", operation, checkpoint_id);
        }
    }

    /// Log recovery operation
    pub fn log_recovery_operation(operation: &str, target: &str, success: bool) {
        if success {
            log::info!("Recovery operation '{}' succeeded for {}", operation, target);
        } else {
            log::warn!("Recovery operation '{}' failed for {}", operation, target);
        }
    }

    /// Log resource usage alert
    pub fn log_resource_alert(resource: &str, current: u64, limit: u64, severity: &str) {
        log::warn!(
            "Resource '{}' usage alert - {} current: {}, limit: {}",
            resource,
            severity,
            current,
            limit
        );
    }
}

/// Metrics and statistics calculation
pub struct MetricsCalculator;

impl MetricsCalculator {
    /// Calculate service uptime from start time
    pub fn calculate_uptime(start_time: DateTime<Utc>) -> Duration {
        let now = Utc::now();
        let duration = now - start_time;
        Duration::from_secs(duration.num_seconds() as u64)
    }

    /// Calculate success rate from metrics
    pub fn calculate_success_rate(successful: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (successful as f64 / total as f64) * 100.0
        }
    }

    /// Calculate health score based on various metrics
    pub fn calculate_health_score(available_time_percent: f64, error_rate: f64, response_time_ms: f64) -> f64 {
        let availability_weight = 0.5;
        let error_weight = 0.3;
        let performance_weight = 0.2;

        // Invert error rate for health score
        let error_score = (1.0 - error_rate.min(1.0)) * 100.0;

        // Convert response time to score (assuming good response is < 1000ms)
        let performance_score = if response_time_ms <= 1000.0 {
            100.0
        } else {
            (1000.0 / response_time_ms).max(0.0) * 100.0
        };

        availability_weight * available_time_percent +
        error_weight * error_score +
        performance_weight * performance_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_validation() {
        // Valid database path
        let result = PathValidator::validate_database_path("/tmp/test.db");
        assert!(result.is_ok());

        // Invalid path (no .db extension)
        let result = PathValidator::validate_database_path("/tmp/test");
        assert!(result.is_err());

        // Valid checkpoint directory
        let result = PathValidator::validate_checkpoint_dir("/tmp/checkpoints");
        assert!(result.is_ok());

        // Invalid checkpoint directory (has file extension)
        let result = PathValidator::validate_checkpoint_dir("/tmp/checkpoints.db");
        assert!(result.is_err());
    }

    #[test]
    fn test_service_name_validation() {
        // Valid names
        assert!(ServiceValidator::validate_service_name("ai-lsp").is_ok());
        assert!(ServiceValidator::validate_service_name("test_service").is_ok());

        // Invalid names
        assert!(ServiceValidator::validate_service_name("").is_err());
        assert!(ServiceValidator::validate_service_name(&"a".repeat(101)).is_err());
        assert!(ServiceValidator::validate_service_name("system").is_err());
        assert!(ServiceValidator::validate_service_name("test service").is_err()); // spaces
    }

    #[test]
    fn test_exponential_backoff() {
        let base_delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(60);

        // First attempt should return base delay
        let delay = TimeUtils::calculate_exponential_backoff(1, base_delay, max_delay);
        assert_eq!(delay.as_secs(), 1);

        // Second attempt should double
        let delay = TimeUtils::calculate_exponential_backoff(2, base_delay, max_delay);
        assert_eq!(delay.as_secs(), 2);

        // Higher attempts should respect max delay
        let delay = TimeUtils::calculate_exponential_backoff(10, base_delay, max_delay);
        assert!(delay.as_secs() <= 60);
    }

    #[test]
    fn test_success_rate_calculation() {
        assert_eq!(MetricsCalculator::calculate_success_rate(0, 0), 0.0);
        assert_eq!(MetricsCalculator::calculate_success_rate(5, 10), 50.0);
        assert_eq!(MetricsCalculator::calculate_success_rate(10, 10), 100.0);
    }

    #[test]
    fn test_command_args_validation() {
        // Valid args
        assert!(ServiceValidator::validate_command_args(&["--help".to_string(), "--verbose".to_string()]).is_ok());

        // Dangerous args
        assert!(ServiceValidator::validate_command_args(&["rm -rf /tmp && true".to_string()]).is_err());
        assert!(ServiceValidator::validate_command_args(&["echo".to_string(), "test | cat".to_string()]).is_err());
    }
}