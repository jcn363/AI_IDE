//! Plugin system for extensible DSL templates

use crate::ast::Template;
use crate::error::{DslError, DslResult};
use crate::types::*;
use async_trait::async_trait;
use rust_ai_ide_common::ProgrammingLanguage;
use serde_json;
use std::collections::HashMap;

/// Plugin manager for handling DSL plugins
#[derive(Debug)]
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn DslPlugin>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin
    pub async fn register_plugin(&mut self, plugin: Box<dyn DslPlugin>) -> DslResult<()> {
        let id = plugin.id().to_string();

        if self.plugins.contains_key(&id) {
            return Err(DslError::plugin(
                id,
                "Plugin already registered".to_string(),
            ));
        }

        self.plugins.insert(id, plugin);
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> bool {
        self.plugins.remove(plugin_id).is_some()
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&dyn DslPlugin> {
        self.plugins.get(plugin_id).map(|p| p.as_ref())
    }

    /// List all registered plugin IDs
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Find plugins that support specific templates
    pub fn find_plugins_for_template(&self, template_name: &str) -> Vec<String> {
        self.plugins
            .iter()
            .filter(|(_, plugin)| {
                plugin
                    .supported_templates()
                    .contains(&template_name.to_string())
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Create a template using available plugins
    pub async fn create_template(
        &self,
        template_name: &str,
        ast: &Template,
    ) -> DslResult<Box<dyn DslTemplate>> {
        // First try to find a plugin that supports this template
        for (_, plugin) in &self.plugins {
            if plugin
                .supported_templates()
                .contains(&template_name.to_string())
            {
                return plugin.create_template(template_name, ast).await;
            }
        }

        // If no plugin found, return error
        Err(DslError::template(
            template_name.to_string(),
            "No plugin found for template".to_string(),
        ))
    }

    /// Validate parameters using plugin validators
    pub async fn validate_plugin_parameters(
        &self,
        template_name: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> DslResult<()> {
        // Collect validation errors from all plugins that support this template
        let mut errors = Vec::new();

        for (_, plugin) in &self.plugins {
            if plugin
                .supported_templates()
                .contains(&template_name.to_string())
            {
                if let Err(e) = plugin.validate_parameters(template_name, params).await {
                    errors.push(e.to_string());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(DslError::validation(
                template_name.to_string(),
                format!("Plugin validation failed: {}", errors.join(", ")),
            ))
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic template plugin that creates executable templates
#[derive(Debug)]
pub struct BasicTemplatePlugin {
    plugin_id: String,
    supported_templates: Vec<String>,
}

impl BasicTemplatePlugin {
    /// Create a new basic template plugin
    pub fn new(plugin_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            supported_templates: Vec::new(),
        }
    }

    /// Add support for a template type
    pub fn with_template(mut self, template_name: impl Into<String>) -> Self {
        self.supported_templates.push(template_name.into());
        self
    }

    /// Create with common template types
    pub fn with_common_templates() -> Self {
        Self::new("basic_templates")
            .with_template("function")
            .with_template("class")
            .with_template("struct")
            .with_template("test")
    }
}

#[async_trait]
impl DslPlugin for BasicTemplatePlugin {
    fn id(&self) -> &str {
        &self.plugin_id
    }

    fn name(&self) -> &str {
        "Basic Template Plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn supported_templates(&self) -> Vec<String> {
        self.supported_templates.clone()
    }

    async fn create_template(
        &self,
        template_name: &str,
        ast: &crate::ast::Template,
    ) -> DslResult<Box<dyn DslTemplate>> {
        if !self
            .supported_templates
            .contains(&template_name.to_string())
        {
            return Err(DslError::plugin(
                self.id().to_string(),
                format!("Template '{}' not supported by this plugin", template_name),
            ));
        }

        // Create a basic executable template
        let executable = Box::new(super::template::ExecutableTemplate::new(ast.clone()));
        Ok(executable)
    }

    async fn validate_parameters(
        &self,
        _template_name: &str,
        _params: &HashMap<String, serde_json::Value>,
    ) -> DslResult<()> {
        // Basic plugin doesn't perform additional validation
        // This can be extended by specific plugins
        Ok(())
    }
}

/// Language-specific plugin base
#[derive(Debug)]
pub struct LanguageTemplatePlugin {
    plugin_id: String,
    language: ProgrammingLanguage,
    supported_templates: Vec<String>,
    plugin_name: String,
}

impl LanguageTemplatePlugin {
    /// Create a new language-specific plugin
    pub fn new(plugin_id: impl Into<String>, language: ProgrammingLanguage) -> Self {
        let plugin_id_str = plugin_id.into();
        Self {
            plugin_id: plugin_id_str.clone(),
            language: language.clone(),
            supported_templates: Vec::new(),
            plugin_name: format!("{} Template Plugin", language_name(&language)),
        }
    }

    /// Add template support
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.supported_templates.push(template.into());
        self
    }

    /// Create TypeScript-specific plugin
    pub fn typescript() -> Self {
        Self::new("typescript_plugin", ProgrammingLanguage::TypeScript)
            .with_template("react_component")
            .with_template("typescript_interface")
            .with_template("web_ui_handler")
    }

    /// Create Rust-specific plugin
    pub fn rust() -> Self {
        Self::new("rust_plugin", ProgrammingLanguage::Rust)
            .with_template("struct_impl")
            .with_template("async_function")
            .with_template("error_handling")
    }
}

#[async_trait]
impl DslPlugin for LanguageTemplatePlugin {
    fn id(&self) -> &str {
        &self.plugin_id
    }

    fn name(&self) -> &str {
        &self.plugin_name
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn supported_templates(&self) -> Vec<String> {
        self.supported_templates.clone()
    }

    async fn create_template(
        &self,
        template_name: &str,
        ast: &crate::ast::Template,
    ) -> DslResult<Box<dyn DslTemplate>> {
        if !self
            .supported_templates
            .contains(&template_name.to_string())
        {
            return Err(DslError::plugin(
                self.id().to_string(),
                format!(
                    "Template '{}' not supported by {}",
                    template_name,
                    language_name(&self.language)
                ),
            ));
        }

        // Language-aware template creation
        let executable = Box::new(super::template::ExecutableTemplate::new(ast.clone()));
        Ok(executable)
    }

    async fn validate_parameters(
        &self,
        template_name: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> DslResult<()> {
        // Language-specific parameter validation can be added here
        // For now, basic validation
        let _ = (template_name, params); // Prevent unused variable warnings
        Ok(())
    }
}

/// Convert programming language to readable name
fn language_name(language: &ProgrammingLanguage) -> String {
    match language {
        ProgrammingLanguage::Rust => "Rust".to_string(),
        ProgrammingLanguage::TypeScript => "TypeScript".to_string(),
        ProgrammingLanguage::JavaScript => "JavaScript".to_string(),
        ProgrammingLanguage::Python => "Python".to_string(),
        ProgrammingLanguage::Java => "Java".to_string(),
        ProgrammingLanguage::CSharp => "C#".to_string(),
        ProgrammingLanguage::Go => "Go".to_string(),
        ProgrammingLanguage::Cpp => "C++".to_string(),
        ProgrammingLanguage::C => "C".to_string(),
        ProgrammingLanguage::Unknown => "Unknown".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BasicTemplatePlugin::with_common_templates());

        assert!(manager.register_plugin(plugin).await.is_ok());
        assert_eq!(manager.list_plugins().len(), 1);
        assert!(manager.get_plugin("basic_templates").is_some());
    }

    #[test]
    fn test_plugin_template_finding() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(
            BasicTemplatePlugin::new("test_plugin")
                .with_template("function")
                .with_template("class"),
        );

        // Since this is runtime and we can't easily await in sync test, we'll use runtime
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            manager.register_plugin(plugin).await.unwrap();

            let plugins = manager.find_plugins_for_template("function");
            assert_eq!(plugins.len(), 1);
            assert_eq!(plugins[0], "test_plugin");

            let no_plugins = manager.find_plugins_for_template("unknown_template");
            assert_eq!(no_plugins.len(), 0);
        });
    }

    #[test]
    fn test_language_plugin_creation() {
        let ts_plugin = LanguageTemplatePlugin::typescript();
        assert_eq!(ts_plugin.language, ProgrammingLanguage::TypeScript);
        assert!(ts_plugin
            .supported_templates
            .contains(&"react_component".to_string()));

        let rust_plugin = LanguageTemplatePlugin::rust();
        assert_eq!(rust_plugin.language, ProgrammingLanguage::Rust);
        assert!(rust_plugin
            .supported_templates
            .contains(&"struct_impl".to_string()));
    }

    #[test]
    fn test_plugin_id_conflicts() {
        let mut manager = PluginManager::new();
        let plugin1 = Box::new(BasicTemplatePlugin::new("duplicate_id"));
        let plugin2 = Box::new(BasicTemplatePlugin::new("duplicate_id"));

        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            assert!(manager.register_plugin(plugin1).await.is_ok());

            // Second registration with same ID should fail
            assert!(manager.register_plugin(plugin2).await.is_err());
        });
    }
}
