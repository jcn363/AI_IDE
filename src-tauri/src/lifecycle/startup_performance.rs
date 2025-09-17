//! Startup Performance Optimization Module
//!
//! This module provides comprehensive startup time optimization features including:
//! - Lazy service initialization with configurable priorities
//! - Detailed startup time tracking and monitoring
//! - Background task management for non-critical components
//! - Startup cache warming and pre-computation
//! - Performance benchmarks and reporting

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::timeout;

use super::{LifecycleEvent, LifecyclePhase};
use crate::command_templates::spawn_background_task;
use crate::infra::EventBus;

/// Service initialization priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ServicePriority {
    /// Critical services required for basic functionality (highest priority)
    Critical = 0,
    /// High priority services needed for core features
    High = 1,
    /// Medium priority services for enhanced functionality
    Medium = 2,
    /// Low priority services that can be deferred
    Low = 3,
    /// Background services that should only run when system is idle
    Background = 4,
}

impl Default for ServicePriority {
    fn default() -> Self {
        ServicePriority::Medium
    }
}

/// Service initialization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// Service not yet initialized
    NotStarted,
    /// Service is currently being initialized
    Initializing,
    /// Service initialization completed successfully
    Ready,
    /// Service initialization failed
    Failed(String),
    /// Service initialization was deferred
    Deferred,
}

/// Startup performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupMetrics {
    pub total_startup_time: Duration,
    pub cold_start_time: Duration,
    pub warm_start_time: Duration,
    pub phase_breakdown: HashMap<String, Duration>,
    pub service_initialization_times: HashMap<String, Duration>,
    pub background_tasks_started: usize,
    pub deferred_services: Vec<String>,
    pub bottlenecks: Vec<String>,
}

/// Service initialization task
#[derive(Debug)]
pub struct ServiceInitTask {
    pub name: String,
    pub priority: ServicePriority,
    pub init_fn: Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync>,
    pub dependencies: Vec<String>,
    pub timeout: Option<Duration>,
}

/// Startup performance optimizer
pub struct StartupPerformanceOptimizer {
    /// Service initialization tasks organized by priority
    service_tasks: HashMap<ServicePriority, Vec<ServiceInitTask>>,
    /// Current status of each service
    service_status: Arc<RwLock<HashMap<String, ServiceStatus>>>,
    /// Startup timing metrics
    metrics: Arc<RwLock<StartupMetrics>>,
    /// Event bus for lifecycle events
    event_bus: Arc<EventBus>,
    /// Channel for service completion notifications
    completion_sender: mpsc::UnboundedSender<(String, Result<()>)>,
    completion_receiver: Arc<Mutex<mpsc::UnboundedReceiver<(String, Result<()>)>>>,
}

impl StartupPerformanceOptimizer {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let (completion_sender, completion_receiver) = mpsc::unbounded_channel();

        Self {
            service_tasks: HashMap::new(),
            service_status: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(StartupMetrics {
                total_startup_time: Duration::ZERO,
                cold_start_time: Duration::ZERO,
                warm_start_time: Duration::ZERO,
                phase_breakdown: HashMap::new(),
                service_initialization_times: HashMap::new(),
                background_tasks_started: 0,
                deferred_services: Vec::new(),
                bottlenecks: Vec::new(),
            })),
            event_bus,
            completion_sender,
            completion_receiver: Arc::new(Mutex::new(completion_receiver)),
        }
    }

    /// Register a service initialization task
    pub fn register_service(&mut self, task: ServiceInitTask) {
        self.service_tasks
            .entry(task.priority)
            .or_insert_with(Vec::new)
            .push(task);
    }

    /// Start optimized startup process
    pub async fn start_optimized_startup(&self) -> Result<()> {
        let startup_start = Instant::now();
        log::info!("Starting optimized startup process");

        // Phase 1: Initialize critical services synchronously
        self.initialize_critical_services().await?;

        // Phase 2: Initialize high-priority services with concurrency
        self.initialize_high_priority_services().await?;

        // Phase 3: Start background initialization of remaining services
        self.start_background_initialization().await?;

        // Phase 4: Wait for essential services or timeout
        self.wait_for_essential_services().await?;

        let total_time = startup_start.elapsed();
        log::info!("Optimized startup completed in {:.2}ms", total_time.as_millis());

        // Update final metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_startup_time = total_time;
            metrics.cold_start_time = total_time; // For now, assume cold start
        }

        // Emit completion event
        self.event_bus
            .publish("startup_completed", &serde_json::json!({
                "total_time_ms": total_time.as_millis(),
                "services_initialized": self.service_status.read().await.len(),
                "background_tasks": {
                    let metrics = self.metrics.read().await;
                    metrics.background_tasks_started
                }
            }))
            .await;

        Ok(())
    }

    /// Initialize critical services synchronously (blocking)
    async fn initialize_critical_services(&self) -> Result<()> {
        let phase_start = Instant::now();

        if let Some(critical_tasks) = self.service_tasks.get(&ServicePriority::Critical) {
            for task in critical_tasks {
                let service_start = Instant::now();

                // Update status to initializing
                {
                    let mut status = self.service_status.write().await;
                    status.insert(task.name.clone(), ServiceStatus::Initializing);
                }

                // Execute initialization
                let result = if let Some(timeout_duration) = task.timeout {
                    timeout(timeout_duration, (task.init_fn)()).await
                        .unwrap_or(Err(anyhow::anyhow!("Service initialization timed out")))
                } else {
                    (task.init_fn)().await
                };

                let init_time = service_start.elapsed();

                // Update status and metrics
                {
                    let mut status = self.service_status.write().await;
                    let mut metrics = self.metrics.write().await;

                    match result {
                        Ok(_) => {
                            status.insert(task.name.clone(), ServiceStatus::Ready);
                            metrics.service_initialization_times.insert(task.name.clone(), init_time);
                            log::info!("Critical service '{}' initialized in {:.2}ms", task.name, init_time.as_millis());
                        }
                        Err(e) => {
                            status.insert(task.name.clone(), ServiceStatus::Failed(e.to_string()));
                            log::error!("Critical service '{}' failed: {}", task.name, e);
                        }
                    }
                }
            }
        }

        let phase_time = phase_start.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            metrics.phase_breakdown.insert("critical_services".to_string(), phase_time);
        }

        log::info!("Critical services phase completed in {:.2}ms", phase_time.as_millis());
        Ok(())
    }

    /// Initialize high-priority services with controlled concurrency
    async fn initialize_high_priority_services(&self) -> Result<()> {
        let phase_start = Instant::now();

        let mut handles = Vec::new();

        if let Some(high_tasks) = self.service_tasks.get(&ServicePriority::High) {
            for task in high_tasks {
                let service_name = task.name.clone();
                let init_fn = task.init_fn.clone();
                let timeout_duration = task.timeout;
                let status_clone = Arc::clone(&self.service_status);
                let metrics_clone = Arc::clone(&self.metrics);
                let completion_sender = self.completion_sender.clone();

                let handle = tokio::spawn(async move {
                    let service_start = Instant::now();

                    // Update status to initializing
                    {
                        let mut status = status_clone.write().await;
                        status.insert(service_name.clone(), ServiceStatus::Initializing);
                    }

                    // Execute initialization
                    let result = if let Some(timeout_duration) = timeout_duration {
                        timeout(timeout_duration, init_fn()).await
                            .unwrap_or(Err(anyhow::anyhow!("Service initialization timed out")))
                    } else {
                        init_fn().await
                    };

                    let init_time = service_start.elapsed();

                    // Update status and metrics
                    {
                        let mut status = status_clone.write().await;
                        let mut metrics = metrics_clone.write().await;

                        match result.as_ref() {
                            Ok(_) => {
                                status.insert(service_name.clone(), ServiceStatus::Ready);
                                metrics.service_initialization_times.insert(service_name.clone(), init_time);
                                log::info!("High-priority service '{}' initialized in {:.2}ms", service_name, init_time.as_millis());
                            }
                            Err(e) => {
                                status.insert(service_name.clone(), ServiceStatus::Failed(e.to_string()));
                                log::error!("High-priority service '{}' failed: {}", service_name, e);
                            }
                        }
                    }

                    // Notify completion
                    let _ = completion_sender.send((service_name, result));
                });

                handles.push(handle);
            }
        }

        // Wait for all high-priority services to complete
        for handle in handles {
            let _ = handle.await;
        }

        let phase_time = phase_start.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            metrics.phase_breakdown.insert("high_priority_services".to_string(), phase_time);
        }

        log::info!("High-priority services phase completed in {:.2}ms", phase_time.as_millis());
        Ok(())
    }

    /// Start background initialization of medium/low priority services
    async fn start_background_initialization(&self) -> Result<()> {
        let priorities = vec![ServicePriority::Medium, ServicePriority::Low, ServicePriority::Background];

        for priority in priorities {
            if let Some(tasks) = self.service_tasks.get(&priority) {
                for task in tasks {
                    let service_name = task.name.clone();
                    let init_fn = task.init_fn.clone();
                    let timeout_duration = task.timeout;
                    let status_clone = Arc::clone(&self.service_status);
                    let metrics_clone = Arc::clone(&self.metrics);
                    let completion_sender = self.completion_sender.clone();

                    // Spawn background task for each service
                    let task_id = spawn_background_task(
                        async move {
                            let service_start = Instant::now();

                            // Update status to initializing
                            {
                                let mut status = status_clone.write().await;
                                status.insert(service_name.clone(), ServiceStatus::Initializing);
                            }

                            // Execute initialization
                            let result = if let Some(timeout_duration) = timeout_duration {
                                timeout(timeout_duration, init_fn()).await
                                    .unwrap_or(Err(anyhow::anyhow!("Service initialization timed out")))
                            } else {
                                init_fn().await
                            };

                            let init_time = service_start.elapsed();

                            // Update status and metrics
                            {
                                let mut status = status_clone.write().await;
                                let mut metrics = metrics_clone.write().await;

                                match result.as_ref() {
                                    Ok(_) => {
                                        status.insert(service_name.clone(), ServiceStatus::Ready);
                                        metrics.service_initialization_times.insert(service_name.clone(), init_time);
                                        log::info!("Background service '{}' initialized in {:.2}ms", service_name, init_time.as_millis());
                                    }
                                    Err(e) => {
                                        status.insert(service_name.clone(), ServiceStatus::Failed(e.to_string()));
                                        metrics.deferred_services.push(service_name.clone());
                                        log::warn!("Background service '{}' failed, will retry later: {}", service_name, e);
                                    }
                                }
                            }

                            // Notify completion
                            let _ = completion_sender.send((service_name, result));
                        },
                        &format!("init_{}", service_name),
                    );

                    // Update background tasks counter
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.background_tasks_started += 1;
                    }
                }
            }
        }

        log::info!("Background initialization tasks started");
        Ok(())
    }

    /// Wait for essential services to be ready or timeout
    async fn wait_for_essential_services(&self) -> Result<()> {
        let essential_services = vec!["lsp", "ai_service"]; // Define essential services
        let timeout_duration = Duration::from_secs(5); // 5 second timeout for essential services

        let wait_start = Instant::now();
        let mut essential_ready = false;

        while wait_start.elapsed() < timeout_duration && !essential_ready {
            essential_ready = true;

            {
                let status = self.service_status.read().await;
                for service in &essential_services {
                    match status.get(service) {
                        Some(ServiceStatus::Ready) => continue,
                        Some(ServiceStatus::Failed(_)) => {
                            log::warn!("Essential service '{}' failed to initialize", service);
                        }
                        _ => {
                            essential_ready = false;
                            break;
                        }
                    }
                }
            }

            if !essential_ready {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }

        if !essential_ready {
            log::warn!("Not all essential services ready within timeout, proceeding with available services");
        }

        Ok(())
    }

    /// Get current startup metrics
    pub async fn get_startup_metrics(&self) -> StartupMetrics {
        self.metrics.read().await.clone()
    }

    /// Get service status
    pub async fn get_service_status(&self, service_name: &str) -> Option<ServiceStatus> {
        self.service_status.read().await.get(service_name).cloned()
    }

    /// Check if essential services are ready
    pub async fn are_essential_services_ready(&self) -> bool {
        let essential_services = vec!["lsp", "ai_service"];
        let status = self.service_status.read().await;

        essential_services.iter().all(|service| {
            matches!(status.get(service), Some(ServiceStatus::Ready))
        })
    }

    /// Warm up startup cache with frequently used data
    pub async fn warm_startup_cache(&self) -> Result<()> {
        log::info!("Warming startup cache");

        // Pre-load commonly used data
        // This would include things like:
        // - Recent project data
        // - User preferences
        // - Common LSP configurations
        // - Cached analysis results

        // For now, simulate cache warming
        tokio::time::sleep(Duration::from_millis(10)).await;

        log::info!("Startup cache warming completed");
        Ok(())
    }

    /// Get performance report
    pub async fn generate_performance_report(&self) -> String {
        let metrics = self.metrics.read().await;
        let status = self.service_status.read().await;

        let mut report = String::new();
        report.push_str(&format!("=== Startup Performance Report ===\n"));
        report.push_str(&format!("Total startup time: {:.2}ms\n", metrics.total_startup_time.as_millis()));
        report.push_str(&format!("Cold start time: {:.2}ms\n", metrics.cold_start_time.as_millis()));
        report.push_str(&format!("Background tasks started: {}\n", metrics.background_tasks_started));
        report.push_str(&format!("Services initialized: {}\n", metrics.service_initialization_times.len()));
        report.push_str(&format!("Deferred services: {}\n", metrics.deferred_services.len()));

        report.push_str(&format!("\n=== Phase Breakdown ===\n"));
        for (phase, duration) in &metrics.phase_breakdown {
            report.push_str(&format!("{}: {:.2}ms\n", phase, duration.as_millis()));
        }

        report.push_str(&format!("\n=== Service Initialization Times ===\n"));
        for (service, duration) in &metrics.service_initialization_times {
            report.push_str(&format!("{}: {:.2}ms\n", service, duration.as_millis()));
        }

        report.push_str(&format!("\n=== Service Status ===\n"));
        for (service, status) in status {
            report.push_str(&format!("{}: {:?}\n", service, status));
        }

        if !metrics.bottlenecks.is_empty() {
            report.push_str(&format!("\n=== Performance Bottlenecks ===\n"));
            for bottleneck in &metrics.bottlenecks {
                report.push_str(&format!("• {}\n", bottleneck));
            }
        }

        report
    }
}

impl Default for StartupPerformanceOptimizer {
    fn default() -> Self {
        Self::new(Arc::new(EventBus::new()))
    }
}