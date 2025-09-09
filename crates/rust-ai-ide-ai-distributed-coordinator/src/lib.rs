//! # Distributed AI Coordinator
//!
//! This crate provides distributed AI processing capabilities including:
//! - Load balancing across multiple AI worker nodes
//! - Model sharding for large neural networks
//! - GPU acceleration support
//! - Predictive model caching and preloading
//! - Intelligent request routing based on model capabilities

pub mod coordinator;
pub mod load_balancer;
pub mod model_sharding;
pub mod worker_pool;
pub mod metrics;
pub mod routing;

#[cfg(feature = "sharding")]
pub mod sharding_engine;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use rust_ai_ide_errors::IDEResult;
use rust_ai_ide_ai_inference::{
    InferenceEngine, GenerationConfig, AnalysisType, GenerationResult, AnalysisResult,
    CodeCompletionConfig, CodeCompletionResult, InferenceError, TokenUsage
};

/// Worker node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub capabilities: WorkerCapabilities,
    pub status: WorkerStatus,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub model_assignments: Vec<ModelAssignment>,
}

/// Worker capabilities (GPU, memory, models, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub has_gpu: bool,
    pub gpu_memory_gb: Option<u64>,
    pub gpu_type: Option<String>,
    pub cpu_cores: u32,
    pub memory_gb: u64,
    pub supported_models: Vec<String>,
    pub max_concurrent_requests: u32,
}

/// Worker status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerStatus {
    Active,
    Busy,
    Offline,
    Maintenance,
}

/// Model assignment to worker nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAssignment {
    pub model_id: String,
    pub shard_info: Option<ShardInfo>,
    pub is_primary: bool,
}

/// Model sharding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_id: String,
    pub shard_index: u32,
    pub total_shards: u32,
    pub memory_offset: u64,
    pub memory_size: u64,
}

/// Distributed AI coordinator trait
#[async_trait]
pub trait DistributedAICoordinator: Send + Sync + 'static {
    /// Register a new worker node
    async fn register_worker(&self, node: WorkerNode) -> IDEResult<()>;

    /// Unregister a worker node
    async fn unregister_worker(&self, worker_id: &str) -> IDEResult<()>;

    /// Get worker health status
    async fn get_worker_status(&self, worker_id: &str) -> IDEResult<WorkerStatus>;

    /// Distribute inference request across workers
    async fn execute_distributed(
        &self,
        request: DistributedInferenceRequest
    ) -> IDEResult<DistributedInferenceResponse>;

    /// Get coordinator statistics
    async fn get_stats(&self) -> IDEResult<CoordinatorStats>;
}

/// Types of distributed inference requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributedInferenceRequest {
    TextGeneration {
        prompts: Vec<String>,
        config: GenerationConfig,
        model_preference: Option<String>,
    },
    CodeAnalysis {
        code_snippets: Vec<String>,
        analysis_type: AnalysisType,
    },
    CodeCompletion {
        contexts: Vec<String>,
        prefixes: Vec<String>,
        config: CodeCompletionConfig,
    },
}

/// Distributed inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedInferenceResponse {
    pub request_id: String,
    pub results: Vec<InferenceResult>,
    pub aggregated_stats: DistributedStats,
    pub latency_ms: u64,
}

/// Individual inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub worker_id: String,
    pub success: bool,
    pub data: Option<InferenceData>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Inference result data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceData {
    Text(GenerationResult),
    Analysis(AnalysisResult),
    Completion(CodeCompletionResult),
}

/// Distributed processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedStats {
    pub total_tokens_processed: u64,
    pub workers_used: usize,
    pub load_distribution: HashMap<String, f64>, // worker_id -> load percentage
    pub cache_hit_ratio: f64,
}

/// Coordinator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorStats {
    pub total_workers: usize,
    pub active_workers: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub average_latency_ms: f64,
    pub memory_usage_gb: f64,
    pub gpu_utilization_percent: Option<f64>,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastLoaded,
    LeastLatent,
    GPUOptimized,
    Adaptive,
}

/// Model sharding strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardingStrategy {
    LayerBased,
    TensorParallelism,
    SequenceParallelism,
    PipelineParallelism,
}

/// Configuration for distributed AI coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedAIConfig {
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub sharding_strategy: ShardingStrategy,
    pub max_workers_per_request: usize,
    pub request_timeout_ms: u64,
    pub health_check_interval_ms: u64,
    pub model_preloading_enabled: bool,
    pub predictive_caching_enabled: bool,
    pub metrics_collection_enabled: bool,
}

impl Default for DistributedAIConfig {
    fn default() -> Self {
        Self {
            load_balancing_strategy: LoadBalancingStrategy::Adaptive,
            sharding_strategy: ShardingStrategy::LayerBased,
            max_workers_per_request: 3,
            request_timeout_ms: 30000,
            health_check_interval_ms: 5000,
            model_preloading_enabled: true,
            predictive_caching_enabled: true,
            metrics_collection_enabled: true,
        }
    }
}

/// Main distributed AI coordinator implementation
pub struct DistributedAICoordinatorImpl {
    config: DistributedAIConfig,
    workers: Arc<RwLock<HashMap<String, WorkerNode>>>,
    stats: Arc<RwLock<CoordinatorStats>>,
    load_balancer: Box<dyn load_balancer::LoadBalancer>,
    metrics: Option<metrics::DistributedMetrics>,
}

impl DistributedAICoordinatorImpl {
    pub fn new(config: DistributedAIConfig) -> Self {
        let load_balancer = match config.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => {
                Box::new(load_balancer::RoundRobinLoadBalancer::new())
            }
            LoadBalancingStrategy::LeastLoaded => {
                Box::new(load_balancer::LeastLoadedBalancer::new())
            }
            LoadBalancingStrategy::GPUOptimized => {
                Box::new(load_balancer::GPUOptimizedBalancer::new())
            }
            LoadBalancingStrategy::Adaptive => {
                Box::new(load_balancer::AdaptiveLoadBalancer::new(config.clone()))
            }
            _ => Box::new(load_balancer::RoundRobinLoadBalancer::new()),
        };

        let metrics = if config.metrics_collection_enabled {
            Some(metrics::DistributedMetrics::new())
        } else {
            None
        };

        Self {
            config,
            workers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CoordinatorStats::default())),
            load_balancer,
            metrics,
        }
    }
}

#[async_trait]
impl DistributedAICoordinator for DistributedAICoordinatorImpl {
    async fn register_worker(&self, node: WorkerNode) -> IDEResult<()> {
        info!("Registering worker node: {}", node.id);

        let mut workers = self.workers.write().await;
        workers.insert(node.id.clone(), node);

        let mut stats = self.stats.write().await;
        stats.total_workers = workers.len();
        stats.active_workers = workers.values()
            .filter(|w| matches!(w.status, WorkerStatus::Active))
            .count();

        Ok(())
    }

    async fn unregister_worker(&self, worker_id: &str) -> IDEResult<()> {
        info!("Unregistering worker node: {}", worker_id);

        let mut workers = self.workers.write().await;
        workers.remove(worker_id);

        let mut stats = self.stats.write().await;
        stats.total_workers = workers.len();
        stats.active_workers = workers.values()
            .filter(|w| matches!(w.status, WorkerStatus::Active))
            .count();

        Ok(())
    }

    async fn get_worker_status(&self, worker_id: &str) -> IDEResult<WorkerStatus> {
        let workers = self.workers.read().await;
        workers.get(worker_id)
            .map(|w| w.status.clone())
            .ok_or_else(|| rust_ai_ide_errors::IDEError::NotFound(format!("Worker {} not found", worker_id)))
    }

    async fn execute_distributed(
        &self,
        request: DistributedInferenceRequest
    ) -> IDEResult<DistributedInferenceResponse> {
        let start_time = std::time::Instant::now();

        // Get available workers for this request
        let workers = self.workers.read().await;
        let available_workers: Vec<_> = workers.values()
            .filter(|w| matches!(w.status, WorkerStatus::Active))
            .cloned()
            .collect();

        if available_workers.is_empty() {
            return Err(rust_ai_ide_errors::IDEError::InternalError(
                "No active workers available".to_string()
            ));
        }

        // Use load balancer to select optimal workers
        let selected_workers = self.load_balancer.select_workers(
            &request,
            &available_workers,
            self.config.max_workers_per_request
        ).await?;

        if selected_workers.is_empty() {
            return Err(rust_ai_ide_errors::IDEError::InternalError(
                "Load balancer could not select workers".to_string()
            ));
        }

        // Distribute request across selected workers
        let request_id = uuid::Uuid::new_v4().to_string();
        let mut results = Vec::new();

        for worker in selected_workers {
            match self.execute_on_worker(&worker, &request).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Request failed on worker {}: {}", worker.id, e);
                    results.push(InferenceResult {
                        worker_id: worker.id,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                        duration_ms: 0,
                    });
                }
            }
        }

        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Update statistics
        if let Some(metrics) = &self.metrics {
            metrics.record_request(latency_ms, results.len()).await;
        }

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        if results.iter().any(|r| r.success) {
            stats.successful_requests += 1;
        }

        let response = DistributedInferenceResponse {
            request_id,
            results,
            aggregated_stats: DistributedStats {
                total_tokens_processed: self.calculate_total_tokens(&results),
                workers_used: results.len(),
                load_distribution: self.calculate_load_distribution(&results),
                cache_hit_ratio: 0.0, // TODO: implement cache hit tracking
            },
            latency_ms,
        };

        Ok(response)
    }

    async fn get_stats(&self) -> IDEResult<CoordinatorStats> {
        let stats = self.stats.read().await.clone();
        Ok(stats)
    }
}

impl DistributedAICoordinatorImpl {
    async fn execute_on_worker(
        &self,
        worker: &WorkerNode,
        request: &DistributedInferenceRequest
    ) -> IDEResult<InferenceResult> {
        let start_time = std::time::Instant::now();

        // TODO: Implement actual HTTP request to worker
        // For now, simulate successful execution
        let result = InferenceResult {
            worker_id: worker.id.clone(),
            success: true,
            data: Some(self.generate_mock_result(request)),
            error: None,
            duration_ms: start_time.elapsed().as_millis() as u64,
        };

        Ok(result)
    }

    fn generate_mock_result(&self, request: &DistributedInferenceRequest) -> InferenceData {
        match request {
            DistributedInferenceRequest::TextGeneration { .. } => {
                InferenceData::Text(GenerationResult {
                    text: "Mock generated text".to_string(),
                    finish_reason: "stop".to_string(),
                    usage: TokenUsage {
                        prompt_tokens: 10,
                        completion_tokens: 20,
                        total_tokens: 30,
                    },
                    generation_time_ms: 150,
                })
            }
            DistributedInferenceRequest::CodeAnalysis { analysis_type, .. } => {
                InferenceData::Analysis(AnalysisResult {
                    analysis: format!("Mock analysis for {:?}", analysis_type),
                    suggestions: vec!["suggestion 1".to_string(), "suggestion 2".to_string()],
                    severity_scores: vec![0.5, 0.7],
                    usage: TokenUsage {
                        prompt_tokens: 50,
                        completion_tokens: 100,
                        total_tokens: 150,
                    },
                })
            }
            DistributedInferenceRequest::CodeCompletion { .. } => {
                InferenceData::Completion(CodeCompletionResult {
                    completion: "mock completion".to_string(),
                    confidence_score: 0.85,
                    suggestions: Some(vec!["alt1".to_string(), "alt2".to_string()]),
                    usage: TokenUsage {
                        prompt_tokens: 25,
                        completion_tokens: 15,
                        total_tokens: 40,
                    },
                })
            }
        }
    }

    fn calculate_total_tokens(&self, results: &[InferenceResult]) -> u64 {
        results.iter()
            .filter_map(|r| r.data.as_ref())
            .map(|data| match data {
                InferenceData::Text(r) => r.usage.total_tokens,
                InferenceData::Analysis(r) => r.usage.total_tokens,
                InferenceData::Completion(r) => r.usage.total_tokens,
            })
            .sum()
    }

    fn calculate_load_distribution(&self, results: &[InferenceResult]) -> HashMap<String, f64> {
        let mut distribution = HashMap::new();
        let total_results = results.len() as f64;

        for result in results {
            let load = distribution.get(&result.worker_id).unwrap_or(&0.0) + 1.0;
            distribution.insert(result.worker_id.clone(), load / total_results);
        }

        distribution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_registration() {
        let config = DistributedAIConfig::default();
        let coordinator = DistributedAICoordinatorImpl::new(config);

        let worker = WorkerNode {
            id: "worker1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            capabilities: WorkerCapabilities {
                has_gpu: true,
                gpu_memory_gb: Some(16),
                gpu_type: Some("A100".to_string()),
                cpu_cores: 8,
                memory_gb: 32,
                supported_models: vec!["codellama".to_string()],
                max_concurrent_requests: 10,
            },
            status: WorkerStatus::Active,
            last_heartbeat: chrono::Utc::now(),
            model_assignments: Vec::new(),
        };

        coordinator.register_worker(worker).await.unwrap();

        let stats = coordinator.get_stats().await.unwrap();
        assert_eq!(stats.total_workers, 1);
        assert_eq!(stats.active_workers, 1);
    }

    #[tokio::test]
    async fn test_distributed_execution_mock() {
        let config = DistributedAIConfig::default();
        let coordinator = DistributedAICoordinatorImpl::new(config);

        // Register a mock worker
        let worker = WorkerNode {
            id: "worker1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            capabilities: WorkerCapabilities {
                has_gpu: true,
                gpu_memory_gb: Some(16),
                gpu_type: Some("A100".to_string()),
                cpu_cores: 8,
                memory_gb: 32,
                supported_models: vec!["codellama".to_string()],
                max_concurrent_requests: 10,
            },
            status: WorkerStatus::Active,
            last_heartbeat: chrono::Utc::now(),
            model_assignments: Vec::new(),
        };

        coordinator.register_worker(worker).await.unwrap();

        // Execute distributed request
        let request = DistributedInferenceRequest::TextGeneration {
            prompts: vec!["Write a hello world function".to_string()],
            config: GenerationConfig {
                max_tokens: 100,
                temperature: 0.7,
                top_p: 0.9,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                stop_sequences: vec![],
                echo: false,
                stream: false,
            },
            model_preference: None,
        };

        let response = coordinator.execute_distributed(request).await.unwrap();

        assert_eq!(response.results.len(), 1);
        assert!(response.results[0].success);
        if let Some(InferenceData::Text(data)) = &response.results[0].data {
            assert_eq!(data.text, "Mock generated text");
        } else {
            panic!("Expected Text inference data");
        }
    }
}