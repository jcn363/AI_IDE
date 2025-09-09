use crate::{AiRequest, AiResponse, IDEResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of analysis that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    /// Code review and feedback
    CodeReview,
    /// Bug detection and suggestions
    BugDetection,
    /// Performance analysis
    PerformanceAnalysis,
    /// Security analysis
    SecurityAnalysis,
    /// Code quality analysis
    QualityAnalysis,
    /// Refactoring suggestions
    RefactoringSuggestions,
}

impl AnalysisType {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::CodeReview => "Code review and feedback",
            Self::BugDetection => "Bug detection and suggestions",
            Self::PerformanceAnalysis => "Performance analysis",
            Self::SecurityAnalysis => "Security analysis",
            Self::QualityAnalysis => "Code quality analysis",
            Self::RefactoringSuggestions => "Refactoring suggestions",
        }
    }
}

/// Configuration for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Type of analysis
    pub analysis_type: AnalysisType,
    /// Analysis parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Maximum time to spend on analysis
    pub timeout_seconds: Option<u64>,
    /// Importance level of issues to report
    pub min_severity: Severity,
}

/// Severity levels for analysis results
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Low impact issues
    Low,
    /// Medium impact issues
    Medium,
    /// High impact issues
    High,
    /// Critical issues that should be fixed immediately
    Critical,
}

impl Severity {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// An issue found during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisIssue {
    /// Issue identifier
    pub id: String,
    /// Issue title
    pub title: String,
    /// Issue description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Location in code
    pub location: Option<CodeLocation>,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Location in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    /// File path
    pub file: String,
    /// Line number (1-based)
    pub line: Option<u32>,
    /// Column number (1-based)
    pub column: Option<u32>,
    /// Code snippet
    pub snippet: Option<String>,
}

/// Result of an analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Analysis configuration
    pub config: AnalysisConfig,
    /// List of issues found
    pub issues: Vec<AnalysisIssue>,
    /// Summary of analysis
    pub summary: Option<String>,
    /// Analysis duration in seconds
    pub duration_seconds: f64,
    /// Metadata about the analysis process
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Analysis service for performing AI-powered code analysis
pub struct AnalysisService {
    /// AI service for processing
    ai_service: Box<dyn AiServiceTrait>,
}

impl AnalysisService {
    /// Create a new analysis service
    pub fn new(ai_service: Box<dyn AiServiceTrait>) -> Self {
        Self { ai_service }
    }

    /// Perform analysis on code
    pub async fn analyze_code(
        &self,
        code: &str,
        config: AnalysisConfig,
    ) -> IDEResult<AnalysisResult> {
        let mut issues = Vec::new();
        let start_time = std::time::Instant::now();

        // Build AI prompt for analysis
        let prompt = self.build_analysis_prompt(code, &config);
        let ai_config = config_to_ai_config(&config);

        let request = AiRequest {
            session_id: uuid::Uuid::new_v4(),
            input: crate::AiInput::Text { prompt },
            model_config: ai_config,
            context: None,
        };

        let response = self.ai_service.process_request(request).await?;
        let duration = start_time.elapsed().as_secs_f64();

        // Parse AI response into analysis result
        let (response_issues, summary) = self.parse_analysis_response(&response)?;

        issues.extend(response_issues);

        let result = AnalysisResult {
            config,
            issues,
            summary: Some(summary),
            duration_seconds: duration,
            metadata: HashMap::new(),
        };

        Ok(result)
    }

    /// Build prompt for AI analysis
    fn build_analysis_prompt(&self, code: &str, config: &AnalysisConfig) -> String {
        format!(
            "Please perform a {} analysis on the following code. Look for issues with severity {} or higher.\n\n{}",
            config.analysis_type.description(),
            config.min_severity.as_str(),
            code
        )
    }

    /// Parse AI response into analysis result
    fn parse_analysis_response(
        &self,
        _response: &AiResponse,
    ) -> IDEResult<(Vec<AnalysisIssue>, String)> {
        // For now, return empty result. In practice, this would parse the AI response.
        Ok((Vec::new(), "Analysis completed successfully".to_string()))
    }
}

/// Trait for AI services
#[async_trait::async_trait]
pub trait AiServiceTrait: Send + Sync {
    /// Process an AI request
    async fn process_request(&self, request: AiRequest) -> IDEResult<AiResponse>;
}

/// Convert analysis config to AI model config
fn config_to_ai_config(_config: &AnalysisConfig) -> crate::AiModelConfig {
    crate::AiModelConfig {
        model_name: "gpt-4".to_string(), // Default model
        temperature: 0.7,
        max_tokens: Some(1000),
        parameters: HashMap::new(),
    }
}
