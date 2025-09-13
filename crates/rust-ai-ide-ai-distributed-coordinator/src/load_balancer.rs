//! Load balancing algorithms for distributed AI processing
//!
//! This module provides different load balancing strategies:
//! - Round Robin: Simple rotation across workers
//! - Least Loaded: Routes to worker with least load
//! - GPU Optimized: Prioritizes GPU-enabled workers for GPU-intensive tasks
//! - Adaptive: Uses performance metrics to make intelligent decisions

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;

use super::{
    DistributedAIConfig, DistributedInferenceRequest, LoadBalancingStrategy, WorkerCapabilities, WorkerNode,
    WorkerStatus,
};

/// Trait for load balancing algorithms
#[async_trait]
pub trait LoadBalancer: Send + Sync + 'static {
    /// Select optimal workers for a distributed inference request
    async fn select_workers(
        &mut self,
        request: &DistributedInferenceRequest,
        available_workers: &[WorkerNode],
        max_workers: usize,
    ) -> Result<Vec<WorkerNode>, String>;

    /// Update load balancing state based on completed request
    async fn update_state(&mut self, worker_id: &str, success: bool, latency_ms: u64);
}

/// Round-robin load balancer
pub struct RoundRobinLoadBalancer {
    current_index: RwLock<usize>,
}

impl RoundRobinLoadBalancer {
    pub fn new() -> Self {
        Self {
            current_index: RwLock::new(0),
        }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    async fn select_workers(
        &mut self,
        _request: &DistributedInferenceRequest,
        available_workers: &[WorkerNode],
        max_workers: usize,
    ) -> Result<Vec<WorkerNode>, String> {
        if available_workers.is_empty() {
            return Err("No workers available".to_string());
        }

        let start_index = {
            let mut current = self.current_index.write().await;
            let index = *current;
            *current = (*current + 1) % available_workers.len();
            index
        };

        let mut selected = Vec::new();
        let worker_count = std::cmp::min(max_workers, available_workers.len());

        for i in 0..worker_count {
            let worker_index = (start_index + i) % available_workers.len();
            selected.push(available_workers[worker_index].clone());
        }

        Ok(selected)
    }

    async fn update_state(&mut self, _worker_id: &str, _success: bool, _latency_ms: u64) {
        // Round-robin doesn't use state updates
    }
}

/// Least-loaded load balancer
pub struct LeastLoadedBalancer {
    worker_load: RwLock<HashMap<String, f64>>, // worker_id -> load_factor (0.0-1.0)
}

impl LeastLoadedBalancer {
    pub fn new() -> Self {
        Self {
            worker_load: RwLock::new(HashMap::new()),
        }
    }

    fn calculate_worker_load(&self, worker: &WorkerNode) -> f64 {
        // Simple load calculation based on capabilities and status
        match worker.status {
            WorkerStatus::Active => {
                // Base load calculation - can be enhanced with actual metrics
                if worker.capabilities.has_gpu {
                    // GPU workers have higher capacity
                    0.2 // Assume 20% load initially
                } else {
                    // CPU-only workers
                    0.3 // Assume 30% load initially
                }
            }
            WorkerStatus::Busy => 0.9, // Nearly full
            _ => 1.0,                  // Offline/maintenance - don't use
        }
    }
}

#[async_trait]
impl LoadBalancer for LeastLoadedBalancer {
    async fn select_workers(
        &mut self,
        _request: &DistributedInferenceRequest,
        available_workers: &[WorkerNode],
        max_workers: usize,
    ) -> Result<Vec<WorkerNode>, String> {
        if available_workers.is_empty() {
            return Err("No workers available".to_string());
        }

        let mut worker_loads: Vec<(String, f64, WorkerNode)> = Vec::new();
        let loads = self.worker_load.read().await;

        for worker in available_workers {
            let load = loads
                .get(&worker.id)
                .copied()
                .unwrap_or_else(|| self.calculate_worker_load(worker));

            if load < 0.95 {
                // Don't use workers >95% load
                worker_loads.push((worker.id.clone(), load, worker.clone()));
            }
        }

        if worker_loads.is_empty() {
            return Err("No available workers with capacity".to_string());
        }

        // Sort by load (ascending) and take top max_workers
        worker_loads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let selected: Vec<_> = worker_loads
            .into_iter()
            .take(max_workers)
            .map(|(_, _, worker)| worker)
            .collect();

        info!("Least loaded balancer selected {} workers", selected.len());
        Ok(selected)
    }

    async fn update_state(&mut self, worker_id: &str, success: bool, latency_ms: u64) {
        let mut loads = self.worker_load.write().await;

        // Update load based on recent performance
        let current_load = loads.get(worker_id).copied().unwrap_or(0.0);
        let adjustment = if success {
            // Success: slightly decrease load, faster for low latency
            if latency_ms < 100 {
                -0.05 // Good performance, reduce load faster
            } else {
                -0.02 // Normal performance
            }
        } else {
            // Failure: significantly increase load
            0.2
        };

        let new_load = (current_load + adjustment).clamp(0.0, 1.0);
        loads.insert(worker_id.to_string(), new_load);
    }
}

/// GPU-optimized load balancer
pub struct GPUOptimizedBalancer {
    gpu_preference_score: RwLock<HashMap<String, f64>>,
}

impl GPUOptimizedBalancer {
    pub fn new() -> Self {
        Self {
            gpu_preference_score: RwLock::new(HashMap::new()),
        }
    }

    fn calculate_gpu_score(&self, worker: &WorkerNode, request: &DistributedInferenceRequest) -> f64 {
        let base_score = if worker.capabilities.has_gpu {
            match self.is_gpu_intensive_request(request) {
                true => 1.0,  // GPU-intensive: prefer GPU workers
                false => 0.6, // Not GPU-intensive: GPU workers still good but not mandatory
            }
        } else {
            match self.is_gpu_intensive_request(request) {
                true => 0.2,  // GPU-intensive on CPU: poor fit
                false => 0.8, // Not GPU-intensive: CPU workers acceptable
            }
        };

        // Factor in GPU memory and type
        let gpu_bonus = if worker.capabilities.has_gpu {
            match worker.capabilities.gpu_memory_gb {
                Some(mem) if mem >= 16 => 0.2, // High memory GPU
                Some(mem) if mem >= 8 => 0.1,  // Medium memory GPU
                _ => 0.0,
            }
        } else {
            0.0
        };

        base_score + gpu_bonus
    }

    fn is_gpu_intensive_request(&self, request: &DistributedInferenceRequest) -> bool {
        match request {
            DistributedInferenceRequest::TextGeneration { config, .. } => {
                // Large token generation is GPU-intensive
                config.max_tokens > 1000
            }
            DistributedInferenceRequest::CodeAnalysis { code_snippets, .. } => {
                // Multiple large snippets are GPU-intensive
                code_snippets.len() > 1 || code_snippets.iter().any(|s| s.len() > 10000)
            }
            DistributedInferenceRequest::CodeCompletion { .. } => {
                // Code completion is generally not too GPU-intensive
                false
            }
        }
    }
}

#[async_trait]
impl LoadBalancer for GPUOptimizedBalancer {
    async fn select_workers(
        &mut self,
        request: &DistributedInferenceRequest,
        available_workers: &[WorkerNode],
        max_workers: usize,
    ) -> Result<Vec<WorkerNode>, String> {
        if available_workers.is_empty() {
            return Err("No workers available".to_string());
        }

        let mut scored_workers: Vec<(String, f64, WorkerNode)> = Vec::new();
        let scores = self.gpu_preference_score.read().await;

        for worker in available_workers {
            // Only include Active workers
            if !matches!(worker.status, WorkerStatus::Active) {
                continue;
            }

            let gpu_score = self.calculate_gpu_score(worker, request);
            let preference_score = scores.get(&worker.id).copied().unwrap_or(0.0);
            let combined_score = gpu_score + preference_score * 0.1; // Small preference bonus

            scored_workers.push((worker.id.clone(), combined_score, worker.clone()));
        }

        if scored_workers.is_empty() {
            return Err("No active workers available".to_string());
        }

        // Sort by score (descending) and take top max_workers
        scored_workers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let selected: Vec<_> = scored_workers
            .into_iter()
            .take(max_workers)
            .map(|(_, _, worker)| worker)
            .collect();

        info!("GPU optimized balancer selected {} workers", selected.len());
        Ok(selected)
    }

    async fn update_state(&mut self, worker_id: &str, success: bool, latency_ms: u64) {
        let mut scores = self.gpu_preference_score.write().await;

        let current_score = scores.get(worker_id).copied().unwrap_or(0.0);
        let adjustment = if success && latency_ms < 200 {
            0.1 // Good GPU performance
        } else if success {
            0.05 // Acceptable performance
        } else {
            -0.1 // Poor performance
        };

        let new_score = (current_score + adjustment).clamp(-1.0, 1.0);
        scores.insert(worker_id.to_string(), new_score);
    }
}

/// Adaptive load balancer that learns from performance metrics
pub struct AdaptiveLoadBalancer {
    config:              DistributedAIConfig,
    performance_history: RwLock<HashMap<String, Vec<PerformanceRecord>>>,
    last_used_strategy:  RwLock<LoadBalancingStrategy>,
}

#[derive(Debug, Clone)]
struct PerformanceRecord {
    timestamp:  std::time::Instant,
    latency_ms: u64,
    success:    bool,
}

impl AdaptiveLoadBalancer {
    pub fn new(config: DistributedAIConfig) -> Self {
        Self {
            config,
            performance_history: RwLock::new(HashMap::new()),
            last_used_strategy: RwLock::new(LoadBalancingStrategy::RoundRobin),
        }
    }

    fn select_best_strategy(&self, workers: &[WorkerNode]) -> LoadBalancingStrategy {
        // Simple strategy selection based on worker characteristics
        let gpu_workers = workers.iter().filter(|w| w.capabilities.has_gpu).count();
        let total_workers = workers.len();

        if gpu_workers as f64 / total_workers as f64 > 0.7 {
            // Mostly GPU workers: use GPU-optimized
            LoadBalancingStrategy::GPUOptimized
        } else if total_workers > 5 {
            // Many workers: use least loaded
            LoadBalancingStrategy::LeastLoaded
        } else {
            // Few workers: use round-robin
            LoadBalancingStrategy::RoundRobin
        }
    }
}

#[async_trait]
impl LoadBalancer for AdaptiveLoadBalancer {
    async fn select_workers(
        &mut self,
        request: &DistributedInferenceRequest,
        available_workers: &[WorkerNode],
        max_workers: usize,
    ) -> Result<Vec<WorkerNode>, String> {
        if available_workers.is_empty() {
            return Err("No workers available".to_string());
        }

        // Select best strategy based on current conditions
        let strategy = self.select_best_strategy(available_workers);
        *self.last_used_strategy.write().await = strategy.clone();

        // Use the selected strategy to choose workers
        let selected = match strategy {
            LoadBalancingStrategy::RoundRobin => {
                let mut rr_lb = RoundRobinLoadBalancer::new();
                rr_lb
                    .select_workers(request, available_workers, max_workers)
                    .await?
            }
            LoadBalancingStrategy::LeastLoaded => {
                let mut ll_lb = LeastLoadedBalancer::new();
                ll_lb
                    .select_workers(request, available_workers, max_workers)
                    .await?
            }
            LoadBalancingStrategy::GPUOptimized => {
                let mut gpu_lb = GPUOptimizedBalancer::new();
                gpu_lb
                    .select_workers(request, available_workers, max_workers)
                    .await?
            }
            _ => {
                let mut rr_lb = RoundRobinLoadBalancer::new();
                rr_lb
                    .select_workers(request, available_workers, max_workers)
                    .await?
            }
        };

        info!(
            "Adaptive balancer selected strategy {:?}, {} workers",
            strategy,
            selected.len()
        );
        Ok(selected)
    }

    async fn update_state(&mut self, worker_id: &str, success: bool, latency_ms: u64) {
        let mut history = self.performance_history.write().await;

        let records = history
            .entry(worker_id.to_string())
            .or_insert_with(Vec::new);
        records.push(PerformanceRecord {
            timestamp: std::time::Instant::now(),
            latency_ms,
            success,
        });

        // Keep only recent records (last 100)
        if records.len() > 100 {
            records.remove(0);
        }

        // Clean old records (older than 5 minutes)
        let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(300);
        records.retain(|r| r.timestamp > cutoff);
    }
}

/// Factory function to create load balancers
pub fn create_load_balancer(
    strategy: &LoadBalancingStrategy,
    config: Option<DistributedAIConfig>,
) -> Box<dyn LoadBalancer> {
    match strategy {
        LoadBalancingStrategy::RoundRobin => Box::new(RoundRobinLoadBalancer::new()),
        LoadBalancingStrategy::LeastLoaded => Box::new(LeastLoadedBalancer::new()),
        LoadBalancingStrategy::GPUOptimized => Box::new(GPUOptimizedBalancer::new()),
        LoadBalancingStrategy::Adaptive => Box::new(AdaptiveLoadBalancer::new(
            config.unwrap_or_else(DistributedAIConfig::default),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GenerationConfig, WorkerCapabilities, WorkerNode, WorkerStatus};

    fn create_test_workers() -> Vec<WorkerNode> {
        vec![
            WorkerNode {
                id:                "gpu-worker-1".to_string(),
                address:           "192.168.1.10".to_string(),
                port:              8080,
                capabilities:      WorkerCapabilities {
                    has_gpu:                 true,
                    gpu_memory_gb:           Some(16),
                    gpu_type:                Some("A100".to_string()),
                    cpu_cores:               8,
                    memory_gb:               32,
                    supported_models:        vec!["codellama".to_string(), "starcoder".to_string()],
                    max_concurrent_requests: 10,
                },
                status:            WorkerStatus::Active,
                last_heartbeat:    chrono::Utc::now(),
                model_assignments: Vec::new(),
            },
            WorkerNode {
                id:                "cpu-worker-1".to_string(),
                address:           "192.168.1.11".to_string(),
                port:              8080,
                capabilities:      WorkerCapabilities {
                    has_gpu:                 false,
                    gpu_memory_gb:           None,
                    gpu_type:                None,
                    cpu_cores:               16,
                    memory_gb:               64,
                    supported_models:        vec!["codellama".to_string()],
                    max_concurrent_requests: 20,
                },
                status:            WorkerStatus::Active,
                last_heartbeat:    chrono::Utc::now(),
                model_assignments: Vec::new(),
            },
        ]
    }

    #[tokio::test]
    async fn test_round_robin_balancer() {
        let mut lb = RoundRobinLoadBalancer::new();
        let workers = create_test_workers();

        let request = DistributedInferenceRequest::TextGeneration {
            prompts:          vec!["test prompt".to_string()],
            config:           GenerationConfig::default(),
            model_preference: None,
        };

        let selected = lb.select_workers(&request, &workers, 1).await.unwrap();
        assert_eq!(selected.len(), 1);

        // Test round-robin rotation
        let selected2 = lb.select_workers(&request, &workers, 1).await.unwrap();
        assert_eq!(selected2.len(), 1);
        assert_ne!(selected[0].id, selected2[0].id);
    }

    #[tokio::test]
    async fn test_least_loaded_balancer() {
        let mut lb = LeastLoadedBalancer::new();
        let workers = create_test_workers();

        let request = DistributedInferenceRequest::TextGeneration {
            prompts:          vec!["test prompt".to_string()],
            config:           GenerationConfig::default(),
            model_preference: None,
        };

        let selected = lb.select_workers(&request, &workers, 2).await.unwrap();
        assert_eq!(selected.len(), 2);

        // Should select both workers (they have capacity)
        let worker_ids: Vec<String> = selected.iter().map(|w| w.id.clone()).collect();
        assert!(worker_ids.contains(&"gpu-worker-1".to_string()));
        assert!(worker_ids.contains(&"cpu-worker-1".to_string()));
    }

    #[tokio::test]
    async fn test_gpu_optimized_balancer() {
        let mut lb = GPUOptimizedBalancer::new();
        let workers = create_test_workers();

        let request = DistributedInferenceRequest::TextGeneration {
            prompts:          vec!["test prompt".to_string()],
            config:           GenerationConfig::default(),
            model_preference: None,
        };

        let selected = lb.select_workers(&request, &workers, 1).await.unwrap();
        assert_eq!(selected.len(), 1);

        // Should prefer GPU worker for text generation
        assert_eq!(selected[0].id, "gpu-worker-1");
    }

    #[tokio::test]
    async fn test_create_load_balancer() {
        let rr_lb = create_load_balancer(&LoadBalancingStrategy::RoundRobin, None);
        assert!(rr_lb
            .as_ref()
            .downcast_ref::<RoundRobinLoadBalancer>()
            .is_some());

        let gpu_lb = create_load_balancer(&LoadBalancingStrategy::GPUOptimized, None);
        assert!(gpu_lb
            .as_ref()
            .downcast_ref::<GPUOptimizedBalancer>()
            .is_some());

        let config = DistributedAIConfig::default();
        let adaptive_lb = create_load_balancer(&LoadBalancingStrategy::Adaptive, Some(config));
        assert!(adaptive_lb
            .as_ref()
            .downcast_ref::<AdaptiveLoadBalancer>()
            .is_some());
    }
}
