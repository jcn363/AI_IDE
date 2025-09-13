//! Performance monitoring for lazy loading and memory management

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Global performance monitor instance
static PERFORMANCE_MONITOR: once_cell::sync::OnceCell<Arc<PerformanceMonitor>> = once_cell::sync::OnceCell::new();

/// Performance monitor for tracking lazy loading and memory usage
pub struct PerformanceMonitor {
    startup_time: Instant,
    component_load_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    memory_usage_history: Arc<RwLock<Vec<MemoryUsagePoint>>>,
    pool_stats_history: Arc<RwLock<Vec<PoolStatsPoint>>>,
    enabled: bool,
}

impl PerformanceMonitor {
    /// Initialize the global performance monitor
    pub async fn init() -> crate::LazyResult<()> {
        let monitor = Arc::new(Self {
            startup_time: Instant::now(),
            component_load_times: Arc::new(RwLock::new(HashMap::new())),
            memory_usage_history: Arc::new(RwLock::new(Vec::new())),
            pool_stats_history: Arc::new(RwLock::new(Vec::new())),
            enabled: true,
        });

        PERFORMANCE_MONITOR
            .set(monitor)
            .map_err(|_| crate::LazyLoadingError::internal("Performance monitor already initialized"))?;

        Ok(())
    }

    /// Get the global performance monitor instance
    pub fn global() -> Option<&'static Arc<Self>> {
        PERFORMANCE_MONITOR.get()
    }

    /// Record component load time
    pub async fn record_component_load(&self, component_name: &str, duration: Duration) {
        if !self.enabled {
            return;
        }

        let mut load_times = self.component_load_times.write().await;
        load_times
            .entry(component_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }

    /// Record memory usage point
    pub async fn record_memory_usage(&self, usage_bytes: usize, pool_name: Option<&str>) {
        if !self.enabled {
            return;
        }

        let point = MemoryUsagePoint {
            timestamp: Instant::now().duration_since(self.startup_time),
            usage_bytes,
            pool_name: pool_name.map(|s| s.to_string()),
        };

        let mut history = self.memory_usage_history.write().await;
        history.push(point);
    }

    /// Record pool statistics
    pub async fn record_pool_stats(&self, stats: crate::memory_pool::PoolStats) {
        if !self.enabled {
            return;
        }

        let point = PoolStatsPoint {
            timestamp: Instant::now().duration_since(self.startup_time),
            stats,
        };

        let mut history = self.pool_stats_history.write().await;
        history.push(point);
    }

    /// Get component load statistics
    pub async fn get_component_load_stats(&self, component_name: &str) -> Option<ComponentLoadStats> {
        let load_times = self.component_load_times.read().await;
        let times = load_times.get(component_name)?;

        if times.is_empty() {
            return None;
        }

        let total_loads = times.len();
        let total_duration: Duration = times.iter().sum();
        let avg_duration = total_duration / total_loads as u32;
        let min_duration = *times.iter().min().unwrap();
        let max_duration = *times.iter().max().unwrap();

        Some(ComponentLoadStats {
            component_name: component_name.to_string(),
            total_loads,
            avg_load_time: avg_duration,
            min_load_time: min_duration,
            max_load_time: max_duration,
            total_load_time: total_duration,
        })
    }

    /// Get memory usage statistics
    pub async fn get_memory_usage_stats(&self) -> MemoryUsageStats {
        let history = self.memory_usage_history.read().await;

        if history.is_empty() {
            return MemoryUsageStats::default();
        }

        let usage_values: Vec<usize> = history.iter().map(|p| p.usage_bytes).collect();
        let total_points = usage_values.len();
        let avg_usage = usage_values.iter().sum::<usize>() / total_points;
        let peak_usage = *usage_values.iter().max().unwrap();
        let current_usage = *usage_values.last().unwrap();

        MemoryUsageStats {
            total_measurements: total_points,
            average_usage_bytes: avg_usage,
            peak_usage_bytes: peak_usage,
            current_usage_bytes: current_usage,
        }
    }

    /// Get pool performance statistics
    pub async fn get_pool_performance_stats(&self) -> Vec<PoolPerformanceStats> {
        let history = self.pool_stats_history.read().await;

        let mut stats_by_pool = HashMap::new();

        for point in history.iter() {
            let pool_name = "combined_pools".to_string(); // For now, aggregate all pools
            let entry = stats_by_pool.entry(pool_name).or_insert_with(Vec::new);
            entry.push(point.clone());
        }

        stats_by_pool
            .into_iter()
            .map(|(pool_name, points)| {
                let analysis_sizes: Vec<usize> = points.iter().map(|p| p.stats.analysis_pool_size).collect();
                let model_sizes: Vec<usize> = points.iter().map(|p| p.stats.model_pool_size).collect();

                PoolPerformanceStats {
                    pool_name,
                    total_measurements: points.len(),
                    avg_analysis_pool_size: analysis_sizes.iter().sum::<usize>() / analysis_sizes.len(),
                    avg_model_pool_size: model_sizes.iter().sum::<usize>() / model_sizes.len(),
                    max_analysis_pool_size: *analysis_sizes.iter().max().unwrap_or(&0),
                    max_model_pool_size: *model_sizes.iter().max().unwrap_or(&0),
                }
            })
            .collect()
    }

    /// Get startup time performance metrics
    pub async fn get_startup_performance(&self) -> StartupPerformance {
        let mut component_stats = Vec::new();
        let load_times = self.component_load_times.read().await;

        for (component_name, times) in load_times.iter() {
            if let Some(stats) = self.get_component_load_stats(component_name).await {
                component_stats.push(stats);
            }
        }

        let total_load_time: Duration = component_stats.iter().map(|s| s.total_load_time).sum();
        let critical_path_time = component_stats
            .iter()
            .map(|s| s.max_load_time)
            .max()
            .unwrap_or(Duration::from_secs(0));

        StartupPerformance {
            total_startup_time: Instant::now().duration_since(self.startup_time),
            total_component_load_time: total_load_time,
            critical_path_load_time: critical_path_time,
            components_loaded: component_stats.len(),
            component_load_stats: component_stats,
        }
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            startup_performance: self.get_startup_performance().await,
            memory_usage_stats: self.get_memory_usage_stats().await,
            pool_performance_stats: self.get_pool_performance_stats().await,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Enable or disable performance monitoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if performance monitoring is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clear all performance data
    pub async fn clear_data(&self) {
        let mut load_times = self.component_load_times.write().await;
        load_times.clear();

        let mut memory_history = self.memory_usage_history.write().await;
        memory_history.clear();

        let mut pool_history = self.pool_stats_history.write().await;
        pool_history.clear();
    }
}

/// Memory usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsagePoint {
    pub timestamp: Duration,
    pub usage_bytes: usize,
    pub pool_name: Option<String>,
}

/// Pool statistics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatsPoint {
    pub timestamp: Duration,
    pub stats: crate::memory_pool::PoolStats,
}

/// Component load statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentLoadStats {
    pub component_name: String,
    pub total_loads: usize,
    pub avg_load_time: Duration,
    pub min_load_time: Duration,
    pub max_load_time: Duration,
    pub total_load_time: Duration,
}

/// Memory usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    pub total_measurements: usize,
    pub average_usage_bytes: usize,
    pub peak_usage_bytes: usize,
    pub current_usage_bytes: usize,
}

/// Pool performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolPerformanceStats {
    pub pool_name: String,
    pub total_measurements: usize,
    pub avg_analysis_pool_size: usize,
    pub avg_model_pool_size: usize,
    pub max_analysis_pool_size: usize,
    pub max_model_pool_size: usize,
}

/// Startup performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupPerformance {
    pub total_startup_time: Duration,
    pub total_component_load_time: Duration,
    pub critical_path_load_time: Duration,
    pub components_loaded: usize,
    pub component_load_stats: Vec<ComponentLoadStats>,
}

/// Complete performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub startup_performance: StartupPerformance,
    pub memory_usage_stats: MemoryUsageStats,
    pub pool_performance_stats: Vec<PoolPerformanceStats>,
    pub timestamp: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_performance_monitor_initialization() {
        PerformanceMonitor::init().await.unwrap();
        let monitor = PerformanceMonitor::global().unwrap();

        assert!(monitor.is_enabled());
    }

    #[tokio::test]
    async fn test_component_load_recording() {
        let monitor = PerformanceMonitor {
            startup_time: Instant::now(),
            component_load_times: Arc::new(RwLock::new(HashMap::new())),
            memory_usage_history: Arc::new(RwLock::new(Vec::new())),
            pool_stats_history: Arc::new(RwLock::new(Vec::new())),
            enabled: true,
        };

        let duration = Duration::from_millis(100);
        monitor.record_component_load("test_component", duration).await;

        let stats = monitor.get_component_load_stats("test_component").await.unwrap();
        assert_eq!(stats.component_name, "test_component");
        assert_eq!(stats.total_loads, 1);
        assert_eq!(stats.avg_load_time, duration);
    }

    #[tokio::test]
    async fn test_memory_usage_recording() {
        let monitor = PerformanceMonitor {
            startup_time: Instant::now(),
            component_load_times: Arc::new(RwLock::new(HashMap::new())),
            memory_usage_history: Arc::new(RwLock::new(Vec::new())),
            pool_stats_history: Arc::new(RwLock::new(Vec::new())),
            enabled: true,
        };

        monitor.record_memory_usage(1024, Some("test_pool")).await;
        monitor.record_memory_usage(2048, Some("test_pool")).await;

        let stats = monitor.get_memory_usage_stats().await;
        assert_eq!(stats.total_measurements, 2);
        assert_eq!(stats.average_usage_bytes, 1536); // (1024 + 2048) / 2
        assert_eq!(stats.peak_usage_bytes, 2048);
        assert_eq!(stats.current_usage_bytes, 2048);
    }

    #[tokio::test]
    async fn test_disabled_monitoring() {
        let mut monitor = PerformanceMonitor {
            startup_time: Instant::now(),
            component_load_times: Arc::new(RwLock::new(HashMap::new())),
            memory_usage_history: Arc::new(RwLock::new(Vec::new())),
            pool_stats_history: Arc::new(RwLock::new(Vec::new())),
            enabled: true,
        };

        monitor.set_enabled(false);
        assert!(!monitor.is_enabled());

        let duration = Duration::from_millis(100);
        monitor.record_component_load("test_component", duration).await;

        let stats = monitor.get_component_load_stats("test_component").await;
        assert!(stats.is_none());
    }
}