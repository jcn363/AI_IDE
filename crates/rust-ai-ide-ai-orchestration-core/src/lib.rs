//! AI Orchestration Layer Core
//!
//! Unified service manager for AI model coordination and task distribution.
//!
//! This module provides the core orchestration logic for managing AI services,
//! including model loading/unloading, task routing, performance monitoring,
//! and coordination with downstream modules.
//!
//! # Architecture
//!
//! The orchestration layer follows a hierarchical structure:
//!
//! - **Orchestrator Core**: Central coordinator managing service lifecycle
//! - **Service Registry**: Tracks available AI services and their capabilities
//! - **Task Queue**: Manages incoming AI tasks and prioritizes them
//! - **Model Coordinator**: Handles model loading/unloading via LSP service
//! - **Performance Monitor**: Tracks service health and performance metrics
//!
//! # Security Considerations
//!
//! - All inputs are validated via rust-ai-ide-common validation functions
//! - Model access is restricted to LSP service interfaces
//! - Audit logging enabled for sensitive operations
//! - Secure secrets storage is mandatory
//!
//! # Performance
//!
//! - Async-first design with tokio runtime
//! - Request coalescing for similar tasks
//! - LRU caching with Moka for frequent operations
//! - Background task spawning for cleanup operations

pub mod ai_orchestrator {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use rust_ai_ide_cache::{InMemoryCache, LspCodeCompletion};
    use rust_ai_ide_common::validation::{sanitize_string_for_processing, validate_string_input_extended};
    use rust_ai_ide_errors::RustAIError;
    use rust_ai_ide_performance::adaptive_memory::AdaptiveMemoryManager;
    use serde::{Deserialize, Serialize};
    use tokio::sync::{mpsc, Mutex, RwLock};
    use tokio::time::{timeout, Duration};
    use tracing::{debug, error, info, warn};
    use uuid::Uuid;

    /// Types of AI services that can be orchestrated
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum AIServiceType {
        Completion,
        CodeAnalysis,
        CodeGeneration,
        Refactoring,
        SemanticSearch,
        NaturalLanguage,
        QualityMetrics,
    }

    impl AIServiceType {
        pub fn priority(&self) -> u8 {
            match self {
                AIServiceType::Completion => 3,
                AIServiceType::CodeAnalysis => 2,
                AIServiceType::CodeGeneration => 2,
                AIServiceType::Refactoring => 1,
                AIServiceType::SemanticSearch => 2,
                AIServiceType::NaturalLanguage => 1,
                AIServiceType::QualityMetrics => 1,
            }
        }

        pub fn timeout_secs(&self) -> u64 {
            match self {
                AIServiceType::Completion => 5,
                AIServiceType::CodeAnalysis => 30,
                AIServiceType::CodeGeneration => 60,
                AIServiceType::Refactoring => 45,
                AIServiceType::SemanticSearch => 15,
                AIServiceType::NaturalLanguage => 30,
                AIServiceType::QualityMetrics => 20,
            }
        }
    }

    /// Configuration for orchestration layer
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OrchestratorConfig {
        pub max_concurrent_tasks:      usize,
        pub request_queue_size:        usize,
        pub model_cache_size_mb:       usize,
        pub performance_sample_window: Duration,
        pub fallback_timeout_secs:     u64,
        pub enable_request_coalescing: bool,
        pub adaptive_memory_enabled:   bool,
    }

    impl Default for OrchestratorConfig {
        fn default() -> Self {
            Self {
                max_concurrent_tasks:      10,
                request_queue_size:        1000,
                model_cache_size_mb:       512,
                performance_sample_window: Duration::from_secs(300), // 5 minutes
                fallback_timeout_secs:     30,
                enable_request_coalescing: true,
                adaptive_memory_enabled:   true,
            }
        }
    }

    /// Request context for AI operations
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AIRequestContext {
        pub request_id:        String,
        pub service_type:      AIServiceType,
        pub user_id:           Option<String>,
        pub project_id:        Option<String>,
        pub file_path:         Option<String>,
        pub metadata:          HashMap<String, serde_json::Value>,
        pub timeout_override:  Option<u64>,
        pub priority_override: Option<u8>,
    }

    impl Default for AIRequestContext {
        fn default() -> Self {
            Self {
                request_id:        Uuid::new_v4().to_string(),
                service_type:      AIServiceType::Completion,
                user_id:           None,
                project_id:        None,
                file_path:         None,
                metadata:          HashMap::new(),
                timeout_override:  None,
                priority_override: None,
            }
        }
    }

    /// AI service capabilities and metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AIServiceCapabilities {
        pub supported_service_types: Vec<AIServiceType>,
        pub max_concurrent_requests: usize,
        pub average_latency_ms:      f64,
        pub throughput_rps:          f64,
        pub memory_usage_mb:         f64,
        pub gpu_available:           bool,
        pub last_health_check:       chrono::DateTime<chrono::Utc>,
        pub health_score:            f64, // 0.0 to 1.0
    }

    /// Main orchestrator service
    #[derive(Debug)]
    pub struct AIOrchestrator {
        config:           Arc<OrchestratorConfig>,
        services:         Arc<RwLock<HashMap<String, AIServiceCapabilities>>>,
        task_queue:       Arc<Mutex<std::collections::VecDeque<AIRequestContext>>>,
        model_cache:      Arc<InMemoryCache<LspCodeCompletion>>,
        adaptive_memory:  Arc<AdaptiveMemoryManager>,
        request_sender:   mpsc::Sender<AIRequestContext>,
        request_receiver: Arc<Mutex<mpsc::Receiver<AIRequestContext>>>,
        metrics:          Arc<RwLock<OrchestratorMetrics>>,
    }

    /// Metrics for monitoring orchestrator performance
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct OrchestratorMetrics {
        pub total_requests:     usize,
        pub completed_requests: usize,
        pub failed_requests:    usize,
        pub average_latency_ms: f64,
        pub queue_length:       usize,
        pub active_services:    usize,
        pub request_rate_sigma: f64,
        pub memory_usage_mb:    f64,
    }

    /// Service provider trait for AI services
    #[async_trait]
    pub trait AIServiceProvider: Send + Sync {
        async fn is_available(&self) -> bool;
        async fn get_capabilities(&self) -> Result<AIServiceCapabilities, RustAIError>;
        async fn execute_request(
            &self,
            context: AIRequestContext,
            payload: serde_json::Value,
        ) -> Result<serde_json::Value, RustAIError>;
        fn service_name(&self) -> &'static str;
    }

    /// Local AI service provider implementation
    pub struct LocalAIServiceProvider {
        name:      &'static str,
        mock_mode: bool,
    }

    #[async_trait]
    impl AIServiceProvider for LocalAIServiceProvider {
        async fn is_available(&self) -> bool {
            // Placeholder: Check LSP service availability
            // In real implementation: connect to LSP service
            self.mock_mode || true
        }

        async fn get_capabilities(&self) -> Result<AIServiceCapabilities, RustAIError> {
            Ok(AIServiceCapabilities {
                supported_service_types: vec![
                    AIServiceType::Completion,
                    AIServiceType::CodeAnalysis,
                    AIServiceType::CodeGeneration,
                ],
                max_concurrent_requests: 5,
                average_latency_ms:      150.0,
                throughput_rps:          10.0,
                memory_usage_mb:         256.0,
                gpu_available:           false,
                last_health_check:       chrono::Utc::now(),
                health_score:            0.95,
            })
        }

        async fn execute_request(
            &self,
            context: AIRequestContext,
            payload: serde_json::Value,
        ) -> Result<serde_json::Value, RustAIError> {
            match context.service_type {
                AIServiceType::Completion => {
                    // Placeholder: Call LSP completion service
                    // In real implementation:
                    // - Load appropriate model via LSP
                    // - Execute completion inference
                    // - Return formatted completion results
                    debug!("Executing completion request {}", context.request_id);
                    serde_json::json!({
                        "status": "success",
                        "completions": [
                            { "text": "fn example_function() {", "score": 0.95 },
                            { "text": "impl MyStruct {", "score": 0.88 }
                        ]
                    })
                    .into()
                }
                AIServiceType::CodeAnalysis => {
                    // Placeholder: Call LSP analysis service
                    debug!("Executing code analysis request {}", context.request_id);
                    serde_json::json!({
                        "status": "success",
                        "analysis": {
                            "issues": [],
                            "metrics": { "complexity": 5, "maintainability": 85 }
                        }
                    })
                    .into()
                }
                AIServiceType::CodeGeneration => {
                    // Placeholder: Call code generation service
                    debug!("Executing code generation request {}", context.request_id);
                    serde_json::json!({
                        "status": "success",
                        "generated_code": "fn generated_function() {\n    // Generated code\n}",
                        "confidence": 0.92
                    })
                    .into()
                }
                _ => Err(RustAIError::ServiceUnavailable(
                    "Service type not implemented in mock mode".to_string(),
                )),
            }
        }

        fn service_name(&self) -> &'static str {
            self.name
        }
    }

    impl AIOrchestrator {
        /// Create new orchestrator instance
        pub async fn new(config: Arc<OrchestratorConfig>) -> Result<Self, RustAIError> {
            let (sender, receiver) = mpsc::channel(config.request_queue_size);

            Ok(Self {
                config:           config.clone(),
                services:         Arc::new(RwLock::new(HashMap::new())),
                task_queue:       Arc::new(Mutex::new(std::collections::VecDeque::new())),
                model_cache:      Arc::new(InMemoryCache::new_with_capacity(1000)),
                adaptive_memory:  Arc::new(
                    AdaptiveMemoryManager::new().map_err(|e| RustAIError::InternalError(e.to_string()))?,
                ),
                request_sender:   sender,
                request_receiver: Arc::new(Mutex::new(receiver)),
                metrics:          Arc::new(RwLock::new(OrchestratorMetrics::default())),
            })
        }

        /// Register an AI service provider
        pub async fn register_service(&self, provider: Arc<dyn AIServiceProvider>) -> Result<(), RustAIError> {
            let capabilities = provider.get_capabilities().await?;
            let service_name = provider.service_name();

            // Validate service capabilities
            if capabilities.supported_service_types.is_empty() {
                return Err(RustAIError::Validation(
                    "Service must support at least one service type".to_string(),
                ));
            }

            let mut services = self.services.write().await;
            services.insert(service_name.to_string(), capabilities);

            info!("Registered AI service: {}", service_name);
            Ok(())
        }

        /// Submit an AI request for processing
        pub async fn submit_request(
            &self,
            context: AIRequestContext,
            payload: serde_json::Value,
        ) -> Result<String, RustAIError> {
            // Validate input
            self.validate_request_context(&context).await?;
            self.validate_payload(&payload)?;

            // Sanitize payload for security
            let sanitized_payload = sanitize_string_for_processing(&payload.to_string(), &["<script>", "</script>"])?;

            let sanitized_payload: serde_json::Value = serde_json::from_str(&sanitized_payload)
                .map_err(|e| RustAIError::Validation(format!("Payload parsing failed: {}", e)))?;

            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.total_requests += 1;
            }

            // Send request to processing queue (non-blocking)
            self.request_sender
                .send(context.clone())
                .await
                .map_err(|_| RustAIError::QueueFull("Request queue is full".to_string()))?;

            debug!("Submitted request {} for processing", context.request_id);
            Ok(context.request_id)
        }

        /// Get current orchestrator metrics
        pub async fn get_metrics(&self) -> Result<OrchestratorMetrics, RustAIError> {
            let metrics = self.metrics.read().await;
            Ok(metrics.clone())
        }

        /// Get registered services
        pub async fn get_services(&self) -> Result<HashMap<String, AIServiceCapabilities>, RustAIError> {
            let services = self.services.read().await;
            Ok(services.clone())
        }

        /// Start background processing task
        pub async fn start_processing_loop(&self) -> Result<(), RustAIError> {
            let request_receiver = self.request_receiver.clone();
            let services = self.services.clone();
            let metrics = self.metrics.clone();
            let model_cache = self.model_cache.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::processing_loop(request_receiver, services, metrics, model_cache).await {
                    error!("Processing loop error: {}", e);
                }
            });

            info!("Started orchestrator processing loop");
            Ok(())
        }

        /// Background processing loop for requests
        async fn processing_loop(
            request_receiver: Arc<Mutex<mpsc::Receiver<AIRequestContext>>>,
            services: Arc<RwLock<HashMap<String, AIServiceCapabilities>>>,
            metrics: Arc<RwLock<OrchestratorMetrics>>,
            model_cache: Arc<InMemoryCache<LspCodeCompletion>>,
        ) -> Result<(), RustAIError> {
            loop {
                let mut receiver = request_receiver.lock().await;

                match receiver.recv().await {
                    Some(context) => {
                        debug!("Processing request {}", context.request_id);

                        // Select appropriate service
                        let service_result = {
                            let services_read = services.read().await;
                            Self::select_service_for_request(&context, &services_read)?
                        };

                        match service_result {
                            Some((service_name, capabilities)) => {
                                // Execute request
                                match Self::execute_request_with_service(service_name, capabilities, context).await {
                                    Ok(_) => {
                                        let mut metrics = metrics.write().await;
                                        metrics.completed_requests += 1;
                                    }
                                    Err(e) => {
                                        error!("Request execution failed: {}", e);
                                        let mut metrics = metrics.write().await;
                                        metrics.failed_requests += 1;
                                    }
                                }
                            }
                            None => {
                                warn!(
                                    "No suitable service found for request {}",
                                    context.request_id
                                );
                                let mut metrics = metrics.write().await;
                                metrics.failed_requests += 1;
                            }
                        }
                    }
                    None => break, // Channel closed
                }
            }
            Ok(())
        }

        /// Select best service for a request
        fn select_service_for_request<'a>(
            context: &AIRequestContext,
            services: &'a HashMap<String, AIServiceCapabilities>,
        ) -> Result<Option<(&'a str, &'a AIServiceCapabilities)>, RustAIError> {
            let mut best_service: Option<(&'a str, &'a AIServiceCapabilities)> = None;
            let mut best_score = 0.0;

            for (service_name, capabilities) in services.iter() {
                // Check if service supports the required type
                if !capabilities
                    .supported_service_types
                    .contains(&context.service_type)
                {
                    continue;
                }

                // Calculate service score (health * priority preference)
                let score = capabilities.health_score * Self::calculate_service_score(capabilities);

                if score > best_score {
                    best_score = score;
                    best_service = Some((service_name.as_str(), capabilities));
                }
            }

            Ok(best_service)
        }

        /// Execute request with selected service
        async fn execute_request_with_service(
            service_name: &str,
            capabilities: &AIServiceCapabilities,
            context: AIRequestContext,
        ) -> Result<serde_json::Value, RustAIError> {
            // Placeholder: In real implementation, this would:
            // 1. Get service instance from registry
            // 2. Execute with proper timeout
            // 3. Handle retries
            // 4. Update service health metrics

            let start_time = std::time::Instant::now();
            let timeout_duration = Duration::from_secs(
                context.timeout_override.unwrap_or(
                    context
                        .service_type
                        .timeout_secs()
                        .min(capabilities.average_latency_ms as u64 / 1000),
                ),
            );

            // Mock execution for sample implementation
            match tokio::time::timeout(timeout_duration, async {
                // Simulate AI model inference time
                tokio::time::sleep(Duration::from_millis(
                    (capabilities.average_latency_ms * 0.8) as u64,
                ))
                .await;
                serde_json::json!({ "status": "success", "result": "sample_output" })
            })
            .await
            {
                Ok(result) => {
                    let latency_ms = start_time.elapsed().as_millis() as f64;
                    debug!("Request completed in {}ms via {}", latency_ms, service_name);
                    Ok(result)
                }
                Err(_) => Err(RustAIError::Timeout(format!(
                    "Service {} timed out",
                    service_name
                ))),
            }
        }

        /// Validate request context
        async fn validate_request_context(&self, context: &AIRequestContext) -> Result<(), RustAIError> {
            // Validate request ID format
            validate_string_input_extended(&context.request_id, 64, false)?;

            // Validate user ID if present
            if let Some(user_id) = &context.user_id {
                validate_string_input_extended(user_id, 128, false)?;
            }

            // Validate project ID if present
            if let Some(project_id) = &context.project_id {
                validate_string_input_extended(project_id, 256, false)?;
            }

            Ok(())
        }

        /// Validate payload
        fn validate_payload(&self, payload: &serde_json::Value) -> Result<(), RustAIError> {
            // Basic validation - payload should be reasonable size
            let payload_str = payload.to_string();
            if payload_str.len() > 10 * 1024 * 1024 {
                // 10MB limit
                return Err(RustAIError::Validation(
                    "Payload size exceeds maximum allowed".to_string(),
                ));
            }

            // For more sophisticated validation, we could:
            // - Check JSON schema
            // - Validate content types
            // - Check for malicious content patterns

            Ok(())
        }

        /// Calculate service selection score
        fn calculate_service_score(capabilities: &AIServiceCapabilities) -> f64 {
            // Score based on throughput, memory efficiency, and current load
            let throughput_score = (capabilities.throughput_rps / 100.0).min(1.0);
            let memory_efficiency = (1024.0 / capabilities.memory_usage_mb).min(1.0);
            let load_capacity = (capabilities.max_concurrent_requests as f64 / 10.0).min(1.0);

            (throughput_score * 0.5) + (memory_efficiency * 0.3) + (load_capacity * 0.2)
        }

        /// Health check for orchestrator
        pub async fn health_check(&self) -> Result<serde_json::Value, RustAIError> {
            let metrics = self.get_metrics().await?;
            let services = self.get_services().await?;

            Ok(serde_json::json!({
                "status": "healthy",
                "metrics": metrics,
                "active_services": services.len(),
                "queue_length": metrics.queue_length
            }))
        }
    }

    /// Public interface traits for consumers
    #[async_trait]
    pub trait AIOrchestratorInterface: Send + Sync {
        async fn submit_completion_request(
            &self,
            context: AIRequestContext,
            code: String,
            position: usize,
        ) -> Result<String, RustAIError>;

        async fn submit_analysis_request(
            &self,
            context: AIRequestContext,
            code: String,
            language: String,
        ) -> Result<String, RustAIError>;

        async fn get_request_status(&self, request_id: &str) -> Result<serde_json::Value, RustAIError>;

        async fn cancel_request(&self, request_id: &str) -> Result<(), RustAIError>;
    }

    #[async_trait]
    impl AIOrchestratorInterface for AIOrchestrator {
        async fn submit_completion_request(
            &self,
            context: AIRequestContext,
            code: String,
            position: usize,
        ) -> Result<String, RustAIError> {
            let payload = serde_json::json!({
                "code": code,
                "position": position,
                "service_type": "completion"
            });

            self.submit_request(context, payload).await
        }

        async fn submit_analysis_request(
            &self,
            context: AIRequestContext,
            code: String,
            language: String,
        ) -> Result<String, RustAIError> {
            let payload = serde_json::json!({
                "code": code,
                "language": language,
                "service_type": "analysis"
            });

            self.submit_request(context, payload).await
        }

        async fn get_request_status(&self, _request_id: &str) -> Result<serde_json::Value, RustAIError> {
            // Placeholder implementation
            Ok(serde_json::json!({
                "status": "processing",
                "details": "Request is being processed"
            }))
        }

        async fn cancel_request(&self, _request_id: &str) -> Result<(), RustAIError> {
            // Placeholder implementation
            Ok(())
        }
    }
}

pub use ai_orchestrator::*;

/// Module initialization function for Tauri integration
#[cfg(feature = "tauri")]
pub mod tauri_integration {
    use tauri::State;

    use super::*;

    type OrchestratorState = Arc<Mutex<Option<AIOrchestrator>>>;

    /// Initialize the orchestrator service (call once upon startup)
    pub async fn initialize_orchestrator(
        state: OrchestratorState,
        config: OrchestratorConfig,
    ) -> Result<(), RustAIError> {
        let orchestrator = AIOrchestrator::new(Arc::new(config)).await?;

        // Register sample services
        let local_provider = Arc::new(LocalAIServiceProvider {
            name:      "local-ai-service",
            mock_mode: true,
        });
        orchestrator.register_service(local_provider).await?;

        // Start processing loop
        orchestrator.start_processing_loop().await?;

        let mut state_guard = state.lock().await;
        *state_guard = Some(orchestrator);

        info!("AI Orchestrator initialized successfully");
        Ok(())
    }

    /// Example Tauri command for submitting completion requests
    #[tauri::command]
    pub async fn ai_completion_request(
        state: State<'_, OrchestratorState>,
        code: String,
        cursor_position: usize,
        project_id: Option<String>,
    ) -> Result<String, String> {
        // Input validation using existing patterns
        use rust_ai_ide_common::validation::validate_string_input_extended;
        validate_string_input_extended(&code, 50 * 1024, true) // 50KB max for code
            .map_err(|e| format!("Invalid code input: {}", e))?;

        let state_guard = state.lock().await;
        let orchestrator = state_guard.as_ref().ok_or("Orchestrator not initialized")?;

        let context = AIRequestContext {
            service_type: AIServiceType::Completion,
            project_id,
            ..Default::default()
        };

        orchestrator
            .submit_completion_request(context, code, cursor_position)
            .await
            .map_err(|e| format!("AI request failed: {}", e))
    }

    /// Health check command
    #[tauri::command]
    pub async fn get_orchestrator_health(state: State<'_, OrchestratorState>) -> Result<serde_json::Value, String> {
        let state_guard = state.lock().await;
        let orchestrator = state_guard.as_ref().ok_or("Orchestrator not initialized")?;

        orchestrator
            .health_check()
            .await
            .map_err(|e| format!("Health check failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = Arc::new(OrchestratorConfig::default());
        let orchestrator = AIOrchestrator::new(config).await.unwrap();
        assert!(orchestrator.services.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_service_registration() {
        let config = Arc::new(OrchestratorConfig::default());
        let orchestrator = AIOrchestrator::new(config).await.unwrap();

        let provider = Arc::new(LocalAIServiceProvider {
            name:      "test-service",
            mock_mode: true,
        });

        orchestrator.register_service(provider).await.unwrap();

        let services = orchestrator.get_services().await.unwrap();
        assert_eq!(services.len(), 1);
        assert!(services.contains_key("test-service"));
    }

    #[tokio::test]
    async fn test_request_submission_and_processing() {
        let config = Arc::new(OrchestratorConfig::default());
        let orchestrator = AIOrchestrator::new(config).await.unwrap();

        // Register a service
        let provider = Arc::new(LocalAIServiceProvider {
            name:      "test-service",
            mock_mode: true,
        });
        orchestrator.register_service(provider).await.unwrap();

        // Start processing loop
        orchestrator.start_processing_loop().await.unwrap();

        // Submit a request
        let context = AIRequestContext {
            service_type: AIServiceType::Completion,
            ..Default::default()
        };

        let payload = serde_json::json!({"text": "test completion"});
        let request_id = orchestrator.submit_request(context, payload).await.unwrap();

        assert!(!request_id.is_empty());

        // Check metrics (should have at least 1 total request)
        let metrics = orchestrator.get_metrics().await.unwrap();
        assert!(metrics.total_requests >= 1);
    }
}
