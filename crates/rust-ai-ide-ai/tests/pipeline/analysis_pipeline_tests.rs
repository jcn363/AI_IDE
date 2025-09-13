//! Tests for the analysis pipeline and its components

use rust_ai_ide_ai::{
    analysis::{
        architectural::{
            CircularDependencyAnalyzer, DependencyInversionAnalyzer, InterfaceSegregationAnalyzer,
            LayerViolationDetector,
        },
        security::{HardcodedSecretsDetector, InsecureCryptoDetector, SqlInjectionDetector},
        AnalysisPipeline, AnalysisPipelineBuilder, AnalysisResult, AnalysisType, Severity,
    },
    test_helpers::*,
};
use std::path::PathBuf;

/// Test the basic pipeline construction and execution
#[test]
fn test_basic_pipeline_execution() {
    // Create a simple pipeline with just the circular dependency analyzer
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .build();

    // Test code with a circular dependency
    let code = r#"
        mod a {
            use super::b;
            pub fn a() { b::b(); }
        }

        mod b {
            use super::a;
            pub fn b() { a::a(); }
        }
    "#;

    let result = pipeline.analyze_code(code, "test.rs").unwrap();

    // Should find the circular dependency
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected"
    );
}

/// Test the pipeline with multiple analyzers
#[test]
fn test_pipeline_with_multiple_analyzers() {
    // Create a pipeline with multiple analyzers
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_architectural_analyzer(LayerViolationDetector::default())
        .with_security_analyzer(InsecureCryptoDetector::default())
        .build();

    // Test code with multiple issues
    let code = r#"
        // Circular dependency
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { a::a(); } }

        // Layer violation
        mod domain {
            use super::infrastructure::Database;
            pub struct Service { db: Database }
        }
        mod infrastructure {
            use super::domain::Service;
            pub struct Database;
        }

        // Insecure crypto
        fn hash_password(pwd: &str) -> String {
            let hash = md5::compute(pwd.as_bytes());
            format!("{:x}", hash)
        }
    "#;

    let result = pipeline.analyze_code(code, "test.rs").unwrap();

    // Should find all three types of issues
    let mut found_circular = false;
    let mut found_layer = false;
    let mut found_crypto = false;

    for finding in &result.findings {
        match finding.analysis_type {
            AnalysisType::CircularDependency => found_circular = true,
            AnalysisType::LayerViolation => found_layer = true,
            AnalysisType::InsecureCrypto => found_crypto = true,
            _ => {}
        }
    }

    assert!(found_circular, "Should find circular dependency");
    assert!(found_layer, "Should find layer violation");
    assert!(found_crypto, "Should find insecure crypto");
}

/// Test file analysis with the pipeline
#[test]
fn test_file_analysis() {
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .build();

    // Create a test file
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(
        &file_path,
        "mod a { pub fn a() { b::b(); } } mod b { pub fn b() { a::a(); } }",
    )
    .unwrap();

    // Analyze the file
    let result = pipeline.analyze_file(file_path.to_str().unwrap()).unwrap();

    // Should find the circular dependency
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected"
    );

    // Clean up
    temp_dir.close().expect("Failed to clean up temp dir");
}

/// Test project analysis with the pipeline
#[test]
fn test_project_analysis() {
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_security_analyzer(HardcodedSecretsDetector::default())
        .build();

    // Create a test project
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // Create multiple source files
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
            pub mod a;
            pub mod b;
            const API_KEY: &str = "sk_test_1234567890";
        "#,
    )
    .unwrap();

    std::fs::write(src_dir.join("a.rs"), "use crate::b; pub fn a() { b::b(); }").unwrap();

    std::fs::write(src_dir.join("b.rs"), "use crate::a; pub fn b() { a::a(); }").unwrap();

    // Analyze the project
    let results = pipeline
        .analyze_project(temp_dir.path().to_str().unwrap())
        .unwrap();

    // Should analyze all files
    assert_eq!(results.len(), 3, "Should analyze all source files");

    // Check lib.rs for hardcoded secret
    let lib_results = results.get("src/lib.rs").expect("lib.rs results not found");
    assert_finding!(
        lib_results,
        AnalysisType::HardcodedSecret,
        Severity::High,
        "Potential hardcoded API key detected"
    );

    // Check a.rs and b.rs for circular dependency
    let a_results = results.get("src/a.rs").expect("a.rs results not found");
    let b_results = results.get("src/b.rs").expect("b.rs results not found");

    let mut found_circular = false;
    for finding in &a_results.findings {
        if finding.analysis_type == AnalysisType::CircularDependency {
            found_circular = true;
            break;
        }
    }

    assert!(found_circular, "Should find circular dependency in a.rs");

    found_circular = false;
    for finding in &b_results.findings {
        if finding.analysis_type == AnalysisType::CircularDependency {
            found_circular = true;
            break;
        }
    }

    assert!(found_circular, "Should find circular dependency in b.rs");

    // Clean up
    temp_dir.close().expect("Failed to clean up temp dir");
}

/// Test the pipeline with custom configuration
#[test]
fn test_pipeline_with_configuration() {
    // Create a pipeline with custom severity levels
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_severity(AnalysisType::CircularDependency, Severity::Error) // Treat as error
        .build();

    let code = r#"
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { a::a(); } }
    "#;

    let result = pipeline.analyze_code(code, "test.rs").unwrap();

    // Should find the circular dependency with ERROR severity
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Error, // Now an error instead of warning
        "Circular dependency detected"
    );
}

/// Test the pipeline with file filters
#[test]
fn test_pipeline_with_file_filters() {
    // Create a pipeline that only analyzes files matching a pattern
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_file_filter(|path| path.ends_with("_test.rs") || path.ends_with("/tests/"))
        .build();

    // Create test files
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let src_dir = temp_dir.path().join("src");
    let tests_dir = temp_dir.path().join("tests");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir_all(&tests_dir).unwrap();

    // Main source file (should be ignored)
    std::fs::write(
        src_dir.join("lib.rs"),
        "mod a { pub fn a() { b::b(); } } mod b { pub fn b() { a::a(); } }",
    )
    .unwrap();

    // Test file (should be analyzed)
    std::fs::write(
        tests_dir.join("test_file.rs"),
        "mod c { pub fn c() { d::d(); } } mod d { pub fn d() { c::c(); } }",
    )
    .unwrap();

    // Analyze the project
    let results = pipeline
        .analyze_project(temp_dir.path().to_str().unwrap())
        .unwrap();

    // Should only analyze the test file
    assert_eq!(results.len(), 1, "Should only analyze test files");
    assert!(
        results.contains_key("tests/test_file.rs"),
        "Should analyze test files"
    );

    // Clean up
    temp_dir.close().expect("Failed to clean up temp dir");
}

/// Test parallel analysis execution
#[test]
fn test_parallel_analysis() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // Create a custom analyzer that counts executions
    #[derive(Default)]
    struct CountingAnalyzer {
        count: Arc<AtomicUsize>,
    }

    impl Analysis for CountingAnalyzer {
        type Finding = AnalysisFinding;

        fn analyze(&self, _ast: &syn::File, _code: &str, _path: &str) -> Vec<Self::Finding> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Vec::new()
        }

        fn analysis_type(&self) -> AnalysisType {
            AnalysisType::Custom("counting".to_string())
        }
    }

    // Create a pipeline with the counting analyzer
    let count = Arc::new(AtomicUsize::new(0));
    let pipeline = AnalysisPipelineBuilder::new()
        .with_custom_analyzer(CountingAnalyzer {
            count: count.clone(),
        })
        .with_parallel_execution(true) // Enable parallel execution
        .build();

    // Create multiple test files
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    for i in 0..10 {
        std::fs::write(
            src_dir.join(format!("file{}.rs", i)),
            format!("fn test{}() {{}}", i),
        )
        .unwrap();
    }

    // Analyze the project
    let results = pipeline
        .analyze_project(temp_dir.path().to_str().unwrap())
        .unwrap();

    // Should analyze all files
    assert_eq!(results.len(), 10, "Should analyze all files");

    // The analyzer should have run once per file
    assert_eq!(
        count.load(Ordering::SeqCst),
        10,
        "Analyzer should run once per file"
    );

    // Clean up
    temp_dir.close().expect("Failed to clean up temp dir");
}

/// Test error handling in the pipeline
#[test]
fn test_pipeline_error_handling() {
    // Create a pipeline with a failing analyzer
    #[derive(Default)]
    struct FailingAnalyzer;

    impl Analysis for FailingAnalyzer {
        type Finding = AnalysisFinding;

        fn analyze(&self, _ast: &syn::File, _code: &str, _path: &str) -> Vec<Self::Finding> {
            panic!("Analyzer failed!");
        }

        fn analysis_type(&self) -> AnalysisType {
            AnalysisType::Custom("failing".to_string())
        }
    }

    let pipeline = AnalysisPipelineBuilder::new()
        .with_custom_analyzer(FailingAnalyzer)
        .with_error_handling(true) // Enable error handling
        .build();

    // Analyze some code
    let result = pipeline.analyze_code("fn main() {}", "test.rs");

    // Should return an error result instead of panicking
    assert!(result.is_err(), "Should handle analyzer errors gracefully");
}

/// Test the pipeline with custom output formatting
#[test]
fn test_pipeline_output_formatting() {
    use rust_ai_ide_ai::analysis::output::OutputFormat;

    // Create a pipeline with JSON output
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_output_format(OutputFormat::Json)
        .build();

    // Analyze some code
    let code = r#"
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { a::a(); } }
    "#;

    let result = pipeline.analyze_code(code, "test.rs").unwrap();

    // Convert to JSON
    let json_output = result.to_json().expect("Failed to convert to JSON");

    // Should be valid JSON
    assert!(
        json_output.contains("findings"),
        "JSON output should contain findings"
    );
    assert!(
        json_output.contains("Circular dependency detected"),
        "JSON output should contain the finding message"
    );
}

/// Test the pipeline with custom result processing
#[test]
fn test_pipeline_result_processing() {
    // Create a custom result processor
    let processed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let processed_clone = processed.clone();

    let processor = move |result: &AnalysisResult| {
        // Mark that we processed the result
        processed_clone.store(true, std::sync::atomic::Ordering::SeqCst);

        // Return a modified result (just for testing)
        let mut modified = result.clone();
        modified.findings[0].message = "Modified by processor".to_string();
        modified
    };

    // Create a pipeline with the result processor
    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_result_processor(Box::new(processor))
        .build();

    // Analyze some code
    let code = r#"
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { a::a(); } }
    "#;

    let result = pipeline.analyze_code(code, "test.rs").unwrap();

    // The processor should have run
    assert!(
        processed.load(std::sync::atomic::Ordering::SeqCst),
        "Result processor should have run"
    );

    // The result should be modified by the processor
    assert_eq!(
        result.findings[0].message, "Modified by processor",
        "Result should be modified by the processor"
    );
}
