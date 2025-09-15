//! Security validation for AI code generation

use std::collections::HashSet;

use regex::Regex;

use crate::error::{CodegenError, Result};
use crate::types::SecurityLevel;

/// Security validator for code generation operations
pub struct SecurityValidator {
    level:              SecurityLevel,
    dangerous_patterns: HashSet<String>,
    input_sanitizer:    rust_ai_ide_common::validation::TauriInputSanitizer,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new(level: SecurityLevel) -> Self {
        let dangerous_patterns = Self::load_dangerous_patterns();
        let input_sanitizer = rust_ai_ide_common::validation::TauriInputSanitizer::new();

        Self {
            level,
            dangerous_patterns,
            input_sanitizer,
        }
    }

    /// Validate input specifications for security issues
    pub fn validate_input(&self, input: &str) -> Result<()> {
        // Sanitize input first
        let sanitized = self
            .input_sanitizer
            .sanitize(input)
            .map_err(|e| CodegenError::SecurityError(format!("Input sanitization failed: {}", e)))?;

        // Check for dangerous patterns
        self.check_dangerous_patterns(&sanitized)?;

        // Additional validation based on security level
        match self.level {
            SecurityLevel::High | SecurityLevel::Strict => {
                self.validate_high_security(&sanitized)?;
            }
            SecurityLevel::Medium => {
                self.validate_medium_security(&sanitized)?;
            }
            SecurityLevel::Low => {
                // Minimal validation for low security
            }
        }

        // Audit log the validation
        self.audit_log(
            "input_validation",
            &format!("Input length: {}", input.len()),
        );

        Ok(())
    }

    /// Validate generated code for security issues
    pub fn validate_generated_code(&self, code: &crate::types::GeneratedCode) -> Result<()> {
        // Check generated code for dangerous patterns
        self.check_dangerous_patterns(&code.content)?;

        // Language-specific security validation
        match code.language {
            crate::types::TargetLanguage::Rust => {
                self.validate_rust_security(&code.content)?;
            }
            crate::types::TargetLanguage::Python => {
                self.validate_python_security(&code.content)?;
            }
            crate::types::TargetLanguage::JavaScript => {
                self.validate_javascript_security(&code.content)?;
            }
            _ => {
                // Generic validation for other languages
                self.validate_generic_security(&code.content)?;
            }
        }

        // Audit log the validation
        self.audit_log(
            "code_validation",
            &format!(
                "Language: {}, Length: {}",
                code.language,
                code.content.len()
            ),
        );

        Ok(())
    }

    /// Check for dangerous patterns in content
    fn check_dangerous_patterns(&self, content: &str) -> Result<()> {
        for pattern in &self.dangerous_patterns {
            if content.contains(pattern) {
                return Err(CodegenError::SecurityError(format!(
                    "Dangerous pattern detected: {}",
                    pattern
                )));
            }
        }

        // Check for SQL injection patterns
        let sql_patterns = [
            r"DROP\s+TABLE",
            r"DELETE\s+FROM",
            r"UPDATE\s+.*SET",
            r"INSERT\s+INTO",
            r"SELECT\s+.*FROM",
        ];

        for pattern in &sql_patterns {
            if let Ok(regex) = Regex::new(&format!(r"(?i){}", pattern)) {
                if regex.is_match(content) {
                    return Err(CodegenError::SecurityError(format!(
                        "Potential SQL injection pattern detected: {}",
                        pattern
                    )));
                }
            }
        }

        // Check for command injection patterns
        let cmd_patterns = [
            r"system\s*\(",
            r"exec\s*\(",
            r"eval\s*\(",
            r"os\.system",
            r"subprocess\.",
        ];

        for pattern in &cmd_patterns {
            if let Ok(regex) = Regex::new(&format!(r"(?i){}", pattern)) {
                if regex.is_match(content) {
                    return Err(CodegenError::SecurityError(format!(
                        "Potential command injection pattern detected: {}",
                        pattern
                    )));
                }
            }
        }

        Ok(())
    }

    /// High security level validation
    fn validate_high_security(&self, input: &str) -> Result<()> {
        // Check input length limits
        if input.len() > 10000 {
            return Err(CodegenError::SecurityError(
                "Input exceeds maximum length limit for high security".to_string(),
            ));
        }

        // Check for suspicious keywords
        let suspicious_keywords = [
            "password",
            "secret",
            "key",
            "token",
            "credential",
            "admin",
            "root",
            "sudo",
            "exec",
            "eval",
        ];

        let input_lower = input.to_lowercase();
        for keyword in &suspicious_keywords {
            if input_lower.contains(keyword) {
                return Err(CodegenError::SecurityError(format!(
                    "Suspicious keyword detected: {}",
                    keyword
                )));
            }
        }

        Ok(())
    }

    /// Medium security level validation
    fn validate_medium_security(&self, input: &str) -> Result<()> {
        // Basic length check
        if input.len() > 50000 {
            return Err(CodegenError::SecurityError(
                "Input exceeds maximum length limit".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate Rust-specific security issues
    fn validate_rust_security(&self, code: &str) -> Result<()> {
        // Check for unsafe code usage
        if code.contains("unsafe") && self.level == SecurityLevel::Strict {
            return Err(CodegenError::SecurityError(
                "Unsafe code usage not allowed in strict security mode".to_string(),
            ));
        }

        // Check for dangerous macro usage
        let dangerous_macros = ["include_str!", "include_bytes!", "env!"];
        for macro_name in &dangerous_macros {
            if code.contains(macro_name) {
                return Err(CodegenError::SecurityError(format!(
                    "Dangerous macro usage detected: {}",
                    macro_name
                )));
            }
        }

        Ok(())
    }

    /// Validate Python-specific security issues
    fn validate_python_security(&self, code: &str) -> Result<()> {
        // Check for dangerous imports
        let dangerous_imports = ["os.system", "subprocess", "pickle", "eval", "exec"];
        for import_name in &dangerous_imports {
            if code.contains(import_name) {
                return Err(CodegenError::SecurityError(format!(
                    "Dangerous import detected: {}",
                    import_name
                )));
            }
        }

        Ok(())
    }

    /// Validate JavaScript-specific security issues
    fn validate_javascript_security(&self, code: &str) -> Result<()> {
        // Check for dangerous functions
        let dangerous_functions = ["eval(", "Function(", "setTimeout(", "setInterval("];
        for func_name in &dangerous_functions {
            if code.contains(func_name) {
                return Err(CodegenError::SecurityError(format!(
                    "Dangerous function usage detected: {}",
                    func_name
                )));
            }
        }

        // Check for innerHTML usage
        if code.contains("innerHTML") {
            return Err(CodegenError::SecurityError(
                "innerHTML usage detected - potential XSS vulnerability".to_string(),
            ));
        }

        Ok(())
    }

    /// Generic security validation for other languages
    fn validate_generic_security(&self, code: &str) -> Result<()> {
        // Basic checks for all languages
        let dangerous_terms = ["system", "exec", "eval", "shell", "cmd"];
        for term in &dangerous_terms {
            if code.to_lowercase().contains(term) {
                return Err(CodegenError::SecurityError(format!(
                    "Potentially dangerous term detected: {}",
                    term
                )));
            }
        }

        Ok(())
    }

    /// Load dangerous patterns from configuration
    fn load_dangerous_patterns() -> HashSet<String> {
        let mut patterns = HashSet::new();

        // Common dangerous patterns
        let common_patterns = [
            "rm -rf",
            "format c:",
            "del /f",
            "sudo",
            "chmod 777",
            "wget",
            "curl",
            "nc ",
            "netcat",
            "bash -i",
            "python -c",
            "node -e",
        ];

        for pattern in &common_patterns {
            patterns.insert(pattern.to_string());
        }

        patterns
    }

    /// Audit log security events
    fn audit_log(&self, event_type: &str, details: &str) {
        // Placeholder audit logging - in production this would use rust_ai_ide_security::audit_logger()
        log::info!("Security audit: {} - {}", event_type, details);
    }
}

/// Input sanitizer for user inputs
pub struct InputSanitizer {
    max_length:    usize,
    allowed_chars: Regex,
}

impl InputSanitizer {
    /// Create a new input sanitizer
    pub fn new() -> Self {
        Self {
            max_length:    10000,
            allowed_chars: Regex::new(r"^[a-zA-Z0-9\s\p{P}\p{S}]*$").unwrap(),
        }
    }

    /// Sanitize user input
    pub fn sanitize(&self, input: &str) -> Result<String> {
        // Check length
        if input.len() > self.max_length {
            return Err(CodegenError::ValidationError(format!(
                "Input exceeds maximum length of {}",
                self.max_length
            )));
        }

        // Remove potentially dangerous characters
        let sanitized = input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || self.is_safe_symbol(*c))
            .collect::<String>();

        // Additional validation
        if !self.allowed_chars.is_match(&sanitized) {
            return Err(CodegenError::SecurityError(
                "Input contains invalid characters".to_string(),
            ));
        }

        Ok(sanitized)
    }

    /// Check if a symbol is safe to include
    fn is_safe_symbol(&self, c: char) -> bool {
        matches!(
            c,
            '.' | ','
                | '!'
                | '?'
                | ':'
                | ';'
                | '-'
                | '_'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '<'
                | '>'
                | '='
                | '+'
                | '*'
                | '/'
                | '%'
                | '&'
                | '|'
                | '^'
                | '~'
                | '@'
                | '#'
                | '$'
        )
    }
}

/// Path validator for file operations
pub struct PathValidator;

impl PathValidator {
    /// Validate a file path for security
    pub fn validate_path(path: &str) -> Result<()> {
        // Use the common validation function
        rust_ai_ide_common::validation::validate_secure_path(path)
            .map_err(|e| CodegenError::SecurityError(format!("Path validation failed: {}", e)))
    }

    /// Check if path is within allowed directories
    pub fn is_path_allowed(path: &str) -> bool {
        // This would check against a whitelist of allowed directories
        // For now, we'll do basic validation
        !path.contains("..") && !path.starts_with('/')
    }
}

/// Rate limiter for code generation operations
pub struct RateLimiter {
    requests:       std::sync::Mutex<std::collections::HashMap<String, Vec<chrono::DateTime<chrono::Utc>>>>,
    max_requests:   u32,
    window_seconds: i64,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_seconds: i64) -> Self {
        Self {
            requests: std::sync::Mutex::new(std::collections::HashMap::new()),
            max_requests,
            window_seconds,
        }
    }

    /// Check if request is allowed
    pub fn check_rate_limit(&self, key: &str) -> Result<()> {
        let mut requests = self.requests.lock().unwrap();
        let now = chrono::Utc::now();
        let window_start = now - chrono::Duration::seconds(self.window_seconds);

        let user_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the window
        user_requests.retain(|&time| time > window_start);

        if user_requests.len() >= self.max_requests as usize {
            return Err(CodegenError::SecurityError(format!(
                "Rate limit exceeded for key: {}",
                key
            )));
        }

        user_requests.push(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_validator_dangerous_patterns() {
        let validator = SecurityValidator::new(SecurityLevel::High);

        // Test dangerous SQL pattern
        assert!(validator.validate_input("DROP TABLE users").is_err());

        // Test command injection pattern
        assert!(validator.validate_input("system('rm -rf /')").is_err());

        // Test safe input
        assert!(validator
            .validate_input("Create a function to add two numbers")
            .is_ok());
    }

    #[test]
    fn test_input_sanitizer() {
        let sanitizer = InputSanitizer::new();

        // Test normal input
        assert_eq!(sanitizer.sanitize("Hello world").unwrap(), "Hello world");

        // Test with safe symbols
        assert_eq!(
            sanitizer.sanitize("Hello, world!").unwrap(),
            "Hello, world!"
        );

        // Test with dangerous characters
        assert!(sanitizer
            .sanitize("Hello<script>alert('xss')</script>")
            .is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, 60); // 2 requests per minute

        let key = "test_user";

        // First request should succeed
        assert!(limiter.check_rate_limit(key).is_ok());

        // Second request should succeed
        assert!(limiter.check_rate_limit(key).is_ok());

        // Third request should fail
        assert!(limiter.check_rate_limit(key).is_err());
    }
}
