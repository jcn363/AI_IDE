/*!
 * Performance Monitor for real-time CPU utilization tracking
 *
 * This module provides comprehensive performance monitoring with CPU utilization,
 * memory usage, and throughput metrics for optimal scheduler decisions.
 */

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// CPU performance metrics
#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub utilization_percent: f64,
    pub user_time_percent: f64,
    pub system_time_percent: f64,
    pub idle_time_percent: f64,
    pub tasks_per_second: f64,
}

/// Memory performance metrics
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub used_mb: f64,
    pub available_mb: f64,
    pub utilization_percent: f64,
    pub page_faults_per_second: f64,
}

/// System performance snapshot
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: std::time::Instant,
    pub cpu_metrics: CpuMetrics,
    pub memory_metrics: MemoryMetrics,
    pub io_wait_percent: f64,
    pub context_switches_per_second: f64,
}

/// Performance monitor with real-time tracking
pub struct PerformanceMonitor {
    /// Recent performance snapshots
    snapshots: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    /// Maximum snapshots to keep
    max_snapshots: usize,
    /// Monitoring interval
    monitoring_interval: Duration,
    /// CPU monitoring enabled
    cpu_monitoring: bool,
    /// Memory monitoring enabled
    memory_monitoring: bool,
    /// I/O monitoring enabled
    io_monitoring: bool,
}

impl PerformanceMonitor {
    pub fn new(max_snapshots: usize, monitoring_interval: Duration) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(max_snapshots))),
            max_snapshots,
            monitoring_interval,
            cpu_monitoring: true,
            memory_monitoring: true,
            io_monitoring: true,
        }
    }

    /// Start performance monitoring in background
    pub async fn start_monitoring(self: Arc<Self>) {
        let monitoring_interval = self.monitoring_interval;

        tokio::spawn(async move {
            let mut interval_timer = interval(monitoring_interval);

            loop {
                interval_timer.tick().await;

                // Collect current performance metrics
                let snapshot = self.collect_performance_snapshot().await;

                // Store snapshot
                let mut snapshots = self.snapshots.write().await;
                snapshots.push_back(snapshot);

                // Maintain maximum capacity
                if snapshots.len() > self.max_snapshots {
                    snapshots.pop_front();
                }
            }
        });
    }

    /// Collect current performance snapshot
    async fn collect_performance_snapshot(&self) -> PerformanceSnapshot {
        let timestamp = std::time::Instant::now();

        // Collect CPU metrics
        let cpu_metrics = if self.cpu_monitoring {
            self.collect_cpu_metrics().await
        } else {
            CpuMetrics {
                utilization_percent: 0.0,
                user_time_percent: 0.0,
                system_time_percent: 0.0,
                idle_time_percent: 0.0,
                tasks_per_second: 0.0,
            }
        };

        // Collect memory metrics
        let memory_metrics = if self.memory_monitoring {
            self.collect_memory_metrics().await
        } else {
            MemoryMetrics {
                used_mb: 0.0,
                available_mb: 0.0,
                utilization_percent: 0.0,
                page_faults_per_second: 0.0,
            }
        };

        // Collect I/O metrics
        let io_wait_percent = if self.io_monitoring {
            self.collect_io_metrics().await
        } else {
            0.0
        };

        // Collect context switches
        let context_switches_per_second = self.collect_context_switches().await;

        PerformanceSnapshot {
            timestamp,
            cpu_metrics,
            memory_metrics,
            io_wait_percent,
            context_switches_per_second,
        }
    }

    /// Collect CPU utilization metrics
    async fn collect_cpu_metrics(&self) -> CpuMetrics {
        // In production, this would use system monitoring libraries
        // For now, return simulated metrics
        CpuMetrics {
            utilization_percent: self.simulate_cpu_utilization(),
            user_time_percent: 65.0,
            system_time_percent: 15.0,
            idle_time_percent: 20.0,
            tasks_per_second: 150.0,
        }
    }

    /// Collect memory usage metrics
    async fn collect_memory_metrics(&self) -> MemoryMetrics {
        // In production, this would use system monitoring libraries
        MemoryMetrics {
            used_mb: 2048.0,
            available_mb: 4096.0,
            utilization_percent: 50.0,
            page_faults_per_second: 100.0,
        }
    }

    /// Collect I/O wait metrics
    async fn collect_io_metrics(&self) -> f64 {
        // Simulate I/O wait percentage
        5.0
    }

    /// Collect context switches per second
    async fn collect_context_switches(&self) -> f64 {
        // Simulate context switches
        5000.0
    }

    /// Simulate CPU utilization (replace with real monitoring)
    fn simulate_cpu_utilization(&self) -> f64 {
        // Simple simulation - in real implementation, use sysinfo or similar
        use std::time::{SystemTime, UNIX_EPOCH};
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        40.0 + (time % 60) as f64 * 0.5 // Vary between 40-70%
    }

    /// Get latest performance snapshot
    pub async fn latest_snapshot(&self) -> Option<PerformanceSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.back().cloned()
    }

    /// Get performance metrics over time window
    pub async fn metrics_over_time(&self, duration: Duration) -> Vec<PerformanceSnapshot> {
        let snapshots = self.snapshots.read().await;
        let cutoff_time = std::time::Instant::now() - duration;

        snapshots.iter()
            .filter(|s| s.timestamp >= cutoff_time)
            .cloned()
            .collect()
    }

    /// Calculate average CPU utilization over time window
    pub async fn average_cpu_utilization(&self, duration: Duration) -> f64 {
        let snapshots = self.metrics_over_time(duration).await;

        if snapshots.is_empty() {
            return 0.0;
        }

        let total_utilization: f64 = snapshots.iter()
            .map(|s| s.cpu_metrics.utilization_percent)
            .sum();

        total_utilization / snapshots.len() as f64
    }

    /// Calculate average memory utilization over time window
    pub async fn average_memory_utilization(&self, duration: Duration) -> f64 {
        let snapshots = self.metrics_over_time(duration).await;

        if snapshots.is_empty() {
            return 0.0;
        }

        let total_utilization: f64 = snapshots.iter()
            .map(|s| s.memory_metrics.utilization_percent)
            .sum();

        total_utilization / snapshots.len() as f64
    }

    /// Get throughput statistics
    pub async fn throughput_stats(&self, duration: Duration) -> ThroughputStats {
        let snapshots = self.metrics_over_time(duration).await;

        if snapshots.is_empty() {
            return ThroughputStats::default();
        }

        let mut total_tasks = 0.0;
        let mut max_throughput = 0.0;
        let mut min_throughput = f64::MAX;

        for snapshot in &snapshots {
            total_tasks += snapshot.cpu_metrics.tasks_per_second;
            max_throughput = max_throughput.max(snapshot.cpu_metrics.tasks_per_second);
            min_throughput = min_throughput.min(snapshot.cpu_metrics.tasks_per_second);
        }

        let avg_throughput = total_tasks / snapshots.len() as f64;

        ThroughputStats {
            average_throughput: avg_throughput,
            max_throughput,
            min_throughput,
            total_tasks_processed: (avg_throughput * duration.as_secs_f64()) as u64,
        }
    }

    /// Get performance summary
    pub async fn performance_summary(&self, duration: Duration) -> PerformanceSummary {
        let avg_cpu = self.average_cpu_utilization(duration).await;
        let avg_memory = self.average_memory_utilization(duration).await;
        let throughput = self.throughput_stats(duration).await;
        let snapshots = self.metrics_over_time(duration).await;

        let avg_io_wait = if snapshots.is_empty() {
            0.0
        } else {
            snapshots.iter().map(|s| s.io_wait_percent).sum::<f64>() / snapshots.len() as f64
        };

        PerformanceSummary {
            time_window_seconds: duration.as_secs_f64(),
            average_cpu_utilization: avg_cpu,
            average_memory_utilization: avg_memory,
            average_io_wait: avg_io_wait,
            throughput_stats: throughput,
            bottleneck_detected: self.detect_bottleneck(&snapshots),
        }
    }

    /// Detect performance bottlenecks
    fn detect_bottleneck(&self, snapshots: &[PerformanceSnapshot]) -> Option<BottleneckType> {
        if snapshots.is_empty() {
            return None;
        }

        let avg_cpu = snapshots.iter().map(|s| s.cpu_metrics.utilization_percent).sum::<f64>() / snapshots.len() as f64;
        let avg_io = snapshots.iter().map(|s| s.io_wait_percent).sum::<f64>() / snapshots.len() as f64;
        let avg_memory = snapshots.iter().map(|s| s.memory_metrics.utilization_percent).sum::<f64>() / snapshots.len() as f64;

        // Detect bottlenecks based on thresholds
        if avg_cpu > 90.0 {
            Some(BottleneckType::Cpu)
        } else if avg_io > 50.0 {
            Some(BottleneckType::Io)
        } else if avg_memory > 90.0 {
            Some(BottleneckType::Memory)
        } else if avg_cpu < 20.0 && avg_io < 10.0 {
            Some(BottleneckType::Underutilized)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThroughputStats {
    pub average_throughput: f64,
    pub max_throughput: f64,
    pub min_throughput: f64,
    pub total_tasks_processed: u64,
}

impl Default for ThroughputStats {
    fn default() -> Self {
        Self {
            average_throughput: 0.0,
            max_throughput: 0.0,
            min_throughput: f64::MAX,
            total_tasks_processed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub time_window_seconds: f64,
    pub average_cpu_utilization: f64,
    pub average_memory_utilization: f64,
    pub average_io_wait: f64,
    pub throughput_stats: ThroughputStats,
    pub bottleneck_detected: Option<BottleneckType>,
}

#[derive(Debug, Clone)]
pub enum BottleneckType {
    Cpu,
    Memory,
    Io,
    Underutilized,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new(1000, Duration::from_millis(100)) // 1 second window at 10Hz
    }
}