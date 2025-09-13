use std::fmt;

use rust_ai_ide_ai::spec_generation::generator::CodeGenerator;
use rust_ai_ide_ai::spec_generation::parser::SpecificationParser;
use rust_ai_ide_ai::spec_generation::system::IntelligentSpecGenerator;
use rust_ai_ide_ai::spec_generation::types::{ParsedSpecification, SpecificationRequest};
use rust_ai_ide_ai::spec_generation::validation::CodeValidator;

const COUNTER_SPEC: &str = r#"
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

#[derive(Debug)]
pub enum VerificationError {
    ParseError(String),
    ValidationError(String),
    GenerationError(String),
    AssertionError(String),
}

impl std::error::Error for VerificationError {}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            VerificationError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            VerificationError::GenerationError(msg) => write!(f, "Generation error: {}", msg),
            VerificationError::AssertionError(msg) => write!(f, "Assertion error: {}", msg),
        }
    }
}

async fn run_parser_test(spec_text: &str) -> Result<ParsedSpecification, VerificationError> {
    println!("1. Testing Parser...");
    let parser = SpecificationParser::new();
    let spec = parser
        .parse_specification(spec_text)
        .await
        .map_err(|e| VerificationError::ParseError(e.to_string()))?;
    println!(
        "✓ Parser test passed! Found {} entities and {} functions.",
        spec.entities.len(),
        spec.functions.len()
    );
    if !spec.entities.iter().any(|e| e.name == "Counter") {
        return Err(VerificationError::AssertionError(
            "Expected entity 'Counter' not found".to_string(),
        ));
    }
    if !spec.functions.iter().any(|f| f.name == "new") {
        return Err(VerificationError::AssertionError(
            "Expected function 'new' not found".to_string(),
        ));
    }
    if !spec.functions.iter().any(|f| f.name == "increment") {
        return Err(VerificationError::AssertionError(
            "Expected function 'increment' not found".to_string(),
        ));
    }
    if !spec.functions.iter().any(|f| f.name == "value") {
        return Err(VerificationError::AssertionError(
            "Expected function 'value' not found".to_string(),
        ));
    }
    Ok(spec)
}

async fn run_validator_test(spec: &ParsedSpecification) -> Result<f64, VerificationError> {
    println!("\n2. Testing Validator...");
    let validator = CodeValidator::new();
    let validation_result = validator.validate_specification(spec);
    println!(
        "✓ Validator test passed! Score: {:.2}",
        validation_result.score
    );
    let threshold = 0.7;
    if validation_result.score <= threshold {
        return Err(VerificationError::AssertionError(format!(
            "Validation score {:.2} is below threshold {:.2}",
            validation_result.score, threshold
        )));
    }
    Ok(validation_result.score)
}

async fn run_generator_test(spec: &ParsedSpecification) -> Result<(), VerificationError> {
    println!("\n3. Testing Generator...");
    let generator = CodeGenerator::new();
    let generated = generator
        .generate_code(spec)
        .await
        .map_err(|e| VerificationError::GenerationError(format!("Failed to generate code: {}", e)))?;
    println!(
        "✓ Generator test passed! Generated {} files and {} resources.",
        generated.files.len(),
        generated.resources.len()
    );
    if generated.files.len() == 0 {
        return Err(VerificationError::AssertionError(
            "No files generated".to_string(),
        ));
    }
    if generated.resources.len() == 0 {
        return Err(VerificationError::AssertionError(
            "No resources generated".to_string(),
        ));
    }
    Ok(())
}

async fn run_end_to_end_test(spec_text: &str) -> Result<(), VerificationError> {
    println!("\n4. Testing End-to-End...");
    let system = IntelligentSpecGenerator::new();
    let request = SpecificationRequest {
        description: spec_text,
        language:    "rust".to_string(),
        context:     None,
    };

    let result = system
        .generate_from_spec(&request)
        .await
        .map_err(|e| VerificationError::GenerationError(format!("Failed to generate from specification: {}", e)))?;
    println!("✓ End-to-End test passed! Generated output:");

    // Added assertion for result.files.len() > 0 as per comment
    if result.files.len() == 0 {
        return Err(VerificationError::AssertionError(
            "No files generated in end-to-end test".to_string(),
        ));
    }

    // Print the first file as an example
    if let Some(file) = result.files.first() {
        println!(
            "\nExample generated file ({}):\n---\n{}\n---",
            file.path,
            &file.content.chars().take(200).collect::<String>() // Print first 200 chars
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), VerificationError> {
    println!("Testing Spec Generation Module");
    println!("=============================\n");

    let spec_text = COUNTER_SPEC;

    let spec = run_parser_test(spec_text).await?;
    let score = run_validator_test(&spec).await?;
    run_generator_test(&spec).await?;
    run_end_to_end_test(spec_text).await?;

    Ok(())
}
