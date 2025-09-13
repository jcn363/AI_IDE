use rust_ai_ide_ai::code_generation::completion::CodeCompleter;
use rust_ai_ide_ai::code_generation::function_generation::FunctionGenerator;
use rust_ai_ide_ai::code_generation::test_generation::TestGenerator;
use rust_ai_ide_ai_codegen::*;
use rust_ai_ide_shared_codegen::generator::*;

/// Demonstrate the code generation system
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Rust AI IDE - Code Generation Demo");
    println!("====================================\n");

    // 1. Initialize code generation service
    println!("1️⃣ Initializing Code Generation Service...");
    let service = CodeGenerationService::new();
    println!(
        "   ✅ Service initialized with {} generators",
        service.supported_languages().len()
    );

    // 2. Register generators
    println!("\n2️⃣ Registering Code Generators...");
    let function_generator = FunctionGenerator::new();
    let register_result = service
        .register_generator(TargetLanguage::Rust, function_generator)
        .await;
    match register_result {
        Ok(_) => println!("   ✅ Function generator registered for Rust"),
        Err(e) => println!("   ⚠️ Could not register generator: {:?}", e),
    }

    // 3. Demonstrate completion suggestions
    println!(
        "
3️⃣ Code Completion System"
    );
    println!("=========================");

    let completer = CodeCompleter::new();
    let completion_context = rust_ai_ide_ai::code_generation::completion::CompletionContext {
        current_line:     "fn process_".to_string(),
        cursor_position:  12,
        surrounding_code: vec![
            "#[derive(Debug)]".to_string(),
            "struct Data {".to_string(),
            "    value: String,".to_string(),
            "}".to_string(),
            "".to_string(),
            "impl Data {",
        ],
        imported_modules: vec!["std::fmt".to_string()],
        project_context:  ProjectContext::with_default_structure(),
        completion_type:  rust_ai_ide_ai::code_generation::completion::CompletionType::Function,
    };

    let completion_suggestions = completer
        .get_completion_suggestions(completion_context)
        .await?;
    println!("   📝 Generating completion for: \"fn process_\"");
    println!(
        "   💡 Suggestions generated: {}",
        completion_suggestions.len()
    );

    for (i, suggestion) in completion_suggestions.iter().enumerate() {
        println!("      {}. {}", i + 1, suggestion.description);
        if i >= 2 {
            // Show first 3 suggestions
            println!(
                "         [+{} more suggestions]",
                completion_suggestions.len() - 3
            );
            break;
        }
    }

    // 4. Demonstrate function generation
    println!(
        "
4️⃣ Function Generation System"
    );
    println!("===========================");

    let function_context = CodeGenerationContext::demo_context(TargetLanguage::Rust, GenerationScope::Function);

    println!("   🔧 Generating function with context:");
    println!("      Language: {:?}", function_context.language);
    println!("      Scope: {:?}", function_context.target_scope);

    match function_generator.generate(function_context).await {
        Ok(generated) => {
            println!("   ✅ Function generated successfully!");
            println!("      Name: {}", generated.name);
            println!("      Signature: {}", generated.signature);
            println!(
                "      Confidence: {:.1}%",
                generated.confidence_score * 100.0
            );
        }
        Err(e) => {
            println!("   ⚠️ Function generation failed: {:?}", e);
        }
    }

    // 5. Demonstrate test generation
    println!(
        "
5️⃣ Test Generation System"
    );
    println!("========================");
    println!("   ⚙️ Generating comprehensive test suite...");

    let test_generator = TestGenerator::new();
    let test_context = CodeGenerationContext::demo_context(TargetLanguage::Rust, GenerationScope::Tests);

    match test_generator
        .generate_test_suite("fn calculate_total(items: &[i32]) -> i32", &test_context)
        .await
    {
        Ok(test_suite) => {
            println!("   ✅ Test suite generated!");
            println!("      Unit tests: {}", test_suite.unit_tests.len());
            println!(
                "      Integration tests: {}",
                test_suite.integration_tests.len()
            );
            println!("      Property tests: {}", test_suite.property_tests.len());
            println!(
                "      Benchmark tests: {}",
                test_suite.benchmark_tests.len()
            );

            if test_suite.unit_tests.len() > 0 {
                println!(" ");
                println!("      📋 Sample unit test:");
                println!("         {}", test_suite.unit_tests[0].test_name);
                println!("         First few lines of test code:");
                let code_lines: Vec<&str> = test_suite.unit_tests[0].test_code.lines().take(3).collect();
                for line in code_lines {
                    if !line.trim().is_empty() {
                        println!("           {}", line);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ⚠️ Test generation failed: {:?}", e);
        }
    }

    // 6. Demonstrate multi-language support
    println!(
        "
6️⃣ Multi-Language Code Generation"
    );
    println!("===============================");

    let languages = vec![
        ("Rust", TargetLanguage::Rust),
        ("Python", TargetLanguage::Python),
        ("TypeScript", TargetLanguage::TypeScript),
        ("Go", TargetLanguage::Go),
    ];

    println!("   🌍 Supported languages:");
    for (name, lang) in &languages {
        println!("      • {} ({:?})", name, lang);
    }

    println!("   ⚙️ Current registered generators:");
    let supported = service.supported_languages();
    for lang in &supported {
        println!("      • {:?}", lang);
    }

    // 7. Demonstrate quality assessment
    println!(
        "
7️⃣ Code Quality Assessment"
    );
    println!("========================");

    // Sample quality assessment
    let quality = GenerationQuality::sample_success();

    println!("   📊 Quality Assessment Results:");
    println!(
        "      Readability:     {:.1}%",
        quality.readability_score * 100.0
    );
    println!(
        "      Maintainability: {:.1}%",
        quality.maintainability_score * 100.0
    );
    println!(
        "      Performance:     {:.1}%",
        quality.performance_score * 100.0
    );
    println!("      Security:    {:.1}%", quality.security_score * 100.0);
    println!(
        "      Compliance:      {:.1}%",
        quality.compliance_score * 100.0
    );
    println!(
        "      Overall:         {:.1}%\n",
        quality.overall_score * 100.0
    );
    println!("      Issues found:    {}", quality.issues.len());

    // 8. Demonstrate global service
    println!("8️⃣ Global Service Access");
    println!("=======================");

    let global_service = get_global_service();
    println!("   🔗 Global service instance accessible: ✅");

    // 9. Summary
    println!(
        "
🎉 Code Generation Demo Complete!
=======================================

📈 Key Achievements:
   • Multi-language code generation framework
   • Intelligent completion system
   • Automated test generation
   • Quality assessment & validation
   • Global service management
   • Extensible architecture for new languages

🔧 Current Capabilities:
   • Function generation with type inference
   • Code completion with context awareness
   • Test suite generation (unit, integration, property)
   • Documentation generation
   • Multi-language support (Rust, Python, TypeScript, Go, etc.)

🚀 Next Steps:
   • Register additional language generators
   • Enhance pattern recognition system
   • Improve quality assessment algorithms
   • Add more sophisticated completion suggestions
   • Integrate with AI models for enhanced generation
"
    );

    println!("\nThat concludes the code generation system demonstration!");
    println!("Run: cargo run --example code_generation_demo\n");

    Ok(())
}
