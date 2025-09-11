//! Custom Rust Performance Instrumentation
//!
//! This module provides custom performance instrumentation for Rust code,
//! including automatic timing, memory tracking, and integration with
//! Prometheus metrics collection.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::metrics::{MetricsRegistry, PrometheusMetric, MetricType, MetricValue};

/// Instrumentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationConfig {
    /// Enable automatic function timing
    pub enable_auto_timing: bool,
    /// Enable memory tracking
    pub enable_memory_tracking: bool,
    /// Enable async operation tracking
    pub enable_async_tracking: bool,
    /// Sampling rate (1.0 = 100% sampling)
    pub sampling_rate: f64,
    /// Maximum number of tracked operations
    pub max_tracked_operations: usize,
}

impl Default for InstrumentationConfig {
    fn default() -> Self {
        Self {
            enable_auto_timing: true,
            enable_memory_tracking: true,
            enable_async_tracking: true,
            sampling_rate: 1.0,
            max_tracked_operations: 10000,
        }
    }
}

/// Performance instrumentation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationData {
    pub operation_name: String,
    pub start_time: Instant,
    pub duration: Option<Duration>,
    pub memory_used: Option<u64>,
    pub cpu_cycles: Option<u64>,
    pub tags: HashMap<String, String>,
}

/// Performance profiler with custom instrumentation
pub struct PerformanceInstrumentor {
    config: InstrumentationConfig,
    active_operations: Arc<RwLock<HashMap<String, InstrumentationData>>>,
    completed_operations: Arc<RwLock<Vec<InstrumentationData>>>,
    metrics_registry: Option<Arc<MetricsRegistry>>,
}

impl PerformanceInstrumentor {
    pub fn new(config: InstrumentationConfig) -> Self {
        Self {
            config,
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            completed_operations: Arc::new(RwLock::new(Vec::new())),
            metrics_registry: None,
        }
    }

    pub fn with_metrics_registry(mut self, registry: Arc<MetricsRegistry>) -> Self {
        self.metrics_registry = Some(registry);
        self
    }

    /// Start tracking an operation
    pub async fn start_operation(&self, operation_name: &str) -> Result<String, InstrumentationError> {
        if !self.should_sample() {
            return Err(InstrumentationError::SamplingSkipped);
        }

        let operation_id = format!("{}_{}", operation_name, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());

        let data = InstrumentationData {
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            duration: None,
            memory_used: if self.config.enable_memory_tracking {
                Some(self.get_current_memory_usage())
            } else {
                None
            },
            cpu_cycles: None, // Would need platform-specific implementation
            tags: HashMap::new(),
        };

        let mut operations = self.active_operations.write().await;
        operations.insert(operation_id.clone(), data);

        Ok(operation_id)
    }

    /// End tracking an operation
    pub async fn end_operation(&self, operation_id: &str) -> Result<InstrumentationData, InstrumentationError> {
        let mut operations = self.active_operations.write().await;

        if let Some(mut data) = operations.remove(operation_id) {
            data.duration = Some(data.start_time.elapsed());

            if self.config.enable_memory_tracking {
                if let Some(start_memory) = data.memory_used {
                    data.memory_used = Some(self.get_current_memory_usage().saturating_sub(start_memory));
                }
            }

            // Store completed operation
            let mut completed = self.completed_operations.write().await;
            completed.push(data.clone());

            // Maintain size limit
            if completed.len() > self.config.max_tracked_operations {
                completed.remove(0);
            }

            // Update metrics if registry is available
            if let Some(registry) = &self.metrics_registry {
                self.update_metrics(registry, &data).await?;
            }

            Ok(data)
        } else {
            Err(InstrumentationError::OperationNotFound(operation_id.to_string()))
        }
    }

    /// Time a synchronous operation
    pub async fn time_sync_operation<F, T>(&self, operation_name: &str, operation: F) -> Result<(T, Duration), InstrumentationError>
    where
        F: FnOnce() -> T,
    {
        let operation_id = self.start_operation(operation_name).await?;

        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();

        let mut data = self.end_operation(&operation_id).await?;
        data.duration = Some(duration);

        Ok((result, duration))
    }

    /// Time an asynchronous operation
    pub async fn time_async_operation<F, Fut, T>(&self, operation_name: &str, operation: F) -> Result<(T, Duration), InstrumentationError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let operation_id = self.start_operation(operation_name).await?;

        let start = Instant::now();
        let result = operation().await;
        let duration = start.elapsed();

        let mut data = self.end_operation(&operation_id).await?;
        data.duration = Some(duration);

        Ok((result, duration))
    }

    /// Add tags to an active operation
    pub async fn add_tags(&self, operation_id: &str, tags: HashMap<String, String>) -> Result<(), InstrumentationError> {
        let mut operations = self.active_operations.write().await;

        if let Some(data) = operations.get_mut(operation_id) {
            data.tags.extend(tags);
            Ok(())
        } else {
            Err(InstrumentationError::OperationNotFound(operation_id.to_string()))
        }
    }

    /// Get operation statistics
    pub async fn get_operation_stats(&self, operation_name: &str) -> HashMap<String, serde_json::Value> {
        let completed = self.completed_operations.read().await;

        let operations: Vec<&InstrumentationData> = completed.iter()
            .filter(|data| data.operation_name == operation_name)
            .collect();

        if operations.is_empty() {
            return HashMap::new();
        }

        let durations: Vec<Duration> = operations.iter()
            .filter_map(|data| data.duration)
            .collect();

        if durations.is_empty() {
            return HashMap::new();
        }

        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / durations.len() as u32;
        let min_duration = durations.iter().min().unwrap().clone();
        let max_duration = durations.iter().max().unwrap().clone();

        let mut stats = HashMap::new();
        stats.insert("count".to_string(), serde_json::json!(operations.len()));
        stats.insert("total_duration_ms".to_string(), serde_json::json!(total_duration.as_millis()));
        stats.insert("avg_duration_ms".to_string(), serde_json::json!(avg_duration.as_millis()));
        stats.insert("min_duration_ms".to_string(), serde_json::json!(min_duration.as_millis()));
        stats.insert("max_duration_ms".to_string(), serde_json::json!(max_duration.as_millis()));

        stats
    }

    /// Get all completed operations
    pub async fn get_completed_operations(&self) -> Vec<InstrumentationData> {
        self.completed_operations.read().await.clone()
    }

    /// Clear completed operations
    pub async fn clear_completed_operations(&self) {
        let mut completed = self.completed_operations.write().await;
        completed.clear();
    }

    /// Update Prometheus metrics based on instrumentation data
    async fn update_metrics(&self, registry: &MetricsRegistry, data: &InstrumentationData) -> Result<(), InstrumentationError> {
        if let Some(duration) = data.duration {
            // Create operation-specific metric name
            let metric_name = format!("rust_ai_ide_operation_duration_{}", sanitize_metric_name(&data.operation_name));

            // Try to update existing metric or create new one
            if let Err(_) = registry.update_counter(&metric_name, 1).await {
                // Metric doesn't exist, create it
                let metric = PrometheusMetric::new(
                    metric_name.clone(),
                    format!("Duration of {} operation", data.operation_name),
                    MetricType::Histogram,
                );

                if let Err(e) = registry.register_metric(metric).await {
                    return Err(InstrumentationError::MetricsError(e.to_string()));
                }
            }

            // For histogram, we'd need to implement bucket logic
            // This is simplified for now
        }

        Ok(())
    }

    /// Get current memory usage (simplified implementation)
    fn get_current_memory_usage(&self) -> u64 {
        // In a real implementation, this would use platform-specific APIs
        // For now, return a placeholder value
        100_000_000 // 100MB placeholder
    }

    /// Determine if operation should be sampled
    fn should_sample(&self) -> bool {
        rand::random::<f64>() < self.config.sampling_rate
    }
}

/// Scoped instrumentation helper
pub struct ScopedInstrumentation<'a> {
    instrumentor: &'a PerformanceInstrumentor,
    operation_id: Option<String>,
}

impl<'a> ScopedInstrumentation<'a> {
    pub async fn new(instrumentor: &'a PerformanceInstrumentor, operation_name: &str) -> Result<Self, InstrumentationError> {
        let operation_id = instrumentor.start_operation(operation_name).await.ok();

        Ok(Self {
            instrumentor,
            operation_id,
        })
    }

    pub async fn add_tag(&self, key: String, value: String) -> Result<(), InstrumentationError> {
        if let Some(ref operation_id) = self.operation_id {
            let mut tags = HashMap::new();
            tags.insert(key, value);
            self.instrumentor.add_tags(operation_id, tags).await?;
        }
        Ok(())
    }
}

impl<'a> Drop for ScopedInstrumentation<'a> {
    fn drop(&mut self) {
        if let Some(ref operation_id) = self.operation_id {
            // Note: We can't make this async in Drop, so we'd need to handle cleanup differently
            // In practice, you might want to spawn a task or use a different pattern
        }
    }
}

/// Instrumentation errors
#[derive(Debug, Clone)]
pub enum InstrumentationError {
    OperationNotFound(String),
    SamplingSkipped,
    MetricsError(String),
    ConfigurationError(String),
}

impl std::fmt::Display for InstrumentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstrumentationError::OperationNotFound(id) => write!(f, "Operation not found: {}", id),
            InstrumentationError::SamplingSkipped => write!(f, "Operation skipped due to sampling"),
            InstrumentationError::MetricsError(msg) => write!(f, "Metrics error: {}", msg),
            InstrumentationError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for InstrumentationError {}

/// Sanitize metric name for Prometheus
fn sanitize_metric_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_instrumentor_creation() {
        let config = InstrumentationConfig::default();
        let instrumentor = PerformanceInstrumentor::new(config);

        assert!(instrumentor.config.enable_auto_timing);
        assert!(instrumentor.config.enable_memory_tracking);
    }

    #[tokio::test]
    async fn test_operation_timing() {
        let instrumentor = PerformanceInstrumentor::new(InstrumentationConfig::default());

        let operation_id = instrumentor.start_operation("test_operation").await.unwrap();

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;

        let data = instrumentor.end_operation(&operation_id).await.unwrap();

        assert_eq!(data.operation_name, "test_operation");
        assert!(data.duration.unwrap() >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_sync_operation_timing() {
        let instrumentor = PerformanceInstrumentor::new(InstrumentationConfig::default());

        let (result, duration) = instrumentor.time_sync_operation("sync_test", || {
            std::thread::sleep(Duration::from_millis(5));
            42
        }).await.unwrap();

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(5));
    }

    #[tokio::test]
    async fn test_operation_stats() {
        let instrumentor = PerformanceInstrumentor::new(InstrumentationConfig::default());

        // Perform multiple operations
        for i in 0..3 {
            let operation_id = instrumentor.start_operation("stats_test").await.unwrap();
            tokio::time::sleep(Duration::from_millis(1)).await;
            instrumentor.end_operation(&operation_id).await.unwrap();
        }

        let stats = instrumentor.get_operation_stats("stats_test").await;

        assert_eq!(stats.get("count"), Some(&serde_json::json!(3)));
        assert!(stats.contains_key("avg_duration_ms"));
        assert!(stats.contains_key("min_duration_ms"));
        assert!(stats.contains_key("max_duration_ms"));
    }
}