//! Metrics collection and performance monitoring for multi-modal AI
//!
//! This module provides metrics collection, performance monitoring, and analytics
//! for the multi-modal AI processing system.

use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Metrics collector for multi-modal AI operations
pub struct MetricsCollector {
    /// Request count metrics
    request_count: Arc<Mutex<u64>>,
    /// Total processing time
    total_processing_time: Arc<Mutex<u64>>,
    /// Average confidence scores
    confidence_scores: Arc<Mutex<Vec<f32>>>,
    /// Error count
    error_count: Arc<Mutex<u64>>,
    /// Cache for result caching
    result_cache: Cache<String, serde_json::Value>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    /// # Errors
    /// Returns an error if initialization fails
    pub async fn new() -> Result<Self, MetricsError> {
        let result_cache = Cache::builder()
            .time_to_live(std::time::Duration::from_secs(300)) // 5 minutes TTL
            .build();

        Ok(Self {
            request_count: Arc::new(Mutex::new(0)),
            total_processing_time: Arc::new(Mutex::new(0)),
            confidence_scores: Arc::new(Mutex::new(Vec::new())),
            error_count: Arc::new(Mutex::new(0)),
            result_cache,
        })
    }

    /// Record a request metric
    /// # Errors
    /// Returns an error if state update fails
    pub async fn record_request(&self, modality_count: usize) -> Result<(), MetricsError> {
        let mut count = self.request_count.lock().await;
        *count += 1;
        // Could log modality_count for analysis
        Ok(())
    }

    /// Record completion of processing
    /// # Errors
    /// Returns an error if state update fails
    pub async fn record_completion(&self, confidence: f32) -> Result<(), MetricsError> {
        let mut scores = self.confidence_scores.lock().await;
        scores.push(confidence);
        if scores.len() > 1000 {
            scores.remove(0); // Keep only last 1000 scores
        }
        Ok(())
    }

    /// Record an error
    /// # Errors
    /// Returns an error if state update fails
    pub async fn record_error(&self) -> Result<(), MetricsError> {
        let mut count = self.error_count.lock().await;
        *count += 1;
        Ok(())
    }

    /// Get current metrics
    /// # Errors
    /// Returns an error if metrics retrieval fails
    pub async fn get_metrics(&self) -> Result<ProcessingMetrics, MetricsError> {
        let request_count = *self.request_count.lock().await;
        let total_processing_time = *self.total_processing_time.lock().await;
        let confidence_scores = self.confidence_scores.lock().await.clone();
        let error_count = *self.error_count.lock().await;

        let avg_confidence = if confidence_scores.is_empty() {
            0.0
        } else {
            confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32
        };

        let avg_processing_time = if request_count == 0 {
            0
        } else {
            total_processing_time / request_count
        };

        Ok(ProcessingMetrics {
            request_count,
            total_processing_time,
            avg_confidence,
            error_count,
            avg_processing_time,
            cache_hit_rate: 0.0, // TODO: Implement cache metrics
        })
    }

    /// Cache a result
    /// # Errors
    /// Returns an error if caching fails
    pub async fn cache_result(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), MetricsError> {
        self.result_cache.insert(key, value).await;
        Ok(())
    }

    /// Get cached result
    /// # Errors
    /// Returns an error if retrieval fails
    pub async fn get_cached_result(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, MetricsError> {
        Ok(self.result_cache.get(key).await)
    }

    /// Clear metrics (for testing)
    /// # Errors
    /// Returns an error if clearing fails
    pub async fn clear(&self) -> Result<(), MetricsError> {
        *self.request_count.lock().await = 0;
        *self.total_processing_time.lock().await = 0;
        self.confidence_scores.lock().await.clear();
        *self.error_count.lock().await = 0;
        self.result_cache.invalidate_all();
        self.result_cache.run_pending_tasks().await;
        Ok(())
    }
}

/// Processing metrics data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessingMetrics {
    /// Total number of requests processed
    pub request_count: u64,
    /// Total processing time across all requests
    pub total_processing_time: u64,
    /// Average confidence score
    pub avg_confidence: f32,
    /// Number of errors
    pub error_count: u64,
    /// Average processing time per request
    pub avg_processing_time: u64,
    /// Cache hit rate
    pub cache_hit_rate: f32,
}

/// Metrics collection error
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// Cache operation error
    #[error("Cache operation error: {0}")]
    CacheError(String),

    /// State lock error
    #[error("State lock error: {0}")]
    LockError(String),
}

impl From<moka::changelog::WriteOpError> for MetricsError {
    fn from(err: moka::changelog::WriteOpError) -> Self {
        MetricsError::CacheError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new().await.unwrap();

        collector.record_request(1).await.unwrap();
        collector.record_completion(0.8).await.unwrap();
        collector.record_error().await.unwrap();

        let metrics = collector.get_metrics().await.unwrap();

        assert_eq!(metrics.request_count, 1);
        assert_eq!(metrics.avg_confidence, 0.8);
        assert_eq!(metrics.error_count, 1);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let collector = MetricsCollector::new().await.unwrap();
        let key = "test_key".to_string();
        let value = serde_json::json!({"test": "data"});

        collector.cache_result(key.clone(), value.clone()).await.unwrap();
        let retrieved = collector.get_cached_result(&key).await.unwrap();

        assert_eq!(retrieved.as_ref(), Some(&value));
    }
}