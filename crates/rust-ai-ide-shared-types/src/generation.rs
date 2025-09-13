//! Code generation engines for different target platforms
//!
//! This module provides the main generation interfaces and implementations
//! for converting Rust types to various target platforms.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::config::{GenerationConfig, TypeScriptConfig};
use crate::errors::TypeGenerationError;
use crate::parsing::TypeParser;
use crate::types::*;

/// Main type generator that orchestrates code generation across platforms
#[derive(Debug)]
pub struct TypeGenerator {
    /// Configuration for generation
    config: GenerationConfig,

    /// Type parser for extracting type information
    parser: TypeParser,

    /// Platform-specific generators
    generators: HashMap<String, Box<dyn CodeGenerator>>,
}

/// Trait for platform-specific code generators
#[async_trait]
pub trait CodeGenerator: Send + Sync + std::fmt::Debug {
    /// Generate code for a set of types
    async fn generate(
        &self,
        types: &[ParsedType],
        config: &GenerationConfig,
    ) -> Result<GeneratedCode, TypeGenerationError>;

    /// Get the target platform name
    fn target_platform(&self) -> String;

    /// Check if this generator can handle the given types
    fn can_handle(&self, types: &[ParsedType]) -> bool;
}

impl TypeGenerator {
    /// Create a new type generator with default configuration
    pub fn new(config: TypeScriptConfig) -> Result<Self, TypeGenerationError> {
        let mut generation_config = GenerationConfig::default();
        generation_config.typescript = config;

        Self::with_full_config(generation_config)
    }

    /// Create a new type generator with full configuration
    pub fn with_full_config(config: GenerationConfig) -> Result<Self, TypeGenerationError> {
        let parser = TypeParser::new();
        let mut generators: HashMap<String, Box<dyn CodeGenerator>> = HashMap::new();

        // Register built-in generators
        generators.insert(
            "typescript".to_string(),
            Box::new(TypeScriptGenerator::new()),
        );

        // Additional generators can be registered here

        Ok(Self {
            config,
            parser,
            generators,
        })
    }

    /// Generate types from a Rust source file
    pub async fn generate_types(
        &self,
        file_path: &str,
        type_names: &[&str],
    ) -> Result<GeneratedCode, TypeGenerationError> {
        let content = std::fs::read_to_string(file_path)?;
        self.generate_types_from_source(&content, file_path, type_names)
            .await
    }

    /// Generate types from Rust source code
    pub async fn generate_types_from_source(
        &self,
        source: &str,
        file_path: &str,
        type_names: &[&str],
    ) -> Result<GeneratedCode, TypeGenerationError> {
        // Parse the source file
        let all_types = self.parser.parse_file(source, file_path)?;

        // Filter types based on requested names
        let filtered_types: Vec<ParsedType> = if type_names.is_empty() {
            all_types
        } else {
            all_types
                .into_iter()
                .filter(|ty| type_names.contains(&ty.name.as_str()))
                .collect()
        };

        // Generate code using TypeScript generator (for now)
        let generator = self
            .generators
            .get("typescript")
            .ok_or_else(|| TypeGenerationError::AnalysisError("TypeScript generator not found".to_string()))?;

        generator.generate(&filtered_types, &self.config).await
    }

    /// Generate types for all supported platforms
    pub async fn generate_cross_platform(
        &self,
        source: &str,
        file_path: &str,
        type_names: &[&str],
    ) -> Result<HashMap<String, GeneratedCode>, TypeGenerationError> {
        let all_types = self.parser.parse_file(source, file_path)?;
        let filtered_types: Vec<ParsedType> = if type_names.is_empty() {
            all_types
        } else {
            all_types
                .into_iter()
                .filter(|ty| type_names.contains(&ty.name.as_str()))
                .collect()
        };

        let mut results = HashMap::new();

        for (platform_name, generator) in &self.generators {
            if generator.can_handle(&filtered_types) {
                let result = generator.generate(&filtered_types, &self.config).await?;
                results.insert(platform_name.clone(), result);
            }
        }

        Ok(results)
    }
}

/// TypeScript code generator
#[derive(Debug)]
pub struct TypeScriptGenerator {
    /// Type transformers
    transformers: HashMap<String, TypeTransformer>,

    /// Template cache
    template_cache: HashMap<String, String>,
}

/// Type transformer for converting Rust types to TypeScript
#[derive(Debug, Clone)]
struct TypeTransformer {
    /// Type mapping rules
    mappings: HashMap<String, String>,

    /// Custom transformation functions
    custom_transforms: Vec<String>,
}

impl Default for TypeTransformer {
    fn default() -> Self {
        let mut mappings = HashMap::new();

        // Standard Rust to TypeScript mappings
        mappings.insert("String".to_string(), "string".to_string());
        mappings.insert("str".to_string(), "string".to_string());
        mappings.insert("i8".to_string(), "number".to_string());
        mappings.insert("i16".to_string(), "number".to_string());
        mappings.insert("i32".to_string(), "number".to_string());
        mappings.insert("i64".to_string(), "number".to_string());
        mappings.insert("u8".to_string(), "number".to_string());
        mappings.insert("u16".to_string(), "number".to_string());
        mappings.insert("u32".to_string(), "number".to_string());
        mappings.insert("u64".to_string(), "number".to_string());
        mappings.insert("f32".to_string(), "number".to_string());
        mappings.insert("f64".to_string(), "number".to_string());
        mappings.insert("bool".to_string(), "boolean".to_string());
        mappings.insert("char".to_string(), "string".to_string());

        // Container types
        mappings.insert("Vec".to_string(), "Array".to_string());
        mappings.insert("HashMap".to_string(), "Record".to_string());
        mappings.insert("HashSet".to_string(), "Set".to_string());
        mappings.insert("BTreeMap".to_string(), "Record".to_string());
        mappings.insert("BTreeSet".to_string(), "Set".to_string());

        // Option and Result types
        mappings.insert("Option".to_string(), "undefined".to_string());
        mappings.insert("Result".to_string(), "any".to_string());

        Self {
            mappings,
            custom_transforms: vec![],
        }
    }
}

impl TypeTransformer {
    /// Transform a Rust type to TypeScript
    fn transform_type(&self, rust_type: &str) -> String {
        if let Some(mapped_type) = self.mappings.get(rust_type) {
            mapped_type.clone()
        } else {
            // Remove generic parameters and try again
            let base_type = rust_type.split('<').next().unwrap_or(rust_type);
            if let Some(mapped_type) = self.mappings.get(base_type) {
                mapped_type.clone()
            } else if rust_type.contains("Result<") {
                // Handle Result<T, E> types
                self.transform_result_type(rust_type)
            } else if rust_type.contains("Option<") {
                // Handle Option<T> types
                self.transform_option_type(rust_type)
            } else {
                // Default to 'any' for unknown types
                "any".to_string()
            }
        }
    }

    /// Transform Result<T, E> types
    fn transform_result_type(&self, rust_type: &str) -> String {
        if let Some(start) = rust_type.find('<') {
            if let Some(end) = rust_type.rfind('>') {
                let inner = &rust_type[start + 1..end];
                let types: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if types.len() >= 1 {
                    let success_type = self.transform_type(types[0]);
                    return success_type;
                }
            }
        }
        "any".to_string()
    }

    /// Transform Option<T> types
    fn transform_option_type(&self, rust_type: &str) -> String {
        if let Some(start) = rust_type.find('<') {
            if let Some(end) = rust_type.rfind('>') {
                let inner = &rust_type[start + 1..end];
                let inner_type = self.transform_type(inner);
                return format!("{} | undefined", inner_type);
            }
        }
        "undefined".to_string()
    }
}

impl TypeScriptGenerator {
    /// Create a new TypeScript generator
    pub fn new() -> Self {
        Self {
            transformers:   HashMap::new(),
            template_cache: HashMap::new(),
        }
    }

    /// Generate TypeScript interface
    fn generate_interface(&self, parsed_type: &ParsedType, config: &TypeScriptConfig) -> String {
        let mut output = String::new();

        // Add documentation comment
        if config.generate_docs {
            if let Some(ref docs) = parsed_type.documentation {
                output.push_str(&format!("/**\n * {}\n */\n", docs.replace("\n", "\n * ")));
            }
        }

        // Generate interface declaration
        output.push_str(&format!("export interface {} {{\n", parsed_type.name));

        // Generate fields
        for field in &parsed_type.fields {
            if config.generate_docs && field.documentation.is_some() {
                output.push_str(&format!(
                    "  /** {} */\n",
                    field.documentation.as_ref().unwrap()
                ));
            }

            let mut ts_type = self.transform_type(&field.ty);
            if !config.strict_null_checks && field.ty.contains("Option<") {
                // In non-strict mode, optional types don't need | undefined
                ts_type = ts_type.replace(" | undefined", "");
            }

            let optional_marker = if field.ty.contains("Option<") && config.strict_null_checks {
                ""
            } else {
                "?"
            };
            output.push_str(&format!(
                "  {}{}: {};\n",
                field.name, optional_marker, ts_type
            ));
        }

        output.push_str("}\n\n");
        output
    }

    /// Generate TypeScript type alias
    fn generate_type_alias(&self, parsed_type: &ParsedType, _config: &TypeScriptConfig) -> String {
        let ts_type = self.transform_type(&parsed_type.name); // This is approximate
        format!("export type {} = {};\n\n", parsed_type.name, ts_type)
    }

    /// Generate TypeScript union type from enum
    fn generate_enum(&self, parsed_type: &ParsedType, config: &TypeScriptConfig) -> String {
        let mut output = String::new();

        if config.generate_docs {
            if let Some(ref docs) = parsed_type.documentation {
                output.push_str(&format!("/**\n * {}\n */\n", docs.replace("\n", "\n * ")));
            }
        }

        // Check if enum has associated data
        let has_data = parsed_type.variants.iter().any(|v| !v.fields.is_empty());

        if has_data {
            // Generate union type for enum with data
            output.push_str(&format!("export type {} =\n", parsed_type.name));
            for (index, variant) in parsed_type.variants.iter().enumerate() {
                let separator = if index == parsed_type.variants.len() - 1 {
                    ";"
                } else {
                    " |"
                };

                if variant.fields.is_empty() {
                    output.push_str(&format!("  {{ type: '{}' }}{}\n", variant.name, separator));
                } else {
                    output.push_str(&format!("  {{ type: '{}', ", variant.name));
                    for (field_index, field) in variant.fields.iter().enumerate() {
                        let ts_type = self.transform_type(&field.ty);
                        let field_name = field
                            .name
                            .clone()
                            .unwrap_or_else(|| format!("field{}", field_index));
                        output.push_str(&format!("{}: {}", field_name, ts_type));
                        if field_index < variant.fields.len() - 1 {
                            output.push_str(", ");
                        }
                    }
                    output.push_str(&format!(" }}{}\n", separator));
                }
            }
        } else {
            // Generate simple union type for unit-only enum
            output.push_str(&format!("export type {} =\n", parsed_type.name));
            for (index, variant) in parsed_type.variants.iter().enumerate() {
                let separator = if index == parsed_type.variants.len() - 1 {
                    ";"
                } else {
                    " |"
                };
                output.push_str(&format!("  '{}'{}\n", variant.name, separator));
            }
        }

        output.push_str("\n");
        output
    }

    /// Transform a type to TypeScript
    fn transform_type(&self, rust_type: &str) -> String {
        let transformer = TypeTransformer::default();
        transformer.transform_type(rust_type)
    }
}

#[async_trait]
impl CodeGenerator for TypeScriptGenerator {
    async fn generate(
        &self,
        types: &[ParsedType],
        config: &GenerationConfig,
    ) -> Result<GeneratedCode, TypeGenerationError> {
        let mut content = String::new();
        let mut dependencies = Vec::new();

        // Add header comment
        content.push_str("// Generated by Rust AI IDE Shared Types\n");
        content.push_str("// Do not edit manually\n\n");

        // Process each type
        for parsed_type in types {
            match parsed_type.kind {
                TypeKind::Struct => {
                    content.push_str(&self.generate_interface(parsed_type, &config.typescript));
                }
                TypeKind::Enum => {
                    content.push_str(&self.generate_enum(parsed_type, &config.typescript));
                }
                TypeKind::TypeAlias => {
                    content.push_str(&self.generate_type_alias(parsed_type, &config.typescript));
                }
                TypeKind::Union => {
                    content.push_str("// Union types not fully supported yet\n");
                }
            }

            // Collect dependencies
            dependencies.extend(parsed_type.dependencies.clone());
        }

        // Generate index file if requested
        if config.typescript.generate_index {
            content.push_str(&self.generate_index_file(types));
        }

        let bytes_generated = content.len();

        Ok(GeneratedCode {
            content,
            target_platform: "typescript".to_string(),
            source_types: types.to_vec(),
            metadata: GenerationMetadata {
                generated_at:      chrono::Utc::now().to_rfc3339(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                config_snapshot:   serde_json::to_value(config).unwrap_or_default(),
                stats:             GenerationStats {
                    types_processed: types.len() as usize,
                    types_generated: types.len() as usize,
                    bytes_generated,
                    generation_time_ms: 0, // Would be set by caller
                    warnings_count: 0,
                    errors_count: 0,
                },
                status:            GenerationStatus::Success,
            },
            dependencies,
        })
    }

    fn target_platform(&self) -> String {
        "typescript".to_string()
    }

    fn can_handle(&self, _types: &[ParsedType]) -> bool {
        true // TypeScript generator can handle any types
    }
}

impl TypeScriptGenerator {
    /// Generate index file for exports
    fn generate_index_file(&self, types: &[ParsedType]) -> String {
        let mut output = String::new();
        output.push_str("// Index file for generated types\n\n");

        for parsed_type in types {
            output.push_str(&format!(
                "export type {{{}}} from './types';\n",
                parsed_type.name
            ));
            output.push_str(&format!(
                "export {{{}}} from './types';\n",
                parsed_type.name
            ));
        }

        output.push_str("\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::TypeParser;

    #[tokio::test]
    async fn test_typescript_struct_generation() {
        let source = r#"
            /// A test structure
            pub struct TestStruct {
                /// The name field
                pub name: String,
                /// The age field
                pub age: i32,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        let generator = TypeScriptGenerator::new();
        let config = GenerationConfig::default();
        let result = generator.generate(&types, &config).await.unwrap();

        assert!(result.content.contains("export interface TestStruct"));
        assert!(result.content.contains("name: string"));
        assert!(result.content.contains("age: number"));
        assert_eq!(result.target_platform, "typescript");
    }

    #[tokio::test]
    async fn test_typescript_enum_generation() {
        let source = r#"
            /// Test enum
            pub enum TestEnum {
                /// Variant A
                VariantA,
                /// Variant B with data
                VariantB(String),
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        let generator = TypeScriptGenerator::new();
        let config = GenerationConfig::default();
        let result = generator.generate(&types, &config).await.unwrap();

        assert!(result.content.contains("export type TestEnum"));
        assert!(result.content.contains("VariantA"));
        assert!(result.content.contains("VariantB"));
    }

    #[test]
    fn test_type_transformer() {
        let transformer = TypeTransformer::default();

        assert_eq!(transformer.transform_type("String"), "string");
        assert_eq!(transformer.transform_type("i32"), "number");
        assert_eq!(transformer.transform_type("bool"), "boolean");
        assert_eq!(transformer.transform_type("Vec<String>"), "Array");
        assert_eq!(
            transformer.transform_type("Option<String>").as_str(),
            "string | undefined"
        );
    }

    #[tokio::test]
    async fn test_type_generator_integration() {
        let generator = TypeGenerator::with_full_config(GenerationConfig::default()).unwrap();

        let source = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
                pub email: Option<String>,
            }
        "#;

        let result = generator
            .generate_types_from_source(source, "test.rs", &["User"])
            .await
            .unwrap();
        assert!(result.content.contains("export interface User"));
        assert!(result.content.contains("email?: string"));
    }
}
