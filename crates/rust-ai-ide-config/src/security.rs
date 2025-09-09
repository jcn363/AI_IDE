//! Security validation and threat prevention for configuration
//!
//! This module provides comprehensive security validation including:
//! - Path traversal prevention
//! - Command injection prevention
//! - Input sanitization
//! - Threat detection and logging

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::SecurityLevel;

/// Security validator for configuration inputs
#[derive(Debug)]
pub struct SecurityValidator {
    /// Security validation level
    level: SecurityLevel,
    /// Path traversal patterns
    path_traversal_patterns: Vec<Regex>,
    /// Command injection patterns
    command_injection_patterns: Vec<Regex>,
    /// Dangerous characters that should be sanitized
    dangerous_chars: Regex,
    /// Maximum input length
    max_input_length: usize,
    /// Maximum path length
    max_path_length: usize,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new(level: SecurityLevel) -> Self {
        let path_traversal_patterns = vec![
            Regex::new(r"\.\./").unwrap(),
            Regex::new(r"\.\.\\").unwrap(),
            Regex::new(r"%2e%2e%2f").unwrap(),
            Regex::new(r"%2e%2e/").unwrap(),
        ];

        let command_injection_patterns = vec![
            Regex::new(r"[;&|`]").unwrap(),
            Regex::new(r"\$\([^)]*\)").unwrap(),
            Regex::new(r"`[^`]*`").unwrap(),
            Regex::new(r"\$\{[^}]*\}").unwrap(),
        ];

        let dangerous_chars = Regex::new(r#"[<>'"&;]"#).unwrap();

        let (max_input_length, max_path_length) = match level {
            SecurityLevel::Basic => (1000, 255),
            SecurityLevel::Standard => (5000, 4096),
            SecurityLevel::High => (10000, 8192),
            SecurityLevel::Paranoid => (5000, 1024),
        };

        Self {
            level,
            path_traversal_patterns,
            command_injection_patterns,
            dangerous_chars,
            max_input_length,
            max_path_length,
        }
    }

    /// Validate and sanitize a string input
    pub fn sanitize_input(&self, input: &str, field_name: &str) -> crate::IDEResult<String> {
        // Length check
        if input.len() > self.max_input_length {
            return Err(self.create_violation(
                SecurityViolation::InputTooLong,
                field_name,
                &format!(
                    "Input length {} exceeds maximum {}",
                    input.len(),
                    self.max_input_length
                ),
            ));
        }

        // Check for dangerous patterns based on security level
        let mut sanitized = input.to_string();

        match self.level {
            SecurityLevel::Basic => {
                // Only basic sanitization
                sanitized = self.dangerous_chars.replace_all(&sanitized, "").to_string();
            }
            SecurityLevel::Standard | SecurityLevel::High => {
                // Standard sanitization + pattern checks
                sanitized = self.dangerous_chars.replace_all(&sanitized, "").to_string();

                for pattern in &self.path_traversal_patterns {
                    if pattern.is_match(&sanitized) {
                        return Err(self.create_violation(
                            SecurityViolation::PathTraversal,
                            field_name,
                            "Path traversal pattern detected",
                        ));
                    }
                }

                for pattern in &self.command_injection_patterns {
                    if pattern.is_match(&sanitized) {
                        return Err(self.create_violation(
                            SecurityViolation::CommandInjection,
                            field_name,
                            "Command injection pattern detected",
                        ));
                    }
                }
            }
            SecurityLevel::Paranoid => {
                // Maximum security - only allow alphanumeric and safe characters
                let safe_pattern = Regex::new(r"[^a-zA-Z0-9@._-]").unwrap();
                sanitized = safe_pattern.replace_all(&sanitized, "").to_string();

                if sanitized != input {
                    return Err(self.create_violation(
                        SecurityViolation::DangerousCharacters,
                        field_name,
                        "Only alphanumeric characters, dots, underscores, and hyphens allowed",
                    ));
                }
            }
        }

        Ok(sanitized)
    }

    /// Validate a file path for security
    pub fn validate_path(
        &self,
        path: &Path,
        base_path: Option<&Path>,
    ) -> crate::IDEResult<PathBuf> {
        let path_str = path.to_string_lossy();

        // Length check
        if path_str.len() > self.max_path_length {
            return Err(self.create_violation(
                SecurityViolation::PathTooLong,
                "path",
                &format!(
                    "Path length {} exceeds maximum {}",
                    path_str.len(),
                    self.max_path_length
                ),
            ));
        }

        // Check for null bytes (common in exploits)
        if path_str.contains('\0') {
            return Err(self.create_violation(
                SecurityViolation::NullByteInjection,
                "path",
                "Null byte detected in path",
            ));
        }

        // Check for path traversal
        for pattern in &self.path_traversal_patterns {
            if pattern.is_match(&path_str) {
                return Err(self.create_violation(
                    SecurityViolation::PathTraversal,
                    "path",
                    "Path traversal pattern detected",
                ));
            }
        }

        // Canonicalize path if possible
        let canonical_path = if path.is_absolute() {
            path.canonicalize().map_err(|e| {
                crate::RustAIError::Path(rust_ai_ide_errors::PathError::new(&format!(
                    "Failed to canonicalize path: {}",
                    e
                )))
            })?
        } else {
            path.to_path_buf()
        };

        // Check against base path if provided (sandboxing)
        if let Some(base) = base_path {
            let base_canonical = base.canonicalize().map_err(|e| {
                crate::RustAIError::Path(rust_ai_ide_errors::PathError::new(&format!(
                    "Failed to canonicalize base path: {}",
                    e
                )))
            })?;

            if !canonical_path.starts_with(base_canonical) {
                return Err(self.create_violation(
                    SecurityViolation::PathOutsideSandbox,
                    "path",
                    "Path resolves outside of allowed sandbox",
                ));
            }
        }

        Ok(canonical_path)
    }

    /// Validate configuration object for security issues
    pub fn validate_config<C>(&self, config: &C) -> crate::IDEResult<()>
    where
        C: serde::Serialize,
    {
        // For now, serialize to JSON and scan for dangerous patterns
        // In a full implementation, this could be more sophisticated
        let json = serde_json::to_string(config)
            .map_err(|e| crate::RustAIError::Serialization(e.to_string()))?;

        let json_lower = json.to_lowercase();

        // Check for suspicious patterns in serialized config
        let suspicious_patterns = [
            "rm ",
            "del ",
            "format ",
            "shutdown",
            "passwd",
            "shadow",
            "/etc/",
            "/proc/",
            "<script",
            "javascript:",
            "onload=",
            "onerror=",
        ];

        for pattern in &suspicious_patterns {
            if json_lower.contains(pattern) {
                return Err(self.create_violation(
                    SecurityViolation::SuspiciousConfig,
                    "config_object",
                    &format!("Suspicious pattern detected: {}", pattern),
                ));
            }
        }

        Ok(())
    }

    /// Create a security violation error
    fn create_violation(
        &self,
        violation: SecurityViolation,
        field: &str,
        message: &str,
    ) -> crate::RustAIError {
        let threat_level = match violation {
            SecurityViolation::PathTraversal
            | SecurityViolation::CommandInjection
            | SecurityViolation::NullByteInjection => ThreatLevel::High,
            SecurityViolation::DangerousCharacters | SecurityViolation::PathOutsideSandbox => {
                ThreatLevel::Medium
            }
            _ => ThreatLevel::Low,
        };

        crate::RustAIError::Validation(format!(
            "Security violation in field '{}': {} (Threat Level: {:?})",
            field, message, threat_level
        ))
    }

    /// Get current security level
    pub fn security_level(&self) -> SecurityLevel {
        self.level
    }
}

/// Types of security violations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecurityViolation {
    /// Path traversal attempt detected
    PathTraversal,
    /// Command injection attempt detected
    CommandInjection,
    /// Dangerous characters detected
    DangerousCharacters,
    /// Input too long
    InputTooLong,
    /// Path too long
    PathTooLong,
    /// Null byte injection detected
    NullByteInjection,
    /// Path resolves outside allowed sandbox
    PathOutsideSandbox,
    /// Suspicious content in configuration
    SuspiciousConfig,
}

/// Threat levels for security violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// Low threat - potential issue but likely benign
    Low,
    /// Medium threat - suspicious activity
    Medium,
    /// High threat - confirmed attack attempt
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_basic_input_sanitization() {
        let validator = SecurityValidator::new(SecurityLevel::Basic);
        let input = "safe<input>";
        let result = validator.sanitize_input(input, "test_field");

        assert!(result.is_ok());
        let sanitized = result.unwrap();
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
    }

    #[test]
    fn test_path_traversal_detection() {
        let validator = SecurityValidator::new(SecurityLevel::High);
        let path = Path::new("../../etc/passwd");

        let result = validator.validate_path(path, None);
        assert!(result.is_err());
        assert!(matches!(result, Err(crate::RustAIError::Validation(_))));
    }

    #[test]
    fn test_sandbox_validation() {
        let validator = SecurityValidator::new(SecurityLevel::High);
        let base_path = Path::new("/safe/base");
        let test_path = Path::new("/safe/base/subdir/file.txt");

        let result = validator.validate_path(test_path, Some(base_path));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sandbox_violation() {
        let validator = SecurityValidator::new(SecurityLevel::High);
        let base_path = Path::new("/safe/base");
        let test_path = Path::new("/unsafe/path/file.txt");

        let result = validator.validate_path(test_path, Some(base_path));
        assert!(result.is_err());
    }

    #[test]
    fn test_paranoid_sanitization() {
        let validator = SecurityValidator::new(SecurityLevel::Paranoid);
        let input = "safe@input.123_456-test";
        let result = validator.sanitize_input(input, "test_field");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), input);

        let dangerous_input = "unsafe<input>";
        let result = validator.sanitize_input(dangerous_input, "test_field");
        assert!(result.is_err());
    }
}
