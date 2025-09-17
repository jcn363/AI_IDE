//! # AI Services Module
//!
//! Core AI service implementations for the Rust AI IDE. This module provides
//! the foundation for AI command functionality with proper state management
//! and service lifecycle handling.

use std::sync::Arc;

use tokio::sync::RwLock;

/// Configuration for AI services
#[derive(Debug, Clone)]
pub struct AIConfig {
    pub max_models_loaded: usize,
    pub cache_size: usize,
    pub timeout_ms: u64,
}

/// Main AI Service that manages all AI operations
pub struct AIService {
    config: AIConfig,
    state: Arc<RwLock<AIServiceState>>,
}

#[derive(Debug, Default)]
pub struct AIServiceState {
    pub is_initialized: bool,
    pub loaded_models: usize,
    pub active_jobs: usize,
}

impl AIService {
    /// Create a new AI service with default configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = AIConfig {
            max_models_loaded: 3,
            cache_size: 1000,
            timeout_ms: 30000, // 30 seconds
        };

        let mut service = Self {
            config,
            state: Arc::new(RwLock::new(AIServiceState::default())),
        };

        service.initialize().await?;

        Ok(service)
    }

    /// Initialize the AI service
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.is_initialized = true;

        log::info!("AI Service initialized with config: {:?}", self.config);
        Ok(())
    }

    /// Get current service health status
    pub async fn health_status(&self) -> std::collections::HashMap<String, serde_json::Value> {
        let state = self.state.read().await;
        let mut health = std::collections::HashMap::new();

        health.insert(
            "initialized".to_string(),
            serde_json::json!(state.is_initialized),
        );
        health.insert(
            "loaded_models".to_string(),
            serde_json::json!(state.loaded_models),
        );
        health.insert(
            "active_jobs".to_string(),
            serde_json::json!(state.active_jobs),
        );
        health.insert(
            "max_models".to_string(),
            serde_json::json!(self.config.max_models_loaded),
        );
        health.insert(
            "cache_size".to_string(),
            serde_json::json!(self.config.cache_size),
        );

        health
    }

    /// Check if service is healthy and operational
    pub async fn is_healthy(&self) -> bool {
        let state = self.state.read().await;
        state.is_initialized && state.loaded_models <= self.config.max_models_loaded
    }
}

/// Result type for AI operations
pub type AIResult<T> = Result<T, AIError>;

/// Error types for AI operations
#[derive(thiserror::Error, Debug)]
pub enum AIError {
    #[error("AI service not initialized")]
    NotInitialized,

    #[error("Model not found: {model_name}")]
    ModelNotFound { model_name: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Operation timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Other error: {message}")]
    Other { message: String },
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AIError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AIError::Other {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_service_creation() {
        let service = AIService::new().await.unwrap();
        assert!(service.is_healthy().await);
    }

    #[tokio::test]
    async fn test_ai_service_health_status() {
        let service = AIService::new().await.unwrap();
        let health = service.health_status().await;

        assert!(health.contains_key("initialized"));
        assert!(health.contains_key("loaded_models"));
        assert!(health.contains_key("active_jobs"));
    }
}
