//! # Phase 9: AI Predictive Development Assistant
//!
//! Advanced AI-powered development assistance including:
//! - Context-aware code suggestions based on project patterns
//! - Proactive refactoring recommendations
//! - Intent prediction and next-action suggestions
//! - Semantic understanding through LSP integration
//! - AI-powered pattern recognition and suggestion scoring
//! - Memory-efficient prediction models
//! - Real-time analysis without blocking user interface
//! - Multi-language support

#[macro_use]
extern crate tracing;

// Import external dependencies
pub use serde::{Deserialize, Serialize};
pub use tokio;
pub use uuid;

// Import internal dependencies
pub use rust_ai_ide_common::validation::TauriInputSanitizer;
pub use rust_ai_ide_errors::IDError;
pub use rust_ai_ide_types::{Language, Position, Range, TextDocument};

// Re-export main modules
pub mod predictive_development;
pub use predictive_development::*;

// Explicit re-exports for easier module usage
pub use predictive_development::{
    AnalysisMode, CacheEvictionPolicy, CodeSuggestion, IntentPrediction, PerformanceMetrics,
    PredictionContext, PredictionSettings, PredictiveDevelopmentEngine, RefactoringRecommendation,
};

/// Errors specific to predictive development functionality
#[derive(Debug, thiserror::Error)]
pub enum PredictiveError {
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("AI model error: {0}")]
    ModelError(String),

    #[error("Prediction failed: {0}")]
    PredictionFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
}

/// Result type alias for predictive operations
pub type PredictiveResult<T> = Result<T, PredictiveError>;

/// Convert general IDEError to PredictiveError
impl From<rust_ai_ide_errors::IDError> for PredictiveError {
    fn from(error: rust_ai_ide_errors::IDError) -> Self {
        PredictiveError::AnalysisFailed(error.to_string())
    }
}

/// Performance metrics tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTracker {
    pub total_predictions: u64,
    pub successful_predictions: u64,
    pub average_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_bytes: u64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self {
            total_predictions: 0,
            successful_predictions: 0,
            average_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            memory_usage_bytes: 0,
            last_update: chrono::Utc::now(),
        }
    }
}

impl PerformanceTracker {
    /// Record a prediction result
    pub fn record_prediction(&mut self, successful: bool, response_time_ms: f64) {
        self.total_predictions += 1;
        if successful {
            self.successful_predictions += 1;
        }

        // Update rolling average
        self.average_response_time_ms = (self.average_response_time_ms
            * (self.total_predictions - 1) as f64
            + response_time_ms)
            / self.total_predictions as f64;

        self.last_update = chrono::Utc::now();
    }

    /// Get success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_predictions == 0 {
            0.0
        } else {
            (self.successful_predictions as f64 / self.total_predictions as f64) * 100.0
        }
    }

    /// Get cache statistics
    pub fn cache_efficiency(&self) -> f64 {
        self.cache_hit_rate
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, usage_bytes: u64) {
        self.memory_usage_bytes = usage_bytes;
    }
}

/// Thread-safe shared performance tracker
pub type SharedPerformanceTracker = Arc<tokio::sync::RwLock<PerformanceTracker>>;

/// Create a new shared performance tracker
pub fn create_performance_tracker() -> SharedPerformanceTracker {
    Arc::new(tokio::sync::RwLock::new(PerformanceTracker::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_tracker_basic() {
        let tracker = PerformanceTracker::default();

        assert_eq!(tracker.total_predictions, 0);
        assert_eq!(tracker.successful_predictions, 0);
        assert_eq!(tracker.average_response_time_ms, 0.0);
        assert_eq!(tracker.success_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_performance_tracker_recording() {
        let mut tracker = PerformanceTracker::default();

        tracker.record_prediction(true, 100.0);
        tracker.record_prediction(false, 200.0);
        tracker.record_prediction(true, 150.0);

        assert_eq!(tracker.total_predictions, 3);
        assert_eq!(tracker.successful_predictions, 2);

        let avg_response_time = (100.0 + 200.0 + 150.0) / 3.0;
        assert!((tracker.average_response_time_ms - avg_response_time).abs() < 0.01);

        let expected_success_rate = (2.0 / 3.0) * 100.0;
        assert!((tracker.success_rate() - expected_success_rate).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_shared_performance_tracker() {
        let tracker = create_performance_tracker();
        {
            let mut lock = tracker.write().await;
            lock.record_prediction(true, 100.0);
        }

        let lock = tracker.read().await;
        assert_eq!(lock.total_predictions, 1);
        assert_eq!(lock.successful_predictions, 1);
    }
}
