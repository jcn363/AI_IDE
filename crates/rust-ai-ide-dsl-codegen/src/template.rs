//! Template management and execution

use crate::ast::{Parameter, ParameterType, Template};
use crate::error::DslResult;
use crate::types::*;
use async_trait::async_trait;
use rust_ai_ide_common::ProgrammingLanguage;
use serde_json;
use std::collections::HashMap;

/// Executable template implementation
#[derive(Debug)]
pub struct ExecutableTemplate {
    /// The parsed template AST
    ast: Template,
    /// Template execution context
    context: HashMap<String, serde_json::Value>,
}

impl ExecutableTemplate {
    /// Create a new executable template from AST
    pub fn new(ast: Template) -> Self {
        Self {
            ast,
            context: HashMap::new(),
        }
    }

    /// Set template execution context
    pub fn with_context(mut self, context: HashMap<String, serde_json::Value>) -> Self {
        self.context = context;
        self
    }
}

#[async_trait]
impl DslTemplate for ExecutableTemplate {
    fn name(&self) -> &str {
        &self.ast.name
    }

    fn description(&self) -> Option<&str> {
        self.ast.description.as_deref()
    }

    fn supported_languages(&self) -> Vec<ProgrammingLanguage> {
        // Extract from AST or return defaults
        vec![ProgrammingLanguage::Rust, ProgrammingLanguage::TypeScript]
    }

    fn parameters(&self) -> &[Parameter] {
        &self.ast.parameters
    }

    async fn validate_parameters(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> DslResult<()> {
        // Check required parameters
        for param in &self.ast.parameters {
            if param.required {
                if !params.contains_key(&param.name) {
                    return Err(crate::error::DslError::validation_with_field(
                        self.name().to_string(),
                        param.name.clone(),
                        format!("Missing required parameter: {}", param.name),
                    ));
                }
            }

            // Validate parameter types
            if let Some(value) = params.get(&param.name) {
                if !validate_parameter_type(value, &param.param_type) {
                    return Err(crate::error::DslError::validation_with_field(
                        self.name().to_string(),
                        param.name.clone(),
                        format!("Parameter {} has invalid type", param.name),
                    ));
                }
            }
        }

        Ok(())
    }

    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
        context: &GenerationContext,
    ) -> DslResult<GeneratedCode> {
        // Validate parameters first
        self.validate_parameters(params).await?;

        // Execute template generation
        self.execute_internal(params, context).await
    }
}

impl ExecutableTemplate {
    async fn execute_internal(
        &self,
        params: &HashMap<String, serde_json::Value>,
        context: &GenerationContext,
    ) -> DslResult<GeneratedCode> {
        let mut generated_code = String::new();
        let imports = Vec::new();
        let warnings = Vec::new();

        // Process template content
        for part in &self.ast.generate.content.parts {
            match part {
                crate::ast::ContentPart::Literal(text) => {
                    generated_code
                        .push_str(&self.interpolate_template(text, params, context).await?);
                }
                crate::ast::ContentPart::Placeholder(_) => {
                    // Placeholder interpolation will be handled here
                    // For Phase 2, we'll implement basic placeholder support
                }
                _ => {
                    // Other content parts will be implemented in later phases
                }
            }
        }

        Ok(GeneratedCode {
            code: generated_code,
            language: context.language.clone(),
            imports,
            tests: Vec::new(), // Tests will be implemented in Phase 5
            warnings,
            metadata: HashMap::new(),
        })
    }

    async fn interpolate_template(
        &self,
        template: &str,
        params: &HashMap<String, serde_json::Value>,
        context: &GenerationContext,
    ) -> DslResult<String> {
        let mut result = template.to_string();

        // Find and replace placeholders like {{variable}}
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").map_err(|e| {
            crate::error::DslError::execution(
                self.name().to_string(),
                format!("Invalid regex for template interpolation: {}", e),
            )
        })?;

        for capture in re.captures_iter(template) {
            if let Some(var_match) = capture.get(1) {
                let var_name = var_match.as_str().trim();

                // Get value from parameters or context
                let value = if let Some(param_value) = params.get(var_name) {
                    self.json_value_to_string(param_value)
                } else if let Some(context_value) = self.context.get(var_name) {
                    self.json_value_to_string(context_value)
                } else {
                    // Check for built-in variables
                    self.get_builtin_variable(var_name, context)?
                };

                result = result.replace(&format!("{{{{{}}}}}", var_name), &value);
            }
        }

        Ok(result)
    }

    fn json_value_to_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(a) => {
                let strings: Vec<String> = a.iter().map(|v| self.json_value_to_string(v)).collect();
                format!("{:?}", strings) // Creates a vector-like representation
            }
            _ => String::new(),
        }
    }

    fn get_builtin_variable(&self, name: &str, context: &GenerationContext) -> DslResult<String> {
        match name {
            "language" => Ok(context.language.to_string()),
            "workspace_root" => Ok(context.workspace_root.display().to_string()),
            "template_name" => Ok(self.name().to_string()),
            _ => Err(crate::error::DslError::execution(
                self.name().to_string(),
                format!("Unknown variable: {}", name),
            )),
        }
    }
}

/// Validate that a JSON value matches a parameter type
fn validate_parameter_type(value: &serde_json::Value, expected_type: &ParameterType) -> bool {
    match expected_type {
        ParameterType::String => value.is_string(),
        ParameterType::Integer => value.is_i64() || value.is_u64(),
        ParameterType::Boolean => value.is_boolean(),
        ParameterType::Float => value.is_f64() || value.is_number(),
        ParameterType::Array(element_type) => {
            if let serde_json::Value::Array(arr) = value {
                arr.iter().all(|v| validate_parameter_type(v, element_type))
            } else {
                false
            }
        }
        ParameterType::Custom(_) => true, // Accept any type for custom types
        ParameterType::ProgrammingLanguage => value.is_string(),
        ParameterType::Identifier(_) => value.is_string(),
    }
}

/// Template registry for managing available templates
pub struct TemplateRegistry {
    templates: HashMap<String, Box<dyn DslTemplate>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn register<T: DslTemplate + 'static>(&mut self, template: T) -> DslResult<()> {
        let name = template.name().to_string();

        if self.templates.contains_key(&name) {
            return Err(crate::error::DslError::template(
                name,
                "Template already registered".to_string(),
            ));
        }

        self.templates.insert(name, Box::new(template));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn DslTemplate> {
        self.templates.get(name).map(|boxed| boxed.as_ref())
    }

    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.templates.remove(name).is_some()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_template_validation() {
        let mut ast = Template::new("TestTemplate");
        ast.parameters.push(Parameter {
            name: "required_param".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: None,
        });

        let template = ExecutableTemplate::new(ast);

        // Missing required parameter should fail
        let mut params = HashMap::new();
        params.insert(
            "optional_param".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        assert!(template.validate_parameters(&params).await.is_err());

        // With required parameter should succeed
        params.insert(
            "required_param".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        assert!(template.validate_parameters(&params).await.is_ok());
    }

    #[test]
    fn test_template_registry() {
        let mut registry = TemplateRegistry::new();

        let ast = Template::new("TestTemplate");
        let template = ExecutableTemplate::new(ast);

        // Register template
        assert!(registry.register(template).is_ok());

        // Check it's registered
        assert!(registry.get("TestTemplate").is_some());

        // List templates
        let templates = registry.list_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0], "TestTemplate");

        // Remove template
        assert!(registry.remove("TestTemplate"));
        assert!(registry.get("TestTemplate").is_none());
    }

    #[test]
    fn test_parameter_type_validation() {
        // Test string validation
        assert!(validate_parameter_type(
            &serde_json::Value::String("test".to_string()),
            &ParameterType::String
        ));
        assert!(!validate_parameter_type(
            &serde_json::Value::Number(123.into()),
            &ParameterType::String
        ));

        // Test array validation
        let string_array = serde_json::Value::Array(vec![
            serde_json::Value::String("a".to_string()),
            serde_json::Value::String("b".to_string()),
        ]);
        assert!(validate_parameter_type(
            &string_array,
            &ParameterType::Array(Box::new(ParameterType::String))
        ));

        let mixed_array = serde_json::Value::Array(vec![
            serde_json::Value::String("a".to_string()),
            serde_json::Value::Number(123.into()),
        ]);
        assert!(!validate_parameter_type(
            &mixed_array,
            &ParameterType::Array(Box::new(ParameterType::String))
        ));
    }
}
