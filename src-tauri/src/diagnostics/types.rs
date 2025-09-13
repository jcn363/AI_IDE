//! Centralized diagnostic types module
//!
//! This module contains all diagnostic-related type definitions,
//! serving as the single source of truth across the backend.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Compiler diagnostics request
#[derive(Debug, Deserialize)]
pub struct CompilerDiagnosticsRequest {
    pub workspace_path:          String,
    pub include_explanations:    bool,
    pub include_suggested_fixes: bool,
    pub use_cache:               bool,
    pub cache_ttl_seconds:       Option<u64>,
    pub timeout_seconds:         Option<u64>,
}

/// Compiler diagnostics result
#[derive(Debug, Clone, Serialize)]
pub struct CompilerDiagnosticsResult {
    pub diagnostics:     Vec<CompilerDiagnostic>,
    pub explanations:    HashMap<String, ErrorCodeExplanation>,
    pub suggested_fixes: Vec<FixSuggestion>,
    pub metadata:        DiagnosticMetadata,
}

/// Diagnostic metadata
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticMetadata {
    pub workspace_path:      String,
    pub timestamp:           chrono::DateTime<chrono::Utc>,
    pub compilation_time_ms: u64,
    pub total_errors:        u32,
    pub total_warnings:      u32,
    pub total_notes:         u32,
    pub cached:              bool,
}

/// Compiler diagnostic information
#[derive(Debug, Clone, Serialize)]
pub struct CompilerDiagnostic {
    pub level:    String, // "error", "warning", "note", "help"
    pub message:  String,
    pub code:     Option<CompilerErrorCode>,
    pub spans:    Vec<CompilerSpan>,
    pub children: Vec<CompilerDiagnostic>,
    pub rendered: Option<String>,
    pub context:  DiagnosticContext,
}

/// Diagnostic context information
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticContext {
    pub file_path:           String,
    pub function_name:       Option<String>,
    pub module_path:         Option<String>,
    pub surrounding_code:    Option<String>,
    pub related_diagnostics: Vec<String>,
}

/// Compiler error code
#[derive(Debug, Clone, Serialize)]
pub struct CompilerErrorCode {
    pub code:        String,
    pub explanation: Option<String>,
}

/// Compiler span information
#[derive(Debug, Clone, Serialize)]
pub struct CompilerSpan {
    pub file_name:                String,
    pub byte_start:               u32,
    pub byte_end:                 u32,
    pub line_start:               u32,
    pub line_end:                 u32,
    pub column_start:             u32,
    pub column_end:               u32,
    pub is_main_span:             bool,
    pub text:                     Vec<SpanText>,
    pub label:                    Option<String>,
    pub suggested_replacement:    Option<String>,
    pub suggestion_applicability: Option<String>,
}

/// Span text information
#[derive(Debug, Clone, Serialize)]
pub struct SpanText {
    pub text:            String,
    pub highlight_start: u32,
    pub highlight_end:   u32,
}

/// Error code explanation
#[derive(Debug, Clone, Serialize)]
pub struct ErrorCodeExplanation {
    pub error_code:          String,
    pub title:               String,
    pub explanation:         String,
    pub examples:            Vec<ErrorExample>,
    pub documentation_links: Vec<DocumentationLink>,
    pub related_errors:      Vec<String>,
    pub common_causes:       Vec<String>,
    pub suggested_solutions: Vec<String>,
}

/// Error example
#[derive(Debug, Clone, Serialize)]
pub struct ErrorExample {
    pub description: String,
    pub code:        String,
    pub explanation: String,
    pub fix:         Option<String>,
}

/// Documentation link
#[derive(Debug, Clone, Serialize)]
pub struct DocumentationLink {
    pub title:       String,
    pub url:         String,
    pub description: String,
    pub category:    String, // "official", "community", "tutorial", "reference"
}

/// Fix suggestion
#[derive(Debug, Clone, Serialize)]
pub struct FixSuggestion {
    pub id:               String,
    pub title:            String,
    pub description:      String,
    pub fix_type:         FixType,
    pub changes:          Vec<CodeChange>,
    pub confidence:       f32,
    pub estimated_effort: EstimatedEffort,
    pub benefits:         Vec<String>,
    pub risks:            Vec<String>,
}

/// Fix type enumeration
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixType {
    QuickFix,
    Refactoring,
    CodeGeneration,
    Import,
    Dependency,
}

/// Code change information
#[derive(Debug, Clone, Serialize)]
pub struct CodeChange {
    pub file_path:   String,
    pub range:       (u32, u32, u32, u32), // (start_line, start_col, end_line, end_col)
    pub old_text:    String,
    pub new_text:    String,
    pub change_type: ChangeType,
}

/// Change type enumeration
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Insert,
    Delete,
    Replace,
    Move,
}

/// Estimated effort for applying a fix
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimatedEffort {
    Trivial,
    Easy,
    Moderate,
    Complex,
    Major,
}

/// Real-time diagnostic update
#[derive(Debug, Serialize)]
pub struct DiagnosticUpdate {
    pub stream_id:   String,
    pub update_type: DiagnosticUpdateType,
    pub diagnostics: Option<CompilerDiagnosticsResult>,
    pub timestamp:   chrono::DateTime<chrono::Utc>,
}

/// Diagnostic update type
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticUpdateType {
    Initial,
    Incremental,
    Complete,
    Error,
}

/// Cache for diagnostic results state type
pub type DiagnosticCacheState = std::sync::Arc<tokio::sync::RwLock<DiagnosticCache>>;

/// Cache for error code explanations state type
pub type ExplanationCacheState = std::sync::Arc<tokio::sync::RwLock<ExplanationCache>>;

/// Real-time diagnostic streaming state type
pub type DiagnosticStreamState = std::sync::Arc<tokio::sync::RwLock<HashMap<String, DiagnosticStream>>>;

// Placeholder structs for state - to be implemented separately
pub struct DiagnosticCache;
pub struct ExplanationCache;
pub struct DiagnosticStream;

// Export re-exports for backward compatibility
pub use ChangeType as CompilerChangeType;
