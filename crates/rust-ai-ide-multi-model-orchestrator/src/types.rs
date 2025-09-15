//! Type definitions for Multi-Model Orchestration
//!
//! This module defines all the core types used by the multi-model orchestration system.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a model instance
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelId(pub Uuid);

impl ModelId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Model performance metrics collected in real-time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub model_id: ModelId,
    pub latency_ms: f64,
    pub accuracy_score: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub last_updated: Instant,
    pub request_count: u64,
    pub error_count: u64,
    pub average_response_time: Duration,
}

impl ModelMetrics {
    pub fn new(model_id: ModelId) -> Self {
        Self {
            model_id,
            latency_ms: 0.0,
            accuracy_score: 0.5, // Default neutral score
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            last_updated: Instant::now(),
            request_count: 0,
            error_count: 0,
            average_response_time: Duration::from_millis(100),
        }
    }

    pub fn update_latency(&mut self, latency: Duration) {
        self.request_count += 1;
        self.latency_ms = self.latency_ms * 0.9 + latency.as_millis() as f64 * 0.1; // Exponential moving average
        self.average_response_time = Duration::from_millis(self.latency_ms as u64);
        self.last_updated = Instant::now();
    }

    pub fn is_healthy(&self) -> bool {
        self.latency_ms < 5000.0 && // Less than 5 seconds
        (self.error_count as f64 / self.request_count.max(1) as f64) < 0.1 && // Less than 10% error rate
        self.last_updated.elapsed() < Duration::from_secs(60) // Updated within last minute
    }
}

/// Model capability specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    pub supported_tasks: Vec<ModelTask>,
    pub max_context_length: usize,
    pub supported_languages: Vec<String>,
    pub quantization_level: Option<String>,
    pub hardware_acceleration: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTask {
    Completion,
    Chat,
    Classification,
    Generation,
    Analysis,
    Refactoring,
    Translation,
}

/// Model instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: ModelId,
    pub name: String,
    pub version: String,
    pub capability: ModelCapability,
    pub status: ModelStatus,
    pub metrics: ModelMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    Available,
    Busy,
    Unhealthy,
    Offline,
    WarmingUp,
}

/// Request characteristics for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub task_type: ModelTask,
    pub input_length: usize,
    pub priority: RequestPriority,
    pub expected_complexity: Complexity,
    pub acceptable_latency: Duration,
    pub preferred_hardware: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
}

/// Model selection recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model_id: ModelId,
    pub confidence_score: f64,
    pub expected_latency: Duration,
    pub resource_cost_estimate: f64,
    pub selection_reason: String,
}

/// Load balancer decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadDecision {
    pub target_model: ModelId,
    pub estimated_queue_time: Duration,
    pub estimated_processing_time: Duration,
    pub load_factor: f64,
}

/// Consensus result from multiple models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub final_result: String,
    pub confidence_score: f64,
    pub model_contributions: HashMap<ModelId, ModelContribution>,
    pub disagreement_score: f64,
    pub primary_model: ModelId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelContribution {
    pub model_id: ModelId,
    pub result: String,
    pub confidence: f64,
    pub weight_in_consensus: f64,
}

/// Offline availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineStatus {
    pub model_id: ModelId,
    pub is_available_locally: bool,
    pub cache_size_bytes: u64,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub offline_capability_score: f64,
}

/// Health monitoring events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEvent {
    ModelAvailable(ModelId),
    ModelUnavailable(ModelId),
    PerformanceDegraded {
        model_id: ModelId,
        metric_type: String,
        current_value: f64,
        threshold: f64,
    },
    ModelRecovered(ModelId),
    SystemOverload {
        resource_type: String,
        utilization: f64,
    },
}

/// Model switching events for UI notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSwitchEvent {
    pub previous_model: ModelId,
    pub new_model: ModelId,
    pub reason: SwitchReason,
    pub performance_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchReason {
    Performance,
    LoadBalancing,
    ModelFailure,
    ResourceConstraint,
    UserPreference,
    Maintenance,
}

/// Configuration for model orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    pub performance_thresholds: PerformanceThresholds,
    pub load_balancing_config: LoadBalancingConfig,
    pub consensus_config: ConsensusConfig,
    pub fallback_config: FallbackConfig,
    pub model_switching_config: ModelSwitchingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_latency_ms: f64,
    pub min_accuracy: f64,
    pub max_memory_mb: f64,
    pub max_cpu_percent: f64,
    pub health_check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub max_concurrent_requests: usize,
    pub queue_capacity: usize,
    pub load_balance_interval_secs: u64,
    pub overload_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub min_models_for_consensus: usize,
    pub confidence_threshold: f64,
    pub disagreement_tolerance: f64,
    pub voting_mechanism: VotingMechanism,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingMechanism {
    Majority,
    Weighted,
    ConfidenceBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub offline_cache_duration_days: u64,
    pub grace_period_secs: u64,
    pub fallback_sequence: Vec<ModelId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSwitchingConfig {
    pub switching_latency_target_ms: f64,
    pub cooldown_duration_secs: u64,
    pub hysteresis_factor: f64,
}
