//! AI-related types for Rust AI IDE
//!
//! This module contains types specific to AI analysis and code generation.

/// AI model configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiModelConfig {
    pub model_name: String,
    pub provider: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub capabilities: Vec<String>,
}

/// AI context information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiContext {
    pub current_file: Option<String>,
    pub selection: Option<String>,
    pub cursor_position: Option<(u32, u32)>,
    pub project_structure: Vec<String>,
    pub language: String,
}

/// AI suggestion with confidence score
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiSuggestion {
    pub content: String,
    pub confidence_score: f32,
    pub suggestion_type: AiSuggestionType,
    pub explanation: Option<String>,
}

/// Types of AI suggestions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiSuggestionType {
    Completion,
    Fix,
    Refactoring,
    Documentation,
    Test,
}

/// AI conversation message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiMessage {
    pub role: AiMessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// AI message roles
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiMessageRole {
    User,
    Assistant,
    System,
}

/// AI analysis result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiAnalysisResult {
    pub analysis_type: AiAnalysisType,
    pub content: String,
    pub suggestions: Vec<AiSuggestion>,
    pub execution_time_ms: u64,
}

/// Analysis types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiAnalysisType {
    CodeReview,
    ErrorResolution,
    Optimization,
    Security,
    Performance,
}
