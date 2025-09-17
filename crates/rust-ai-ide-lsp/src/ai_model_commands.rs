//! Tauri Commands for AI Model Operations
//!
//! This module provides Tauri command handlers for AI model loading/unloading operations
//! within the LSP service infrastructure. Commands follow the project's standardized
//! command template patterns with proper error handling, validation, and async execution.

use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::lsp_ai_model_service::{
    LSPAIModelService, LSPAIModelServiceConfig, ModelType, LSPServiceHealth,
};
use crate::ai_model_service::ModelMetadata;
use rust_ai_ide_errors::IDEError;

/// Command configuration for AI model operations
const AI_MODEL_COMMAND_CONFIG: crate::command_templates::CommandConfig = crate::command_templates::CommandConfig {
    enable_logging: true,
    log_level: log::Level::Info,
    enable_validation: true,
    async_timeout_secs: Some(300), // 5 minutes for model operations
};

/// Input parameters for loading a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelParams {
    pub model_path: String,
    pub model_type: String,
}

/// Input parameters for unloading a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadModelParams {
    pub model_id: String,
}

/// Input parameters for getting model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetModelInfoParams {
    pub model_id: String,
}

/// Response for model operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOperationResponse {
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
    pub memory_usage: Option<u64>,
    pub model_id: Option<String>,
}

/// Response for model list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListResponse {
    pub models: Vec<ModelMetadata>,
    pub total_count: usize,
}

/// Response for service health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub message: String,
}

/// Type alias for thread-safe LSP AI model service
pub type LSPAIModelServiceState = Arc<RwLock<LSPAIModelService>>;

/// Initialize LSP AI Model Service command
#[tauri::command]
pub async fn initialize_ai_model_service(
    state: State<'_, LSPAIModelServiceState>,
    config: Option<LSPAIModelServiceConfig>,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "initialize_ai_model_service",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Use provided config or default
            let init_config = config.unwrap_or_default();

            // Initialize the service
            service_guard.initialize().await
                .map_err(|e| format!("Failed to initialize AI model service: {}", e))?;

            Ok(serde_json::json!({
                "status": "initialized",
                "message": "AI model service initialized successfully",
                "config": init_config
            }).to_string())
        }
    )
}

/// Shutdown LSP AI Model Service command
#[tauri::command]
pub async fn shutdown_ai_model_service(
    state: State<'_, LSPAIModelServiceState>,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "shutdown_ai_model_service",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Shutdown the service
            service_guard.shutdown().await
                .map_err(|e| format!("Failed to shutdown AI model service: {}", e))?;

            Ok(serde_json::json!({
                "status": "shutdown",
                "message": "AI model service shut down successfully"
            }).to_string())
        }
    )
}

/// Load AI model command
#[tauri::command]
pub async fn load_ai_model(
    state: State<'_, LSPAIModelServiceState>,
    params: LoadModelParams,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "load_ai_model",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Validate input parameters
            if params.model_path.trim().is_empty() {
                return Err("Model path cannot be empty".to_string());
            }

            // Parse model type
            let model_type = match params.model_type.as_str() {
                "CodeCompletion" => ModelType::CodeCompletion,
                "SemanticAnalysis" => ModelType::SemanticAnalysis,
                "Refactoring" => ModelType::Refactoring,
                "Custom" => ModelType::Custom("custom".to_string()),
                _ => return Err(format!("Invalid model type: {}", params.model_type)),
            };

            // Load the model
            let result = service_guard.load_model(&params.model_path, model_type).await
                .map_err(|e| format!("Failed to load model: {}", e))?;

            // Convert to response format
            let response = ModelOperationResponse {
                success: result.success,
                message: result.message,
                duration_ms: result.duration_ms,
                memory_usage: result.memory_usage,
                model_id: Some(result.model_id),
            };

            Ok(serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))?)
        }
    )
}

/// Unload AI model command
#[tauri::command]
pub async fn unload_ai_model(
    state: State<'_, LSPAIModelServiceState>,
    params: UnloadModelParams,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "unload_ai_model",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Validate input parameters
            if params.model_id.trim().is_empty() {
                return Err("Model ID cannot be empty".to_string());
            }

            // Unload the model
            let result = service_guard.unload_model(&params.model_id).await
                .map_err(|e| format!("Failed to unload model: {}", e))?;

            // Convert to response format
            let response = ModelOperationResponse {
                success: result.success,
                message: result.message,
                duration_ms: result.duration_ms,
                memory_usage: result.memory_usage,
                model_id: Some(result.model_id),
            };

            Ok(serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))?)
        }
    )
}

/// Get model information command
#[tauri::command]
pub async fn get_ai_model_info(
    state: State<'_, LSPAIModelServiceState>,
    params: GetModelInfoParams,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "get_ai_model_info",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Validate input parameters
            if params.model_id.trim().is_empty() {
                return Err("Model ID cannot be empty".to_string());
            }

            // Get model information
            let model_info = service_guard.get_model_info(&params.model_id).await
                .map_err(|e| format!("Failed to get model info: {}", e))?;

            match model_info {
                Some(metadata) => {
                    Ok(serde_json::to_string(&metadata)
                        .map_err(|e| format!("Failed to serialize model info: {}", e))?)
                }
                None => {
                    Ok(serde_json::json!({
                        "error": "Model not found",
                        "model_id": params.model_id
                    }).to_string())
                }
            }
        }
    )
}

/// List loaded models command
#[tauri::command]
pub async fn list_loaded_ai_models(
    state: State<'_, LSPAIModelServiceState>,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "list_loaded_ai_models",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Get list of loaded models
            let models = service_guard.list_loaded_models().await
                .map_err(|e| format!("Failed to list loaded models: {}", e))?;

            // Create response
            let response = ModelListResponse {
                models,
                total_count: models.len(),
            };

            Ok(serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize model list: {}", e))?)
        }
    )
}

/// Check if model is loaded command
#[tauri::command]
pub async fn is_ai_model_loaded(
    state: State<'_, LSPAIModelServiceState>,
    params: GetModelInfoParams,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "is_ai_model_loaded",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Validate input parameters
            if params.model_id.trim().is_empty() {
                return Err("Model ID cannot be empty".to_string());
            }

            // Check if model is loaded
            let is_loaded = service_guard.is_model_loaded(&params.model_id).await
                .map_err(|e| format!("Failed to check model status: {}", e))?;

            Ok(serde_json::json!({
                "model_id": params.model_id,
                "is_loaded": is_loaded
            }).to_string())
        }
    )
}

/// Get service metrics command
#[tauri::command]
pub async fn get_ai_model_service_metrics(
    state: State<'_, LSPAIModelServiceState>,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "get_ai_model_service_metrics",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Get service metrics
            let metrics = service_guard.get_metrics().await;

            Ok(serde_json::to_string(&metrics)
                .map_err(|e| format!("Failed to serialize metrics: {}", e))?)
        }
    )
}

/// Service health check command
#[tauri::command]
pub async fn check_ai_model_service_health(
    state: State<'_, LSPAIModelServiceState>,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "check_ai_model_service_health",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Check service health
            let health = service_guard.health_check().await;

            let (status, message) = match health {
                LSPServiceHealth::Healthy => ("healthy", "Service is operating normally"),
                LSPServiceHealth::NotInitialized => ("not_initialized", "Service has not been initialized"),
                LSPServiceHealth::MemoryLimitExceeded => ("memory_limit_exceeded", "Service has exceeded memory limits"),
                LSPServiceHealth::HighFailureRate => ("high_failure_rate", "Service has a high rate of operation failures"),
                LSPServiceHealth::ServiceUnavailable => ("unavailable", "Service is temporarily unavailable"),
            };

            let response = HealthCheckResponse {
                status: status.to_string(),
                message: message.to_string(),
            };

            Ok(serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize health check response: {}", e))?)
        }
    )
}

/// Get service configuration command
#[tauri::command]
pub async fn get_ai_model_service_config(
    state: State<'_, LSPAIModelServiceState>,
) -> Result<String, String> {
    crate::command_templates::execute_command!(
        "get_ai_model_service_config",
        &AI_MODEL_COMMAND_CONFIG,
        async move || {
            let service_guard = state.lock().await;

            // Get service configuration
            let config = service_guard.get_config().await;

            Ok(serde_json::to_string(&config)
                .map_err(|e| format!("Failed to serialize config: {}", e))?)
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_load_model_validation() {
        // Test empty model path validation
        let params = LoadModelParams {
            model_path: "".to_string(),
            model_type: "CodeCompletion".to_string(),
        };

        let config = LSPAIModelServiceConfig::default();
        let service = LSPAIModelService::new(config).unwrap();
        let state = Arc::new(RwLock::new(service));

        // This should fail due to empty path validation
        let result = load_ai_model(State::from(state), params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unload_model_validation() {
        // Test empty model ID validation
        let params = UnloadModelParams {
            model_id: "".to_string(),
        };

        let config = LSPAIModelServiceConfig::default();
        let service = LSPAIModelService::new(config).unwrap();
        let state = Arc::new(RwLock::new(service));

        // This should fail due to empty model ID validation
        let result = unload_ai_model(State::from(state), params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_model_type() {
        // Test invalid model type
        let params = LoadModelParams {
            model_path: "/valid/path/model.bin".to_string(),
            model_type: "InvalidType".to_string(),
        };

        let config = LSPAIModelServiceConfig::default();
        let service = LSPAIModelService::new(config).unwrap();
        let state = Arc::new(RwLock::new(service));

        // This should fail due to invalid model type
        let result = load_ai_model(State::from(state), params).await;
        assert!(result.is_err());
    }
}