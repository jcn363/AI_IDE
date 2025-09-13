//! Performance Profiling Utilities for Learning System
//!
//! This module provides runtime performance profiling and analysis
//! to track performance characteristics during development and testing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::models::{LearnedPattern, LearningPreferences};
use super::types::LearningResult;

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_count:  u64,
    pub total_duration:   Duration,
    pub average_duration: Duration,
    pub min_duration:     Duration,
    pub max_duration:     Duration,
    pub success_count:    u64,
    pub error_count:      u64,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            operation_count:  0,
            total_duration:   Duration::new(0, 0),
            average_duration: Duration::new(0, 0),
            min_duration:     Duration::MAX,
            max_duration:     Duration::new(0, 0),
            success_count:    0,
            error_count:      0,
        }
    }

    fn record_operation(&mut self, duration: Duration, success: bool) {
        self.operation_count += 1;
        self.total_duration += duration;

        if duration < self.min_duration {
            self.min_duration = duration;
        }

        if duration > self.max_duration {
            self.max_duration = duration;
        }

        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }

        self.average_duration = self.total_duration / self.operation_count as u32;
    }

    pub fn success_rate(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.operation_count as f64
        }
    }

    pub fn operations_per_second(&self) -> f64 {
        let total_secs = self.total_duration.as_secs_f64();
        if total_secs > 0.0 {
            self.operation_count as f64 / total_secs
        } else {
            0.0
        }
    }
}

impl std::fmt::Display for PerformanceMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Performance Metrics:\n")?;
        write!(f, "  Operations: {}\n", self.operation_count)?;
        write!(
            f,
            "  Total Time: {:.2}s\n",
            self.total_duration.as_secs_f64()
        )?;
        write!(
            f,
            "  Average Time: {:.2}ms\n",
            self.average_duration.as_millis()
        )?;
        write!(f, "  Min Time: {:.2}ms\n", self.min_duration.as_millis())?;
        write!(f, "  Max Time: {:.2}ms\n", self.max_duration.as_millis())?;
        write!(f, "  Success Rate: {:.1}%\n", self.success_rate() * 100.0)?;
        write!(f, "  Ops/Sec: {:.0}\n", self.operations_per_second())?;
        Ok(())
    }
}

/// Performance profiler for tracking system performance
pub struct PerformanceProfiler {
    metrics:     Mutex<HashMap<String, PerformanceMetrics>>,
    start_times: Mutex<HashMap<String, Instant>>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            metrics:     Mutex::new(HashMap::new()),
            start_times: Mutex::new(HashMap::new()),
        }
    }

    pub fn start_operation(&self, operation_name: &str) {
        let mut start_times = self.start_times.lock().unwrap();
        start_times.insert(operation_name.to_string(), Instant::now());
    }

    pub fn end_operation(&self, operation_name: &str, success: bool) {
        let mut start_times = self.start_times.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(start_time) = start_times.remove(operation_name) {
            let duration = start_time.elapsed();
            let operation_name = operation_name.to_string();

            let operation_metrics = metrics
                .entry(operation_name)
                .or_insert_with(PerformanceMetrics::new);
            operation_metrics.record_operation(duration, success);
        }
    }

    pub fn get_metrics(&self, operation_name: &str) -> Option<PerformanceMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(operation_name).cloned()
    }

    pub fn get_all_metrics(&self) -> HashMap<String, PerformanceMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }

    pub fn reset(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        let mut start_times = self.start_times.lock().unwrap();
        metrics.clear();
        start_times.clear();
    }

    pub fn print_report(&self) {
        let metrics = self.get_all_metrics();

        if metrics.is_empty() {
            println!("No performance data collected.");
            return;
        }

        println!("=== Performance Profile Report ===");
        println!("Total operations tracked: {}", metrics.len());
        println!();

        for (operation, op_metrics) in metrics {
            println!("Operation: {}", operation);
            println!("{}", op_metrics);
            println!("-----------------------------------");
        }
    }
}

/// Helper macro for performance profiling
#[macro_export]
macro_rules! profile_operation {
    ($profiler:expr, $operation:expr, $code:block) => {{
        $profiler.start_operation($operation);
        let result = $code;
        match &result {
            Ok(_) => $profiler.end_operation($operation, true),
            Err(_) => $profiler.end_operation($operation, false),
        }
        result
    }};
}

/// Memory usage profiler
pub struct MemoryProfiler {
    pub initial_memory: usize,
    pub peak_memory:    usize,
    pub current_memory: usize,
    pub allocations:    usize,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            initial_memory: 0,
            peak_memory:    0,
            current_memory: 0,
            allocations:    0,
        }
    }

    pub fn start_tracking(&mut self) {
        // In a real implementation, this would use system APIs to track memory
        // For now, this is just a placeholder structure
        self.initial_memory = 0;
        self.peak_memory = 0;
    }

    pub fn record_memory_usage(&mut self, memory_used: usize) {
        self.current_memory = memory_used;
        if memory_used > self.peak_memory {
            self.peak_memory = memory_used;
        }
    }

    pub fn print_memory_report(&self) {
        println!("Memory Usage Report:");
        println!("  Initial: {} bytes", self.initial_memory);
        println!("  Peak: {} bytes", self.peak_memory);
        println!("  Current: {} bytes", self.current_memory);
        println!(
            "  Memory Growth: {} bytes",
            self.current_memory as isize - self.initial_memory as isize
        );
    }
}

/// Generic profiler that tracks both performance and memory
pub struct AdvancedProfiler {
    performance_profiler: PerformanceProfiler,
    memory_profiler:      Mutex<MemoryProfiler>,
}

impl AdvancedProfiler {
    pub fn new() -> Self {
        Self {
            performance_profiler: PerformanceProfiler::new(),
            memory_profiler:      Mutex::new(MemoryProfiler::new()),
        }
    }

    pub fn profile<T, F>(&self, operation_name: &str, f: F) -> LearningResult<T>
    where
        F: FnOnce() -> LearningResult<T>,
    {
        self.performance_profiler.start_operation(operation_name);

        let result = f();

        match &result {
            Ok(_) => self
                .performance_profiler
                .end_operation(operation_name, true),
            Err(_) => self
                .performance_profiler
                .end_operation(operation_name, false),
        }

        result
    }

    pub fn print_comprehensive_report(&self) {
        println!("=== Comprehensive Performance Report ===\n");

        println!("Performance Metrics:");
        println!("-------------------");
        self.performance_profiler.print_report();

        println!("\nMemory Metrics:");
        println!("---------------");
        let memory_profiler = self.memory_profiler.lock().unwrap();
        memory_profiler.print_memory_report();

        println!("\n=== End Report ===");
    }

    pub fn get_performance_profiler(&self) -> &PerformanceProfiler {
        &self.performance_profiler
    }
}

// Global profiler instance for application-wide performance tracking
lazy_static::lazy_static! {
    pub static ref GLOBAL_PROFILER: AdvancedProfiler = AdvancedProfiler::new();
}

/// Convenience function for quick performance profiling
pub fn quick_profile<T, F>(operation_name: &str, f: F) -> LearningResult<T>
where
    F: FnOnce() -> LearningResult<T>,
{
    GLOBAL_PROFILER.profile(operation_name, f)
}

/// Quick profiling macro with global profiler
#[macro_export]
macro_rules! quick_profile {
    ($operation:expr, $code:block) => {{
        use crate::performance_profile::GLOBAL_PROFILER;
        GLOBAL_PROFILER.profile($operation, || $code)
    }};
}

/// System health checker that combines performance metrics with health indicators
pub struct SystemHealthChecker {
    profiler:           Arc<AdvancedProfiler>,
    warning_thresholds: HealthThresholds,
}

#[derive(Debug, Clone)]
pub struct HealthThresholds {
    pub max_average_response_time_ms: u128,
    pub min_success_rate:             f64,
    pub max_error_rate:               f64,
    pub max_memory_usage_mb:          usize,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_average_response_time_ms: 1000, // 1 second
            min_success_rate:             0.95, // 95% success rate
            max_error_rate:               0.05, // 5% error rate
            max_memory_usage_mb:          100,  // 100MB
        }
    }
}

impl SystemHealthChecker {
    pub fn new(profiler: Arc<AdvancedProfiler>) -> Self {
        Self {
            profiler,
            warning_thresholds: HealthThresholds::default(),
        }
    }

    pub fn check_system_health(&self, operation_name: &str) -> HealthStatus {
        let metrics = self
            .profiler
            .performance_profiler
            .get_metrics(operation_name);

        match metrics {
            Some(metrics) => {
                let success_rate = metrics.success_rate();
                let avg_response_time = metrics.average_duration.as_millis();
                let error_rate = metrics.error_count as f64 / metrics.operation_count as f64;

                let mut warnings = Vec::new();
                let mut critical_issues = Vec::new();

                if avg_response_time > self.warning_thresholds.max_average_response_time_ms {
                    critical_issues.push(format!(
                        "Response time too high: {} ms > {} ms",
                        avg_response_time, self.warning_thresholds.max_average_response_time_ms
                    ));
                }

                if success_rate < self.warning_thresholds.min_success_rate {
                    critical_issues.push(format!(
                        "Success rate too low: {:.1}% < {:.1}%",
                        success_rate * 100.0,
                        self.warning_thresholds.min_success_rate * 100.0
                    ));
                }

                if error_rate > self.warning_thresholds.max_error_rate {
                    warnings.push(format!(
                        "Error rate high: {:.1}% > {:.1}%",
                        error_rate * 100.0,
                        self.warning_thresholds.max_error_rate * 100.0
                    ));
                }

                if critical_issues.is_empty() && warnings.is_empty() {
                    HealthStatus::Healthy
                } else if !critical_issues.is_empty() {
                    HealthStatus::Critical(critical_issues)
                } else {
                    HealthStatus::Warning(warnings)
                }
            }
            None => HealthStatus::Unknown,
        }
    }

    pub fn print_health_report(&self, operations: &[&str]) {
        println!("=== System Health Report ===");

        for operation in operations {
            match self.check_system_health(operation) {
                HealthStatus::Healthy => {
                    println!("✅ {}: Healthy", operation);
                }
                HealthStatus::Warning(warnings) => {
                    println!("⚠️  {}: Warning", operation);
                    for warning in warnings {
                        println!("   - {}", warning);
                    }
                }
                HealthStatus::Critical(issues) => {
                    println!("❌ {}: Critical", operation);
                    for issue in issues {
                        println!("   - {}", issue);
                    }
                }
                HealthStatus::Unknown => {
                    println!("❓ {}: Unknown (no data)", operation);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Warning(Vec<String>),
    Critical(Vec<String>),
    Unknown,
}

/// Benchmark-style performance test
pub async fn run_performance_test() -> LearningResult<()> {
    println!("Starting Learning System Performance Test...");

    let profiler = Arc::new(AdvancedProfiler::new());
    let health_checker = SystemHealthChecker::new(Arc::clone(&profiler));

    // Test different operations
    let test_operations = ["stored_pattern", "similarity_search", "cache_hit"];

    // Simulate operations
    println!("\nRunning simulated operations...");

    for i in 0..100 {
        // Simulate database operations
        let _result = profiler
            .profile("database_operation", || {
                // Simulate some work
                std::thread::sleep(std::time::Duration::from_micros((i % 10) as u64 * 100));
                Ok(())
            })
            .unwrap();
    }

    // Simulate similarity searches
    for i in 0..50 {
        let _result = profiler
            .profile("similarity_search", || {
                std::thread::sleep(std::time::Duration::from_micros((i % 5) as u64 * 200));
                if i % 10 == 0 {
                    Err(super::types::LearningError::ConfigurationError(
                        "Simulated error".to_string(),
                    ))
                } else {
                    Ok(())
                }
            })
            .unwrap_err(); // Should handle errors properly
    }

    println!("\n=== Performance Test Results ===");
    profiler.print_comprehensive_report();

    println!("\n=== Health Check Results ===");
    health_checker.print_health_report(&test_operations);

    Ok(())
}

/// Quick performance test runner
pub fn run_quick_performance_test(db_size: usize, query_count: usize) {
    println!(
        "Quick Performance Test: {} patterns, {} queries",
        db_size, query_count
    );

    // This would be implemented with actual learning system operations
    println!("✅ Performance test framework initialized");
    println!("✅ Ready for benchmarking operations");
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use tempfile::TempDir;
    use tokio::runtime::Runtime;

    use super::*;

    #[test]
    fn test_performance_profiler_basic() {
        let profiler = AdvancedProfiler::new();

        let result = profiler.profile("test_operation", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(())
        });

        assert!(result.is_ok());

        let metrics = profiler
            .get_performance_profiler()
            .get_metrics("test_operation");
        assert!(metrics.is_some());
        assert_eq!(metrics.unwrap().operation_count, 1);
    }

    #[test]
    fn test_health_checker_basic() {
        let profiler = Arc::new(AdvancedProfiler::new());
        let health_checker = SystemHealthChecker::new(Arc::clone(&profiler));

        // Add some test data
        profiler
            .profile("test_op", || {
                std::thread::sleep(std::time::Duration::from_millis(1));
                Ok(())
            })
            .unwrap();

        let status = health_checker.check_system_health("test_op");
        match status {
            HealthStatus::Healthy => (),
            HealthStatus::Warning(_) => (),
            HealthStatus::Critical(_) => (),
            HealthStatus::Unknown => panic!("Should have metrics"),
        }
    }

    #[tokio::test]
    async fn test_async_performance_tracking() {
        let profiler = AdvancedProfiler::new();

        let result = profiler.profile("async_operation", || async {
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            Ok(())
        });

        let metrics = profiler
            .get_performance_profiler()
            .get_metrics("async_operation");
        assert!(metrics.is_some());
        assert!(metrics.unwrap().average_duration.as_millis() >= 5);
    }
}
