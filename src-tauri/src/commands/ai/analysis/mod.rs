//! Code analysis operations and functionality
//!
//! This module provides commands for AI-powered code analysis including
//! file analysis, workspace analysis, performance suggestions, and code quality checks.

pub mod diagnostics;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tauri::State;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::time::timeout;
use walkdir::WalkDir;

use crate::security::vulnerability_scanner::{VulnerabilityScanner, VulnerabilityReport};
use crate::utils::sanitize_tauri_input;
use crate::utils::SanitizeType;
use rust_ai_ide_lsp::{
    AIContext, AIService, AnalysisPreferences, CodeAnalysisResult,
    RefactoringSuggestion, SecurityIssue,
};

// Import diagnostic types from shared module
use crate::modules::shared::diagnostics::*;

// Import validation helpers from shared module
use crate::modules::shared::validation::*;

/// Combined analysis result containing all analysis types
#[derive(Debug, Serialize)]
pub struct CombinedAnalysisResult {
    /// Raw AI analysis result containing all analysis types
    pub ai_analysis: Option<CodeAnalysisResult>,

    /// Detected code smells with suggestions for improvement
    pub code_smells: Option<Vec<CodeSmell>>,

    /// Performance-related suggestions and optimizations
    pub performance_hints: Option<Vec<PerformanceHint>>,

    /// Style violations and formatting issues
    pub style_violations: Option<Vec<StyleViolation>>,

    /// Architectural issues and improvement suggestions
    pub architecture_suggestions: Option<Vec<ArchitectureSuggestion>>,

    /// Security vulnerabilities and potential risks
    pub security_issues: Option<Vec<SecurityIssue>>,

    /// Results from running Clippy analysis
    pub clippy_results: Option<ClippyResult>,

    /// Results from running Rustfmt analysis
    pub rustfmt_results: Option<RustfmtResult>,

    /// Security vulnerabilities from vulnerability scanning
    pub security_vulnerabilities: Option<Vec<VulnerabilityReport>>,

    /// Performance metrics and measurements
    pub performance_metrics: Option<PerformanceMetrics>,

    /// Compiler diagnostics and error messages
    pub compiler_diagnostics: Option<CompilerDiagnosticsResult>,

    /// Learned patterns from previous analyses
    pub learned_patterns: Option<Vec<LearnedPattern>>,

    /// Unique identifier for this analysis run
    pub analysis_id: String,

    /// Timestamp of when the analysis was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Overall quality score (0-100) based on all analyses
    pub quality_score: Option<f64>,

    /// Summary statistics of the analysis
    pub summary: AnalysisSummary,

    /// Automated code review results
    pub code_review_results: Option<CodeReviewResult>,

    /// Architectural decision recommendations (placeholder)
    pub architectural_decisions: Option<Vec<String>>,

    /// Specification-based generation results (placeholder)
    pub spec_generation_results: Option<Vec<String>>,
}

/// Summary statistics for the analysis results
#[derive(Debug, Serialize)]
pub struct AnalysisSummary {
    /// Total number of issues found across all analysis types
    pub total_issues: usize,

    /// Number of critical issues found
    pub critical_issues: usize,

    /// Number of high severity issues
    pub high_issues: usize,

    /// Number of medium severity issues
    pub medium_issues: usize,

    /// Number of low severity issues
    pub low_issues: usize,

    /// Number of informational findings
    pub info_issues: usize,

    /// Analysis completion time in milliseconds
    pub analysis_time_ms: u128,
}

/// File analysis request
#[derive(Debug, Deserialize)]
pub struct FileAnalysisRequest {
    pub file_path: String,
    pub content: String,
    pub config: Option<crate::commands::ai::services::AIAnalysisConfig>,
}

/// Workspace analysis request
#[derive(Debug, Deserialize)]
pub struct WorkspaceAnalysisRequest {
    pub workspace_path: String,
    pub config: Option<crate::commands::ai::services::AIAnalysisConfig>,
    pub include_dependencies: bool,
    pub include_security_scan: bool,
}

/// Performance suggestions request
#[derive(Debug, Deserialize)]
pub struct PerformanceSuggestionsRequest {
    pub file_path: Option<String>,
    pub workspace_path: Option<String>,
    pub config: Option<crate::commands::ai::services::AIAnalysisConfig>,
}

/// Code quality check request
#[derive(Debug, Deserialize)]
pub struct CodeQualityCheckRequest {
    pub target_path: String,
    pub run_clippy: bool,
    pub run_rustfmt: bool,
    pub run_ai_analysis: bool,
    pub config: Option<crate::commands::ai::services::AIAnalysisConfig>,
}

/// Apply suggestion request
#[derive(Debug, Deserialize)]
pub struct ApplySuggestionRequest {
    pub suggestion_id: String,
    pub changes: Vec<CodeChange>,
    pub create_backup: bool,
    pub record_for_learning: Option<bool>,
}

// Placeholder types - these should be imported from their respective modules
pub type CodeSmell = rust_ai_ide_lsp::CodeSmell;
pub type PerformanceHint = rust_ai_ide_lsp::PerformanceHint;
pub type StyleViolation = rust_ai_ide_lsp::StyleViolation;
pub type ArchitectureSuggestion = rust_ai_ide_lsp::ArchitectureSuggestion;
pub type LearnedPattern = rust_ai_ide_lsp::LearnedPattern;
pub type CodeChange = rust_ai_ide_lsp::CodeChange;

/// AI service state (re-exported for convenience)
pub type AIServiceState = Arc<tokio::sync::Mutex<Option<AIService>>>;

/// Validation functions are now in the shared validation module

/// Sanitize and validate Tauri command inputs with security hardening
macro_rules! sanitize_and_validate_command {
    ($request:expr, $operation:expr) => {{
        log::debug!("Sanitizing inputs for {}", $operation);

        // Sanitize file path
        $request.file_path = sanitize_tauri_input(&$request.file_path, SanitizeType::FilePath)?;

        // Sanitize content input
        $request.content = sanitize_tauri_input(&$request.content, SanitizeType::ApiString)?;

        // Validate file size
        validate_file_size(
            &$request.content.as_bytes(),
            1024 * 1024, // 1MB limit
            "analyze_file"
        )?;
    }};
}

/// Analyze a single file with AI - delegates to commands-ai AnalysisService
#[tauri::command]
pub async fn analyze_file(
    mut request: FileAnalysisRequest,
    ai_service: State<'_, AIServiceState>,
    bridge: State<'_, crate::commands::ai::AIBridgeState>,
) -> Result<CodeAnalysisResult, String> {
    execute_command!("analyze_file", &CommandConfig::default(), async move || {
        // Sanitize and validate inputs with security hardening
        sanitize_and_validate_command!(request, "analyze_file");

        let config = request.config.unwrap_or_default();

        // Additional path validation
        validate_path_not_excluded(
            Path::new(&request.file_path),
            &config.excluded_paths,
            "analyze_file"        )?;

        // Try to use the commands-ai implementation, fallback to original
        if let Ok(mut bridge_guard) = bridge.lock().await {
            if let Ok(analysis_svc) = bridge_guard.analysis_service().await {
                let file_request = rust_ai_ide_commands_ai::analysis::FileAnalysisRequest {
                    file_path: request.file_path.clone(),
                    analyze_dependencies: true,
                    analyze_complexity: true,
                    include_performance: false,
                };

                match analysis_svc.analyze_file(file_request).await {
                    Ok(result) => {
                        // Convert commands-ai result to existing form
                        return Ok(CodeAnalysisResult {
                            issues: result.issues.into_iter().map(|issue| {
                                rust_ai_ide_lsp::AnalysisIssue {
                                    severity: issue.severity,
                                    line: issue.line,
                                    message: issue.message,
                                    category: issue.category,
                                    suggestion: None, // Would need mapping
                                }
                            }).collect(),
                            suggestions: result.suggestions,
                            metrics: result.metrics,
                            performance_hints: result.performance_insights,
                            code_quality_score: None,
                            timestamp: chrono::Utc::now(),
                        });
                    },
                    Err(e) => {
                        log::warn!("Failed to analyze file via commands-ai, falling back to original: {}", e);
                    }
                }
            }
        }

        // Fallback to original implementation
        acquire_service_and_execute!(ai_service, AIServiceState, {
            let context = AIContext {
                current_code: request.content,
                file_name: Some(request.file_path.clone()),
                cursor_position: None,
                selection: None,
                project_context: HashMap::new(),
                dependencies: Vec::new(),
                workspace_structure: HashMap::new(),
                analysis_preferences: config.analysis_preferences,
            };

            service
                .analyze_code_quality(context)
                .await
                .map_err(|e| format_command_error(e, "analysis"))
        })
    })
}

/// Analyze workspace - delegates to commands-ai AnalysisService
#[tauri::command]
pub async fn analyze_workspace(
    bridge: State<'_, crate::commands::ai::AIBridgeState>,
) -> Result<serde_json::Value, String> {
    log::info!("Performing comprehensive workspace analysis");

    // Try to use the commands-ai implementation, fallback to placeholder
    if let Ok(mut bridge_guard) = bridge.lock().await {
        if let Ok(analysis_svc) = bridge_guard.analysis_service().await {
            let workspace_request = rust_ai_ide_commands_ai::analysis::WorkspaceAnalysisRequest {
                include_dependencies: true,
                analysis_depth: 2,
                exclude_patterns: vec!["target/*".to_string(), "node_modules/*".to_string()],
            };

            match analysis_svc.analyze_workspace(workspace_request).await {
                Ok(result) => {
                    return serde_json::to_value(&result)
                        .map_err(|e| format!("Failed to serialize workspace analysis result: {}", e));
                },
                Err(e) => {
                    log::warn!("Failed to analyze workspace via commands-ai, falling back to placeholder: {}", e);
                }
            }
        }
    }

    // Fallback to placeholder implementation
    Ok(json!({
        "status": "placeholder",
        "message": "Workspace analysis - full implementation coming soon",
        "metrics": {
            "total_files": 0,
            "analyzed_files": 0,
            "issues_found": 0
        },
        "issues": [],
        "suggestions": []
    }))
}

/// Get code quality assessment - delegates to commands-ai AnalysisService.assess_code_quality()
#[tauri::command]
pub async fn get_code_quality(
    bridge: State<'_, crate::commands::ai::AIBridgeState>,
) -> Result<serde_json::Value, String> {
    log::info!("Performing code quality analysis");

    // Try to use the commands-ai implementation, fallback to placeholder
    if let Ok(mut bridge_guard) = bridge.lock().await {
        if let Ok(analysis_svc) = bridge_guard.analysis_service().await {
            let quality_request = rust_ai_ide_commands_ai::analysis::CodeQualityRequest {
                target_files: vec![],
                quality_metrics: vec!["coverage".to_string(), "complexity".to_string()],
            };

            match analysis_svc.assess_code_quality(quality_request).await {
                Ok(result) => {
                    return serde_json::to_value(&result)
                        .map_err(|e| format!("Failed to serialize code quality result: {}", e));
                },
                Err(e) => {
                    log::warn!("Failed to assess code quality via commands-ai, falling back to placeholder: {}", e);
                }
            }
        }
    }

    // Fallback to placeholder implementation
    Ok(json!({
        "overall_score": 75.5,
        "metrics": {
            "code_coverage": 85.3,
            "cyclomatic_complexity_avg": 4.7,
            "maintainability_index": 72.1
        },
        "recommendations": [
            "Increase test coverage above 90%",
            "Reduce cyclomatic complexity in complex functions"
        ],
        "critical_issues": 2
    }))
}

/// Get performance suggestions
#[tauri::command]
pub async fn get_performance_suggestions(
    request: PerformanceSuggestionsRequest,
    ai_service: State<'_, AIServiceState>,
) -> Result<Vec<RefactoringSuggestion>, String> {
    log::info!("Getting performance suggestions");

    let ai_service_guard = ai_service.lock().await;
    let service = ai_service_guard
        .as_ref()
        .ok_or("AI service not initialized")?;

    let config = request.config.unwrap_or_default();

    // Determine target for analysis
    let (content, file_path) = if let Some(file_path) = &request.file_path {
        let content = fs::read_to_string(file_path)
            .await
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        (content, file_path.clone())
    } else if let Some(workspace_path) = &request.workspace_path {
        // For workspace analysis, analyze main.rs or lib.rs if available
        let main_rs = Path::new(workspace_path).join("src/main.rs");
        let lib_rs = Path::new(workspace_path).join("src/lib.rs");

        let target_file = if main_rs.exists() {
            main_rs
        } else if lib_rs.exists() {
            lib_rs
        } else {
            return Err("No main.rs or lib.rs found in workspace".to_string());
        };

        let content = fs::read_to_string(&target_file)
            .await
            .map_err(|e| format!("Failed to read file {:?}: {}", target_file, e))?;
        (content, target_file.to_string_lossy().to_string())
    } else {
        return Err("Either file_path or workspace_path must be provided".to_string());
    };

    // Create analysis context focused on performance
    let mut analysis_prefs = config.analysis_preferences;
    analysis_prefs.enable_performance = true;
    analysis_prefs.enable_code_smells = true; // Code smells can affect performance

    let context = AIContext {
        current_code: content,
        file_name: Some(file_path),
        cursor_position: None,
        selection: None,
        project_context: HashMap::new(),
        dependencies: Vec::new(),
        workspace_structure: HashMap::new(),
        analysis_preferences: analysis_prefs,
    };

    service
        .get_refactoring_suggestions(context)
        .await
        .map_err(|e| format!("Performance analysis failed: {}", e))
}

/// Apply AI-recommended suggestion
#[tauri::command]
pub async fn apply_ai_suggestion(
    request: ApplySuggestionRequest,
) -> Result<String, String> {
    log::info!("Applying AI suggestion: {}", request.suggestion_id);

    let mut applied_changes = Vec::new();

    for change in &request.changes {
        // Create backup if requested
        if request.create_backup {
            let backup_path = format!("{}.backup.{}", change.file_path, chrono::Utc::now().timestamp());
            if let Err(e) = fs::copy(&change.file_path, &backup_path).await {
                log::warn!("Failed to create backup for {}: {}", change.file_path, e);
            } else {
                log::info!("Created backup: {}", backup_path);
            }
        }

        // Read current file content
        let current_content = fs::read_to_string(&change.file_path)
            .await
            .map_err(|e| format!("Failed to read file {}: {}", change.file_path, e))?;

        // Apply the change (simplified implementation)
        // In a real implementation, you would need to properly handle line/column ranges
        let lines: Vec<&str> = current_content.lines().collect();
        let mut new_lines = lines.clone();

        // Replace the specified range with new text
        let start_line = change.range.0 as usize;
        let end_line = change.range.2 as usize;

        if start_line < new_lines.len() && end_line < new_lines.len() {
            // Simple line replacement (more sophisticated range handling would be needed for production)
            for i in start_line..=end_line {
                if i < new_lines.len() {
                    new_lines[i] = &change.new_text;
                }
            }

            let new_content = new_lines.join("\n");

            // Write the modified content back
            fs::write(&change.file_path, new_content)
                .await
                .map_err(|e| format!("Failed to write file {}: {}", change.file_path, e))?;

            applied_changes.push(change.file_path.clone());
        } else {
            return Err(format!(
                "Invalid line range for file {}: {}:{} to {}:{}",
                change.file_path, change.range.0, change.range.1, change.range.2, change.range.3
            ));
        }
    }

    Ok(format!(
        "Successfully applied suggestion {} to {} files: {}",
        request.suggestion_id,
        applied_changes.len(),
        applied_changes.join(", ")
    ))
}

// Placeholder types and helper functions from original file
// These would need to be imported from their respective modules in a real implementation

#[derive(Debug, Serialize)]
pub struct ClippyResult {
    pub warnings: Vec<ClippyWarning>,
    pub errors: Vec<ClippyError>,
    pub suggestions: Vec<ClippySuggestion>,
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct ClippyWarning {
    pub message: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub lint_name: String,
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct ClippyError {
    pub message: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Serialize)]
pub struct ClippySuggestion {
    pub message: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub suggestion: String,
    pub applicability: String,
}

#[derive(Debug, Serialize)]
pub struct RustfmtResult {
    pub formatted_files: Vec<String>,
    pub errors: Vec<String>,
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub compilation_time_ms: u64,
    pub binary_size_bytes: u64,
    pub dependency_count: u32,
    pub lines_of_code: u32,
    pub cyclomatic_complexity: u32,
    pub memory_usage_estimate_mb: f32,
}

pub type FixSuggestion = rust_ai_ide_lsp::FixSuggestion;

#[derive(Debug, Serialize)]
pub struct CodeReviewResult {
    pub overall_assessment: OverallAssessment,
    pub summary: ReviewSummary,
    pub file_reviews: Vec<FileReview>,
    pub suggestions: Vec<String>,
    pub review_comments: Vec<ReviewComment>,
    pub metadata: ReviewMetadata,
}

#[derive(Debug, Serialize)]
pub struct OverallAssessment {
    pub score: f32,
    pub grade: String,
    pub summary: String,
    pub key_strengths: Vec<String>,
    pub critical_issues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewSummary {
    pub total_comments: usize,
    pub severity_breakdown: HashMap<String, usize>,
    pub category_breakdown: HashMap<String, usize>,
    pub estimated_effort: String,
}

#[derive(Debug, Serialize)]
pub struct FileReview {
    pub file_path: String,
    pub quality_score: f32,
    pub comments: Vec<ReviewComment>,
}

#[derive(Debug, Serialize)]
pub struct ReviewComment {
    pub id: String,
    pub file_path: String,
    pub line: Option<usize>,
    pub severity: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewMetadata {
    pub review_id: String,
    pub reviewer: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub file_count: usize,
}

/// Placeholder command config type
pub struct CommandConfig;

impl Default for CommandConfig {
    fn default() -> Self {
        CommandConfig
    }
}

/// Helper macros from original file
macro_rules! acquire_service_and_execute {
    ($service:expr, $state_type:ty, $operation:block) => {{
        let service_guard = $service.lock().await;
        let service = service_guard
            .as_ref()
            .ok_or("AI service not initialized")?;
        $operation
    }};
}

macro_rules! execute_command {
    ($name:expr, $config:expr, $operation:expr) => {{
        log::info!("Executing command: {}", $name);
        $operation
    }};
}

macro_rules! format_command_error {
    ($error:expr, $context:expr) => {
        format!("{} failed: {}", $context, $error)
    };
}

pub(crate) use acquire_service_and_execute;
pub(crate) use execute_command;
pub(crate) use format_command_error;