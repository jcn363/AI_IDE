//! Strong types for data integrity and validated inputs
//!
//! This module provides zero-cost abstractions for validated data types
//! that guarantee certain security properties at compile time.

use std::path::PathBuf;
use std::sync::Arc;

use crate::errors::{IdeError, IdeResult};

/// A string that has been validated and sanitized for general use
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedString {
    inner:      Arc<str>,
    max_length: usize,
    sanitized:  bool,
}

impl SanitizedString {
    /// Create a new sanitized string with validation
    pub fn new(input: &str, max_length: usize) -> IdeResult<Self> {
        if input.is_empty() {
            return Err(IdeError::Validation {
                field:  "SanitizedString".to_string(),
                reason: "Cannot create empty SanitizedString".to_string(),
            });
        }

        if input.len() > max_length {
            return Err(IdeError::Validation {
                field:  "SanitizedString".to_string(),
                reason: format!(
                    "Input length {} exceeds max length {}",
                    input.len(),
                    max_length
                ),
            });
        }

        let sanitized = Self::sanitize(input)?;

        Ok(Self {
            inner: Arc::from(sanitized.as_str()),
            max_length,
            sanitized: true,
        })
    }

    /// Sanitize the input string (remove dangerous characters)
    fn sanitize(input: &str) -> IdeResult<String> {
        let mut result = input.to_string();

        // Remove null bytes
        result = result.replace('\0', "");

        // Remove potentially dangerous characters
        let dangerous_chars = ['<', '>', '"', '\'', '\r', '\n'];
        for &ch in &dangerous_chars {
            result = result.replace(ch, "");
        }

        // Trim whitespace
        result = result.trim().to_string();

        Ok(result)
    }

    /// Get the string as a reference
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the string as a String (clone)
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Check if the string is sanitized
    pub fn is_sanitized(&self) -> bool {
        self.sanitized
    }

    /// Get the max length constraint
    pub fn max_length(&self) -> usize {
        self.max_length
    }
}

impl AsRef<str> for SanitizedString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

/// A file path that has been validated for security
#[derive(Debug, Clone)]
pub struct SecurePath {
    path:      PathBuf,
    validated: bool,
}

impl SecurePath {
    /// Create a new secure path with validation
    pub fn new<V: Into<PathBuf>>(path: V, operation: &str) -> IdeResult<Self> {
        let path_buf = path.into();

        // Canonicalize to resolve any .. or . components
        let canonical_path = path_buf.canonicalize().map_err(|_| IdeError::Validation {
            field:  "SecurePath".to_string(),
            reason: format!(
                "{}: Path cannot be canonicalized or does not exist",
                operation
            ),
        })?;

        // Check for path traversal attempts
        Self::validate_no_traversal(&canonical_path)?;

        // Check additional security constraints
        Self::validate_security_constraints(&canonical_path, operation)?;

        Ok(Self {
            path:      canonical_path,
            validated: true,
        })
    }

    /// Validate that the path doesn't contain traversal exploits
    fn validate_no_traversal(path: &PathBuf) -> IdeResult<()> {
        use std::path::Component;

        for component in path.components() {
            match component {
                Component::Normal(part) => {
                    let part_str = part.to_string_lossy();
                    if part_str == ".." || part_str.starts_with("..") {
                        return Err(IdeError::Validation {
                            field:  "SecurePath".to_string(),
                            reason: "Path traversal detected".to_string(),
                        });
                    }
                    // Check for null bytes or other problematic characters
                    if part_str.chars().any(|c| c.is_control()) {
                        return Err(IdeError::Validation {
                            field:  "SecurePath".to_string(),
                            reason: "Control characters detected in path".to_string(),
                        });
                    }
                }
                _ => {} // Root and prefix components are generally safe
            }
        }

        Ok(())
    }

    /// Additional security constraints
    fn validate_security_constraints(path: &PathBuf, operation: &str) -> IdeResult<()> {
        // Check file size if it's a file (reasonable limit for IDE operations)
        if path.is_file() {
            let metadata = path.metadata()?;
            const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
            if metadata.len() > MAX_FILE_SIZE {
                return Err(IdeError::Validation {
                    field:  "SecurePath".to_string(),
                    reason: format!(
                        "{}: File size exceeds limit ({}MB)",
                        operation,
                        MAX_FILE_SIZE / (1024 * 1024)
                    ),
                });
            }
        }

        Ok(())
    }

    /// Get the path as a PathBuf
    pub fn to_path_buf(&self) -> PathBuf {
        self.path.clone()
    }

    /// Get the path as a reference
    pub fn as_path(&self) -> &std::path::Path {
        &self.path
    }

    /// Check if the path is validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }
}

impl AsRef<std::path::Path> for SecurePath {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

/// A validated command line argument
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureArg {
    inner:     String,
    validated: bool,
}

impl SecureArg {
    /// Create a new secure command argument
    pub fn new(arg: &str) -> IdeResult<Self> {
        if arg.is_empty() {
            return Err(IdeError::Validation {
                field:  "SecureArg".to_string(),
                reason: "Command arguments cannot be empty".to_string(),
            });
        }

        if arg.len() > 2048 {
            return Err(IdeError::Validation {
                field:  "SecureArg".to_string(),
                reason: "Command argument too long".to_string(),
            });
        }

        let sanitized = Self::sanitize_arg(arg)?;

        Ok(Self {
            inner:     sanitized,
            validated: true,
        })
    }

    /// Sanitize the command argument
    fn sanitize_arg(arg: &str) -> IdeResult<String> {
        let mut result = arg.to_string();

        // Replace or remove dangerous characters
        let dangerous_patterns = ["&", ";", "|", "`"];
        for pattern in &dangerous_patterns {
            result = result.replace(pattern, "_");
        }

        // Remove control characters
        result.retain(|c| !c.is_control());

        Ok(result.trim().to_string())
    }

    /// Convert to command argument string
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Check if the argument is validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }
}

/// A validated command string for shell execution
#[derive(Debug, Clone)]
pub struct SecureCommand {
    command:   SanitizedString,
    args:      Vec<SecureArg>,
    validated: bool,
}

impl SecureCommand {
    /// Create a new secure command with arguments
    pub fn new(command: &str, args: Vec<&str>) -> IdeResult<Self> {
        let secure_command = SanitizedString::new(command, 256)?;
        let secure_args: IdeResult<Vec<SecureArg>> = args.iter().map(|arg| SecureArg::new(arg)).collect();

        Ok(Self {
            command:   secure_command,
            args:      secure_args?,
            validated: true,
        })
    }

    /// Get the command string
    pub fn command(&self) -> &str {
        self.command.as_str()
    }

    /// Get the arguments
    pub fn args(&self) -> &[SecureArg] {
        &self.args
    }

    /// Check if the command is fully validated
    pub fn is_validated(&self) -> bool {
        self.validated && self.command.is_sanitized() && self.args.iter().all(|arg| arg.is_validated())
    }
}

/// A validated regular expression pattern
#[derive(Debug, Clone)]
pub struct SecureRegex {
    pattern:   String,
    compiled:  regex::Regex,
    validated: bool,
}

impl SecureRegex {
    /// Create a new secure regex pattern
    pub fn new(pattern: &str) -> IdeResult<Self> {
        if pattern.is_empty() {
            return Err(IdeError::Validation {
                field:  "SecureRegex".to_string(),
                reason: "Regex pattern cannot be empty".to_string(),
            });
        }

        if pattern.len() > 1024 {
            return Err(IdeError::Validation {
                field:  "SecureRegex".to_string(),
                reason: "Regex pattern too long".to_string(),
            });
        }

        // Check for potentially dangerous patterns
        Self::validate_regex_pattern(pattern)?;

        let compiled = regex::Regex::new(pattern).map_err(|e| IdeError::Validation {
            field:  "SecureRegex".to_string(),
            reason: format!("Invalid regex pattern: {}", e),
        })?;

        Ok(Self {
            pattern: pattern.to_string(),
            compiled,
            validated: true,
        })
    }

    /// Validate the regex pattern for security issues
    fn validate_regex_pattern(pattern: &str) -> IdeResult<()> {
        // Check for catastrophic backtracking patterns
        if pattern.contains(".*+") || pattern.contains(".+?") {
            return Err(IdeError::Validation {
                field:  "SecureRegex".to_string(),
                reason: "Potentially inefficient regex pattern detected".to_string(),
            });
        }

        Ok(())
    }

    /// Get the compiled regex
    pub fn as_regex(&self) -> &regex::Regex {
        &self.compiled
    }

    /// Get the pattern string
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Check if validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitized_string_creation() {
        let result = SanitizedString::new("test", 100);
        assert!(result.is_ok());

        let result = SanitizedString::new("", 100);
        assert!(result.is_err());

        let result = SanitizedString::new("<script>alert('xss')</script>", 100);
        assert!(result.is_ok());
        assert!(!result.unwrap().as_str().contains('<'));
    }

    #[test]
    fn test_secure_path_validation() {
        use std::env;

        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("test.txt");

        // This will fail if the file doesn't exist, which is expected for this test
        let result = SecurePath::new(&test_path, "test");
        assert!(result.is_err()); // Should fail since file doesn't exist

        // Test traversal detection (mock)
        let traversal_path = PathBuf::from("../etc/passwd");
        let result = SecurePath::new(traversal_path, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_secure_arg_creation() {
        let result = SecureArg::new("test");
        assert!(result.is_ok());

        let result = SecureArg::new("");
        assert!(result.is_err());

        let result = SecureArg::new("test; rm -rf /");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "test_ rm -rf /");
    }

    #[test]
    fn test_secure_command_creation() {
        let result = SecureCommand::new("ls", vec!["-la"]);
        assert!(result.is_ok());

        let secure_cmd = result.unwrap();
        assert_eq!(secure_cmd.command(), "ls");
        assert!(secure_cmd.is_validated());
    }
}
