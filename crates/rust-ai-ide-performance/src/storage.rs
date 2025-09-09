//! Metrics Storage Backend
//!
//! This module provides various storage backends for performance metrics
//! with historical tracking and trend analysis capabilities.

use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration as ChronoDuration, Timelike};
use rust_ai_ide_shared_types::PerformanceMetrics;

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    /// Maximum number of metrics to keep in memory
    pub max_in_memory_metrics: usize,
    /// Retain metrics for this many days
    pub retention_days: i64,
    /// Data directory for file-based storage
    pub data_directory: String,
    /// Enable automatic compaction
    pub auto_compaction: bool,
    /// Compaction interval in hours
    pub compaction_interval_hours: i64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::InMemory,
            max_in_memory_metrics: 10_000,
            retention_days: 30,
            data_directory: "performance_data".to_string(),
            auto_compaction: true,
            compaction_interval_hours: 24,
        }
    }
}

/// Storage backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    /// In-memory storage only (volatile)
    InMemory,
    /// File-based JSON storage
    File,
    /// Database storage (future extension)
    Database,
}

/// Historical metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshots by time bucket (hourly)
    pub snapshots: HashMap<i64, AggregatedMetrics>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Aggregated metrics for a time bucket
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregatedMetrics {
    /// Number of samples in this bucket
    pub sample_count: u64,
    /// Average metrics
    pub averages: PerformanceMetrics,
    /// Min values
    pub mins: PerformanceMetrics,
    /// Max values
    pub maxs: PerformanceMetrics,
    /// Standard deviations
    pub stds: MetricStdDevs,
    /// Percentiles (P50, P90, P95, P99)
    pub percentiles: MetricPercentiles,
}

/// Standard deviations for key metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricStdDevs {
    pub cpu_usage_percent: f64,
    pub memory_bytes: f64,
    pub response_time_ns: f64,
}

/// Percentiles for key metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricPercentiles {
    /// CPU usage percentiles
    pub cpu_percentiles: PercentileValues,
    /// Memory usage percentiles
    pub memory_percentiles: PercentileValues,
    /// Response time percentiles
    pub response_time_percentiles: PercentileValues,
}

/// Percentile values (P50, P90, P95, P99)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PercentileValues {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Metric name
    pub metric_name: String,
    /// Trend direction (positive for increasing, negative for decreasing)
    pub trend_coefficient: f64,
    /// Trend confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Trend description
    pub description: String,
    /// Prediction for next timeframe
    pub next_prediction: f64,
}

/// Performance metrics storage trait
#[async_trait::async_trait]
pub trait MetricsStorage: Send + Sync {
    /// Store metrics
    async fn store_metrics(&self, metrics: &PerformanceMetrics) -> Result<(), StorageError>;

    /// Retrieve metrics for a time range
    async fn get_metrics(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PerformanceMetrics>, StorageError>;

    /// Get aggregated metrics for time buckets
    async fn get_aggregated_metrics(&self, bucket_hours: i64) -> Result<Vec<(DateTime<Utc>, AggregatedMetrics)>, StorageError>;

    /// Get trend analysis
    async fn get_trends(&self, metric_name: &str, days: i64) -> Result<TrendAnalysis, StorageError>;

    /// Cleanup old data based on retention policy
    async fn cleanup(&self) -> Result<(), StorageError>;

    /// Export data (for backup/analysis)
    async fn export(&self, format: ExportFormat) -> Result<Vec<u8>, StorageError>;
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Parquet,
    Prometheus,
}

/// Storage error types
#[derive(Debug, Clone)]
pub enum StorageError {
    IoError(String),
    SerializationError(String),
    DatabaseError(String),
    NotFound(String),
    InvalidConfig(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(msg) => write!(f, "IO Error: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            StorageError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            StorageError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            StorageError::InvalidConfig(msg) => write!(f, "Invalid Config: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

/// In-memory metrics storage
pub struct InMemoryStorage {
    /// Storage buffer
    buffer: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    /// Maximum buffer size
    max_size: usize,
    /// Current snapshots (hourly)
    snapshots: Arc<RwLock<HashMap<i64, AggregatedMetrics>>>,
}

impl InMemoryStorage {
    /// Create new in-memory storage
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(max_size))),
            max_size,
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl MetricsStorage for InMemoryStorage {
    async fn store_metrics(&self, metrics: &PerformanceMetrics) -> Result<(), StorageError> {
        // Buffer operations within separate scope
        {
            let mut buffer = self.buffer.write().unwrap();
            buffer.push_back(metrics.clone());

            // Maintain size
            if buffer.len() > self.max_size {
                buffer.pop_front();
            }
        }

        // Update snapshots outside the lock scope
        self.update_snapshot(metrics.clone()).await;

        Ok(())
    }

    async fn get_metrics(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PerformanceMetrics>, StorageError> {
        let buffer = self.buffer.read().unwrap();
        let start_ts = start.timestamp() as u64;
        let end_ts = end.timestamp() as u64;
        let metrics = buffer
            .iter()
            .filter(|m| m.timestamp >= start_ts && m.timestamp <= end_ts)
            .cloned()
            .collect();

        Ok(metrics)
    }

    async fn get_aggregated_metrics(&self, _bucket_hours: i64) -> Result<Vec<(DateTime<Utc>, AggregatedMetrics)>, StorageError> {
        let snapshots = self.snapshots.read().unwrap();
        let result: Vec<(DateTime<Utc>, AggregatedMetrics)> = snapshots
            .iter()
            .map(|(timestamp, metrics)| {
                let dt = DateTime::from_timestamp(*timestamp, 0).unwrap_or(Utc::now());
                (dt, metrics.clone())
            })
            .collect();

        Ok(result)
    }

    async fn get_trends(&self, metric_name: &str, days: i64) -> Result<TrendAnalysis, StorageError> {
        let buffer = self.buffer.read().unwrap();
        let cutoff = Utc::now() - ChronoDuration::days(days);

        // Extract metric values
        let cutoff_ts = cutoff.timestamp() as u64;
        let values: Vec<f64> = buffer
            .iter()
            .filter(|m| m.timestamp >= cutoff_ts)
            .filter_map(|m| self.extract_metric_value(m, metric_name))
            .collect();

        if values.len() < 2 {
            return Err(StorageError::NotFound(format!("Insufficient data for trend analysis of {}", metric_name)));
        }

        // Simple linear regression for trend
        let (slope, confidence) = self.linear_regression(&values);

        let description = if slope > 0.0 {
            format!("{} is trending upward", metric_name)
        } else if slope < 0.0 {
            format!("{} is trending downward", metric_name)
        } else {
            format!("{} is stable with no clear trend", metric_name)
        };

        // Simple prediction for next value
        let next_prediction = values.last().unwrap() + slope;

        Ok(TrendAnalysis {
            metric_name: metric_name.to_string(),
            trend_coefficient: slope,
            confidence,
            description,
            next_prediction,
        })
    }

    async fn cleanup(&self) -> Result<(), StorageError> {
        // For in-memory storage, cleanup is just clearing old data
        // Keep only the most recent 80% of capacity
        let keep_size = (self.max_size * 4) / 5;

        // Drain within the lock scope to avoid mutable/immutable borrow conflict
        {
            let mut buffer = self.buffer.write().unwrap();
            if buffer.len() > keep_size {
                buffer.drain(0..(buffer.len() - keep_size));
            }
        }

        let mut snapshots = self.snapshots.write().unwrap();
        let cutoff = (Utc::now() - ChronoDuration::days(7)).timestamp();
        snapshots.retain(|timestamp, _| *timestamp > cutoff);

        Ok(())
    }

    async fn export(&self, format: ExportFormat) -> Result<Vec<u8>, StorageError> {
        match format {
            ExportFormat::Json => {
                let buffer = self.buffer.read().unwrap();
                serde_json::to_vec(&*buffer).map_err(|e| StorageError::SerializationError(e.to_string()))
            }
            ExportFormat::Csv => {
                // TODO: Implement CSV export
                Err(StorageError::InvalidConfig("CSV export not implemented yet".to_string()))
            }
            ExportFormat::Parquet => {
                // TODO: Implement parquet export when tokio-parquet is available
                Err(StorageError::InvalidConfig("Parquet export not implemented yet".to_string()))
            }
            ExportFormat::Prometheus => {
                // TODO: Implement Prometheus format export
                Err(StorageError::InvalidConfig("Prometheus export not implemented yet".to_string()))
            }
        }
    }
}

impl InMemoryStorage {
    /// Extract metric value by name
    fn extract_metric_value(&self, metrics: &PerformanceMetrics, metric_name: &str) -> Option<f64> {
        match metric_name {
            "cpu_usage_percent" => metrics.rates.cpu_usage_percent,
            "memory_bytes" => metrics.resources.memory_bytes.map(|v| v as f64),
            "response_time_ns" => metrics.timing.response_time_ns.map(|v| v as f64),
            "latency_average" => Some(calculate_average(&metrics.timing.latency_ns)),
            _ => None,
        }
    }

    /// Update hourly snapshots
    async fn update_snapshot(&self, metrics: PerformanceMetrics) {
        let hour = (metrics.timestamp / 3600) as i64;
        let mut snapshots = self.snapshots.write().unwrap();

        let entry = snapshots.entry(hour).or_insert_with(AggregatedMetrics::default);
        self.aggregate_metric(entry, &metrics);
    }

    /// Aggregate a metric into snapshot
    fn aggregate_metric(&self, aggregated: &mut AggregatedMetrics, metrics: &PerformanceMetrics) {
        aggregated.sample_count += 1;

        // CPU aggregation
        if let Some(cpu) = metrics.rates.cpu_usage_percent {
            self.update_aggregation(&mut aggregated.averages.rates.cpu_usage_percent,
                                   &mut aggregated.mins.rates.cpu_usage_percent,
                                   &mut aggregated.maxs.rates.cpu_usage_percent,
                                   Some(cpu), cpu, cpu);
        }

        // Memory aggregation - convert u64 to f64 for aggregation
        if let Some(mem) = metrics.resources.memory_bytes {
            let mem_f64 = mem as f64;
            self.update_aggregation_memory(&mut aggregated.averages.resources.memory_bytes,
                                          &mut aggregated.mins.resources.memory_bytes,
                                          &mut aggregated.maxs.resources.memory_bytes,
                                          Some(mem), mem, mem);
        }
    }

    /// Update aggregation values for f64 metrics
    fn update_aggregation(&self, avg: &mut Option<f64>, min: &mut Option<f64>, max: &mut Option<f64>, current: Option<f64>, min_val: f64, max_val: f64) {
        if let Some(current_val) = current {
            self.update_aggregation_inner(avg, min, max, current_val, min_val, max_val);
        }
    }

    /// Update aggregation values for u64 metrics (like memory_bytes)
    fn update_aggregation_u64(&self, avg: &mut Option<f64>, min: &mut Option<f64>, max: &mut Option<f64>, current: Option<u64>, min_val: u64, max_val: u64) {
        if let Some(current_val) = current {
            let current_f64 = current_val as f64;
            let min_val_f64 = min_val as f64;
            let max_val_f64 = max_val as f64;
            self.update_aggregation_inner(avg, min, max, current_f64, min_val_f64, max_val_f64);
        }
    }

    /// Update aggregation for memory (u64) fields in PerformanceMetrics
    fn update_aggregation_memory(&self, avg: &mut Option<u64>, min: &mut Option<u64>, max: &mut Option<u64>, current: Option<u64>, min_val: u64, max_val: u64) {
        if let Some(current_val) = current {
            // Average (weighted) - convert to f64 internally for calculation
            let count = if avg.is_some() { 1.0 } else { 0.0 };
            let new_avg = match *avg {
                Some(cur_avg) => {
                    let cur_f64 = cur_avg as f64;
                    ((cur_f64 * count + current_val as f64) / (count + 1.0)) as u64
                },
                None => current_val,
            };
            *avg = Some(new_avg);

            // Min/Max for u64 - fix borrowing issues
            if let Some(cur_min) = min {
                *min = Some(*cur_min.min(&min_val));
            } else {
                *min = Some(min_val);
            }
            if let Some(cur_max) = max {
                *max = Some(*cur_max.max(&max_val));
            } else {
                *max = Some(max_val);
            }
        }
    }

    /// Inner aggregation logic for f64 values
    fn update_aggregation_inner(&self, avg: &mut Option<f64>, min: &mut Option<f64>, max: &mut Option<f64>, current_val: f64, min_val: f64, max_val: f64) {
        // Average (weighted)
        let count = match avg {
            Some(_) => 1.0,
            None => 0.0,
        };
        let new_avg = match *avg {
            Some(cur_avg) => (cur_avg * count + current_val) / (count + 1.0),
            None => current_val,
        };
        *avg = Some(new_avg);

        // Min/Max
        if let Some(cur_min) = min {
            *min = Some(cur_min.min(min_val));
        } else {
            *min = Some(min_val);
        }
        if let Some(cur_max) = max {
            *max = Some(cur_max.max(max_val));
        } else {
            *max = Some(max_val);
        }
    }

    /// Simple linear regression
    fn linear_regression(&self, values: &[f64]) -> (f64, f64) {
        let n = values.len() as f64;
        if n < 2.0 {
            return (0.0, 0.0);
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        let confidence = if !slope.is_nan() && slope.is_finite() { 0.8 } else { 0.0 };

        (if slope.is_nan() { 0.0 } else { slope }, confidence)
    }
}

/// File-based metrics storage
pub struct FileStorage {
    config: StorageConfig,
    in_memory: InMemoryStorage,
}

impl FileStorage {
    /// Create new file-based storage
    pub fn new(config: StorageConfig) -> Self {
        let max_size = config.max_in_memory_metrics;
        Self {
            config,
            in_memory: InMemoryStorage::new(max_size),
        }
    }

    /// Save metrics to file
    async fn save_to_file(&self, metrics: &PerformanceMetrics) -> Result<(), StorageError> {
        // Ensure directory exists
        tokio::fs::create_dir_all(&self.config.data_directory).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let filename = format!("{}/metrics_{}.json",
                              self.config.data_directory,
                              metrics.timestamp);

        let data = serde_json::to_string_pretty(metrics)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        tokio::fs::write(&filename, data).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load metrics from file (future implementation)
    async fn load_from_file(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<Vec<PerformanceMetrics>, StorageError> {
        // TODO: Implement file loading
        Ok(Vec::new())
    }
}

#[async_trait::async_trait]
impl MetricsStorage for FileStorage {
    async fn store_metrics(&self, metrics: &PerformanceMetrics) -> Result<(), StorageError> {
        // First store in memory
        self.in_memory.store_metrics(metrics).await?;

        // Then persist to file
        self.save_to_file(metrics).await?;

        Ok(())
    }

    async fn get_metrics(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PerformanceMetrics>, StorageError> {
        // First try memory
        let mut metrics = self.in_memory.get_metrics(start, end).await?;

        // Then supplement with file data if needed
        let file_metrics = self.load_from_file(start, end).await?;
        metrics.extend(file_metrics);

        // Sort by timestamp
        metrics.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(metrics)
    }

    async fn get_aggregated_metrics(&self, bucket_hours: i64) -> Result<Vec<(DateTime<Utc>, AggregatedMetrics)>, StorageError> {
        self.in_memory.get_aggregated_metrics(bucket_hours).await
    }

    async fn get_trends(&self, metric_name: &str, days: i64) -> Result<TrendAnalysis, StorageError> {
        self.in_memory.get_trends(metric_name, days).await
    }

    async fn cleanup(&self) -> Result<(), StorageError> {
        // Memory cleanup
        self.in_memory.cleanup().await?;

        // File cleanup (remove old files)
        let cutoff = Utc::now() - ChronoDuration::days(self.config.retention_days);

        if let Ok(mut entries) = tokio::fs::read_dir(&self.config.data_directory).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time = DateTime::<Utc>::from(modified);
                        if modified_time < cutoff {
                            let _ = tokio::fs::remove_file(entry.path()).await; // Ignore errors
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn export(&self, format: ExportFormat) -> Result<Vec<u8>, StorageError> {
        self.in_memory.export(format).await
    }
}

/// Calculate average of u64 slice
fn calculate_average(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: u64 = values.iter().sum();
    sum as f64 / values.len() as f64
}

/// Storage factory
pub struct StorageFactory;

impl StorageFactory {
    /// Create storage backend from config
    pub fn create(config: StorageConfig) -> Box<dyn MetricsStorage> {
        match config.backend {
            StorageBackend::InMemory => {
                Box::new(InMemoryStorage::new(config.max_in_memory_metrics))
            }
            StorageBackend::File => {
                Box::new(FileStorage::new(config))
            }
            StorageBackend::Database => {
                // For now, fall back to in-memory with database placeholder
                // TODO: Implement actual database storage
                Box::new(InMemoryStorage::new(config.max_in_memory_metrics))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryStorage::new(100);

        let metrics = PerformanceMetrics::new();
        storage.store_metrics(&metrics).await.unwrap();

        let start = Utc::now() - ChronoDuration::hours(1);
        let end = Utc::now() + ChronoDuration::hours(1);
        let retrieved = storage.get_metrics(start, end).await.unwrap();

        assert_eq!(retrieved.len(), 1);
    }

    #[test]
    fn test_linear_regression() {
        let storage = InMemoryStorage::new(10);
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let (slope, confidence) = storage.linear_regression(&values);
        assert!(slope > 0.9 && slope < 1.1, "Slope should be approximately 1.0");
        assert!(confidence > 0.5, "Confidence should be reasonable");
    }

    #[test]
    fn test_calculate_average() {
        let values = vec![1u64, 2, 3, 4, 5];
        let avg = calculate_average(&values);
        assert_eq!(avg, 3.0);
    }

    #[test]
    fn test_empty_average() {
        let values: Vec<u64> = vec![];
        let avg = calculate_average(&values);
        assert_eq!(avg, 0.0);
    }
}