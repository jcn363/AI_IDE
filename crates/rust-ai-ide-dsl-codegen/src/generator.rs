//! Main DSL code generation engine

use std::collections::HashMap;

use async_trait::async_trait;
use rust_ai_ide_cache::{Cache, InMemoryCache};
use rust_ai_ide_common::ProgrammingLanguage;

use crate::ast::Template;
use crate::error::{DslError, DslResult};
use crate::parser::DslDocumentParser;
use crate::plugins::PluginManager;
use crate::template::{ExecutableTemplate, TemplateRegistry};
use crate::types::*;

/// Main DSL generation engine
pub struct DslCodeGenerator {
    /// Template registry for managing loaded templates
    template_registry: TemplateRegistry,
    /// Plugin manager for extensibility
    plugin_manager:    PluginManager,
    /// Global cache for performance optimization
    cache:             Option<std::sync::Arc<dyn Cache<String, serde_json::Value>>>,
    /// Generation configuration
    config:            GenerationConfig,
}

impl DslCodeGenerator {
    /// Create a new DSL code generator
    pub fn new() -> Self {
        Self {
            template_registry: TemplateRegistry::new(),
            plugin_manager:    PluginManager::new(),
            cache:             Some(std::sync::Arc::new(InMemoryCache::new(
                &rust_ai_ide_cache::CacheConfig::default(),
            ))),
            config:            GenerationConfig::default(),
        }
    }

    /// Create generator with custom configuration
    pub fn with_config(config: GenerationConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Load templates from DSL source code
    pub async fn load_dsl(&mut self, dsl_source: &str) -> DslResult<()> {
        let document = DslDocumentParser::parse(dsl_source)?;

        for template_ast in document.templates {
            let template = self.create_executable_template(template_ast).await?;
            self.template_registry.register(template)?;
        }

        Ok(())
    }

    /// Load a single template from DSL
    pub async fn load_template(&mut self, dsl_template: &str) -> DslResult<String> {
        let template_ast = DslDocumentParser::parse_template_str(dsl_template)?;
        let template = self.create_executable_template(template_ast).await?;
        let name = template.name().to_string();

        self.template_registry.register(template)?;
        Ok(name)
    }

    /// Execute a loaded template
    pub async fn execute_template(
        &self,
        template_name: &str,
        params: HashMap<String, serde_json::Value>,
        language: ProgrammingLanguage,
    ) -> DslResult<GeneratedCode> {
        let template = self
            .template_registry
            .get(template_name)
            .ok_or_else(|| DslError::template(template_name.to_string(), "Template not found".to_string()))?;

        let context = self.create_generation_context(language.clone());

        // Check cache first if enabled
        if let Some(cache) = &self.cache {
            let cache_key = self.generate_cache_key(template_name, &params, &language);
            if let Ok(Some(cached_result)) = cache.get(&cache_key).await {
                // Return cached result if available
                if let Ok(generated) = serde_json::from_value::<GeneratedCode>(cached_result) {
                    return Ok(generated);
                }
            }
        }

        // Execute template
        let result = template.execute(&params, &context).await?;

        // Cache result if caching is enabled
        if let Some(cache) = &self.cache {
            let cache_key = self.generate_cache_key(template_name, &params, &language);
            if let Ok(cached_value) = serde_json::to_value(&result) {
                let _ = cache
                    .insert(
                        cache_key,
                        cached_value,
                        Some(std::time::Duration::from_secs(300)), // 5 minute TTL
                    )
                    .await;
            }
        }

        Ok(result)
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.template_registry.list_templates()
    }

    /// Get template information
    pub async fn template_info(&self, name: &str) -> DslResult<TemplateInfo> {
        let template = self
            .template_registry
            .get(name)
            .ok_or_else(|| DslError::template(name.to_string(), "Template not found".to_string()))?;

        Ok(TemplateInfo {
            name:                template.name().to_string(),
            description:         template.description().map(|s| s.to_string()),
            supported_languages: template.supported_languages(),
            parameters:          template.parameters().to_vec(),
            patterns:            Vec::new(), // Will be populated when AST is enhanced
            version:             Some("0.1.0".to_string()),
        })
    }

    /// Register a DSL plugin
    pub async fn register_plugin(&mut self, plugin: Box<dyn DslPlugin>) -> DslResult<()> {
        self.plugin_manager.register_plugin(plugin).await
    }

    /// Generate code from DSL string directly
    pub async fn generate_from_dsl(
        &mut self,
        dsl_code: &str,
        params: HashMap<String, serde_json::Value>,
        language: ProgrammingLanguage,
    ) -> DslResult<GeneratedCode> {
        self.load_dsl(dsl_code).await?;

        // Get the first template (for single-template DSL strings)
        let template_names = self.list_templates();
        let template_name = template_names.first().ok_or_else(|| DslError::Execution {
            template: "DSL".to_string(),
            message:  "No templates found in DSL".to_string(),
            context:  None,
        })?;

        self.execute_template(template_name, params, language).await
    }

    /// Clear all caches
    pub async fn clear_cache(&self) -> DslResult<()> {
        if let Some(cache) = &self.cache {
            cache.clear().await.map_err(|e| DslError::Internal {
                message: e.to_string(),
            })?;
        }
        Ok(())
    }

    /// Get generation statistics
    pub async fn generation_stats(&self) -> DslResult<DslStats> {
        let template_count = self.template_registry.list_templates().len();
        let plugin_count = self.plugin_manager.list_plugins().len();

        // Cache statistics
        let (cache_entries, hit_ratio) = if let Some(cache) = &self.cache {
            let stats = cache.stats().await;
            (stats.total_entries as u32, stats.hit_ratio as f32)
        } else {
            (0, 0.0)
        };

        Ok(DslStats {
            template_count,
            plugin_count,
            cache_entries,
            cache_hit_ratio: hit_ratio,
        })
    }

    // Private helper methods
    async fn create_executable_template(&self, ast: Template) -> DslResult<ExecutableTemplate> {
        Ok(ExecutableTemplate::new(ast))
    }

    fn create_generation_context(&self, language: ProgrammingLanguage) -> GenerationContext {
        GenerationContext {
            language,
            config: self.config.clone(),
            workspace_root: std::env::current_dir().unwrap_or_default(),
            ai_analysis: None, // Will be populated in Phase 3
            cache: self.cache.clone(),
        }
    }

    fn generate_cache_key(
        &self,
        template_name: &str,
        params: &HashMap<String, serde_json::Value>,
        language: &ProgrammingLanguage,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        template_name.hash(&mut hasher);
        language.hash(&mut hasher);

        // Hash parameters
        let params_str = serde_json::to_string(params).unwrap_or_default();
        params_str.hash(&mut hasher);

        format!("dsl_{:x}", hasher.finish())
    }
}

impl Default for DslCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generation statistics
#[derive(Debug, Clone)]
pub struct DslStats {
    pub template_count:  usize,
    pub plugin_count:    usize,
    pub cache_entries:   u32,
    pub cache_hit_ratio: f32,
}

#[async_trait]
impl DslEngine for DslCodeGenerator {
    async fn register_plugin(&mut self, plugin: Box<dyn DslPlugin>) -> DslResult<()> {
        self.plugin_manager.register_plugin(plugin).await
    }

    async fn load_template(&self, dsl_source: &str) -> DslResult<Box<dyn DslTemplate>> {
        let ast = DslDocumentParser::parse_template_str(dsl_source)?;
        let template = ExecutableTemplate::new(ast);
        Ok(Box::new(template))
    }

    async fn execute_template(
        &self,
        template_name: &str,
        params: HashMap<String, serde_json::Value>,
        context: &GenerationContext,
    ) -> DslResult<GeneratedCode> {
        let template = self
            .template_registry
            .get(template_name)
            .ok_or_else(|| DslError::template(template_name.to_string(), "Template not found".to_string()))?;

        template.execute(&params, context).await
    }

    fn available_templates(&self) -> Vec<String> {
        self.template_registry.list_templates()
    }

    async fn template_info(&self, name: &str) -> DslResult<TemplateInfo> {
        self.template_info(name).await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_basic_template_execution() {
        let mut generator = DslCodeGenerator::new();

        let dsl_template = r#"
            template HelloWorld {
                name: "hello_world"
                description: "Simple hello world template"

                parameters: {
                    name: String!
                }

                generate: {
                    content: "fn hello() {\n    println!("Hello {{name}}!");\n}"
                }
            }
        "#;

        // Load template
        generator
            .load_dsl(dsl_template)
            .await
            .expect("Should load template");

        // Execute template
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("World"));

        let result = generator
            .execute_template("hello_world", params, ProgrammingLanguage::Rust)
            .await
            .expect("Should execute template");

        assert_eq!(result.language, ProgrammingLanguage::Rust);
        assert!(result.code.contains("Hello World"));
    }

    #[test]
    fn test_template_listing() {
        let mut generator = DslCodeGenerator::new();

        let templates = generator.list_templates();
        assert_eq!(templates.len(), 0);

        // Stats should be zero
        let stats = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(generator.generation_stats())
            .unwrap();

        assert_eq!(stats.template_count, 0);
        assert_eq!(stats.plugin_count, 0);
    }
}
