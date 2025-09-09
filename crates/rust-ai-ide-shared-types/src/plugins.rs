//! Plugin system for custom type transformations
//!
//! This module provides a fully extensible plugin system that allows
//! third-party plugins to customize type transformations, add support
//! for new target platforms, and extend the generation capabilities.

//! Go plugin for generating Go structs
pub mod go_plugin;
//! GraphQL plugin for generating GraphQL schemas
pub mod graphql_plugin;
//! OpenAPI plugin for generating REST API specifications
pub mod openapi_plugin;

use crate::errors::PluginError;
use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Main plugin system that manages loaded plugins
#[derive(Debug, Clone)]
pub struct PluginSystem {
    /// Loaded transformer plugins
    transformers: HashMap<String, Arc<dyn TransformerPluginTrait>>,

    /// Loaded generator plugins
    generators: HashMap<String, Arc<dyn GeneratorPluginTrait>>,

    /// Plugin configurations
    configs: HashMap<String, serde_json::Value>,

    /// Plugin search directories
    search_dirs: Vec<PathBuf>,

    /// Plugin loading options
    options: PluginOptions,
}

/// Options for plugin loading and execution
#[derive(Debug, Clone)]
pub struct PluginOptions {
    /// Enable plugin system
    pub enabled: bool,

    /// Allow unsafe plugin operations
    pub allow_unsafe: bool,

    /// Plugin search directories
    pub search_dirs: Vec<PathBuf>,

    /// Enabled plugins (by name)
    pub enabled_plugins: Vec<String>,

    /// Plugin-specific timeouts (in seconds)
    pub timeouts: HashMap<String, u64>,
}

impl Default for PluginOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_unsafe: false,
            search_dirs: vec![
                PathBuf::from("plugins"),
                PathBuf::from("target/debug/plugins"),
                PathBuf::from("target/release/plugins"),
            ],
            enabled_plugins: vec![],
            timeouts: HashMap::new(),
        }
    }
}

/// Core trait for transformer plugins
#[async_trait]
pub trait TransformerPluginTrait: Send + Sync + std::fmt::Debug {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Get the plugin version
    fn version(&self) -> &str;

    /// Check if plugin supports the given platform
    fn supports_platform(&self, platform: &str) -> bool;

    /// Get supported file extensions for this plugin
    fn supported_extensions(&self) -> Vec<String>;

    /// Transform a single type
    async fn transform_type(
        &self,
        rust_type: &ParsedType,
        context: &TransformationContext,
    ) -> Result<Option<GeneratedCode>, PluginError>;

    /// Transform a field type
    async fn transform_field(
        &self,
        field_name: &str,
        field_type: &str,
        context: &TransformationContext,
    ) -> Result<Option<String>, PluginError>;

    /// Transform an entire file
    async fn transform_file(
        &self,
        types: &[ParsedType],
        context: &TransformationContext,
    ) -> Result<Option<GeneratedCode>, PluginError>;

    /// Validate plugin compatibility
    async fn validate(&self) -> Result<(), PluginError>;

    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
}

/// Trait for generator plugins that add new target platforms
#[async_trait]
pub trait GeneratorPluginTrait: Send + Sync + std::fmt::Debug {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Get the target platforms supported by this plugin
    fn target_platforms(&self) -> Vec<String>;

    /// Get supported input formats
    fn supported_formats(&self) -> Vec<String>;

    /// Generate code for the plugin's target platforms
    async fn generate(
        &self,
        types: &[ParsedType],
        platform: &str,
        config: &serde_json::Value,
    ) -> Result<GeneratedCode, PluginError>;

    /// Validate plugin compatibility
    async fn validate(&self) -> Result<(), PluginError>;

    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
}

/// Metadata about a plugin
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Author information
    pub author: String,

    /// Description
    pub description: String,

    /// Homepage or repository URL
    pub homepage: Option<String>,

    /// Supported platforms
    pub platforms: Vec<String>,

    /// License
    pub license: Option<String>,

    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Information about loaded plugin
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Plugin status
    pub status: PluginStatus,

    /// Load timestamp
    pub loaded_at: chrono::DateTime<chrono::Utc>,

    /// Plugin instance (if loaded)
    #[serde(skip)]
    pub instance: Option<PluginInstance>,
}

/// Plugin instance enum
pub enum PluginInstance {
    Transformer(Arc<dyn TransformerPluginTrait>),
    Generator(Arc<dyn GeneratorPluginTrait>),
}

/// Plugin status
#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    /// Plugin is loaded and ready
    Loaded,

    /// Plugin failed to load
    Failed(String),

    /// Plugin is disabled
    Disabled,

    /// Plugin has incompatible version
    Incompatible(String),
}

impl PluginSystem {
    /// Create a new plugin system with default options
    pub fn new() -> Self {
        Self {
            transformers: HashMap::new(),
            generators: HashMap::new(),
            configs: HashMap::new(),
            search_dirs: vec![
                PathBuf::from("plugins"),
                PathBuf::from("target/debug/plugins"),
                PathBuf::from("target/release/plugins"),
            ],
            options: PluginOptions::default(),
        }
    }

    /// Create a plugin system with custom options
    pub fn with_options(options: PluginOptions) -> Self {
        Self {
            search_dirs: options.search_dirs.clone(),
            options,
            transformers: HashMap::new(),
            generators: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Load all available plugins
    pub async fn load_plugins(&mut self) -> Result<Vec<LoadedPlugin>, PluginError> {
        let mut loaded_plugins = Vec::new();

        // Load dynamic plugins (when plugins feature is enabled)
        #[cfg(feature = "plugins")]
        {
            for search_dir in &self.search_dirs {
                if search_dir.exists() {
                    let dir_plugins = self.load_plugins_from_directory(search_dir).await?;
                    loaded_plugins.extend(dir_plugins);
                }
            }
        }

        // Load built-in plugins
        loaded_plugins.extend(self.load_builtin_plugins().await?);

        // Filter enabled plugins only
        if !self.options.enabled_plugins.is_empty() {
            loaded_plugins.retain(|p| self.options.enabled_plugins.contains(&p.metadata.name));
        }

        // Register loaded plugins
        for plugin in &loaded_plugins {
            if plugin.status == PluginStatus::Loaded {
                if let Some(instance) = &plugin.instance {
                    match instance {
                        PluginInstance::Transformer(transformer) => {
                            self.transformers.insert(plugin.metadata.name.clone(), transformer.clone());
                        }
                        PluginInstance::Generator(generator) => {
                            self.generators.insert(plugin.metadata.name.clone(), generator.clone());
                        }
                    }
                }
            }
        }

        Ok(loaded_plugins)
    }

    /// Unload all plugins
    pub async fn unload_plugins(&mut self) -> Result<(), PluginError> {
        self.transformers.clear();
        self.generators.clear();
        Ok(())
    }

    /// Get available transformer plugins
    pub fn get_transformer_plugins(&self) -> Vec<Arc<dyn TransformerPluginTrait>> {
        self.transformers.values().cloned().collect()
    }

    /// Get available generator plugins
    pub fn get_generator_plugins(&self) -> Vec<Arc<dyn GeneratorPluginTrait>> {
        self.generators.values().cloned().collect()
    }

    /// Get a specific transformer plugin by name
    pub fn get_transformer(&self, name: &str) -> Option<Arc<dyn TransformerPluginTrait>> {
        self.transformers.get(name).cloned()
    }

    /// Get a specific generator plugin by name
    pub fn get_generator(&self, name: &str) -> Option<Arc<dyn GeneratorPluginTrait>> {
        self.generators.get(name).cloned()
    }

    /// Apply all transformer plugins to a type
    pub async fn apply_transformers(
        &self,
        rust_type: &ParsedType,
        context: &TransformationContext,
    ) -> Result<Option<GeneratedCode>, PluginError> {
        for transformer in self.transformers.values() {
            if transformer.supports_platform(&context.target_platform) {
                match transformer.transform_type(rust_type, context).await {
                    Ok(Some(result)) => return Ok(Some(result)),
                    Ok(None) => continue,
                    Err(e) => {
                        // Log error but continue with other transformers
                        eprintln!("Transformer {} failed: {}", transformer.name(), e);
                        continue;
                    }
                }
            }
        }
        Ok(None)
    }

    /// Load plugins from a directory (for dynamic loading)
    #[cfg(feature = "plugins")]
    async fn load_plugins_from_directory(&self, dir: &PathBuf) -> Result<Vec<LoadedPlugin>, PluginError> {
        use libloading::{Library, Symbol};

        let mut plugins = Vec::new();

        for entry in std::fs::read_dir(dir).map_err(|e| PluginError::LoadError(e.to_string()))? {
            let entry = entry.map_err(|e| PluginError::LoadError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("so") ||
               path.extension().and_then(|s| s.to_str()) == Some("dll") ||
               path.extension().and_then(|s| s.to_str()) == Some("dylib") {

                match self.load_dynamic_plugin(&path).await {
                    Ok(plugin) => plugins.push(plugin),
                    Err(e) => {
                        plugins.push(LoadedPlugin {
                            metadata: PluginMetadata {
                                name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                                version: "unknown".to_string(),
                                author: "unknown".to_string(),
                                description: format!("Dynamic plugin failed to load: {}", e),
                                homepage: None,
                                platforms: vec![],
                                license: None,
                                dependencies: vec![],
                            },
                            status: PluginStatus::Failed(e.to_string()),
                            loaded_at: chrono::Utc::now(),
                            instance: None,
                        });
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Load a single dynamic plugin
    #[cfg(feature = "plugins")]
    async fn load_dynamic_plugin(&self, path: &PathBuf) -> Result<LoadedPlugin, PluginError> {
        use libloading::{Library, Symbol};

        // Implementation would load the dynamic library and instantiate the plugin
        // This is a simplified placeholder for the concept
        Err(PluginError::LoadError("Dynamic plugin loading not fully implemented".to_string()))
    }

    /// Load built-in plugins
    async fn load_builtin_plugins(&self) -> Result<Vec<LoadedPlugin>, PluginError> {
        let mut plugins = Vec::new();

        // Register built-in transformer plugins
        let json_transformer = JsonTransformerPlugin::new();
        plugins.push(LoadedPlugin {
            metadata: json_transformer.metadata(),
            status: PluginStatus::Loaded,
            loaded_at: chrono::Utc::now(),
            instance: Some(PluginInstance::Transformer(Arc::new(json_transformer))),
        });

        // Register built-in generator plugins
        let python_generator = PythonGeneratorPlugin::new();
        plugins.push(LoadedPlugin {
            metadata: python_generator.metadata(),
            status: PluginStatus::Loaded,
            loaded_at: chrono::Utc::now(),
            instance: Some(PluginInstance::Generator(Arc::new(python_generator))),
        });

        // Register Go generator plugin
        let go_generator = crate::plugins::go_plugin::GoGeneratorPlugin::new();
        plugins.push(LoadedPlugin {
            metadata: go_generator.metadata(),
            status: PluginStatus::Loaded,
            loaded_at: chrono::Utc::now(),
            instance: Some(PluginInstance::Generator(Arc::new(go_generator))),
        });

        // Register GraphQL generator plugin
        let graphql_generator = crate::plugins::graphql_plugin::GraphQLGeneratorPlugin::new();
        plugins.push(LoadedPlugin {
            metadata: graphql_generator.metadata(),
            status: PluginStatus::Loaded,
            loaded_at: chrono::Utc::now(),
            instance: Some(PluginInstance::Generator(Arc::new(graphql_generator))),
        });

        // Register OpenAPI generator plugin
        let openapi_generator = crate::plugins::openapi_plugin::OpenAPIGeneratorPlugin::new();
        plugins.push(LoadedPlugin {
            metadata: openapi_generator.metadata(),
            status: PluginStatus::Loaded,
            loaded_at: chrono::Utc::now(),
            instance: Some(PluginInstance::Generator(Arc::new(openapi_generator))),
        });

        Ok(plugins)
    }

    /// Set configuration for a plugin
    pub fn configure_plugin(&mut self, plugin_name: &str, config: serde_json::Value) {
        self.configs.insert(plugin_name.to_string(), config);
    }

    /// Get plugin configuration
    pub fn get_plugin_config(&self, plugin_name: &str) -> Option<&serde_json::Value> {
        self.configs.get(plugin_name)
    }
}

// Built-in Transformer Plugins

/// JSON Schema transformer plugin
#[derive(Debug)]
pub struct JsonTransformerPlugin;

impl JsonTransformerPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TransformerPluginTrait for JsonTransformerPlugin {
    fn name(&self) -> &str { "json-schema-transformer" }

    fn version(&self) -> &str { "1.0.0" }

    fn supports_platform(&self, platform: &str) -> bool {
        platform == "json-schema"
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["json".to_string()]
    }

    async fn transform_type(
        &self,
        rust_type: &ParsedType,
        _context: &TransformationContext,
    ) -> Result<Option<GeneratedCode>, PluginError> {
        // Generate JSON Schema from Rust type
        let schema = self.generate_json_schema(rust_type)?;
        let json = serde_json::to_string_pretty(&schema)
            .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

        Ok(Some(GeneratedCode {
            content: json,
            target_platform: "json-schema".to_string(),
            source_types: vec![rust_type.clone()],
            metadata: GenerationMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                config_snapshot: serde_json::Value::Null,
                stats: GenerationStats {
                    types_processed: 1,
                    types_generated: 1,
                    bytes_generated: json.len(),
                    generation_time_ms: 0,
                    warnings_count: 0,
                    errors_count: 0,
                },
                status: crate::generation::GenerationStatus::Success,
            },
            dependencies: vec![],
        }))
    }

    async fn transform_field(
        &self,
        _field_name: &str,
        _field_type: &str,
        _context: &TransformationContext,
    ) -> Result<Option<String>, PluginError> {
        Ok(None) // Field-level transformation not implemented for JSON Schema
    }

    async fn transform_file(
        &self,
        types: &[ParsedType],
        _context: &TransformationContext,
    ) -> Result<Option<GeneratedCode>, PluginError> {
        // Generate combined JSON Schema
        let mut combined_schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {},
            "required": []
        });

        for rust_type in types {
            let schema = self.generate_json_schema(rust_type)?;
            if let Some(properties) = combined_schema["properties"].as_object_mut() {
                properties.insert(rust_type.name.clone(), schema);
            }
        }

        let json = serde_json::to_string_pretty(&combined_schema)
            .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

        Ok(Some(GeneratedCode {
            content: json,
            target_platform: "json-schema".to_string(),
            source_types: types.to_vec(),
            metadata: GenerationMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                config_snapshot: serde_json::Value::Null,
                stats: GenerationStats {
                    types_processed: types.len() as usize,
                    types_generated: 1,
                    bytes_generated: json.len(),
                    generation_time_ms: 0,
                    warnings_count: 0,
                    errors_count: 0,
                },
                status: crate::generation::GenerationStatus::Success,
            },
            dependencies: vec![],
        }))
    }

    async fn validate(&self) -> Result<(), PluginError> {
        Ok(()) // JSON Schema plugin is always valid
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "json-schema-transformer".to_string(),
            version: "1.0.0".to_string(),
            author: "Rust AI IDE Team".to_string(),
            description: "Transforms Rust types into JSON Schema format".to_string(),
            homepage: Some("https://github.com/rust-ai-ide/rust-ai-ide".to_string()),
            platforms: vec!["json-schema".to_string()],
            license: Some("MIT OR Apache-2.0".to_string()),
            dependencies: vec![],
        }
    }
}

impl JsonTransformerPlugin {
    fn generate_json_schema(&self, rust_type: &ParsedType) -> Result<serde_json::Value, PluginError> {
        let mut schema = serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        });

        if let Some(properties) = schema["properties"].as_object_mut() {
            for field in &rust_type.fields {
                let field_schema = self.generate_field_schema(&field.ty);
                properties.insert(field.name.clone(), field_schema);
            }
        }

        // Add required fields
        if let Some(required) = schema["required"].as_array_mut() {
            for field in &rust_type.fields {
                if !field.ty.contains("Option<") {  // Non-optional fields
                    required.push(serde_json::Value::String(field.name.clone()));
                }
            }
        }

        Ok(schema)
    }

    fn generate_field_schema(&self, field_type: &str) -> serde_json::Value {
        let (json_type, format) = match field_type {
            t if t == "String" || t.starts_with("String") => ("string", None),
            t if t == "i32" || t == "i64" || t == "u32" || t == "u64" => ("integer", None),
            t if t == "f32" || t == "f64" => ("number", None),
            t if t == "bool" => ("boolean", None),
            t if t.contains("Vec<") => ("array", None),
            t if t.contains("Option<") => {
                let inner = t.trim_start_matches("Option<").trim_end_matches(">");
                let inner_schema = self.generate_field_schema(inner);
                return serde_json::json!({
                    "oneOf": [
                        inner_schema,
                        {"type": "null"}
                    ]
                });
            }
            _ => ("object", None), // Default to object for unknown types
        };

        let mut schema = serde_json::json!({"type": json_type});

        if let Some(fmt) = format {
            schema["format"] = serde_json::Value::String(fmt.to_string());
        }

        schema
    }
}

// Built-in Generator Plugins

/// Python generator plugin
#[derive(Debug)]
pub struct PythonGeneratorPlugin;

impl PythonGeneratorPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GeneratorPluginTrait for PythonGeneratorPlugin {
    fn name(&self) -> &str { "python-generator" }

    fn target_platforms(&self) -> Vec<String> {
        vec!["python".to_string(), "python-dataclasses".to_string()]
    }

    fn supported_formats(&self) -> Vec<String> {
        vec!["dataclass".to_string(), "pydantic".to_string(), "typeddict".to_string()]
    }

    async fn generate(
        &self,
        types: &[ParsedType],
        platform: &str,
        config: &serde_json::Value,
    ) -> Result<GeneratedCode, PluginError> {
        let format = config.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("dataclass");

        let mut content = String::new();
        content.push_str("# Generated Python types\n");
        content.push_str("# Do not edit manually\n\n");

        match format {
            "dataclass" => self.generate_dataclasses(&mut content, types)?,
            "pydantic" => self.generate_pydantic(&mut content, types)?,
            "typeddict" => self.generate_typeddict(&mut content, types)?,
            _ => return Err(PluginError::ExecutionError(format!("Unsupported format: {}", format))),
        }

        Ok(GeneratedCode {
            content,
            target_platform: platform.to_string(),
            source_types: types.to_vec(),
            metadata: GenerationMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                config_snapshot: config.clone(),
                stats: GenerationStats {
                    types_processed: types.len(),
                    types_generated: types.len(),
                    bytes_generated: content.len(),
                    generation_time_ms: 0,
                    warnings_count: 0,
                    errors_count: 0,
                },
                status: crate::generation::GenerationStatus::Success,
            },
            dependencies: vec!["typing".to_string()],
        })
    }

    async fn validate(&self) -> Result<(), PluginError> {
        Ok(()) // Python generator is always valid
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "python-generator".to_string(),
            version: "1.0.0".to_string(),
            author: "Rust AI IDE Team".to_string(),
            description: "Generates Python type definitions from Rust types".to_string(),
            homepage: Some("https://github.com/rust-ai-ide/rust-ai-ide".to_string()),
            platforms: self.target_platforms(),
            license: Some("MIT OR Apache-2.0".to_string()),
            dependencies: vec![],
        }
    }
}

impl PythonGeneratorPlugin {
    fn generate_dataclasses(&self, content: &mut String, types: &[ParsedType]) -> Result<(), PluginError> {
        content.push_str("from dataclasses import dataclass\nfrom typing import Optional, List, Dict\n\n");

        for rust_type in types {
            if let Some(ref docs) = rust_type.documentation {
                content.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", docs.replace("\n", "\n")));
            }

            content.push_str("@dataclass\n");
            content.push_str(&format!("class {}:\n", rust_type.name));

            for field in &rust_type.fields {
                let py_type = self.rust_to_python_type(&field.ty);
                content.push_str(&format!("    {}: {}\n", field.name, py_type));
            }
            content.push_str("\n");
        }
        Ok(())
    }

    fn generate_pydantic(&self, content: &mut String, types: &[ParsedType]) -> Result<(), PluginError> {
        content.push_str("from pydantic import BaseModel\nfrom typing import Optional, List, Dict\n\n");

        for rust_type in types {
            if let Some(ref docs) = rust_type.documentation {
                content.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", docs.replace("\n", "\n")));
            }

            content.push_str(&format!("class {}(BaseModel):\n", rust_type.name));

            for field in &rust_type.fields {
                let py_type = self.rust_to_python_type(&field.ty);
                content.push_str(&format!("    {}: {}\n", field.name, py_type));
            }
            content.push_str("\n");
        }
        Ok(())
    }

    fn generate_typeddict(&self, content: &mut String, types: &[ParsedType]) -> Result<(), PluginError> {
        content.push_str("from typing import TypedDict, Optional, List, Dict\n\n");

        for rust_type in types {
            if let Some(ref docs) = rust_type.documentation {
                content.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", docs.replace("\n", "\n")));
            }

            content.push_str(&format!("class {}TypedDict(TypedDict, total=False):\n", rust_type.name));

            for field in &rust_type.fields {
                let py_type = self.rust_to_python_type(&field.ty);
                if field.ty.contains("Option<") {
                    content.push_str(&format!("    {}: {}\n", field.name, py_type));
                } else {
                    content.push_str(&format!("    {}: {}\n", field.name, py_type));
                }
            }
            content.push_str("\n");
        }
        Ok(())
    }

    fn rust_to_python_type(&self, rust_type: &str) -> String {
        match rust_type {
            t if t == "String" => "str".to_string(),
            t if t == "i32" || t == "i64" || t == "u32" || t == "u64" => "int".to_string(),
            t if t == "f32" || t == "f64" => "float".to_string(),
            t if t == "bool" => "bool".to_string(),
            t if t.contains("Option<") => {
                let inner = t.trim_start_matches("Option<").trim_end_matches(">");
                format!("Optional[{}]", self.rust_to_python_type(inner))
            }
            t if t.contains("Vec<") => {
                let inner = t.trim_start_matches("Vec<").trim_end_matches(">");
                format!("List[{}]", self.rust_to_python_type(inner))
            }
            t if t.contains("HashMap<") => {
                format!("Dict[Any, Any]") // Simplified for now
            }
            _ => "Any".to_string(), // Unknown types default to Any
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::TypeParser;

    #[test]
    fn test_plugin_system_creation() {
        let plugin_system = PluginSystem::new();
        assert!(plugin_system.transformers.is_empty());
        assert!(plugin_system.generators.is_empty());
    }

    #[tokio::test]
    async fn test_load_builtin_plugins() {
        let mut plugin_system = PluginSystem::new();
        let plugins = plugin_system.load_plugins().await.unwrap();

        assert!(!plugins.is_empty());
        // Should have at least the JSON transformer and Python generator
        assert!(plugins.len() >= 2);
    }

    #[tokio::test]
    async fn test_json_schema_transformer() {
        let transformer = JsonTransformerPlugin::new();

        let source = r#"
            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub name: String,
                pub age: Option<i32>,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        let context = TransformationContext {
            source_platform: "rust".to_string(),
            target_platform: "json-schema".to_string(),
            type_mappings: HashMap::new(),
            rules: HashMap::new(),
            options: HashMap::new(),
        };

        let result = transformer.transform_type(&types[0], &context).await.unwrap();

        assert!(result.is_some());
        let generated = result.unwrap();
        assert!(generated.content.contains("User"));
        assert!(generated.target_platform == "json-schema");

        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&generated.content).unwrap();
    }

    #[tokio::test]
    async fn test_python_generator() {
        let generator = PythonGeneratorPlugin::new();

        let source = r#"
            pub struct Person {
                pub name: String,
                pub age: i32,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        let config = serde_json::json!({"format": "dataclass"});
        let result = generator.generate(&types, "python-dataclasses", &config).await.unwrap();

        assert!(result.content.contains("@dataclass"));
        assert!(result.content.contains("class Person"));
        assert!(result.content.contains("name: str"));
        assert!(result.content.contains("age: int"));
    }
}