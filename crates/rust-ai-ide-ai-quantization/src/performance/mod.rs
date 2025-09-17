use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::IDEError;

/// Performance metrics for quantization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationMetrics {
    /// Number of models quantized
    pub models_quantized: u64,
    /// Total quantization time in milliseconds
    pub total_quantization_time_ms: u64,
    /// Average quantization time per model
    pub average_quantization_time_ms: f64,
    /// Peak memory usage in bytes
    pub peak_memory_usage_bytes: u64,
    /// Current memory usage in bytes
    pub current_memory_usage_bytes: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    /// Quantization success rate (0.0 to 1.0)
    pub success_rate: f64,
}

impl Default for QuantizationMetrics {
    fn default() -> Self {
        Self {
            models_quantized: 0,
            total_quantization_time_ms: 0,
            average_quantization_time_ms: 0.0,
            peak_memory_usage_bytes: 0,
            current_memory_usage_bytes: 0,
            cache_hit_ratio: 0.0,
            success_rate: 1.0,
        }
    }
}

/// Performance tracker for quantization operations
pub struct QuantizationPerformanceTracker {
    /// Collected metrics
    metrics: Arc<Mutex<QuantizationMetrics>>,
    /// Number of successful operations
    successful_operations: Arc<Mutex<u64>>,
    /// Total operations attempted
    total_operations: Arc<Mutex<u64>>,
    /// Cache hits counter
    cache_hits: Arc<Mutex<u64>>,
    /// Cache misses counter
    cache_misses: Arc<Mutex<u64>>,
}

impl QuantizationPerformanceTracker {
    /// Create new performance tracker
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(QuantizationMetrics::default())),
            successful_operations: Arc::new(Mutex::new(0)),
            total_operations: Arc::new(Mutex::new(0)),
            cache_hits: Arc::new(Mutex::new(0)),
            cache_misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> QuantizationMetrics {
        let metrics_guard = self.metrics.lock().await;
        metrics_guard.clone()
    }

    /// Update cache hit statistics
    pub async fn record_cache_hit(&self) {
        let mut cache_hits = self.cache_hits.lock().await;
        *cache_hits += 1;
        self.update_cache_hit_ratio().await;
    }

    /// Update cache miss statistics
    pub async fn record_cache_miss(&self) {
        let mut cache_misses = self.cache_misses.lock().await;
        *cache_misses += 1;
        self.update_cache_hit_ratio().await;
    }

    /// Record successful quantization operation
    pub async fn record_success(&self, quantization_time: Duration, memory_used: u64) {
        let mut successful_ops = self.successful_operations.lock().await;
        let mut total_ops = self.total_operations.lock().await;
        let mut metrics = self.metrics.lock().await;

        *successful_ops += 1;
        *total_ops += 1;

        metrics.models_quantized += 1;
        metrics.total_quantization_time_ms += quantization_time.as_millis() as u64;

        if metrics.models_quantized > 0 {
            metrics.average_quantization_time_ms =
                metrics.total_quantization_time_ms as f64 / metrics.models_quantized as f64;
        }

        metrics.current_memory_usage_bytes = memory_used;
        if memory_used > metrics.peak_memory_usage_bytes {
            metrics.peak_memory_usage_bytes = memory_used;
        }

        self.update_success_rate().await;
    }

    /// Record failed quantization operation
    pub async fn record_failure(&self) {
        let mut total_ops = self.total_operations.lock().await;
        *total_ops += 1;
        self.update_success_rate().await;
    }

    /// Record memory usage
    pub async fn record_memory_usage(&self, bytes: u64) {
        let mut metrics = self.metrics.lock().await;
        metrics.current_memory_usage_bytes = bytes;
        if bytes > metrics.peak_memory_usage_bytes {
            metrics.peak_memory_usage_bytes = bytes;
        }
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().await;
        let mut successful_ops = self.successful_operations.lock().await;
        let mut total_ops = self.total_operations.lock().await;
        let mut cache_hits = self.cache_hits.lock().await;
        let mut cache_misses = self.cache_misses.lock().await;

        *metrics = QuantizationMetrics::default();
        *successful_ops = 0;
        *total_ops = 0;
        *cache_hits = 0;
        *cache_misses = 0;
    }

    /// Internal: Update cache hit ratio
    async fn update_cache_hit_ratio(&self) {
        let cache_hits = *self.cache_hits.lock().await;
        let cache_misses = *self.cache_misses.lock().await;
        let total_cache_ops = cache_hits + cache_misses;

        let mut metrics = self.metrics.lock().await;
        if total_cache_ops > 0 {
            metrics.cache_hit_ratio = cache_hits as f64 / total_cache_ops as f64;
        } else {
            metrics.cache_hit_ratio = 0.0;
        }
    }

    /// Internal: Update success rate
    async fn update_success_rate(&self) {
        let successful_ops = *self.successful_operations.lock().await;
        let total_ops = *self.total_operations.lock().await;

        let mut metrics = self.metrics.lock().await;
        if total_ops > 0 {
            metrics.success_rate = successful_ops as f64 / total_ops as f64;
        } else {
            metrics.success_rate = 1.0;
        }
    }
}

/// Context manager for tracking quantization performance
pub struct QuantizationPerformanceContext<'a> {
    tracker: &'a QuantizationPerformanceTracker,
    start_time: Instant,
    memory_start: u64,
    operation_successful: bool,
}

impl<'a> QuantizationPerformanceContext<'a> {
    /// Create new performance context
    pub fn new(tracker: &'a QuantizationPerformanceTracker) -> Self {
        let start_time = Instant::now();
        let memory_start = Self::get_current_memory_usage();

        Self {
            tracker,
            start_time,
            memory_start,
            operation_successful: false,
        }
    }

    /// Mark operation as successful
    pub fn mark_success(&mut self) {
        self.operation_successful = true;
    }

    /// Get current memory usage (simplified estimation)
    fn get_current_memory_usage() -> u64 {
        // In a real implementation, this would query system memory usage
        // For now, return a placeholder value
        1024 * 1024 * 100 // 100MB placeholder
    }
}

impl<'a> Drop for QuantizationPerformanceContext<'a> {
    fn drop(&mut self) {
        if self.operation_successful {
            let duration = self.start_time.elapsed();
            let memory_used = Self::get_current_memory_usage();

            // Record metrics (async in tokio context)
            let tracker = self.tracker;
            tokio::spawn(async move {
                tracker.record_success(duration, memory_used).await;
            });
        } else {
            // Record failure
            let tracker = self.tracker;
            tokio::spawn(async move {
                tracker.record_failure().await;
            });
        }
    }
}

/// Macro to create performance context for quantization operations
#[macro_export]
macro_rules! with_quantization_performance {
    ($tracker:expr, $body:block) => {{
        let mut ctx = QuantizationPerformanceContext::new($tracker);
        let result = $body;
        if result.is_ok() {
            ctx.mark_success();
        }
        result
    }};
}
