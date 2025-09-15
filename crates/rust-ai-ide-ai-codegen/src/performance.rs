//! Performance monitoring and optimization for AI code generation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

/// Performance metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total operations performed
    pub total_operations:      u64,
    /// Total time spent (nanoseconds)
    pub total_time_ns:         u128,
    /// Average operation time (nanoseconds)
    pub avg_operation_time_ns: u128,
    /// Maximum operation time (nanoseconds)
    pub max_operation_time_ns: u128,
    /// Minimum operation time (nanoseconds)
    pub min_operation_time_ns: u128,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate:        f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes:    u64,
    /// Operations per second
    pub ops_per_second:        f64,
}

/// Performance monitor for tracking operations
pub struct PerformanceMonitor {
    metrics:    Arc<Mutex<HashMap<String, PerformanceMetrics>>>,
    start_time: Instant,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics:    Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a cache hit
    pub async fn record_cache_hit(&self) {
        let mut metrics = self.metrics.lock().await;
        let cache_metrics = metrics.entry("cache".to_string()).or_default();
        cache_metrics.total_operations += 1;
        // Cache hits don't count as full operations for timing
    }

    /// Record a generation operation
    pub async fn record_generation(&self, duration_ns: u64) {
        self.record_operation("generation", duration_ns as u128)
            .await;
    }

    /// Record a completion operation
    pub async fn record_completion(&self, duration_ns: u64) {
        self.record_operation("completion", duration_ns as u128)
            .await;
    }

    /// Record a refactoring operation
    pub async fn record_refactoring(&self, duration_ns: u64) {
        self.record_operation("refactoring", duration_ns as u128)
            .await;
    }

    /// Record a documentation generation operation
    pub async fn record_documentation_generation(&self, duration_ns: u64) {
        self.record_operation("documentation", duration_ns as u128)
            .await;
    }

    /// Record a test generation operation
    pub async fn record_test_generation(&self, duration_ns: u64) {
        self.record_operation("test_generation", duration_ns as u128)
            .await;
    }

    /// Record an analysis operation
    pub async fn record_analysis(&self, duration_ns: u64) {
        self.record_operation("analysis", duration_ns as u128).await;
    }

    /// Record a generic operation
    pub async fn record_operation(&self, operation_type: &str, duration_ns: u128) {
        let mut metrics = self.metrics.lock().await;
        let op_metrics = metrics.entry(operation_type.to_string()).or_default();

        op_metrics.total_operations += 1;
        op_metrics.total_time_ns += duration_ns;

        if duration_ns > op_metrics.max_operation_time_ns {
            op_metrics.max_operation_time_ns = duration_ns;
        }

        if op_metrics.min_operation_time_ns == 0 || duration_ns < op_metrics.min_operation_time_ns {
            op_metrics.min_operation_time_ns = duration_ns;
        }

        if op_metrics.total_operations > 0 {
            op_metrics.avg_operation_time_ns = op_metrics.total_time_ns / op_metrics.total_operations as u128;
        }

        // Calculate operations per second
        let elapsed_seconds = self.start_time.elapsed().as_secs_f64();
        if elapsed_seconds > 0.0 {
            op_metrics.ops_per_second = op_metrics.total_operations as f64 / elapsed_seconds;
        }
    }

    /// Get performance metrics for an operation type
    pub async fn get_metrics(&self, operation_type: &str) -> PerformanceMetrics {
        let metrics = self.metrics.lock().await;
        metrics.get(operation_type).cloned().unwrap_or_default()
    }

    /// Get all performance metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, PerformanceMetrics> {
        self.metrics.lock().await.clone()
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.clear();
        // Note: We don't reset start_time to maintain uptime tracking
    }

    /// Generate a performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let all_metrics = self.get_all_metrics().await;
        let uptime_seconds = self.start_time.elapsed().as_secs_f64();
        let total_operations = all_metrics.values().map(|m| m.total_operations).sum();

        PerformanceReport {
            uptime_seconds,
            operation_metrics: all_metrics,
            total_operations,
            average_response_time_ms: self.calculate_average_response_time(&HashMap::new()), // Placeholder
            peak_memory_usage_mb: self.estimate_memory_usage().await,
            recommendations: Vec::new(), // Placeholder
        }
    }

    fn calculate_average_response_time(&self, metrics: &HashMap<String, PerformanceMetrics>) -> f64 {
        let mut total_time = 0u128;
        let mut total_ops = 0u64;

        for metric in metrics.values() {
            total_time += metric.total_time_ns;
            total_ops += metric.total_operations;
        }

        if total_ops > 0 {
            (total_time as f64 / total_ops as f64) / 1_000_000.0 // Convert to milliseconds
        } else {
            0.0
        }
    }

    async fn estimate_memory_usage(&self) -> f64 {
        // This is a simplified estimation - in a real implementation,
        // you would use system monitoring or profiling tools

        // Estimate based on cache size and operation count
        let metrics = self.metrics.lock().await;
        let total_operations = metrics.values().map(|m| m.total_operations).sum::<u64>();

        // Rough estimation: ~1KB per operation for internal state
        (total_operations as f64 * 1024.0) / (1024.0 * 1024.0) // Convert to MB
    }

    async fn generate_recommendations(&self, metrics: &HashMap<String, PerformanceMetrics>) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for slow operations
        for (operation_type, metric) in metrics {
            let avg_time_ms = (metric.avg_operation_time_ns as f64) / 1_000_000.0;

            if avg_time_ms > 2000.0 {
                // More than 2 seconds
                recommendations.push(format!(
                    "Consider optimizing {} operations (avg: {:.2}ms)",
                    operation_type, avg_time_ms
                ));
            }
        }

        // Check cache hit rates
        if let Some(cache_metric) = metrics.get("cache") {
            let hit_rate = cache_metric.cache_hit_rate;
            if hit_rate < 0.5 {
                recommendations.push(format!(
                    "Low cache hit rate ({:.2}%) - consider increasing cache size or adjusting TTL",
                    hit_rate * 100.0
                ));
            }
        }

        // Check memory usage
        let memory_mb = self.estimate_memory_usage().await;
        if memory_mb > 100.0 {
            recommendations.push(format!(
                "High memory usage ({:.2}MB) - consider implementing memory optimization strategies",
                memory_mb
            ));
        }

        recommendations
    }
}

/// Performance report for analysis
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// System uptime in seconds
    pub uptime_seconds:           f64,
    /// Metrics per operation type
    pub operation_metrics:        HashMap<String, PerformanceMetrics>,
    /// Total operations across all types
    pub total_operations:         u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Peak memory usage in MB
    pub peak_memory_usage_mb:     f64,
    /// Performance recommendations
    pub recommendations:          Vec<String>,
}

impl std::fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Performance Report")?;
        writeln!(f, "==================")?;
        writeln!(f, "Uptime: {:.2}s", self.uptime_seconds)?;
        writeln!(f, "Total Operations: {}", self.total_operations)?;
        writeln!(
            f,
            "Average Response Time: {:.2}ms",
            self.average_response_time_ms
        )?;
        writeln!(f, "Peak Memory Usage: {:.2}MB", self.peak_memory_usage_mb)?;

        if !self.operation_metrics.is_empty() {
            writeln!(f, "\nOperation Breakdown:")?;
            for (op_type, metrics) in &self.operation_metrics {
                let avg_time_ms = (metrics.avg_operation_time_ns as f64) / 1_000_000.0;
                writeln!(
                    f,
                    "  {}: {} ops, avg {:.2}ms",
                    op_type, metrics.total_operations, avg_time_ms
                )?;
            }
        }

        if !self.recommendations.is_empty() {
            writeln!(f, "\nRecommendations:")?;
            for rec in &self.recommendations {
                writeln!(f, "  - {}", rec)?;
            }
        }

        Ok(())
    }
}

/// Performance profiler for detailed analysis
pub struct PerformanceProfiler {
    samples: Arc<Mutex<Vec<PerformanceSample>>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub operation_type: String,
    pub start_time:     Instant,
    pub duration:       Option<Duration>,
    pub memory_before:  u64,
    pub memory_after:   Option<u64>,
    pub metadata:       HashMap<String, String>,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start profiling an operation
    pub async fn start_operation(&self, operation_type: &str) -> OperationProfile {
        let memory_before = self.get_memory_usage();
        let start_time = Instant::now();

        OperationProfile {
            operation_type: operation_type.to_string(),
            start_time,
            memory_before,
            profiler: Arc::clone(&self.samples),
        }
    }

    /// Get all performance samples
    pub async fn get_samples(&self) -> Vec<PerformanceSample> {
        self.samples.lock().await.clone()
    }

    /// Generate profiling report
    pub async fn generate_profiling_report(&self) -> ProfilingReport {
        let samples = self.get_samples().await;

        let mut operation_stats: HashMap<String, OperationStats> = HashMap::new();

        for sample in &samples {
            let stats = operation_stats
                .entry(sample.operation_type.clone())
                .or_default();

            if let Some(duration) = sample.duration {
                stats.call_count += 1;
                stats.total_time += duration;
                stats.avg_time = stats.total_time / stats.call_count as u32;

                if duration > stats.max_time {
                    stats.max_time = duration;
                }

                if stats.min_time == Duration::default() || duration < stats.min_time {
                    stats.min_time = duration;
                }
            }

            if let Some(memory_after) = sample.memory_after {
                stats.memory_delta += memory_after as i64 - sample.memory_before as i64;
            }
        }

        ProfilingReport {
            total_samples: samples.len(),
            operation_stats,
        }
    }

    fn get_memory_usage(&self) -> u64 {
        // This is a placeholder - in a real implementation, you would use
        // system monitoring or Rust's profiling tools
        0
    }
}

/// Profile for a single operation
pub struct OperationProfile {
    operation_type: String,
    start_time:     Instant,
    memory_before:  u64,
    profiler:       Arc<Mutex<Vec<PerformanceSample>>>,
}

impl Drop for OperationProfile {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let memory_after = 0; // Placeholder

        let sample = PerformanceSample {
            operation_type: self.operation_type.clone(),
            start_time:     self.start_time,
            duration:       Some(duration),
            memory_before:  self.memory_before,
            memory_after:   Some(memory_after),
            metadata:       HashMap::new(),
        };

        // This is not ideal for async, but works for demonstration
        // In a real implementation, you'd want to handle this differently
        let profiler = Arc::clone(&self.profiler);
        tokio::spawn(async move {
            let mut samples = profiler.lock().await;
            samples.push(sample);
        });
    }
}

#[derive(Debug, Clone, Default)]
pub struct OperationStats {
    pub call_count:   usize,
    pub total_time:   Duration,
    pub avg_time:     Duration,
    pub max_time:     Duration,
    pub min_time:     Duration,
    pub memory_delta: i64,
}

#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub total_samples:   usize,
    pub operation_stats: HashMap<String, OperationStats>,
}

impl std::fmt::Display for ProfilingReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Profiling Report")?;
        writeln!(f, "================")?;
        writeln!(f, "Total Samples: {}", self.total_samples)?;

        for (operation, stats) in &self.operation_stats {
            writeln!(f, "\nOperation: {}", operation)?;
            writeln!(f, "  Call Count: {}", stats.call_count)?;
            writeln!(f, "  Total Time: {:.2}ms", stats.total_time.as_millis())?;
            writeln!(f, "  Average Time: {:.2}ms", stats.avg_time.as_millis())?;
            writeln!(f, "  Max Time: {:.2}ms", stats.max_time.as_millis())?;
            writeln!(f, "  Min Time: {:.2}ms", stats.min_time.as_millis())?;
            writeln!(f, "  Memory Delta: {} bytes", stats.memory_delta)?;
        }

        Ok(())
    }
}

/// Performance optimization strategies
pub enum OptimizationStrategy {
    /// Increase cache size
    IncreaseCacheSize(u64),
    /// Adjust cache TTL
    AdjustCacheTtl(Duration),
    /// Enable background processing
    EnableBackgroundProcessing,
    /// Implement connection pooling
    ImplementConnectionPooling,
    /// Optimize memory usage
    OptimizeMemoryUsage,
    /// Add operation batching
    AddOperationBatching,
}

/// Performance optimizer
pub struct PerformanceOptimizer {
    monitor:  Arc<PerformanceMonitor>,
    profiler: Arc<PerformanceProfiler>,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(monitor: Arc<PerformanceMonitor>, profiler: Arc<PerformanceProfiler>) -> Self {
        Self { monitor, profiler }
    }

    /// Analyze performance and suggest optimizations
    pub async fn analyze_and_optimize(&self) -> Vec<OptimizationStrategy> {
        let report = self.monitor.generate_report().await;
        let profiling_report = self.profiler.generate_profiling_report().await;

        let mut strategies = Vec::new();

        // Analyze cache performance
        if report.average_response_time_ms > 1000.0 {
            strategies.push(OptimizationStrategy::IncreaseCacheSize(2000));
        }

        // Analyze memory usage
        if report.peak_memory_usage_mb > 50.0 {
            strategies.push(OptimizationStrategy::OptimizeMemoryUsage);
        }

        // Analyze operation patterns
        for (operation, stats) in &profiling_report.operation_stats {
            if stats.call_count > 100 && stats.avg_time > Duration::from_millis(500) {
                strategies.push(OptimizationStrategy::EnableBackgroundProcessing);
            }
        }

        strategies
    }

    /// Apply an optimization strategy
    pub async fn apply_strategy(&self, strategy: OptimizationStrategy) -> crate::error::Result<()> {
        match strategy {
            OptimizationStrategy::IncreaseCacheSize(new_size) => {
                log::info!("Increasing cache size to {}", new_size);
                // Implementation would modify cache configuration
            }
            OptimizationStrategy::AdjustCacheTtl(new_ttl) => {
                log::info!("Adjusting cache TTL to {:?}", new_ttl);
                // Implementation would modify cache configuration
            }
            OptimizationStrategy::EnableBackgroundProcessing => {
                log::info!("Enabling background processing");
                // Implementation would spawn background tasks
            }
            OptimizationStrategy::ImplementConnectionPooling => {
                log::info!("Implementing connection pooling");
                // Implementation would set up connection pools
            }
            OptimizationStrategy::OptimizeMemoryUsage => {
                log::info!("Optimizing memory usage");
                // Implementation would optimize memory allocation
            }
            OptimizationStrategy::AddOperationBatching => {
                log::info!("Adding operation batching");
                // Implementation would batch similar operations
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();

        // Record some operations
        monitor.record_generation(100_000_000).await; // 100ms
        monitor.record_generation(200_000_000).await; // 200ms

        let metrics = monitor.get_metrics("generation").await;

        assert_eq!(metrics.total_operations, 2);
        assert_eq!(metrics.total_time_ns, 300_000_000);
        assert_eq!(metrics.avg_operation_time_ns, 150_000_000);
    }

    #[tokio::test]
    async fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new();

        {
            let _profile = profiler.start_operation("test_operation").await;
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let samples = profiler.get_samples().await;
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].operation_type, "test_operation");
        assert!(samples[0].duration.is_some());
    }
}
