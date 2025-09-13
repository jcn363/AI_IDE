/// Verification harness for the spec generation module.
///
/// This harness tests individual components (utilities, parser, generator)
/// and the full system integration. It ensures critical conditions are met
/// during each step and provides detailed output for debugging.
use rust_ai_ide_ai::spec_generation::{
    generator::{CodeGenerator, GeneratedArtifact},
    parser::SpecificationParser,
    system::IntelligentSpecGenerator,
    test_utils::create_test_specification,
    types::{Specification, SpecificationRequest},
};

const SAMPLE_SPEC: &str = r#"
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

const LANG_RUST: &str = "rust";
const MAX_PREVIEW_CHARS: usize = 200;
const SEPARATOR_WIDTH: usize = 80;

async fn test_utils() -> Result<Specification, String> {
    println!("1. Testing test utility functions...");
    let test_spec = create_test_specification();
    println!(
        "   ✓ Created test specification with {} entities and {} functions",
        test_spec.entities.len(),
        test_spec.functions.len()
    );
    assert!(
        test_spec.entities.len() > 0,
        "Test specification must have entities"
    );
    Ok(test_spec)
}

async fn test_parser() -> Result<(), String> {
    println!("\n2. Testing parser...");
    let parser = SpecificationParser::new();
    let spec_text = SAMPLE_SPEC;

    let spec = parser
        .parse_specification(spec_text)
        .await
        .map_err(|e| e.to_string())?;
    println!("   ✓ Successfully parsed specification");
    println!("     - Found {} entities", spec.entities.len());
    println!("     - Found {} functions", spec.functions.len());
    assert!(
        spec.entities.len() > 0,
        "Parsed specification must have entities"
    );
    Ok(())
}

async fn test_generator(test_spec: &Specification) -> Result<(), String> {
    println!("\n3. Testing code generator...");
    let code_generator = CodeGenerator::new();
    let generated_artifacts = code_generator
        .generate_code(test_spec)
        .await
        .map_err(|e| e.to_string())?;
    println!("   ✓ Successfully generated code");
    println!("     - Generated {} files", generated_artifacts.files.len());
    println!(
        "     - Generated {} resources",
        generated_artifacts.resources.len()
    );
    assert!(
        generated_artifacts.files.len() > 0,
        "Code generator must produce files"
    );
    Ok(())
}

async fn test_system(test_spec: &Specification) -> Result<(), String> {
    println!("\n4. Testing full system...");
    let intelligent_generator = IntelligentSpecGenerator::new();
    let request = SpecificationRequest {
        description: SAMPLE_SPEC.to_string(),
        language:    LANG_RUST.to_string(),
        context:     None,
    };

    let result = intelligent_generator
        .generate_from_spec(&request)
        .await
        .map_err(|e| e.to_string())?;
    println!("   ✓ Successfully executed end-to-end generation");

    if let Some(file) = result.files.first() {
        println!("\n📄 First generated file ({})", file.path);
        println!("{}", "-".repeat(SEPARATOR_WIDTH));
        let preview = file
            .content
            .chars()
            .take(MAX_PREVIEW_CHARS)
            .collect::<String>();
        println!("{}", preview);
        if file.content.len() > MAX_PREVIEW_CHARS {
            println!("... (truncated)");
        }
        println!("{}", "-".repeat(SEPARATOR_WIDTH));
    }

    Ok(())
}

async fn run() -> Result<(), String> {
    let test_spec = test_utils().await?;
    test_parser().await?;
    test_generator(&test_spec).await?;
    test_system(&test_spec).await?;

    println!("\n✅ Spec Generation Module Verification Complete!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("🔍 Verifying Spec Generation Module");
    println!("================================\n");
    run().await?;
    Ok(())
}
