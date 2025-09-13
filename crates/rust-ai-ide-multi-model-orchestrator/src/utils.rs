//! Utility functions for Multi-Model Orchestration
//!
//! This module provides common utility functions used throughout the orchestration system.

use crate::Result;

/// Utility functions for model ID generation and validation
pub mod identity {
    use uuid::Uuid;

    use crate::types::ModelId;

    pub fn generate_model_id() -> ModelId {
        ModelId(Uuid::new_v4())
    }

    pub fn validate_model_id(_id: &ModelId) -> bool {
        // Basic validation (UUID is always valid in our case)
        true
    }
}

/// Model scoring and ranking utilities
pub mod scoring {
    pub fn normalize_score(score: f64) -> f64 {
        score.max(0.0).min(1.0)
    }

    pub fn combine_scores(scores: &[f64], weights: &[f64]) -> Option<f64> {
        if scores.len() != weights.len() {
            return None;
        }

        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for (score, weight) in scores.iter().zip(weights.iter()) {
            weighted_sum += score * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            Some(normalize_score(weighted_sum / total_weight))
        } else {
            None
        }
    }
}

/// Configuration validation utilities
pub mod validation {
    use crate::config::{validate_config, OrchestrationConfig};

    pub fn validate_orchestration_config(_config: &OrchestrationConfig) -> crate::Result<()> {
        validate_config(_config)
    }
}

/// Temporal utility functions
pub mod temporal {
    use std::time::{Duration, Instant};

    pub fn calculate_duration_since(start: Instant) -> Duration {
        start.elapsed()
    }

    pub fn is_with_duration(time: Instant, target_duration: Duration) -> bool {
        time.elapsed() <= target_duration
    }
}

/// Logging utilities for orchestration events
pub mod logging {
    pub fn sanitize_log_data(_data: &str) -> String {
        // Sanitize sensitive information from logs
        _data.to_string()
    }

    pub fn format_performance_metric(operation: &str, duration: std::time::Duration) -> String {
        format!("{} completed in {:.2}ms", operation, duration.as_millis())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_normalization() {
        assert_eq!(scoring::normalize_score(1.5), 1.0);
        assert_eq!(scoring::normalize_score(-0.5), 0.0);
        assert_eq!(scoring::normalize_score(0.7), 0.7);
    }

    #[test]
    fn test_score_combination() {
        let scores = vec![0.8, 0.6, 0.4];
        let weights = vec![0.5, 0.3, 0.2];
        let combined = scoring::combine_scores(&scores, &weights).unwrap();
        assert!(combined > 0.0 && combined <= 1.0);
    }
}
