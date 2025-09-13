//! Tests for the analysis registry and its functionality

use rust_ai_ide_ai::analysis::architectural::{
    CircularDependencyAnalyzer, DependencyInversionAnalyzer, InterfaceSegregationAnalyzer, LayerViolationDetector,
};
use rust_ai_ide_ai::analysis::{AnalysisRegistry, AnalysisResult, AnalysisType, Severity};
use rust_ai_ide_ai::test_helpers::*;

#[test]
fn test_register_architectural_analyzers() {
    let mut registry = AnalysisRegistry::default();

    // Register all architectural analyzers
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());
    registry.register_architectural_analyzer(LayerViolationDetector::default());
    registry.register_architectural_analyzer(InterfaceSegregationAnalyzer::default());
    registry.register_architectural_analyzer(DependencyInversionAnalyzer::default());

    // Verify all analyzers are registered
    assert_eq!(registry.architectural_analyzers().len(), 4);
}

#[test]
fn test_analyze_code_with_no_analyzers() {
    let registry = AnalysisRegistry::default();
    let code = r#"
        fn main() {
            println!("Hello, world!");
        }
    "#;

    let result = registry.analyze_code(code, "test.rs").unwrap();

    // With no analyzers, we should get an empty result
    assert_success(&result);
    assert!(
        result.findings.is_empty(),
        "Expected no findings with no analyzers"
    );
}

#[test]
fn test_analyze_code_with_architectural_analyzers() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

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

    let result = registry.analyze_code(code, "test.rs").unwrap();

    // Should detect circular dependency
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected"
    );
}

#[test]
fn test_analyze_code_with_multiple_analyzers() {
    let mut registry = AnalysisRegistry::default();

    // Register multiple analyzers
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());
    registry.register_architectural_analyzer(LayerViolationDetector::default());

    let code = r#"
        // This code has both circular dependencies and potential layer violations
        mod domain {
            use super::infrastructure::Database;

            pub struct Service {
                db: Database,
            }
        }

        mod infrastructure {
            use super::domain;

            pub struct Database;
        }
    "#;

    let result = registry.analyze_code(code, "test.rs").unwrap();

    // Should find both circular dependency and layer violation
    let circular_dep_found = result
        .findings
        .iter()
        .any(|f| f.analysis_type == AnalysisType::CircularDependency);
    let layer_violation_found = result
        .findings
        .iter()
        .any(|f| f.analysis_type == AnalysisType::LayerViolation);

    assert!(circular_dep_found, "Expected circular dependency finding");
    assert!(layer_violation_found, "Expected layer violation finding");
}

#[test]
fn test_analyze_code_with_invalid_syntax() {
    let registry = AnalysisRegistry::default();
    let code = "fn main() { let x = ; }"; // Invalid syntax

    let result = registry.analyze_code(code, "test.rs");

    // Should return an error for invalid syntax
    assert!(result.is_err(), "Expected error for invalid syntax");
}

#[test]
fn test_analyze_file() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    // Create a test file with circular dependencies
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

    let file_path = create_test_file("registry_tests", "circular_deps.rs", code);
    let result = registry.analyze_file(&file_path).unwrap();

    // Clean up test file
    std::fs::remove_file(file_path).expect("Failed to clean up test file");

    // Should detect circular dependency
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected"
    );
}

#[test]
fn test_analyze_project() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());
    registry.register_architectural_analyzer(LayerViolationDetector::default());

    // Create a test project with multiple files
    let project_files = vec![
        (
            "src/lib.rs",
            r#"
            pub mod domain;
            pub mod infrastructure;
        "#,
        ),
        (
            "src/domain.rs",
            r#"
            use crate::infrastructure::Database;

            pub struct Service {
                db: Database,
            }
        "#,
        ),
        (
            "src/infrastructure.rs",
            r#"
            use crate::domain;

            pub struct Database;
        "#,
        ),
    ];

    let project_path = create_test_project("test_project", &project_files);
    let results = registry.analyze_project(&project_path).unwrap();

    // Clean up test project
    std::fs::remove_dir_all(project_path).expect("Failed to clean up test project");

    // Should analyze all files and find issues
    assert!(
        !results.is_empty(),
        "Expected analysis results for the project"
    );

    // Check that we found both circular dependency and layer violation
    let mut found_circular = false;
    let mut found_layer = false;

    for result in results.values() {
        for finding in &result.findings {
            if finding.analysis_type == AnalysisType::CircularDependency {
                found_circular = true;
            } else if finding.analysis_type == AnalysisType::LayerViolation {
                found_layer = true;
            }
        }

        if found_circular && found_layer {
            break;
        }
    }

    assert!(
        found_circular,
        "Expected to find circular dependency in project analysis"
    );
    assert!(
        found_layer,
        "Expected to find layer violation in project analysis"
    );
}
