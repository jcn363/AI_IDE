//! # Model Warmup Prediction System
//!
//! This crate implements an advanced predictive warmup system for multi-model orchestration
//! in the Rust AI IDE. It analyzes user behavior patterns, predicts model needs, and
//! proactively warms up models to reduce cold start times and improve responsiveness.
//!
//! ## Architecture
//!
//! The system consists of several key components:
//! - `UsagePatternAnalyzer`: Analyzes historical usage patterns and user behavior
//! - `PredictionEngine`: Machine learning-based prediction of future model needs
//! - `WarmupScheduler`: Intelligent scheduling of model warmup operations
//! - `ResourceManager`: Manages resources allocated to warmup operations
//! - `WarmupQueue`: Priority-based queue for warmup tasks
//! - `PerformancePredictor`: Predicts warmup impact on system performance
//! - `ModelWarmupMetrics`: Tracks warmup effectiveness and prediction accuracy

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

pub mod error;
pub mod types;
pub mod usage_pattern_analyzer;
pub mod prediction_engine;
pub mod warmup_scheduler;
pub mod resource_manager;
pub mod warmup_queue;
pub mod performance_predictor;
pub mod metrics;

// Advanced ML modules
pub mod ml_trainer;
pub mod ml_evaluator;
// Advanced modules
pub mod advanced_patterns;
pub mod benchmark_tools;
pub mod comprehensive_performance_tests;

// Re-exports for public API
pub use error::{WarmupError, Result};
pub use types::*;
pub use usage_pattern_analyzer::UsagePatternAnalyzer;
pub use prediction_engine::PredictionEngine;
pub use warmup_scheduler::WarmupScheduler;
pub use resource_manager::ResourceManager;
pub use warmup_queue::WarmupQueue;
pub use performance_predictor::PerformancePredictor;
pub use metrics::ModelWarmupMetrics;

// Advanced ML modules
pub use ml_trainer::MLModelTrainer;
pub use ml_evaluator::MLModelEvaluator;
pub use advanced_patterns::AdvancedPatternAnalyzer;
pub use benchmark_tools::PerformanceBenchmarker;

// Comprehensive performance testing
pub use comprehensive_performance_tests::*;

/// Main entry point for the model warmup prediction system
#[derive(Debug)]
pub struct ModelWarmupPredictor {
    /// Usage pattern analyzer
    usage_analyzer: UsagePatternAnalyzer,
    /// Prediction engine
    prediction_engine: PredictionEngine,
    /// Warmup scheduler
    scheduler: WarmupScheduler,
    /// Resource manager
    resource_manager: ResourceManager,
    /// Warmup queue
    queue: WarmupQueue,
    /// Performance predictor
    performance_predictor: PerformancePredictor,
    /// Metrics collector
    metrics: ModelWarmupMetrics,
}

impl ModelWarmupPredictor {
    /// Create a new model warmup predictor with default configuration
    pub async fn new() -> Result<Self> {
        let config = WarmupConfig::default();

        Self::with_config(config).await
    }

    /// Create a new model warmup predictor with custom configuration
    pub async fn with_config(config: WarmupConfig) -> Result<Self> {
        Ok(Self {
            usage_analyzer: UsagePatternAnalyzer::new(config.clone()).await?,
            prediction_engine: PredictionEngine::new(config.clone()).await?,
            scheduler: WarmupScheduler::new(config.clone()).await?,
            resource_manager: ResourceManager::new(config.clone()).await?,
            queue: WarmupQueue::new(config.clone()).await?,
            performance_predictor: PerformancePredictor::new(config.clone()).await?,
            metrics: ModelWarmupMetrics::new(),
        })
    }

    /// Analyze current request and predict models to warm up
    pub async fn predict_and_warm(&self, request: &WarmupRequest) -> Result<WarmupPrediction> {
        // Record usage pattern for learning
        self.usage_analyzer.record_usage(request).await?;

        // Generate predictions
        let predictions = self.prediction_engine.predict_models(request).await?;

        // Check resource availability
        let available_resources = self.resource_manager.get_available_resources().await?;

        // Schedule warmup operations
        let schedule = self.scheduler.schedule_warmup(&predictions, &available_resources).await?;

        // Assess performance impact
        let performance_impact = self.performance_predictor.assess_impact(&schedule).await?;

        // Queue warmup tasks
        for task in &schedule.tasks {
            self.queue.enqueue_task(task.clone()).await?;
        }

        let result = WarmupPrediction {
            predicted_models: predictions,
            schedule,
            performance_impact,
            confidence_score: 0.85, // Placeholder - would be calculated
        };

        // Record metrics
        self.metrics.record_prediction(&result).await?;

        Ok(result)
    }

    /// Get current system metrics
    pub fn get_metrics(&self) -> &ModelWarmupMetrics {
        &self.metrics
    }

    /// Update configuration
    pub async fn update_config(&mut self, config: WarmupConfig) -> Result<()> {
        self.usage_analyzer.update_config(config.clone()).await?;
        self.prediction_engine.update_config(config.clone()).await?;
        self.scheduler.update_config(config.clone()).await?;
        self.resource_manager.update_config(config.clone()).await?;
        self.queue.update_config(config.clone()).await?;
        self.performance_predictor.update_config(config).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_warmup_predictor_creation() {
        let predictor = ModelWarmupPredictor::new().await.unwrap();
        assert!(predictor.get_metrics().total_predictions() >= 0);
    }
}