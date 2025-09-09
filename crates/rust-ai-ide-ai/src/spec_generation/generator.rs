use crate::spec_generation::types::{
    CodeFile, Entity, EntityType, FunctionSpec, GeneratedCode, ParsedSpecification, ResourceFile,
};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::json;
use std::collections::HashMap;

/// Generator for converting specifications into code
pub struct CodeGenerator {
    templates: Handlebars<'static>,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    /// Create a new CodeGenerator with default templates
    pub fn new() -> Self {
        let mut templates = Handlebars::new();
        templates.set_strict_mode(true);
        templates
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

        templates
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

        // Add more templates as needed...

        Self { templates }
    }

    /// Generate code from a parsed specification
    pub async fn generate_code(&self, spec: &ParsedSpecification) -> Result<GeneratedCode> {
        let mut files = Vec::new();
        let mut resources = Vec::new();

        // Generate code for each entity
        for entity in &spec.entities {
            let content = self.generate_entity(entity)?;
            let path = self.entity_file_path(entity);
            files.push(CodeFile {
                path,
                content,
                is_test: false,
            });
        }

        // Generate tests
        let test_files = self.generate_tests(&spec.entities, &spec.functions)?;
        files.extend(test_files);

        // Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml(spec);
        resources.push(cargo_toml);

        // Generate README
        let readme = self.generate_readme(spec);
        resources.push(readme);

        Ok(GeneratedCode {
            files,
            resources,
            build_instructions: self.generate_build_instructions(),
            next_steps: self.generate_next_steps(spec),
        })
    }

    /// Generate code for a single entity
    fn generate_entity(&self, entity: &Entity) -> Result<String> {
        let template_name = match entity.entity_type {
            EntityType::Struct => "struct",
            EntityType::Trait => "trait",
            EntityType::Enum => "enum",
            EntityType::Module => "mod",
            EntityType::TypeAlias => "type",
        };

        let data = json!({
            "name": entity.name,
            "docs": entity.docs,
            "fields": entity.fields.iter().map(|f| {
                json!({
                    "name": f.name,
                    "type": f.field_type,
                    "is_optional": f.is_optional,
                    "docs": f.docs
                })
            }).collect::<Vec<_>>()
        });

        self.templates
            .render(template_name, &data)
            .context("Failed to render entity template")
    }

    /// Generate test files
    fn generate_tests(
        &self,
        entities: &[Entity],
        functions: &[FunctionSpec],
    ) -> Result<Vec<CodeFile>> {
        let mut test_files = Vec::new();

        // Generate unit tests for each function
        for func in functions {
            let test_name = format!("test_{}", func.name);
            let test_content = format!(
                r#"
                #[test]
                fn {}() {{
                    // TODO: Add test implementation for {}
                    assert!(true);
                }}"#,
                test_name, func.name
            );

            test_files.push(CodeFile {
                path: format!("tests/{}_test.rs", func.name),
                content: test_content,
                is_test: true,
            });
        }

        // Generate integration tests for entities
        for entity in entities {
            let test_content = format!(
                r#"
                #[cfg(test)]
                mod {}_tests {{
                    use super::*;
                    
                    #[test]
                    fn test_{}_creation() {{
                        // TODO: Add test implementation for {}
                        assert!(true);
                    }}
                }}"#,
                entity.name.to_lowercase(),
                entity.name.to_lowercase(),
                entity.name
            );

            test_files.push(CodeFile {
                path: format!("tests/{}_test.rs", entity.name.to_lowercase()),
                content: test_content,
                is_test: true,
            });
        }

        Ok(test_files)
    }

    /// Generate Cargo.toml content
    fn generate_cargo_toml(&self, spec: &ParsedSpecification) -> ResourceFile {
        let mut dependencies = HashMap::new();

        // Add common dependencies
        dependencies.insert("serde".to_string(), "1.0".to_string());
        dependencies.insert("serde_json".to_string(), "1.0".to_string());
        dependencies.insert("anyhow".to_string(), "1.0".to_string());

        // Add dependencies based on patterns
        if spec.patterns.iter().any(|p| p.name.contains("Repository")) {
            dependencies.insert("async-trait".to_string(), "0.1".to_string());
        }

        let toml_content = format!(
            r#"[package]
name = "generated-code"
version = "0.1.0"
edition = "2021"
authors = ["Generated by Rust AI IDE"]
description = "Code generated from specification"

[dependencies]
{}"#,
            dependencies
                .iter()
                .map(|(name, version)| format!("{} = \"{}\"", name, version))
                .collect::<Vec<_>>()
                .join("\n")
        );

        ResourceFile {
            path: "Cargo.toml".to_string(),
            content: toml_content,
        }
    }

    /// Generate README content
    fn generate_readme(&self, spec: &ParsedSpecification) -> ResourceFile {
        let mut content = String::new();

        content.push_str(&format!("# {}\n\n", "Generated Code"));
        content.push_str("This code was automatically generated from a specification.\n\n");

        if !spec.requirements.is_empty() {
            content.push_str("## Requirements\n\n");
            for req in &spec.requirements {
                content.push_str(&format!("- [{}] {}\n", req.id, req.description));
            }
            content.push('\n');
        }

        if !spec.entities.is_empty() {
            content.push_str("## Entities\n\n");
            for entity in &spec.entities {
                content.push_str(&format!("- `{}` ({:?})\n", entity.name, entity.entity_type));
            }
            content.push('\n');
        }

        if !spec.patterns.is_empty() {
            content.push_str("## Detected Patterns\n\n");
            for pattern in &spec.patterns {
                content.push_str(&format!("### {}\n", pattern.name));
                content.push_str(&format!("{}\n\n", pattern.description));
                content.push_str("**Components:**\n");
                for component in &pattern.components {
                    content.push_str(&format!("- {} ({})\n", component.name, component.role));
                }
                content.push('\n');
            }
        }

        ResourceFile {
            path: "README.md".to_string(),
            content,
        }
    }

    /// Generate build instructions
    fn generate_build_instructions(&self) -> String {
        r#"To build and run the generated code:

1. Make sure you have Rust installed (https://www.rust-lang.org/tools/install)
2. Navigate to the generated project directory
3. Run `cargo build` to build the project
4. Run `cargo test` to run the tests
5. Run `cargo run` to execute the application (if applicable)"#
            .to_string()
    }

    /// Generate next steps
    fn generate_next_steps(&self, _spec: &ParsedSpecification) -> Vec<String> {
        vec![
            "Review the generated code for accuracy and completeness".to_string(),
            "Implement any missing functionality in the generated tests".to_string(),
            "Add documentation and examples as needed".to_string(),
            "Consider adding integration tests for end-to-end functionality".to_string(),
            "Update dependencies to their latest versions if needed".to_string(),
        ]
    }

    /// Generate file path for an entity
    fn entity_file_path(&self, entity: &Entity) -> String {
        let dir = match entity.entity_type {
            EntityType::Trait => "src/traits",
            EntityType::Module => "src/modules",
            _ => "src/models",
        };

        format!("{}/{}.rs", dir, entity.name.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_generation::types::{
        Entity, EntityType, Field, FunctionSpec, Parameter, ParsedSpecification, Requirement,
    };

    #[tokio::test]
    async fn test_generate_entity() {
        let generator = CodeGenerator::new();
        let entity = Entity {
            name: "User".to_string(),
            entity_type: EntityType::Struct,
            fields: vec![
                Field {
                    name: "id".to_string(),
                    field_type: "String".to_string(),
                    is_optional: false,
                    docs: vec!["Unique identifier for the user".to_string()],
                },
                Field {
                    name: "name".to_string(),
                    field_type: "String".to_string(),
                    is_optional: false,
                    docs: vec!["User's full name".to_string()],
                },
                Field {
                    name: "email".to_string(),
                    field_type: "String".to_string(),
                    is_optional: true,
                    docs: vec!["User's email address".to_string()],
                },
            ],
            docs: vec![
                "Represents a user in the system".to_string(),
                "This struct stores basic user information".to_string(),
            ],
            requirements: vec!["REQ-001".to_string()],
        };

        let result = generator.generate_entity(&entity).unwrap();
        assert!(result.contains("pub struct User"));
        assert!(result.contains("pub id: String"));
        assert!(result.contains("pub name: String"));
        assert!(result.contains("pub email: Option<String>"));
        assert!(result.contains("Represents a user in the system"));
        assert!(result.contains("User's full name"));
    }

    #[tokio::test]
    async fn test_generate_code() {
        let generator = CodeGenerator::new();
        let spec = ParsedSpecification {
            requirements: vec![Requirement {
                id: "REQ-001".to_string(),
                description: "The system must store user information".to_string(),
                priority: 1,
                related_to: vec!["User".to_string()],
            }],
            patterns: vec![],
            entities: vec![Entity {
                name: "User".to_string(),
                entity_type: EntityType::Struct,
                fields: vec![Field {
                    name: "id".to_string(),
                    field_type: "String".to_string(),
                    is_optional: false,
                    docs: vec!["Unique identifier for the user".to_string()],
                }],
                docs: vec!["Represents a user in the system".to_string()],
                requirements: vec!["REQ-001".to_string()],
            }],
            functions: vec![FunctionSpec {
                name: "create_user".to_string(),
                return_type: "Result<User, String>".to_string(),
                parameters: vec![
                    Parameter {
                        name: "name".to_string(),
                        param_type: "String".to_string(),
                        is_mut: false,
                        is_ref: false,
                    },
                    Parameter {
                        name: "email".to_string(),
                        param_type: "String".to_string(),
                        is_mut: false,
                        is_ref: false,
                    },
                ],
                docs: vec!["Creates a new user with the given name and email".to_string()],
                requirements: vec!["REQ-001".to_string()],
                error_types: vec!["String".to_string()],
            }],
        };

        let result = generator.generate_code(&spec).await.unwrap();

        // Check that we have the expected number of files
        assert!(!result.files.is_empty());
        assert!(!result.resources.is_empty());

        // Check that we have a Cargo.toml and README.md
        assert!(result.resources.iter().any(|f| f.path == "Cargo.toml"));
        assert!(result.resources.iter().any(|f| f.path == "README.md"));

        // Check that we have the entity file
        assert!(result.files.iter().any(|f| f.path.ends_with("user.rs")));

        // Check that we have test files
        assert!(result
            .files
            .iter()
            .any(|f| f.path.starts_with("tests/") && f.is_test));
    }
}
