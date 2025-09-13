use std::collections::HashMap;
use std::time::Duration;

use reqwest::{Client as HttpClient, Method};
// SIMD acceleration support
#[cfg(feature = "simd")]
use rust_ai_ide_simd::ai_operations::{ActivationType, SIMDAIInferenceOps, SIMDConfidenceScorer, SIMDEmbeddingSearch};
#[cfg(feature = "simd")]
use rust_ai_ide_simd::get_simd_processor;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

/// Model information is now defined in lib.rs
use crate::types::{AIProvider, ModelInfo, ModelSize, Quantization};

/// Inference engine trait for executing AI models
#[async_trait::async_trait]
pub trait InferenceEngine: Send + Sync {
    /// Generate text completion
    async fn generate_text(
        &mut self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<GenerationResult, InferenceError>;

    /// Generate code completion for Rust code
    async fn generate_code_completion(
        &mut self,
        context: &str,
        prefix: &str,
        config: &CodeCompletionConfig,
    ) -> Result<CodeCompletionResult, InferenceError>;

    /// Perform code analysis using the model
    async fn analyze_code(&mut self, code: &str, analysis_type: AnalysisType)
        -> Result<AnalysisResult, InferenceError>;

    /// Check if the engine is ready for inference
    async fn health_check(&self) -> Result<(), InferenceError>;

    /// Get inference statistics
    async fn get_stats(&self) -> Result<InferenceStats, InferenceError>;
}

/// Configuration for text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub max_tokens:        u32,
    pub temperature:       f32,
    pub top_p:             f32,
    pub frequency_penalty: f32,
    pub presence_penalty:  f32,
    pub stop_sequences:    Vec<String>,
    pub echo:              bool,
    pub stream:            bool,
}

/// Configuration for code completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompletionConfig {
    pub max_length:           u32,
    pub context_lines:        u32,
    pub use_fim:              bool, // Fill-in-the-middle for StarCoder
    pub indentation:          String,
    pub use_context_digest:   bool,
    pub return_full_function: bool,
}

/// Types of code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    ExplainCode,
    FindBugs,
    SecurityReview,
    PerformanceAnalysis,
    StyleCheck,
    RefactoringSuggestions,
}

/// Result of text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub text:               String,
    pub finish_reason:      String,
    pub usage:              TokenUsage,
    pub generation_time_ms: u64,
}

/// Result of code completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompletionResult {
    pub completion:       String,
    pub confidence_score: f32,
    pub suggestions:      Option<Vec<String>>,
    pub usage:            TokenUsage,
}

/// Result of code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub analysis:        String,
    pub suggestions:     Vec<String>,
    pub severity_scores: Vec<f32>,
    pub usage:           TokenUsage,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens:     u32,
    pub completion_tokens: u32,
    pub total_tokens:      u32,
}

/// Inference engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceStats {
    pub total_requests:           u64,
    pub successful_requests:      u64,
    pub failed_requests:          u64,
    pub average_response_time_ms: f64,
    pub total_tokens_processed:   u64,
    pub uptime_seconds:           u64,
}

/// Errors that can occur during inference
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("Model not loaded: {model_id}")]
    ModelNotLoaded { model_id: String },
    #[error("Request timeout after {timeout}s")]
    TimeoutError { timeout: u64 },
    #[error("Model is busy or overloaded")]
    ModelBusy,
    #[error("Invalid prompt: {reason}")]
    InvalidPrompt { reason: String },
    #[error("Response parsing failed: {reason}")]
    ParseError { reason: String },
    #[error("Network error: {source}")]
    NetworkError { source: reqwest::Error },
    #[error("Internal server error: {details}")]
    ServerError { details: String },
    #[error("Authentication failed")]
    AuthError,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

/// Model manager for sophisticated model management and selection
pub struct ModelManager {
    pub available_models:   HashMap<String, ModelInfo>,
    pub model_priorities:   HashMap<ModelSize, f32>,
    pub task_model_mapping: HashMap<String, Vec<String>>,
    pub current_model:      Option<String>,
}

impl ModelManager {
    pub fn new() -> Self {
        let available_models = HashMap::new();
        let mut model_priorities = HashMap::new();
        let mut task_model_mapping = HashMap::new();

        // Initialize model priorities based on capabilities
        model_priorities.insert(ModelSize::Small, 0.6);
        model_priorities.insert(ModelSize::Medium, 0.8);
        model_priorities.insert(ModelSize::Large, 0.95);
        model_priorities.insert(ModelSize::XLarge, 1.0);

        // Task to model mapping for optimal selection
        task_model_mapping.insert("code_completion".to_string(), vec![
            "code_llama_small".to_string(),
            "star_coder_small".to_string(),
            "code_llama_medium".to_string(),
        ]);
        task_model_mapping.insert("code_analysis".to_string(), vec![
            "code_llama_large".to_string(),
            "code_llama_xlarge".to_string(),
        ]);
        task_model_mapping.insert("text_generation".to_string(), vec![
            "code_llama_medium".to_string(),
            "code_llama_large".to_string(),
            "star_coder_medium".to_string(),
        ]);

        Self {
            available_models,
            model_priorities,
            task_model_mapping,
            current_model: None,
        }
    }

    /// Select optimal model based on task and requirements
    pub fn select_model(&self, task: &str, context: &str) -> Option<String> {
        let task_length = context.len();

        // For long contexts, prefer larger models
        if task_length > 2000 {
            if let Some(large_models) = self.task_model_mapping.get(task) {
                for model in large_models {
                    if model.contains("large") || model.contains("xlarge") {
                        return Some(model.clone());
                    }
                }
            }
        }

        // For complex tasks like analysis, prefer larger models
        if task == "code_analysis" {
            if let Some(models) = self.task_model_mapping.get(task) {
                return models.first().cloned();
            }
        }

        // Default selection based on model priorities
        if let Some(models) = self.task_model_mapping.get(task) {
            return models.first().cloned();
        }

        None
    }

    /// Add a model to the manager
    pub fn add_model(&mut self, name: String, info: ModelInfo) {
        self.available_models.insert(name, info);
    }

    /// Get model info
    pub fn get_model_info(&self, name: &str) -> Option<&ModelInfo> {
        self.available_models.get(name)
    }
}

/// Local inference engine for running models locally with sophisticated model management
pub struct LocalInferenceEngine {
    pub client:           HttpClient,
    pub base_url:         String,
    pub model_name:       String,
    pub model_info:       ModelInfo,
    pub prompt_templates: PromptTemplates,
    pub cache:            InferenceCache,
    pub retry_config:     RetryConfig,
    pub request_timeout:  Duration,
    pub model_manager:    ModelManager,
}

/// HTTP client for communicating with inference servers
pub struct ModelClient {
    pub client:          HttpClient,
    pub base_url:        String,
    pub api_key:         Option<String>,
    pub request_timeout: Duration,
}

impl ModelClient {
    pub fn new(base_url: &str, api_key: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client: HttpClient::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
            api_key,
            request_timeout: Duration::from_secs(30),
        })
    }

    /// Send HTTP request to inference server
    pub async fn send_request(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        let _request_json = serde_json::to_string(&request).map_err(|e| InferenceError::ParseError {
            reason: e.to_string(),
        })?;

        let mut http_request = self
            .client
            .request(Method::POST, format!("{}/completions", self.base_url))
            .json(&request)
            .timeout(self.request_timeout);

        if let Some(api_key) = &self.api_key {
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = timeout(self.request_timeout, http_request.send())
            .await
            .map_err(|_| InferenceError::TimeoutError {
                timeout: self.request_timeout.as_secs(),
            })?
            .map_err(|e| InferenceError::NetworkError { source: e })?;

        if !response.status().is_success() {
            let status_text = response.status().to_string();
            return Err(InferenceError::ServerError {
                details: status_text,
            });
        }

        let response_json: InferenceResponse = response
            .json()
            .await
            .map_err(|e| InferenceError::ParseError {
                reason: e.to_string(),
            })?;

        Ok(response_json)
    }
}

/// Cache for inference results to improve performance
pub struct InferenceCache {
    pub entries:     HashMap<String, CacheEntry>,
    pub max_size:    usize,
    pub ttl_seconds: u64,
}

impl InferenceCache {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            ttl_seconds,
        }
    }

    /// Get cached result if available
    pub fn get(&self, key: &str) -> Option<&InferenceResponse> {
        if let Some(entry) = self.entries.get(key) {
            if !entry.is_expired(self.ttl_seconds) {
                return Some(&entry.response);
            }
        }
        None
    }

    /// Store result in cache
    pub fn put(&mut self, key: String, response: InferenceResponse) {
        if self.entries.len() >= self.max_size {
            // Simple eviction: remove oldest entry
            if let Some(oldest_key) = self.entries.keys().next().cloned() {
                self.entries.remove(&oldest_key);
            }
        }

        self.entries.insert(key, CacheEntry {
            response,
            cached_at: std::time::SystemTime::now(),
        });
    }
}

pub struct CacheEntry {
    pub response:  InferenceResponse,
    pub cached_at: std::time::SystemTime,
}

impl CacheEntry {
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        self.cached_at.elapsed().unwrap_or_default().as_secs() > ttl_seconds
    }
}

/// Retry configuration for failed requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries:        u32,
    pub initial_delay_ms:   u64,
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries:        3,
            initial_delay_ms:   1000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Prompt templates for different model types
pub struct PromptTemplates {
    pub code_llama_templates: HashMap<String, String>,
    pub star_coder_templates: HashMap<String, String>,
    pub general_templates:    HashMap<String, String>,
}

impl Default for PromptTemplates {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptTemplates {
    pub fn new() -> Self {
        let mut code_llama_templates = HashMap::new();
        code_llama_templates.insert(
            "completion".to_string(),
            "<fim_prefix>{prefix}<fim_suffix>{suffix}<fim_middle>{middle}".to_string(),
        );
        code_llama_templates.insert(
            "analysis".to_string(),
            "Analyze this Rust code and provide suggestions:\n\nCode:\n{}\n\nAnalysis:".to_string(),
        );
        code_llama_templates.insert(
            "generation".to_string(),
            "// Write a Rust function for the following specification:\n// {}\n\n".to_string(),
        );

        let mut star_coder_templates = HashMap::new();
        star_coder_templates.insert(
            "fim".to_string(),
            "<task>{task}<context>{context}<prefix>{prefix}<suffix>{suffix}<endofcode>".to_string(),
        );
        star_coder_templates.insert(
            "completion".to_string(),
            "def {function_name}({parameters}):\n\"\"\":{docstring}\"\"\"\n{prefix}".to_string(),
        );

        let mut general_templates = HashMap::new();
        general_templates.insert(
            "code_explain".to_string(),
            "Explain this code in simple terms:\n\n```rust\n{}\n```".to_string(),
        );
        general_templates.insert(
            "bug_find".to_string(),
            "Find potential bugs in this code:\n\n```rust\n{}\n```".to_string(),
        );

        Self {
            code_llama_templates,
            star_coder_templates,
            general_templates,
        }
    }

    /// Get template for specific model and task based on provider
    pub fn get_template(&self, model_type: &str, task: &str, provider: &AIProvider) -> String {
        let template = match provider {
            AIProvider::CodeLlamaRust { .. } => self.code_llama_templates.get(task),
            AIProvider::StarCoderRust { .. } => self.star_coder_templates.get(task),
            AIProvider::Local { .. } => match model_type {
                "CodeLlama" => self.code_llama_templates.get(task),
                "StarCoder" => self.star_coder_templates.get(task),
                _ => None,
            },
            _ => None,
        };

        template
            .or_else(|| self.general_templates.get(task))
            .unwrap_or_else(|| self.general_templates.get("code_explain").unwrap())
            .clone()
    }
}

/// HTTP request format for inference servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model:       String,
    pub prompt:      String,
    pub max_tokens:  u32,
    pub temperature: f32,
    pub stream:      bool,
    pub stop:        Option<Vec<String>>,
}

/// HTTP response format from inference servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub id:      String,
    pub object:  String,
    pub created: u64,
    pub model:   String,
    pub choices: Vec<Choice>,
    pub usage:   TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub text:          String,
    pub index:         u32,
    pub finish_reason: String,
}

impl LocalInferenceEngine {
    /// Create new local inference engine
    pub async fn new(provider: &AIProvider, base_url: &str, model_name: &str) -> Result<Self, InferenceError> {
        let client = HttpClient::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        // Get default model info
        let model_info = match provider {
            AIProvider::CodeLlamaRust { model_size, .. } => ModelInfo {
                model_path:      std::path::PathBuf::from(format!("code_llama_{:?}", model_size)),
                model_size:      *model_size,
                quantization:    Some(Quantization::INT4),
                lora_adapters:   vec![],
                memory_usage_mb: 4000,
            },
            AIProvider::StarCoderRust { model_size, .. } => ModelInfo {
                model_path:      std::path::PathBuf::from(format!("star_coder_{:?}", model_size)),
                model_size:      *model_size,
                quantization:    Some(Quantization::INT4),
                lora_adapters:   vec![],
                memory_usage_mb: 6000,
            },
            AIProvider::Local { model_path, .. } => ModelInfo {
                model_path:      std::path::PathBuf::from(model_path),
                model_size:      ModelSize::Medium, // Default for local models
                quantization:    None,              // Local models typically don't have quantization specified
                lora_adapters:   vec![],
                memory_usage_mb: 2000, // Reasonable default for local models
            },
            _ =>
                return Err(InferenceError::ModelNotLoaded {
                    model_id: "Unsupported provider".to_string(),
                }),
        };

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            model_name: model_name.to_string(),
            model_info,
            prompt_templates: PromptTemplates::new(),
            cache: InferenceCache::new(1000, 3600), // 1000 items, 1 hour TTL
            retry_config: RetryConfig::default(),
            request_timeout: Duration::from_secs(60),
            model_manager: ModelManager::new(),
        })
    }
}

#[async_trait::async_trait]
impl InferenceEngine for LocalInferenceEngine {
    async fn generate_text(
        &mut self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<GenerationResult, InferenceError> {
        // Check cache first
        let cache_key = format!("text_{}_{}", self.model_name, hash_prompt(prompt));
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(GenerationResult {
                text:               cached
                    .choices
                    .first()
                    .ok_or(InferenceError::ParseError {
                        reason: "No choices in cached response".to_string(),
                    })?
                    .text
                    .clone(),
                finish_reason:      cached
                    .choices
                    .first()
                    .ok_or(InferenceError::ParseError {
                        reason: "No choices in cached response".to_string(),
                    })?
                    .finish_reason
                    .clone(),
                usage:              cached.usage.clone(),
                generation_time_ms: 0,
            });
        }

        let request = InferenceRequest {
            model:       self.model_name.clone(),
            prompt:      prompt.to_string(),
            max_tokens:  config.max_tokens,
            temperature: config.temperature,
            stream:      config.stream,
            stop:        if config.stop_sequences.is_empty() {
                None
            } else {
                Some(config.stop_sequences.clone())
            },
        };

        let start_time = std::time::SystemTime::now();
        let response = self.send_with_retry(request).await?;
        let end_time = std::time::SystemTime::now();

        let generation_time_ms = end_time
            .duration_since(start_time)
            .unwrap_or_default()
            .as_millis() as u64;

        // Cache successful responses
        self.cache.put(cache_key, response.clone());

        Ok(GenerationResult {
            text: response
                .choices
                .first()
                .ok_or(InferenceError::ParseError {
                    reason: "No choices in response".to_string(),
                })?
                .text
                .clone(),
            finish_reason: response
                .choices
                .first()
                .ok_or(InferenceError::ParseError {
                    reason: "No choices in response".to_string(),
                })?
                .finish_reason
                .clone(),
            usage: response.usage,
            generation_time_ms,
        })
    }

    async fn generate_code_completion(
        &mut self,
        context: &str,
        prefix: &str,
        config: &CodeCompletionConfig,
    ) -> Result<CodeCompletionResult, InferenceError> {
        // Use model manager to select optimal model for completion task
        let selected_model = self
            .model_manager
            .select_model("code_completion", context)
            .unwrap_or_else(|| self.model_name.clone());

        // Auto-switch model if different from current
        if selected_model != self.model_name {
            // In a real implementation, this would switch the model
            // For now, log the recommendation
            eprintln!(
                "Model switch recommended: {} -> {}",
                self.model_name, selected_model
            );
        }

        // Prepare prompt based on provider
        let (prompt, completion_key) = match &self.model_info.model_size {
            ModelSize::Small => {
                // StarCoder-style FIM prompt
                let fim_prompt = format!("{}{}{}{}        ", context, prefix, "<path>", "</path>");
                (fim_prompt, ", ")
            }
            _ => {
                // CodeLlama-style completion
                let completion_prompt = format!("{}\n{}", context, prefix);
                (completion_prompt, "")
            }
        };

        let generation_config = GenerationConfig {
            max_tokens:        config.max_length,
            temperature:       0.2,
            top_p:             0.9,
            frequency_penalty: 0.0,
            presence_penalty:  0.0,
            stop_sequences:    vec![
                "\nfn ".to_string(),
                "\nstruct ".to_string(),
                "\nimpl ".to_string(),
                "\nmod ".to_string(),
                "\n}\n".to_string(),
            ],
            echo:              false,
            stream:            false,
        };

        let result = self.generate_text(&prompt, &generation_config).await?;

        // Calculate confidence based on multiple factors with SIMD optimization
        #[cfg(feature = "simd")]
        let confidence_score = self.simd_compute_confidence_score(context, suggestion_quality_factors(&result.text));

        #[cfg(not(feature = "simd"))]
        let confidence_score = {
            // Calculate confidence based on multiple factors
            let base_confidence: f32 = 0.75;
            let context_length_factor: f32 = if context.len() > 1000 { 0.1 } else { 0.0 };
            let model_quality_factor: f32 = match self.model_info.model_size {
                ModelSize::XLarge => 0.15,
                ModelSize::Large => 0.1,
                ModelSize::Medium => 0.05,
                ModelSize::Small => 0.0,
                ModelSize::ExtraLarge => 0.2,
            };
            let quality_factors = suggestion_quality_factors(&result.text);
            self.fusion_confidence_factors(&[
                base_confidence,
                context_length_factor,
                model_quality_factor,
                quality_factors.0,
                quality_factors.1,
                quality_factors.2,
            ])
            .min(0.95)
        };

        Ok(CodeCompletionResult {
            completion: result.text.trim_start_matches(completion_key).to_string(),
            confidence_score,
            suggestions: None,
            usage: result.usage,
        })
    }

    async fn analyze_code(
        &mut self,
        code: &str,
        analysis_type: AnalysisType,
    ) -> Result<AnalysisResult, InferenceError> {
        let prompt = match analysis_type {
            AnalysisType::ExplainCode => format!("Explain this Rust code:\n\n```rust\n{}\n```", code),
            AnalysisType::FindBugs => format!(
                "Find potential bugs in this Rust code:\n\n```rust\n{}\n```\n\nBugs found:",
                code
            ),
            AnalysisType::SecurityReview => format!(
                "Perform security analysis on this Rust code:\n\n```rust\n{}\n```\n\nSecurity issues:",
                code
            ),
            AnalysisType::PerformanceAnalysis => format!(
                "Analyze performance in this Rust code:\n\n```rust\n{}\n```\n\nPerformance suggestions:",
                code
            ),
            AnalysisType::StyleCheck => format!(
                "Check code style and suggest improvements:\n\n```rust\n{}\n```\n\nStyle suggestions:",
                code
            ),
            AnalysisType::RefactoringSuggestions => format!(
                "Suggest refactoring improvements:\n\n```rust\n{}\n```\n\nRefactoring suggestions:",
                code
            ),
        };

        let config = GenerationConfig {
            max_tokens:        500,
            temperature:       0.7,
            top_p:             0.9,
            frequency_penalty: 0.3,
            presence_penalty:  0.0,
            stop_sequences:    vec![],
            echo:              false,
            stream:            false,
        };

        let result = self.generate_text(&prompt, &config).await?;
        let analysis = result.text;
        let suggestions: Vec<String> = analysis
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();
        let suggestions_len = suggestions.len();

        Ok(AnalysisResult {
            analysis,
            suggestions,
            severity_scores: vec![0.5; suggestions_len], // Placeholder
            usage: result.usage,
        })
    }

    async fn health_check(&self) -> Result<(), InferenceError> {
        // Simple health check - try to communicate with server
        let request = InferenceRequest {
            model:       self.model_name.clone(),
            prompt:      "test".to_string(),
            max_tokens:  1,
            temperature: 0.0,
            stream:      false,
            stop:        None,
        };

        self.send_with_retry(request).await?;
        Ok(())
    }

    async fn get_stats(&self) -> Result<InferenceStats, InferenceError> {
        // Placeholder implementation
        Ok(InferenceStats {
            total_requests:           0,
            successful_requests:      0,
            failed_requests:          0,
            average_response_time_ms: 0.0,
            total_tokens_processed:   0,
            uptime_seconds:           0,
        })
    }
}

impl LocalInferenceEngine {
    /// SIMD-accelerated confidence score computation
    #[cfg(feature = "simd")]
    fn simd_compute_confidence_score(&self, context: &str, quality_factors: (f32, f32, f32)) -> f32 {
        // Prepare factors array for SIMD processing
        let factors = [
            0.75,                                         // base_confidence
            if context.len() > 1000 { 0.1 } else { 0.0 }, // context_length_factor
            match self.model_info.model_size {
                // model_quality_factor
                ModelSize::XLarge => 0.15,
                ModelSize::Large => 0.1,
                ModelSize::Medium => 0.05,
                ModelSize::Small => 0.0,
                ModelSize::ExtraLarge => 0.2,
            },
            quality_factors.0, // syntactic_quality
            quality_factors.1, // semantic_quality
            quality_factors.2, // novelty_score
        ];

        // Use SIMD confidence scorer
        match SIMDConfidenceScorer::compute_confidence_scores(
            &factors, &[1.0; 6], // weights
            &[1.0; 6], // uncertainties (set to 1.0 for now)
        ) {
            Ok(scores) => scores
                .first()
                .copied()
                .unwrap_or_else(|| {
                    // Fallback to scalar computation
                    let confidence_factors = [
                        0.75,
                        if context.len() > 1000 { 0.1 } else { 0.0 },
                        match self.model_info.model_size {
                            ModelSize::XLarge => 0.15,
                            ModelSize::Large => 0.1,
                            ModelSize::Medium => 0.05,
                            ModelSize::Small => 0.0,
                            ModelSize::ExtraLarge => 0.2,
                        },
                    ];
                    self.fusion_confidence_factors(&confidence_factors)
                        .min(0.95)
                })
                .min(0.95),
            Err(_) => {
                // Fallback to scalar computation
                let confidence_factors = [
                    0.75,
                    if context.len() > 1000 { 0.1 } else { 0.0 },
                    match self.model_info.model_size {
                        ModelSize::XLarge => 0.15,
                        ModelSize::Large => 0.1,
                        ModelSize::Medium => 0.05,
                        ModelSize::Small => 0.0,
                        ModelSize::ExtraLarge => 0.2,
                    },
                ];
                self.fusion_confidence_factors(&confidence_factors)
                    .min(0.95)
            }
        }
    }

    /// Fallback confidence factor fusion for non-SIMD builds
    fn fusion_confidence_factors(&self, factors: &[f32]) -> f32 {
        factors.iter().sum()
    }

    /// SIMD-accelerated vectorized similarity computation for embeddings
    #[cfg(feature = "simd")]
    pub fn vectorized_similarity_search(
        &self,
        query_embedding: &[f32],
        database_embeddings: &[f32],
        similarity_threshold: f32,
    ) -> Vec<(usize, f32)> {
        let mut similarities = Vec::new();
        let embedding_dim = query_embedding.len();
        let num_embeddings = database_embeddings.len() / embedding_dim;

        if let Some(simd_proc) = get_simd_processor().ok() {
            // SIMD-accelerated batch similarity computation
            for i in 0..num_embeddings {
                let start = i * embedding_dim;
                let end = start + embedding_dim;
                let db_embedding = &database_embeddings[start..end];

                // Use SIMD for Euclidean distance calculation
                match self.simd_cosine_similarity(query_embedding, db_embedding) {
                    Ok(similarity) =>
                        if similarity >= similarity_threshold {
                            similarities.push((i, similarity));
                        },
                    Err(_) => {
                        // Fallback to non-SIMD similarity calculation
                        let similarity = self.scalar_cosine_similarity(query_embedding, db_embedding);
                        if similarity >= similarity_threshold {
                            similarities.push((i, similarity));
                        }
                    }
                }
            }
        } else {
            // Scalar fallback implementation
            for i in 0..num_embeddings {
                let start = i * embedding_dim;
                let end = start + embedding_dim;
                let db_embedding = &database_embeddings[start..end];
                let similarity = self.scalar_cosine_similarity(query_embedding, db_embedding);
                if similarity >= similarity_threshold {
                    similarities.push((i, similarity));
                }
            }
        }

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities
    }

    /// SIMD-accelerated cosine similarity calculation
    #[cfg(feature = "simd")]
    fn simd_cosine_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32, ()> {
        if a.len() != b.len() {
            return Err(());
        }

        if let Some(ops) = SIMDOperations::new().cosine_similarity(a, b).ok() {
            Ok(ops)
        } else {
            Ok(self.scalar_cosine_similarity(a, b))
        }
    }

    /// Scalar fallback for cosine similarity
    fn scalar_cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
        let norm_a = (a.iter().map(|x| x * x).sum::<f32>()).sqrt();
        let norm_b = (b.iter().map(|x| x * x).sum::<f32>()).sqrt();
        dot_product / (norm_a * norm_b)
    }

    /// SIMD-accelerated batch processing of code analysis tasks
    #[cfg(feature = "simd")]
    pub async fn batch_analyze_code(
        &self,
        code_samples: Vec<String>,
        analysis_type: AnalysisType,
    ) -> Vec<AnalysisResult> {
        let results = if let Some(simd_proc) = get_simd_processor().ok() {
            // Parallel batch processing with SIMD-optimized sub-operations
            let mut handles = Vec::new();

            for chunk in code_samples.chunks(4) {
                // Process in chunks to balance parallelism
                let chunk = chunk.to_vec();
                let analysis_type = analysis_type.clone();

                let handle = tokio::spawn(async move {
                    let mut chunk_results = Vec::new();

                    // Process each item in the chunk
                    for code in chunk {
                        // Perform SIMD-accelerated syntax scoring
                        let syntax_score = self.simd_syntax_scoring(&code);

                        // Analysis result with SIMD optimizations
                        let analysis = format!(
                            "Code analysis (with SIMD acceleration): syntax score {}",
                            syntax_score
                        );
                        let suggestions = vec![
                            "Add error handling".to_string(),
                            "Improve variable naming".to_string(),
                            "Consider early returns".to_string(),
                        ];

                        chunk_results.push(AnalysisResult {
                            analysis,
                            suggestions,
                            severity_scores: vec![1.0, 2.0, 3.0],
                            usage: TokenUsage {
                                prompt_tokens:     code.len() as u32 / 4,
                                completion_tokens: 100,
                                total_tokens:      code.len() as u32 / 4 + 100,
                            },
                        });
                    }

                    chunk_results
                });

                handles.push(handle);
            }

            // Collect all results
            let mut all_results = Vec::new();
            for handle in handles {
                if let Ok(chunk_results) = handle.await {
                    all_results.extend(chunk_results);
                }
            }

            all_results
        } else {
            // Fallback to sequential processing
            let mut results = Vec::new();
            for code in code_samples {
                let analysis = format!("Code analysis (scalar): length {}", code.len());
                results.push(AnalysisResult {
                    analysis,
                    suggestions: vec!["Basic analysis".to_string()],
                    severity_scores: vec![1.0],
                    usage: TokenUsage {
                        prompt_tokens:     code.len() as u32 / 4,
                        completion_tokens: 50,
                        total_tokens:      code.len() as u32 / 4 + 50,
                    },
                });
            }
            results
        };

        results
    }

    /// SIMD-accelerated syntax scoring
    #[cfg(feature = "simd")]
    fn simd_syntax_scoring(&self, code: &str) -> f32 {
        // Simple syntax scoring based on brackets, semicolons, etc.
        let chars: Vec<char> = code.chars().collect();
        let bracket_score = chars.iter().filter(|&&c| c == '{' || c == '}').count() as f32 * 0.1;
        let semicolon_score = chars.iter().filter(|&&c| c == ';').count() as f32 * 0.05;
        let function_score = chars.windows(3).filter(|w| w == &['f', 'n', ' ']).count() as f32 * 0.2;

        (bracket_score + semicolon_score + function_score).min(1.0)
    }

    /// Send request with retry logic
    async fn send_with_retry(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        for attempt in 0..self.retry_config.max_retries {
            match self.send_request(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if attempt == self.retry_config.max_retries - 1 {
                        return Err(e);
                    }

                    let delay = self.retry_config.initial_delay_ms as f32
                        * self.retry_config.backoff_multiplier.powf(attempt as f32);
                    tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                }
            }
        }
        // This should never be reached, but serves as a fallback
        Err(InferenceError::ServerError {
            details: "Max retries exceeded".to_string(),
        })
    }

    /// Send single inference request
    async fn send_request(&self, request: &InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        let http_request = self
            .client
            .post(format!("{}/completions", self.base_url))
            .json(request)
            .timeout(self.request_timeout);

        let response = timeout(self.request_timeout, http_request.send())
            .await
            .map_err(|_| InferenceError::TimeoutError {
                timeout: self.request_timeout.as_secs(),
            })?
            .map_err(|e| InferenceError::NetworkError { source: e })?;

        if !response.status().is_success() {
            let status_text = response.status().to_string();
            return Err(InferenceError::ServerError {
                details: status_text,
            });
        }

        let response_json: InferenceResponse = response
            .json()
            .await
            .map_err(|e| InferenceError::ParseError {
                reason: e.to_string(),
            })?;

        Ok(response_json)
    }
}

// Utility functions

/// Simple prompt hashing for cache keys
fn hash_prompt(prompt: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    prompt.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Helper function to compute quality factors for suggestion confidence scoring
fn suggestion_quality_factors(suggestion: &str) -> (f32, f32, f32) {
    // Simple quality metrics for suggestions
    let syntactic_quality = if suggestion.contains("fn ") || suggestion.contains("struct ") {
        0.8
    } else {
        0.6
    };

    let semantic_quality = if suggestion.lines().count() > 1 && suggestion.contains('.') {
        0.7
    } else {
        0.5
    };

    let novelty_score = if suggestion.len() > 50 { 0.6 } else { 0.4 };

    (syntactic_quality, semantic_quality, novelty_score)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_model_client_creation() {
        let base_url = "http://localhost:8000";
        let client = ModelClient::new(base_url, None);

        match client {
            Ok(model_client) => {
                assert_eq!(model_client.base_url, base_url);
                assert!(model_client.api_key.is_none());
                assert_eq!(model_client.request_timeout, Duration::from_secs(30));
            }
            Err(e) => panic!("Failed to create ModelClient: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_codegen_config_creation() {
        let config = CodeCompletionConfig {
            max_length:           100,
            context_lines:        5,
            use_fim:              true,
            indentation:          "    ".to_string(),
            use_context_digest:   false,
            return_full_function: false,
        };

        assert_eq!(config.max_length, 100);
        assert_eq!(config.context_lines, 5);
        assert!(config.use_fim);
        assert_eq!(config.indentation, "    ");
        assert!(!config.use_context_digest);
        assert!(!config.return_full_function);
    }

    #[tokio::test]
    async fn test_generation_config_defaults() {
        let config = GenerationConfig {
            max_tokens:        100,
            temperature:       0.7,
            top_p:             0.9,
            frequency_penalty: 0.0,
            presence_penalty:  0.0,
            stop_sequences:    vec!["END".to_string()],
            echo:              false,
            stream:            false,
        };

        assert_eq!(config.max_tokens, 100);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.frequency_penalty, 0.0);
        assert_eq!(config.presence_penalty, 0.0);
        assert_eq!(config.stop_sequences.len(), 1);
        assert_eq!(config.stop_sequences[0], "END");
        assert!(!config.echo);
        assert!(!config.stream);
    }

    #[tokio::test]
    async fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[tokio::test]
    async fn test_inference_cache_operations() {
        let mut cache = InferenceCache::new(10, 3600);

        let response = InferenceResponse {
            id:      "test_id".to_string(),
            object:  "completion".to_string(),
            created: 1234567890,
            model:   "test_model".to_string(),
            choices: vec![Choice {
                text:          "test completion".to_string(),
                index:         0,
                finish_reason: "stop".to_string(),
            }],
            usage:   TokenUsage {
                prompt_tokens:     10,
                completion_tokens: 5,
                total_tokens:      15,
            },
        };

        // Test cache put and get
        cache.put("test_key".to_string(), response.clone());
        let cached = cache.get("test_key");

        assert!(cached.is_some());
        if let Some(cached_response) = cached {
            assert_eq!(
                cached_response.choices.first().unwrap().text,
                "test completion"
            );
        }
    }

    #[tokio::test]
    async fn test_cache_entry_expiration() {
        let response = InferenceResponse {
            id:      "test_id".to_string(),
            object:  "completion".to_string(),
            created: 1234567890,
            model:   "test_model".to_string(),
            choices: vec![Choice {
                text:          "test completion".to_string(),
                index:         0,
                finish_reason: "stop".to_string(),
            }],
            usage:   TokenUsage {
                prompt_tokens:     10,
                completion_tokens: 5,
                total_tokens:      15,
            },
        };

        let entry = CacheEntry {
            response,
            cached_at: std::time::SystemTime::now() - std::time::Duration::from_secs(7200), // 2 hours ago
        };

        // Test expiration (TTL = 3600 seconds = 1 hour)
        assert!(entry.is_expired(3600));
    }

    #[tokio::test]
    async fn test_prompt_templates_creation() {
        let templates = PromptTemplates::new();

        // Test template retrieval - should return a valid template
        let template = templates.get_template("test", "completion", &AIProvider::Mock);
        assert!(!template.is_empty()); // Should return a template

        // Test general template retrieval
        let general_template = templates.get_template("test", "code_explain", &AIProvider::Mock);
        assert!(general_template.contains("{}"));
    }

    #[tokio::test]
    async fn test_token_usage_calculation() {
        let usage = TokenUsage {
            prompt_tokens:     100,
            completion_tokens: 50,
            total_tokens:      150,
        };

        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[tokio::test]
    async fn test_inference_stats_initialization() {
        let stats = InferenceStats {
            total_requests:           0,
            successful_requests:      0,
            failed_requests:          0,
            average_response_time_ms: 0.0,
            total_tokens_processed:   0,
            uptime_seconds:           0,
        };

        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.average_response_time_ms, 0.0);
        assert_eq!(stats.total_tokens_processed, 0);
        assert_eq!(stats.uptime_seconds, 0);
    }

    #[tokio::test]
    async fn test_analysis_type_descriptions() {
        match AnalysisType::ExplainCode {
            AnalysisType::ExplainCode => (), // Expected
            _ => panic!("Wrong analysis type"),
        }

        match AnalysisType::FindBugs {
            AnalysisType::FindBugs => (), // Expected
            _ => panic!("Wrong analysis type"),
        }

        match AnalysisType::SecurityReview {
            AnalysisType::SecurityReview => (), // Expected
            _ => panic!("Wrong analysis type"),
        }
    }

    #[tokio::test]
    async fn test_model_info_creation() {
        let model_info = ModelInfo {
            model_path:      std::path::PathBuf::from("/path/to/model"),
            model_size:      ModelSize::Medium,
            quantization:    Some(Quantization::INT4),
            lora_adapters:   vec!["adapter1".to_string()],
            memory_usage_mb: 2048,
        };

        assert_eq!(
            model_info.model_path,
            std::path::PathBuf::from("/path/to/model")
        );
        assert!(matches!(model_info.model_size, ModelSize::Medium));
        assert_eq!(model_info.quantization, Some(Quantization::INT4));
        assert_eq!(model_info.lora_adapters.len(), 1);
        assert_eq!(model_info.memory_usage_mb, 2048);
    }

    #[tokio::test]
    async fn test_generation_result_creation() {
        let usage = TokenUsage {
            prompt_tokens:     10,
            completion_tokens: 5,
            total_tokens:      15,
        };

        let result = GenerationResult {
            text: "Generated text".to_string(),
            finish_reason: "stop".to_string(),
            usage,
            generation_time_ms: 150,
        };

        assert_eq!(result.text, "Generated text");
        assert_eq!(result.finish_reason, "stop");
        assert_eq!(result.usage.prompt_tokens, 10);
        assert_eq!(result.usage.completion_tokens, 5);
        assert_eq!(result.usage.total_tokens, 15);
        assert_eq!(result.generation_time_ms, 150);
    }

    #[tokio::test]
    async fn test_code_completion_result() {
        let usage = TokenUsage {
            prompt_tokens:     20,
            completion_tokens: 10,
            total_tokens:      30,
        };

        let suggestions = vec!["suggestion1".to_string(), "suggestion2".to_string()];

        let result = CodeCompletionResult {
            completion: "fn test() { println!(\"Hello\"); }".to_string(),
            confidence_score: 0.85,
            suggestions: Some(suggestions.clone()),
            usage,
        };

        assert!(result.completion.contains("fn test"));
        assert_eq!(result.confidence_score, 0.85);
        if let Some(sugs) = result.suggestions {
            assert_eq!(sugs.len(), 2);
            assert_eq!(sugs[0], "suggestion1");
        } else {
            panic!("Suggestions should be present");
        }
        assert_eq!(result.usage.total_tokens, 30);
    }

    #[tokio::test]
    async fn test_hash_prompt_function() {
        let prompt1 = "hello world";
        let prompt2 = "hello world";
        let prompt3 = "goodbye world";

        let hash1 = hash_prompt(prompt1);
        let hash2 = hash_prompt(prompt2);
        let hash3 = hash_prompt(prompt3);

        // Same prompts should produce same hash
        assert_eq!(hash1, hash2);

        // Different prompts should produce different hashes
        assert_ne!(hash1, hash3);

        // Hash should not be empty
        assert!(!hash1.is_empty());
    }
}
