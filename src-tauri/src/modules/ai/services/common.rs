//! Common AI Service interfaces and abstractions
//!
//! This module provides core traits and implementations for AI service management,
//! including service discovery, connection pooling, and unified interfaces.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
/// Add chrono for timestamps
use chrono;
use rust_ai_ide_ai_inference::{
    AIProvider as InferenceAIProvider, AnalysisType, CodeCompletionConfig, GenerationConfig, InferenceEngine,
    LocalInferenceEngine, ModelSize,
};
use rust_ai_ide_common::validation::{validate_secure_path, TauriInputSanitizer};
use rust_ai_ide_errors::{IDEError, IDEResult};
use rust_ai_ide_lsp::{AIContext, Completion, LSPClient, LSPClientConfig};
use rust_ai_ide_security::audit_logger;
use rust_ai_ide_security::encryption::{EncryptionManager, GLOBAL_ENCRYPTION_MANAGER};
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, RwLock as TokioRwLock, Semaphore};

use crate::command_templates::{execute_with_retry, spawn_background_task};

/// Add logging capability
#[macro_use]
extern crate log;

/// Core AI Service with LSP integration
#[derive(Clone)]
pub struct AIService {
    lsp_client:         Arc<RwLock<Option<LSPClient>>>,
    initialized:        Arc<RwLock<bool>>,
    provider_config:    Arc<RwLock<Option<AIProvider>>>,
    encryption_manager: Arc<EncryptionManager>,
}

impl AIService {
    /// Create a new AI service instance
    pub fn new() -> Self {
        Self {
            lsp_client:         Arc::new(RwLock::new(None)),
            initialized:        Arc::new(RwLock::new(false)),
            provider_config:    Arc::new(RwLock::new(None)),
            encryption_manager: GLOBAL_ENCRYPTION_MANAGER.clone(),
        }
    }

    /// Initialize the AI service with LSP integration
    pub async fn initialize(&self, provider: AIProvider) -> IDEResult<()> {
        let mut initialized = self
            .initialized
            .write()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire initialization lock: {}", e)))?;

        if *initialized {
            return Ok(());
        }

        // Validate provider configuration
        self.validate_provider_config(&provider).await?;

        // Initialize LSP client
        let lsp_config = LSPClientConfig {
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            ..Default::default()
        };

        let lsp_client = LSPClient::new(lsp_config)
            .await
            .map_err(|e| IDEError::AIService(format!("Failed to create LSP client: {:?}", e)))?;

        // Store configuration securely
        let mut config = self
            .provider_config
            .write()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire config lock: {}", e)))?;
        *config = Some(provider);

        let mut client_lock = self
            .lsp_client
            .write()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire LSP client lock: {}", e)))?;
        *client_lock = Some(lsp_client);

        *initialized = true;

        // Audit log initialization
        audit_logger::log_event(
            "ai_service_initialized",
            &serde_json::json!({
                "provider": format!("{:?}", provider),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        log::info!(
            "AI service initialized successfully with provider: {:?}",
            provider
        );
        Ok(())
    }

    /// Validate provider configuration
    async fn validate_provider_config(&self, provider: &AIProvider) -> IDEResult<()> {
        match provider {
            AIProvider::Local { model_path } => {
                validate_secure_path(model_path, "AI model path")
                    .map_err(|e| IDEError::Validation(format!("Invalid model path: {}", e)))?;
                if !std::path::Path::new(model_path).exists() {
                    return Err(IDEError::Validation(format!(
                        "Model path does not exist: {}",
                        model_path
                    )));
                }
            }
            AIProvider::Claude { api_key: _ } => {
                // API key is already encrypted, validation handled in setter
            }
            _ => {} // Other providers don't need special validation
        }
        Ok(())
    }

    /// Get completions using LSP integration
    pub async fn get_completions_lsp(&self, context: &AIContext) -> IDEResult<Vec<Completion>> {
        self.ensure_initialized().await?;
        let client_guard = self
            .lsp_client
            .read()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire LSP client: {}", e)))?;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| IDEError::AIService("LSP client not initialized".to_string()))?;

        // Use LSP for completions
        let completions = execute_with_retry(
            || client.get_completions_with_ai(context.clone()),
            3,
            "LSP completion request",
        )
        .await
        .map_err(|e| IDEError::AIService(format!("LSP completion failed: {}", e)))?;

        audit_logger::log_event(
            "ai_completion_requested",
            &serde_json::json!({
                "completions_count": completions.len(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        Ok(completions)
    }

    /// Ensure service is initialized
    async fn ensure_initialized(&self) -> IDEResult<()> {
        let initialized = self
            .initialized
            .read()
            .map_err(|e| IDEError::AIService(format!("Failed to check initialization: {}", e)))?;
        if !*initialized {
            return Err(IDEError::AIService(
                "AI service not initialized".to_string(),
            ));
        }
        Ok(())
    }

    /// Shutdown the service
    pub async fn shutdown(&self) -> IDEResult<()> {
        let mut initialized = self
            .initialized
            .write()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire initialization lock: {}", e)))?;
        *initialized = false;

        let mut client_lock = self
            .lsp_client
            .write()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire LSP client lock: {}", e)))?;
        *client_lock = None;

        audit_logger::log_event(
            "ai_service_shutdown",
            &serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        log::info!("AI service shut down successfully");
        Ok(())
    }
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}

/// Local AIProvider enum with secure credential handling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIProvider {
    Mock,
    OpenAI,
    Anthropic,
    CodeLlamaRust {
        model_size: rust_ai_ide_ai_inference::ModelSize,
    },
    StarCoderRust {
        model_size: rust_ai_ide_ai_inference::ModelSize,
    },
    Local {
        model_path: String,
    },
    Claude {
        /// Encrypted API key identifier (not the plain text key)
        api_key_id: String,
    },
    GenericAPI {
        /// Base URL for the API endpoint
        base_url:   String,
        /// Encrypted API key identifier
        api_key_id: String,
        /// Model identifier for the API
        model:      String,
    },
}

impl AIProvider {
    /// Set API key securely for providers that need it
    pub async fn set_api_key(&mut self, plain_key: &str) -> IDEResult<()> {
        let encryption_manager = GLOBAL_ENCRYPTION_MANAGER.clone();
        let encrypted_key = encryption_manager
            .encrypt(plain_key.as_bytes())
            .await
            .map_err(|e| IDEError::Security(format!("Failed to encrypt API key: {}", e)))?;

        let key_id = format!(
            "api_key_{}_{}",
            self.provider_name(),
            chrono::Utc::now().timestamp()
        );

        // In a real implementation, store encrypted_key in secure storage with key_id
        // For now, we'll just use the key_id as a placeholder
        match self {
            AIProvider::Claude { api_key_id } => {
                *api_key_id = key_id;
            }
            AIProvider::GenericAPI { api_key_id, .. } => {
                *api_key_id = key_id;
            }
            _ => {
                return Err(IDEError::Validation(format!(
                    "Provider {} does not support API keys",
                    self.provider_name()
                )));
            }
        }

        audit_logger::log_event(
            "api_key_set",
            &serde_json::json!({
                "provider": self.provider_name(),
                "key_id": key_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        Ok(())
    }

    /// Get API key securely (decrypt from storage)
    pub async fn get_api_key(&self) -> IDEResult<String> {
        let key_id = match self {
            AIProvider::Claude { api_key_id } => api_key_id,
            AIProvider::GenericAPI { api_key_id, .. } => api_key_id,
            _ =>
                return Err(IDEError::Validation(format!(
                    "Provider {} does not have API key",
                    self.provider_name()
                ))),
        };

        // In a real implementation, retrieve encrypted_key from secure storage using key_id
        // For now, return a placeholder
        let encryption_manager = GLOBAL_ENCRYPTION_MANAGER.clone();
        let encrypted_key = Vec::new(); // Placeholder - would retrieve from storage

        let decrypted_key = encryption_manager
            .decrypt(&encrypted_key)
            .await
            .map_err(|e| IDEError::Security(format!("Failed to decrypt API key: {}", e)))?;

        String::from_utf8(decrypted_key).map_err(|e| IDEError::Security(format!("Invalid decrypted API key: {}", e)))
    }

    /// Get provider name for logging
    pub fn provider_name(&self) -> &'static str {
        match self {
            AIProvider::Mock => "mock",
            AIProvider::OpenAI => "openai",
            AIProvider::Anthropic => "anthropic",
            AIProvider::CodeLlamaRust { .. } => "codellama_rust",
            AIProvider::StarCoderRust { .. } => "starcoder_rust",
            AIProvider::Local { .. } => "local",
            AIProvider::Claude { .. } => "claude",
            AIProvider::GenericAPI { .. } => "generic_api",
        }
    }
}

/// Trait for AI services to implement common functionality
#[async_trait]
pub trait AIServiceTrait: Send + Sync {
    async fn get_completions(&mut self, context: AIContext) -> IDEResult<Vec<Completion>>;
    async fn get_task_response(&mut self, context: AIContext, task: String) -> IDEResult<String>;
    async fn is_healthy(&self) -> IDEResult<bool>;
    fn provider_type(&self) -> &str;
}

/// Wrapper for AIService to implement AIServiceTrait with enhanced functionality
pub struct WrappedAIService {
    service:          Arc<AIService>,
    provider:         AIProvider,
    inference_engine: Arc<TokioRwLock<Option<Arc<Mutex<dyn InferenceEngine + Send + Sync>>>>>,
    background_tasks: Arc<TokioRwLock<Vec<String>>>,
}

impl WrappedAIService {
    pub fn new(service: Arc<AIService>, provider: AIProvider) -> Self {
        Self {
            service,
            provider,
            inference_engine: Arc::new(RwLock::new(None)),
            background_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn new_with_inference_engine(
        service: Arc<AIService>,
        provider: AIProvider,
        inference_engine: Arc<Mutex<dyn InferenceEngine + Send + Sync>>,
    ) -> Self {
        let mut instance = Self::new(service, provider);
        {
            let mut engine_lock = instance.inference_engine.try_write().unwrap();
            *engine_lock = Some(inference_engine);
        }
        instance
    }

    /// Initialize the service with background task management
    pub async fn initialize(&self) -> IDEResult<()> {
        let service_clone = self.service.clone();
        let provider_clone = self.provider.clone();

        let task_id = spawn_background_task(
            async move {
                if let Err(e) = service_clone.initialize(provider_clone).await {
                    log::error!("Failed to initialize AI service in background: {:?}", e);
                }
            },
            "ai_service_init",
        );

        let mut tasks = self
            .background_tasks
            .write()
            .await
            .map_err(|e| IDEError::AIService(format!("Failed to acquire background tasks lock: {}", e)))?;
        tasks.push(task_id);

        Ok(())
    }

    /// Cleanup background tasks on drop
    pub async fn cleanup(&self) -> IDEResult<()> {
        let tasks: Vec<String> = {
            let mut tasks_lock = self
                .background_tasks
                .write()
                .await
                .map_err(|e| IDEError::AIService(format!("Failed to acquire background tasks lock: {}", e)))?;
            tasks_lock.drain(..).collect()
        };

        for task_id in tasks {
            log::debug!("Cleaning up background task: {}", task_id);
            // Task cleanup would be handled by the spawn_background_task macro
        }

        self.service.shutdown().await
    }

    /// Create inference engine based on provider configuration
    async fn create_inference_engine(&self) -> Result<Arc<Mutex<dyn InferenceEngine + Send + Sync>>, String> {
        match &self.provider {
            AIProvider::OpenAI => {
                // Create OpenAI inference engine
                let engine = LocalInferenceEngine::new(
                    &InferenceAIProvider::OpenAI,
                    "https://api.openai.com",
                    "gpt-4",
                )
                .await
                .map_err(|e| format!("Failed to create OpenAI inference engine: {:?}", e))?;
                Ok(Arc::new(Mutex::new(engine)))
            }
            AIProvider::CodeLlamaRust { model_size } => {
                // Create CodeLlama inference engine
                let engine = LocalInferenceEngine::new(
                    &InferenceAIProvider::CodeLlamaRust {
                        model_size: *model_size,
                    },
                    "http://localhost:8000", // Local inference server
                    &format!("codellama_{:?}", model_size),
                )
                .await
                .map_err(|e| format!("Failed to create CodeLlama inference engine: {:?}", e))?;
                Ok(Arc::new(Mutex::new(engine)))
            }
            AIProvider::StarCoderRust { model_size } => {
                // Create StarCoder inference engine
                let engine = LocalInferenceEngine::new(
                    &InferenceAIProvider::StarCoderRust {
                        model_size: *model_size,
                    },
                    "http://localhost:8000", // Local inference server
                    &format!("starcoder_{:?}", model_size),
                )
                .await
                .map_err(|e| format!("Failed to create StarCoder inference engine: {:?}", e))?;
                Ok(Arc::new(Mutex::new(engine)))
            }
            AIProvider::Local { model_path } => {
                // Create local model inference engine
                let engine = LocalInferenceEngine::new(
                    &InferenceAIProvider::Local {
                        model_path: model_path.clone(),
                    },
                    "http://localhost:8000",
                    "local_model",
                )
                .await
                .map_err(|e| format!("Failed to create local inference engine: {:?}", e))?;
                Ok(Arc::new(Mutex::new(engine)))
            }
            _ => Err(format!(
                "Unsupported provider for inference: {:?}",
                self.provider
            )),
        }
    }

    /// Get or create inference engine
    async fn get_inference_engine(&mut self) -> Result<Arc<Mutex<dyn InferenceEngine + Send + Sync>>, String> {
        if let Some(engine) = &self.inference_engine {
            return Ok(engine.clone());
        }

        let engine = self.create_inference_engine().await?;
        self.inference_engine = Some(engine.clone());
        Ok(engine)
    }
}

#[async_trait]
impl AIServiceTrait for WrappedAIService {
    async fn get_completions(&mut self, context: AIContext) -> IDEResult<Vec<Completion>> {
        // Log the completion request
        log::info!(
            "Processing completion request for provider: {:?}",
            self.provider
        );

        let start_time = Instant::now();

        match self.provider {
            AIProvider::Mock => {
                // Return mock completions for testing
                Ok(vec![Completion {
                    label: "println!(\"Hello, world!\");".to_string(),
                    detail: Some("Mock completion".to_string()),
                    documentation: None,
                    kind: Some(rust_ai_ide_lsp::CompletionItemKind::Function),
                    insert_text: Some("println!(\"Hello, world!\");".to_string()),
                    insert_text_format: Some(rust_ai_ide_lsp::InsertTextFormat::PlainText),
                    ..Default::default()
                }])
            }
            _ => {
                // Try LSP first, fallback to inference engine
                match self.service.get_completions_lsp(&context).await {
                    Ok(completions) => {
                        let duration = start_time.elapsed();
                        log::info!("LSP completion request completed in {:?}", duration);
                        Ok(completions)
                    }
                    Err(e) => {
                        log::warn!(
                            "LSP completion failed, falling back to inference engine: {:?}",
                            e
                        );

                        // Fallback to inference engine
                        let engine_lock = self.inference_engine.read().await;
                        if let Some(engine) = engine_lock.as_ref() {
                            let mut engine = engine
                                .lock()
                                .map_err(|e| IDEError::AIService(format!("Failed to lock inference engine: {}", e)))?;

                            let config = CodeCompletionConfig {
                                max_length:           100,
                                context_lines:        5,
                                use_fim:              matches!(self.provider, AIProvider::StarCoderRust { .. }),
                                indentation:          "    ".to_string(),
                                use_context_digest:   true,
                                return_full_function: false,
                            };

                            let current_code = context.current_code.clone();
                            let prefix = context
                                .cursor_position
                                .and_then(|(line, col)| {
                                    current_code.lines().nth(line as usize).map(|line_content| {
                                        if (col as usize) < line_content.len() {
                                            line_content[..col as usize].to_string()
                                        } else {
                                            line_content.to_string()
                                        }
                                    })
                                })
                                .unwrap_or_else(|| current_code.clone());

                            let suffix = context
                                .cursor_position
                                .and_then(|(line, col)| {
                                    current_code.lines().nth(line as usize).map(|line_content| {
                                        if (col as usize) < line_content.len() {
                                            line_content[col as usize..].to_string()
                                        } else {
                                            String::new()
                                        }
                                    })
                                })
                                .unwrap_or_default();

                            let context_str = format!("{}\n{}", prefix, suffix);

                            let result = execute_with_retry(
                                || engine.generate_code_completion(&context_str, &prefix, &config),
                                3,
                                "inference engine completion",
                            )
                            .await
                            .map_err(|e| IDEError::AIService(format!("Inference engine error: {:?}", e)))?;

                            let completion = Completion {
                                label: result.completion.clone(),
                                detail: Some(format!("Confidence: {:.2}", result.confidence_score)),
                                documentation: None,
                                kind: Some(rust_ai_ide_lsp::CompletionItemKind::Function),
                                insert_text: Some(result.completion),
                                insert_text_format: Some(rust_ai_ide_lsp::InsertTextFormat::PlainText),
                                ..Default::default()
                            };

                            let duration = start_time.elapsed();
                            log::info!("Fallback completion request completed in {:?}", duration);

                            Ok(vec![completion])
                        } else {
                            Err(IDEError::AIService(
                                "No inference engine available for completion".to_string(),
                            ))
                        }
                    }
                }
            }
        }
    }

    async fn get_task_response(&mut self, context: AIContext, task: String) -> IDEResult<String> {
        // Log the task request
        log::info!(
            "Processing task request: {} for provider: {:?}",
            task,
            self.provider
        );

        let start_time = Instant::now();

        // Sanitize input
        let sanitizer = TauriInputSanitizer::new();
        let sanitized_task = sanitizer
            .sanitize_string(&task, "task description")
            .map_err(|e| IDEError::Validation(format!("Invalid task input: {}", e)))?;

        match self.provider {
            AIProvider::Mock => {
                // Return mock response for testing
                Ok(format!("Mock response for task: {}", sanitized_task))
            }
            _ => {
                // Use inference engine with retry logic
                let engine_lock = self.inference_engine.read().await;
                if let Some(engine) = engine_lock.as_ref() {
                    let mut engine = engine
                        .lock()
                        .map_err(|e| IDEError::AIService(format!("Failed to lock inference engine: {}", e)))?;

                    let config = GenerationConfig {
                        max_tokens:        500,
                        temperature:       0.7,
                        top_p:             0.9,
                        frequency_penalty: 0.0,
                        presence_penalty:  0.0,
                        stop_sequences:    vec![],
                        echo:              false,
                        stream:            false,
                    };

                    let prompt = format!("{}\n\nTask: {}", context.current_code, sanitized_task);

                    let result = execute_with_retry(
                        || engine.generate_text(&prompt, &config),
                        3,
                        "inference engine task response",
                    )
                    .await
                    .map_err(|e| IDEError::AIService(format!("Inference engine error: {:?}", e)))?;

                    let duration = start_time.elapsed();
                    log::info!("Task response generated in {:?}", duration);

                    Ok(result.text)
                } else {
                    Err(IDEError::AIService(
                        "No inference engine available for task response".to_string(),
                    ))
                }
            }
        }
    }

    async fn is_healthy(&self) -> IDEResult<bool> {
        // Perform actual health checks by testing service connectivity and response times
        match &self.provider {
            AIProvider::Mock => Ok(true), // Mock is always healthy
            _ => {
                // Check LSP service health
                let lsp_healthy = match self
                    .service
                    .get_completions_lsp(&AIContext {
                        current_code:    "test".to_string(),
                        file_name:       None,
                        cursor_position: Some((0, 0)),
                        selection:       None,
                        project_context: HashMap::new(),
                    })
                    .await
                {
                    Ok(_) => true,
                    Err(e) => {
                        log::debug!("LSP health check failed: {:?}", e);
                        false
                    }
                };

                if lsp_healthy {
                    return Ok(true);
                }

                // Fallback to inference engine health check
                let engine_lock = self.inference_engine.read().await;
                if let Some(engine) = engine_lock.as_ref() {
                    // Basic health check: verify engine is accessible and not poisoned
                    match engine.try_lock() {
                        Ok(_) => {
                            log::debug!(
                                "Inference engine health check passed for provider: {:?}",
                                self.provider
                            );
                            Ok(true)
                        }
                        Err(_) => {
                            log::warn!("Engine lock is poisoned for provider: {:?}", self.provider);
                            Ok(false)
                        }
                    }
                } else {
                    log::warn!("No inference engine available for health check");
                    Ok(false)
                }
            }
        }
    }

    fn provider_type(&self) -> &str {
        self.provider.provider_name()
    }
}

impl Drop for WrappedAIService {
    fn drop(&mut self) {
        // Cleanup background tasks
        let runtime = tokio::runtime::Handle::try_current();
        if let Ok(handle) = runtime {
            handle.spawn(async move {
                if let Err(e) = self.cleanup().await {
                    log::error!("Failed to cleanup AI service during drop: {:?}", e);
                }
            });
        } else {
            // If no runtime, we can't cleanup async, but we can at least log
            log::warn!("No tokio runtime available for AI service cleanup");
        }
    }
}

/// Configuration for a pooled service
#[derive(Clone, Debug)]
pub struct PooledServiceConfig {
    pub provider:           AIProvider,
    pub max_connections:    usize,
    pub connection_timeout: Duration,
    pub idle_timeout:       Duration,
}

/// Connection pool entry
pub struct PooledConnection<T> {
    service:   Arc<T>,
    last_used: Instant,
    in_use:    bool,
}

impl<T> PooledConnection<T> {
    pub fn new(service: Arc<T>) -> Self {
        Self {
            service,
            last_used: Instant::now(),
            in_use: false,
        }
    }

    pub fn is_expired(&self, config: &PooledServiceConfig) -> bool {
        self.last_used.elapsed() > config.idle_timeout
    }
}

/// Generic connection pool for any AI service
pub struct ConnectionPool<T: Send + Sync> {
    config:      PooledServiceConfig,
    connections: Mutex<Vec<PooledConnection<T>>>,
    semaphore:   Semaphore,
}

impl<T: Send + Sync> ConnectionPool<T> {
    pub fn new(config: PooledServiceConfig, initial_services: Vec<Arc<T>>) -> Self {
        let mut connections = Vec::new();
        for service in initial_services {
            connections.push(PooledConnection::new(service));
        }

        Self {
            connections: Mutex::new(connections),
            semaphore: Semaphore::new(config.max_connections),
            config,
        }
    }

    /// Acquire a service connection from the pool
    pub async fn acquire(&self) -> IDEResult<PoolGuard<T>> {
        let permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| IDEError::AIService(format!("Failed to acquire semaphore permit: {}", e)))?;
        let mut connections = self
            .connections
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire connections lock: {}", e)))?;

        // Find an available connection
        for conn in connections.iter_mut() {
            if !conn.in_use && !conn.is_expired(&self.config) {
                conn.in_use = true;
                conn.last_used = Instant::now();
                return Ok(PoolGuard {
                    connection: conn.service.clone(),
                    _permit:    permit,
                    pool:       self,
                });
            }
        }

        Err(IDEError::AIService(
            "No available connections in pool".to_string(),
        ))
    }

    /// Return a connection to the pool (internal method)
    fn release(&self, service: &Arc<T>) {
        let mut connections = match self.connections.lock() {
            Ok(lock) => lock,
            Err(e) => {
                log::error!("Failed to acquire connections lock for release: {}", e);
                return;
            }
        };

        for conn in connections.iter_mut() {
            if Arc::ptr_eq(&conn.service, service) {
                conn.in_use = false;
                break;
            }
        }
    }

    /// Add a new service to the pool
    pub fn add_service(&self, service: Arc<T>) -> IDEResult<()> {
        let mut connections = self
            .connections
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire connections lock: {}", e)))?;
        connections.push(PooledConnection::new(service));
        Ok(())
    }

    /// Get pool status
    pub fn status(&self) -> IDEResult<PoolStatus> {
        let connections = self
            .connections
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire connections lock: {}", e)))?;
        let total = connections.len();
        let in_use = connections.iter().filter(|c| c.in_use).count();
        let available = total - in_use;

        Ok(PoolStatus {
            total,
            available,
            in_use,
        })
    }
}

/// Guard for pooled connections
pub struct PoolGuard<'a, T: Send + Sync> {
    connection: Arc<T>,
    _permit:    tokio::sync::SemaphorePermit<'a>,
    pool:       &'a ConnectionPool<T>,
}

impl<'a, T: Send + Sync> std::ops::Deref for PoolGuard<'a, T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<'a, T: Send + Sync> Drop for PoolGuard<'a, T> {
    fn drop(&mut self) {
        self.pool.release(&self.connection);
    }
}

/// Pool status information
#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub total:     usize,
    pub available: usize,
    pub in_use:    usize,
}

/// AI Service Registry for service discovery and management
pub struct AIServiceRegistry {
    services: Mutex<HashMap<String, Arc<WrappedAIService>>>,
    pools:    Mutex<HashMap<String, Arc<ConnectionPool<WrappedAIService>>>>,
}

impl AIServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Mutex::new(HashMap::new()),
            pools:    Mutex::new(HashMap::new()),
        }
    }

    /// Register a direct service (non-pooled)
    pub fn register_service(&self, name: &str, service: Arc<WrappedAIService>) -> IDEResult<()> {
        let mut services = self
            .services
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire services lock: {}", e)))?;
        if services.contains_key(name) {
            return Err(IDEError::Validation(format!(
                "Service '{}' already registered",
                name
            )));
        }
        services.insert(name.to_string(), service);

        audit_logger::log_event(
            "ai_service_registered",
            &serde_json::json!({
                "service_name": name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        Ok(())
    }

    /// Register a pooled service
    pub fn register_pooled_service(
        &self,
        name: &str,
        config: PooledServiceConfig,
        initial_services: Vec<WrappedAIService>,
    ) -> IDEResult<()> {
        let services: Vec<Arc<WrappedAIService>> = initial_services.into_iter().map(Arc::new).collect();
        let pool = Arc::new(ConnectionPool::new(config, services));

        let mut pools = self
            .pools
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire pools lock: {}", e)))?;
        pools.insert(name.to_string(), pool);

        audit_logger::log_event(
            "ai_pooled_service_registered",
            &serde_json::json!({
                "service_name": name,
                "pool_size": services.len(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        Ok(())
    }

    /// Get a direct service by name
    pub fn get_service(&self, name: &str) -> IDEResult<Arc<WrappedAIService>> {
        let services = self
            .services
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire services lock: {}", e)))?;
        services
            .get(name)
            .cloned()
            .ok_or_else(|| IDEError::Validation(format!("Service '{}' not found", name)))
    }

    /// Get a pooled service connection
    pub async fn get_pooled_service(&self, name: &str) -> IDEResult<PoolGuard<WrappedAIService>> {
        let pools = self
            .pools
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire pools lock: {}", e)))?;
        let pool = pools
            .get(name)
            .ok_or_else(|| IDEError::Validation(format!("Pooled service '{}' not found", name)))?;
        pool.acquire().await.map_err(|e| IDEError::AIService(e))
    }

    /// List all registered services
    pub fn list_services(&self) -> IDEResult<Vec<String>> {
        let services = self
            .services
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire services lock: {}", e)))?;
        let service_names: Vec<String> = services.keys().cloned().collect();
        Ok(service_names)
    }

    /// List all pooled services
    pub fn list_pooled_services(&self) -> IDEResult<Vec<String>> {
        let pools = self
            .pools
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire pools lock: {}", e)))?;
        let pool_names: Vec<String> = pools.keys().cloned().collect();
        Ok(pool_names)
    }

    /// Get health status of all services
    pub async fn health_check(&self) -> IDEResult<HashMap<String, bool>> {
        let mut status = HashMap::new();

        // Check direct services
        let services = self
            .services
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire services lock: {}", e)))?;

        for (name, service) in services.iter() {
            let is_healthy = match service.is_healthy().await {
                Ok(healthy) => healthy,
                Err(e) => {
                    log::warn!("Health check failed for service {}: {:?}", name, e);
                    false
                }
            };
            status.insert(name.clone(), is_healthy);
        }

        // Check pooled services
        let pools = self
            .pools
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire pools lock: {}", e)))?;

        for (name, pool) in pools.iter() {
            let pool_status = pool
                .status()
                .map_err(|e| IDEError::AIService(format!("Failed to get pool status for {}: {}", name, e)))?;
            let is_healthy = pool_status.available > 0;
            status.insert(name.clone(), is_healthy);
        }

        audit_logger::log_event(
            "ai_services_health_check",
            &serde_json::json!({
                "services_checked": status.len(),
                "healthy_count": status.values().filter(|&&h| h).count(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        Ok(status)
    }

    /// Unregister a service
    pub fn unregister_service(&self, name: &str) -> IDEResult<()> {
        let mut services = self
            .services
            .lock()
            .map_err(|e| IDEError::AIService(format!("Failed to acquire services lock: {}", e)))?;
        if services.remove(name).is_none() {
            return Err(IDEError::Validation(format!(
                "Service '{}' not found",
                name
            )));
        }

        audit_logger::log_event(
            "ai_service_unregistered",
            &serde_json::json!({
                "service_name": name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        Ok(())
    }
}

/// Global service registry instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_AI_REGISTRY: Arc<AIServiceRegistry> = Arc::new(AIServiceRegistry::new());
}

impl Default for AIServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
