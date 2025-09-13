//! Cross-platform type bridging and validation
//!
//! This module provides functionality for validating types across different
//! platforms and ensuring compatibility between Rust backend and frontend types.

use std::collections::HashMap;

use crate::config::GenerationConfig;
use crate::errors::TypeBridgeError;
use crate::types::*;

/// Main type bridge for cross-platform validation
#[derive(Debug)]
pub struct TypeBridge {
    /// Cross-platform mapping rules
    platform_mappings: HashMap<String, PlatformMapping>,

    /// Validation rules
    validation_rules: Vec<ValidationRule>,

    /// Compatibility checker
    compatibility_checker: CompatibilityChecker,
}

/// Platform-specific type mappings
#[derive(Debug, Clone)]
struct PlatformMapping {
    /// Type mappings for this platform
    type_mappings: HashMap<String, String>,

    /// Platform-specific features
    features: Vec<String>,

    /// Compatibility flags
    compatibility_flags: HashMap<String, bool>,
}

/// Validation rule for type checking
#[derive(Debug, Clone)]
struct ValidationRule {
    /// Rule name
    name: String,

    /// Platforms this rule applies to
    platforms: Vec<String>,

    /// Validation function (simplified as a string for now)
    rule: String,

    /// Severity level
    severity: ValidationSeverity,
}

/// Severity levels for validation issues
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ValidationSeverity {
    /// Error - prevents generation
    Error,
    /// Warning - generation continues but with warnings
    Warning,
    /// Info - informational only
    Info,
}

/// Compatibility checker for cross-platform validation
#[derive(Debug)]
struct CompatibilityChecker {
    /// Compatibility matrix
    compatibility_matrix: HashMap<(String, String), CompatibilityLevel>,
}

/// Compatibility level between two platforms
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
enum CompatibilityLevel {
    /// Fully compatible
    Full,
    /// Partially compatible with workarounds
    Partial,
    /// Not compatible
    None,
}

/// Validation result for a set of types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    /// Overall compatibility status
    pub compatible: bool,

    /// Validation issues found
    pub issues: Vec<ValidationIssue>,

    /// Compatibility score (0.0 to 1.0)
    pub compatibility_score: f32,

    /// Recommended fixes
    pub recommendations: Vec<String>,

    /// Type mappings that were applied
    pub applied_mappings: HashMap<String, String>,
}

/// Individual validation issue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationIssue {
    /// Issue type
    pub issue_type: ValidationIssueType,

    /// Severity
    pub severity: ValidationSeverity,

    /// Source type name
    pub source_type: String,

    /// Target platform
    pub target_platform: String,

    /// Description of the issue
    pub description: String,

    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Types of validation issues
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ValidationIssueType {
    /// Type not supported on target platform
    UnsupportedType,
    /// Type mapping unknown
    UnknownMapping,
    /// Platform-specific feature missing
    MissingFeature,
    /// Compatibility issue
    CompatibilityIssue,
    /// Performance concern
    PerformanceIssue,
}

impl TypeBridge {
    /// Create a new type bridge with default configuration
    pub fn new(config: GenerationConfig) -> Result<Self, TypeBridgeError> {
        let mut platform_mappings = HashMap::new();

        // Initialize TypeScript mappings
        let mut ts_mappings = HashMap::new();
        ts_mappings.insert("String".to_string(), "string".to_string());
        ts_mappings.insert("i32".to_string(), "number".to_string());
        ts_mappings.insert("bool".to_string(), "boolean".to_string());

        platform_mappings.insert("typescript".to_string(), PlatformMapping {
            type_mappings:       ts_mappings,
            features:            vec!["interfaces".to_string(), "unions".to_string()],
            compatibility_flags: HashMap::new(),
        });

        // Initialize JavaScript mappings
        let mut js_mappings = HashMap::new();
        js_mappings.insert("String".to_string(), "string".to_string());
        js_mappings.insert("i32".to_string(), "number".to_string());
        js_mappings.insert("bool".to_string(), "boolean".to_string());

        platform_mappings.insert("javascript".to_string(), PlatformMapping {
            type_mappings:       js_mappings,
            features:            vec!["objects".to_string(), "functions".to_string()],
            compatibility_flags: HashMap::new(),
        });

        // Initialize validation rules
        let validation_rules = vec![
            ValidationRule {
                name:      "check_basic_types".to_string(),
                platforms: vec!["typescript".to_string(), "javascript".to_string()],
                rule:      "basic_type_check".to_string(),
                severity:  ValidationSeverity::Error,
            },
            ValidationRule {
                name:      "check_complex_types".to_string(),
                platforms: vec!["typescript".to_string()],
                rule:      "complex_type_check".to_string(),
                severity:  ValidationSeverity::Warning,
            },
        ];

        // Initialize compatibility matrix
        let mut compatibility_matrix = HashMap::new();
        compatibility_matrix.insert(
            ("rust".to_string(), "typescript".to_string()),
            CompatibilityLevel::Full,
        );
        compatibility_matrix.insert(
            ("rust".to_string(), "javascript".to_string()),
            CompatibilityLevel::Partial,
        );
        compatibility_matrix.insert(
            ("typescript".to_string(), "javascript".to_string()),
            CompatibilityLevel::Full,
        );

        let compatibility_checker = CompatibilityChecker {
            compatibility_matrix,
        };

        Ok(Self {
            platform_mappings,
            validation_rules,
            compatibility_checker,
        })
    }

    /// Validate types for cross-platform compatibility
    pub async fn validate_types(&self, types: &[ParsedType]) -> Result<ValidationResult, TypeBridgeError> {
        let mut issues = Vec::new();
        let mut applied_mappings = HashMap::new();
        let mut compatible_types = 0;

        for parsed_type in types {
            let type_issues = self.validate_single_type(parsed_type).await?;
            issues.extend(type_issues.issues);
            applied_mappings.extend(type_issues.mappings);

            if type_issues.compatible {
                compatible_types += 1;
            }
        }

        let compatibility_score = if types.is_empty() {
            1.0
        } else {
            compatible_types as f32 / types.len() as f32
        };

        let compatible = issues
            .iter()
            .all(|issue| issue.severity != ValidationSeverity::Error);

        let recommendations = self.generate_recommendations(&issues);

        Ok(ValidationResult {
            compatible,
            issues,
            compatibility_score,
            recommendations,
            applied_mappings,
        })
    }

    /// Validate a single parsed type
    async fn validate_single_type(&self, parsed_type: &ParsedType) -> Result<TypeValidationResult, TypeBridgeError> {
        let mut issues = Vec::new();
        let mut mappings = HashMap::new();
        let mut compatible = true;

        // Check each field for each supported platform
        for (platform_name, platform_mapping) in &self.platform_mappings {
            for field in &parsed_type.fields {
                let field_issues = self.validate_field_type(&field.ty, platform_name, platform_mapping)?;
                issues.extend(field_issues);

                if let Some(mapping) = platform_mapping.type_mappings.get(&field.ty) {
                    mappings.insert(
                        format!("{}.{}", parsed_type.name, field.name),
                        mapping.clone(),
                    );
                }
            }
        }

        // Check overall type compatibility
        if !self.check_type_compatibility(parsed_type) {
            issues.push(ValidationIssue {
                issue_type:      ValidationIssueType::CompatibilityIssue,
                severity:        ValidationSeverity::Warning,
                source_type:     parsed_type.name.clone(),
                target_platform: "all".to_string(),
                description:     format!("Type '{}' may have compatibility issues", parsed_type.name),
                suggestion:      Some(
                    "Consider simplifying the type structure or adding platform-specific attributes".to_string(),
                ),
            });
        }

        if issues
            .iter()
            .any(|issue| issue.severity == ValidationSeverity::Error)
        {
            compatible = false;
        }

        Ok(TypeValidationResult {
            compatible,
            issues,
            mappings,
        })
    }

    /// Validate a field type for a specific platform
    fn validate_field_type(
        &self,
        field_type: &str,
        platform: &str,
        platform_mapping: &PlatformMapping,
    ) -> Result<Vec<ValidationIssue>, TypeBridgeError> {
        let mut issues = Vec::new();

        // Check basic type mapping
        let is_mapped = platform_mapping.type_mappings.contains_key(field_type);
        let is_basic_rust_type = Self::is_basic_rust_type(field_type);

        if !is_mapped && is_basic_rust_type {
            issues.push(ValidationIssue {
                issue_type:      ValidationIssueType::UnknownMapping,
                severity:        ValidationSeverity::Error,
                source_type:     field_type.to_string(),
                target_platform: platform.to_string(),
                description:     format!(
                    "No mapping found for type '{}' on platform '{}'",
                    field_type, platform
                ),
                suggestion:      Some(format!(
                    "Add type mapping for '{}' in the configuration",
                    field_type
                )),
            });
        }

        // Check for complex types that might not be supported
        if field_type.contains("Result<") {
            issues.push(ValidationIssue {
                issue_type:      ValidationIssueType::CompatibilityIssue,
                severity:        ValidationSeverity::Warning,
                source_type:     field_type.to_string(),
                target_platform: platform.to_string(),
                description:     format!("Result types are simplified on platform '{}'", platform),
                suggestion:      Some("Consider using union types or separate success/error responses".to_string()),
            });
        }

        Ok(issues)
    }

    /// Check if a type is a basic Rust type that should have a mapping
    fn is_basic_rust_type(type_name: &str) -> bool {
        let basic_types = [
            "String", "str", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
        ];

        basic_types.contains(&type_name) || type_name.starts_with("Option<") || type_name.starts_with("Result<")
    }

    /// Check overall type compatibility
    fn check_type_compatibility(&self, parsed_type: &ParsedType) -> bool {
        match parsed_type.kind {
            TypeKind::Struct => {
                // Check for overly complex structs
                parsed_type.fields.len() <= 50 // Arbitrary limit
            }
            TypeKind::Enum => {
                // Check for enums with too many variants or complex associated data
                parsed_type.variants.len() <= 20 && parsed_type.variants.iter().all(|v| v.fields.len() <= 5)
            }
            TypeKind::Union => {
                // Unions are generally complex to represent
                false
            }
            TypeKind::TypeAlias => {
                // Type aliases are generally compatible
                true
            }
        }
    }

    /// Generate recommendations based on validation issues
    fn generate_recommendations(&self, issues: &[ValidationIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let error_count = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .count();
        let warning_count = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .count();

        if error_count > 0 {
            recommendations.push(format!(
                "Fix {} validation errors to ensure compatibility",
                error_count
            ));
        }

        if warning_count > 0 {
            recommendations.push(format!(
                "Address {} validation warnings for better compatibility",
                warning_count
            ));
        }

        let unknown_mappings = issues
            .iter()
            .filter(|i| i.issue_type == ValidationIssueType::UnknownMapping)
            .count();

        if unknown_mappings > 0 {
            recommendations.push("Add type mappings for unmapped types in the configuration".to_string());
        }

        recommendations
    }
}

/// Internal result type for single type validation
struct TypeValidationResult {
    compatible: bool,
    issues:     Vec<ValidationIssue>,
    mappings:   HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::TypeParser;

    #[tokio::test]
    async fn test_basic_type_validation() {
        let bridge = TypeBridge::new(GenerationConfig::default()).unwrap();

        let source = r#"
            pub struct TestStruct {
                pub name: String,
                pub age: i32,
                pub active: bool,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        let result = bridge.validate_types(&types).await.unwrap();

        assert!(result.compatible);
        assert!(result.compatibility_score > 0.8);
        assert!(result.issues.is_empty());
    }

    #[tokio::test]
    async fn test_result_type_validation() {
        let bridge = TypeBridge::new(GenerationConfig::default()).unwrap();

        let source = r#"
            pub struct TestStruct {
                pub result: Result<String, i32>,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        let result = bridge.validate_types(&types).await.unwrap();

        assert!(result.compatible); // Should still be compatible with warnings
        assert!(!result.issues.is_empty()); // Should have warnings about Result types
    }

    #[test]
    fn test_platform_mapping() {
        let config = GenerationConfig::default();
        let bridge = TypeBridge::new(config).unwrap();

        let ts_mapping = bridge.platform_mappings.get("typescript").unwrap();
        assert_eq!(
            ts_mapping.type_mappings.get("String"),
            Some(&"string".to_string())
        );
        assert_eq!(
            ts_mapping.type_mappings.get("bool"),
            Some(&"boolean".to_string())
        );
    }

    #[tokio::test]
    async fn test_validation_issue_generation() {
        let bridge = TypeBridge::new(GenerationConfig::default()).unwrap();

        let source = r#"
            pub struct ComplexStruct {
                pub field1: String,
                pub field2: SomeUnknownType,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        let result = bridge.validate_types(&types).await.unwrap();

        assert!(!result.issues.is_empty());
        // Should have at least one error about unknown type mapping
    }
}
