//! Template engine for code generation

use handlebars::Handlebars;
use std::collections::HashMap;

/// Template engine for rendering code templates
#[derive(Clone)]
pub struct TemplateEngine {
    engine: Handlebars<'static>,
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

        Self { engine }
    }

    /// Render a template with context
    pub fn render(
        &self,
        template: &str,
        context: &serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.engine.render(template, context).map_err(Into::into)
    }

    /// Register a custom template
    pub fn register_template(
        &mut self,
        name: &str,
        template: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.engine
            .register_template_string(name, template)
            .map_err(Into::into)
    }

    /// Check if template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.engine.has_template(name)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.has_template("struct"));
    }

    #[test]
    fn test_template_rendering() {
        let engine = TemplateEngine::new();
        let context = json!({"name": "User", "fields": ["id", "name"]});
        let result = engine.render("struct", &context);
        assert!(result.is_ok() || !engine.has_template("struct")); // Either renders or template doesn't exist
    }
}
