//! # Test Utilities and Helper Functions
//!
//! Provides common testing utilities for refactoring system integration tests

use std::collections::HashMap;
use rust_ai_ide_ai::refactoring::{
    RefactoringContext, RefactoringOptions, CodeRange, SymbolKind,
    RefactoringType, BackendCapabilitiesResponse, BackendFeatures
};

// Test file paths and content
pub const TEST_RUST_FILE: &str = "tests/data/test_file.rs";
pub const TEST_RUST_CONTENT: &str = r#"fn test_function() {
    let x = 1;
    let y = 2;
    println!("Result: {}", x + y);
}

fn another_function(param: &str) {
    if param.is_empty() {
        return;
    }
    println!("Hello, {}", param);
}
"#;

pub const COMPLEX_RUST_CONTENT: &str = r#"use std::collections::HashMap;

pub struct Calculator {
    pub memory: HashMap<String, f64>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            memory: HashMap::new(),
        }
    }

    pub fn add(&self, a: f64, b: f64) -> f64 {
        a + b
    }

    pub fn multiply(&self, a: f64, b: f64) -> f64 {
        a * b
    }

    pub fn divide(&self, a: f64, b: f64) -> Option<f64> {
        if b == 0.0 {
            None
        } else {
            Some(a / b)
        }
    }

    pub fn calculate_complex_formula(&self, values: &[f64]) -> f64 {
        values.iter().sum()
    }

    pub fn store(&mut self, key: String, value: f64) {
        self.memory.insert(key, value);
    }
}
"#;

/// Create a standard test refactoring context
pub fn create_test_context(file_path: &str, line: usize, column: usize) -> RefactoringContext {
    RefactoringContext {
        file_path: file_path.to_string(),
        cursor_line: line,
        cursor_character: column,
        selection: None,
        symbol_name: Some("test_symbol".to_string()),
        symbol_kind: Some(SymbolKind::Function),
    }
}

/// Create test context with selection
pub fn create_test_context_with_selection(
    file_path: &str,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
) -> RefactoringContext {
    RefactoringContext {
        file_path: file_path.to_string(),
        cursor_line: start_line,
        cursor_character: start_col,
        selection: Some(CodeRange {
            start_line,
            start_character: start_col,
            end_line,
            end_character: end_col,
        }),
        symbol_name: Some("selected_code".to_string()),
        symbol_kind: Some(SymbolKind::Function),
    }
}

/// Create test refactoring options
pub fn create_test_options(create_backup: bool, generate_tests: bool) -> RefactoringOptions {
    RefactoringOptions {
        create_backup,
        generate_tests,
        apply_to_all_occurrences: false,
        preserve_references: true,
        ignore_safe_operations: false,
    }
}

/// Create options with some undefined values to test filtering
pub fn create_options_with_undefined() -> HashMap<String, Option<String>> {
    let mut options = HashMap::new();
    options.insert("createBackup".to_string(), Some("true".to_string()));
    options.insert("generateTests".to_string(), Some("false".to_string()));
    options.insert("invalidOption".to_string(), None); // This should be filtered out
    options.insert("anotherOption".to_string(), Some("".to_string()));
    options
}

/// Filter options map, removing undefined values
pub fn filter_options(options: &HashMap<String, Option<String>>) -> HashMap<String, String> {
    options
        .iter()
        .filter_map(|(key, value)| {
            value.as_ref().and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some((key.clone(), v.clone()))
                }
            })
        })
        .collect()
}

/// Create mock backend capabilities response
pub fn create_mock_capabilities() -> BackendCapabilitiesResponse {
    BackendCapabilitiesResponse {
        supported_refactorings: vec![
            "rename".to_string(),
            "extract-function".to_string(),
            "extract-variable".to_string(),
            "inline-function".to_string(),
            "move-method".to_string(),
        ],
        supported_file_types: vec![
            "rs".to_string(),
            "ts".to_string(),
            "js".to_string(),
        ],
        features: BackendFeatures {
            batch_operations: true,
            analysis: true,
            backup_recovery: true,
            test_generation: false,
            ai_analysis: false,
            lsp_integration: false,
            git_integration: true,
            cross_language_support: true,
            parallel_processing: true,
        },
        performance_metrics: {
            let mut metrics = HashMap::new();
            metrics.insert("fresh_cache_entries".to_string(), 10);
            metrics.insert("total_cache_entries".to_string(), 20);
            metrics.insert("operation_count".to_string(), 5);
            metrics
        },
        configuration_options: vec![
            "create_backup".to_string(),
            "generate_tests".to_string(),
            "apply_to_all_occurrences".to_string(),
        ],
    }
}

/// Mock frontend context structure for testing
pub struct MockFrontendContext {
    pub filePath: String,
    pub startLine: u32,
    pub startCharacter: u32,
    pub endLine: u32,
    pub endCharacter: u32,
    pub selectedText: Option<String>,
    pub symbolName: Option<String>,
    pub symbolKind: Option<String>,
}

/// Create mock frontend context
pub fn create_frontend_test_context() -> MockFrontendContext {
    MockFrontendContext {
        filePath: "src/main.rs".to_string(),
        startLine: 10,
        startCharacter: 5,
        endLine: 15,
        endCharacter: 10,
        selectedText: Some("let x = 42;".to_string()),
        symbolName: Some("calculate".to_string()),
        symbolKind: Some("function".to_string()),
    }
}

/// Map frontend context to backend format
pub fn map_to_backend_context(frontend: &MockFrontendContext) -> RefactoringContext {
    let selection = if frontend.startLine != 0 || frontend.startCharacter != 0 ||
                        frontend.endLine != 0 || frontend.endCharacter != 0 {
        Some(CodeRange {
            start_line: frontend.startLine as usize,
            start_character: frontend.startCharacter as usize,
            end_line: std::cmp::max(frontend.endLine, frontend.startLine) as usize,
            end_character: if frontend.endLine == frontend.startLine {
                std::cmp::max(frontend.endCharacter, frontend.startCharacter) as usize
            } else {
                frontend.endCharacter as usize
            },
        })
    } else {
        None
    };

    // Map symbol kind from frontend string to backend enum
    let symbol_kind = match frontend.symbolKind.as_deref() {
        Some("function") => Some(SymbolKind::Function),
        Some("variable") => Some(SymbolKind::Variable),
        Some("class") => Some(SymbolKind::Class),
        Some("interface") => Some(SymbolKind::Interface),
        Some("method") => Some(SymbolKind::Method),
        Some("struct") => Some(SymbolKind::Struct),
        Some("enum") => Some(SymbolKind::Enum),
        Some("module") => Some(SymbolKind::Module),
        _ => Some(SymbolKind::Function),
    };

    RefactoringContext {
        file_path: frontend.filePath.clone(),
        cursor_line: frontend.startLine as usize,
        cursor_character: frontend.startCharacter as usize,
        selection,
        symbol_name: frontend.symbolName.clone(),
        symbol_kind,
    }
}

/// Assert that two contexts are equivalent for testing
pub fn assert_contexts_equal(a: &RefactoringContext, b: &RefactoringContext) {
    assert_eq!(a.file_path, b.file_path);
    assert_eq!(a.cursor_line, b.cursor_line);
    assert_eq!(a.cursor_character, b.cursor_character);
    assert_eq!(a.symbol_name, b.symbol_name);
    assert_eq!(a.symbol_kind, b.symbol_kind);

    match (&a.selection, &b.selection) {
        (Some(sel_a), Some(sel_b)) => {
            assert_eq!(sel_a.start_line, sel_b.start_line);
            assert_eq!(sel_a.start_character, sel_b.start_character);
            assert_eq!(sel_a.end_line, sel_b.end_line);
            assert_eq!(sel_a.end_character, sel_b.end_character);
        },
        (None, None) => {},
        _ => panic!("Selection mismatch: one context has selection, other doesn't"),
    }
}

/// Test helper to create various refactoring types for testing
pub fn get_all_refactoring_types() -> Vec<RefactoringType> {
    vec![
        RefactoringType::Rename,
        RefactoringType::ExtractFunction,
        RefactoringType::ExtractVariable,
        RefactoringType::ExtractClass,
        RefactoringType::ExtractInterface,
        RefactoringType::InlineFunction,
        RefactoringType::InlineVariable,
        RefactoringType::MoveMethod,
        RefactoringType::MoveClass,
        RefactoringType::RemoveParameter,
        RefactoringType::IntroduceParameter,
        RefactoringType::ConvertToAsync,
        RefactoringType::GenerateGettersSetters,
    ]
}

/// Performance and reliability test utilities
pub struct PerformanceMetrics {
    pub total_operations: i32,
    pub average_response_time_ms: f64,
    pub cache_hit_ratio: f64,
    pub error_rate: f64,
}

pub struct CacheStatistics {
    pub total_entries: i64,
    pub fresh_entries: i64,
    pub stale_entries: i64,
    pub evicted_entries: i64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        PerformanceMetrics {
            total_operations: 0,
            average_response_time_ms: 0.0,
            cache_hit_ratio: 0.0,
            error_rate: 0.0,
        }
    }
}

impl Default for CacheStatistics {
    fn default() -> Self {
        CacheStatistics {
            total_entries: 0,
            fresh_entries: 0,
            stale_entries: 0,
            evicted_entries: 0,
        }
    }
}