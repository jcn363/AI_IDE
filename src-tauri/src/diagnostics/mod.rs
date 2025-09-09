//! Centralized diagnostic module providing unified types, parsing, and error handling patterns
//!
//! This module consolidates all diagnostic-related functionality from compiler_integration.rs,
//! ai_analysis_commands.rs, and io.rs to eliminate ~47% code duplication.

// Re-export types from rust_ai_ide_lsp
pub use rust_ai_ide_lsp::ChangeType;
pub use rust_ai_ide_lsp::CodeChange;
pub use rust_ai_ide_lsp::CompilerDiagnostic;
pub use rust_ai_ide_lsp::error_resolution::FixSuggestion;

// Define missing types
#[derive(Debug, serde::Deserialize)]
pub struct CompilerDiagnosticsRequest {
    pub workspace_path: String,
    pub include_explanations: bool,
    pub include_suggested_fixes: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct CompilerDiagnosticsResult {
    pub diagnostics: Vec<CompilerDiagnostic>,
    pub explanations: std::collections::HashMap<String, ErrorCodeExplanation>,
    pub suggested_fixes: Vec<FixSuggestion>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CompilerErrorCode {
    pub code: String,
    pub explanation: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CompilerSpan {
    pub file_name: String,
    pub byte_start: u32,
    pub byte_end: u32,
    pub line_start: u32,
    pub line_end: u32,
    pub column_start: u32,
    pub column_end: u32,
    pub is_main_span: bool,
    pub text: Vec<SpanText>,
    pub label: Option<String>,
    pub suggested_replacement: Option<String>,
    pub suggestion_applicability: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DocumentationLink {
    pub title: String,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ErrorCodeExplanation {
    pub error_code: String,
    pub title: String,
    pub explanation: String,
    pub examples: Vec<ErrorExample>,
    pub documentation_links: Vec<DocumentationLink>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ErrorExample {
    pub title: String,
    pub code: String,
    pub explanation: String,
}

pub type EstimatedEffort = String;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SpanText {
    pub text: String,
    pub highlight_start: u32,
    pub highlight_end: u32,
}

// Export FixType
pub use rust_ai_ide_lsp::error_resolution::FixType;

// Keep the backward compatibility
pub use self::ChangeType as CompilerChangeType;

// Submodules
pub mod parsing;
pub mod caching;
pub mod error_handling;

// Re-export commonly used types from submodules for convenience
pub use parsing::*;
pub use caching::*;
pub use error_handling::*;