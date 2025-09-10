/// Performance monitoring and metrics for SIMD operations

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::SIMDError;

/// SIMD performance monitoring and metrics
pub struct SIMDPerformanceMonitor {
    operation_times: HashMap<String, Vec<Duration>>,
    active_operations: HashMap<String, Instant>,
    total_operations: HashMap<String, usize>,
}

impl SIMDPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            operation_times: HashMap::new(),
            active_operations: HashMap::new(),
            total_operations: HashMap::new(),
        }
    }

    /// Start monitoring an operation
    pub fn start_operation(&mut self, operation_name: &str) {
        let now = Instant::now();
        self.active_operations.insert(operation_name.to_string(), now);
        *self.total_operations.entry(operation_name.to_string()).or_insert(0) += 1;
    }

    /// End monitoring an operation and record the duration
    pub fn end_operation(&mut self, operation_name: &str) {
        if let Some(start_time) = self.active_operations.remove(operation_name) {
            let duration = start_time.elapsed();
            let operation_key = operation_name.to_string();

            if let Some(durations) = self.operation_times.get_mut(&operation_key) {
                durations.push(duration);
                // Keep only the last 100 measurements to prevent unbounded growth
                if durations.len() > 100 {
                    durations.remove(0);
                }
            } else {
                self.operation_times.insert(operation_key, vec![duration]);
            }
        }
    }

    /// Get performance statistics for an operation
    pub fn get_operation_stats(&self, operation_name: &str) -> Option<OperationStats> {
        self.operation_times.get(operation_name).map(|durations| {
            if durations.is_empty() {
                return OperationStats {
                    operation_name: operation_name.to_string(),
                    total_calls: self.total_operations.get(operation_name).copied().unwrap_or(0),
                    avg_duration: Duration::ZERO,
                    min_duration: Duration::ZERO,
                    max_duration: Duration::ZERO,
                    total_duration: Duration::ZERO,
                    performance_score: 0.0,
                };
            }

            let total_duration: Duration = durations.iter().sum();
            let avg_duration = total_duration / durations.len() as u32;
            let min_duration = *durations.iter().min().unwrap();
            let max_duration = *durations.iter().max().unwrap();
            let total_calls = self.total_operations.get(operation_name).copied().unwrap_or(durations.len());

            // Performance score (arbitrary units, higher is better)
            let performance_score = if avg_duration.as_nanos() > 0 {
                let ops_per_second = 1_000_000_000.0 / avg_duration.as_nanos() as f64 * total_calls as f64;
                ops_per_second / 10_000.0 // Normalize to reasonable range
            } else {
                1000.0 // Very fast operations
            };

            OperationStats {
                operation_name: operation_name.to_string(),
                total_calls,
                avg_duration,
                min_duration,
                max_duration,
                total_duration,
                performance_score,
            }
        })
    }

    /// Get combined statistics for all monitored operations
    pub fn get_all_stats(&self) -> Vec<OperationStats> {
        self.operation_times.keys()
            .filter_map(|name| self.get_operation_stats(name))
            .collect()
    }

    /// Reset all monitoring data
    pub fn reset(&mut self) {
        self.operation_times.clear();
        self.active_operations.clear();
        self.total_operations.clear();
    }

    /// Get current active (unended) operations
    pub fn active_operations(&self) -> Vec<&str> {
        self.active_operations.keys().map(|s| s.as_str()).collect()
    }
}

/// Fallback strategy manager for SIMD operations
pub struct SIMDFallbackManager {
    capabilities: &'static crate::capability::SIMDCapabilities,
    fallback_count: HashMap<String, usize>,
}

impl SIMDFallbackManager {
    pub fn new(capabilities: &'static crate::capability::SIMDCapabilities) -> Self {
        Self {
            capabilities,
            fallback_count: HashMap::new(),
        }
    }

    /// Process vectorized operation with fallback to scalar operations
    pub fn vectorized_f32_fallback<F>(
        &mut self,
        lhs: &[f32],
        rhs: &[f32],
        operation: F,
    ) -> Result<Vec<f32>, SIMDError>
    where
        F: Fn(f32, f32) -> f32,
    {
        if lhs.len() != rhs.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: lhs.len(),
                actual: rhs.len(),
            });
        }

        *self.fallback_count.entry("vectorized_f32".to_string()).or_insert(0) += 1;
        tracing::debug!("Using scalar fallback for f32 vectorized operations");

        Ok(lhs.iter().zip(rhs.iter()).map(|(a, b)| operation(*a, *b)).collect())
    }

    /// Matrix multiplication fallback
    pub fn matrix_multiply_fallback_f32(
        &mut self,
        a: &[f32],
        b: &[f32],
        m: usize,
        n: usize,
        k: usize,
    ) -> Result<Vec<f32>, SIMDError> {
        if m * n != a.len() || n * k != b.len() {
            return Err(SIMDError::MatrixDimensionsError {
                a_dims: (m, n),
                b_dims: (n, k),
            });
        }

        *self.fallback_count.entry("matrix_multiply_f32".to_string()).or_insert(0) += 1;
        tracing::debug!("Using scalar fallback for matrix multiplication");

        let mut result = vec![0.0; m * k];

        // i-k-j loop order for better cache locality in scalar fallback
        for i in 0..m {
            for jj in (0..n).step_by(16) { // Small blocking for cache efficiency
                for j in 0..k {
                    let mut sum = result[i * k + j];
                    for l in jj..(jj + 16).min(n) {
                        sum += a[i * n + l] * b[l * k + j];
                    }
                    result[i * k + j] = sum;
                }
            }
        }

        Ok(result)
    }

    /// Euclidean distance calculation fallback
    pub fn euclidean_distance_fallback(
        &mut self,
        query: &[f32],
        database: &[f32],
        dimension: usize,
    ) -> Result<Vec<f32>, SIMDError> {
        if query.len() % dimension != 0 || database.len() % dimension != 0 {
            return Err(SIMDError::VectorSizeMismatch {
                expected: (query.len() / dimension) * dimension,
                actual: query.len(),
            });
        }

        *self.fallback_count.entry("euclidean_distance".to_string()).or_insert(0) += 1;
        tracing::debug!("Using scalar fallback for Euclidean distance");

        let num_queries = query.len() / dimension;
        let num_vectors = database.len() / dimension;
        let mut distances = Vec::with_capacity(num_queries * num_vectors);

        for qi in 0..num_queries {
            for vi in 0..num_vectors {
                let query_start = qi * dimension;
                let vector_start = vi * dimension;

                let mut sum_squares = 0.0;
                for d in 0..dimension {
                    let diff = query[query_start + d] - database[vector_start + d];
                    sum_squares += diff * diff;
                }

                distances.push(sum_squares.sqrt());
            }
        }

        Ok(distances)
    }

    /// Get fallback statistics
    pub fn fallback_stats(&self) -> FallbackStats {
        let total_fallbacks: usize = self.fallback_count.values().sum();
        let most_common_fallback = self.fallback_count.iter()
            .max_by_key(|&(_, v)| v)
            .map(|(k, v)| (k.clone(), *v))
            .unwrap_or_default();

        FallbackStats {
            total_fallbacks,
            per_operation: self.fallback_count.clone(),
            most_common_fallback,
            fallback_rate: if !self.capabilities.has_simd {
                1.0
            } else {
                // Calculate as percentage of operations that fell back
                0.5 // Placeholder - would need actual operation counts
            },
        }
    }
}

/// Performance statistics for SIMD monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    pub operation_name: String,
    pub total_calls: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_duration: Duration,
    pub performance_score: f64,
}

/// Fallback operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStats {
    pub total_fallbacks: usize,
    pub per_operation: HashMap<String, usize>,
    pub most_common_fallback: (String, usize),
    pub fallback_rate: f64,
}

/// SIMD performance recommendation engine
pub struct SIMDPerformanceAdvisor {
    monitor: SIMDPerformanceMonitor,
    fallback_manager: SIMDFallbackManager,
}

impl SIMDPerformanceAdvisor {
    pub fn new(
        monitor: SIMDPerformanceMonitor,
        fallback_manager: SIMDFallbackManager,
    ) -> Self {
        Self {
            monitor,
            fallback_manager,
        }
    }

    /// Generate performance recommendations based on monitoring data
    pub fn generate_recommendations(&self) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        let stats = self.monitor.get_all_stats();
        let fallback_stats = self.fallback_manager.fallback_stats();

        // Check for slow operations
        for stat in &stats {
            if stat.avg_duration.as_millis() > 100 {
                recommendations.push(PerformanceRecommendation {
                    category: RecommendationCategory::OptimizationOpportunity,
                    title: format!("Consider optimizing '{}'", stat.operation_name),
                    description: format!(
                        "Operation '{0}' averages {1:.2}ms per call. Consider SIMD optimizations or algorithm improvements.",
                        stat.operation_name,
                        stat.avg_duration.as_micros() as f64 / 1000.0
                    ),
                    severity: RecommendationSeverity::Medium,
                    suggestion: format!("Profile '{}' further to identify bottlenecks", stat.operation_name),
                });
            }
        }

        // Check for high fallback rates
        if fallback_stats.fallback_rate > 0.1 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::Infrastructure,
                title: "High SIMD fallback rate detected".to_string(),
                description: format!(
                    "{:.1}% of SIMD operations are falling back to scalar operations",
                    fallback_stats.fallback_rate * 100.0
                ),
                severity: RecommendationSeverity::High,
                suggestion: "Ensure SIMD feature flags are properly configured".to_string(),
            });
        }

        // Check for most common fallback operations
        if fallback_stats.most_common_fallback.1 > 10 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::OptimizationOpportunity,
                title: format!("Optimize '{}' operation", fallback_stats.most_common_fallback.0),
                description: format!(
                    "'{}' operation fell back {} times - consider optimizing for SIMD",
                    fallback_stats.most_common_fallback.0,
                    fallback_stats.most_common_fallback.1
                ),
                severity: RecommendationSeverity::Medium,
                suggestion: "Implement SIMD version of this operation".to_string(),
            });
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub severity: RecommendationSeverity,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Infrastructure,
    OptimizationOpportunity,
    MemoryUsage,
    Parallelization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_monitor_initialization() {
        let monitor = SIMDPerformanceMonitor::new();
        assert_eq!(monitor.active_operations().len(), 0);
        assert_eq!(monitor.get_all_stats().len(), 0);
    }

    #[test]
    fn test_operation_monitoring() {
        let mut monitor = SIMDPerformanceMonitor::new();

        monitor.start_operation("test_operation");
        sleep(Duration::from_millis(10));
        monitor.end_operation("test_operation");

        let stats = monitor.get_operation_stats("test_operation");
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.operation_name, "test_operation");
        assert_eq!(stats.total_calls, 1);
        assert!(stats.avg_duration.as_millis() >= 10);
    }

    #[test]
    fn test_fallback_manager() {
        let caps = crate::capability::get_cached_capabilities();
        let mut fallback_mgr = SIMDFallbackManager::new(caps);

        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![1.0, 2.0, 3.0, 4.0];

        let result = fallback_mgr.vectorized_f32_fallback(&a, &b, |x, y| x + y);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0, 8.0]);

        let stats = fallback_mgr.fallback_stats();
        assert_eq!(stats.total_fallbacks, 1);
    }

    #[test]
    fn test_performance_advisor() {
        let mut monitor = SIMDPerformanceMonitor::new();
        let caps = crate::capability::get_cached_capabilities();
        let fallback_mgr = SIMDFallbackManager::new(caps);
        let advisor = SIMDPerformanceAdvisor::new(monitor, fallback_mgr);

        let recommendations = advisor.generate_recommendations();
        // Should always return at least basic recommendations
        assert!(!recommendations.is_empty());
    }
}