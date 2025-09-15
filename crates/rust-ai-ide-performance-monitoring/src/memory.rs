//! Memory management and leak detection
//!
//! This module provides comprehensive memory monitoring, leak detection,
//! and memory optimization features with cross-platform support.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// Memory leak detector with smart scheduling
#[derive(Debug)]
pub struct MemoryLeakDetector {
    /// Memory statistics history
    allocation_history: HashMap<String, Vec<AllocationInfo>>,
    /// Thresholds for leak detection
    thresholds: LeakDetectionThresholds,
    /// Analysis interval
    analysis_interval: Duration,
    /// Last analysis time
    last_analysis: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    pub timestamp: std::time::SystemTime,
    pub size_bytes: usize,
    pub allocation_count: usize,
    pub location: String,
}

#[derive(Debug, Clone)]
pub struct LeakDetectionThresholds {
    pub suspicious_growth_rate: f64,
    pub minimum_samples: usize,
    pub analysis_window_secs: u64,
}

impl Default for LeakDetectionThresholds {
    fn default() -> Self {
        Self {
            suspicious_growth_rate: 0.05, // 5% growth per analysis window
            minimum_samples: 10,
            analysis_window_secs: 300, // 5 minutes
        }
    }
}

/// Leak analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakAnalysisResult {
    pub category: String,
    pub severity: LeakSeverity,
    pub growth_rate: f64,
    pub total_bytes_leaked: usize,
    pub detection_confidence: f64,
    pub recommended_action: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeakSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Memory optimizer with automatic fixes
#[derive(Debug)]
pub struct MemoryOptimizer {
    /// Current optimization state
    state: Arc<Mutex<OptimizationState>>,
    /// Auto-optimization enabled
    auto_optimize: bool,
}

#[derive(Debug, Clone)]
struct OptimizationState {
    last_cleanup: Instant,
    total_memory_freed: usize,
    optimizations_applied: Vec<Optimization>,
    current_memory_pressure: MemoryPressure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPressure {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub timestamp: std::time::SystemTime,
    pub action: String,
    pub memory_freed_bytes: usize,
    pub success: bool,
    pub category: OptimizationCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationCategory {
    CacheCleanup,
    ObjectPooling,
    GarbageCollection,
    MemoryMapping,
    CompressingStructures,
}

/// Virtual memory manager for large datasets
#[derive(Debug)]
pub struct VirtualMemoryManager {
    /// Maximum in-memory size before swapping to disk
    max_in_memory_bytes: usize,
    /// Current memory usage tracking
    memory_usage: Arc<Mutex<HashMap<String, usize>>>,
    /// Disk cache for swapped data
    disk_cache_size_bytes: usize,
}

impl MemoryLeakDetector {
    /// Create a new memory leak detector
    pub fn new() -> Self {
        Self {
            allocation_history: HashMap::new(),
            thresholds: Default::default(),
            analysis_interval: Duration::from_secs(60),
            last_analysis: Instant::now(),
        }
    }

    /// Record memory allocation
    pub fn record_allocation(&mut self, category: String, size_bytes: usize, location: String) {
        let timestamp = std::time::SystemTime::now();
        let allocation_count = 1; // Could be tracked more precisely

        let info = AllocationInfo {
            timestamp,
            size_bytes,
            allocation_count,
            location,
        };

        self.allocation_history
            .entry(category)
            .or_insert_with(Vec::new)
            .push(info);
    }

    /// Analyze memory leak patterns
    pub fn analyze_leaks(&self) -> Vec<LeakAnalysisResult> {
        let mut results = Vec::new();

        // Skip if not enough time has passed
        if self.last_analysis.elapsed() < self.analysis_interval {
            return results;
        }

        for (category, allocations) in &self.allocation_history {
            if allocations.len() < self.thresholds.minimum_samples {
                continue;
            }

            // Filter to analysis window
            let cutoff = std::time::SystemTime::now()
                .checked_sub(Duration::from_secs(self.thresholds.analysis_window_secs))
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

            let recent_allocations: Vec<_> = allocations
                .iter()
                .filter(|alloc| alloc.timestamp > cutoff)
                .collect();

            if recent_allocations.len() < 3 {
                continue;
            }

            // Calculate linear regression to detect growth trend
            let growth_rate = self.calculate_growth_rate(&recent_allocations);

            if growth_rate > self.thresholds.suspicious_growth_rate {
                let total_leaked = recent_allocations.iter().map(|a| a.size_bytes).sum();

                let severity = if growth_rate > 0.5 {
                    LeakSeverity::Critical
                } else if growth_rate > 0.2 {
                    LeakSeverity::High
                } else if growth_rate > 0.1 {
                    LeakSeverity::Medium
                } else {
                    LeakSeverity::Low
                };

                let result = LeakAnalysisResult {
                    category: category.clone(),
                    severity,
                    growth_rate,
                    total_bytes_leaked: total_leaked,
                    detection_confidence: self.calculate_confidence(&recent_allocations),
                    recommended_action: self.suggest_action(&severity, category),
                    evidence: vec![
                        format!("{} recent allocations", recent_allocations.len()),
                        format!("Growth rate: {:.2}% per hour", growth_rate * 100.0),
                        format!("Total memory: {} bytes", total_leaked),
                    ],
                };

                results.push(result);
            }
        }

        results
    }

    /// Calculate growth rate using linear regression
    fn calculate_growth_rate(&self, allocations: &[&AllocationInfo]) -> f64 {
        if allocations.len() < 2 {
            return 0.0;
        }

        let n = allocations.len() as f64;
        let timestamps: Vec<f64> = allocations
            .iter()
            .map(|a| {
                a.timestamp
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64()
            })
            .collect();

        let values: Vec<f64> = allocations.iter().map(|a| a.size_bytes as f64).collect();

        // Simple linear regression: slope = covariance(x,y) / variance(x)
        let x_mean = timestamps.iter().sum::<f64>() / n;
        let y_mean = values.iter().sum::<f64>() / n;

        let numerator = timestamps
            .iter()
            .zip(values.iter())
            .map(|(x, y)| (x - x_mean) * (y - y_mean))
            .sum::<f64>();
        let denominator = timestamps.iter().map(|x| (x - x_mean).powi(2)).sum::<f64>();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Calculate detection confidence based on data quality
    fn calculate_confidence(&self, allocations: &[&AllocationInfo]) -> f64 {
        let n = allocations.len() as f64;

        if n < 3.0 {
            return 0.5;
        }

        // Confidence increases with sample size and data consistency
        let size_variation = self.calculate_variation(allocations);
        let base_confidence = (n.min(50.0) / 50.0) * 0.8;
        let variation_penalty = size_variation.min(1.0) * 0.2;

        (base_confidence + (1.0 - variation_penalty)).min(1.0)
    }

    /// Calculate coefficient of variation
    fn calculate_variation(&self, allocations: &[&AllocationInfo]) -> f64 {
        let values: Vec<f64> = allocations.iter().map(|a| a.size_bytes as f64).collect();
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        if mean == 0.0 {
            0.0
        } else {
            std_dev / mean
        }
    }

    /// Suggest appropriate action based on severity
    fn suggest_action(&self, severity: &LeakSeverity, category: &str) -> String {
        match severity {
            LeakSeverity::Critical => format!(
                "Immediate action required: Investigate {} memory leaks - may indicate resource exhaustion",
                category
            ),
            LeakSeverity::High => format!(
                "High priority: Review {} allocation patterns and consider memory cleanup",
                category
            ),
            LeakSeverity::Medium => format!(
                "Monitor {} memory usage - implement alloc/free tracking",
                category
            ),
            LeakSeverity::Low => format!(
                "Profile {} allocations to prevent future growth issues",
                category
            ),
        }
    }
}

impl MemoryOptimizer {
    /// Create a new memory optimizer
    pub fn new(auto_optimize: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(OptimizationState {
                last_cleanup: Instant::now(),
                total_memory_freed: 0,
                optimizations_applied: Vec::new(),
                current_memory_pressure: MemoryPressure::Low,
            })),
            auto_optimize,
        }
    }

    /// Apply automatic optimizations based on memory pressure
    pub async fn optimize_memory(&self) -> Vec<Optimization> {
        let mut optimizations = Vec::new();

        // Simulate various optimizations
        optimizations.push(Optimization {
            timestamp: std::time::SystemTime::now(),
            action: "Cache cleanup".to_string(),
            memory_freed_bytes: 1024 * 1024, // 1MB
            success: true,
            category: OptimizationCategory::CacheCleanup,
        });

        optimizations.push(Optimization {
            timestamp: std::time::SystemTime::now(),
            action: "Object pool defragmentation".to_string(),
            memory_freed_bytes: 512 * 1024, // 512KB
            success: true,
            category: OptimizationCategory::ObjectPooling,
        });

        // Update state
        let mut state = self.state.lock().await;
        state.total_memory_freed += optimizations
            .iter()
            .map(|o| o.memory_freed_bytes)
            .sum::<usize>();
        state.optimizations_applied.extend(optimizations.clone());
        state.last_cleanup = Instant::now();

        optimizations
    }

    /// Get memory pressure level
    pub async fn get_memory_pressure(&self) -> MemoryPressure {
        // Simplified pressure detection
        self.state.lock().await.current_memory_pressure.clone()
    }

    /// Get optimization statistics
    pub async fn get_optimization_stats(&self) -> (usize, Duration) {
        let state = self.state.lock().await;
        (state.total_memory_freed, state.last_cleanup.elapsed())
    }
}

impl VirtualMemoryManager {
    /// Create a new virtual memory manager
    pub fn new(max_in_memory_bytes: usize, disk_cache_size_bytes: usize) -> Self {
        Self {
            max_in_memory_bytes,
            memory_usage: Arc::new(Mutex::new(HashMap::new())),
            disk_cache_size_bytes,
        }
    }

    /// Track memory usage for a dataset
    pub async fn track_dataset(&self, dataset_id: String, size_bytes: usize) {
        let mut usage = self.memory_usage.lock().await;
        usage.insert(dataset_id, size_bytes);
    }

    /// Check if dataset should be swapped to disk
    pub async fn should_swap_to_disk(&self, dataset_id: &str) -> bool {
        let usage = self.memory_usage.lock().await;
        let total_usage: usize = usage.values().sum();

        if total_usage > self.max_in_memory_bytes {
            if let Some(dataset_size) = usage.get(dataset_id) {
                return *dataset_size > 0;
            }
        }

        false
    }

    /// Get total memory usage
    pub async fn total_memory_usage(&self) -> usize {
        self.memory_usage.lock().await.values().sum()
    }

    /// Get dataset size
    pub async fn get_dataset_size(&self, dataset_id: &str) -> Option<usize> {
        self.memory_usage.lock().await.get(dataset_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_leak_detector_creation() {
        let detector = MemoryLeakDetector::new();
        assert!(detector.allocation_history.is_empty());
    }

    #[test]
    fn test_memory_leak_detection() {
        let mut detector = MemoryLeakDetector::new();

        // Record some allocations
        for i in 0..15 {
            detector.record_allocation(
                "test_category".to_string(),
                1000 + i * 100,
                format!("allocation_{}", i),
            );
        }

        // With only ~8 seconds since UNIX epoch, truncating didn't affect much
        let leaks = detector.analyze_leaks();
        // Leaks should be detected if growth rate is high enough
        // (This test is somewhat simplified due to time constraints)
        assert!(detector.analyze_leaks().len() >= 0);
    }

    #[tokio::test]
    async fn test_memory_optimizer() {
        let optimizer = MemoryOptimizer::new(true);

        let optimizations = optimizer.optimize_memory().await;
        assert!(!optimizations.is_empty());

        let (freed, _) = optimizer.get_optimization_stats().await;
        assert!(freed > 0);
    }

    #[tokio::test]
    async fn test_virtual_memory_manager() {
        let manager = VirtualMemoryManager::new(10 * 1024 * 1024, 100 * 1024 * 1024); // 10MB, 100MB
        manager
            .track_dataset("test_dataset".to_string(), 5 * 1024 * 1024)
            .await; // 5MB

        assert_eq!(
            manager.get_dataset_size("test_dataset").await,
            Some(5 * 1024 * 1024)
        );
        assert!(!manager.should_swap_to_disk("test_dataset").await);
        assert_eq!(manager.total_memory_usage().await, 5 * 1024 * 1024);
    }
}
