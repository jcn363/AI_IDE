//! Test Performance Analyzer
//!
//! This crate provides performance analysis and testing utilities for the Rust AI IDE.
//! It includes benchmarking tools, performance regression detection, and analysis reporting.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub duration: Duration,
    pub memory_usage: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Performance analyzer for running and collecting test metrics
#[derive(Debug)]
pub struct PerformanceAnalyzer {
    results: Vec<PerformanceTestResult>,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run a performance test with the given name and test function
    pub async fn run_test<F, Fut>(&mut self, test_name: &str, test_fn: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();

        let result = match test_fn().await {
            Ok(()) => PerformanceTestResult {
                test_name: test_name.to_string(),
                duration: start_time.elapsed(),
                memory_usage: self.get_memory_usage() - start_memory,
                success: true,
                error_message: None,
            },
            Err(e) => PerformanceTestResult {
                test_name: test_name.to_string(),
                duration: start_time.elapsed(),
                memory_usage: self.get_memory_usage() - start_memory,
                success: false,
                error_message: Some(e.to_string()),
            },
        };

        self.results.push(result);
        Ok(())
    }

    /// Get current memory usage in bytes
    fn get_memory_usage(&self) -> u64 {
        // Simple memory usage estimation
        // In a real implementation, this would use more sophisticated memory tracking
        use sysinfo::{ProcessExt, System, SystemExt};
        
        let mut system = System::new();
        system.refresh_process(sysinfo::get_current_pid().unwrap());
        
        if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
            process.memory() * 1024 // Convert from KB to bytes
        } else {
            0
        }
    }

    /// Get all test results
    pub fn get_results(&self) -> &[PerformanceTestResult] {
        &self.results
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let total_tests = self.results.len();
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - successful_tests;

        let avg_duration = if !self.results.is_empty() {
            self.results.iter().map(|r| r.duration).sum::<Duration>() / total_tests as u32
        } else {
            Duration::from_secs(0)
        };

        let total_memory = self.results.iter().map(|r| r.memory_usage).sum();

        PerformanceReport {
            total_tests,
            successful_tests,
            failed_tests,
            average_duration: avg_duration,
            total_memory_usage: total_memory,
            results: self.results.clone(),
        }
    }

    /// Clear all results
    pub fn clear_results(&mut self) {
        self.results.clear();
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub average_duration: Duration,
    pub total_memory_usage: u64,
    pub results: Vec<PerformanceTestResult>,
}

impl PerformanceReport {
    /// Export report as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Print a summary of the report
    pub fn print_summary(&self) {
        println!("=== Performance Test Report ===");
        println!("Total tests: {}", self.total_tests);
        println!("Successful: {}", self.successful_tests);
        println!("Failed: {}", self.failed_tests);
        println!("Average duration: {:?}", self.average_duration);
        println!("Total memory usage: {} bytes", self.total_memory_usage);
        
        if self.failed_tests > 0 {
            println!("\nFailed tests:");
            for result in &self.results {
                if !result.success {
                    println!("  - {}: {}", result.test_name, 
                           result.error_message.as_deref().unwrap_or("Unknown error"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_analyzer() {
        let mut analyzer = PerformanceAnalyzer::new();
        
        // Test successful operation
        analyzer.run_test("test_success", || async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        }).await.unwrap();

        // Test failed operation
        analyzer.run_test("test_failure", || async {
            Err(anyhow::anyhow!("Test error"))
        }).await.unwrap();

        let report = analyzer.generate_report();
        assert_eq!(report.total_tests, 2);
        assert_eq!(report.successful_tests, 1);
        assert_eq!(report.failed_tests, 1);
    }

    #[test]
    fn test_performance_report_json() {
        let report = PerformanceReport {
            total_tests: 1,
            successful_tests: 1,
            failed_tests: 0,
            average_duration: Duration::from_millis(100),
            total_memory_usage: 1024,
            results: vec![PerformanceTestResult {
                test_name: "test".to_string(),
                duration: Duration::from_millis(100),
                memory_usage: 1024,
                success: true,
                error_message: None,
            }],
        };

        let json = report.to_json().unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("1024"));
    }
}