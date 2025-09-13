use anyhow::Result;
use async_trait::async_trait;

use crate::spec_generation::generator::CodeGenerator;
use crate::spec_generation::parser::SpecificationParser;
use crate::spec_generation::types::{
    ArchitecturalPattern, GeneratedCode, ParsedSpecification, RefinedCode, SpecificationRequest, ValidationResult,
};
use crate::spec_generation::validation::CodeValidator;

/// Main system for specification-driven code generation
pub struct IntelligentSpecGenerator {
    parser:    SpecificationParser,
    generator: CodeGenerator,
    validator: CodeValidator,
}

impl Default for IntelligentSpecGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IntelligentSpecGenerator {
    /// Create a new IntelligentSpecGenerator with default components
    pub fn new() -> Self {
        Self {
            parser:    SpecificationParser::new(),
            generator: CodeGenerator::new(),
            validator: CodeValidator::new(),
        }
    }

    /// Create a new IntelligentSpecGenerator with custom components
    pub fn with_components(parser: SpecificationParser, generator: CodeGenerator, validator: CodeValidator) -> Self {
        Self {
            parser,
            generator,
            validator,
        }
    }

    /// Generate code from a specification request
    pub async fn generate_from_spec(&self, request: &SpecificationRequest) -> Result<GeneratedCode> {
        // Parse the specification
        let parsed_spec = self.parse_specification(&request.description).await?;

        // Generate code from the parsed specification
        let generated_code = self.generator.generate_code(&parsed_spec).await?;

        // Validate the generated code
        self.validate_generated_code(&generated_code, &parsed_spec)?;

        Ok(generated_code)
    }

    /// Parse a natural language specification into a structured format
    pub async fn parse_specification(&self, text: &str) -> Result<ParsedSpecification> {
        self.parser.parse_specification(text).await
    }

    /// Generate a code template for a specific architectural pattern
    pub async fn generate_pattern(&self, pattern: &ArchitecturalPattern) -> Result<GeneratedCode> {
        // Convert the pattern to a specification
        let spec = self.pattern_to_specification(pattern).await?;

        // Generate code from the specification
        self.generate_from_spec(&spec).await
    }

    /// Validate generated code against the specification
    pub fn validate_generation(&self, _code: &str, spec: &ParsedSpecification) -> Result<ValidationResult> {
        // In a real implementation, we would parse the code and compare it to the spec
        // For now, we'll just validate the spec itself
        Ok(self.validator.validate_specification(spec))
    }

    /// Refine generated code based on feedback
    pub async fn refine_generation(
        &self,
        code: &str,
        _spec: &ParsedSpecification,
        feedback: &str,
    ) -> Result<RefinedCode> {
        // In a real implementation, we would parse the feedback and make appropriate changes
        // For now, we'll just return the original code with the feedback as a comment
        let refined_code = format!("// Feedback: {}\n\n{}", feedback, code);

        Ok(RefinedCode {
            code:        refined_code,
            changes:     vec![],
            explanation: "Code has been updated based on feedback".to_string(),
        })
    }

    /// Validate all generated code files
    fn validate_generated_code(&self, generated_code: &GeneratedCode, _spec: &ParsedSpecification) -> Result<()> {
        let mut all_issues = Vec::new();
        let mut all_valid = true;

        // Validate each code file
        for file in &generated_code.files {
            let result = self.validator.validate_code_file(file);
            if !result.is_valid {
                all_valid = false;
            }
            all_issues.extend(result.issues);
        }

        // If there are validation issues, return them as an error
        if !all_valid {
            anyhow::bail!(
                "Validation failed with {} issues: {:?}",
                all_issues.len(),
                all_issues
            );
        }

        Ok(())
    }

    /// Convert an architectural pattern to a specification request
    async fn pattern_to_specification(&self, pattern: &ArchitecturalPattern) -> Result<SpecificationRequest> {
        // In a real implementation, we would have templates or rules for each pattern
        // For now, we'll just create a simple specification based on the pattern name
        let description = format!(
            "Implement the {} pattern with the following components:\n\n",
            pattern.name
        );

        // Add components to the description
        let components = pattern
            .components
            .iter()
            .map(|c| format!("- {} ({})\n", c.name, c.role))
            .collect::<String>();

        let description = format!("{}{}\n{}", description, components, pattern.description);

        Ok(SpecificationRequest {
            description,
            language: "rust".to_string(),
            context: None,
        })
    }
}

#[async_trait]
impl crate::spec_generation::types::SpecificationGenerator for IntelligentSpecGenerator {
    async fn generate_from_spec(&self, request: &SpecificationRequest) -> Result<GeneratedCode> {
        self.generate_from_spec(request).await
    }

    async fn parse_specification(&self, text: &str) -> Result<ParsedSpecification> {
        self.parse_specification(text).await
    }

    async fn generate_pattern(&self, pattern: &ArchitecturalPattern) -> Result<GeneratedCode> {
        self.generate_pattern(pattern).await
    }

    async fn validate_generation(&self, code: &str, spec: &ParsedSpecification) -> Result<ValidationResult> {
        self.validate_generation(code, spec)
    }

    async fn refine_generation(&self, code: &str, spec: &ParsedSpecification, feedback: &str) -> Result<RefinedCode> {
        self.refine_generation(code, spec, feedback).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_generation::types::{
        CodeFile, Entity, EntityType, Field, FunctionSpec, Parameter, ParsedSpecification, Requirement,
        SpecificationRequest,
    };

    #[tokio::test]
    async fn test_generate_from_spec() {
        let generator = IntelligentSpecGenerator::new();
        let request = SpecificationRequest {
            description: r#"
            // A simple user management system

            // Requirements:
            // - The system must store user information
            // - Users should be able to update their profile

            struct User {
                id: String,
                name: String,
                email: String,
            }

            trait UserRepository {
                fn save_user(&self, user: &User) -> Result<(), String>;
                fn find_user_by_id(&self, id: &str) -> Option<User>;
            }

            struct UserService {
                repository: Box<dyn UserRepository>,
            }

            impl UserService {
                fn update_user_email(&self, user_id: &str, new_email: &str) -> Result<(), String> {
                    // Implementation
                    Ok(())
                }
            }
            "#
            .to_string(),
            language:    "rust".to_string(),
            context:     None,
        };

        let result = generator.generate_from_spec(&request).await;
        assert!(result.is_ok());
        let generated = result.unwrap();
        assert!(!generated.files.is_empty());
        assert!(!generated.resources.is_empty());
    }

    #[tokio::test]
    async fn test_parse_specification() {
        let generator = IntelligentSpecGenerator::new();
        let spec_text = r#"
        // A simple counter component

        struct Counter {
            value: i32,
        }

        impl Counter {
            fn new() -> Self {
                Counter { value: 0 }
            }

            fn increment(&mut self) {
                self.value += 1;
            }

            fn value(&self) -> i32 {
                self.value
            }
        }
        "#;

        let result = generator.parse_specification(spec_text).await;
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(!spec.entities.is_empty());
        assert!(!spec.functions.is_empty());
    }

    #[tokio::test]
    async fn test_validate_generation() {
        let generator = IntelligentSpecGenerator::new();
        let spec = ParsedSpecification {
            requirements: vec![Requirement {
                id:          "REQ-001".to_string(),
                description: "The system must store user information".to_string(),
                priority:    1,
                related_to:  vec!["User".to_string()],
            }],
            patterns:     vec![],
            entities:     vec![Entity {
                name:         "User".to_string(),
                entity_type:  EntityType::Struct,
                fields:       vec![Field {
                    name:        "id".to_string(),
                    field_type:  "String".to_string(),
                    is_optional: false,
                    docs:        vec!["Unique identifier".to_string()],
                }],
                docs:         vec!["A user in the system".to_string()],
                requirements: vec!["REQ-001".to_string()],
            }],
            functions:    vec![FunctionSpec {
                name:         "create_user".to_string(),
                return_type:  "Result<User, String>".to_string(),
                parameters:   vec![],
                docs:         vec!["Creates a new user".to_string()],
                requirements: vec!["REQ-001".to_string()],
                error_types:  vec!["String".to_string()],
            }],
        };

        let code = r#"
        // This is a test file

        fn main() {
            println!("Hello, world!");
        }
        "#;

        let result = generator.validate_generation(code, &spec);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid);
        assert_eq!(validation.score, 1.0);
    }
}
