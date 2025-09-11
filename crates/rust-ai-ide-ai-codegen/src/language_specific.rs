//! # Language-Specific Code Generation Module
//!
//! This module provides language-specific code generation capabilities
//! for TypeScript, Python, JavaScript, Java, C++, Go, and other languages.

use crate::function_generation::GeneratedFunction;
use rust_ai_ide_shared_codegen::generator::{
    CodeGenerationContext, TargetLanguage, UserPreferences,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Language-specific code generator
#[derive(Debug, Clone)]
pub struct LanguageSpecificGenerator {
    templates: HashMap<String, LanguageTemplates>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageTemplates {
    pub function_template: String,
    pub class_template: String,
    pub interface_template: String,
    pub type_definition_template: String,
    pub async_function_template: String,
}

impl Default for LanguageSpecificGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageSpecificGenerator {
    /// Create a new language-specific generator with default templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // TypeScript templates
        templates.insert("typescript".to_string(), LanguageTemplates {
            function_template: "export function {{name}}({{parameters}}): {{return_type|void}} {\n    {{#each body}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            class_template: "export class {{name}} {{#if extends}}extends {{extends}}{{/if}} {\n    {{#each fields}}\n    {{visibility}} {{name}}: {{type}};\n    {{/each}}\n\n    {{#each methods}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            interface_template: "export interface {{name}} {{#if extends}}extends {{extends}} {{/if}}{\n    {{#each fields}}\n    {{name}}: {{type}};\n    {{/each}}\n}".to_string(),
            type_definition_template: "export type {{name}} = {{#if union}}{{union}}{{else}}{{#if object}}{ {{#each fields}}{{name}}: {{type}};{{/each}} }{{else}}{{primitive}}{{/if}}{{/if}};".to_string(),
            async_function_template: "export async function {{name}}({{parameters}}): Promise<{{return_type|void}}> {\n    {{#each body}}\n    {{this}}\n    {{/each}}\n}".to_string(),
        });

        // Python templates
        templates.insert("python".to_string(), LanguageTemplates {
            function_template: "def {{name}}({{parameters}}) {{#if return_type}}-> {{return_type}}{{/if}}:\n    {{#each body}}\n    {{this}}\n    {{/each}}".to_string(),
            class_template: "class {{name}}{{#if extends}}({{extends}}){{/if}}:\n    def __init__(self{{#each fields}}, {{name}}: {{type}}{{/each}}):\n        {{#each fields}}\n        self.{{name}} = {{name}}\n        {{/each}}\n\n    {{#each methods}}\n    {{this}}\n    {{/each}}".to_string(),
            interface_template: "from abc import ABC, abstractmethod\n\nclass {{name}}(ABC):\n    {{#each methods}}\n    @abstractmethod\n    {{this}}\n    {{/each}}".to_string(),
            type_definition_template: "# Type hint for {{name}}\n{{name}}: {{#if union}}{{union}}{{else}}{{#if object}}Dict[str, Any]{{else}}{{primitive}}{{/if}}{{/if}}".to_string(),
            async_function_template: "async def {{name}}({{parameters}}) {{#if return_type}}-> {{return_type}}{{/if}}:\n    {{#each body}}\n    {{this}}\n    {{/each}}".to_string(),
        });

        // Go templates
        templates.insert("go".to_string(), LanguageTemplates {
            function_template: "func {{name}}({{parameters}}) {{#if return_type}}{{return_type}}{{/if}} {\n    {{#each body}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            class_template: "// Go uses structs instead of classes\ntype {{name}} struct {\n    {{#each fields}}\n    {{name}} {{type}}\n    {{/each}}\n}\n\n// Constructor\nfunction New{{name}}({{parameters}}) *{{name}} {\n    return &{{name}}{ {{#each fields}}\n        {{name}}: {{name}},\n    {{/each}} }\n}".to_string(),
            interface_template: "type {{name}} interface {\n    {{#each methods}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            type_definition_template: "// Type definition\ntype {{name}} {{#if union}}interface{}{{else}}struct{{/if}} {\n    {{#if union}}\n    // Union type represented as interface\n    {{#else}}\n    // Struct fields here\n    {{/if}}\n}".to_string(),
            async_function_template: "// Go uses goroutines for concurrency\nfunc {{name}}({{parameters}}) {{#if return_type}}{{return_type}}{{/if}} {\n    go func() {\n        {{#each body}}\n        {{this}}\n        {{/each}}\n    }()\n}".to_string(),
        });

        // Java templates
        templates.insert("java".to_string(), LanguageTemplates {
            function_template: "public {{#if static}}static {{/if}}{{return_type|void}} {{name}}({{parameters}}) {\n    {{#each body}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            class_template: "public class {{name}} {{#if extends}}extends {{extends}} {{/if}}{{#if implements}}implements {{implements}} {{/if}}{\n    {{#each fields}}\n    {{visibility}} {{type}} {{name}};\n    {{/each}}\n\n    {{#each methods}}\n    {{this}}\n    {{/each}}\n\n    {{#if constructor}}\n    public {{name}}({{constructor_params}}) {\n        {{#each constructor_body}}\n        {{this}}\n        {{/each}}\n    }\n    {{/if}}\n}".to_string(),
            interface_template: "public interface {{name}} {{#if extends}}extends {{extends}} {{/if}}{\n    {{#each methods}}\n    {{return_type|void}} {{name}}({{parameters}});\n    {{/each}}\n}".to_string(),
            type_definition_template: "// Java type definition - use classes or interfaces\n// {{name}} would be implemented as a class or interface".to_string(),
            async_function_template: "// Java async - would use CompletableFuture or similar\npublic CompletableFuture<{{return_type|Void}}> {{name}}({{parameters}}) {\n    return CompletableFuture.supplyAsync(() -> {\n        {{#each body}}\n        {{this}}\n        {{/each}}\n        return null;\n    });\n}".to_string(),
        });

        // JavaScript templates
        templates.insert("javascript".to_string(), LanguageTemplates {
            function_template: "function {{name}}({{parameters}}) {\n    {{#each body}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            class_template: "class {{name}} {{#if extends}}extends {{extends}} {{/if}}{\n    constructor({{constructor_params}}) {\n        {{#each constructor_body}}\n        {{this}}\n        {{/each}}\n    }\n\n    {{#each methods}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            interface_template: "// JavaScript doesn't have interfaces - using JSDoc\n/**\n * @interface {{name}}\n {{#each methods}}\n * @method {{name}}\n {{/each}}\n */\nfunction {{name}}() {}\n\n// Implementation would be a regular object".to_string(),
            type_definition_template: "// JSDoc type definition\n/**\n * @typedef {{{#if union}}{{union}}{{else}}{{#if object}}Object{{else}}{{primitive}}{{/if}}{{/if}}} {{name}}\n */".to_string(),
            async_function_template: "async function {{name}}({{parameters}}) {\n    {{#each body}}\n    {{#if ../is_await}}await {{/if}}{{this}}\n    {{/each}}\n}".to_string(),
        });

        // C++ templates
        templates.insert("cpp".to_string(), LanguageTemplates {
            function_template: "{{return_type|void}} {{name}}({{parameters}}) {\n    {{#each body}}\n    {{this}}\n    {{/each}}\n}".to_string(),
            class_template: "class {{name}} {{#if extends}}: public {{extends}} {{/if}}{\n{{#each visibility_sections}}\n{{visibility}}:\n    {{#each fields}}\n    {{type}} {{name}};\n    {{/each}}\n\n    {{#each methods}}\n    {{#if virtual}}virtual {{/if}}{{return_type|void}} {{name}}({{parameters}}){{#if const}} const{{/if}} {{#if override}} override{{/if}} {\n        {{#each body}}\n        {{this}}\n        {{/each}}\n    }\n    {{/each}}\n{{/each}}\n};".to_string(),
            interface_template: "class {{name}} {\npublic:\n    {{#each methods}}\n    virtual {{return_type|void}} {{name}}({{parameters}}) = 0;\n    {{/each}}\n    virtual ~{{name}}() = default;\n};".to_string(),
            type_definition_template: "// C++ type definition\n// Complex types would use templates or variants\nusing {{name}} = {{#if union}}std::variant<{{union_types}}>{{else}}{{#if primitive}}{{primitive}}{{else}}auto{{/if}}{{/if}};".to_string(),
            async_function_template: "std::future<{{return_type|void}}> {{name}}({{parameters}}) {\n    return std::async(std::launch::async, []() {\n        {{#each body}}\n        {{this}}\n        {{/each}}\n        return {{#if return_type}}result{{else}}{} {{/if}};\n    });\n}".to_string(),
        });

        Self { templates }
    }

    /// Generate code for a specific language
    pub fn generate_for_language(
        &self,
        language: &TargetLanguage,
        template_type: &str,
        context: &CodeGenerationContext,
        variables: HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let lang_key = Self::language_to_key(language);
        let templates = self
            .templates
            .get(&lang_key)
            .ok_or_else(|| format!("No templates available for language {:?}", language))?;

        let template = match template_type {
            "function" => &templates.function_template,
            "class" => &templates.class_template,
            "interface" => &templates.interface_template,
            "type" => &templates.type_definition_template,
            "async_function" => &templates.async_function_template,
            _ => return Err(format!("Unknown template type: {}", template_type).into()),
        };

        self.render_template(template, variables, context)
    }

    /// Generate a function in the specified language
    pub fn generate_function(
        &self,
        context: &CodeGenerationContext,
        function_name: &str,
        parameters: Vec<(String, String)>,
        return_type: Option<&str>,
        body_lines: Vec<String>,
    ) -> Result<GeneratedFunction, Box<dyn std::error::Error + Send + Sync>> {
        let mut variables = HashMap::new();

        // Format parameters
        let params_str = parameters
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, ty))
            .collect::<Vec<_>>()
            .join(", ");
        variables.insert("name".to_string(), function_name.to_string());
        variables.insert("parameters".to_string(), params_str);

        if let Some(ret_ty) = return_type {
            variables.insert("return_type".to_string(), ret_ty.to_string());
        }

        // Add body variables
        for (i, line) in body_lines.iter().enumerate() {
            variables.insert(format!("body_{}", i), line.clone());
        }

        let code = self.generate_for_language(&context.language, "function", context, variables)?;

        Ok(GeneratedFunction {
            name: function_name.to_string(),
            signature: code.clone(),
            body: body_lines.join("\n"),
            imports: vec![],
            documentation: None,
            tests: None,
            complexity: 1.0,
            confidence_score: 0.8,
            language: Some(context.language.clone()),
            code,
            parameters: parameters.into_iter().map(|(n, _)| n).collect(),
            return_type: return_type.map(|s| s.to_string()),
        })
    }

    /// Check if a language is supported
    pub fn is_language_supported(&self, language: &TargetLanguage) -> bool {
        let lang_key = Self::language_to_key(language);
        self.templates.contains_key(&lang_key)
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<TargetLanguage> {
        vec![
            TargetLanguage::TypeScript,
            TargetLanguage::JavaScript,
            TargetLanguage::Python,
            TargetLanguage::Go,
            TargetLanguage::Java,
            TargetLanguage::Cpp,
        ]
    }

    /// Convert TargetLanguage to string key
    fn language_to_key(language: &TargetLanguage) -> String {
        match language {
            TargetLanguage::TypeScript => "typescript",
            TargetLanguage::JavaScript => "javascript",
            TargetLanguage::Python => "python",
            TargetLanguage::Go => "go",
            TargetLanguage::Java => "java",
            TargetLanguage::Cpp => "cpp",
            TargetLanguage::Rust => "rust",
            _ => "other",
        }
        .to_string()
    }

    /// Render template with variables (simplified Handlebars-like syntax)
    fn render_template(
        &self,
        template: &str,
        variables: HashMap<String, String>,
        context: &CodeGenerationContext,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = template.to_string();

        // Replace simple variables
        for (key, value) in &variables {
            result = result.replace(&format!("{{{}}}", key), value);
        }

        // Apply language-specific formatting preferences
        self.apply_formatting(&mut result, context.user_preferences.clone());

        Ok(result)
    }

    /// Apply formatting preferences
    fn apply_formatting(&self, code: &mut String, prefs: UserPreferences) {
        // Apply indentation
        let indent: String = prefs.indentation.chars().collect();

        // Apply line length constraints
        self.apply_line_wrapping(code, prefs.max_line_length);
    }

    /// Apply line wrapping based on max line length
    fn apply_line_wrapping(&self, code: &mut String, max_length: usize) {
        // Simple line wrapping implementation
        let lines: Vec<String> = code
            .lines()
            .map(|line| {
                if line.len() > max_length {
                    // Insert line breaks for long lines (simplified)
                    format!(
                        "{}\n    {}",
                        &line[..max_length / 2],
                        &line[max_length / 2..]
                    )
                } else {
                    line.to_string()
                }
            })
            .collect();

        *code = lines.join("\n");
    }
}
