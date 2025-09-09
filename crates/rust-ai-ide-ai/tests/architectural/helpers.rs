//! Test helpers for architectural analysis tests

use std::path::Path;

use crate::analysis::{
    architectural::{
        CircularDependencyAnalyzer, DependencyInversionAnalyzer, InterfaceSegregationAnalyzer,
        LayerViolationDetector,
    },
    AnalysisRegistry, AnalysisResult, AnalysisType, Severity,
};

/// Create a test analysis registry with all architectural analyzers registered
pub fn create_test_registry() -> AnalysisRegistry {
    let mut registry = AnalysisRegistry::default();
    
    // Register all architectural analyzers
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());
    registry.register_architectural_analyzer(LayerViolationDetector::default());
    registry.register_architectural_analyzer(InterfaceSegregationAnalyzer::default());
    registry.register_architectural_analyzer(DependencyInversionAnalyzer::default());
    
    registry
}

/// Assert that a result contains a finding with the specified properties
#[macro_export]
macro_rules! assert_finding {
    ($result:expr, $type:expr, $severity:expr, $message:expr) => {
        assert!(
            $result.findings.iter().any(|f| {
                f.analysis_type == $type
                    && f.severity == $severity
                    && f.message.contains($message)
            }),
            "Expected finding with type {:?}, severity {:?} and message containing '{}' in {:?}",
            $type,
            $severity,
            $message,
            $result.findings
        )
    };
}

/// Assert that a result does not contain any findings with the specified properties
#[macro_export]
macro_rules! assert_no_finding {
    ($result:expr, $type:expr, $severity:expr, $message:expr) => {
        assert!(
            !$result.findings.iter().any(|f| {
                f.analysis_type == $type
                    && f.severity == $severity
                    && f.message.contains($message)
            }),
            "Unexpected finding with type {:?}, severity {:?} and message containing '{}' in {:?}",
            $type,
            $severity,
            $message,
            $result.findings
        )
    };
}

/// Create a test file path for the given module and file name
pub fn test_file_path(module: &str, file_name: &str) -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fixtures")
        .join(module);
    
    std::fs::create_dir_all(&path).unwrap();
    path = path.join(file_name);
    
    path.to_string_lossy().into_owned()
}

/// Assert that a result is successful and contains no errors
pub fn assert_success(result: &AnalysisResult) {
    assert!(
        result.is_ok(),
        "Expected successful analysis, got errors: {:?}",
        result.errors
    );
    assert!(
        result.errors.is_empty(),
        "Expected no analysis errors, got: {:?}",
        result.errors
    );
}

/// Assert that a result contains errors
pub fn assert_has_errors(result: &AnalysisResult, expected_errors: usize) {
    assert!(
        !result.is_ok() || !result.errors.is_empty(),
        "Expected analysis to have errors, but it succeeded"
    );
    assert_eq!(
        result.errors.len(),
        expected_errors,
        "Expected {} errors, got {}: {:?}",
        expected_errors,
        result.errors.len(),
        result.errors
    );
}

/// Helper function to check for specific findings
pub fn has_findings(result: &AnalysisResult, analysis_type: AnalysisType, severity: Severity) -> bool {
    result.findings.iter().any(|f| f.analysis_type == analysis_type && f.severity == severity)
}

/// Create a test AST from source code
pub fn create_test_ast(source: &str) -> syn::File {
    syn::parse_file(source).expect("Failed to parse test source code")
}

/// Create a test file with the given content and return its path
pub fn create_test_file(module: &str, file_name: &str, content: &str) -> String {
    let path = test_file_path(module, file_name);
    std::fs::write(&path, content).expect("Failed to write test file");
    path
}

/// Remove test files created during testing
pub fn cleanup_test_files(module: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fixtures")
        .join(module);
    
    if path.exists() {
        std::fs::remove_dir_all(&path).expect("Failed to clean up test files");
    }
}

/// Create a test project structure with multiple files
pub fn create_test_project(project_name: &str, files: &[(&str, &str)]) -> String {
    let project_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fixtures")
        .join(project_name);
    
    if project_path.exists() {
        std::fs::remove_dir_all(&project_path).expect("Failed to clean up test project");
    }
    
    std::fs::create_dir_all(&project_path).expect("Failed to create test project directory");
    
    for (file_path, content) in files {
        let file_path = project_path.join(file_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directory for test file");
        }
        std::fs::write(&file_path, content).expect("Failed to write test file");
    }
    
    project_path.to_string_lossy().into_owned()
}
