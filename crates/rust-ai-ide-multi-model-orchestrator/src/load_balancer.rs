//! Intelligent Model Load Balancer
//!
//! This module implements intelligent load balancing and request routing across multiple AI models,
//! with priority-based processing and failure detection.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio::time::{Duration, Instant};

use crate::config::{validate_config, OrchestrationConfig};
use crate::types::{
    LoadDecision, ModelId, ModelMetrics, ModelSwitchEvent, ModelTask, RequestContext, RequestPriority, SwitchReason,
};
use crate::{OrchestrationError, Result};

/// Represents a queued request with priority
#[derive(Debug, Clone)]
pub struct QueuedRequest {
    pub id:           uuid::Uuid,
    pub context:      RequestContext,
    pub submitted_at: Instant,
    pub priority:     RequestPriority,
}

impl QueuedRequest {
    pub fn new(context: RequestContext) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            context,
            submitted_at: Instant::now(),
            priority: context.priority,
        }
    }

    pub fn age(&self) -> Duration {
        self.submitted_at.elapsed()
    }
}

impl PartialEq for QueuedRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.age() == other.age()
    }
}

impl Eq for QueuedRequest {}

impl PartialOrd for QueuedRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then age (older requests first)
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => self.submitted_at.cmp(&other.submitted_at), // Older requests first
            pri_cmp => pri_cmp,
        }
    }
}

/// Request routing intelligence based on model capabilities and load
#[derive(Debug)]
pub struct RequestRouter {
    model_capabilities: Arc<RwLock<HashMap<ModelId, Vec<ModelTask>>>>,
    active_requests:    Arc<RwLock<HashMap<ModelId, usize>>>,
    model_health:       Arc<RwLock<HashMap<ModelId, ModelMetrics>>>,
    routing_history:    Arc<RwLock<Vec<(Instant, ModelId)>>>,
}

impl RequestRouter {
    pub async fn route_request(&self, request: QueuedRequest) -> Result<LoadDecision> {
        let model_capabilities = self.model_capabilities.read().await;
        let active_requests = self.active_requests.read().await;
        let model_health = self.model_health.read().await;

        // Find compatible models
        let compatible_models: Vec<ModelId> = model_capabilities
            .iter()
            .filter(|(_, capabilities)| capabilities.contains(&request.context.task_type))
            .map(|(model_id, _)| *model_id)
            .collect();

        if compatible_models.is_empty() {
            return Err(OrchestrationError::LoadBalancingError(
                "No compatible models available".to_string(),
            ));
        }

        // Score models based on current load and performance
        let mut model_scores: Vec<(ModelId, f64, Duration)> = Vec::new();

        for model_id in compatible_models {
            let load_factor = *active_requests.get(&model_id).unwrap_or(&0) as f64;
            let max_concurrent = 10.0; // Placeholder for model capacity
            let load_score = 1.0 - (load_factor / max_concurrent);

            // Calculate expected latency based on current load
            let base_latency = if let Some(metrics) = model_health.get(&model_id) {
                Duration::from_millis(metrics.average_response_time.as_millis() as u64)
            } else {
                Duration::from_millis(500) // Default
            };

            let expected_latency = base_latency.mul_f64(1.0 + load_factor.loaded()); // Increase for higher load

            let final_score = load_score * self.apply_priority_weight(&request.priority);

            model_scores.push((model_id, final_score, expected_latency));
        }

        // Sort by score (highest first)
        model_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (best_model, _, expected_latency) = model_scores[0];

        // Calculate queue time estimate
        let current_requests = *active_requests.get(&best_model).unwrap_or(&0);
        let queue_time_estimate = if current_requests > 0 {
            // Simple estimate: current requests * average latency
            expected_latency.mul_f64(current_requests as f64)
        } else {
            Duration::from_millis(0)
        };

        let decision = LoadDecision {
            target_model:              best_model,
            estimated_queue_time:      queue_time_estimate,
            estimated_processing_time: expected_latency,
            load_factor:               *active_requests.get(&best_model).unwrap_or(&0) as f64 / 10.0, // Normalize
        };

        // Update routing history
        let mut history = self.routing_history.write().await;
        history.push((Instant::now(), best_model));
        if history.len() > 1000 {
            history.remove(0);
        }

        // Update active requests count
        drop(active_requests);
        let mut active_requests = self.active_requests.write().await;
        *active_requests.entry(best_model).or_insert(0) += 1;

        Ok(decision)
    }

    fn apply_priority_weight(&self, priority: &RequestPriority) -> f64 {
        match priority {
            RequestPriority::Critical => 2.0,
            RequestPriority::High => 1.5,
            RequestPriority::Medium => 1.0,
            RequestPriority::Low => 0.7,
        }
    }

    pub async fn complete_request(&self, model_id: &ModelId) {
        let mut active_requests = self.active_requests.write().await;
        if let Some(count) = active_requests.get_mut(model_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    pub async fn update_model_capabilities(&self, model_id: ModelId, capabilities: Vec<ModelTask>) -> Result<()> {
        let mut model_capabilities = self.model_capabilities.write().await;
        model_capabilities.insert(model_id, capabilities);
        Ok(())
    }

    pub async fn update_model_health(&self, model_id: ModelId, metrics: ModelMetrics) -> Result<()> {
        let mut model_health = self.model_health.write().await;
        model_health.insert(model_id, metrics);
        Ok(())
    }
}

/// Intelligent priority-based request queuing system
#[derive(Debug)]
pub struct QueueManager {
    request_queue:       Arc<Mutex<BinaryHeap<QueuedRequest>>>,
    processing_channel:  mpsc::Sender<QueuedRequest>,
    processing_receiver: Arc<Mutex<Option<mpsc::Receiver<QueuedRequest>>>>,
    max_queue_size:      usize,
    processed_requests:  Arc<RwLock<HashMap<uuid::Uuid, Instant>>>,
}

impl QueueManager {
    pub fn new(max_queue_size: usize, processing_tx: mpsc::Sender<QueuedRequest>) -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            request_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            processing_channel: tx,
            processing_receiver: Arc::new(Mutex::new(Some(rx))),
            max_queue_size,
            processed_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn submit_request(&self, request: QueuedRequest) -> Result<()> {
        let mut queue = self.request_queue.lock().await;

        if queue.len() >= self.max_queue_size {
            return Err(OrchestrationError::LoadBalancingError(
                "Request queue is full".to_string(),
            ));
        }

        queue.push(request.clone());

        // Try to notify processing
        let _ = self.processing_channel.try_send(request);

        Ok(())
    }

    pub async fn get_next_request(&self) -> Option<QueuedRequest> {
        let mut queue = self.request_queue.lock().await;

        if let Some(request) = queue.pop() {
            // Track processed request
            let mut processed = self.processed_requests.write().await;
            processed.insert(request.id, Instant::now());
            Some(request)
        } else {
            None
        }
    }

    pub async fn get_queue_size(&self) -> usize {
        let queue = self.request_queue.lock().await;
        queue.len()
    }

    pub async fn get_average_wait_time(&self) -> Duration {
        let processed = self.processed_requests.read().await;

        if processed.is_empty() {
            return Duration::from_millis(0);
        }

        let total_wait: Duration = processed.values().map(|&start| start.elapsed()).sum();

        total_wait / processed.len() as u32
    }

    pub async fn clear_old_processed(&self, max_age: Duration) {
        let mut processed = self.processed_requests.write().await;
        let now = Instant::now();

        processed.retain(|_, start_time| now.duration_since(*start_time) < max_age);
    }
}

/// System capacity monitoring and constraint management
#[derive(Debug)]
pub struct SystemCapacityMonitor {
    system_resources:      Arc<RwLock<SystemCapacity>>,
    model_capacity_limits: Arc<RwLock<HashMap<ModelId, usize>>>,
    capacity_history:      Arc<RwLock<Vec<Instant>>>,
}

#[derive(Debug, Clone)]
pub struct SystemCapacity {
    pub total_memory_mb:        f64,
    pub available_memory_mb:    f64,
    pub total_cpu_cores:        usize,
    pub available_cpu_percent:  f64,
    pub network_bandwidth_mbps: f64,
    pub last_updated:           Instant,
}

impl SystemCapacityMonitor {
    pub async fn is_capacity_available(&self, request: &QueuedRequest) -> bool {
        let capacity = self.system_resources.read().await;
        let model_limits = self.model_capacity_limits.read().await;

        // Check memory requirements
        let memory_required = self.estimate_memory_usage(&request.context);
        if memory_required > capacity.available_memory_mb {
            return false;
        }

        // Check CPU availability
        let cpu_required = self.estimate_cpu_usage(&request.context);
        if cpu_required > capacity.available_cpu_percent {
            return false;
        }

        // Check model-specific limits will be handled by RequestRouter

        true
    }

    fn estimate_memory_usage(&self, _context: &RequestContext) -> f64 {
        // Simple estimation based on task type
        match _context.task_type {
            ModelTask::Completion => 256.0,     // 256MB for code completion
            ModelTask::Chat => 512.0,           // 512MB for chat
            ModelTask::Generation => 1024.0,    // 1GB for code generation
            ModelTask::Analysis => 512.0,       // 512MB for analysis
            ModelTask::Refactoring => 1024.0,   // 1GB for refactoring
            ModelTask::Classification => 256.0, // 256MB for classification
            ModelTask::Translation => 128.0,    // 128MB for translation
        }
    }

    fn estimate_cpu_usage(&self, _context: &RequestContext) -> f64 {
        // Estimate CPU percentage usage
        match _context.expected_complexity {
            crate::types::Complexity::Simple => 5.0,
            crate::types::Complexity::Medium => 15.0,
            crate::types::Complexity::Complex => 30.0,
        }
    }

    pub async fn update_capacity(&self, capacity: SystemCapacity) -> Result<()> {
        let mut system_resources = self.system_resources.write().await;
        *system_resources = capacity;

        // Update history
        let mut history = self.capacity_history.write().await;
        history.push(Instant::now());
        if history.len() > 3600 {
            // Keep 1 hour of history (1 entry per second)
            history.remove(0);
        }

        Ok(())
    }

    pub async fn set_model_limit(&self, model_id: ModelId, max_concurrent: usize) -> Result<()> {
        let mut limits = self.model_capacity_limits.write().await;
        limits.insert(model_id, max_concurrent);
        Ok(())
    }
}

/// Failover engine for transparent recovery from failures
#[derive(Debug)]
pub struct FailoverEngine {
    failover_counts:   Arc<RwLock<HashMap<ModelId, usize>>>,
    max_failovers:     usize,
    failover_cooldown: Duration,
    last_failovers:    Arc<RwLock<HashMap<ModelId, Instant>>>,
}

impl FailoverEngine {
    pub async fn should_failover(&self, model_id: &ModelId) -> bool {
        let counts = self.failover_counts.read().await;
        let last = self.last_failovers.read().await;

        if let Some(count) = counts.get(model_id) {
            if *count >= self.max_failovers {
                // Check if cooldown has expired
                if let Some(last_time) = last.get(model_id) {
                    return self.failover_cooldown.has_elapsed(last_time);
                }
            }
            *count <= self.max_failovers
        } else {
            true // First failure for this model
        }
    }

    pub async fn record_failover(&self, model_id: ModelId) -> Result<()> {
        // Increment count
        let mut counts = self.failover_counts.write().await;
        let count = counts.entry(model_id).or_insert(0);
        *count += 1;

        // Update last failover time
        let mut last = self.last_failovers.write().await;
        last.insert(model_id, Instant::now());

        Ok(())
    }

    pub async fn reset_failover_count(&self, model_id: &ModelId) -> Result<()> {
        let mut counts = self.failover_counts.write().await;
        counts.remove(model_id);

        let mut last = self.last_failovers.write().await;
        last.remove(model_id);

        Ok(())
    }
}

/// Dynamic concurrency limiter based on system capacity
#[derive(Debug)]
pub struct ConcurrencyLimiter {
    semaphore:               Arc<Semaphore>,
    current_limit:           Arc<RwLock<usize>>,
    max_possible_concurrent: usize,
    adjustment_interval:     Duration,
}

impl ConcurrencyLimiter {
    pub fn new(max_possible_concurrent: usize) -> Self {
        let semaphore = Arc::new(Semaphore::new(max_possible_concurrent));

        Self {
            semaphore,
            current_limit: Arc::new(RwLock::new(max_possible_concurrent)),
            max_possible_concurrent,
            adjustment_interval: Duration::from_secs(30),
        }
    }

    pub async fn acquire(&self) -> Result<tokio::sync::SemaphorePermit<'_>> {
        self.semaphore
            .acquire()
            .await
            .map_err(|e| OrchestrationError::LoadBalancingError(format!("Failed to acquire concurrency permit: {}", e)))
    }

    pub async fn adjust_limit(&self, new_limit: usize) -> Result<()> {
        let mut current = self.current_limit.write().await;

        if new_limit > self.max_possible_concurrent {
            return Err(OrchestrationError::LoadBalancingError(
                "New limit exceeds maximum possible concurrent requests".to_string(),
            ));
        }

        // Update semaphore by creating a new one (this is a simplified approach)
        // In practice, you'd want to handle permit transitions more gracefully
        *current = new_limit;

        Ok(())
    }

    pub async fn current_limit(&self) -> usize {
        let limit = self.current_limit.read().await;
        *limit
    }
}

/// Main Model Load Balancer
#[derive(Debug)]
pub struct ModelLoadBalancer {
    pub request_router:      Arc<RequestRouter>,
    pub queue_manager:       Arc<QueueManager>,
    pub capacity_monitor:    Arc<SystemCapacityMonitor>,
    pub failover_engine:     Arc<FailoverEngine>,
    pub concurrency_limiter: Arc<ConcurrencyLimiter>,
    config:                  OrchestrationConfig,
}

impl ModelLoadBalancer {
    pub async fn new(config: OrchestrationConfig) -> Result<Self> {
        validate_config(&config)?;
        let (processing_tx, processing_rx) = mpsc::channel(100);

        let queue_manager = Arc::new(QueueManager::new(
            config.load_balancing_config.queue_capacity,
            processing_tx,
        ));

        // Start background processing task
        let queue_manager_clone = queue_manager.clone();
        tokio::spawn(async move {
            Self::run_background_processing(queue_manager_clone, processing_rx).await;
        });

        // Start capacity monitoring task
        let capacity_monitor = Arc::new(SystemCapacityMonitor {
            system_resources:      Arc::new(RwLock::new(SystemCapacity {
                total_memory_mb:        8192.0, // Default 8GB
                available_memory_mb:    4096.0, // Default 4GB available
                total_cpu_cores:        num_cpus::get(),
                available_cpu_percent:  50.0,   // Default 50% CPU available
                network_bandwidth_mbps: 1000.0, // Default 1Gbps
                last_updated:           Instant::now(),
            })),
            model_capacity_limits: Arc::new(RwLock::new(HashMap::new())),
            capacity_history:      Arc::new(RwLock::new(Vec::new())),
        });

        Ok(Self {
            request_router: Arc::new(RequestRouter {
                model_capabilities: Arc::new(RwLock::new(HashMap::new())),
                active_requests:    Arc::new(RwLock::new(HashMap::new())),
                model_health:       Arc::new(RwLock::new(HashMap::new())),
                routing_history:    Arc::new(RwLock::new(Vec::new())),
            }),
            queue_manager,
            capacity_monitor,
            failover_engine: Arc::new(FailoverEngine {
                failover_counts:   Arc::new(RwLock::new(HashMap::new())),
                max_failovers:     3,
                failover_cooldown: Duration::from_secs(300), // 5 minutes
                last_failovers:    Arc::new(RwLock::new(HashMap::new())),
            }),
            concurrency_limiter: Arc::new(ConcurrencyLimiter::new(
                config.load_balancing_config.max_concurrent_requests,
            )),
            config,
        })
    }

    pub async fn submit_request(&self, context: RequestContext) -> Result<LoadDecision> {
        let request = QueuedRequest::new(context);

        // Check capacity before submitting
        if !self.capacity_monitor.is_capacity_available(&request).await {
            return Err(OrchestrationError::LoadBalancingError(
                "Insufficient system capacity".to_string(),
            ));
        }

        // Submit to queue
        self.queue_manager.submit_request(request).await?;

        // Route the request (this will update active request counts)
        let decision = self.request_router.route_request(request).await?;

        Ok(decision)
    }

    async fn run_background_processing(
        queue_manager: Arc<QueueManager>,
        mut processing_rx: mpsc::Receiver<QueuedRequest>,
    ) {
        loop {
            tokio::select! {
                Some(request) = processing_rx.recv() => {
                    // Process request here
                    // This is a simplified version - in practice you'd have a pool of workers
                    tracing::info!("Processing queued request: {}", request.id);

                    // Simulate processing time
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Clean up processed requests periodically
                    queue_manager.clear_old_processed(Duration::from_secs(3600)).await;
                }
                _ = tokio::time::sleep(queue_manager::DEFAULT_CHECK_INTERVAL) => {
                    // Periodic cleanup
                    queue_manager.clear_old_processed(Duration::from_secs(3600)).await;
                }
            }
        }
    }

    pub async fn get_queue_stats(&self) -> LoadBalancerStats {
        LoadBalancerStats {
            queue_size:                  self.queue_manager.get_queue_size().await,
            average_wait_time:           self.queue_manager.get_average_wait_time().await,
            current_concurrent_requests: self.concurrency_limiter.current_limit().await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub queue_size:                  usize,
    pub average_wait_time:           Duration,
    pub current_concurrent_requests: usize,
}

const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(60);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrchestrationConfigBuilder;
    use crate::types::Complexity;

    #[tokio::test]
    async fn test_load_balancing() {
        let config = OrchestrationConfigBuilder::new().build();
        let lb = ModelLoadBalancer::new(config).await.unwrap();

        // Test request submission
        let context = RequestContext {
            task_type:           ModelTask::Completion,
            input_length:        100,
            priority:            RequestPriority::Medium,
            expected_complexity: Complexity::Medium,
            acceptable_latency:  Duration::from_secs(5),
            preferred_hardware:  None,
        };

        // This should fail without models, but test the submission flow
        let result = lb.submit_request(context).await;

        // Expected to fail due to no models available
        assert!(result.is_err());
    }
}
