use std::collections::HashSet;

use regex::Regex;

use crate::spec_generation::types::{
    CodeFile, Entity, FunctionSpec, ParsedSpecification, Severity, ValidationIssue, ValidationResult,
};

/// Validator for generated code and specifications
pub struct CodeValidator {
    // Cache for compiled regex patterns
    identifier_regex:     Regex,
    doc_comment_regex:    Regex,
    error_type_whitelist: HashSet<&'static str>,
}

impl Default for CodeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeValidator {
    /// Create a new CodeValidator with default settings
    pub fn new() -> Self {
        Self {
            identifier_regex:     Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").expect("Invalid identifier regex"),
            doc_comment_regex:    Regex::new(r#"///.*"#).expect("Invalid doc comment regex"),
            error_type_whitelist: [
                "String",
                "&str",
                "std::io::Error",
                "anyhow::Error",
                "Box<dyn std::error::Error>",
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }

    /// Validate a parsed specification
    pub fn validate_specification(&self, spec: &ParsedSpecification) -> ValidationResult {
        let mut issues = Vec::new();
        let mut valid = true;

        // Validate entities
        for entity in &spec.entities {
            let entity_issues = self.validate_entity(entity);
            if !entity_issues.is_empty() {
                valid = false;
                issues.extend(entity_issues);
            }
        }

        // Validate functions
        for func in &spec.functions {
            let func_issues = self.validate_function(func);
            if !func_issues.is_empty() {
                valid = false;
                issues.extend(func_issues);
            }
        }

        // Validate requirements
        for req in &spec.requirements {
            if req.id.is_empty() || req.description.is_empty() {
                issues.push(ValidationIssue {
                    severity:   Severity::Error,
                    message:    "Requirement must have both ID and description".to_string(),
                    location:   format!("Requirement: {}", req.id),
                    suggestion: Some("Add missing ID or description".to_string()),
                });
                valid = false;
            }
        }

        // Validate patterns
        for pattern in &spec.patterns {
            if pattern.name.is_empty() {
                issues.push(ValidationIssue {
                    severity:   Severity::Error,
                    message:    "Pattern name cannot be empty".to_string(),
                    location:   "Pattern validation".to_string(),
                    suggestion: Some("Add a name to the pattern".to_string()),
                });
                valid = false;
            }

            if pattern.confidence < 0.0 || pattern.confidence > 1.0 {
                issues.push(ValidationIssue {
                    severity:   Severity::Warning,
                    message:    format!(
                        "Pattern '{}' has invalid confidence value: {}",
                        pattern.name, pattern.confidence
                    ),
                    location:   format!("Pattern: {}", pattern.name),
                    suggestion: Some("Confidence should be between 0.0 and 1.0".to_string()),
                });
            }
        }

        // Calculate score (simple implementation)
        let score = if issues.is_empty() {
            1.0
        } else {
            let error_count = issues
                .iter()
                .filter(|i| i.severity == Severity::Error)
                .count();
            let warning_count = issues
                .iter()
                .filter(|i| i.severity == Severity::Warning)
                .count();
            let info_count = issues
                .iter()
                .filter(|i| i.severity == Severity::Info)
                .count();

            // Simple scoring: 1.0 - (0.5 * errors + 0.3 * warnings + 0.1 * infos)
            let penalty = (0.5 * error_count as f32) + (0.3 * warning_count as f32) + (0.1 * info_count as f32);
            (1.0 - penalty).max(0.0)
        };

        ValidationResult {
            is_valid: valid,
            issues,
            score,
        }
    }

    /// Validate a generated code file
    pub fn validate_code_file(&self, file: &CodeFile) -> ValidationResult {
        let mut issues = Vec::new();
        let mut valid = true;

        // Check file extension
        if !file.path.ends_with(".rs") && !file.path.ends_with(".toml") && !file.path.ends_with(".md") {
            issues.push(ValidationIssue {
                severity:   Severity::Warning,
                message:    format!("Unexpected file extension for path: {}", file.path),
                location:   file.path.clone(),
                suggestion: Some("Use standard file extensions (.rs, .toml, .md)".to_string()),
            });
        }

        // Check for empty files
        if file.content.trim().is_empty() {
            issues.push(ValidationIssue {
                severity:   Severity::Error,
                message:    "File is empty".to_string(),
                location:   file.path.clone(),
                suggestion: Some("Add content to the file".to_string()),
            });
            valid = false;
        }

        // Check for TODOs in production code
        if !file.is_test && file.content.contains("TODO") {
            issues.push(ValidationIssue {
                severity:   Severity::Warning,
                message:    "TODO comment found in production code".to_string(),
                location:   file.path.clone(),
                suggestion: Some("Address the TODO or move it to an issue tracker".to_string()),
            });
        }

        // Check for unwrap/expect in production code
        if !file.is_test && (file.content.contains(".unwrap(") || file.content.contains(".expect(")) {
            issues.push(ValidationIssue {
                severity:   Severity::Warning,
                message:    "unwrap() or expect() found in production code".to_string(),
                location:   file.path.clone(),
                suggestion: Some("Use proper error handling with Result or Option".to_string()),
            });
        }

        // Calculate error count for validation
        let error_count = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();

        // Calculate score (simple implementation)
        let score = if issues.is_empty() {
            1.0
        } else {
            let warning_count = issues
                .iter()
                .filter(|i| i.severity == Severity::Warning)
                .count();

            // Simple scoring: 1.0 - (0.7 * errors + 0.3 * warnings)
            let penalty = (0.7 * error_count as f32) + (0.3 * warning_count as f32);
            (1.0 - penalty).max(0.0)
        };

        ValidationResult {
            is_valid: valid && error_count == 0,
            issues,
            score,
        }
    }

    /// Validate a single entity
    fn validate_entity(&self, entity: &Entity) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check entity name
        if !self.is_valid_identifier(&entity.name) {
            issues.push(ValidationIssue {
                severity:   Severity::Error,
                message:    format!("Invalid entity name: '{}'", entity.name),
                location:   format!("Entity: {}", entity.name),
                suggestion: Some("Use PascalCase for type names".to_string()),
            });
        }

        // Check for documentation
        if entity.docs.is_empty() {
            issues.push(ValidationIssue {
                severity:   Severity::Warning,
                message:    format!("Entity '{}' is missing documentation", entity.name),
                location:   format!("Entity: {}", entity.name),
                suggestion: Some("Add documentation comments (///)".to_string()),
            });
        }

        // Validate fields
        for field in &entity.fields {
            if !self.is_valid_identifier(&field.name) {
                issues.push(ValidationIssue {
                    severity:   Severity::Error,
                    message:    format!(
                        "Invalid field name: '{}' in entity '{}'",
                        field.name, entity.name
                    ),
                    location:   format!("Entity: {} Field: {}", entity.name, field.name),
                    suggestion: Some("Use snake_case for field names".to_string()),
                });
            }

            if field.field_type.trim().is_empty() {
                issues.push(ValidationIssue {
                    severity:   Severity::Error,
                    message:    format!(
                        "Field '{}' in entity '{}' has no type",
                        field.name, entity.name
                    ),
                    location:   format!("Entity: {} Field: {}", entity.name, field.name),
                    suggestion: Some("Add a type to the field".to_string()),
                });
            }
        }

        issues
    }

    /// Validate a single function
    fn validate_function(&self, func: &FunctionSpec) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check function name
        if !self.is_valid_identifier(&func.name) || !func.name.chars().next().unwrap().is_lowercase() {
            issues.push(ValidationIssue {
                severity:   Severity::Error,
                message:    format!("Invalid function name: '{}'", func.name),
                location:   format!("Function: {}", func.name),
                suggestion: Some("Use snake_case for function names".to_string()),
            });
        }

        // Check for documentation
        if func.docs.is_empty() {
            issues.push(ValidationIssue {
                severity:   Severity::Warning,
                message:    format!("Function '{}' is missing documentation", func.name),
                location:   format!("Function: {}", func.name),
                suggestion: Some("Add documentation comments (///)".to_string()),
            });
        }

        // Check parameters
        for param in &func.parameters {
            if !self.is_valid_identifier(&param.name) {
                issues.push(ValidationIssue {
                    severity:   Severity::Error,
                    message:    format!(
                        "Invalid parameter name: '{}' in function '{}'",
                        param.name, func.name
                    ),
                    location:   format!("Function: {} Parameter: {}", func.name, param.name),
                    suggestion: Some("Use snake_case for parameter names".to_string()),
                });
            }

            if param.param_type.trim().is_empty() {
                issues.push(ValidationIssue {
                    severity:   Severity::Error,
                    message:    format!(
                        "Parameter '{}' in function '{}' has no type",
                        param.name, func.name
                    ),
                    location:   format!("Function: {} Parameter: {}", func.name, param.name),
                    suggestion: Some("Add a type to the parameter".to_string()),
                });
            }
        }

        // Check return type documentation
        if !func.return_type.is_empty()
            && !func
                .docs
                .iter()
                .any(|d| d.contains("Returns:") || d.contains("# Returns"))
        {
            issues.push(ValidationIssue {
                severity:   Severity::Info,
                message:    format!(
                    "Function '{}' has a return type but no return documentation",
                    func.name
                ),
                location:   format!("Function: {}", func.name),
                suggestion: Some("Add a 'Returns:' section to the documentation".to_string()),
            });
        }

        // Check error types
        for error_type in &func.error_types {
            if !self.error_type_whitelist.contains(error_type.as_str())
                && !error_type.starts_with("crate::")
                && !error_type.starts_with("std::")
            {
                issues.push(ValidationIssue {
                    severity:   Severity::Warning,
                    message:    format!(
                        "Uncommon error type '{}' in function '{}'",
                        error_type, func.name
                    ),
                    location:   format!("Function: {} Error type: {}", func.name, error_type),
                    suggestion: Some("Use a more common error type or document why this one is needed".to_string()),
                });
            }
        }

        issues
    }

    /// Check if a string is a valid Rust identifier
    fn is_valid_identifier(&self, s: &str) -> bool {
        !s.is_empty() && self.identifier_regex.is_match(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_generation::types::{
        CodeFile, Entity, EntityType, Field, FunctionSpec, Parameter, ParsedSpecification, Requirement, Severity,
    };

    #[test]
    fn test_validate_entity() {
        let validator = CodeValidator::new();
        let entity = Entity {
            name:         "User".to_string(),
            entity_type:  EntityType::Struct,
            fields:       vec![
                Field {
                    name:        "id".to_string(),
                    field_type:  "String".to_string(),
                    is_optional: false,
                    docs:        vec!["Unique identifier".to_string()],
                },
                Field {
                    name:        "user_name".to_string(),
                    field_type:  "String".to_string(),
                    is_optional: false,
                    docs:        vec![],
                },
            ],
            docs:         vec!["A user in the system".to_string()],
            requirements: vec!["REQ-001".to_string()],
        };

        let issues = validator.validate_entity(&entity);
        assert_eq!(issues.len(), 1); // Warning for missing field docs
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("is missing documentation"));
    }

    #[test]
    fn test_validate_function() {
        let validator = CodeValidator::new();
        let func = FunctionSpec {
            name:         "create_user".to_string(),
            return_type:  "Result<User, String>".to_string(),
            parameters:   vec![
                Parameter {
                    name:       "user_name".to_string(),
                    param_type: "String".to_string(),
                    is_mut:     false,
                    is_ref:     false,
                },
                Parameter {
                    name:       "email".to_string(),
                    param_type: "String".to_string(),
                    is_mut:     false,
                    is_ref:     false,
                },
            ],
            docs:         vec![
                "Creates a new user with the given username and email".to_string(),
                "# Arguments".to_string(),
                "* `user_name` - The username for the new user".to_string(),
                "* `email` - The email address for the new user".to_string(),
                "# Returns".to_string(),
                "A Result containing the new User or an error message".to_string(),
            ],
            requirements: vec!["REQ-001".to_string()],
            error_types:  vec!["String".to_string()],
        };

        let issues = validator.validate_function(&func);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_specification() {
        let validator = CodeValidator::new();
        let spec = ParsedSpecification {
            requirements: vec![Requirement {
                id:          "REQ-001".to_string(),
                description: "The system must store user information".to_string(),
                priority:    1,
                related_to:  vec!["User".to_string()],
            }],
            patterns:     vec![],
            entities:     vec![Entity {
                name:         "User".to_string(),
                entity_type:  EntityType::Struct,
                fields:       vec![Field {
                    name:        "id".to_string(),
                    field_type:  "String".to_string(),
                    is_optional: false,
                    docs:        vec!["Unique identifier".to_string()],
                }],
                docs:         vec!["A user in the system".to_string()],
                requirements: vec!["REQ-001".to_string()],
            }],
            functions:    vec![FunctionSpec {
                name:         "create_user".to_string(),
                return_type:  "Result<User, String>".to_string(),
                parameters:   vec![],
                docs:         vec!["Creates a new user".to_string()],
                requirements: vec!["REQ-001".to_string()],
                error_types:  vec!["String".to_string()],
            }],
        };

        let result = validator.validate_specification(&spec);
        assert!(result.is_valid);
        assert_eq!(result.score, 1.0);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_code_file() {
        let validator = CodeValidator::new();
        let file = CodeFile {
            path:    "src/main.rs".to_string(),
            content: r#"
            // This is a test file

            fn main() {
                println!("Hello, world!");
            }
            "#
            .to_string(),
            is_test: false,
        };

        let result = validator.validate_code_file(&file);
        assert!(result.is_valid);
        assert_eq!(result.score, 1.0);
        assert!(result.issues.is_empty());
    }
}
