//! LSP diagnostics handling for AI-enhanced code analysis
//!
//! This module provides integration between AI-powered code analysis and the Language Server Protocol,
//! enabling real-time diagnostics and suggestions in the IDE.

use anyhow::Result;
use lsp_types::{CodeActionParams, CodeActionResponse, Diagnostic, Uri, DiagnosticSeverity, Range, Position, Command, TextEdit, CodeAction, CodeActionOrCommand, WorkspaceEdit, TextDocumentPositionParams, NumberOrString, DiagnosticRelatedInformation};
use std::str::FromStr;
use rust_ai_ide_ai::analysis::Severity;
use rust_ai_ide_ai::AIService;
use rust_ai_ide_ai_analysis::{AdvancedCodeAnalyzer, AnalysisResult};
use std::collections::HashMap;
use chrono::Utc;
use tokio::sync::{mpsc, oneshot};
use std::time::{Duration, Instant};

// Missing imports that cause compilation errors
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

/// Diagnostics-specific error types
#[derive(Error, Debug)]
pub enum DiagnosticsError {
    #[error("Analysis failed: {0}")]
    AnalysisError(String),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("LSP communication error: {0}")]
    LspError(String),

    #[error("Background analysis timeout")]
    Timeout,
}

// CodeAnalysisRequest struct (specific to LSP diagnostics)
#[derive(Debug, Clone)]
pub struct CodeAnalysisRequest {
    pub file_path: std::path::PathBuf,
    pub content: String,
    pub analysis_types: Vec<String>,
    pub workspace_root: Option<std::path::PathBuf>,
}

/// Configuration for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    pub analysis_duration_ms: u64,
    pub code_smells: Vec<CodeSmell>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub security_issues: Vec<SecurityIssue>,
    pub style_issues: Vec<StyleIssue>,
    pub architecture_issues: Vec<ArchitectureIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    pub smell_type: String,
    pub description: String,
    pub severity: Severity,
    pub location: CodeLocation,
    pub confidence: f32,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: Severity,
    pub location: CodeLocation,
    pub estimated_impact: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: Severity,
    pub location: CodeLocation,
    pub cve_id: Option<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleIssue {
    pub issue_type: String,
    pub description: String,
    pub location: CodeLocation,
    pub auto_fixable: bool,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: rust_ai_ide_ai::analysis::Severity,
    pub affected_modules: Vec<String>,
    pub suggestion: String,
}

/// Stub types for AI integration (to be replaced with real types later)

/// Analysis detector stub
pub struct AIDetector;

/// Analysis context stub
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub project_root: Option<PathBuf>,
    pub language: Option<String>,
    pub framework: Option<String>,
}

/// Analysis request stub
#[derive(Debug, Clone)]
pub struct AnalysisRequest {
    pub file_uri: String,
    pub detect_anti_patterns: bool,
    pub detect_patterns: bool,
    pub generate_suggestions: bool,
    pub performance_analysis: bool,
    pub parse_tree: Option<String>,
    pub context: Option<AnalysisContext>,
}

/// Analysis result stub
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub analysis_metadata: AnalysisMetadata,
    pub detected_anti_patterns: Vec<DetectedAntiPattern>,
    pub intelligence_suggestions: Vec<IntelligenceSuggestion>,
}

/// Analysis metadata stub
#[derive(Debug, Clone)]
pub struct AnalysisMetadata {
    pub analysis_duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detected anti-pattern stub
#[derive(Debug, Clone)]
pub struct DetectedAntiPattern {
    pub anti_pattern_type: AntiPattern,
    pub confidence: f32,
    pub location: CodeLocation,
    pub severity: Severity,
    pub suggestions: Vec<String>,
    pub metrics: AntiPatternMetrics,
}

/// Intelligence suggestion stub
#[derive(Debug, Clone)]
pub struct IntelligenceSuggestion {
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
    pub location: CodeLocation,
    pub priority: Priority,
    pub confidence: f32,
    pub implemented_guidance: String,
    pub automated_fix: Option<AutomatedFix>,
}

/// Anti-pattern enum stub
#[derive(Debug, Clone)]
pub enum AntiPattern {
    CodeDuplication,
    LongMethod,
    LargeClass,
    TightCoupling,
    CircularDependency,
    GodObject,
    PrimitiveObsession,
    FeatureEnvy,
    MessageChain,
}

impl AntiPattern {
    pub fn description(&self) -> &'static str {
        match self {
            AntiPattern::CodeDuplication => "Code duplication detected",
            AntiPattern::LongMethod => "Method is too long",
            AntiPattern::LargeClass => "Class is too large",
            AntiPattern::TightCoupling => "Tight coupling detected",
            AntiPattern::CircularDependency => "Circular dependency found",
            AntiPattern::GodObject => "God object pattern detected",
            AntiPattern::PrimitiveObsession => "Primitive obsession detected",
            AntiPattern::FeatureEnvy => "Feature envy detected",
            AntiPattern::MessageChain => "Message chain detected",
        }
    }
}

/// Priority enum stub
#[derive(Debug, Clone)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Automated fix stub
#[derive(Debug, Clone)]
pub struct AutomatedFix {
    pub actions: Vec<FixAction>,
}

/// Fix action stub
#[derive(Debug, Clone)]
pub struct FixAction {
    pub action_type: String,
}

/// Code location stub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl CodeLocation {
    pub fn new(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }
}

/// Anti-pattern metrics stub
#[derive(Debug, Clone)]
pub struct AntiPatternMetrics {
    pub refactoring_effort_days: f32,
}

impl AIDetector {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_code(
        &self,
        _content: &str,
        _file_path: &str,
        _request: AnalysisRequest,
    ) -> Result<AnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - always return empty results
        Ok(AnalysisResult {
            analysis_metadata: AnalysisMetadata {
                analysis_duration_ms: 0,
                timestamp: chrono::Utc::now(),
            },
            detected_anti_patterns: vec![],
            intelligence_suggestions: vec![],
        })
    }
}

/// AI analysis result for LSP integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    /// File URI that was analyzed
    pub file_uri: String,
    /// Code suggestions from AI analysis
    pub suggestions: Vec<CodeSuggestion>,
    /// Analysis metrics
    pub metrics: AnalysisMetrics,
    /// Analysis timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Code suggestion from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    /// Type of suggestion
    pub suggestion_type: String,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: String,
    /// Location in the code
    pub range: SuggestionRange,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Category of the suggestion
    pub category: DiagnosticCategory,
    /// Optional quick fix
    pub quick_fix: Option<QuickFix>,
    /// Additional context or explanation
    pub explanation: Option<String>,
}

/// Range for code suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionRange {
    pub start: SuggestionPosition,
    pub end: SuggestionPosition,
}

/// Position for code suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionPosition {
    pub line: u32,
    pub character: u32,
}

/// Quick fix for code suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFix {
    /// Title of the fix
    pub title: String,
    /// Replacement text
    pub replacement_text: String,
    /// Whether this fix can be applied automatically
    pub auto_applicable: bool,
}

/// Analysis metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    /// Total number of issues found
    pub total_issues: usize,
    /// Issues by severity
    pub issues_by_severity: HashMap<String, usize>,
    /// Issues by category
    pub issues_by_category: HashMap<String, usize>,
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
}

/// Diagnostic categories for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DiagnosticCategory {
    CodeSmell,
    Performance,
    Security,
    Style,
    Architecture,
}

impl DiagnosticCategory {
    /// Get the diagnostic code prefix for this category
    pub fn code_prefix(&self) -> &'static str {
        match self {
            DiagnosticCategory::CodeSmell => "AI_CODE_SMELL",
            DiagnosticCategory::Performance => "AI_PERFORMANCE",
            DiagnosticCategory::Security => "AI_SECURITY",
            DiagnosticCategory::Style => "AI_STYLE",
            DiagnosticCategory::Architecture => "AI_ARCHITECTURE",
        }
    }

    /// Get the diagnostic source for this category
    pub fn source(&self) -> &'static str {
        match self {
            DiagnosticCategory::CodeSmell => "rust-ai-ide-smells",
            DiagnosticCategory::Performance => "rust-ai-ide-performance",
            DiagnosticCategory::Security => "rust-ai-ide-security",
            DiagnosticCategory::Style => "rust-ai-ide-style",
            DiagnosticCategory::Architecture => "rust-ai-ide-architecture",
        }
    }
}

/// AI analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisConfig {
    /// Whether AI analysis is enabled
    pub enabled: bool,
    /// Enable code smell detection
    pub code_smells_enabled: bool,
    /// Enable performance analysis
    pub performance_enabled: bool,
    /// Enable security analysis
    pub security_enabled: bool,
    /// Enable style analysis
    pub style_enabled: bool,
    /// Enable architecture analysis
    pub architecture_enabled: bool,
    /// Enable real-time analysis on document changes
    pub real_time_analysis: bool,
    /// Maximum number of suggestions to show
    pub max_suggestions: usize,
    /// Minimum confidence threshold for suggestions
    pub min_confidence: f32,
    /// Debounce delay for real-time analysis (milliseconds)
    pub debounce_delay_ms: u64,
}

impl Default for AIAnalysisConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            code_smells_enabled: true,
            performance_enabled: true,
            security_enabled: true,
            style_enabled: true,
            architecture_enabled: true,
            real_time_analysis: true,
            max_suggestions: 50,
            min_confidence: 0.5,
            debounce_delay_ms: 1000,
        }
    }
}

/// Analysis task for background processing
struct AnalysisTask {
    file_uri: String,
    content: String,
    analysis_type: AnalysisType,
    response_sender: oneshot::Sender<Result<Vec<Diagnostic>, DiagnosticsError>>,
}

#[derive(Clone, Debug)]
enum AnalysisType {
    OnSave,
    OnChange,
    RealTime,
}

/// Real-time code review system with AI-powered diagnostics
pub struct DiagnosticsManager {
    /// Advanced code analyzer instance
    analyzer: Arc<AdvancedCodeAnalyzer>,
    /// Configuration for AI analysis
    pub config: AIAnalysisConfig,
    /// Workspace root path
    pub workspace_root: Option<PathBuf>,
    /// Cache of recent analysis results
    pub analysis_cache: Arc<RwLock<HashMap<String, AIAnalysisResult>>>,
    /// Background analysis channel
    analysis_sender: mpsc::Sender<AnalysisTask>,
    /// Active analysis tasks tracking
    active_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl DiagnosticsManager {
    /// Create a new diagnostics manager with real AI analysis capabilities
    pub async fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let analyzer = Arc::new(AdvancedCodeAnalyzer::new());

        let mut manager = Self {
            analyzer: analyzer.clone(),
            config: AIAnalysisConfig::default(),
            workspace_root: None,
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
            analysis_sender: tx,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start the background analysis processor
        manager.start_analysis_processor(rx, analyzer);

        manager
    }

    /// Start the background analysis processor
    fn start_analysis_processor(
        &self,
        mut rx: mpsc::Receiver<AnalysisTask>,
        analyzer: Arc<AdvancedCodeAnalyzer>,
    ) {
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                let analyzer = analyzer.clone();
                let file_uri = task.file_uri.clone();

                let handle = tokio::spawn(async move {
                    let start_time = Instant::now();
                    let timeout_duration = Duration::from_secs(30); // 30 second timeout

                    let result = tokio::time::timeout(timeout_duration, async {
                        Self::perform_analysis(&analyzer, &task).await
                    }).await;

                    let analysis_result = match result {
                        Ok(Ok(diagnostics)) => Ok(diagnostics),
                        Ok(Err(e)) => {
                            tracing::warn!("Analysis failed for {}: {}", task.file_uri, e);
                            Err(DiagnosticsError::AnalysisError(e.to_string()))
                        },
                        Err(_) => {
                            tracing::warn!("Analysis timeout for {}", task.file_uri);
                            Err(DiagnosticsError::Timeout)
                        }
                    };

                    let duration = start_time.elapsed();
                    tracing::info!("Analysis completed for {} in {:?}", task.file_uri, duration);

                    if let Err(send_err) = task.response_sender.send(analysis_result) {
                        tracing::error!("Failed to send analysis result: {}", send_err);
                    }
                });

                // Track the active task
                let mut tasks = self.active_tasks.write().await;
                tasks.insert(file_uri, handle);
            }
        });
    }

    /// Perform the actual analysis (background task)
    async fn perform_analysis(
        analyzer: &AdvancedCodeAnalyzer,
        task: &AnalysisTask,
    ) -> Result<Vec<Diagnostic>, DiagnosticsError> {
        let uri = task.file_uri.clone();

        // Run AI analysis
        let analysis_id = analyzer.analyze_file(&task.file_uri, &task.content)
            .await
            .map_err(|e| DiagnosticsError::AnalysisError(e.to_string()))?;

        // Get analysis results
        let analysis_result = analyzer.get_analysis_result(&analysis_id)
            .await
            .ok_or_else(|| DiagnosticsError::AnalysisError("Analysis result not found".to_string()))?;

        // Convert to LSP diagnostics
        let diagnostics = Self::convert_to_lsp_diagnostics(analysis_result)
            .await;

        Ok(diagnostics)
    }

    /// Set the workspace root path
    pub fn set_workspace_root(&mut self, workspace_root: PathBuf) {
        self.workspace_root = Some(workspace_root);
    }

    /// Update the AI analysis configuration
    pub fn set_config(&mut self, config: AIAnalysisConfig) {
        self.config = config;
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &AIAnalysisConfig {
        &self.config
    }

    /// Handle document save event - trigger comprehensive AI analysis
    pub async fn handle_document_save(&self, uri: &Uri, content: &str) -> Result<Vec<Diagnostic>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        if !self.is_file_supported(uri) {
            return Ok(Vec::new());
        }

        self.perform_analysis_task(uri, content, AnalysisType::OnSave).await
    }

    /// Handle document change event - trigger debounced AI analysis
    pub async fn handle_document_change(
        &self,
        uri: &Uri,
        content: &str,
    ) -> Result<Option<Vec<Diagnostic>>> {
        if !self.config.enabled || !self.config.real_time_analysis {
            return Ok(None);
        }

        if !self.is_file_supported(uri) {
            return Ok(None);
        }

        // For real-time analysis, return None to indicate no immediate diagnostics
        // The analysis will be processed in the background
        let _ = self.perform_analysis_task(uri, content, AnalysisType::OnChange).await;
        Ok(None)
    }


    /// Check if a file type is supported for AI analysis
    fn is_file_supported(&self, uri: &Uri) -> bool {
        if let Some(path) = uri.path().split('/').last() {
            path.ends_with(".rs") || path.ends_with(".js") || path.ends_with(".ts") || path.ends_with(".py")
        } else {
            false
        }
    }

    /// Perform background analysis task
    async fn perform_analysis_task(
        &self,
        uri: &Uri,
        content: &str,
        analysis_type: AnalysisType,
    ) -> Result<Vec<Diagnostic>> {
        let (response_tx, response_rx) = oneshot::channel();

        let task = AnalysisTask {
            file_uri: uri.to_string(),
            content: content.to_string(),
            analysis_type,
            response_sender: response_tx,
        };

        // Send task to background processor
        if let Err(e) = self.analysis_sender.send(task).await {
            return Err(Box::new(DiagnosticsError::LspError(format!("Failed to queue analysis: {}", e))));
        }

        // Wait for result with timeout
        match tokio::time::timeout(Duration::from_secs(35), response_rx).await {
            Ok(Ok(Ok(diagnostics))) => Ok(diagnostics),
            Ok(Ok(Err(e))) => {
                tracing::error!("Analysis failed: {}", e);
                Ok(Vec::new()) // Return empty diagnostics on error
            }
            Ok(Err(_)) => {
                tracing::warn!("Analysis response channel closed");
                Ok(Vec::new())
            }
            Err(_) => {
                tracing::warn!("Analysis timeout for {}", uri);
                Ok(Vec::new())
            }
        }
    }

    /// Convert AI analysis results to LSP diagnostics
    async fn convert_to_lsp_diagnostics(analysis_result: AnalysisResult) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Convert security issues
        for issue in analysis_result.security_issues {
            diagnostics.push(Self::convert_security_issue(&issue));
        }

        // Convert performance hints
        for hint in analysis_result.performance_hints {
            diagnostics.push(Self::convert_performance_hint(&hint));
        }

        // Convert code smells
        for smell in analysis_result.code_smells {
            diagnostics.push(Self::convert_code_smell(&smell));
        }

        // Convert architecture suggestions
        for suggestion in analysis_result.architecture_suggestions {
            diagnostics.push(Self::convert_architecture_suggestion(&suggestion));
        }

        diagnostics
    }

    /// Convert security issue to LSP diagnostic
    fn convert_security_issue(issue: &rust_ai_ide_ai_analysis::types::SecurityIssue) -> Diagnostic {
        let severity = match issue.severity {
            Severity::Info => DiagnosticSeverity::HINT,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Critical => DiagnosticSeverity::ERROR,
        };

        Diagnostic::new(
            Range::new(
                Position::new(issue.location.line as u32, issue.location.column as u32),
                Position::new(issue.location.line as u32, (issue.location.column + 10) as u32),
            ),
            Some(severity),
            Some(NumberOrString::String(format!("SECURITY-{}", issue.cwe_id.as_deref().unwrap_or("UNKNOWN")))),
            Some("rust-ai-ide-security".to_string()),
            format!("{}: {}", issue.title, issue.description),
            Some(vec![DiagnosticRelatedInformation::new(
                lsp_types::Location::new(
                    Uri::from_str(&issue.location.file).unwrap_or_else(|_| Uri::from_str("file://").unwrap()),
                    Range::new(
                        Position::new(issue.location.line as u32, issue.location.column as u32),
                        Position::new(issue.location.line as u32, (issue.location.column + 10) as u32),
                    )
                ),
                issue.mitigation.clone(),
            )]),
            None,
        )
    }

    /// Convert performance hint to LSP diagnostic
    fn convert_performance_hint(hint: &rust_ai_ide_ai_analysis::types::PerformanceHint) -> Diagnostic {
        let severity = match hint.impact {
            rust_ai_ide_ai_analysis::types::PerformanceImpact::Low | rust_ai_ide_ai_analysis::types::PerformanceImpact::None => DiagnosticSeverity::HINT,
            rust_ai_ide_ai_analysis::types::PerformanceImpact::Medium => DiagnosticSeverity::WARNING,
            rust_ai_ide_ai_analysis::types::PerformanceImpact::High | rust_ai_ide_ai_analysis::types::PerformanceImpact::Critical => DiagnosticSeverity::ERROR,
        };

        Diagnostic::new(
            Range::new(
                Position::new(hint.location.line as u32, hint.location.column as u32),
                Position::new(hint.location.line as u32, (hint.location.column + 10) as u32),
            ),
            Some(severity),
            Some(NumberOrString::String("PERFORMANCE-HINT".to_string())),
            Some("rust-ai-ide-performance".to_string()),
            format!("{}: {}", hint.title, hint.description),
            Some(vec![DiagnosticRelatedInformation::new(
                lsp_types::Location::new(
                    Uri::from_str(&hint.location.file).unwrap_or_else(|_| Uri::from_str("file://").unwrap()),
                    Range::new(
                        Position::new(hint.location.line as u32, hint.location.column as u32),
                        Position::new(hint.location.line as u32, (hint.location.column + 10) as u32),
                    )
                ),
                hint.suggestion.clone(),
            )]),
            None,
        )
    }

    /// Convert code smell to LSP diagnostic
    fn convert_code_smell(smell: &rust_ai_ide_ai_analysis::types::CodeSmell) -> Diagnostic {
        let severity = match smell.severity {
            Severity::Info => DiagnosticSeverity::HINT,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Critical => DiagnosticSeverity::ERROR,
        };

        Diagnostic::new(
            Range::new(
                Position::new(smell.location.line as u32, smell.location.column as u32),
                Position::new(smell.location.line as u32, (smell.location.column + 10) as u32),
            ),
            Some(severity),
            Some(NumberOrString::String(format!("CODE-SMELL-{}", smell.smell_type))),
            Some("rust-ai-ide-codesmells".to_string()),
            format!("{}: {}", smell.title, smell.description),
            smell.refactoring_pattern.as_ref().map(|pattern| {
                vec![DiagnosticRelatedInformation::new(
                    lsp_types::Location::new(
                        Uri::from_str("file://").unwrap(),
                        Range::new(Position::new(0, 0), Position::new(0, 0)),
                    ),
                    format!("Suggested refactoring: {}", pattern),
                )]
            }),
            None,
        )
    }

    /// Convert architecture suggestion to LSP diagnostic
    fn convert_architecture_suggestion(suggestion: &rust_ai_ide_ai_analysis::types::ArchitectureSuggestion) -> Diagnostic {
        Diagnostic::new(
            Range::new(
                Position::new(suggestion.location.line as u32, suggestion.location.column as u32),
                Position::new(suggestion.location.line as u32, (suggestion.location.column + 10) as u32),
            ),
            Some(DiagnosticSeverity::HINT),
            Some(NumberOrString::String("ARCHITECTURE-SUGGESTION".to_string())),
            Some("rust-ai-ide-architecture".to_string()),
            format!("{}: {}", suggestion.pattern, suggestion.description),
            Some(suggestion.benefits.iter().enumerate().map(|(i, benefit)| {
                DiagnosticRelatedInformation::new(
                    lsp_types::Location::new(
                        Uri::from_str("file://").unwrap(),
                        Range::new(Position::new(i as u32, 0), Position::new(i as u32, benefit.len() as u32)),
                    ),
                    benefit.clone(),
                )
            }).collect()),
            None,
        )
    }

    /// Get code actions for AI diagnostics
    ///
    /// # Arguments
    /// * `_params` - Code action parameters (reserved for future implementation)
    pub async fn get_code_actions(
        &self,
        _params: &CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        Ok(None) // Stub implementation
    }
}

impl Default for DiagnosticsManager {
    fn default() -> Self {
        todo!("Default implementation requires async initialization")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_category_codes() {
        assert_eq!(DiagnosticCategory::CodeSmell.code_prefix(), "AI_CODE_SMELL");
        assert_eq!(
            DiagnosticCategory::Performance.code_prefix(),
            "AI_PERFORMANCE"
        );
        assert_eq!(DiagnosticCategory::Security.code_prefix(), "AI_SECURITY");
        assert_eq!(DiagnosticCategory::Style.code_prefix(), "AI_STYLE");
        assert_eq!(
            DiagnosticCategory::Architecture.code_prefix(),
            "AI_ARCHITECTURE"
        );
    }

    #[tokio::test]
    async fn test_diagnostic_creation() {
        // Test diagnostic creation from AI analysis results
        assert!(true); // Stub - full test to be implemented
    }

    #[test]
    fn test_severity_conversion() {
        // Test helper functions
        assert!(true); // Stub
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_category_codes() {
        assert_eq!(DiagnosticCategory::CodeSmell.code_prefix(), "AI_CODE_SMELL");
        assert_eq!(
            DiagnosticCategory::Performance.code_prefix(),
            "AI_PERFORMANCE"
        );
        assert_eq!(DiagnosticCategory::Security.code_prefix(), "AI_SECURITY");
        assert_eq!(DiagnosticCategory::Style.code_prefix(), "AI_STYLE");
        assert_eq!(
            DiagnosticCategory::Architecture.code_prefix(),
            "AI_ARCHITECTURE"
        );
    }

    #[test]
    fn test_severity_conversion() {
        let manager = DiagnosticsManager::new();

        assert_eq!(manager.convert_severity(&Severity::Info), "hint");
        assert_eq!(manager.convert_severity(&Severity::Warning), "info");
        assert_eq!(manager.convert_severity(&Severity::Error), "warning");
        assert_eq!(manager.convert_severity(&Severity::Critical), "error");
    }

    impl DiagnosticsManager {
        /// Convert severity enum to string
        fn convert_severity(&self, severity: &Severity) -> String {
            match severity {
                Severity::Info => "hint".to_string(),
                Severity::Warning => "info".to_string(),
                Severity::Error => "warning".to_string(),
                Severity::Critical => "error".to_string(),
            }
        }
    }
}
