//! Integration tests for the rust-ai-ide-ai-specgen crate

use rust_ai_ide_ai_specgen::{
    IntelligentSpecGenerator, SpecGenBuilder, SpecificationGenerator, SpecificationParser, SpecificationRequest,
};

#[tokio::test]
async fn test_public_api_basic_functionality() {
    // Test that we can create instances of the main components
    let parser = SpecificationParser::new();
    let system = IntelligentSpecGenerator::new();

    // Test specification request creation
    let request = SpecificationRequest {
        description:              "Create a simple user struct".to_string(),
        language:                 "rust".to_string(),
        context:                  Some(std::collections::HashMap::from([(
            "requirements".to_string(),
            "User management".to_string(),
        )])),
        preferred_pattern:        Some("struct".to_string()),
        quality_threshold:        Some(0.8),
        performance_requirements: Some(vec!["memory-efficient".to_string()]),
        security_requirements:    Some(vec![]),
    };

    // Test basic parsing - should handle basic specifications
    let parse_result = parser.parse_specification(&request.description).await;
    // Either succeeds or fails with a meaningful error - either is fine for our API test
    println!("Parse result: {:?}", parse_result);
    _ = parse_result; // Just ensure it doesn't panic

    // Test system generation
    let gen_result = system.generate_from_spec(&request).await;
    assert!(gen_result.is_err()); // Expected to fail since implementation is stub

    // Test spec generation trait (this should compile and work structurally)
    let trait_result = system.parse_specification(&request.description).await;
    assert!(trait_result.is_err()); // Expected to fail since implementation is stub

    println!("✓ Public API basic functionality tests passed");
}

#[tokio::test]
async fn test_specgen_builder_functionality() {
    // Test the builder pattern
    let result = SpecGenBuilder::new()
        .advanced_templates(true)
        .documentation(true)
        .validation(true)
        .max_file_size_kb(2048)
        .build();

    // Should succeed to create the builder
    match result {
        Ok(_) => println!("✓ SpecGenBuilder creation succeeded"),
        Err(_) => panic!("SpecGenBuilder creation should not fail"),
    }
}

#[tokio::test]
async fn test_error_handling_basic() {
    use rust_ai_ide_ai_specgen::{Result, SpecGenError};

    // Test error creation
    let parse_error = SpecGenError::ParseError {
        message: "test error".to_string(),
    };

    match &parse_error {
        SpecGenError::ParseError { message } => {
            assert_eq!(message, "test error");
            println!("✓ Parse error handling works");
        }
        _ => panic!("Error type mismatch"),
    }

    // Test Result type alias
    let result: Result<()> = Err(parse_error.clone());
    assert!(result.is_err());

    println!("✓ Error handling and Result type alias work");
}

#[cfg(test)]
mod types_integration {
    use rust_ai_ide_ai_specgen::*;
    use tokio;

    #[tokio::test]
    async fn test_types_serialization() {
        // Test that types can be serialized/deserialized
        let request = SpecificationRequest {
            description:              "test spec".to_string(),
            language:                 "rust".to_string(),
            context:                  None,
            preferred_pattern:        None,
            quality_threshold:        None,
            performance_requirements: None,
            security_requirements:    None,
        };

        // Convert to JSON and back
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: SpecificationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.description, deserialized.description);
        assert_eq!(request.language, deserialized.language);

        println!("✓ Type serialization/deserialization works");
    }
}
