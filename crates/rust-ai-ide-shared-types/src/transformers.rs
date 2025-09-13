//! Built-in type transformation logic
//!
//! This module contains the core transformation algorithms for converting
//! Rust types to various target platforms. These transformations can be
//! extended and customized through the plugin system.

use std::collections::HashMap;

use crate::errors::TypeGenerationError;
use crate::types::{ParsedType, TransformationContext};

/// Main type transformer with built-in transformation logic
#[derive(Debug)]
pub struct TypeTransformer {
    /// Built-in transformation rules
    built_in_rules: HashMap<String, TransformationRule>,

    /// Custom transformation rules
    custom_rules: HashMap<String, TransformationRule>,
}

/// Transformation rule defining how to convert a type
#[derive(Debug, Clone)]
pub struct TransformationRule {
    /// Source platform
    pub source_platform: String,

    /// Target platform
    pub target_platform: String,

    /// Type pattern to match (supports wildcards)
    pub type_pattern: String,

    /// Transformation function or template
    pub transformation: String,

    /// Priority (higher numbers take precedence)
    pub priority: i32,
}

/// Result of a type transformation
#[derive(Debug, Clone)]
pub struct TransformationResult {
    /// Transformed type
    pub transformed_type: String,

    /// Additional metadata about the transformation
    pub metadata: HashMap<String, String>,

    /// Whether the transformation was successful
    pub success: bool,

    /// Warnings or notes about the transformation
    pub warnings: Vec<String>,
}

impl Default for TypeTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeTransformer {
    /// Create a new type transformer with built-in rules
    pub fn new() -> Self {
        let mut built_in_rules = HashMap::new();
        let custom_rules = HashMap::new();

        // Add built-in Rust to TypeScript transformation rules
        Self::add_builtin_rust_to_typescript_rules(&mut built_in_rules);

        Self {
            built_in_rules,
            custom_rules,
        }
    }

    /// Add a custom transformation rule
    pub fn add_custom_rule(&mut self, rule: TransformationRule) {
        let key = format!(
            "{}:{}:{}",
            rule.source_platform, rule.target_platform, rule.type_pattern
        );
        self.custom_rules.insert(key, rule);
    }

    /// Transform a single type
    pub fn transform_type(
        &self,
        rust_type: &str,
        context: &TransformationContext,
    ) -> Result<TransformationResult, TypeGenerationError> {
        // First check custom rules (higher priority)
        if let Some(result) = self.apply_custom_rules(rust_type, context)? {
            return Ok(result);
        }

        // Then check built-in rules
        if let Some(result) = self.apply_builtin_rules(rust_type, context)? {
            return Ok(result);
        }

        // Fallback: return the type as-is with a warning
        Ok(TransformationResult {
            transformed_type: rust_type.to_string(),
            metadata:         HashMap::new(),
            success:          false,
            warnings:         vec![format!(
                "No transformation rule found for type '{}'",
                rust_type
            )],
        })
    }

    /// Transform all types in a parsed type
    pub fn transform_parsed_type(
        &self,
        parsed_type: &ParsedType,
        target_platform: &str,
    ) -> Result<TransformedType, TypeGenerationError> {
        let context = TransformationContext {
            source_platform: "rust".to_string(),
            target_platform: target_platform.to_string(),
            ..TransformationContext::default()
        };

        let mut transformed_fields = Vec::new();

        for field in &parsed_type.fields {
            let result = self.transform_type(&field.ty, &context)?;
            transformed_fields.push(TransformedField {
                name:             field.name.clone(),
                original_type:    field.ty.clone(),
                transformed_type: result.transformed_type,
                warnings:         result.warnings,
            });
        }

        Ok(TransformedType {
            original_name: parsed_type.name.clone(),
            name:          parsed_type.name.clone(), // Could be modified by transformations
            fields:        transformed_fields,
            kind:          parsed_type.kind.clone(),
            documentation: parsed_type.documentation.clone(),
        })
    }

    /// Apply custom transformation rules
    fn apply_custom_rules(
        &self,
        rust_type: &str,
        context: &TransformationContext,
    ) -> Result<Option<TransformationResult>, TypeGenerationError> {
        for rule in self.custom_rules.values() {
            if self.rule_matches(rule, rust_type, context)? {
                return Ok(Some(self.apply_rule(rule, rust_type, context)?));
            }
        }
        Ok(None)
    }

    /// Apply built-in transformation rules
    fn apply_builtin_rules(
        &self,
        rust_type: &str,
        context: &TransformationContext,
    ) -> Result<Option<TransformationResult>, TypeGenerationError> {
        for rule in self.built_in_rules.values() {
            if self.rule_matches(rule, rust_type, context)? {
                return Ok(Some(self.apply_rule(rule, rust_type, context)?));
            }
        }
        Ok(None)
    }

    /// Check if a transformation rule matches the given type and context
    fn rule_matches(
        &self,
        rule: &TransformationRule,
        rust_type: &str,
        context: &TransformationContext,
    ) -> Result<bool, TypeGenerationError> {
        // Check platform match
        if rule.source_platform != context.source_platform || rule.target_platform != context.target_platform {
            return Ok(false);
        }

        // Check type pattern match (simple wildcard support)
        if rule.type_pattern == "*" || rule.type_pattern == rust_type {
            return Ok(true);
        }

        // Support for generic type patterns
        if rule.type_pattern.starts_with("Option<") && rust_type.starts_with("Option<") {
            return Ok(true);
        }
        if rule.type_pattern.starts_with("Result<") && rust_type.starts_with("Result<") {
            return Ok(true);
        }
        if rule.type_pattern.starts_with("Vec<") && rust_type.starts_with("Vec<") {
            return Ok(true);
        }

        Ok(false)
    }

    /// Apply a transformation rule
    fn apply_rule(
        &self,
        rule: &TransformationRule,
        rust_type: &str,
        _context: &TransformationContext,
    ) -> Result<TransformationResult, TypeGenerationError> {
        // For now, use simple string replacements
        // In a real implementation, this could use templates or more sophisticated logic
        let transformed_type = match rule.transformation.as_str() {
            "string" => "string".to_string(),
            "number" => "number".to_string(),
            "boolean" => "boolean".to_string(),
            "Array" => "Array".to_string(),
            "undefined" => "undefined".to_string(),
            "any" => "any".to_string(),
            // Generic type transformations
            "option_to_undefined" => self.transform_option_to_undefined(rust_type),
            "result_to_any" => "any".to_string(),
            "vec_to_array" => self.transform_vec_to_array(rust_type),
            _ => rust_type.to_string(),
        };

        Ok(TransformationResult {
            transformed_type,
            metadata: HashMap::new(),
            success: true,
            warnings: vec![],
        })
    }

    /// Transform Option<T> to T | undefined
    fn transform_option_to_undefined(&self, rust_type: &str) -> String {
        if let Some(start) = rust_type.find('<') {
            if let Some(end) = rust_type.rfind('>') {
                let inner = &rust_type[start + 1..end];
                let transformer = TypeTransformer::new();
                let context = TransformationContext::default();
                if let Ok(result) = transformer.transform_type(inner, &context) {
                    return format!("{} | undefined", result.transformed_type);
                }
            }
        }
        "undefined".to_string()
    }

    /// Transform Vec<T> to Array<T>
    fn transform_vec_to_array(&self, rust_type: &str) -> String {
        if let Some(start) = rust_type.find('<') {
            if let Some(end) = rust_type.rfind('>') {
                let inner = &rust_type[start + 1..end];
                let transformer = TypeTransformer::new();
                let context = TransformationContext::default();
                if let Ok(result) = transformer.transform_type(inner, &context) {
                    return format!("Array<{}>", result.transformed_type);
                }
            }
        }
        "Array".to_string()
    }

    /// Add built-in Rust to TypeScript transformation rules
    fn add_builtin_rust_to_typescript_rules(rules: &mut HashMap<String, TransformationRule>) {
        let base_rules = vec![
            ("rust:typescript:String", "string"),
            ("rust:typescript:str", "string"),
            ("rust:typescript:i8", "number"),
            ("rust:typescript:i16", "number"),
            ("rust:typescript:i32", "number"),
            ("rust:typescript:i64", "number"),
            ("rust:typescript:u8", "number"),
            ("rust:typescript:u16", "number"),
            ("rust:typescript:u32", "number"),
            ("rust:typescript:u64", "number"),
            ("rust:typescript:f32", "number"),
            ("rust:typescript:f64", "number"),
            ("rust:typescript:bool", "boolean"),
            ("rust:typescript:char", "string"),
            ("rust:typescript:Vec<*>", "vec_to_array"),
            ("rust:typescript:Option<*>", "option_to_undefined"),
            ("rust:typescript:Result<*>", "result_to_any"),
            ("rust:typescript:HashMap<*>", "record"),
            ("rust:typescript:BTreeMap<*>", "record"),
            ("rust:typescript:HashSet<*>", "set"),
            ("rust:typescript:BTreeSet<*>", "set"),
        ];

        for (pattern, transformation) in base_rules {
            let key = pattern.to_string();
            let parts: Vec<&str> = pattern.split(':').collect();
            if parts.len() == 3 {
                rules.insert(key, TransformationRule {
                    source_platform: parts[0].to_string(),
                    target_platform: parts[1].to_string(),
                    type_pattern:    parts[2].to_string(),
                    transformation:  transformation.to_string(),
                    priority:        0,
                });
            }
        }
    }
}

/// A transformed type with all fields processed
#[derive(Debug, Clone)]
pub struct TransformedType {
    /// Original type name
    pub original_name: String,

    /// Transformed type name (may be different)
    pub name: String,

    /// Transformed fields
    pub fields: Vec<TransformedField>,

    /// Type kind
    pub kind: crate::types::TypeKind,

    /// Documentation
    pub documentation: Option<String>,
}

/// A transformed field
#[derive(Debug, Clone)]
pub struct TransformedField {
    /// Field name
    pub name: String,

    /// Original Rust type
    pub original_type: String,

    /// Transformed type
    pub transformed_type: String,

    /// Any warnings from the transformation
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeKind;

    #[test]
    fn test_basic_type_transformation() {
        let transformer = TypeTransformer::new();
        let context = TransformationContext::default();

        let result = transformer.transform_type("String", &context).unwrap();
        assert_eq!(result.transformed_type, "string");
        assert!(result.success);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_option_transformation() {
        let transformer = TypeTransformer::new();
        let context = TransformationContext::default();

        let result = transformer
            .transform_type("Option<String>", &context)
            .unwrap();
        assert_eq!(result.transformed_type, "string | undefined");
        assert!(result.success);
    }

    #[test]
    fn test_vec_transformation() {
        let transformer = TypeTransformer::new();
        let context = TransformationContext::default();

        let result = transformer.transform_type("Vec<i32>", &context).unwrap();
        assert_eq!(result.transformed_type, "Array<number>");
        assert!(result.success);
    }

    #[test]
    fn test_unknown_type_fallback() {
        let transformer = TypeTransformer::new();
        let context = TransformationContext::default();

        let result = transformer
            .transform_type("SomeUnknownType", &context)
            .unwrap();
        assert_eq!(result.transformed_type, "SomeUnknownType");
        assert!(!result.success);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_parsed_type_transformation() {
        let transformer = TypeTransformer::new();
        let parsed_type = ParsedType {
            name:             "TestStruct".to_string(),
            kind:             TypeKind::Struct,
            documentation:    None,
            visibility:       crate::types::Visibility::Public,
            generics:         vec![],
            fields:           vec![
                crate::types::Field {
                    name:          "name".to_string(),
                    ty:            "String".to_string(),
                    documentation: None,
                    visibility:    crate::types::Visibility::Public,
                    is_mutable:    false,
                    attributes:    vec![],
                },
                crate::types::Field {
                    name:          "count".to_string(),
                    ty:            "i32".to_string(),
                    documentation: None,
                    visibility:    crate::types::Visibility::Public,
                    is_mutable:    false,
                    attributes:    vec![],
                },
            ],
            variants:         vec![],
            associated_items: vec![],
            attributes:       vec![],
            source_location:  crate::types::SourceLocation {
                file:        "test.rs".to_string(),
                line:        1,
                column:      1,
                module_path: vec![],
            },
            dependencies:     vec![],
            metadata:         crate::types::TypeMetadata::default(),
        };

        let transformed = transformer
            .transform_parsed_type(&parsed_type, "typescript")
            .unwrap();
        assert_eq!(transformed.fields.len(), 2);
        assert_eq!(transformed.fields[0].transformed_type, "string");
        assert_eq!(transformed.fields[1].transformed_type, "number");
    }

    #[test]
    fn test_custom_rule_addition() {
        let mut transformer = TypeTransformer::new();
        let custom_rule = TransformationRule {
            source_platform: "rust".to_string(),
            target_platform: "typescript".to_string(),
            type_pattern:    "CustomType".to_string(),
            transformation:  "string".to_string(),
            priority:        10,
        };

        transformer.add_custom_rule(custom_rule);
        let context = TransformationContext::default();

        let result = transformer.transform_type("CustomType", &context).unwrap();
        assert_eq!(result.transformed_type, "string");
        assert!(result.success);
    }
}
