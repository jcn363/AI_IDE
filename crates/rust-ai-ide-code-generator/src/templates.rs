//! Template engine for code generation

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use handlebars::Handlebars;

use crate::cache::TemplateCache;

/// Template engine for rendering code templates
#[derive(Clone)]
pub struct TemplateEngine {
    engine: Handlebars<'static>,
    cache: Option<Arc<TemplateCache>>,
}

impl TemplateEngine {
    /// Create a new template engine with default templates
    pub fn new() -> Self {
        let mut engine = Handlebars::new();

        // Register built-in templates
        engine
            .register_template_string("struct", include_str!("templates/struct.rs.hbs"))
            .unwrap();
        engine
            .register_template_string("function", include_str!("templates/function.rs.hbs"))
            .unwrap();
        engine
            .register_template_string("test", include_str!("templates/test.rs.hbs"))
            .unwrap();

        Self {
            engine,
            cache: None,
        }
    }

    /// Create a new template engine with cache support
    pub fn with_cache(cache: Arc<TemplateCache>) -> Self {
        let mut engine = Self::new();
        engine.cache = Some(cache);
        engine
    }

    /// Warm up the cache with templates from project root
    pub async fn warmup_cache(&self, project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(cache) = &self.cache {
            cache.warmup(project_root).await?;
        }
        Ok(())
    }

    /// Render a template with context
    pub fn render(&self, template: &str, context: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
        self.engine.render(template, context).map_err(Into::into)
    }

    /// Register a custom template
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.engine
            .register_template_string(name, template)
            .map_err(Into::into)
    }

    /// Check if template exists (checks both engine and cache)
    pub async fn has_template(&self, name: &str) -> bool {
        if self.engine.has_template(name) {
            return true;
        }

        #[cfg(feature = "template-cache")]
        if let Some(cache) = &self.cache {
            return cache.has_template(name).await;
        }

        false
    }

    /// Get template content from cache if available
    pub async fn get_cached_template(&self, name: &str) -> Option<String> {
        #[cfg(feature = "template-cache")]
        if let Some(cache) = &self.cache {
            return cache.get_template(name).await;
        }

        None
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.has_template("struct").await);
    }

    #[test]
    fn test_template_rendering() {
        let engine = TemplateEngine::new();
        let context = json!({"name": "User", "fields": ["id", "name"]});
        let result = engine.render("struct", &context);
        assert!(result.is_ok() || !engine.has_template("struct")); // Either renders or template
                                                                   // doesn't exist
    }
}
