//! Integration types for security analyzer
//!
//! This module provides types and structures for integrating
//! the security analyzer with other analysis engines and external tools.

#[derive(Debug, Clone)]
pub struct AnalysisEngineFinding {
    pub message: String,
    pub severity: AnalysisEngineSeverity,
    pub category: AnalysisEngineCategory,
    pub range: AnalysisEngineRange,
    pub suggestion: Option<String>,
    pub confidence: f32,
    pub rule_id: String,
    pub cwe_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum AnalysisEngineSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub enum AnalysisEngineCategory {
    Security,
}

#[derive(Debug, Clone)]
pub struct AnalysisEngineRange {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}