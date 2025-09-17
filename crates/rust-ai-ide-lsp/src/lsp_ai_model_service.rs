//! LSP AI Model Service Integration
//!
//! This module provides LSP service initialization and integration for AI model management.
//! It combines the AI model service with LSP infrastructure to provide secure,
//! async model loading/unloading capabilities within the LSP service context.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use rust_ai_ide_errors::IDEError;
use rust_ai_ide_common::validation::TauriInputSanitizer;

use crate::ai_model_service::{
    SecureAIModelManager, AIModelService, ModelLoadConfig, ModelOperationResult,
    ModelType, DefaultAuditLogger,
};

/// LSP AI Model Service Configuration
#[derive(Debug, Clone)]
pub struct LSPAIModelServiceConfig {
    /// Maximum memory limit for all models (in bytes)
    pub max_memory_limit: u64,
    /// Default timeout for model operations (in seconds)
    pub default_timeout_seconds: u32,
    /// Whether to enable caching
    pub enable_caching: bool,
    /// Maximum number of concurrent model operations
    pub max_concurrent_operations: usize,
    /// Audit logging level (0 = disabled, 1 = basic, 2 = detailed)
    pub audit_log_level: u8,
}

impl Default for LSPAIModelServiceConfig {
    fn default() -> Self {
        Self {
            max_memory_limit: 2 * 1024 * 1024 * 1024, // 2GB default
            default_timeout_seconds: 300, // 5 minutes
            enable_caching: true,
            max_concurrent_operations: 3,
            audit_log_level: 1,
        }
    }
}

/// LSP AI Model Service State
#[derive(Debug)]
pub struct LSPAIModelServiceState {
    /// Configuration
    config: LSPAIModelServiceConfig,
    /// AI model manager
    model_manager: Arc<dyn AIModelService>,
    /// Service initialization status
    initialized: bool,
    /// Performance metrics
    metrics: LSPAIModelServiceMetrics,
}

/// Performance metrics for LSP AI model service
#[derive(Debug, Clone)]
pub struct LSPAIModelServiceMetrics {
    pub total_models_loaded: usize,
    pub total_memory_used: u64,
    pub average_load_time_ms: f64,
    pub total_operations: usize,
    pub failed_operations: usize,
}

impl Default for LSPAIModelServiceMetrics {
    fn default() -> Self {
        Self {
            total_models_loaded: 0,
            total_memory_used: 0,
            average_load_time_ms: 0.0,
            total_operations: 0,
            failed_operations: 0,
        }
    }
}

/// Main LSP AI Model Service
#[derive(Debug)]
pub struct LSPAIModelService {
    /// Thread-safe service state
    state: Arc<RwLock<LSPAIModelServiceState>>,
    /// Input sanitizer for security
    sanitizer: TauriInputSanitizer,
}

impl LSPAIModelService {
    /// Create a new LSP AI Model Service
    pub fn new(config: LSPAIModelServiceConfig) -> Result<Self, IDEError> {
        // Create audit logger
        let audit_logger = Arc::new(DefaultAuditLogger);

        // Create model manager
        let model_manager = Arc::new(SecureAIModelManager::new(
            config.max_memory_limit,
            audit_logger,
        ));

        let state = LSPAIModelServiceState {
            config: config.clone(),
            model_manager,
            initialized: false,
            metrics: LSPAIModelServiceMetrics::default(),
        };

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            sanitizer: TauriInputSanitizer::new(),
        })
    }

    /// Initialize the LSP AI model service
    pub async fn initialize(&self) -> Result<(), IDEError> {
        let mut state = self.state.write().await;

        if state.initialized {
            debug!("LSP AI Model Service already initialized");
            return Ok(());
        }

        info!("Initializing LSP AI Model Service");

        // Initialize the model manager
        if let Ok(manager) = state.model_manager.as_any().downcast_ref::<SecureAIModelManager>() {
            manager.initialize().await?;
        }

        state.initialized = true;

        info!("LSP AI Model Service initialized successfully");
        Ok(())
    }

    /// Shutdown the LSP AI model service
    pub async fn shutdown(&self) -> Result<(), IDEError> {
        let mut state = self.state.write().await;

        if !state.initialized {
            debug!("LSP AI Model Service not initialized");
            return Ok(());
        }

        info!("Shutting down LSP AI Model Service");

        // Shutdown the model manager
        if let Ok(manager) = state.model_manager.as_any().downcast_ref::<SecureAIModelManager>() {
            manager.shutdown().await?;
        }

        state.initialized = false;

        info!("LSP AI Model Service shut down successfully");
        Ok(())
    }

    /// Load a model through the LSP service
    pub async fn load_model(
        &self,
        model_path: &str,
        model_type: ModelType,
    ) -> Result<ModelOperationResult, IDEError> {
        // Validate input
        self.sanitizer.validate_path(model_path)?;
        self.sanitizer.validate_string(&format!("{:?}", model_type))?;

        let mut state = self.state.write().await;

        if !state.initialized {
            return Err(IDEError::ServiceError(
                "LSP AI Model Service not initialized".to_string()
            ));
        }

        // Create model load configuration
        let config = ModelLoadConfig {
            model_path: model_path.into(),
            model_type,
            memory_limit: Some(state.config.max_memory_limit / 4), // Use 1/4 of total limit per model
            timeout_seconds: state.config.default_timeout_seconds,
            enable_caching: state.config.enable_caching,
        };

        // Load the model
        let result = state.model_manager.load_model(config).await?;

        // Update metrics
        if result.success {
            state.metrics.total_models_loaded += 1;
            if let Some(memory) = result.memory_usage {
                state.metrics.total_memory_used += memory;
            }
            state.metrics.total_operations += 1;

            // Update average load time
            let current_avg = state.metrics.average_load_time_ms;
            let total_ops = state.metrics.total_operations as f64;
            state.metrics.average_load_time_ms =
                (current_avg * (total_ops - 1.0) + result.duration_ms as f64) / total_ops;
        } else {
            state.metrics.failed_operations += 1;
        }

        Ok(result)
    }

    /// Unload a model through the LSP service
    pub async fn unload_model(&self, model_id: &str) -> Result<ModelOperationResult, IDEError> {
        // Validate input
        self.sanitizer.validate_string(model_id)?;

        let mut state = self.state.write().await;

        if !state.initialized {
            return Err(IDEError::ServiceError(
                "LSP AI Model Service not initialized".to_string()
            ));
        }

        // Unload the model
        let result = state.model_manager.unload_model(model_id).await?;

        // Update metrics
        if result.success {
            state.metrics.total_models_loaded = state.metrics.total_models_loaded.saturating_sub(1);
            if let Some(memory) = result.memory_usage {
                state.metrics.total_memory_used = state.metrics.total_memory_used.saturating_sub(memory);
            }
            state.metrics.total_operations += 1;
        } else {
            state.metrics.failed_operations += 1;
        }

        Ok(result)
    }

    /// Get model information through the LSP service
    pub async fn get_model_info(&self, model_id: &str) -> Result<Option<crate::ai_model_service::ModelMetadata>, IDEError> {
        // Validate input
        self.sanitizer.validate_string(model_id)?;

        let state = self.state.read().await;

        if !state.initialized {
            return Err(IDEError::ServiceError(
                "LSP AI Model Service not initialized".to_string()
            ));
        }

        state.model_manager.get_model_info(model_id).await
    }

    /// List all loaded models through the LSP service
    pub async fn list_loaded_models(&self) -> Result<Vec<crate::ai_model_service::ModelMetadata>, IDEError> {
        let state = self.state.read().await;

        if !state.initialized {
            return Err(IDEError::ServiceError(
                "LSP AI Model Service not initialized".to_string()
            ));
        }

        state.model_manager.list_loaded_models().await
    }

    /// Check if a model is loaded through the LSP service
    pub async fn is_model_loaded(&self, model_id: &str) -> Result<bool, IDEError> {
        // Validate input
        self.sanitizer.validate_string(model_id)?;

        let state = self.state.read().await;

        if !state.initialized {
            return Err(IDEError::ServiceError(
                "LSP AI Model Service not initialized".to_string()
            ));
        }

        state.model_manager.is_model_loaded(model_id).await
    }

    /// Get current service metrics
    pub async fn get_metrics(&self) -> LSPAIModelServiceMetrics {
        let state = self.state.read().await;
        state.metrics.clone()
    }

    /// Get service health status
    pub async fn health_check(&self) -> LSPServiceHealth {
        let state = self.state.read().await;

        if !state.initialized {
            return LSPServiceHealth::NotInitialized;
        }

        // Check if memory usage is within limits
        if state.metrics.total_memory_used > state.config.max_memory_limit {
            return LSPServiceHealth::MemoryLimitExceeded;
        }

        // Check if there are too many failed operations
        let failure_rate = if state.metrics.total_operations > 0 {
            state.metrics.failed_operations as f64 / state.metrics.total_operations as f64
        } else {
            0.0
        };

        if failure_rate > 0.5 {
            return LSPServiceHealth::HighFailureRate;
        }

        LSPServiceHealth::Healthy
    }

    /// Get service configuration
    pub async fn get_config(&self) -> LSPAIModelServiceConfig {
        let state = self.state.read().await;
        state.config.clone()
    }
}

/// Service health status
#[derive(Debug, Clone, PartialEq)]
pub enum LSPServiceHealth {
    Healthy,
    NotInitialized,
    MemoryLimitExceeded,
    HighFailureRate,
    ServiceUnavailable,
}

/// Extension trait for AIModelService to support downcasting
pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AsAny for SecureAIModelManager {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_lsp_ai_model_service_initialization() {
        let config = LSPAIModelServiceConfig::default();
        let service = LSPAIModelService::new(config).unwrap();

        // Test initialization
        assert!(service.initialize().await.is_ok());

        // Test double initialization (should be ok)
        assert!(service.initialize().await.is_ok());

        // Test shutdown
        assert!(service.shutdown().await.is_ok());

        // Test shutdown when not initialized (should be ok)
        assert!(service.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_model_operations_without_initialization() {
        let config = LSPAIModelServiceConfig::default();
        let service = LSPAIModelService::new(config).unwrap();

        // Test operations without initialization
        assert!(service.load_model("/fake/path", ModelType::CodeCompletion).await.is_err());
        assert!(service.list_loaded_models().await.is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = LSPAIModelServiceConfig::default();
        let service = LSPAIModelService::new(config).unwrap();

        // Service should not be healthy when not initialized
        assert_eq!(service.health_check().await, LSPServiceHealth::NotInitialized);

        // Initialize service
        service.initialize().await.unwrap();

        // Service should be healthy after initialization
        assert_eq!(service.health_check().await, LSPServiceHealth::Healthy);
    }

    #[tokio::test]
    async fn test_input_validation() {
        let config = LSPAIModelServiceConfig::default();
        let service = LSPAIModelService::new(config).unwrap();
        service.initialize().await.unwrap();

        // Test invalid path
        assert!(service.load_model("", ModelType::CodeCompletion).await.is_err());

        // Test invalid model ID
        assert!(service.get_model_info("").await.is_err());
        assert!(service.is_model_loaded("").await.is_err());
    }
}