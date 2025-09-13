//! Tests for incremental analysis functionality

use std::collections::HashMap;

use rust_ai_ide_ai::analysis::architectural::CircularDependencyAnalyzer;
use rust_ai_ide_ai::analysis::incremental::{IncrementalAnalysis, IncrementalAnalysisState};
use rust_ai_ide_ai::analysis::{AnalysisRegistry, AnalysisResult, AnalysisType, Severity};
use rust_ai_ide_ai::test_helpers::*;

/// Test that unchanged files are not re-analyzed
#[test]
fn test_incremental_analysis_skips_unchanged_files() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // First analysis - should process everything
    let files = vec![
        ("file1.rs", "mod a { pub fn a() {} }"),
        ("file2.rs", "mod b { pub fn b() {} }"),
    ];

    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 2, "Should analyze both files initially");

    // Second analysis with no changes - should skip both files
    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 0, "Should skip unchanged files");

    // Modify one file
    let files = vec![
        ("file1.rs", "mod a { pub fn a() { /* modified */ } }"),
        ("file2.rs", "mod b { pub fn b() {} }"),
    ];

    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 1, "Should only analyze modified file");
    assert!(
        results.contains_key("file1.rs"),
        "Should analyze modified file1.rs"
    );
}

/// Test that dependent files are re-analyzed when dependencies change
#[test]
fn test_incremental_analysis_dependencies() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // Initial files with dependency: file2.rs depends on file1.rs
    let files = vec![
        ("file1.rs", "pub mod a { pub fn a() {} }"),
        (
            "file2.rs",
            "use super::file1::a; pub mod b { pub fn b() { super::a::a(); } }",
        ),
    ];

    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 2, "Should analyze both files initially");

    // Modify the dependency (file1.rs)
    let files = vec![
        ("file1.rs", "pub mod a { pub fn a() { /* modified */ } }"),
        (
            "file2.rs",
            "use super::file1::a; pub mod b { pub fn b() { super::a::a(); } }",
        ),
    ];

    let results = analyze_files(&mut incremental, &files);

    // Both files should be re-analyzed because file2.rs depends on file1.rs
    assert_eq!(
        results.len(),
        2,
        "Should analyze both files when dependency changes"
    );
}

/// Test that the analysis state is correctly serialized and deserialized
#[test]
fn test_analysis_state_serialization() {
    let mut state = IncrementalAnalysisState::default();

    // Add some file hashes
    state.update_file_hash("file1.rs", 12345);
    state.update_file_hash("file2.rs", 67890);

    // Add some dependencies
    state.add_dependency("file2.rs", "file1.rs");

    // Serialize and deserialize
    let serialized = serde_json::to_string(&state).expect("Serialization failed");
    let deserialized: IncrementalAnalysisState = serde_json::from_str(&serialized).expect("Deserialization failed");

    // Verify the state was preserved
    assert_eq!(deserialized.get_file_hash("file1.rs"), Some(12345));
    assert_eq!(deserialized.get_file_hash("file2.rs"), Some(67890));
    assert!(deserialized
        .get_dependents("file1.rs")
        .unwrap()
        .contains("file2.rs"));
}

/// Test that the analysis cache is properly invalidated
#[test]
fn test_cache_invalidation() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // Initial analysis
    let files = vec![("file1.rs", "mod a { pub fn a() {} }")];
    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 1, "Should analyze file initially");

    // Invalidate the cache
    incremental.invalidate_cache("file1.rs");

    // Re-analyze - should process the file again
    let results = analyze_files(&mut incremental, &files);
    assert_eq!(
        results.len(),
        1,
        "Should re-analyze file after cache invalidation"
    );
}

/// Test that the analysis handles file deletions
#[test]
fn test_file_deletion() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // Initial analysis with two files
    let files = vec![
        ("file1.rs", "mod a { pub fn a() {} }"),
        ("file2.rs", "mod b { pub fn b() {} }"),
    ];

    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 2, "Should analyze both files initially");

    // Remove one file
    let files = vec![("file1.rs", "mod a { pub fn a() {} }")];

    let results = analyze_files(&mut incremental, &files);

    // The state should be updated to reflect the deletion
    assert!(
        !incremental.state().has_file("file2.rs"),
        "File2 should be removed from state"
    );
}

/// Test that the analysis handles file renames
#[test]
fn test_file_renaming() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // Initial analysis
    let files = vec![("old_name.rs", "mod a { pub fn a() {} }")];
    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 1, "Should analyze file initially");

    // Rename the file
    let files = vec![("new_name.rs", "mod a { pub fn a() {} }")];

    // Need to notify about the rename
    incremental.file_renamed("old_name.rs", "new_name.rs");

    let results = analyze_files(&mut incremental, &files);

    // The state should be updated with the new filename
    assert!(
        !incremental.state().has_file("old_name.rs"),
        "Old filename should be removed"
    );
    assert!(
        incremental.state().has_file("new_name.rs"),
        "New filename should be added"
    );
    assert_eq!(results.len(), 1, "Should analyze renamed file");
}

/// Test that the analysis handles errors gracefully
#[test]
fn test_error_handling() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // File with syntax error
    let files = vec![("error.rs", "fn main() { let x = ; }")];

    let results = analyze_files(&mut incremental, &files);

    // Should return an error result
    assert_eq!(results.len(), 1, "Should process file with error");
    assert!(
        !results["error.rs"].is_ok(),
        "Should have error for invalid syntax"
    );

    // Fix the error
    let files = vec![("error.rs", "fn main() { let x = 42; }")];

    let results = analyze_files(&mut incremental, &files);

    // Should now process successfully
    assert_eq!(results.len(), 1, "Should process fixed file");
    assert!(
        results["error.rs"].is_ok(),
        "Should not have error after fix"
    );
}

/// Helper function to analyze multiple files with the incremental analyzer
fn analyze_files(incremental: &mut IncrementalAnalysis, files: &[(&str, &str)]) -> HashMap<String, AnalysisResult> {
    let mut results = HashMap::new();

    for (path, content) in files {
        let result = incremental.analyze_code(content, path);
        results.insert(path.to_string(), result);
    }

    results
}

/// Test that the analysis handles file modifications correctly
#[test]
fn test_file_modification() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // Initial file
    let files = vec![("mod.rs", "pub mod a; pub mod b;")];
    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 1, "Should analyze file initially");

    // Modify the file
    let files = vec![("mod.rs", "pub mod a; pub mod b; pub mod c;")];

    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 1, "Should analyze modified file");

    // The state should be updated with the new content hash
    let old_hash = incremental.state().get_file_hash("mod.rs").unwrap();
    assert_ne!(old_hash, 0, "File hash should be updated");
}

/// Test that the analysis handles file additions
#[test]
fn test_file_addition() {
    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());

    let mut incremental = IncrementalAnalysis::new(registry);

    // Initial file
    let files = vec![("mod.rs", "pub mod a;")];
    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 1, "Should analyze initial file");

    // Add a new file
    let files = vec![("a.rs", "pub fn a() {}")];

    let results = analyze_files(&mut incremental, &files);
    assert_eq!(results.len(), 1, "Should analyze new file");

    // The state should include the new file
    assert!(
        incremental.state().has_file("a.rs"),
        "New file should be in state"
    );
}
