use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::IDEError;

/// Unique identifier for AI assistant sessions
pub type AiSessionId = Uuid;

/// Configuration for an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelConfig {
    /// Model identifier
    pub model_name:  String,
    /// Temperature for generation (0.0 to 1.0)
    pub temperature: f64,
    /// Maximum tokens to generate
    pub max_tokens:  Option<u32>,
    /// Additional model parameters
    pub parameters:  HashMap<String, serde_json::Value>,
}

/// Request to an AI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    /// Session identifier
    pub session_id:   AiSessionId,
    /// Input prompt or messages
    pub input:        AiInput,
    /// Model configuration
    pub model_config: AiModelConfig,
    /// Optional conversation context
    pub context:      Option<AiContext>,
}

/// Different types of AI input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AiInput {
    /// Simple text prompt
    Text { prompt: String },
    /// Chat messages
    Messages { messages: Vec<ChatMessage> },
    /// Completion request
    Completion {
        prefix: String,
        suffix: Option<String>,
    },
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role (system, user, assistant)
    pub role:    MessageRole,
    /// Message content
    pub content: String,
}

/// Role of a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
}

/// Context for AI requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContext {
    /// Previous messages in conversation
    pub history:  Vec<ChatMessage>,
    /// Additional context data
    pub metadata: HashMap<String, String>,
}

/// Response from an AI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// Session identifier
    pub session_id: AiSessionId,
    /// Generated content
    pub content:    AiOutput,
    /// Model used
    pub model:      String,
    /// Usage statistics
    pub usage:      Option<AiUsage>,
    /// Response metadata
    pub metadata:   Option<AiMetadata>,
}

/// Different types of AI output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AiOutput {
    /// Simple text response
    Text { text: String },
    /// Chat response with messages
    Messages { messages: Vec<ChatMessage> },
    /// Code completion
    Completion { completed: String },
}

/// Usage statistics for AI request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsage {
    /// Prompt tokens used
    pub prompt_tokens:     Option<u32>,
    /// Completion tokens used
    pub completion_tokens: Option<u32>,
    /// Total tokens used
    pub total_tokens:      Option<u32>,
}

/// Additional response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMetadata {
    /// Confidence score (0.0 to 1.0)
    pub confidence:         Option<f64>,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
    /// Finish reason
    pub finish_reason:      Option<String>,
}

/// AI provider interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvider {
    /// Provider name (e.g., "openai", "anthropic")
    pub name:             String,
    /// Supported models
    pub supported_models: Vec<String>,
    /// Provider configuration
    pub config:           HashMap<String, serde_json::Value>,
}

/// Error returned by AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiServiceError {
    /// Error code
    pub code:    String,
    /// Error message
    pub message: String,
    /// Optional error details
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl From<AiServiceError> for IDEError {
    fn from(err: AiServiceError) -> Self {
        IDEError::Generic(err.message)
    }
}
