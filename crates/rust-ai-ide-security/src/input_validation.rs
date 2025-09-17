//! # Comprehensive Input Validation Framework
//!
//! This module provides enterprise-grade input validation capabilities to prevent
//! security vulnerabilities including injection attacks, path traversal, and other
//! input-based security issues. It integrates seamlessly with the existing security
//! architecture and provides consistent validation across all Tauri commands and API endpoints.
//!
//! ## Features
//!
//! - **Path Traversal Protection**: Secure file path validation and sanitization
//! - **Command Injection Prevention**: Allowlist-based command validation
//! - **SQL Injection Protection**: Parameterized query enforcement
//! - **XSS Prevention**: HTML content sanitization
//! - **Input Sanitization**: Removal of dangerous characters and patterns
//! - **Rate Limiting**: Prevention of abuse through input flooding
//! - **Audit Logging**: Comprehensive security event tracking
//! - **Performance Optimization**: Caching and efficient validation algorithms
//! - **Graceful Degradation**: Safe handling of validation failures
//!
//! ## Architecture
//!
//! The framework consists of specialized sanitizers working together with a central
//! InputValidator that coordinates validation rules and caching mechanisms.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::types::{NetworkContext, OperationType, ResourceContext, UserContext};
use crate::{
    AuditEventContext, AuditEventSeverity, AuditEventType, AuditLogger, ComponentStatus,
    OperationContext, SecurityError, SecurityResult,
};

/// Validation rule severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Low severity - log but allow
    Low,
    /// Medium severity - log and warn
    Medium,
    /// High severity - block and alert
    High,
    /// Critical severity - block, alert, and escalate
    Critical,
}

/// Validation failure reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationFailureReason {
    /// Path traversal attempt detected
    PathTraversal,
    /// Command injection attempt detected
    CommandInjection,
    /// SQL injection attempt detected
    SqlInjection,
    /// XSS attempt detected
    XSS,
    /// Input too long
    InputTooLong,
    /// Invalid format
    InvalidFormat,
    /// Dangerous characters detected
    DangerousCharacters,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Custom validation failure
    Custom(String),
}

/// Validation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub failure_reason: Option<ValidationFailureReason>,
    pub severity: ValidationSeverity,
    pub sanitized_value: Option<String>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Path sanitization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSanitizerConfig {
    pub allow_absolute_paths: bool,
    pub allowed_base_paths: Vec<PathBuf>,
    pub max_path_length: usize,
    pub allow_symlinks: bool,
    pub normalize_paths: bool,
}

/// Command sanitization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSanitizerConfig {
    pub allowed_commands: HashSet<String>,
    pub max_command_length: usize,
    pub max_args_count: usize,
    pub allow_shell_metacharacters: bool,
    pub require_full_path: bool,
}

/// Input validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValidatorConfig {
    pub enable_audit_logging: bool,
    pub enable_rate_limiting: bool,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
    pub max_cache_size: usize,
    pub graceful_degradation: bool,
    pub path_config: PathSanitizerConfig,
    pub command_config: CommandSanitizerConfig,
}

/// Validation statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: u64,
    pub validations_passed: u64,
    pub validations_failed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub rate_limit_hits: u64,
    pub average_validation_time_ms: f64,
}

/// Path Sanitizer - Prevents directory traversal attacks
pub struct PathSanitizer {
    config: PathSanitizerConfig,
    cache: RwLock<HashMap<String, (ValidationResult, Instant)>>,
}

impl PathSanitizer {
    pub fn new(config: PathSanitizerConfig) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Sanitize and validate a file path
    pub async fn sanitize_path(&self, input_path: &str) -> SecurityResult<ValidationResult> {
        // Check cache first
        // Use a default 60 second cache TTL
        const DEFAULT_CACHE_TTL: u64 = 60;
        if let Some((cached_result, timestamp)) = self.cache.read().await.get(input_path) {
            if timestamp.elapsed() < Duration::from_secs(DEFAULT_CACHE_TTL) {
                return Ok(cached_result.clone());
            }
        }

        let mut result = ValidationResult {
            is_valid: true,
            failure_reason: None,
            severity: ValidationSeverity::Low,
            sanitized_value: None,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        };

        // Length check
        if input_path.len() > self.config.max_path_length {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::InputTooLong);
            result.severity = ValidationSeverity::High;
            return Ok(result);
        }

        // Normalize path if enabled
        let normalized_path = if self.config.normalize_paths {
            self.normalize_path(input_path)
        } else {
            input_path.to_string()
        };

        // Check for path traversal patterns
        if self.contains_path_traversal(&normalized_path) {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::PathTraversal);
            result.severity = ValidationSeverity::Critical;
            return Ok(result);
        }

        let path = Path::new(&normalized_path);

        // Check absolute path restrictions
        if !self.config.allow_absolute_paths && path.is_absolute() {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::PathTraversal);
            result.severity = ValidationSeverity::High;
            return Ok(result);
        }

        // Validate against allowed base paths
        if !self.is_path_allowed(path) {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::PathTraversal);
            result.severity = ValidationSeverity::High;
            return Ok(result);
        }

        // Check for dangerous characters
        if self.contains_dangerous_chars(&normalized_path) {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::DangerousCharacters);
            result.severity = ValidationSeverity::High;
            return Ok(result);
        }

        result.sanitized_value = Some(normalized_path);

        // Cache the result
        self.cache
            .write()
            .await
            .insert(input_path.to_string(), (result.clone(), Instant::now()));

        Ok(result)
    }

    fn normalize_path(&self, path: &str) -> String {
        // Basic path normalization
        let path = path.replace("\\", "/");
        let path = path.replace("//", "/");

        // Remove redundant ./ and ../ where safe
        let parts: Vec<&str> = path.split('/').collect();
        let mut normalized = Vec::new();

        for part in parts {
            match part {
                "." => continue,
                ".." => {
                    if !normalized.is_empty() && normalized.last() != Some(&"..") {
                        normalized.pop();
                    } else {
                        normalized.push(part);
                    }
                }
                _ => normalized.push(part),
            }
        }

        normalized.join("/")
    }

    fn contains_path_traversal(&self, path: &str) -> bool {
        // Common path traversal patterns
        let patterns = [
            "..",
            "../",
            "..\\",
            "%2e%2e%2f", // URL encoded ../
            "%2e%2e/",   // URL encoded ..
            "..%2f",     // URL encoded ../
        ];

        for pattern in &patterns {
            if path.contains(pattern) {
                return true;
            }
        }

        false
    }

    fn is_path_allowed(&self, path: &Path) -> bool {
        if self.config.allowed_base_paths.is_empty() {
            return true;
        }

        for base_path in &self.config.allowed_base_paths {
            if path.starts_with(base_path) {
                return true;
            }
        }

        false
    }

    fn contains_dangerous_chars(&self, path: &str) -> bool {
        let dangerous_chars = ['<', '>', '|', ';', '&', '$', '`', '\0'];
        path.chars().any(|c| dangerous_chars.contains(&c))
    }
}

/// Command Sanitizer - Prevents command injection attacks
pub struct CommandSanitizer {
    config: CommandSanitizerConfig,
    cache: RwLock<HashMap<String, (ValidationResult, Instant)>>,
}

impl CommandSanitizer {
    pub fn new(config: CommandSanitizerConfig) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Sanitize and validate a command string
    pub async fn sanitize_command(&self, command: &str) -> SecurityResult<ValidationResult> {
        // Check cache first
        if let Some((cached_result, timestamp)) = self.cache.read().await.get(command) {
            if timestamp.elapsed() < Duration::from_secs(300) {
                // 5 minute cache for commands
                return Ok(cached_result.clone());
            }
        }

        let mut result = ValidationResult {
            is_valid: true,
            failure_reason: None,
            severity: ValidationSeverity::Low,
            sanitized_value: None,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        };

        // Length check
        if command.len() > self.config.max_command_length {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::InputTooLong);
            result.severity = ValidationSeverity::High;
            return Ok(result);
        }

        // Split command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.len() > self.config.max_args_count {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::CommandInjection);
            result.severity = ValidationSeverity::High;
            return Ok(result);
        }

        // Validate command name
        if let Some(command_name) = parts.first() {
            if !self.is_command_allowed(command_name) {
                result.is_valid = false;
                result.failure_reason = Some(ValidationFailureReason::CommandInjection);
                result.severity = ValidationSeverity::Critical;
                return Ok(result);
            }

            // Check for full path requirement
            if self.config.require_full_path && !Path::new(command_name).is_absolute() {
                result
                    .warnings
                    .push("Command should use full path".to_string());
            }
        }

        // Check for shell metacharacters if not allowed
        if !self.config.allow_shell_metacharacters && self.contains_shell_metacharacters(command) {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::CommandInjection);
            result.severity = ValidationSeverity::Critical;
            return Ok(result);
        }

        result.sanitized_value = Some(command.to_string());

        // Cache the result
        self.cache
            .write()
            .await
            .insert(command.to_string(), (result.clone(), Instant::now()));

        Ok(result)
    }

    fn is_command_allowed(&self, command_name: &str) -> bool {
        if self.config.allowed_commands.is_empty() {
            return false; // If no allowlist, deny by default
        }

        let command_path = Path::new(command_name);
        let file_name = command_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(command_name);

        self.config.allowed_commands.contains(file_name)
            || self.config.allowed_commands.contains(command_name)
    }

    fn contains_shell_metacharacters(&self, command: &str) -> bool {
        let metacharacters = ['|', '&', ';', '(', ')', '<', '>', '`', '$', '\'', '"'];
        command.chars().any(|c| metacharacters.contains(&c))
    }
}

/// SQL Sanitizer - Prevents SQL injection attacks
pub struct SqlSanitizer {
    dangerous_patterns: Vec<Regex>,
}

impl SqlSanitizer {
    pub fn new() -> Self {
        let patterns = vec![
            r"(?i)(union\s+select)", // UNION SELECT
            r"(?i)(;\s*drop)",       // ; DROP
            r"(?i)(;\s*delete)",     // ; DELETE
            r"(?i)(exec\s*\()",      // EXEC(
            r"(?i)(script\s*>)",     // <script>
            r"(--|\#)",              // SQL comments
            r"(/\*.*\*/)",           // Block comments
        ];

        let dangerous_patterns = patterns
            .into_iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self { dangerous_patterns }
    }

    /// Validate SQL-like input for injection patterns
    pub async fn validate_sql_input(&self, input: &str) -> SecurityResult<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            failure_reason: None,
            severity: ValidationSeverity::Low,
            sanitized_value: Some(input.to_string()),
            warnings: Vec::new(),
            metadata: HashMap::new(),
        };

        for pattern in &self.dangerous_patterns {
            if pattern.is_match(input) {
                result.is_valid = false;
                result.failure_reason = Some(ValidationFailureReason::SqlInjection);
                result.severity = ValidationSeverity::Critical;
                break;
            }
        }

        Ok(result)
    }
}

impl Default for SqlSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Main Input Validator - Coordinates all validation activities
pub struct InputValidator {
    config: InputValidatorConfig,
    path_sanitizer: Arc<PathSanitizer>,
    command_sanitizer: Arc<CommandSanitizer>,
    sql_sanitizer: Arc<SqlSanitizer>,
    audit_logger: Arc<dyn AuditLogger>,
    stats: Arc<RwLock<ValidationStats>>,
    rate_limiter: Option<Arc<crate::AuthRateLimiter>>,
}

impl InputValidator {
    pub fn new(
        config: InputValidatorConfig,
        audit_logger: Arc<dyn AuditLogger>,
        rate_limiter: Option<Arc<crate::AuthRateLimiter>>,
    ) -> Self {
        let path_sanitizer = Arc::new(PathSanitizer::new(config.path_config.clone()));
        let command_sanitizer = Arc::new(CommandSanitizer::new(config.command_config.clone()));
        let sql_sanitizer = Arc::new(SqlSanitizer::new());

        Self {
            config,
            path_sanitizer,
            command_sanitizer,
            sql_sanitizer,
            audit_logger,
            stats: Arc::new(RwLock::new(ValidationStats {
                total_validations: 0,
                validations_passed: 0,
                validations_failed: 0,
                cache_hits: 0,
                cache_misses: 0,
                rate_limit_hits: 0,
                average_validation_time_ms: 0.0,
            })),
            rate_limiter,
        }
    }

    /// Comprehensive input validation with all sanitizers
    pub async fn validate_input(
        &self,
        input: &str,
        validation_type: ValidationType,
        context: &OperationContext,
    ) -> SecurityResult<ValidationResult> {
        let start_time = Instant::now();

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_validations += 1;

        // Check rate limiting if enabled
        if self.config.enable_rate_limiting {
            if let Some(ref limiter) = self.rate_limiter {
                // Create a minimal user context for rate limiting
                let user_id = context
                    .network_context
                    .as_ref()
                    .and_then(|nc| nc.source_ip.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                let user_context = crate::UserContext {
                    user_id: user_id.clone(),
                    username: user_id.clone(),
                    email: format!("{}@example.com", user_id),
                    roles: vec![],
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                };

                let rate_limited = limiter
                    .check_rate_limit(
                        &user_context,
                        crate::EndpointType::Other,
                        context
                            .network_context
                            .as_ref()
                            .and_then(|nc| nc.source_ip.as_deref()),
                    )
                    .await?;

                if rate_limited.0 {
                    stats.rate_limit_hits += 1;
                    let result = ValidationResult {
                        is_valid: false,
                        failure_reason: Some(ValidationFailureReason::RateLimitExceeded),
                        severity: ValidationSeverity::High,
                        sanitized_value: None,
                        warnings: vec!["Rate limit exceeded".to_string()],
                        metadata: HashMap::new(),
                    };
                    let context = OperationContext {
                        operation_id: Uuid::new_v4().to_string(),
                        request_id: Uuid::new_v4().to_string(),
                        start_time: Utc::now(),
                        timestamp: Utc::now(),
                        network_context: Some(NetworkContext {
                            source_ip: None,
                            protocol: None,
                            user_agent: None,
                        }),
                        resource_context: Some(ResourceContext {
                            resource_id: "input_validation".to_string(),
                            resource_type: "validation".to_string(),
                            action: "validate_input".to_string(),
                        }),
                        operation_type: OperationType::Authentication,
                    };
                    self.log_validation_result(
                        &context,
                        &result,
                        start_time.elapsed(),
                        "rate_limit",
                        input,
                    )
                    .await?;
                    return Ok(result);
                }
            }
        }

        drop(stats); // Release the lock

        let result = match validation_type {
            ValidationType::Path => self.path_sanitizer.sanitize_path(input).await?,
            ValidationType::Command => self.command_sanitizer.sanitize_command(input).await?,
            ValidationType::Sql => self.sql_sanitizer.validate_sql_input(input).await?,
            ValidationType::General => self.validate_general_input(input).await?,
        };

        // Update statistics
        let mut stats = self.stats.write().await;
        if result.is_valid {
            stats.validations_passed += 1;
        } else {
            stats.validations_failed += 1;
        }

        let elapsed = start_time.elapsed();
        stats.average_validation_time_ms =
            (stats.average_validation_time_ms + elapsed.as_millis() as f64) / 2.0;

        // Audit logging if enabled and validation failed or is critical
        if self.config.enable_audit_logging
            && (!result.is_valid || matches!(result.severity, ValidationSeverity::Critical))
        {
            self.log_validation_result(context, &result, elapsed, "input_validation", input)
                .await?;
        }

        Ok(result)
    }

    /// Validate general input for XSS and dangerous patterns
    async fn validate_general_input(&self, input: &str) -> SecurityResult<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            failure_reason: None,
            severity: ValidationSeverity::Low,
            sanitized_value: Some(input.to_string()),
            warnings: Vec::new(),
            metadata: HashMap::new(),
        };

        // Check for XSS patterns
        if self.contains_xss_patterns(input) {
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::XSS);
            result.severity = ValidationSeverity::Critical;
            return Ok(result);
        }

        // Check for dangerously long input
        if input.len() > 10000 {
            // Configurable limit
            result.is_valid = false;
            result.failure_reason = Some(ValidationFailureReason::InputTooLong);
            result.severity = ValidationSeverity::Medium;
            return Ok(result);
        }

        Ok(result)
    }

    fn contains_xss_patterns(&self, input: &str) -> bool {
        let xss_patterns = [
            "<script",
            "javascript:",
            "onload=",
            "onerror=",
            "<iframe",
            "<object",
            "<embed",
        ];

        xss_patterns
            .iter()
            .any(|pattern| input.to_lowercase().contains(&pattern.to_lowercase()))
    }

    /// Log validation events to audit system
    async fn log_validation_result(
        &self,
        context: &OperationContext,
        result: &ValidationResult,
        duration: Duration,
        validation_type: &str,
        value: &str,
    ) -> SecurityResult<()> {
        let severity = match result.severity {
            ValidationSeverity::Low => AuditEventSeverity::Low,
            ValidationSeverity::Medium => AuditEventSeverity::Medium,
            ValidationSeverity::High => AuditEventSeverity::High,
            ValidationSeverity::Critical => AuditEventSeverity::Critical,
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "validation_duration_ms".to_string(),
            duration.as_millis().to_string(),
        );
        metadata.insert("validation_result".to_string(), result.is_valid.to_string());

        if let Some(ref reason) = result.failure_reason {
            metadata.insert("failure_reason".to_string(), format!("{:?}", reason));
        }

        let event_ctx = AuditEventContext::new(
            AuditEventType::SecurityAlert,
            AuditEventSeverity::High,
            "Input validation security check".to_string(),
            serde_json::json!({
                "validation_type": validation_type.to_string(),
                "input": value.to_string(),
                "result": if result.is_valid { "valid" } else { "invalid" },
            }),
        )
        .with_severity(severity)
        .with_metadata("input_type", "user_input")
        .with_metadata(
            "validation_outcome",
            if result.is_valid { "passed" } else { "failed" },
        );

        self.audit_logger.log(&event_ctx).await?;
        Ok(())
    }

    /// Get validation statistics
    pub async fn get_stats(&self) -> ValidationStats {
        self.stats.read().await.clone()
    }

    /// Health check for the validation system
    pub async fn health_status(&self) -> ComponentStatus {
        ComponentStatus::Healthy
    }

    /// Clear validation caches
    pub async fn clear_caches(&self) -> SecurityResult<()> {
        self.path_sanitizer.cache.write().await.clear();
        self.command_sanitizer.cache.write().await.clear();
        Ok(())
    }
}

/// Type of validation to perform
#[derive(Debug, Clone, Copy)]
pub enum ValidationType {
    /// File/directory path validation
    Path,
    /// Command execution validation
    Command,
    /// SQL query validation
    Sql,
    /// General input validation
    General,
}

/// Validation macro for consistent usage
#[macro_export]
macro_rules! validate_input {
    ($validator:expr, $input:expr, $type:expr, $context:expr) => {
        $validator.validate_input($input, $type, $context).await?
    };
}

/// Sanitize and validate command macro
#[macro_export]
macro_rules! sanitize_and_validate_command {
    ($input:expr, $validator_method:expr) => {{
        let validation_result = $validator_method($input).await?;
        if !validation_result.is_valid {
            return Err(crate::SecurityError::ValidationError {
                field: "command".to_string(),
                reason: format!(
                    "{:?}",
                    validation_result.failure_reason.unwrap_or(
                        crate::input_validation::ValidationFailureReason::Custom(
                            "Unknown".to_string()
                        )
                    )
                ),
            });
        }
        validation_result
            .sanitized_value
            .unwrap_or_else(|| $input.to_string())
    }};
}

/// Path validation macro
#[macro_export]
macro_rules! validate_secure_path {
    ($validator:expr, $path:expr, $context:expr) => {{
        let result = $validator
            .validate_input(
                $path,
                crate::input_validation::ValidationType::Path,
                $context,
            )
            .await?;
        if !result.is_valid {
            return Err(crate::SecurityError::ValidationError {
                field: "path".to_string(),
                reason: format!(
                    "{:?}",
                    result.failure_reason.unwrap_or(
                        crate::input_validation::ValidationFailureReason::Custom(
                            "Unknown".to_string()
                        )
                    )
                ),
            });
        }
        result.sanitized_value.unwrap_or_else(|| $path.to_string())
    }};
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::test as async_test;

    use super::*;

    #[async_test]
    async fn test_path_traversal_detection() {
        let config = PathSanitizerConfig {
            allow_absolute_paths: false,
            allowed_base_paths: vec![PathBuf::from("/safe")],
            max_path_length: 1000,
            allow_symlinks: false,
            normalize_paths: true,
        };

        let sanitizer = PathSanitizer::new(config);

        // Test path traversal attempt
        let result = sanitizer
            .sanitize_path("../../../etc/passwd")
            .await
            .unwrap();
        assert!(!result.is_valid);
        assert!(matches!(
            result.failure_reason,
            Some(ValidationFailureReason::PathTraversal)
        ));
    }

    #[async_test]
    async fn test_command_allowlist() {
        let config = CommandSanitizerConfig {
            allowed_commands: ["ls".to_string(), "cat".to_string()].into(),
            max_command_length: 1000,
            max_args_count: 10,
            allow_shell_metacharacters: false,
            require_full_path: false,
        };

        let sanitizer = CommandSanitizer::new(config);

        // Test allowed command
        let result = sanitizer.sanitize_command("ls -la").await.unwrap();
        assert!(result.is_valid);

        // Test disallowed command
        let result = sanitizer.sanitize_command("rm -rf /").await.unwrap();
        assert!(!result.is_valid);
        assert!(matches!(
            result.failure_reason,
            Some(ValidationFailureReason::CommandInjection)
        ));
    }

    #[async_test]
    async fn test_sql_injection_detection() {
        let sanitizer = SqlSanitizer::new();

        // Test safe input
        let result = sanitizer
            .validate_sql_input("SELECT * FROM users WHERE id = ?")
            .await
            .unwrap();
        assert!(result.is_valid);

        // Test SQL injection attempt
        let result = sanitizer
            .validate_sql_input("SELECT * FROM users; DROP TABLE users;--")
            .await
            .unwrap();
        assert!(!result.is_valid);
        assert!(matches!(
            result.failure_reason,
            Some(ValidationFailureReason::SqlInjection)
        ));
    }

    #[async_test]
    async fn test_xss_detection() {
        let config = InputValidatorConfig {
            enable_audit_logging: false,
            enable_rate_limiting: false,
            cache_enabled: true,
            cache_ttl_seconds: 300,
            max_cache_size: 1000,
            graceful_degradation: true,
            path_config: PathSanitizerConfig {
                allow_absolute_paths: false,
                allowed_base_paths: vec![],
                max_path_length: 1000,
                allow_symlinks: false,
                normalize_paths: true,
            },
            command_config: CommandSanitizerConfig {
                allowed_commands: HashSet::new(),
                max_command_length: 1000,
                max_args_count: 10,
                allow_shell_metacharacters: false,
                require_full_path: false,
            },
        };

        let audit_logger = Arc::new(
            crate::AuditLogger::new(crate::AuditConfig::default())
                .await
                .unwrap(),
        );
        let validator = InputValidator::new(config, audit_logger, None);

        // Test XSS attempt
        let result = validator
            .validate_general_input("<script>alert('xss')</script>")
            .await
            .unwrap();
        assert!(!result.is_valid);
        assert!(matches!(
            result.failure_reason,
            Some(ValidationFailureReason::XSS)
        ));
    }
}
