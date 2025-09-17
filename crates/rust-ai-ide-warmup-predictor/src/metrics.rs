//! Metrics collection and tracking for warmup prediction system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use crate::error::{Result, WarmupError};
use crate::types::{ModelId, PredictionAccuracy, WarmupPrediction};

#[derive(Debug, Clone)]
pub struct ModelWarmupMetrics {
    predictions_made: u64,
    accurate_predictions: u64,
    total_warmup_time: Duration,
    avg_prediction_confidence: f64,
    model_metrics: HashMap<ModelId, ModelMetrics>,
    last_updated: Instant,
}

#[derive(Debug, Clone)]
pub struct ModelMetrics {
    predictions: u64,
    accurate: u64,
    avg_confidence: f64,
    total_warmup_time: Duration,
    cache_hit_rate: f64,
}

impl ModelWarmupMetrics {
    pub fn new() -> Self {
        Self {
            predictions_made: 0,
            accurate_predictions: 0,
            total_warmup_time: Duration::from_secs(0),
            avg_prediction_confidence: 0.0,
            model_metrics: HashMap::new(),
            last_updated: Instant::now(),
        }
    }

    pub async fn record_prediction(&mut self, prediction: &WarmupPrediction) -> Result<()> {
        self.predictions_made += prediction.predicted_models.len() as u64;

        let total_confidence: f64 = prediction.predicted_models.iter()
            .map(|p| p.confidence_score)
            .sum();

        if !prediction.predicted_models.is_empty() {
            self.avg_prediction_confidence = total_confidence / prediction.predicted_models.len() as f64;
        }

        self.last_updated = Instant::now();

        // Update model-specific metrics
        for model_prediction in &prediction.predicted_models {
            let metrics = self.model_metrics.entry(model_prediction.model_id)
                .or_insert_with(|| ModelMetrics {
                    predictions: 0,
                    accurate: 0,
                    avg_confidence: 0.0,
                    total_warmup_time: Duration::from_secs(0),
                    cache_hit_rate: 0.0,
                });

            metrics.predictions += 1;
            metrics.avg_confidence = (metrics.avg_confidence * (metrics.predictions - 1) as f64 + model_prediction.confidence_score) / metrics.predictions as f64;
        }

        Ok(())
    }

    pub fn total_predictions(&self) -> u64 {
        self.predictions_made
    }

    pub fn accuracy_rate(&self) -> f64 {
        if self.predictions_made == 0 {
            0.0
        } else {
            self.accurate_predictions as f64 / self.predictions_made as f64
        }
    }

    pub fn avg_warmup_time(&self) -> Duration {
        if self.predictions_made == 0 {
            Duration::from_secs(0)
        } else {
            self.total_warmup_time / self.predictions_made as u32
        }
    }

    pub fn get_model_metrics(&self, model_id: &ModelId) -> Option<&ModelMetrics> {
        self.model_metrics.get(model_id)
    }

    pub fn get_all_model_metrics(&self) -> &HashMap<ModelId, ModelMetrics> {
        &self.model_metrics
    }

    pub fn record_warmup_time(&mut self, model_id: &ModelId, duration: Duration) {
        self.total_warmup_time += duration;

        if let Some(metrics) = self.model_metrics.get_mut(model_id) {
            metrics.total_warmup_time += duration;
        }
    }

    pub fn record_accuracy(&mut self, model_id: &ModelId, accurate: bool) {
        if accurate {
            self.accurate_predictions += 1;

            if let Some(metrics) = self.model_metrics.get_mut(model_id) {
                metrics.accurate += 1;
            }
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for ModelWarmupMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ModelPrediction;

    #[tokio::test]
    async fn test_metrics_recording() {
        let mut metrics = ModelWarmupMetrics::new();

        let prediction = WarmupPrediction {
            predicted_models: vec![
                ModelPrediction {
                    model_id: ModelId::new(),
                    confidence_score: 0.8,
                    usage_probability: 0.7,
                    time_until_needed: Duration::from_secs(30),
                    reasoning: vec!["Test".to_string()],
                }
            ],
            schedule: crate::types::WarmupSchedule {
                tasks: vec![],
                total_estimated_time: Duration::from_secs(10),
                resource_requirements: Default::default(),
                priority: crate::types::RequestPriority::High,
            },
            performance_impact: crate::types::PerformanceImpact {
                cpu_impact_percent: 5.0,
                memory_impact_mb: 100,
                network_impact_mbps: 1.0,
                latency_increase_ms: 10.0,
                responsiveness_impact: 0.05,
                is_acceptable: true,
            },
            confidence_score: 0.8,
        };

        metrics.record_prediction(&prediction).await.unwrap();

        assert_eq!(metrics.total_predictions(), 1);
        assert_eq!(metrics.avg_prediction_confidence, 0.8);
    }

    #[test]
    fn test_accuracy_calculation() {
        let mut metrics = ModelWarmupMetrics::new();
        assert_eq!(metrics.accuracy_rate(), 0.0);

        let model_id = ModelId::new();
        metrics.record_accuracy(&model_id, true);
        assert_eq!(metrics.accuracy_rate(), 1.0);

        metrics.record_accuracy(&model_id, false);
        assert_eq!(metrics.accuracy_rate(), 0.5);
    }
}