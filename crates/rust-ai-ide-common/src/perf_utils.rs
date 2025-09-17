use std::future::Future;
/// ! Performance monitoring and profiling utilities
/// !
/// ! Provides timing utilities, performance markers, and hooks for observability
/// ! integration. Designed to be lightweight and non-intrusive.
use std::time::{Duration, Instant};

/// Performance metrics structure
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub duration: Duration,
    pub started_at: Instant,
    pub finished_at: Instant,
}

/// Timing result from measuring an operation
#[derive(Debug, Clone)]
pub struct TimedOperation<T> {
    pub result: T,
    pub metrics: PerformanceMetrics,
}

/// Timer for measuring operation duration
#[derive(Debug)]
pub struct Timer {
    started_at: Instant,
    operation_name: String,
}

impl Timer {
    /// Start a new timer
    pub fn start(operation_name: impl Into<String>) -> Self {
        Self {
            started_at: Instant::now(),
            operation_name: operation_name.into(),
        }
    }

    /// Complete the timer and return metrics
    pub fn finish(self) -> PerformanceMetrics {
        let finished_at = Instant::now();
        let duration = finished_at.duration_since(self.started_at);

        log::debug!("Performance: {} took {:?}", self.operation_name, duration);

        PerformanceMetrics {
            operation_name: self.operation_name,
            duration,
            started_at: self.started_at,
            finished_at,
        }
    }

    /// Create a scoped timer that finishes when dropped
    pub fn scoped(operation_name: impl Into<String>) -> ScopedTimer {
        ScopedTimer::new(operation_name.into())
    }
}

/// Scoped timer that automatically finishes when dropped
pub struct ScopedTimer {
    operation_name: String,
    started_at: Instant,
}

impl ScopedTimer {
    pub fn new(operation_name: String) -> Self {
        log::trace!("Starting operation: {}", operation_name);
        Self {
            operation_name,
            started_at: Instant::now(),
        }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let duration = Instant::now().duration_since(self.started_at);
        log::trace!(
            "Operation {} completed in {:?}",
            self.operation_name,
            duration
        );
    }
}

/// Measure the execution time of a synchronous operation
pub fn time_operation<T, F>(operation_name: impl Into<String>, operation: F) -> TimedOperation<T>
where
    F: FnOnce() -> T,
{
    let timer = Timer::start(operation_name);
    let result = operation();
    let metrics = timer.finish();

    TimedOperation { result, metrics }
}

/// Measure the execution time of an asynchronous operation
pub async fn time_async_operation<T, F, Fut>(
    operation_name: impl Into<String>,
    operation: F,
) -> TimedOperation<T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let timer = Timer::start(operation_name);
    let result = operation().await;
    let metrics = timer.finish();

    TimedOperation { result, metrics }
}

/// Performance marker for observability
#[derive(Debug, Clone)]
pub struct PerformanceMarker {
    pub name: String,
    pub tags: std::collections::HashMap<String, String>,
    pub timestamp: Instant,
}

impl PerformanceMarker {
    /// Create a new performance marker
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tags: std::collections::HashMap::new(),
            timestamp: Instant::now(),
        }
    }

    /// Add a tag to the marker
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: std::collections::HashMap<String, String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Emit the marker
    pub fn emit(self) {
        // In a real implementation, this would send to monitoring system
        log::info!(
            "Performance marker: {} at {:?} with tags {:?}",
            self.name,
            self.timestamp,
            self.tags
        );
    }
}

/// Profiling markers for common code sections
pub mod markers {
    use super::PerformanceMarker;

    /// Mark LSP request start
    pub fn lsp_request_start(request_type: &str) -> PerformanceMarker {
        PerformanceMarker::new(format!("lsp_request_start_{}", request_type))
            .with_tag("request_type", request_type)
    }

    /// Mark LSP request complete
    pub fn lsp_request_complete(
        request_type: &str,
        duration: std::time::Duration,
    ) -> PerformanceMarker {
        PerformanceMarker::new(format!("lsp_request_complete_{}", request_type))
            .with_tag("request_type", request_type)
            .with_tag("duration_ms", duration.as_millis().to_string())
    }

    /// Mark cache operation
    pub fn cache_operation(operation: &str, hit: bool) -> PerformanceMarker {
        PerformanceMarker::new("cache_operation")
            .with_tag("operation", operation)
            .with_tag("hit", hit.to_string())
    }

    /// Mark filesystem operation
    pub fn filesystem_operation(operation: &str, bytes: Option<u64>) -> PerformanceMarker {
        let mut marker =
            PerformanceMarker::new("filesystem_operation").with_tag("operation", operation);

        if let Some(bytes) = bytes {
            marker = marker.with_tag("bytes", bytes.to_string());
        }

        marker
    }

    /// Mark workspace operation
    pub fn workspace_operation(operation: &str, path_count: Option<usize>) -> PerformanceMarker {
        let mut marker =
            PerformanceMarker::new("workspace_operation").with_tag("operation", operation);

        if let Some(count) = path_count {
            marker = marker.with_tag("path_count", count.to_string());
        }

        marker
    }
}

/// Collect performance metrics for a batch of operations
#[derive(Debug, Default)]
pub struct PerformanceCollector {
    pub operations: Vec<PerformanceMetrics>,
    pub total_duration: Duration,
}

impl PerformanceCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a metrics sample
    pub fn add_sample(&mut self, metrics: PerformanceMetrics) {
        let duration = metrics.duration;
        self.operations.push(metrics);
        self.total_duration += duration;
    }

    /// Get average duration
    pub fn average_duration(&self) -> Option<Duration> {
        if self.operations.is_empty() {
            None
        } else {
            Some(self.total_duration / self.operations.len() as u32)
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> PerformanceSummary {
        if self.operations.is_empty() {
            return PerformanceSummary::default();
        }

        let mut durations: Vec<_> = self.operations.iter().map(|m| m.duration).collect();
        durations.sort();

        // Simple percentile calculation
        let count = durations.len();
        let p50 = durations[count / 2];
        let p95 = durations[(count as f64 * 0.95) as usize].min(durations[count - 1]);
        let p99 = durations[(count as f64 * 0.99) as usize].min(durations[count - 1]);

        PerformanceSummary {
            count,
            total_duration: self.total_duration,
            average: self.total_duration / count as u32,
            p50,
            p95,
            p99,
        }
    }

    /// Reset the collector
    pub fn reset(&mut self) {
        self.operations.clear();
        self.total_duration = Duration::default();
    }
}

/// Summary of performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub count: usize,
    pub total_duration: Duration,
    pub average: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            count: 0,
            total_duration: Duration::ZERO,
            average: Duration::ZERO,
            p50: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
        }
    }
}
