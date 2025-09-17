//! Performance-Based Model Selector
//!
//! This module implements intelligent model selection based on real-time performance metrics,
//! load patterns, and predictive quality intelligence.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::config::{validate_config, OrchestrationConfig};
use crate::types::{
    Complexity, ModelId, ModelInfo, ModelMetrics, ModelRecommendation, ModelSwitchingConfig,
    ModelTask, PerformanceThresholds, RequestContext, RequestPriority,
};
use crate::{OrchestrationError, Result};

/// Performance tracker for collecting and analyzing model metrics
#[derive(Debug)]
pub struct ModelPerformanceTracker {
    metrics_store: Arc<RwLock<HashMap<ModelId, ModelMetrics>>>,
    config: OrchestrationConfig,
}

impl ModelPerformanceTracker {
    pub async fn update_metrics(&self, model_id: ModelId, metrics: ModelMetrics) -> Result<()> {
        let mut store = self.metrics_store.write().await;
        store.insert(model_id, metrics);
        Ok(())
    }

    pub async fn get_metrics(&self, model_id: &ModelId) -> Option<ModelMetrics> {
        let store = self.metrics_store.read().await;
        store.get(model_id).cloned()
    }

    pub async fn get_all_metrics(&self) -> HashMap<ModelId, ModelMetrics> {
        let store = self.metrics_store.read().await;
        store.clone()
    }
}

/// Load predictor for estimating future model load patterns
#[derive(Debug)]
pub struct LoadPredictor {
    historical_loads: Arc<RwLock<HashMap<ModelId, Vec<f64>>>>,
    prediction_horizon: usize,
}

impl LoadPredictor {
    pub async fn predict_load(&self, model_id: &ModelId) -> f64 {
        let historical_loads = self.historical_loads.read().await;
        if let Some(history) = historical_loads.get(model_id) {
            if history.len() < 2 {
                return 0.5; // Default moderate load
            }

            // Simple exponential moving average prediction
            let recent = history.iter().rev().take(3).collect::<Vec<_>>();
            let ema = recent
                .iter()
                .enumerate()
                .map(|(i, &load)| load * (0.5f64.powi(i as i32)))
                .sum::<f64>();

            ema.min(1.0).max(0.0) // Clamp to [0, 1]
        } else {
            0.5 // Default moderate load
        }
    }

    pub async fn record_load(&self, model_id: ModelId, load: f64) -> Result<()> {
        let mut historical_loads = self.historical_loads.write().await;
        let history = historical_loads.entry(model_id).or_insert_with(Vec::new);
        history.push(load);
        // Keep last N measurements
        if history.len() > 100 {
            history.remove(0);
        }
        Ok(())
    }
}

/// Resource usage monitor for tracking system resource consumption
#[derive(Debug)]
pub struct ResourceUsageMonitor {
    system_metrics: Arc<RwLock<SystemResourceMetrics>>,
}

#[derive(Debug, Clone)]
pub struct SystemResourceMetrics {
    pub total_memory_mb: f64,
    pub available_memory_mb: f64,
    pub cpu_usage_percent: f64,
    pub gpu_memory_usage: HashMap<String, f64>,
}

impl ResourceUsageMonitor {
    pub async fn get_system_resources(&self) -> SystemResourceMetrics {
        let metrics = self.system_metrics.read().await;
        metrics.clone()
    }

    pub async fn update_system_resources(&self, metrics: SystemResourceMetrics) -> Result<()> {
        let mut system_metrics = self.system_metrics.write().await;
        *system_metrics = metrics;
        Ok(())
    }
}

/// Model warmer for pre-loading frequently used models
#[derive(Debug)]
pub struct ModelWarmer {
    cache: HashMap<ModelId, std::time::Instant>,
    max_cache_size: usize,
}

impl ModelWarmer {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size,
        }
    }

    pub fn warm_model(&mut self, model_id: ModelId) -> bool {
        if self.cache.len() >= self.max_cache_size {
            // Remove least recently used
            let mut entries: Vec<_> = self.cache.iter().collect();
            entries.sort_by(|a, b| a.1.cmp(b.1));
            if let Some((&key, _)) = entries.first() {
                self.cache.remove(&key);
            }
        }

        self.cache.insert(model_id, std::time::Instant::now());
        true
    }

    pub fn get_warmer_models(&self, limit: usize) -> Vec<ModelId> {
        let mut entries: Vec<_> = self.cache.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1)); // Most recent first
        entries
            .iter()
            .take(limit)
            .map(|(&model_id, _)| model_id)
            .collect()
    }
}

/// Core model selection engine
#[derive(Debug)]
pub struct ModelSelectionEngine {
    available_models: Arc<RwLock<Vec<ModelInfo>>>,
    switching_cooldowns: Arc<RwLock<HashMap<ModelId, std::time::Instant>>>,
    selection_history: Arc<RwLock<Vec<ModelRecommendation>>>,
    config: OrchestrationConfig,
}

impl ModelSelectionEngine {
    pub async fn select_model(
        &self,
        context: &RequestContext,
        performance_tracker: &ModelPerformanceTracker,
        load_predictor: &LoadPredictor,
        resource_monitor: &ResourceUsageMonitor,
    ) -> Result<ModelRecommendation> {
        let models = self.available_models.read().await;
        let cooldowns = self.switching_cooldowns.read().await;

        let mut candidates = Vec::new();

        for model_info in models.iter() {
            // Check if model is in cooldown period
            if let Some(last_switch) = cooldowns.get(&model_info.id) {
                if last_switch.elapsed()
                    < std::time::Duration::from_secs(
                        self.config.model_switching_config.cooldown_duration_secs,
                    )
                {
                    continue; // Skip models in cooldown
                }
            }

            // Check basic compatibility
            if !self.is_model_compatible(&model_info.capability, context) {
                continue;
            }

            // Calculate selection score
            let score = self
                .calculate_selection_score(
                    &model_info.id,
                    context,
                    performance_tracker,
                    load_predictor,
                    resource_monitor,
                )
                .await;

            candidates.push((model_info.id, score));
        }

        if candidates.is_empty() {
            return Err(OrchestrationError::ModelSelectionError(
                "No compatible models available".to_string(),
            ));
        }

        // Sort by score (highest first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (best_model, confidence_score) = candidates[0];

        let recommendation = ModelRecommendation {
            model_id: best_model,
            confidence_score,
            expected_latency: std::time::Duration::from_millis(100), // Placeholder
            resource_cost_estimate: 1.0 - confidence_score,          // Inverse relationship
            selection_reason: self.generate_selection_reason(&best_model, context),
        };

        // Update selection history
        let mut history = self.selection_history.write().await;
        history.push(recommendation.clone());
        // Keep last 100 selections
        if history.len() > 100 {
            history.remove(0);
        }

        Ok(recommendation)
    }

    fn is_model_compatible(
        &self,
        capability: &crate::types::ModelCapability,
        request: &RequestContext,
    ) -> bool {
        // Check task compatibility
        if !capability.supported_tasks.contains(&request.task_type) {
            return false;
        }

        // Check input length limits
        if request.input_length > capability.max_context_length {
            return false;
        }

        // Check complexity requirements
        match request.task_type {
            ModelTask::Analysis | ModelTask::Refactoring
                if !capability.supported_languages.contains(&"rust".to_string()) =>
            {
                return false;
            }
            ModelTask::Chat | ModelTask::Generation => {
                // These tasks can be handled by general models
            }
            _ => {}
        }

        true
    }

    async fn calculate_selection_score(
        &self,
        model_id: &ModelId,
        context: &RequestContext,
        performance_tracker: &ModelPerformanceTracker,
        load_predictor: &LoadPredictor,
        resource_monitor: &ResourceUsageMonitor,
    ) -> f64 {
        let mut score = 0.0;

        // Performance component (40% weight)
        if let Some(metrics) = performance_tracker.get_metrics(model_id).await {
            let latency_score = if metrics.latency_ms < 500.0 { 1.0 } else { 0.5 };
            let accuracy_score = metrics.accuracy_score;
            score += (latency_score * 0.6 + accuracy_score * 0.4) * 0.4;
        }

        // Load component (30% weight)
        let predicted_load = load_predictor.predict_load(model_id).await;
        let load_score = 1.0 - predicted_load; // Lower load = higher score
        score += load_score * 0.3;

        // Resource availability component (20% weight)
        let system_resources = resource_monitor.get_system_resources().await;
        let resource_score = if system_resources.available_memory_mb > 2048.0 {
            1.0
        } else {
            0.8
        };
        score += resource_score * 0.2;

        // Priority component (10% weight)
        let priority_score = match context.priority {
            RequestPriority::Critical => 1.5,
            RequestPriority::High => 1.2,
            RequestPriority::Medium => 1.0,
            RequestPriority::Low => 0.8,
        };
        score += (priority_score - 1.0) * 0.1;

        // Apply hysteresis to prevent thrashing
        score *= self.config.model_switching_config.hysteresis_factor;

        score.max(0.0).min(1.0) // Clamp to [0, 1]
    }

    fn generate_selection_reason(&self, model_id: &ModelId, context: &RequestContext) -> String {
        format!(
            "Selected model {} for {} task with {} complexity and {} priority",
            model_id.0,
            match context.task_type {
                ModelTask::Completion => "code completion",
                ModelTask::Chat => "chat",
                ModelTask::Classification => "classification",
                ModelTask::Generation => "code generation",
                ModelTask::Analysis => "analysis",
                ModelTask::Refactoring => "refactoring",
                ModelTask::Translation => "translation",
            },
            match context.expected_complexity {
                Complexity::Simple => "simple",
                Complexity::Medium => "medium",
                Complexity::Complex => "complex",
            },
            match context.priority {
                RequestPriority::Critical => "critical",
                RequestPriority::High => "high",
                RequestPriority::Medium => "medium",
                RequestPriority::Low => "low",
            }
        )
    }
}

/// Main Performance-Based Model Selector
#[derive(Debug)]
pub struct PerformanceBasedModelSelector {
    pub performance_tracker: Arc<ModelPerformanceTracker>,
    pub load_predictor: Arc<LoadPredictor>,
    pub resource_monitor: Arc<ResourceUsageMonitor>,
    pub model_warmer: Arc<ModelWarmer>,
    pub selection_engine: Arc<ModelSelectionEngine>,
}

impl PerformanceBasedModelSelector {
    pub fn new(config: OrchestrationConfig) -> Result<Self> {
        validate_config(&config)?;

        Ok(Self {
            performance_tracker: Arc::new(ModelPerformanceTracker {
                metrics_store: Arc::new(RwLock::new(HashMap::new())),
                config: config.clone(),
            }),
            load_predictor: Arc::new(LoadPredictor {
                historical_loads: Arc::new(RwLock::new(HashMap::new())),
                prediction_horizon: 10,
            }),
            resource_monitor: Arc::new(ResourceUsageMonitor {
                system_metrics: Arc::new(RwLock::new(SystemResourceMetrics {
                    total_memory_mb: 8192.0,     // Default 8GB
                    available_memory_mb: 4096.0, // Default 4GB available
                    cpu_usage_percent: 50.0,     // Default 50% CPU usage
                    gpu_memory_usage: HashMap::new(),
                })),
            }),
            model_warmer: Arc::new(ModelWarmer::new(5)), // Warm up to 5 models
            selection_engine: Arc::new(ModelSelectionEngine {
                available_models: Arc::new(RwLock::new(Vec::new())),
                switching_cooldowns: Arc::new(RwLock::new(HashMap::new())),
                selection_history: Arc::new(RwLock::new(Vec::new())),
                config,
            }),
        })
    }

    pub async fn select_model(&self, context: &RequestContext) -> Result<ModelRecommendation> {
        self.selection_engine
            .select_model(
                context,
                &self.performance_tracker,
                &self.load_predictor,
                &self.resource_monitor,
            )
            .await
    }

    pub async fn warmup_models(&self, models: Vec<ModelId>) {
        for model_id in models {
            self.model_warmer.warm_model(model_id);
        }
    }

    pub async fn update_model_metrics(
        &self,
        model_id: ModelId,
        metrics: ModelMetrics,
    ) -> Result<()> {
        self.performance_tracker
            .update_metrics(model_id, metrics)
            .await?;

        // Also update load predictor
        let load_factor = metrics.cpu_usage_percent / 100.0; // Convert to 0-1 scale
        self.load_predictor
            .record_load(model_id, load_factor)
            .await?;

        Ok(())
    }

    pub async fn add_model(&self, model_info: ModelInfo) -> Result<()> {
        let mut models = self.selection_engine.available_models.write().await;
        models.push(model_info);

        // Warm up the new model
        self.warmup_models(vec![model_info.id]).await;

        Ok(())
    }

    pub async fn get_selection_history(&self) -> Vec<ModelRecommendation> {
        let history = self.selection_engine.selection_history.read().await;
        history.clone()
    }

    /// Record a model switch for cooldown tracking
    pub async fn record_model_switch(&self, model_id: ModelId) {
        let mut cooldowns = self.selection_engine.switching_cooldowns.write().await;
        cooldowns.insert(model_id, std::time::Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrchestrationConfigBuilder;

    #[tokio::test]
    async fn test_model_selection() {
        let config = OrchestrationConfigBuilder::new().build();
        let selector = PerformanceBasedModelSelector::new(config).unwrap();

        // Add a test model
        let test_model_id = ModelId::new();
        let model_info = ModelInfo {
            id: test_model_id,
            name: "test-model".to_string(),
            version: "1.0.0".to_string(),
            capability: crate::types::ModelCapability {
                supported_tasks: vec![ModelTask::Completion],
                max_context_length: 2048,
                supported_languages: vec!["rust".to_string()],
                quantization_level: None,
                hardware_acceleration: vec![],
            },
            status: crate::types::ModelStatus::Available,
            metrics: ModelMetrics::new(test_model_id),
        };

        selector.add_model(model_info).await.unwrap();

        // Update metrics
        let mut metrics = ModelMetrics::new(test_model_id);
        metrics.accuracy_score = 0.9;
        metrics.latency_ms = 200.0;
        selector
            .update_model_metrics(test_model_id, metrics)
            .await
            .unwrap();

        // Test selection
        let context = RequestContext {
            task_type: ModelTask::Completion,
            input_length: 100,
            priority: RequestPriority::Medium,
            expected_complexity: Complexity::Medium,
            acceptable_latency: std::time::Duration::from_millis(500),
            preferred_hardware: None,
        };

        let recommendation = selector.select_model(&context).await.unwrap();
        assert_eq!(recommendation.model_id, test_model_id);
        assert!(recommendation.confidence_score > 0.0);
    }
}
