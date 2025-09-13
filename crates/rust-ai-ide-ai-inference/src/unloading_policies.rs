//! # Unloading Policies Module
//!
//! Implementation of various model unloading policies for resource management.

use std::collections::HashMap;

use crate::model_handle::ModelHandle;
use crate::resource_monitor::{MemoryPressure, ResourceSummary, SystemMonitor};
use crate::resource_types::{UnloadingPolicy, BYTES_PER_MB};

/// Policy evaluator that determines which models should be unloaded
#[derive(Debug, Clone)]
pub struct PolicyEvaluator {
    system_monitor: SystemMonitor,
}

impl PolicyEvaluator {
    /// Create a new policy evaluator
    pub fn new(system_monitor: SystemMonitor) -> Self {
        Self { system_monitor }
    }

    /// Evaluate models against a policy and return IDs of models to unload
    pub async fn evaluate_models(
        &self,
        models: &HashMap<String, ModelHandle>,
        policy: &UnloadingPolicy,
    ) -> Vec<String> {
        match policy {
            UnloadingPolicy::LRU { max_age_hours } => self.evaluate_lru_policy(models, *max_age_hours),
            UnloadingPolicy::MemoryThreshold { max_memory_gb } =>
                self.evaluate_memory_threshold_policy(models, *max_memory_gb)
                    .await,
            UnloadingPolicy::TimeBased { max_age_hours } => self.evaluate_time_based_policy(models, *max_age_hours),
            UnloadingPolicy::Hybrid {
                max_age_hours,
                max_memory_gb,
            } =>
                self.evaluate_hybrid_policy(models, *max_age_hours, *max_memory_gb)
                    .await,
        }
    }

    /// Evaluate LRU policy - unload least recently used models
    fn evaluate_lru_policy(&self, models: &HashMap<String, ModelHandle>, max_age_hours: u32) -> Vec<String> {
        models
            .iter()
            .filter(|(_, handle)| handle.should_unload_lru(max_age_hours))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Evaluate memory threshold policy
    async fn evaluate_memory_threshold_policy(
        &self,
        models: &HashMap<String, ModelHandle>,
        max_memory_gb: f64,
    ) -> Vec<String> {
        let summary = self.system_monitor.get_resource_summary().await;

        if summary.is_above_threshold(max_memory_gb) {
            self.select_models_for_memory_unloading(models, &summary)
        } else {
            Vec::new()
        }
    }

    /// Evaluate time-based policy - unload old models
    fn evaluate_time_based_policy(&self, models: &HashMap<String, ModelHandle>, max_age_hours: u32) -> Vec<String> {
        models
            .iter()
            .filter(|(_, handle)| handle.should_unload_time_based(max_age_hours))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Evaluate hybrid policy (LRU + Memory threshold)
    async fn evaluate_hybrid_policy(
        &self,
        models: &HashMap<String, ModelHandle>,
        max_age_hours: u32,
        max_memory_gb: f64,
    ) -> Vec<String> {
        let summary = self.system_monitor.get_resource_summary().await;

        if !summary.is_above_threshold(max_memory_gb) {
            return Vec::new();
        }

        // Find models that are both old and memory-intensive
        models
            .iter()
            .filter(|(_, handle)| handle.should_unload_lru(max_age_hours) || self.is_memory_intensive(handle, &summary))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Select models for memory-based unloading based on priority
    fn select_models_for_memory_unloading(
        &self,
        models: &HashMap<String, ModelHandle>,
        _summary: &ResourceSummary,
    ) -> Vec<String> {
        let mut candidates: Vec<_> = models.iter().collect();

        // Sort by priority: older models first, then larger models
        candidates.sort_by(|(_, a), (_, b)| {
            // Primary sort: by last accessed (older first)
            let access_cmp = a
                .resource_usage
                .last_accessed
                .cmp(&b.resource_usage.last_accessed);

            // Secondary sort: by memory usage (larger first)
            if access_cmp == std::cmp::Ordering::Equal {
                b.resource_usage
                    .memory_usage_bytes
                    .cmp(&a.resource_usage.memory_usage_bytes)
            } else {
                access_cmp
            }
        });

        candidates.into_iter().map(|(id, _)| id.clone()).collect()
    }

    /// Check if a model is memory intensive relative to system
    fn is_memory_intensive(&self, handle: &ModelHandle, summary: &ResourceSummary) -> bool {
        let memory_percentage =
            (handle.resource_usage.memory_usage_bytes as f64 / summary.total_memory_bytes as f64) * 100.0;
        memory_percentage >= 20.0 // Models using >= 20% of system memory
    }

    /// Get detailed unloading recommendations
    pub async fn get_unloading_recommendations(
        &self,
        models: &HashMap<String, ModelHandle>,
        policy: &UnloadingPolicy,
    ) -> UnloadingRecommendation {
        let models_to_unload = self.evaluate_models(models, policy).await;

        if models_to_unload.is_empty() {
            return UnloadingRecommendation::None {
                reason: "No models meet unloading criteria".to_string(),
            };
        }

        let memory_freed: u64 = models_to_unload
            .iter()
            .filter_map(|id| models.get(id))
            .map(|handle| handle.resource_usage.memory_usage_bytes)
            .sum();

        let memory_freed_mb = memory_freed as f64 / BYTES_PER_MB;

        UnloadingRecommendation::Unload {
            model_ids: models_to_unload,
            memory_freed_mb,
            reason: format!("Policy {:?} triggered unloading", policy),
            urgency: self.calculate_urgency(policy).await,
        }
    }

    /// Calculate unloading urgency based on system state
    async fn calculate_urgency(&self, _policy: &UnloadingPolicy) -> UnloadingUrgency {
        let summary = self.system_monitor.get_resource_summary().await;

        match summary.pressure_level() {
            MemoryPressure::Critical => UnloadingUrgency::High,
            MemoryPressure::High => UnloadingUrgency::Medium,
            _ => UnloadingUrgency::Low,
        }
    }
}

/// Unloading recommendation
#[derive(Debug, Clone)]
pub enum UnloadingRecommendation {
    /// No models need to be unloaded
    None { reason: String },
    /// Models should be unloaded
    Unload {
        model_ids:       Vec<String>,
        memory_freed_mb: f64,
        reason:          String,
        urgency:         UnloadingUrgency,
    },
}

impl UnloadingRecommendation {
    /// Check if unloading is recommended
    pub fn should_unload(&self) -> bool {
        matches!(self, UnloadingRecommendation::Unload { .. })
    }

    /// Get the list of models to unload (empty if none)
    pub fn models_to_unload(&self) -> &[String] {
        match self {
            UnloadingRecommendation::Unload { model_ids, .. } => model_ids,
            _ => &[],
        }
    }

    /// Get memory that would be freed in MB
    pub fn memory_freed_mb(&self) -> f64 {
        match self {
            UnloadingRecommendation::Unload {
                memory_freed_mb, ..
            } => *memory_freed_mb,
            _ => 0.0,
        }
    }
}

/// Urgency levels for unloading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnloadingUrgency {
    Low,
    Medium,
    High,
}

impl UnloadingUrgency {
    /// Get urgency factor (higher = more urgent)
    pub fn factor(&self) -> f32 {
        match self {
            UnloadingUrgency::Low => 0.3,
            UnloadingUrgency::Medium => 0.6,
            UnloadingUrgency::High => 1.0,
        }
    }

    /// Check if unloading should be prioritized
    pub fn is_prioritized(&self) -> bool {
        matches!(self, UnloadingUrgency::High)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_handle::ModelHandleBuilder;
    use crate::resource_types::UnloadingPolicy;

    #[tokio::test]
    async fn test_lru_policy_evaluation() {
        let monitor = SystemMonitor::new();
        let evaluator = PolicyEvaluator::new(monitor);

        let mut models = HashMap::new();

        // Create a model handle for testing
        let handle = ModelHandleBuilder::new()
            .id("test_model")
            .path("/tmp/test")
            .size(ModelSize::Large)
            .model_type(crate::resource_types::ModelType::CodeLlama)
            .build()
            .unwrap();

        models.insert("test_model".to_string(), handle);

        // Test LRU policy with 1 hour threshold - should not unload fresh models
        let policy = UnloadingPolicy::LRU { max_age_hours: 1 };
        let to_unload = evaluator.evaluate_models(&models, &policy).await;

        assert!(to_unload.is_empty());
    }

    #[test]
    fn test_unloading_recommendation_methods() {
        let recommendation = UnloadingRecommendation::Unload {
            model_ids:       vec!["test1".to_string(), "test2".to_string()],
            memory_freed_mb: 1024.0,
            reason:          "Test reason".to_string(),
            urgency:         UnloadingUrgency::Medium,
        };

        assert!(recommendation.should_unload());
        assert_eq!(recommendation.models_to_unload().len(), 2);
        assert_eq!(recommendation.memory_freed_mb(), 1024.0);

        let no_unload = UnloadingRecommendation::None {
            reason: "No models to unload".to_string(),
        };

        assert!(!no_unload.should_unload());
        assert_eq!(no_unload.models_to_unload().len(), 0);
        assert_eq!(no_unload.memory_freed_mb(), 0.0);
    }

    #[test]
    fn test_unloading_urgency() {
        assert_eq!(UnloadingUrgency::High.factor(), 1.0);
        assert!(UnloadingUrgency::High.is_prioritized());
        assert!(!UnloadingUrgency::Low.is_prioritized());
    }
}
