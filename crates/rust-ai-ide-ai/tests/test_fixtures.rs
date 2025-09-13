pub const USER_MANAGEMENT_SPEC: &str = r#"
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
"#;

/// Test helper function to create a physical time ParsedSpecification with functions
pub fn create_test_user_spec_with_functions(
    functions: Vec<rust_ai_ide_ai::spec_generation::types::FunctionSpec>,
) -> rust_ai_ide_ai::spec_generation::types::ParsedSpecification {
    rust_ai_ide_ai::spec_generation::types::ParsedSpecification {
        requirements: vec![],
        patterns: vec![],
        entities: vec![rust_ai_ide_ai::spec_generation::types::Entity {
            name:         "User".to_string(),
            entity_type:  rust_ai_ide_ai::spec_generation::types::EntityType::Struct,
            fields:       vec![rust_ai_ide_ai::spec_generation::types::Field {
                name:        "id".to_string(),
                field_type:  "String".to_string(),
                is_optional: false,
                docs:        vec!["Unique identifier".to_string()],
            }],
            docs:         vec!["A user in the system".to_string()],
            requirements: vec!["REQ-001".to_string()],
        }],
        functions,
    }
}

/// TestSpecBuilder for fluent construction of ParsedSpecification and Entity instances
pub struct TestSpecBuilder {
    entities:  Vec<rust_ai_ide_ai::spec_generation::types::Entity>,
    functions: Vec<rust_ai_ide_ai::spec_generation::types::FunctionSpec>,
}

impl TestSpecBuilder {
    pub fn new() -> Self {
        TestSpecBuilder {
            entities:  vec![],
            functions: vec![],
        }
    }

    pub fn with_entity(mut self, entity: rust_ai_ide_ai::spec_generation::types::Entity) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn with_function(mut self, function: rust_ai_ide_ai::spec_generation::types::FunctionSpec) -> Self {
        self.functions.push(function);
        self
    }

    pub fn build(self) -> rust_ai_ide_ai::spec_generation::types::ParsedSpecification {
        rust_ai_ide_ai::spec_generation::types::ParsedSpecification {
            requirements: vec![],
            patterns:     vec![],
            entities:     self.entities,
            functions:    self.functions,
        }
    }
}
