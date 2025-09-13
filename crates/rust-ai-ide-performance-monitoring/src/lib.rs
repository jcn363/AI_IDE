//! Unified Performance Monitoring System for Rust AI IDE
//!
//! This crate provides comprehensive performance monitoring with real-time metrics,
//! cross-platform support, and integration with the EventBus for distributed events.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};
use tokio::sync::{Mutex, RwLock};
use tokio::time;

pub mod battery;
pub mod cross_platform;
pub mod memory;
pub mod processing;

pub use battery::*;
pub use cross_platform::*;
pub use memory::*;
pub use processing::*;

/// Performance metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent:    f64,
    pub memory_used_mb:       f64,
    pub memory_total_mb:      f64,
    pub memory_usage_percent: f64,
    pub disk_read_mb:         f64,
    pub disk_write_mb:        f64,
    pub network_rx_mb:        f64,
    pub network_tx_mb:        f64,
    pub process_count:        usize,
    pub temperature_celsius:  Option<f64>,
    pub uptime_seconds:       u64,
    pub timestamp:            DateTime<Utc>,
}

/// Process-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid:                u32,
    pub name:               String,
    pub cpu_usage_percent:  f64,
    pub memory_used_mb:     f64,
    pub disk_read_mb:       f64,
    pub disk_write_mb:      f64,
    pub virtual_memory_mb:  f64,
    pub resident_memory_mb: f64,
    pub thread_count:       usize,
    pub uptime_seconds:     u64,
}

/// Performance alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub high_cpu_threshold:        f64,
    pub high_memory_threshold:     f64,
    pub low_disk_space_threshold:  f64,
    pub network_anomaly_threshold: f64,
}

/// Unified Performance Monitor
#[derive(Debug)]
pub struct PerformanceMonitor {
    system:              Arc<RwLock<System>>,
    thresholds:          PerformanceThresholds,
    history:             Arc<RwLock<Vec<(DateTime<Utc>, SystemMetrics)>>>,
    process_metrics:     Arc<RwLock<HashMap<u32, ProcessMetrics>>>,
    last_collection:     Arc<RwLock<Instant>>,
    collection_interval: Duration,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default thresholds
    pub fn new() -> Self {
        let thresholds = PerformanceThresholds {
            high_cpu_threshold:        80.0,
            high_memory_threshold:     85.0,
            low_disk_space_threshold:  10.0,
            network_anomaly_threshold: 1000.0, // 1GB/s spike threshold
        };

        Self {
            system: Arc::new(RwLock::new(System::new())),
            thresholds,
            history: Arc::new(RwLock::new(Vec::new())),
            process_metrics: Arc::new(RwLock::new(HashMap::new())),
            last_collection: Arc::new(RwLock::new(Instant::now())),
            collection_interval: Duration::from_secs(5),
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(thresholds: PerformanceThresholds) -> Self {
        let mut monitor = Self::new();
        monitor.thresholds = thresholds;
        monitor
    }

    /// Start continuous monitoring in background
    pub async fn start_monitoring(&self) -> Result<(), String> {
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.monitoring_loop().await;
        });
        Ok(())
    }

    /// Collect current system metrics
    pub async fn collect_metrics(&self) -> Result<SystemMetrics, String> {
        let mut system = self.system.write().await;
        system.refresh_all();

        let temperature = self.get_system_temperature(&system)?;
        let uptime = system.uptime();

        // CPU metrics
        let cpu = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f64>() / system.cpus().len() as f64;
        let cpus = system.cpus();

        // Memory metrics
        let memory_used_mb = system.used_memory() as f64 / 1024.0 / 1024.0;
        let memory_total_mb = system.total_memory() as f64 / 1024.0 / 1024.0;
        let memory_usage_percent = (memory_used_mb / memory_total_mb) * 100.0;

        // Disk metrics (placeholder for enhancement)
        let disk_read_mb = 0.0;
        let disk_write_mb = 0.0;

        // Network metrics (placeholder for enhancement)
        let network_rx_mb = 0.0;
        let network_tx_mb = 0.0;

        Ok(SystemMetrics {
            cpu_usage_percent: cpus
                .iter()
                .next()
                .unwrap_or(&sysinfo::Cpu::new())
                .cpu_usage()
                .into(),
            memory_used_mb,
            memory_total_mb,
            memory_usage_percent,
            disk_read_mb,
            disk_write_mb,
            network_rx_mb,
            network_tx_mb,
            process_count: system.processes().len(),
            temperature_celsius: temperature,
            uptime_seconds: uptime,
            timestamp: Utc::now(),
        })
    }

    /// Get process-specific metrics
    pub async fn collect_process_metrics(&self, pid: u32) -> Result<Option<ProcessMetrics>, String> {
        let system = self.system.read().await;

        if let Some(process) = system.process(pid) {
            let metrics = ProcessMetrics {
                pid:                process.pid().as_u32(),
                name:               process.name().to_string(),
                cpu_usage_percent:  process.cpu_usage(),
                memory_used_mb:     process.memory() as f64 / 1024.0 / 1024.0,
                disk_read_mb:       process.disk_usage().read_bytes as f64 / 1024.0 / 1024.0,
                disk_write_mb:      process.disk_usage().written_bytes as f64 / 1024.0 / 1024.0,
                virtual_memory_mb:  process.virtual_memory() as f64 / 1024.0 / 1024.0,
                resident_memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                thread_count:       process.thread_count(),
                uptime_seconds:     process.run_time(),
            };
            Ok(Some(metrics))
        } else {
            Ok(None)
        }
    }

    /// Check if system is under heavy load
    pub async fn is_heavy_load(&self) -> Result<bool, String> {
        let metrics = self.collect_metrics().await?;
        Ok(
            metrics.cpu_usage_percent > self.thresholds.high_cpu_threshold
                || metrics.memory_usage_percent > self.thresholds.high_memory_threshold,
        )
    }

    /// Get performance history
    pub async fn get_history(&self, duration: Duration) -> Vec<(DateTime<Utc>, SystemMetrics)> {
        let history = self.history.read().await;
        let cutoff = Utc::now() - duration;

        history
            .iter()
            .filter(|(timestamp, _)| timestamp >= &cutoff)
            .cloned()
            .collect()
    }

    /// Main monitoring loop
    async fn monitoring_loop(&self) {
        let mut interval = time::interval(self.collection_interval);

        loop {
            interval.tick().await;

            match self.collect_metrics().await {
                Ok(metrics) => {
                    // Store in history (keep last 1000 entries)
                    let mut history = self.history.write().await;
                    history.push((metrics.timestamp, metrics.clone()));

                    if history.len() > 1000 {
                        history.remove(0);
                    }
                    drop(history);

                    // Check thresholds and emit alerts if configured
                    self.check_thresholds(&metrics).await.ok();

                    // Collect process metrics
                    self.collect_all_process_metrics().await.ok();
                }
                Err(e) => {
                    eprintln!("Failed to collect performance metrics: {}", e);
                }
            }
        }
    }

    /// Check thresholds and emit alerts
    async fn check_thresholds(&self, metrics: &SystemMetrics) -> Result<(), String> {
        if metrics.cpu_usage_percent > self.thresholds.high_cpu_threshold {
            println!("ALERT: High CPU usage: {:.1}%", metrics.cpu_usage_percent);
        }

        if metrics.memory_usage_percent > self.thresholds.high_memory_threshold {
            println!(
                "ALERT: High memory usage: {:.1}%",
                metrics.memory_usage_percent
            );
        }

        Ok(())
    }

    /// Collect metrics for all processes
    async fn collect_all_process_metrics(&self) -> Result<(), String> {
        let system = self.system.read().await;
        let mut process_metrics = self.process_metrics.write().await;

        for (pid, process) in system.processes() {
            let pid_u32 = pid.as_u32();
            let metrics = ProcessMetrics {
                pid:                pid_u32,
                name:               process.name().to_string(),
                cpu_usage_percent:  process.cpu_usage(),
                memory_used_mb:     process.memory() as f64 / 1024.0 / 1024.0,
                disk_read_mb:       process.disk_usage().read_bytes as f64 / 1024.0 / 1024.0,
                disk_write_mb:      process.disk_usage().written_bytes as f64 / 1024.0 / 1024.0,
                virtual_memory_mb:  process.virtual_memory() as f64 / 1024.0 / 1024.0,
                resident_memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                thread_count:       process.thread_count(),
                uptime_seconds:     process.run_time(),
            };

            process_metrics.insert(pid_u32, metrics);
        }

        // Clean up old metrics for processes that no longer exist
        let existing_pids: std::collections::HashSet<u32> = system.processes().keys().map(|p| p.as_u32()).collect();
        process_metrics.retain(|pid, _| existing_pids.contains(pid));

        Ok(())
    }

    /// Get system temperature (platform-specific)
    fn get_system_temperature(&self, system: &System) -> Result<Option<f64>, String> {
        // Platform-specific temperature reading
        // This is a simplified implementation - real implementation would use platform APIs
        Ok(None) // Placeholder for now
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            system:              Arc::clone(&self.system),
            thresholds:          self.thresholds.clone(),
            history:             Arc::clone(&self.history),
            process_metrics:     Arc::clone(&self.process_metrics),
            last_collection:     Arc::clone(&self.last_collection),
            collection_interval: self.collection_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.collection_interval, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_collect_metrics() {
        let monitor = PerformanceMonitor::new();
        let result = monitor.collect_metrics().await;
        assert!(
            result.is_ok(),
            "Failed to collect metrics: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_thresholds() {
        let thresholds = PerformanceThresholds {
            high_cpu_threshold:        10.0,
            high_memory_threshold:     10.0,
            low_disk_space_threshold:  50.0,
            network_anomaly_threshold: 100.0,
        };

        let monitor = PerformanceMonitor::with_thresholds(thresholds);
        assert_eq!(monitor.thresholds.high_cpu_threshold, 10.0);
    }
}
