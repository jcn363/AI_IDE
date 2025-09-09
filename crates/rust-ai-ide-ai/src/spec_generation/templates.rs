//! Template management for code generation
//!
//! This module provides templates for generating Rust code from specifications.
//! It uses the Handlebars templating engine to render code from structured data.

use handlebars::Handlebars;
use serde_json::json;

/// Template manager for code generation
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    /// Create a new TemplateEngine with default templates
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register default templates
        Self::register_default_templates(&mut handlebars);

        Self { handlebars }
    }

    /// Register default templates
    fn register_default_templates(handlebars: &mut Handlebars<'static>) {
        // Struct template
        handlebars
            .register_template_string(
                "struct",
                r#"{{#if docs}}
{{#each docs}}
/// {{this}}
{{/each}}
{{/if}}
pub struct {{name}} {
    {{#each fields}}
    {{#if docs}}
    {{#each docs}}
    /// {{this}}
    {{/each}}
    {{/if}}
    pub {{name}}: {{#if is_optional}}Option<{{type}}>{{else}}{{type}}{{/if}}{{#unless @last}},{{/unless}}
    {{/fields}}
}"#,
            )
            .expect("Failed to register struct template");

        // Trait template
        handlebars
            .register_template_string(
                "trait",
                r#"{{#if docs}}
{{#each docs}}
/// {{this}}
{{/each}}
{{/if}}
pub trait {{name}} {
    {{#each methods}}
    {{#if docs}}
    {{#each docs}}
    /// {{this}}
    {{/each}}
    {{/if}}
    fn {{name}}(&self{{#if has_params}}, {{/if}}{{#each params}}{{name}}: {{type}}{{#unless @last}}, {{/unless}}{{/each}}){{#if return_type}} -> {{return_type}}{{/if}};
    {{/each}}
}"#,
            )
            .expect("Failed to register trait template");

        // Impl block template
        handlebars
            .register_template_string(
                "impl",
                r#"{{#if docs}}
{{#each docs}}
/// {{this}}
{{/each}}
{{/if}}
impl {{#if trait_name}}{{trait_name}} for {{/if}}{{type_name}} {
    {{#each methods}}
    {{#if docs}}
    {{#each docs}}
    /// {{this}}
    {{/each}}
    {{/if}}
    fn {{name}}(&self{{#if has_params}}, {{/if}}{{#each params}}{{name}}: {{type}}{{#unless @last}}, {{/unless}}{{/each}}){{#if return_type}} -> {{return_type}}{{/if}} {
        {{#if is_todo}}todo!("Implement {{name}}")
        {{~^~}}// TODO: Implement {{name}}
        unimplemented!()
        {{/if}}
    }
    {{/each}}
}"#,
            )
            .expect("Failed to register impl template");

        // Module template
        handlebars
            .register_template_string(
                "module",
                r#"{{#if docs}}
{{#each docs}}
//! {{this}}
{{/each}}
{{/if}}
pub mod {{name}} {
    {{#each items}}
    {{#if is_pub}}pub {{/if}}mod {{name}};
    {{/each}}
}"#,
            )
            .expect("Failed to register module template");
    }

    /// Render a template with the given data
    pub fn render(&self, template_name: &str, data: &serde_json::Value) -> anyhow::Result<String> {
        self.handlebars
            .render(template_name, data)
            .map_err(|e| anyhow::anyhow!("Failed to render template '{}': {}", template_name, e))
    }

    /// Register a custom template
    pub fn register_template(&mut self, name: &str, template: &str) -> anyhow::Result<()> {
        self.handlebars
            .register_template_string(name, template)
            .map_err(|e| anyhow::anyhow!("Failed to register template '{}': {}", name, e))?;
        Ok(())
    }

    /// Render a struct definition
    pub fn render_struct(
        &self,
        name: &str,
        fields: &[(&str, &str, bool, Vec<&str>)],
        docs: &[&str],
    ) -> anyhow::Result<String> {
        let fields_data: Vec<serde_json::Value> = fields
            .iter()
            .map(|(name, field_type, is_optional, field_docs)| {
                json!({
                    "name": name,
                    "type": field_type,
                    "is_optional": is_optional,
                    "docs": field_docs
                })
            })
            .collect();

        let data = json!({
            "name": name,
            "docs": docs,
            "fields": fields_data
        });

        self.render("struct", &data)
    }

    /// Render a trait definition
    pub fn render_trait(
        &self,
        name: &str,
        methods: &[(&str, &str, Vec<(&str, &str)>, Option<&str>, Vec<&str>)],
        docs: &[&str],
    ) -> anyhow::Result<String> {
        let methods_data: Vec<serde_json::Value> = methods
            .iter()
            .map(|(name, return_type, params, _opt_return, _docs)| {
                let params_data: Vec<serde_json::Value> = params
                    .iter()
                    .map(|(name, param_type)| {
                        json!({
                            "name": name,
                            "type": param_type
                        })
                    })
                    .collect();

                json!({
                    "name": name,
                    "return_type": return_type,
                    "params": params_data,
                    "has_params": !params.is_empty(),
                    "docs": _docs
                })
            })
            .collect();

        let data = json!({
            "name": name,
            "docs": docs,
            "methods": methods_data
        });

        self.render("trait", &data)
    }

    /// Render an implementation block
    pub fn render_impl(
        &self,
        type_name: &str,
        trait_name: Option<&str>,
        methods: &[(&str, &str, Vec<(&str, &str)>, bool, Option<&str>, Vec<&str>)],
        docs: &[&str],
    ) -> anyhow::Result<String> {
        let methods_data: Vec<serde_json::Value> = methods
            .iter()
            .map(|(name, return_type, params, is_todo, body, _docs)| {
                let params_data: Vec<serde_json::Value> = params
                    .iter()
                    .map(|(name, param_type)| {
                        json!({
                            "name": name,
                            "type": param_type
                        })
                    })
                    .collect();

                json!({
                    "name": name,
                    "return_type": return_type,
                    "params": params_data,
                    "has_params": !params.is_empty(),
                    "is_todo": is_todo,
                    "body": body,
                    "docs": _docs
                })
            })
            .collect();

        let data = json!({
            "type_name": type_name,
            "trait_name": trait_name,
            "docs": docs,
            "methods": methods_data
        });

        self.render("impl", &data)
    }

    /// Render a module definition
    pub fn render_module(
        &self,
        name: &str,
        items: &[(&str, bool)],
        docs: &[&str],
    ) -> anyhow::Result<String> {
        let items_data: Vec<serde_json::Value> = items
            .iter()
            .map(|(name, is_pub)| {
                json!({
                    "name": name,
                    "is_pub": is_pub
                })
            })
            .collect();

        let data = json!({
            "name": name,
            "docs": docs,
            "items": items_data
        });

        self.render("module", &data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_struct() {
        let engine = TemplateEngine::new();
        let result = engine
            .render_struct(
                "User",
                &[
                    ("id", "String", false, vec!["The user's unique identifier"]),
                    ("name", "String", false, vec!["The user's full name"]),
                    ("email", "String", true, vec!["The user's email address"]),
                ],
                &[
                    "Represents a user in the system",
                    "This is a multi-line doc comment",
                ],
            )
            .unwrap();

        assert!(result.contains("pub struct User"));
        assert!(result.contains("pub id: String"));
        assert!(result.contains("pub name: String"));
        assert!(result.contains("pub email: Option<String>"));
        assert!(result.contains("Represents a user in the system"));
        assert!(result.contains("The user's unique identifier"));
    }

    #[test]
    fn test_render_trait() {
        let engine = TemplateEngine::new();
        let result = engine
            .render_trait(
                "Repository",
                &[(
                    "find_by_id",
                    "Option<Self::Entity>",
                    vec![("id", "&str")],
                    Some("Finds an entity by its ID"),
                    vec!["# Arguments", "* `id` - The ID of the entity to find"],
                )],
                &["A generic repository trait for data access"],
            )
            .unwrap();

        assert!(result.contains("pub trait Repository"));
        assert!(result.contains("fn find_by_id(&self, id: &str) -> Option<Self::Entity>;"));
        assert!(result.contains("A generic repository trait"));
        assert!(result.contains("Finds an entity by its ID"));
    }

    #[test]
    fn test_render_impl() {
        let engine = TemplateEngine::new();
        let result = engine
            .render_impl(
                "UserRepository",
                None,
                &[(
                    "find_by_id",
                    "Option<User>",
                    vec![("id", "&str")],
                    true,
                    Some("// Implementation here"),
                    vec!["Finds a user by ID"],
                )],
                &["Implementation of user repository"],
            )
            .unwrap();

        assert!(result.contains("impl UserRepository"));
        assert!(result.contains("fn find_by_id(&self, id: &str) -> Option<User>"));
        assert!(result.contains("Implementation of user repository"));
        assert!(result.contains("todo!(\"Implement find_by_id\")"));
    }

    #[test]
    fn test_render_module() {
        let engine = TemplateEngine::new();
        let result = engine
            .render_module(
                "user",
                &[
                    ("model", true),
                    ("repository", true),
                    ("service", true),
                    ("tests", false),
                ],
                &["User-related functionality"],
            )
            .unwrap();

        assert!(result.contains("pub mod user"));
        assert!(result.contains("pub mod model;"));
        assert!(result.contains("pub mod repository;"));
        assert!(result.contains("mod tests;"));
        assert!(result.contains("User-related functionality"));
    }
}
