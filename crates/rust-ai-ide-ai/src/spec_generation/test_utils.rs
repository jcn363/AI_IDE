//! Test utilities for the spec_generation module

/// Lightweight constructor for Requirement
impl super::types::Requirement {
    pub fn new(id: impl Into<String>, description: impl Into<String>, priority: u8, related_to: Vec<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            priority,
            related_to,
        }
    }
}

/// Lightweight constructor for ArchitecturalPattern
impl super::types::ArchitecturalPattern {
    pub fn new(
        name: impl Into<String>,
        confidence: f32,
        description: impl Into<String>,
        components: Vec<super::types::PatternComponent>,
    ) -> Self {
        Self {
            name: name.into(),
            confidence,
            description: description.into(),
            components,
        }
    }
}

use super::types::{Entity, EntityType, Field, FunctionSpec, Parameter};

/// Lightweight constructor for Field
impl Field {
    pub fn new(name: impl Into<String>, field_type: impl Into<String>, is_optional: bool, docs: Vec<String>) -> Self {
        Self {
            name: name.into(),
            field_type: field_type.into(),
            is_optional,
            docs,
        }
    }
}

/// Lightweight constructor for Parameter
impl Parameter {
    pub fn new(name: impl Into<String>, param_type: impl Into<String>, is_mut: bool, is_ref: bool) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            is_mut,
            is_ref,
        }
    }
}

/// Lightweight constructor for Entity
impl Entity {
    pub fn new(
        name: impl Into<String>,
        entity_type: EntityType,
        fields: Vec<Field>,
        docs: Vec<String>,
        requirements: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            entity_type,
            fields,
            docs,
            requirements,
        }
    }
}

/// Lightweight constructor for FunctionSpec
impl FunctionSpec {
    pub fn new(
        name: impl Into<String>,
        return_type: impl Into<String>,
        parameters: Vec<Parameter>,
        docs: Vec<String>,
        requirements: Vec<String>,
        error_types: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            return_type: return_type.into(),
            parameters,
            docs,
            requirements,
            error_types,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_generation::validation::CodeValidator;

    /// Creates a simple test specification for testing (moved inside mod tests for test-only
    /// compilation)
    fn create_test_specification() -> ParsedSpecification {
        ParsedSpecification {
            requirements: vec![Requirement::new(
                "REQ-001",
                "The system must manage users",
                1,
                vec![],
            )],
            patterns:     vec![ArchitecturalPattern::new(
                "Layered",
                1.0f32,
                "Standard layered architecture pattern",
                vec![],
            )],
            entities:     vec![Entity::new(
                "User",
                EntityType::Struct,
                vec![
                    Field::new("id", "String", false, vec!["Unique identifier".to_string()]),
                    Field::new(
                        "name",
                        "String",
                        false,
                        vec!["User's full name".to_string()],
                    ),
                ],
                vec!["Represents a user in the system".to_string()],
                vec!["REQ-001".to_string()],
            )],
            functions:    vec![FunctionSpec::new(
                "create_user",
                "Result<User, String>",
                vec![Parameter::new("name", "String", false, false)],
                vec!["Creates a new user with the given name".to_string()],
                vec!["REQ-001".to_string()],
                vec!["String".to_string()],
            )],
        }
    }

    #[test]
    fn test_create_test_specification() {
        let spec = create_test_specification();
        assert!(!spec.requirements.is_empty());
        assert!(!spec.patterns.is_empty());
        assert!(!spec.entities.is_empty());
        assert!(!spec.functions.is_empty());
        assert_eq!(spec.entities.len(), 1);
        assert_eq!(spec.functions.len(), 1);
        assert_eq!(spec.entities[0].name, "User");
        assert_eq!(spec.entities[0].fields.len(), 2);
        assert_eq!(spec.entities[0].fields[0].name, "id");
        assert_eq!(spec.entities[0].fields[1].name, "name");
        assert_eq!(spec.functions[0].name, "create_user");
        assert_eq!(spec.functions[0].parameters.len(), 1);
        assert_eq!(spec.functions[0].parameters[0].name, "name");

        // Validate the test specification
        let validator = CodeValidator::new();
        let validation = validator.validate_specification(&spec);
        assert!(validation.is_valid, "Test spec should be valid");
    }

    #[test]
    fn test_invalid_specification() {
        let invalid_spec = ParsedSpecification {
            requirements: vec![], // No requirements
            patterns:     vec![],
            entities:     vec![],
            functions:    vec![],
        };
        let validator = CodeValidator::new();
        let validation = validator.validate_specification(&invalid_spec);
        assert!(!validation.is_valid, "Empty spec should be invalid");
    }
}
