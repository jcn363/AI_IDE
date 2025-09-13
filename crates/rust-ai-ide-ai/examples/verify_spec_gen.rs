//! Example binary to verify the spec generation module

use rust_ai_ide_ai::spec_generation::{
    generator::CodeGenerator, parser::SpecificationParser, system::IntelligentSpecGenerator,
    types::SpecificationRequest,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Verifying Spec Generation Module");
    println!("================================\n");

    // 1. Test the parser
    println!("1. Testing Parser...");
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

    let spec = parser
        .parse_specification(spec_text)
        .await
        .with_context("Failed to parse specification")?;
    println!("   ✓ Successfully parsed specification");
    println!("     - Found {} entities", spec.entities.len());
    println!("     - Found {} functions", spec.functions.len());

    // 2. Test the generator
    println!("\n2. Testing Code Generator...");
    let generator = CodeGenerator::new();
    let generated = generator
        .generate_code(&spec)
        .await
        .with_context("Failed to generate code")?;
    println!("   ✓ Successfully generated code");
    println!("     - Generated {} files", generated.files.len());
    println!("     - Generated {} resources", generated.resources.len());

    // 3. Test the full system
    println!("\n3. Testing Full System...");
    let system = IntelligentSpecGenerator::new();
    let request = SpecificationRequest {
        description: spec_text.to_string(),
        language: "rust".to_string(),
        context: None,
    };

    let result = system
        .generate_from_spec(&request)
        .await
        .with_context("Failed to generate from specification")?;
    println!("   ✓ Successfully executed end-to-end generation");

    if let Some(file) = result.files.first() {
        println!("\n📄 First generated file ({}):", file.path);
        println!("{}", "-".repeat(80));
        println!("{}", &file.content);
        println!("{}", "-".repeat(80));
    } else {
        println!("No files were generated.");
    }

    println!("\n✅ Spec Generation Module Verification Complete!");
    Ok(())
}
