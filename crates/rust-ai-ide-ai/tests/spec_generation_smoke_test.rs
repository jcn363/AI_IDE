use rust_ai_ide_ai::spec_generation::{
    generator::CodeGenerator,
    parser::SpecificationParser,
    system::IntelligentSpecGenerator,
    types::{Entity, EntityType, Field, FunctionSpec, ParsedSpecification, SpecificationRequest},
    validation::CodeValidator,
};

#[tokio::test]
async fn test_smoke_test() {
    // 1. Create a simple specification
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

    // 2. Test Parser
    let parser = SpecificationParser::new();
    let parse_result = parser.parse_specification(spec_text).await;
    assert!(
        parse_result.is_ok(),
        "Parser failed: {:?}",
        parse_result.err()
    );

    let spec = parse_result.unwrap();
    assert!(!spec.entities.is_empty(), "No entities were parsed");
    assert!(!spec.functions.is_empty(), "No functions were parsed");

    // 3. Test Validator
    let validator = CodeValidator::new();
    let validation = validator.validate_specification(&spec);
    assert!(
        validation.is_valid,
        "Validation failed: {:?}",
        validation.issues
    );

    // 4. Test Generator
    let generator = CodeGenerator::new();
    let generate_result = generator.generate_code(&spec).await;
    assert!(
        generate_result.is_ok(),
        "Generator failed: {:?}",
        generate_result.err()
    );

    let generated = generate_result.unwrap();
    assert!(!generated.files.is_empty(), "No files were generated");

    // 5. Test End-to-End with IntelligentSpecGenerator
    let system = IntelligentSpecGenerator::new();
    let request = SpecificationRequest {
        description: spec_text.to_string(),
        language: "rust".to_string(),
        context: None,
    };

    let result = system.generate_from_spec(&request).await;
    assert!(
        result.is_ok(),
        "End-to-end generation failed: {:?}",
        result.err()
    );

    let generated = result.unwrap();
    assert!(
        !generated.files.is_empty(),
        "No files were generated in end-to-end test"
    );

    println!("✅ All smoke tests passed!");
}
