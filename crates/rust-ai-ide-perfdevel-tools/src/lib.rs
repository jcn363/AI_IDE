//! Performance benchmarking and development tools for Rust AI IDE

use std::time::Duration;

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Name of the operation being measured
    pub operation: String,
    /// Duration of the operation
    pub duration: Duration,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl PerformanceMetrics {
    /// Create a new performance metrics instance
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            duration: Duration::default(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the duration of the operation
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add metadata to the metrics
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Benchmark a function's execution time
#[cfg(feature = "benchmarking")]
pub fn benchmark<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    use std::time::Instant;

    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();

    if cfg!(feature = "metrics_export") {
        // In a real implementation, this would export the metrics
        println!("Benchmark '{}' took: {:?}", name, duration);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new("test_operation")
            .with_duration(Duration::from_millis(100))
            .with_metadata("key", "value");

        assert_eq!(metrics.operation, "test_operation");
        assert_eq!(metrics.duration.as_millis(), 100);
        assert_eq!(metrics.metadata.get("key").unwrap(), "value");
    }
}
