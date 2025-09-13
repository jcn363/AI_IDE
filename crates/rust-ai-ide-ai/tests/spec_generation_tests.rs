use rust_ai_ide_ai::spec_generation::{
    generator::CodeGenerator,
    parser::SpecificationParser,
    system::IntelligentSpecGenerator,
    types::{Entity, EntityType, Field, FunctionSpec, ParsedSpecification, SpecificationRequest},
    validation::CodeValidator,
};
// Common constants to avoid duplication
const LANG_RUST: &str = "rust";
const FAILED_PARSE_MSG: &str = "Failed to parse specification";
const FAILED_GENERATE_MSG: &str = "Failed to generate code";
const FAILED_E2E_MSG: &str = "Failed to generate from spec";

mod test_fixtures;


#[tokio::test]
async fn test_spec_parser() {
    let parser = SpecificationParser::new();
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

    let result = parser.parse_specification(spec_text).await;
    assert!(result.is_ok());
    let spec = result.expect(FAILED_PARSE_MSG);
    assert!(!spec.entities.is_empty());
    assert!(!spec.functions.is_empty());
}

#[tokio::test]
async fn test_code_generator() {
    let generator = CodeGenerator::new();
    let spec = test_fixtures::TestSpecBuilder::new()
        .with_entity(test_fixtures::standard_user_entity())
        .with_function(test_fixtures::get_create_user_function())
        .build();

    let generated_files = generator.generate_code(&spec).await.expect(FAILED_GENERATE_MSG);
    assert!(!generated_files.files.is_empty());
    assert!(!generated_files.resources.is_empty());
}

#[tokio::test]
async fn test_validator() {
    let validator = CodeValidator::new();
    let spec = test_fixtures::TestSpecBuilder::new()
        .with_entity(test_fixtures::standard_user_entity())
        .build();

    let result = validator.validate_specification(&spec);
    assert!(result.is_valid);
    assert!(result.score >= 0.95, "Validation score {:.2} is below threshold 0.95", result.score);
    assert!(result.issues.is_empty());
}

#[tokio::test]
async fn test_end_to_end() {
    // Skip test if RUN_SPEC_INTEGRATION_TESTS is not set to true for determinism
    if std::env::var("RUN_SPEC_INTEGRATION_TESTS").unwrap_or_else(|_| String::from("false")) != "true" {
        return;
    }
    let generator = IntelligentSpecGenerator::new();
    let request = SpecificationRequest {
        description: test_fixtures::USER_MANAGEMENT_SPEC.to_string(),
        language: LANG_RUST.to_string(),
        context: None,
    };

    let e2e_generated = generator.generate_from_spec(&request).await.expect(FAILED_E2E_MSG);
    // Assert presence of specific expected files like user service
    assert!(e2e_generated.files.iter().any(|f| f.path.to_lowercase().contains("user")), "No file path contains 'user'");
    assert!(!e2e_generated.files.is_empty(), "Generated files should not be empty");
    assert!(e2e_generated.resources.iter().any(|r| r.resource_type.contains("module") || r.resource_type.contains("service")), "Expected some service or module resource");
    assert!(!e2e_generated.resources.is_empty(), "Generated resources should not be empty");
}
