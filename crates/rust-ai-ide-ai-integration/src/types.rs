//! Common types and interfaces for AI integration layer
//!
//! This module defines the shared data structures and interfaces used throughout
//! the AI service integration system.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Unique identifier for AI requests and responses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(uuid::Uuid);

impl RequestId {
    /// Generate a new random request ID
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

/// AI request context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequestContext {
    /// Unique request identifier
    pub request_id: RequestId,
    /// User ID making the request
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// AI model type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiModel {
    /// Text generation model
    TextGeneration,
    /// Code completion model
    CodeCompletion,
    /// Code analysis model
    CodeAnalysis,
    /// Refactoring model
    Refactoring,
    /// Testing model
    Testing,
    /// Documentation model
    Documentation,
    /// Custom model with identifier
    Custom(String),
}

/// AI response priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsePriority {
    /// Low priority - can be delayed
    Low = 1,
    /// Normal priority - standard processing
    Normal = 2,
    /// High priority - expedited processing
    High = 3,
    /// Critical priority - immediate processing
    Critical = 4,
}

/// AI capability descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapability {
    /// Model type this capability applies to
    pub model_type: AiModel,
    /// Capability name (e.g., "code_completion", "refactoring")
    pub name: String,
    /// Maximum input tokens supported
    pub max_input_tokens: Option<usize>,
    /// Maximum output tokens supported
    pub max_output_tokens: Option<usize>,
    /// Supported output formats
    pub supported_formats: Vec<String>,
    /// Performance characteristics
    pub performance_profile: PerformanceProfile,
}

/// Performance profile for an AI capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
    /// Success rate as percentage (0-100)
    pub success_rate_percent: u8,
    /// Throughput in tokens per second
    pub throughput_tokens_per_sec: f64,
    /// Cost per token
    pub cost_per_token: Option<f64>,
}

/// LSP AI completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspAiCompletion {
    /// Original LSP completion request
    pub original_request: serde_json::Value,
    /// AI-enhanced completion
    pub ai_completion: Option<serde_json::Value>,
    /// Enhancement metadata
    pub enhancement_metadata: HashMap<String, serde_json::Value>,
    /// Completion confidence score (0.0-1.0)
    pub confidence_score: Option<f64>,
}

/// Frontend AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendAiResponse {
    /// Request ID that generated this response
    pub request_id: RequestId,
    /// Response content
    pub content: AiResponseContent,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Response status
    pub status: ResponseStatus,
}

/// Response content variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AiResponseContent {
    /// Text completion response
    TextCompletion {
        /// Generated text
        text: String,
        /// Completion scoring
        score: Option<f64>,
    },
    /// Code suggestion response
    CodeSuggestion {
        /// Suggested code
        code: String,
        /// Language identifier
        language: Option<String>,
        /// Explanation of the suggestion
        explanation: Option<String>,
    },
    /// Error response
    Error {
        /// Error message
        message: String,
        /// Error code
        code: Option<String>,
    },
    /// Status update
    Status {
        /// Status message
        message: String,
        /// Progress percentage (0-100)
        progress: Option<u8>,
    },
}

/// Response status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Request processed successfully
    Success,
    /// Request processing failed
    Failed,
    /// Request is still being processed
    InProgress,
    /// Request was cancelled
    Cancelled,
}

/// User behavior pattern data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehaviorPattern {
    /// User identifier
    pub user_id: String,
    /// Pattern type
    pub pattern_type: String,
    /// Pattern data (flexible JSON structure)
    pub data: serde_json::Value,
    /// Pattern confidence score
    pub confidence: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// TypeScript interface descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptInterface {
    /// Interface name
    pub name: String,
    /// Interface definition
    pub definition: String,
    /// Validation rules
    pub validation_rules: Vec<String>,
    /// Documentation
    pub documentation: Option<String>,
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Response times in milliseconds
    pub response_times_ms: Vec<u64>,
    /// Success rates
    pub success_rates: Vec<f64>,
    /// Throughput measurements
    pub throughput_measurements: Vec<f64>,
    /// Timestamp of this snapshot
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Configuration structure for AI integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIntegrationConfig {
    /// Enable LSP AI bridge
    pub enable_lsp_bridge: bool,
    /// Enable frontend interface
    pub enable_frontend_interface: bool,
    /// Enable performance router
    pub enable_performance_router: bool,
    /// Enable UX optimizer
    pub enable_ux_optimizer: bool,
    /// Maximum concurrent AI requests
    pub max_concurrent_requests: usize,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}
