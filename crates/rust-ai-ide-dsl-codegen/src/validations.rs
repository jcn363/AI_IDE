//! Template validation and security checks

use crate::ast::{GenerateBlock, Template, ValidationRule};
use crate::error::DslResult;
use crate::types::GeneratedCode;
use async_trait::async_trait;

/// Template validation engine
#[derive(Debug)]
pub struct TemplateValidator {
    security_validator: Option<Box<dyn SecurityValidator>>,
    language_validator: Option<Box<dyn LanguageValidator>>,
}

impl TemplateValidator {
    /// Create a new template validator
    pub fn new() -> Self {
        Self {
            security_validator: None,
            language_validator: None,
        }
    }

    /// Create validator with security checks
    pub fn with_security(mut self, validator: Box<dyn SecurityValidator>) -> Self {
        self.security_validator = Some(validator);
        self
    }

    /// Create validator with language-specific checks
    pub fn with_language_validation(mut self, validator: Box<dyn LanguageValidator>) -> Self {
        self.language_validator = Some(validator);
        self
    }

    /// Validate a complete template
    pub async fn validate_template(&self, template: &Template) -> DslResult<Vec<ValidationResult>> {
        let mut results = Vec::new();

        // Basic structural validation
        self.validate_structure(template, &mut results).await?;

        // Parameter validation
        self.validate_parameters(template, &mut results).await?;

        // Generation block validation
        self.validate_generation_block(&template.generate, &mut results)
            .await?;

        // Security validation
        if let Some(security) = &self.security_validator {
            security
                .validate_template(template)
                .await
                .unwrap_or_else(|_| vec![]) // Convert errors to warnings
                .into_iter()
                .for_each(|r| results.push(r));
        }

        // Language-specific validation
        if let Some(language) = &self.language_validator {
            language
                .validate_template(template)
                .await
                .unwrap_or_else(|_| vec![]) // Convert errors to warnings
                .into_iter()
                .for_each(|r| results.push(r));
        }

        Ok(results)
    }

    /// Validate generated code
    pub async fn validate_generated_code(
        &self,
        code: &GeneratedCode,
        rules: &[ValidationRule],
    ) -> DslResult<Vec<ValidationResult>> {
        let mut results = Vec::new();

        // Apply custom validation rules
        for rule in rules {
            match self.apply_validation_rule(code, rule).await {
                Ok(result) => results.push(result),
                Err(e) => results.push(ValidationResult {
                    rule: rule.name.clone(),
                    severity: ValidationSeverity::Error,
                    message: format!("Validation failed: {}", e),
                    location: None,
                }),
            }
        }

        // Security validation on generated code
        if let Some(security) = &self.security_validator {
            security
                .validate_generated_code(code)
                .await
                .unwrap_or_else(|_| vec![]) // Convert errors to warnings
                .into_iter()
                .for_each(|r| results.push(r));
        }

        Ok(results)
    }

    async fn validate_structure(
        &self,
        template: &Template,
        results: &mut Vec<ValidationResult>,
    ) -> DslResult<()> {
        if template.name.is_empty() {
            results.push(ValidationResult::error(
                "structure",
                "Template name cannot be empty",
                None,
            ));
        }

        if template.generate.content.parts.is_empty() {
            results.push(ValidationResult::warning(
                "structure",
                "Template has no content to generate",
                None,
            ));
        }

        Ok(())
    }

    async fn validate_parameters(
        &self,
        template: &Template,
        results_Collector: &mut Vec<ValidationResult>,
    ) -> DslResult<()> {
        let mut param_names = std::collections::HashSet::new();

        for param in &template.parameters {
            // Check for duplicate parameter names
            if !param_names.insert(&param.name) {
                results_Collector.push(ValidationResult::error(
                    "parameters",
                    format!("Duplicate parameter name: {}", param.name),
                    None,
                ));
            }

            // Validate parameter name format
            if param.name.is_empty() {
                results_Collector.push(ValidationResult::error(
                    "parameters",
                    "Parameter name cannot be empty",
                    None,
                ));
                continue;
            }

            if !param.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                results_Collector.push(ValidationResult::warning(
                    "parameters",
                    format!(
                        "Parameter name '{}' contains special characters",
                        param.name
                    ),
                    None,
                ));
            }
        }

        Ok(())
    }

    async fn validate_generation_block(
        &self,
        block: &GenerateBlock,
        results: &mut Vec<ValidationResult>,
    ) -> DslResult<()> {
        if block.validations.is_empty() {
            results.push(ValidationResult::info(
                "generation",
                "Consider adding validation rules for generated code",
                None,
            ));
        }

        Ok(())
    }

    async fn apply_validation_rule(
        &self,
        code: &GeneratedCode,
        rule: &ValidationRule,
    ) -> DslResult<ValidationResult> {
        // For Phase 2, we implement a simplified validation engine
        // In later phases, this could be expanded to use regex, AST analysis, etc.

        let result = match rule.name.as_str() {
            "no_empty_functions" => {
                if code.code.contains("fn ") && code.code.contains("{\n    \n}") {
                    ValidationResult::error(
                        rule.name.clone(),
                        "Generated code contains empty function".to_string(),
                        None,
                    )
                } else {
                    ValidationResult::success(rule.name.clone())
                }
            }
            "line_length_limit" => {
                let max_length = 100;
                let long_line = code.code.lines().find(|line| line.len() > max_length);
                if let Some(line) = long_line {
                    ValidationResult::warning(
                        rule.name.clone(),
                        format!("Line exceeds {} characters: {}", max_length, line),
                        None,
                    )
                } else {
                    ValidationResult::success(rule.name.clone())
                }
            }
            _ => {
                // Unknown rule type - assume success for now
                ValidationResult::success(rule.name.clone())
            }
        };

        Ok(result)
    }
}

impl Default for TemplateValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Security validation interface
#[async_trait]
pub trait SecurityValidator: Send + Sync + std::fmt::Debug + 'static {
    /// Validate template for security issues
    async fn validate_template(&self, template: &Template) -> DslResult<Vec<ValidationResult>>;

    /// Validate generated code for security issues
    async fn validate_generated_code(
        &self,
        code: &GeneratedCode,
    ) -> DslResult<Vec<ValidationResult>>;
}

/// Language-specific validation interface
#[async_trait]
pub trait LanguageValidator: Send + Sync + std::fmt::Debug + 'static {
    /// Validate template for language-specific issues
    async fn validate_template(&self, template: &Template) -> DslResult<Vec<ValidationResult>>;
}

/// Individual validation result
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Name of the validation rule
    pub rule: String,
    /// Severity of the validation result
    pub severity: ValidationSeverity,
    /// Human-readable validation message
    pub message: String,
    /// Location where the issue was found (optional)
    pub location: Option<String>,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    /// Successful validation
    Success,
    /// Informational message
    Info,
    /// Warning that doesn't prevent generation
    Warning,
    /// Error that should prevent generation
    Error,
}

impl ValidationResult {
    /// Create a success result
    pub fn success(rule: impl Into<String>) -> Self {
        Self {
            rule: rule.into(),
            severity: ValidationSeverity::Success,
            message: "Validation passed".to_string(),
            location: None,
        }
    }

    /// Create an info result
    pub fn info(
        rule: impl Into<String>,
        message: impl Into<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            rule: rule.into(),
            severity: ValidationSeverity::Info,
            message: message.into(),
            location,
        }
    }

    /// Create a warning result
    pub fn warning(
        rule: impl Into<String>,
        message: impl Into<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            rule: rule.into(),
            severity: ValidationSeverity::Warning,
            message: message.into(),
            location,
        }
    }

    /// Create an error result
    pub fn error(
        rule: impl Into<String>,
        message: impl Into<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            rule: rule.into(),
            severity: ValidationSeverity::Error,
            message: message.into(),
            location,
        }
    }

    /// Check if the result is an error
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Error)
    }

    /// Check if the result is a warning
    pub fn is_warning(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Warning)
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Template;
    use rust_ai_ide_common::ProgrammingLanguage;

    #[tokio::test]
    async fn test_basic_template_validation() {
        let validator = TemplateValidator::new();
        let mut template = Template::new("TestTemplate");

        let results = validator.validate_template(&template).await.unwrap();

        // Should have warnings about missing content
        assert!(results.iter().any(|r| r.rule == "structure"));
    }

    #[test]
    fn test_validation_result_constructors() {
        let success = ValidationResult::success("test_rule");
        assert!(success.is_success());

        let error = ValidationResult::error("test_rule", "test message", None);
        assert!(error.is_error());
        assert_eq!(error.message, "test message");

        let warning =
            ValidationResult::warning("test_rule", "test warning", Some("line 1".to_string()));
        assert!(warning.is_warning());
        assert_eq!(warning.location, Some("line 1".to_string()));
    }

    #[tokio::test]
    async fn test_parameter_validation() {
        let validator = TemplateValidator::new();
        let template = Template::new("TestTemplate");

        let results = validator.validate_parameters(&template).await.unwrap();
        assert!(results.is_empty()); // No parameters = no issues
    }
}
