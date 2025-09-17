//! AI Model Service for LSP Integration
//!
//! This module provides AI model loading/unloading functionality within the LSP service infrastructure.
//! It implements secure model management with proper async state handling, path validation, and audit logging.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use rust_ai_ide_errors::IDEError;
use rust_ai_ide_common::validation::TauriInputSanitizer;
use tracing::{debug, error, info, warn};
use moka::future::Cache;

/// Model metadata for tracking loaded models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub model_path: PathBuf,
    pub model_type: ModelType,
    pub size_bytes: u64,
    pub loaded_at: std::time::SystemTime,
    pub last_used: std::time::SystemTime,
    pub memory_usage: Option<u64>,
    pub checksum: String,
}

/// Types of AI models supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelType {
    /// Language model for code completion
    CodeCompletion,
    /// Model for semantic analysis
    SemanticAnalysis,
    /// Model for code refactoring
    Refactoring,
    /// Custom model type
    Custom(String),
}

/// Model loading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadConfig {
    pub model_path: PathBuf,
    pub model_type: ModelType,
    pub memory_limit: Option<u64>,
    pub timeout_seconds: u32,
    pub enable_caching: bool,
}

/// Model loading/unloading result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOperationResult {
    pub success: bool,
    pub model_id: String,
    pub message: String,
    pub duration_ms: u64,
    pub memory_usage: Option<u64>,
}

/// AI Model Service trait
#[async_trait::async_trait]
pub trait AIModelService: Send + Sync {
    /// Load a model into memory
    async fn load_model(&self, config: ModelLoadConfig) -> Result<ModelOperationResult, IDEError>;

    /// Unload a model from memory
    async fn unload_model(&self, model_id: &str) -> Result<ModelOperationResult, IDEError>;

    /// Get information about a loaded model
    async fn get_model_info(&self, model_id: &str) -> Result<Option<ModelMetadata>, IDEError>;

    /// List all loaded models
    async fn list_loaded_models(&self) -> Result<Vec<ModelMetadata>, IDEError>;

    /// Check if a model is loaded
    async fn is_model_loaded(&self, model_id: &str) -> Result<bool, IDEError>;

    /// Get model memory usage
    async fn get_memory_usage(&self, model_id: &str) -> Result<Option<u64>, IDEError>;
}

/// Secure AI Model Manager Implementation
#[derive(Debug)]
pub struct SecureAIModelManager {
    /// Thread-safe storage of loaded models
    loaded_models: Arc<RwLock<HashMap<String, ModelMetadata>>>,

    /// Model cache for performance optimization
    model_cache: Cache<String, ModelMetadata>,

    /// Input sanitizer for security validation
    sanitizer: TauriInputSanitizer,

    /// Maximum memory limit for all models combined
    max_memory_limit: u64,

    /// Current memory usage
    current_memory_usage: Arc<RwLock<u64>>,

    /// Audit logger for security events
    audit_logger: Arc<dyn AuditLogger>,
}

/// Audit logging trait for security compliance
#[async_trait::async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log_model_operation(
        &self,
        operation: &str,
        model_id: &str,
        user_id: Option<&str>,
        success: bool,
        details: HashMap<String, String>,
    ) -> Result<(), IDEError>;
}

/// Secure path validation for model files
async fn validate_model_path(path: &Path) -> Result<(), IDEError> {
    // Validate path is within allowed directories
    rust_ai_ide_common::validation::validate_secure_path(path)?;

    // Check file extension for security
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_str().unwrap_or("");
        match ext_str {
            "bin" | "gguf" | "safetensors" | "pt" | "onnx" => {
                // Valid model file extensions
                debug!("Valid model file extension: {}", ext_str);
            }
            _ => {
                error!("Invalid model file extension: {}", ext_str);
                return Err(IDEError::SecurityError(format!(
                    "Invalid model file extension: {}", ext_str
                )));
            }
        }
    } else {
        return Err(IDEError::SecurityError(
            "Model file must have a valid extension".to_string()
        ));
    }

    // Check file size limits
    if let Ok(metadata) = tokio::fs::metadata(path).await {
        let size = metadata.len();
        const MAX_MODEL_SIZE: u64 = 10 * 1024 * 1024 * 1024; // 10GB limit

        if size > MAX_MODEL_SIZE {
            error!("Model file too large: {} bytes", size);
            return Err(IDEError::SecurityError(format!(
                "Model file too large: {} bytes (max: {} bytes)",
                size, MAX_MODEL_SIZE
            )));
        }

        // Check available system memory
        if let Ok(mem_info) = sys_info::mem_info() {
            let available_mb = mem_info.free as u64 / 1024 / 1024;
            let model_size_mb = size / 1024 / 1024;

            if available_mb < model_size_mb * 2 {
                warn!("Low memory warning: {} MB available, model needs ~{} MB",
                      available_mb, model_size_mb);
                return Err(IDEError::SecurityError(
                    "Insufficient system memory for model loading".to_string()
                ));
            }
        }
    }

    Ok(())
}

/// Calculate checksum for model file integrity
async fn calculate_model_checksum(path: &Path) -> Result<String, IDEError> {
    use sha3::{Digest, Sha3_256};
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha3_256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

impl SecureAIModelManager {
    pub fn new(max_memory_limit: u64, audit_logger: Arc<dyn AuditLogger>) -> Self {
        Self {
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
            model_cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(std::time::Duration::from_secs(3600)) // 1 hour TTL
                .build(),
            sanitizer: TauriInputSanitizer::new(),
            max_memory_limit,
            current_memory_usage: Arc::new(RwLock::new(0)),
            audit_logger,
        }
    }

    /// Initialize the model service with basic setup
    pub async fn initialize(&self) -> Result<(), IDEError> {
        info!("Initializing AI Model Service");

        // Log initialization
        self.audit_logger.log_model_operation(
            "service_init",
            "system",
            None,
            true,
            HashMap::from([
                ("service".to_string(), "ai_model_manager".to_string()),
                ("max_memory_limit".to_string(), self.max_memory_limit.to_string()),
            ]),
        ).await?;

        Ok(())
    }

    /// Shutdown the model service and unload all models
    pub async fn shutdown(&self) -> Result<(), IDEError> {
        info!("Shutting down AI Model Service");

        let mut models_to_unload = Vec::new();

        // Get list of loaded models
        {
            let loaded_models = self.loaded_models.read().await;
            models_to_unload.extend(loaded_models.keys().cloned());
        }

        // Unload all models
        for model_id in models_to_unload {
            if let Err(e) = self.unload_model(&model_id).await {
                error!("Error unloading model {} during shutdown: {}", model_id, e);
            }
        }

        // Log shutdown
        self.audit_logger.log_model_operation(
            "service_shutdown",
            "system",
            None,
            true,
            HashMap::new(),
        ).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl AIModelService for SecureAIModelManager {
    async fn load_model(&self, config: ModelLoadConfig) -> Result<ModelOperationResult, IDEError> {
        let start_time = std::time::Instant::now();
        let model_id = config.model_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Validate input parameters
        self.sanitizer.validate_path(&config.model_path.to_string_lossy())?;

        // Validate model path for security
        validate_model_path(&config.model_path).await?;

        // Check if model is already loaded
        if self.is_model_loaded(&model_id).await? {
            return Ok(ModelOperationResult {
                success: true,
                model_id: model_id.clone(),
                message: "Model already loaded".to_string(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                memory_usage: self.get_memory_usage(&model_id).await?,
            });
        }

        // Check memory limits
        let model_size = if let Ok(metadata) = tokio::fs::metadata(&config.model_path).await {
            metadata.len()
        } else {
            return Err(IDEError::FileSystemError(format!(
                "Cannot access model file: {}",
                config.model_path.display()
            )));
        };

        {
            let current_usage = *self.current_memory_usage.read().await;
            if current_usage + model_size > self.max_memory_limit {
                return Err(IDEError::ResourceError(format!(
                    "Loading model would exceed memory limit: {} + {} > {}",
                    current_usage, model_size, self.max_memory_limit
                )));
            }
        }

        // Calculate checksum for integrity
        let checksum = calculate_model_checksum(&config.model_path).await?;

        info!("Loading AI model: {} from {}", model_id, config.model_path.display());

        // Simulate model loading (replace with actual model loading logic)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Create model metadata
        let metadata = ModelMetadata {
            model_id: model_id.clone(),
            model_path: config.model_path.clone(),
            model_type: config.model_type,
            size_bytes: model_size,
            loaded_at: std::time::SystemTime::now(),
            last_used: std::time::SystemTime::now(),
            memory_usage: Some(model_size), // Estimate memory usage
            checksum,
        };

        // Store model metadata
        {
            let mut loaded_models = self.loaded_models.write().await;
            loaded_models.insert(model_id.clone(), metadata.clone());

            // Update cache
            self.model_cache.insert(model_id.clone(), metadata.clone()).await;
        }

        // Update memory usage
        {
            let mut current_usage = self.current_memory_usage.write().await;
            *current_usage += model_size;
        }

        let duration = start_time.elapsed().as_millis() as u64;

        // Audit log the operation
        self.audit_logger.log_model_operation(
            "load_model",
            &model_id,
            None, // user_id would come from authentication context
            true,
            HashMap::from([
                ("model_path".to_string(), config.model_path.to_string_lossy().to_string()),
                ("model_type".to_string(), format!("{:?}", config.model_type)),
                ("model_size".to_string(), model_size.to_string()),
                ("duration_ms".to_string(), duration.to_string()),
            ]),
        ).await?;

        Ok(ModelOperationResult {
            success: true,
            model_id,
            message: "Model loaded successfully".to_string(),
            duration_ms: duration,
            memory_usage: Some(model_size),
        })
    }

    async fn unload_model(&self, model_id: &str) -> Result<ModelOperationResult, IDEError> {
        let start_time = std::time::Instant::now();

        // Validate input
        self.sanitizer.validate_string(model_id)?;

        info!("Unloading AI model: {}", model_id);

        let metadata = {
            let mut loaded_models = self.loaded_models.write().await;
            loaded_models.remove(model_id)
        };

        match metadata {
            Some(metadata) => {
                // Update memory usage
                if let Some(memory_usage) = metadata.memory_usage {
                    let mut current_usage = self.current_memory_usage.write().await;
                    *current_usage = current_usage.saturating_sub(memory_usage);
                }

                // Remove from cache
                self.model_cache.invalidate(model_id).await;

                let duration = start_time.elapsed().as_millis() as u64;

                // Audit log the operation
                self.audit_logger.log_model_operation(
                    "unload_model",
                    model_id,
                    None,
                    true,
                    HashMap::from([
                        ("model_path".to_string(), metadata.model_path.to_string_lossy().to_string()),
                        ("model_type".to_string(), format!("{:?}", metadata.model_type)),
                        ("duration_ms".to_string(), duration.to_string()),
                    ]),
                ).await?;

                Ok(ModelOperationResult {
                    success: true,
                    model_id: model_id.to_string(),
                    message: "Model unloaded successfully".to_string(),
                    duration_ms: duration,
                    memory_usage: metadata.memory_usage,
                })
            }
            None => {
                // Model not loaded, log as informational
                self.audit_logger.log_model_operation(
                    "unload_model",
                    model_id,
                    None,
                    true, // Still successful, just no-op
                    HashMap::from([
                        ("status".to_string(), "not_loaded".to_string()),
                    ]),
                ).await?;

                Ok(ModelOperationResult {
                    success: true,
                    model_id: model_id.to_string(),
                    message: "Model not loaded".to_string(),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    memory_usage: None,
                })
            }
        }
    }

    async fn get_model_info(&self, model_id: &str) -> Result<Option<ModelMetadata>, IDEError> {
        // Validate input
        self.sanitizer.validate_string(model_id)?;

        // Try cache first
        if let Some(metadata) = self.model_cache.get(model_id).await {
            return Ok(Some(metadata));
        }

        // Check loaded models
        let loaded_models = self.loaded_models.read().await;
        Ok(loaded_models.get(model_id).cloned())
    }

    async fn list_loaded_models(&self) -> Result<Vec<ModelMetadata>, IDEError> {
        let loaded_models = self.loaded_models.read().await;
        Ok(loaded_models.values().cloned().collect())
    }

    async fn is_model_loaded(&self, model_id: &str) -> Result<bool, IDEError> {
        // Validate input
        self.sanitizer.validate_string(model_id)?;

        let loaded_models = self.loaded_models.read().await;
        Ok(loaded_models.contains_key(model_id))
    }

    async fn get_memory_usage(&self, model_id: &str) -> Result<Option<u64>, IDEError> {
        // Validate input
        self.sanitizer.validate_string(model_id)?;

        let loaded_models = self.loaded_models.read().await;
        Ok(loaded_models.get(model_id).and_then(|m| m.memory_usage))
    }
}

/// Default audit logger implementation
pub struct DefaultAuditLogger;

#[async_trait::async_trait]
impl AuditLogger for DefaultAuditLogger {
    async fn log_model_operation(
        &self,
        operation: &str,
        model_id: &str,
        user_id: Option<&str>,
        success: bool,
        details: HashMap<String, String>,
    ) -> Result<(), IDEError> {
        let level = if success { "INFO" } else { "ERROR" };
        let user = user_id.unwrap_or("system");

        info!(
            "[{}] Model operation: {} by user: {}, model: {}, details: {:?}",
            level, operation, user, model_id, details
        );

        Ok(())
    }
}