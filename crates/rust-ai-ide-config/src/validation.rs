//! Configuration validation and error handling
//!
//! Provides comprehensive validation with detailed error reporting,
//! constraint checking, and developer-friendly error messages.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Validation result with detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation successful
    pub success:  bool,
    /// Validation errors
    pub errors:   Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Validation metadata
    pub metadata: HashMap<String, String>,
}

impl ValidationResult {
    /// Create successful validation result
    pub fn success() -> Self {
        Self {
            success:  true,
            errors:   Vec::new(),
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create failed validation result
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            success: false,
            errors,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add error to validation result
    pub fn with_error(mut self, error: ValidationError) -> Self {
        self.errors.push(error);
        self.success = false;
        self
    }

    /// Add warning to validation result
    pub fn with_warning(mut self, warning: ValidationWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if validation has any errors or warnings
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }

    /// Get all error messages
    pub fn error_messages(&self) -> Vec<String> {
        self.errors.iter().map(|e| e.message.clone()).collect()
    }

    /// Get all warning messages
    pub fn warning_messages(&self) -> Vec<String> {
        self.warnings.iter().map(|w| w.message.clone()).collect()
    }
}

/// Validation error with detailed context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code:       String,
    /// Human-readable error message
    pub message:    String,
    /// Field that caused the error
    pub field:      Option<String>,
    /// Path to the field in nested structures
    pub field_path: Vec<String>,
    /// Expected value type/format
    pub expected:   Option<String>,
    /// Actual value that was provided
    pub actual:     Option<String>,
    /// Constraint that was violated
    pub constraint: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Severity level
    pub severity:   ValidationSeverity,
}

impl ValidationError {
    /// Create new validation error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code:       code.into(),
            message:    message.into(),
            field:      None,
            field_path: Vec::new(),
            expected:   None,
            actual:     None,
            constraint: None,
            suggestion: None,
            severity:   ValidationSeverity::Error,
        }
    }

    /// Set field name
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Set field path
    pub fn with_field_path(mut self, path: Vec<String>) -> Self {
        self.field_path = path;
        self
    }

    /// Set expected value
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Set actual value
    pub fn with_actual(mut self, actual: impl Into<String>) -> Self {
        self.actual = Some(actual.into());
        self
    }

    /// Set constraint
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraint = Some(constraint.into());
        self
    }

    /// Set suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: ValidationSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// Validation warning (non-blocking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code:       String,
    /// Human-readable warning message
    pub message:    String,
    /// Field related to the warning
    pub field:      Option<String>,
    /// Suggested improvement
    pub suggestion: Option<String>,
}

impl ValidationWarning {
    /// Create new validation warning
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code:       code.into(),
            message:    message.into(),
            field:      None,
            suggestion: None,
        }
    }

    /// Set field name
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Set suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical error that prevents operation
    Critical,
    /// Regular error
    Error,
    /// Warning that might cause issues
    Warning,
    /// Information about potential improvements
    Info,
}

/// Configuration validator
pub struct ValidationEngine {
    /// Validation rules by configuration type
    rules: HashMap<String, Vec<Box<dyn ValidationRule + Send + Sync>>>,
}

impl ValidationEngine {
    /// Create new validation engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Register validation rule for a configuration type
    pub fn register_rule<T>(&mut self, rule: T)
    where
        T: ValidationRule + Send + Sync + 'static,
    {
        let config_type = rule.config_type();
        self.rules
            .entry(config_type)
            .or_insert_with(Vec::new)
            .push(Box::new(rule));
    }

    /// Validate configuration
    pub fn validate<C>(&self, config: &C) -> crate::IDEResult<ValidationResult>
    where
        C: crate::Config + serde::Serialize,
    {
        let config_name = C::FILE_PREFIX.to_string();
        let mut result = ValidationResult::success();

        // Run custom validation from the Config trait
        if let Err(e) = config.validate() {
            result.errors.push(ValidationError::new(
                "CONFIG_VALIDATION_FAILED",
                format!("Configuration validation failed: {}", e),
            ));
        }

        // Run registered validation rules
        if let Some(rules) = self.rules.get(&config_name) {
            for rule in rules {
                let rule_result = rule.validate(config)?;
                result.errors.extend(rule_result.errors);
                result.warnings.extend(rule_result.warnings);
            }
        }

        // Run default validations
        let default_result = self.run_default_validations(config)?;
        result.errors.extend(default_result.errors);
        result.warnings.extend(default_result.warnings);

        result.success = result.errors.is_empty();

        Ok(result)
    }

    /// Run default validation rules
    fn run_default_validations<C>(&self, config: &C) -> crate::IDEResult<ValidationResult>
    where
        C: serde::Serialize,
    {
        let mut result = ValidationResult::success();

        // Validate JSON structure
        let json = serde_json::to_value(config)
            .map_err(|e| crate::RustAIError::Serialization(format!("Invalid JSON structure: {}", e)))?;

        // Check for required fields (basic structure validation)
        if let serde_json::Value::Object(obj) = &json {
            if obj.is_empty() {
                result.warnings.push(
                    ValidationWarning::new("EMPTY_CONFIG", "Configuration appears to be empty")
                        .with_suggestion("Add required configuration fields"),
                );
            }

            // Check for potentially sensitive fields
            let sensitive_keywords = ["password", "secret", "key", "token"];
            for (field_name, _) in obj {
                for keyword in &sensitive_keywords {
                    if field_name.to_lowercase().contains(keyword) {
                        result.warnings.push(
                            ValidationWarning::new(
                                "POTENTIAL_SENSITIVE_FIELD",
                                format!("Field '{}' may contain sensitive information", field_name),
                            )
                            .with_suggestion("Ensure sensitive fields are properly encrypted"),
                        );
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get validation rules for a configuration type
    pub fn get_rules(&self, config_type: &str) -> Vec<String> {
        self.rules
            .get(config_type)
            .map(|rules| rules.iter().map(|r| r.name()).collect())
            .unwrap_or_default()
    }
}

/// Validation rule trait
pub trait ValidationRule {
    /// Get the configuration type this rule applies to
    fn config_type(&self) -> String;

    /// Get rule name
    fn name(&self) -> String;

    /// Validate configuration
    fn validate(&self, config: &dyn std::any::Any) -> crate::IDEResult<ValidationResult>;
}

/// Common validation rule implementations

/// Range validation rule for numeric fields
pub struct RangeValidationRule {
    config_type: String,
    field:       String,
    min:         Option<f64>,
    max:         Option<f64>,
}

impl RangeValidationRule {
    pub fn new(config_type: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            config_type: config_type.into(),
            field:       field.into(),
            min:         None,
            max:         None,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }
}

impl ValidationRule for RangeValidationRule {
    fn config_type(&self) -> String {
        self.config_type.clone()
    }

    fn name(&self) -> String {
        format!("range_validation_{}", self.field)
    }

    fn validate(&self, config: &dyn std::any::Any) -> crate::IDEResult<ValidationResult> {
        // This would deserialize the config and check the field range
        // For now, return success
        Ok(ValidationResult::success())
    }
}

/// Required field validation rule
pub struct RequiredFieldRule {
    config_type: String,
    field:       String,
}

impl RequiredFieldRule {
    pub fn new(config_type: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            config_type: config_type.into(),
            field:       field.into(),
        }
    }
}

impl ValidationRule for RequiredFieldRule {
    fn config_type(&self) -> String {
        self.config_type.clone()
    }

    fn name(&self) -> String {
        format!("required_field_{}", self.field)
    }

    fn validate(&self, config: &dyn std::any::Any) -> crate::IDEResult<ValidationResult> {
        // This would check if the field exists and is not null/empty
        // For now, return success
        Ok(ValidationResult::success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestConfig {
        number: i32,
        text:   String,
    }

    impl crate::Config for TestConfig {
        const FILE_PREFIX: &'static str = "test";
        const DESCRIPTION: &'static str = "Test Configuration";

        fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
            let mut errors = Vec::new();
            if self.number < 0 {
                errors.push("Number must be non-negative".to_string());
            }
            Ok(errors)
        }

        fn default_config() -> Self {
            Self {
                number: 42,
                text:   "default".to_string(),
            }
        }
    }

    #[test]
    fn test_validation_success() {
        let validator = ValidationEngine::new();
        let config = TestConfig {
            number: 10,
            text:   "valid".to_string(),
        };

        let result = validator.validate(&config).unwrap();
        assert!(result.success);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_failure() {
        let validator = ValidationEngine::new();
        let config = TestConfig {
            number: -5,
            text:   "invalid".to_string(),
        };

        let result = validator.validate(&config).unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError::new("TEST_ERROR", "Test validation error")
            .with_field("test_field")
            .with_expected("positive number")
            .with_actual("-5")
            .with_suggestion("Use a positive number");

        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.field.as_ref().unwrap(), "test_field");
        assert_eq!(error.expected.as_ref().unwrap(), "positive number");
        assert_eq!(error.actual.as_ref().unwrap(), "-5");
        assert_eq!(error.suggestion.as_ref().unwrap(), "Use a positive number");
    }

    #[test]
    fn test_validation_result_manipulation() {
        let mut result = ValidationResult::success();
        assert!(result.success);

        result = result.with_error(ValidationError::new("TEST", "error"));
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);

        result = result.with_warning(ValidationWarning::new("TEST", "warning"));
        assert_eq!(result.warnings.len(), 1);

        result = result.with_metadata("test_key", "test_value");
        assert_eq!(result.metadata.get("test_key").unwrap(), "test_value");
    }
}
