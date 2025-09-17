// Performance monitoring module with real-time system monitoring

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, DiskExt, NetworkExt, PidExt, ProcessExt, System, SystemExt};
use tokio::sync::mpsc;
use tokio::time;

use rust_ai_ide_shared_types::{PerformanceMetrics, MetricValue, RateType};

/// Real-time system monitor that collects and streams performance metrics
pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    metrics_buffer: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    max_buffer_size: usize,
    collection_interval: Duration,
    is_monitoring: Arc<RwLock<bool>>,
    metrics_sender: Option<mpsc::UnboundedSender<PerformanceMetrics>>,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: Arc::new(RwLock::new(system)),
            metrics_buffer: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_size: 1000,
            collection_interval: Duration::from_secs(5),
            is_monitoring: Arc::new(RwLock::new(false)),
            metrics_sender: None,
        }
    }

    /// Create a new system monitor with custom buffer size
    pub fn with_buffer_size(max_buffer_size: usize) -> Self {
        let mut monitor = Self::new();
        monitor.max_buffer_size = max_buffer_size;
        monitor
    }

    /// Start real-time monitoring with streaming
    pub async fn start_monitoring(&self) -> Result<(), String> {
        let mut is_monitoring = self.is_monitoring.write().unwrap();
        if *is_monitoring {
            return Err("Monitor is already running".to_string());
        }
        *is_monitoring = true;
        drop(is_monitoring);

        let system = Arc::clone(&self.system);
        let metrics_buffer = Arc::clone(&self.metrics_buffer);
        let max_buffer_size = self.max_buffer_size;
        let collection_interval = self.collection_interval;
        let is_monitoring_flag = Arc::clone(&self.is_monitoring);
        let sender = self.metrics_sender.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(collection_interval);

            loop {
                interval.tick().await;

                // Check if monitoring should stop
                {
                    let monitoring = is_monitoring_flag.read().unwrap();
                    if !*monitoring {
                        break;
                    }
                }

                let metrics = Self::collect_current_metrics(&system).await;

                // Add to buffer
                {
                    let mut buffer = metrics_buffer.write().unwrap();
                    buffer.push_back(metrics.clone());
                    if buffer.len() > max_buffer_size {
                        buffer.pop_front();
                    }
                }

                // Send to streaming channel if available
                if let Some(sender) = &sender {
                    let _ = sender.send(metrics);
                }
            }
        });

        Ok(())
    }

    /// Stop monitoring
    pub fn stop_monitoring(&self) {
        let mut is_monitoring = self.is_monitoring.write().unwrap();
        *is_monitoring = false;
    }

    /// Collect current system metrics
    async fn collect_current_metrics(system: &Arc<RwLock<System>>) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::new();
        let collection_start = Instant::now();

        // Refresh system information
        {
            let mut sys = system.write().unwrap();
            sys.refresh_all();
        }

        let sys = system.read().unwrap();

        // CPU metrics
        let global_cpu = sys.global_cpu_info();
        metrics.set_rate(RateType::CpuUsage, global_cpu.cpu_usage() as f64);

        // Memory metrics
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let available_memory = sys.available_memory();

        metrics.resources.memory_bytes = Some(used_memory);
        metrics.resources.peak_memory_bytes = Some(total_memory);
        metrics.rates.memory_usage_percent = Some(
            if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            }
        );

        // Disk metrics
        let mut total_disk_space = 0u64;
        let mut used_disk_space = 0u64;

        for disk in sys.disks() {
            total_disk_space += disk.total_space();
            used_disk_space += disk.available_space();
        }

        let used_disk = total_disk_space.saturating_sub(used_disk_space);
        let disk_usage_percent = if total_disk_space > 0 {
            (used_disk as f64 / total_disk_space as f64) * 100.0
        } else {
            0.0
        };

        metrics.disk_io_mb_per_sec = Some(disk_usage_percent);
        metrics.add_extension(
            "disk_total_bytes".to_string(),
            MetricValue::Integer(total_disk_space as i64),
        );
        metrics.add_extension(
            "disk_used_bytes".to_string(),
            MetricValue::Integer(used_disk as i64),
        );

        // Network metrics
        let mut total_received = 0u64;
        let mut total_transmitted = 0u64;

        for (_interface_name, data) in sys.networks() {
            total_received += data.total_received();
            total_transmitted += data.total_transmitted();
        }

        let total_network_io = (total_received + total_transmitted) / 1_000_000; // Convert to MB
        metrics.network_io_mb_per_sec = Some(total_network_io as f64);

        // Process monitoring for IDE and LSP servers
        let ide_processes = Self::monitor_ide_processes(&sys);
        for (name, proc_metrics) in ide_processes {
            metrics.add_extension(
                format!("process_{}_cpu", name.replace("-", "_")),
                MetricValue::Float(proc_metrics.cpu_usage),
            );
            metrics.add_extension(
                format!("process_{}_memory", name.replace("-", "_")),
                MetricValue::Integer(proc_metrics.memory_bytes as i64),
            );
        }

        // Response time measurement
        metrics.timing.response_time_ns = Some(collection_start.elapsed().as_nanos() as u64);

        metrics
    }

    /// Monitor IDE and LSP server processes
    fn monitor_ide_processes(system: &System) -> std::collections::HashMap<String, ProcessInfo> {
        let mut processes = std::collections::HashMap::new();
        let patterns = [
            "rust-ai-ide",
            "tauri",
            "cargo",
            "rustc",
            "rust-analyzer",
            "typescript-language-server",
            "vscode",
            "code",
        ];

        for (pid, process) in system.processes() {
            let process_name = process.name().to_lowercase();

            for pattern in &patterns {
                if process_name.contains(pattern) {
                    let key = if pattern == &"vscode" || pattern == &"code" {
                        "vscode".to_string()
                    } else {
                        pattern.to_string()
                    };

                    processes.insert(
                        key,
                        ProcessInfo {
                            pid: pid.as_u32(),
                            cpu_usage: process.cpu_usage() as f64,
                            memory_bytes: process.memory(),
                            status: format!("{:?}", process.status()),
                        },
                    );
                    break;
                }
            }
        }

        processes
    }

    /// Get current metrics (non-async version for trait compatibility)
    pub fn get_current_metrics(&self) -> PerformanceMetrics {
        // Refresh system for current metrics
        {
            let mut sys = self.system.write().unwrap();
            sys.refresh_all();
        }

        // Use a runtime to call the async function
        tokio::runtime::Handle::current().block_on(async {
            Self::collect_current_metrics(&self.system).await
        })
    }

    /// Get metrics history
    pub fn get_metrics_history(&self) -> Vec<PerformanceMetrics> {
        let buffer = self.metrics_buffer.read().unwrap();
        buffer.iter().cloned().collect()
    }

    /// Get latest metrics from buffer
    pub fn get_latest_metrics(&self) -> Option<PerformanceMetrics> {
        let buffer = self.metrics_buffer.read().unwrap();
        buffer.back().cloned()
    }

    /// Enable streaming to a channel
    pub fn enable_streaming(&mut self, sender: mpsc::UnboundedSender<PerformanceMetrics>) {
        self.metrics_sender = Some(sender);
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        *self.is_monitoring.read().unwrap()
    }
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    cpu_usage: f64,
    memory_bytes: u64,
    status: String,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced monitoring trait with streaming support
pub trait Monitor {
    fn start_monitoring(&mut self) -> impl std::future::Future<Output = Result<(), String>>;
    fn stop_monitoring(&mut self);
    fn get_current_metrics(&self) -> PerformanceMetrics;
    fn enable_streaming(&mut self, sender: mpsc::UnboundedSender<PerformanceMetrics>);
    fn is_monitoring(&self) -> bool;
}

// Enhanced default monitoring implementation
pub struct DefaultSystemMonitor {
    monitor: SystemMonitor,
}

impl DefaultSystemMonitor {
    pub fn new() -> Self {
        Self {
            monitor: SystemMonitor::new(),
        }
    }

    pub fn with_buffer_size(max_buffer_size: usize) -> Self {
        Self {
            monitor: SystemMonitor::with_buffer_size(max_buffer_size),
        }
    }
}

impl Monitor for DefaultSystemMonitor {
    async fn start_monitoring(&mut self) -> Result<(), String> {
        self.monitor.start_monitoring().await
    }

    fn stop_monitoring(&mut self) {
        self.monitor.stop_monitoring();
    }

    fn get_current_metrics(&self) -> PerformanceMetrics {
        self.monitor.get_current_metrics()
    }

    fn enable_streaming(&mut self, sender: mpsc::UnboundedSender<PerformanceMetrics>) {
        self.monitor.enable_streaming(sender);
    }

    fn is_monitoring(&self) -> bool {
        self.monitor.is_monitoring()
    }
}

impl Default for DefaultSystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
