//! # Core Types for AI Inference
//!
//! This module defines core types shared across the inference system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Custom result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Security errors for AI inference operations
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum SecurityError {
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },

    #[error("Authorization denied: {permission}")]
    AuthorizationError { permission: String },

    #[error("Security policy violation: {policy}")]
    PolicyViolation { policy: String },

    #[error("Dangerous content detected: {content_type}")]
    DangerousContent { content_type: String },

    #[error("Rate limit exceeded for {operation}")]
    RateLimitExceeded { operation: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Access denied to resource: {resource}")]
    AccessDenied { resource: String },
}

/// Edit operation for text editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    pub range: serde_json::Value, // Using Value for flexibility
    pub new_text: String,
}

/// Basic model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_path: PathBuf,
    pub model_size: ModelSize,
    pub quantization: Option<Quantization>,
    pub lora_adapters: Vec<String>,
    pub memory_usage_mb: u64,
}

/// AI Provider enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AIProvider {
    Mock,
    OpenAI,
    Anthropic,
    CodeLlamaRust { model_size: ModelSize },
    StarCoderRust { model_size: ModelSize },
    Local { model_path: String },
}

/// Model device specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelDevice {
    Cpu,
    Gpu,
    Auto,
}

/// Model load configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadConfig {
    pub quantization: Option<Quantization>,
    pub lora_adapters: Vec<String>,
    pub memory_limit_mb: Option<u64>,
    pub device: ModelDevice,
    pub lazy_loading: bool,
    pub enable_cache: bool,
}

/// Analysis configuration for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisConfig {
    pub provider: AIProvider,
}

/// AI Service placeholder
#[derive(Debug)]
pub struct AIService {
    // Placeholder for AI service implementation
}

/// Model size enumeration for memory estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelSize {
    Small,
    Medium,
    Large,
    XLarge,
    ExtraLarge,
}

/// Quantization options for model optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quantization {
    None,
    FP32,
    FP16,
    INT8,
    INT4,
    GPTQ,
}

/// Model handle types
#[derive(Debug, Clone)]
pub struct ModelHandle {
    // Placeholder for model handle implementation
}

/// Code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    // Placeholder for code analysis result
}

/// Analysis issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisIssue {
    // Placeholder for analysis issue
}

/// Coding style preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodingStyle {
    Functional,
    Imperative,
    ObjectOriented,
    Procedural,
}

impl Default for CodingStyle {
    fn default() -> Self {
        CodingStyle::Functional
    }
}

/// Model types available in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    CodeLlama,
    StarCoder,
}</search>
