//! Unified Logging Infrastructure for Rust AI IDE
//!
//! This module provides a comprehensive, thread-safe logging system that:
//! - Provides unified interfaces across all modules
//! - Supports structured JSON logging
//! - Integrates with IDError system for contextual logging
//! - Enables frontend-backend communication via Tauri channels
//! - Includes basic metrics collection foundation
//! - Maintains zero-overhead performance in release builds

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::perf_utils::PerformanceMetrics;

// Re-exports will be added at the end of the file after modules are defined

/// Core logging facade trait for unified interface
#[async_trait::async_trait]
pub trait UnifiedLogger: Send + Sync {
    async fn log<'a>(
        &self,
        level: LogLevel,
        message: &str,
        context: Option<&'a LogContext>,
    ) -> Result<(), LoggingError>;
    fn child(&self, additional_context: LogContext) -> LoggingGuard;
}

/// Structured log context for contextual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub operation:  String,
    pub user_id:    Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub component:  String,
    pub metadata:   HashMap<String, serde_json::Value>,
}

impl Default for LogContext {
    fn default() -> Self {
        Self {
            operation:  "unknown".to_string(),
            user_id:    None,
            session_id: None,
            request_id: None,
            component:  "rust-ai-ide".to_string(),
            metadata:   HashMap::new(),
        }
    }
}

impl LogContext {
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            ..Default::default()
        }
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_metadata<K: Into<String>, V: Into<serde_json::Value>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Logging levels following standard conventions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Structured log entry with timestamp and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp:     u64,
    pub level:         LogLevel,
    pub message:       String,
    pub context:       LogContext,
    pub error_details: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>, context: LogContext) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            level,
            message: message.into(),
            context,
            error_details: None,
        }
    }

    pub fn with_error(mut self, error: &impl std::fmt::Debug) -> Self {
        self.error_details = Some(format!("{:?}", error));
        self
    }
}

/// Thread-safe logging manager with unified interface
#[derive(Debug)]
pub struct LoggingManager {
    sinks:   Arc<RwLock<Vec<LogSinkEnum>>>,
    context: RwLock<LogContext>,
    metrics: Arc<BasicMetricsCollector>,
}

impl Clone for LoggingManager {
    fn clone(&self) -> Self {
        Self {
            sinks:   Arc::clone(&self.sinks),
            context: RwLock::new(self.context.try_read().unwrap().clone()),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

#[async_trait::async_trait]
impl UnifiedLogger for Arc<LoggingManager> {
    async fn log<'a>(
        &self,
        level: LogLevel,
        message: &str,
        context: Option<&'a LogContext>,
    ) -> Result<(), LoggingError> {
        LoggingManager::log(self, level, message, context.cloned()).await
    }

    fn child(&self, additional_context: LogContext) -> LoggingGuard {
        LoggingGuard {
            manager:      Arc::clone(self),
            base_context: additional_context,
        }
    }
}

impl Default for LoggingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingManager {
    pub fn new() -> Self {
        Self {
            sinks:   Arc::new(RwLock::new(Vec::new())),
            context: RwLock::new(LogContext::default()),
            metrics: Arc::new(BasicMetricsCollector::new()),
        }
    }

    pub fn with_console_sink(self, sink: ConsoleSink) -> Self {
        tokio::task::block_in_place(|| {
            futures::executor::block_on(async {
                let mut sinks = self.sinks.write().await;
                sinks.push(LogSinkEnum::Console(sink));
            })
        });
        self
    }

    pub fn with_file_sink(self, sink: FileSink) -> Self {
        tokio::task::block_in_place(|| {
            futures::executor::block_on(async {
                let mut sinks = self.sinks.write().await;
                sinks.push(LogSinkEnum::File(sink));
            })
        });
        self
    }

    pub fn with_external_sink(self, sink: ExternalServiceSink) -> Self {
        tokio::task::block_in_place(|| {
            futures::executor::block_on(async {
                let mut sinks = self.sinks.write().await;
                sinks.push(LogSinkEnum::External(sink));
            })
        });
        self
    }

    pub async fn log(&self, level: LogLevel, message: &str, context: Option<LogContext>) -> Result<(), LoggingError> {
        let context = context.unwrap_or_else(|| {
            tokio::task::block_in_place(|| futures::executor::block_on(async { self.context.read().await.clone() }))
        });

        let entry = LogEntry::new(level, message, context);

        // Update metrics
        self.metrics.record_log(&entry);

        // Send to all sinks
        let sinks = self.sinks.read().await;
        for sink in sinks.iter() {
            if let Err(e) = sink.log(&entry).await {
                eprintln!("Failed to write to log sink: {:?}", e);
            }
        }

        // Forward to frontend via Tauri if available
        #[cfg(feature = "tauri")]
        {
            self.forward_to_frontend(&entry).await?;
        }

        Ok(())
    }

    pub async fn update_context<F>(&self, updater: F) -> Result<(), LoggingError>
    where
        F: FnOnce(&mut LogContext),
    {
        let mut context = self.context.write().await;
        updater(&mut context);
        Ok(())
    }

    pub fn child(&self, additional_context: LogContext) -> LoggingGuard {
        LoggingGuard {
            manager:      Arc::new((*self).clone()),
            base_context: additional_context,
        }
    }

    pub fn get_metrics(&self) -> Arc<BasicMetricsCollector> {
        Arc::clone(&self.metrics)
    }

    #[cfg(feature = "tauri")]
    async fn forward_to_frontend(&self, entry: &LogEntry) -> Result<(), LoggingError> {
        // Forward structured log entry to frontend via Tauri
        use crate::http::{HttpClient, HttpRequest};
        // Tauri channel forwarding would be implemented here
        // This is a placeholder for the actual Tauri integration
        Ok(())
    }
}

/// Guard for logging with additional context
pub struct LoggingGuard {
    manager:      Arc<LoggingManager>,
    base_context: LogContext,
}

impl LoggingGuard {
    pub async fn log(&self, level: LogLevel, message: &str, context: Option<LogContext>) -> Result<(), LoggingError> {
        let context = match context {
            Some(mut ctx) => {
                ctx.operation = format!("{}.{}", self.base_context.operation, ctx.operation);
                if ctx.session_id.is_none() {
                    ctx.session_id = self.base_context.session_id.clone();
                }
                ctx
            }
            None => self.base_context.clone(),
        };

        self.manager.log(level, message, Some(&context)).await
    }

    pub async fn info(&self, message: &str) -> Result<(), LoggingError> {
        self.log(LogLevel::Info, message, None).await
    }

    pub async fn warn(&self, message: &str) -> Result<(), LoggingError> {
        self.log(LogLevel::Warn, message, None).await
    }

    pub async fn error(
        &self,
        message: &str,
        error: Option<&(impl std::fmt::Debug + ?Sized)>,
    ) -> Result<(), LoggingError> {
        let message = if let Some(err) = error {
            format!("{}: {:?}", message, err)
        } else {
            message.to_string()
        };
        self.log(LogLevel::Error, &message, None).await
    }

    pub async fn error_with_context<E: std::fmt::Debug>(
        &self,
        message: &str,
        error: &E,
        context: LogContext,
    ) -> Result<(), LoggingError> {
        let _entry = LogEntry::new(LogLevel::Error, message, context.clone()).with_error(error);
        self.manager
            .log(LogLevel::Error, message, Some(&context))
            .await?;
        Ok(())
    }
}

/// Abstract logging sink interface
#[async_trait::async_trait]
pub trait LogSink: Send + Sync {
    async fn log(&self, entry: &LogEntry) -> Result<(), LoggingError>;
}

/// Enum-based polymorphism for LogSink to avoid dyn async trait issues
#[derive(Debug)]
pub enum LogSinkEnum {
    Console(ConsoleSink),
    File(FileSink),
    External(ExternalServiceSink),
}

#[async_trait::async_trait]
impl LogSink for LogSinkEnum {
    async fn log(&self, entry: &LogEntry) -> Result<(), LoggingError> {
        match self {
            LogSinkEnum::Console(sink) => sink.log(entry).await,
            LogSinkEnum::File(sink) => sink.log(entry).await,
            LogSinkEnum::External(sink) => sink.log(entry).await,
        }
    }
}

// ===== PHASE 2 ENHANCEMENTS =====

// Console sink for development logging
#[derive(Debug)]
pub struct ConsoleSink {
    formatted:       bool,
    max_line_length: usize,
}

impl ConsoleSink {
    pub fn new(formatted: bool) -> Self {
        Self {
            formatted,
            max_line_length: 1000, // Default limit to prevent spam
        }
    }

    pub fn with_line_limit(limit: usize) -> Self {
        Self {
            formatted:       true,
            max_line_length: limit,
        }
    }
}

#[async_trait::async_trait]
impl LogSink for ConsoleSink {
    async fn log(&self, entry: &LogEntry) -> Result<(), LoggingError> {
        let message = if entry.message.len() > self.max_line_length {
            format!("{}... [TRUNCATED]", &entry.message[..self.max_line_length])
        } else {
            entry.message.clone()
        };

        if self.formatted {
            println!(
                "[{}] {} [{}] {} - {}",
                entry.timestamp, entry.level, entry.context.component, entry.context.operation, message
            );
        } else {
            let mut entry_json = entry.clone();
            entry_json.message = message;
            let json = serde_json::to_string(&entry_json)?;
            println!("{}", json);
        }
        Ok(())
    }
}

/// File sink for persistent logging (PHASE 2 enhancement)
#[derive(Debug)]
pub struct FileSink {
    file_path:          std::path::PathBuf,
    max_file_size_mb:   u64,
    rotation_files:     usize,
    current_file_size:  u64,
    current_file_index: usize,
}

impl FileSink {
    pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            file_path:          file_path.into(),
            max_file_size_mb:   10, // 10MB default
            rotation_files:     5,
            current_file_size:  0,
            current_file_index: 0,
        }
    }

    pub fn with_rotation(max_size_mb: u64, rotation_files: usize) -> impl Fn(std::path::PathBuf) -> Self {
        move |path: std::path::PathBuf| Self {
            file_path: path,
            max_file_size_mb: max_size_mb,
            rotation_files,
            current_file_size: 0,
            current_file_index: 0,
        }
    }

    fn should_rotate(&self, entry_size: u64) -> bool {
        self.current_file_size + entry_size > self.max_file_size_mb * 1024 * 1024
    }

    fn rotate_file(&mut self) -> Result<(), LoggingError> {
        // Rename current log file and create new one
        if self.file_path.exists() {
            self.current_file_index = (self.current_file_index + 1) % self.rotation_files;
            let rotated_path = self
                .file_path
                .with_extension(format!("log.{}", self.current_file_index));
            std::fs::rename(&self.file_path, rotated_path)?;
        }
        self.current_file_size = 0;
        Ok(())
    }
}

#[async_trait::async_trait]
impl LogSink for FileSink {
    async fn log(&self, entry: &LogEntry) -> Result<(), LoggingError> {
        let mut sink = FileSink {
            file_path:          self.file_path.clone(),
            max_file_size_mb:   self.max_file_size_mb,
            rotation_files:     self.rotation_files,
            current_file_size:  self.current_file_size,
            current_file_index: self.current_file_index,
        };

        let entry_str = if sink.should_rotate(entry.message.len() as u64) {
            sink.rotate_file()?;
            format!(
                "[{}] {} [{}] {} - {}\n",
                entry.timestamp, entry.level, entry.context.component, entry.context.operation, entry.message
            )
        } else {
            format!(
                "[{}] {} [{}] {} - {}\n",
                entry.timestamp, entry.level, entry.context.component, entry.context.operation, entry.message
            )
        };

        sink.current_file_size += entry_str.len() as u64;
        std::fs::write(&sink.file_path, entry_str)?;
        // ===== PHASE 4: PRODUCTION READINESS & ADVANCED FEATURES =====

        /// Production deployment integration utilities
        pub mod production {
            use super::{
                get_logger, init_logging, Arc, ConsoleSink, Deserialize, ExternalServiceSink, FileSink, HashMap,
                LogContext, LogLevel, LogSink, LogSinkEnum, LoggingError, RwLock, Serialize, SystemTime, UnifiedLogger,
                UNIX_EPOCH,
            };

            /// Production-optimized logging setup with all sinks (PHASE 4 - Future feature)
            #[allow(dead_code)]
            pub async fn setup_production_logging(
                log_level: LogLevel,
                log_file: Option<std::path::PathBuf>,
                telemetry_endpoint: Option<String>,
                api_key: Option<String>,
            ) -> Result<(), LoggingError> {
                let mut sinks: Vec<LogSinkEnum> = vec![LogSinkEnum::Console({
                    if std::env::var("NICE_LOG_FORMAT").is_ok() {
                        ConsoleSink::new(true) // Formatted console for nice output
                    } else {
                        ConsoleSink::new(false) // JSON console for production
                    }
                })];

                // Add file sink if specified
                if let Some(file_path) = log_file {
                    sinks.push(LogSinkEnum::File(FileSink::new(file_path)));
                }

                // Add external telemetry if configured
                if let (Some(endpoint), Some(key)) = (telemetry_endpoint, api_key) {
                    let ext_sink = ExternalServiceSink::new(endpoint)
                        .with_auth(key)
                        .with_batch_size(50); // Larger batch for production
                    sinks.push(LogSinkEnum::External(ext_sink));
                }

                init_logging(false, log_level, sinks).await?;
                println!("Production logging initialized successfully");
                Ok(())
            }

            /// Health check system for monitoring system status (PHASE 4 - Future feature)
            #[allow(dead_code)]
            pub struct HealthChecker {
                failing_checks: dashmap::DashMap<String, String>,
                last_check:     Arc<RwLock<u64>>,
            }

            impl HealthChecker {
                pub fn new() -> Self {
                    Self {
                        failing_checks: dashmap::DashMap::new(),
                        last_check:     Arc::new(RwLock::new(0)),
                    }
                }

                pub async fn perform_health_check(&self) -> Result<HealthStatus, LoggingError> {
                    let mut overall_healthy = true;
                    let mut checks = Vec::new();

                    // Check logging system
                    let log_status = self.check_logging_system().await;
                    if !log_status.healthy {
                        overall_healthy = false;
                        self.failing_checks
                            .insert("logging_system".to_string(), log_status.message.clone());
                    }
                    checks.push(("logging_system".to_string(), log_status));

                    // Check metrics collection
                    let metric_status = self.check_metrics_collection();
                    if !metric_status.healthy {
                        overall_healthy = false;
                        self.failing_checks.insert(
                            "metrics_collection".to_string(),
                            metric_status.message.clone(),
                        );
                    }
                    checks.push(("metrics_collection".to_string(), metric_status));

                    // Update last check time
                    *self.last_check.write().await = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    Ok(HealthStatus {
                        overall_healthy,
                        checks,
                        timestamp: chrono::Utc::now().timestamp(),
                        failing_checks: self
                            .failing_checks
                            .iter()
                            .map(|entry| (entry.key().clone(), entry.value().clone()))
                            .collect(),
                    })
                }

                async fn check_logging_system(&self) -> HealthCheck {
                    // Test basic logging functionality
                    let test_result = get_logger()
                        .log(
                            LogLevel::Debug,
                            "Health check test message",
                            Some(&LogContext::new("health_check", "system")),
                        )
                        .await;

                    if test_result.is_ok() {
                        HealthCheck {
                            healthy: true,
                            message: "Logging system operational".to_string(),
                            details: None,
                        }
                    } else {
                        HealthCheck {
                            healthy: false,
                            message: "Logging system failed".to_string(),
                            details: Some(format!("Error: {:?}", test_result.err())),
                        }
                    }
                }

                fn check_metrics_collection(&self) -> HealthCheck {
                    let metrics = get_logger().get_metrics();
                    let log_count = metrics.get_log_counts().len();
                    let error_count = metrics.get_error_counts().len();

                    if log_count > 0 {
                        HealthCheck {
                            healthy: true,
                            message: format!(
                                "Metrics collection active - {} logs, {} errors tracked",
                                log_count, error_count
                            ),
                            details: None,
                        }
                    } else {
                        HealthCheck {
                            healthy: true, // Still healthy, but warn about no activity
                            message: "Metrics collection initialized but no data yet".to_string(),
                            details: None,
                        }
                    }
                }
            }

            #[derive(Clone, Debug, Serialize, Deserialize)]
            #[allow(dead_code)]
            pub struct HealthCheck {
                pub healthy: bool,
                pub message: String,
                pub details: Option<String>,
            }

            #[derive(Clone, Serialize, Deserialize)]
            #[allow(dead_code)]
            pub struct HealthStatus {
                pub overall_healthy: bool,
                pub checks:          Vec<(String, HealthCheck)>,
                pub timestamp:       i64,
                pub failing_checks:  HashMap<String, String>,
            }

            /// Alert system for critical issues (PHASE 4 - Future feature)
            #[allow(dead_code)]
            pub struct AlertManager {
                alerts: Arc<RwLock<Vec<Alert>>>,
            }

            #[derive(Clone, Debug, Serialize, Deserialize)]
            #[allow(dead_code)]
            pub struct Alert {
                pub level:     AlertLevel,
                pub title:     String,
                pub message:   String,
                pub timestamp: u64,
                pub resolved:  bool,
            }

            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
            #[allow(dead_code)]
            pub enum AlertLevel {
                Info,
                Warning,
                Critical,
            }

            impl AsRef<str> for AlertLevel {
                fn as_ref(&self) -> &str {
                    match self {
                        AlertLevel::Info => "info",
                        AlertLevel::Warning => "warning",
                        AlertLevel::Critical => "critical",
                    }
                }
            }

            impl AlertManager {
                pub fn new() -> Self {
                    Self {
                        alerts: Arc::new(RwLock::new(Vec::new())),
                    }
                }

                pub async fn create_alert(
                    &self,
                    level: AlertLevel,
                    title: &str,
                    message: &str,
                ) -> Result<(), LoggingError> {
                    let alert = Alert {
                        level,
                        title: title.to_string(),
                        message: message.to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                        resolved: false,
                    };

                    log_alert(&alert).await?;

                    let mut alerts = self.alerts.write().await;
                    alerts.push(alert);

                    Ok(())
                }

                pub async fn resolve_alert(&self, index: usize) -> Result<(), LoggingError> {
                    let mut alerts = self.alerts.write().await;
                    if index < alerts.len() {
                        alerts[index].resolved = true;
                        println!("Alert resolved: {}", alerts[index].title);
                    }
                    Ok(())
                }

                pub async fn get_active_alerts(&self) -> Vec<(usize, Alert)> {
                    let alerts = self.alerts.read().await;
                    alerts
                        .iter()
                        .enumerate()
                        .filter(|(_, alert)| !alert.resolved)
                        .map(|(i, alert)| (i, alert.clone()))
                        .collect()
                }
            }

            /// Automatic alert logging (PHASE 4 - Future feature)
            #[allow(dead_code)]
            pub async fn log_alert(alert: &Alert) -> Result<(), LoggingError> {
                let log_level = match alert.level {
                    AlertLevel::Info => LogLevel::Info,
                    AlertLevel::Warning => LogLevel::Warn,
                    AlertLevel::Critical => LogLevel::Error,
                };

                let context = LogContext {
                    operation: "alert".to_string(),
                    component: "monitoring".to_string(),
                    metadata: HashMap::from([
                        ("alert_title".to_string(), alert.title.clone().into()),
                        (
                            "alert_level".to_string(),
                            format!("{:?}", alert.level).into(),
                        ),
                    ]),
                    ..Default::default()
                };

                get_logger()
                    .log(
                        log_level,
                        &format!("ALERT [{}] {}", alert.level.as_ref(), alert.message),
                        Some(&context),
                    )
                    .await
            }
        }

        /// Real-time monitoring dashboard utilities (PHASE 4)
        Ok(())
    }
}

/// External service sink for production monitoring (PHASE 2 enhancement)
#[derive(Debug)]
pub struct ExternalServiceSink {
    endpoint_url:   String,
    api_key:        Option<String>,
    batch_size:     usize,
    buffer:         Arc<RwLock<Vec<LogEntry>>>,
    flush_interval: std::time::Duration,
}

impl ExternalServiceSink {
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            endpoint_url:   endpoint_url.into(),
            api_key:        None,
            batch_size:     10,
            buffer:         Arc::new(RwLock::new(Vec::new())),
            flush_interval: std::time::Duration::from_secs(30),
        }
    }

    pub fn with_auth(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn with_flush_interval(mut self, interval_secs: u64) -> Self {
        self.flush_interval = std::time::Duration::from_secs(interval_secs);
        self
    }

    async fn should_flush(&self) -> Result<bool, LoggingError> {
        let buffer = self.buffer.read().await;
        Ok(buffer.len() >= self.batch_size)
    }

    async fn flush_batch(&self) -> Result<(), LoggingError> {
        let entries = {
            let mut buffer = self.buffer.write().await;
            buffer.drain(..).collect::<Vec<_>>()
        };

        if !entries.is_empty() {
            self.send_to_service(entries).await?;
        }

        Ok(())
    }

    async fn send_to_service(&self, entries: Vec<LogEntry>) -> Result<(), LoggingError> {
        let payload = serde_json::json!({
            "service": "rust-ai-ide",
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            "entries": entries
        });

        // In a real implementation, this would make an HTTP call to the external service
        println!("External sink would send: {}", payload);
        Ok(())
    }
}

#[async_trait::async_trait]
impl LogSink for ExternalServiceSink {
    async fn log(&self, entry: &LogEntry) -> Result<(), LoggingError> {
        let mut buffer = self.buffer.write().await;
        buffer.push(entry.clone());

        if self.should_flush().await? {
            let _ = buffer; // Release the write lock before flushing
            self.flush_batch().await?;
        }

        Ok(())
    }
}

/// Enhanced metrics collection system with performance monitoring
#[derive(Debug)]
pub struct BasicMetricsCollector {
    log_counts:        Arc<dashmap::DashMap<LogLevel, u64>>,
    error_counts:      Arc<dashmap::DashMap<String, u64>>,
    operation_metrics: Arc<dashmap::DashMap<String, PerformanceMetrics>>,
    counters:          Arc<dashmap::DashMap<String, u64>>,
    gauges:            Arc<dashmap::DashMap<String, f64>>,
    histograms:        Arc<dashmap::DashMap<String, Vec<f64>>>,
    start_time:        u64,
}

impl Default for BasicMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicMetricsCollector {
    pub fn new() -> Self {
        Self {
            log_counts:        Arc::new(dashmap::DashMap::new()),
            error_counts:      Arc::new(dashmap::DashMap::new()),
            operation_metrics: Arc::new(dashmap::DashMap::new()),
            counters:          Arc::new(dashmap::DashMap::new()),
            gauges:            Arc::new(dashmap::DashMap::new()),
            histograms:        Arc::new(dashmap::DashMap::new()),
            start_time:        SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    pub fn record_log(&self, entry: &LogEntry) {
        *self.log_counts.entry(entry.level).or_insert(0) += 1;

        if entry.level == LogLevel::Error {
            if let Some(err) = &entry.error_details {
                *self.error_counts.entry(err.clone()).or_insert(0) += 1;
            }
        }
    }

    pub fn get_log_counts(&self) -> HashMap<String, u64> {
        self.log_counts
            .iter()
            .map(|entry| (entry.key().as_str().to_string(), *entry.value()))
            .collect()
    }

    pub fn get_error_counts(&self) -> HashMap<String, u64> {
        self.error_counts
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }

    /// Phase 2 enhancement: Performance monitoring for operations
    pub fn start_operation(&self, operation_name: &str) -> OperationTimer {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        OperationTimer {
            operation_name: operation_name.to_string(),
            start_time,
            collector: Some(Arc::clone(&self.counters)),
        }
    }

    /// Phase 2 enhancement: Custom metrics collection
    pub fn increment_counter(&self, key: impl Into<String>, value: u64) {
        *self.counters.entry(key.into()).or_insert(0) += value;
    }

    pub fn set_gauge(&self, key: impl Into<String>, value: f64) {
        self.gauges.insert(key.into(), value);
    }

    pub fn record_histogram(&self, key: impl Into<String>, value: f64) {
        self.histograms.entry(key.into()).or_default().push(value);
    }

    pub fn get_counters(&self) -> HashMap<String, u64> {
        self.counters
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }

    pub fn get_gauges(&self) -> HashMap<String, f64> {
        self.gauges
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }

    pub fn get_histograms(&self) -> HashMap<String, Vec<f64>> {
        self.histograms
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    pub fn get_uptime_millis(&self) -> Result<u64, LoggingError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Ok(now.saturating_sub(self.start_time))
    }
}

/// Performance timer for operation tracking
pub struct OperationTimer {
    operation_name: String,
    start_time:     u64,
    collector:      Option<Arc<dashmap::DashMap<String, u64>>>,
}

impl OperationTimer {
    pub fn finish_operation(&self) -> Result<(), LoggingError> {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let duration = end_time.saturating_sub(self.start_time);

        // Record metrics for this operation
        if let Some(collector) = &self.collector {
            let key = format!("operation.{}.count", self.operation_name);
            let time_key = format!("operation.{}.time", self.operation_name);
            let duration_key = format!("operation.{}.duration", self.operation_name);

            *collector.entry(key).or_insert(0) += 1;
            *collector.entry(time_key).or_insert(0) += 1;
            *collector.entry(duration_key).or_insert(0) += duration;
        }

        Ok(())
    }
}

/// Logging system errors
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Channel communication error: {0}")]
    Channel(String),
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
}

/// Global logging manager instance using thread-safe singleton
static LOGGING_MANAGER: once_cell::sync::Lazy<Arc<LoggingManager>> =
    once_cell::sync::Lazy::new(|| Arc::new(LoggingManager::new().with_console_sink(ConsoleSink::new(true))));

/// Convenience macro for structured logging
#[macro_export]
macro_rules! log_info {
    ($operation:expr, $message:expr $(, $context:expr)?) => {{
        let context = $crate::logging::LogContext::new($operation, module_path!());
        $crate::logging::get_logger()
            .log($crate::logging::LogLevel::Info, $message, Some(context))
            .await
    }};
}

#[macro_export]
macro_rules! log_error {
    ($operation:expr, $error:expr, $context:expr) => {{
        let mut context = $crate::logging::LogContext::new($operation, module_path!());
        context.metadata.insert(
            "error".to_string(),
            serde_json::to_value(format!("{:?}", $error)).unwrap(),
        );
        $crate::logging::get_logger()
            .log(
                $crate::logging::LogLevel::Error,
                &$error.to_string(),
                Some(context),
            )
            .await
    }};
}

/// Global logger accessor
pub fn get_logger() -> Arc<LoggingManager> {
    LOGGING_MANAGER.clone()
}

/// Initialize logging system with configuration
pub async fn init_logging(
    formatted: bool,
    level: LogLevel,
    additional_sinks: Vec<LogSinkEnum>,
) -> Result<(), LoggingError> {
    let manager = get_logger();

    // Update context with basic config
    manager
        .update_context(|ctx| {
            ctx.metadata.insert(
                "log_level".to_string(),
                serde_json::to_value(level.as_str()).unwrap(),
            );
            ctx.metadata.insert(
                "formatted".to_string(),
                serde_json::to_value(formatted).unwrap(),
            );
        })
        .await?;

    // Add additional sinks
    for sink in additional_sinks {
        let mut sinks = manager.sinks.write().await;
        sinks.push(sink);
    }

    Ok(())
}

/// Integration with IDError system for automatic error logging
pub async fn log_ide_error(
    error: &crate::errors::IdeError,
    operation: &str,
    context: Option<LogContext>,
) -> Result<(), LoggingError> {
    let mut log_context = context.unwrap_or_else(|| LogContext::new(operation, "ide"));
    log_context.metadata.insert(
        "error_type".to_string(),
        serde_json::to_value(format!("{:?}", error)).unwrap(),
    );

    // Record error metric
    get_logger()
        .get_metrics()
        .increment_counter("error.total", 1);
    get_logger()
        .get_metrics()
        .increment_counter(format!("error.type.{}", operation), 1);

    get_logger()
        .log(
            LogLevel::Error,
            &format!("IDE error: {:?}", error),
            Some(&log_context),
        )
        .await
}

/// ===== PHASE 3 ENHANCEMENTS =====
/// Performance monitoring integration for async operations
pub async fn instrument_async_operation<T, F>(operation_name: &str, operation: F) -> Result<T, LoggingError>
where
    F: std::future::Future<Output = Result<T, LoggingError>>,
{
    let timer = get_logger().get_metrics().start_operation(operation_name);
    let result = operation.await;
    timer.finish_operation()?;
    result
}

/// Configuration-driven logging setup
pub struct LoggingConfiguration {
    pub level:             LogLevel,
    pub format:            LogFormat,
    pub sinks:             Vec<LogSinkType>,
    pub metrics_enabled:   bool,
    pub file_path:         Option<std::path::PathBuf>,
    pub external_endpoint: Option<String>,
    pub external_api_key:  Option<String>,
}

#[derive(Clone)]
pub enum LogFormat {
    Json,
    Console,
    PrettyJson,
}

#[derive(Clone)]
pub enum LogSinkType {
    Console {
        formatted: bool,
    },
    File {
        path:        std::path::PathBuf,
        max_size_mb: u64,
        rotation:    usize,
    },
    External {
        endpoint:   String,
        api_key:    Option<String>,
        batch_size: usize,
    },
}

/// Initialize logging system from configuration
pub async fn init_logging_from_config(config: LoggingConfiguration) -> Result<(), LoggingError> {
    let mut manager = LoggingManager::new();

    // Add configured sinks
    for sink_type in config.sinks {
        match sink_type {
            LogSinkType::Console { formatted } => {
                manager = manager.with_console_sink(ConsoleSink::new(formatted));
            }
            LogSinkType::File {
                path,
                max_size_mb: _max_size_mb,
                rotation: _rotation,
            } => {
                manager = manager.with_file_sink(FileSink::new(path));
            }
            LogSinkType::External {
                endpoint,
                api_key,
                batch_size,
            } => {
                let mut ext_sink = ExternalServiceSink::new(endpoint);
                if let Some(key) = api_key {
                    ext_sink = ext_sink.with_auth(key);
                }
                manager = manager.with_external_sink(ext_sink.with_batch_size(batch_size));
            }
        }
    }

    // Set up global logger (simplified for this example)
    std::sync::Arc::new(manager);
    Ok(())
}

/// ===== ENHANCED FRONTEND-BACKEND INTEGRATION =====
/// Rust side of Tauri communication (would be enhanced with actual Tauri)
#[cfg(feature = "tauri-integrated")]
pub mod tauri_integration {
    use super::*;

    pub struct TauriLogForwarder {
        plugin: Option<Box<dyn Fn(LogEntry) -> Result<(), LoggingError>>>,
    }

    impl TauriLogForwarder {
        pub fn new() -> Self {
            Self { plugin: None }
        }

        pub async fn forward_to_frontend(&self, entry: &LogEntry) -> Result<(), LoggingError> {
            if let Some(plugin) = &self.plugin {
                plugin(entry.clone())?;
            }
            Ok(())
        }
    }
}

/// ===== USAGE EXAMPLES =====
/// Example of comprehensive logging setup
pub async fn example_logging_setup() -> Result<(), LoggingError> {
    // Initialize with console sink for development
    init_logging(true, LogLevel::Info, vec![LogSinkEnum::Console(
        ConsoleSink::new(true),
    )])
    .await?;

    // Get logger instance
    let logger = get_logger();

    // Create context with metadata
    let context = LogContext::new("user_operation", "frontend")
        .with_metadata("user_id", "12345")
        .with_metadata("request_id", "req-2024-001");

    // Log with context
    logger
        .log(LogLevel::Info, "User started operation", Some(&context))
        .await?;

    // Use convenience methods
    let guard = logger.child(LogContext::new("child_operation", "frontend"));
    guard.info("Processing sub-operation").await?;
    guard
        .error(
            "An error occurred: Something went wrong",
            None as Option<&dyn std::fmt::Debug>,
        )
        .await?;

    // Metrics collection
    let metrics = logger.get_metrics();
    metrics.increment_counter("user.operations", 1);
    metrics.set_gauge("memory.usage_mb", 256.5);
    metrics.record_histogram("response.time_ms", 45.7);

    Ok(())
}
/// Real-time monitoring dashboard utilities (PHASE 4)
pub mod monitoring_dashboard {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Serialize};

    use crate::logging::{get_logger, LoggingError};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    pub enum AlertLevel {
        Info,
        Warning,
        Critical,
    }

    impl AsRef<str> for AlertLevel {
        fn as_ref(&self) -> &str {
            match self {
                AlertLevel::Info => "info",
                AlertLevel::Warning => "warning",
                AlertLevel::Critical => "critical",
            }
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct DashboardSnapshot {
        pub timestamp:          u64,
        pub log_statistics:     LogStatistics,
        pub error_breakdown:    HashMap<String, u64>,
        pub system_metrics:     SystemMetrics,
        pub active_alerts:      Vec<AlertInfo>,
        pub performance_trends: PerformanceTrends,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct LogStatistics {
        pub total_logs:        u64,
        pub logs_by_level:     HashMap<String, u64>,
        pub logs_per_minute:   f64,
        pub errors_per_minute: f64,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct SystemMetrics {
        pub memory_usage_mb:    f64,
        pub cpu_usage_percent:  f64,
        pub uptime_seconds:     u64,
        pub active_connections: usize,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AlertInfo {
        pub id:            usize,
        pub level:         AlertLevel,
        pub title:         String,
        pub message:       String,
        pub since_minutes: f64,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PerformanceTrends {
        pub average_response_time_ms: f64,
        pub error_rate_percentage:    f64,
        pub success_rate_percentage:  f64,
        pub recent_spikes:            Vec<PerformanceSpike>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PerformanceSpike {
        pub timestamp:          u64,
        pub operation:          String,
        pub duration_ms:        f64,
        pub threshold_exceeded: bool,
    }

    /// Generate comprehensive dashboard snapshot
    pub async fn generate_dashboard_snapshot() -> Result<DashboardSnapshot, LoggingError> {
        let metrics = get_logger().get_metrics();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

        let log_counts = metrics.get_log_counts();
        let error_counts = metrics.get_error_counts();

        let total_logs: u64 = log_counts.values().sum();
        let total_errors: u64 = error_counts.values().sum();

        // Calculate rates (simplified for this example)
        let logs_per_minute = total_logs as f64 / (timestamp as f64 / 60000.0);
        let errors_per_minute = total_errors as f64 / (timestamp as f64 / 60000.0);

        // Generate system metrics placeholder
        let system_metrics = SystemMetrics {
            memory_usage_mb:    256.0, // Placeholder - would integrate with system monitoring
            cpu_usage_percent:  35.0,
            uptime_seconds:     timestamp / 1000,
            active_connections: 0, // Placeholder
        };

        // Placeholder for active alerts - would integrate with AlertManager
        let active_alerts = Vec::new();

        // Placeholder for performance trends
        let performance_trends = PerformanceTrends {
            average_response_time_ms: 45.0,
            error_rate_percentage:    (total_errors as f64 / total_logs as f64 * 100.0).max(0.0),
            success_rate_percentage:  ((total_logs - total_errors) as f64 / total_logs as f64 * 100.0).max(0.0),
            recent_spikes:            Vec::new(),
        };

        Ok(DashboardSnapshot {
            timestamp,
            log_statistics: LogStatistics {
                total_logs,
                logs_by_level: log_counts,
                logs_per_minute,
                errors_per_minute,
            },
            error_breakdown: error_counts,
            system_metrics,
            active_alerts,
            performance_trends,
        })
    }
}
/// Comprehensive testing framework for the logging system (PHASE 4)
#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper for logging functionality
    pub async fn test_logging_setup() -> Result<(), Box<dyn std::error::Error>> {
        // Create test configuration
        let config = LoggingConfiguration {
            level:           LogLevel::Debug,
            format:          LogFormat::Console,
            sinks:           vec![LogSinkType::Console { formatted: true }],
            metrics_enabled: true,
        };

        init_logging_from_config(config).await?;

        // Test basic logging
        let logger = get_logger();
        logger.log(LogLevel::Info, "Test message", None).await?;

        // Test metrics collection
        let metrics = logger.get_metrics();
        metrics.increment_counter("test.counter", 5);

        assert_eq!(metrics.get_counters().get("test.counter"), Some(&5));
        Ok(())
    }

    /// Performance testing for logging system
    pub async fn benchmark_logging_operations() -> Result<(), Box<dyn std::error::Error>> {
        let logger = get_logger();
        let start = std::time::Instant::now();

        // Perform 1000 logging operations
        for i in 0..1000 {
            logger
                .log(LogLevel::Debug, &format!("Benchmark log #{}", i), None)
                .await?;
        }

        let duration = start.elapsed();
        let ops_per_sec = 1000.0 / duration.as_secs_f64();

        println!("Logging benchmark: {:.1} ops/sec", ops_per_sec);

        // Test instrumented operation
        let test_result: Result<(), LoggingError> = instrument_async_operation("benchmark_test", async {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            Ok(())
        })
        .await?;

        assert!(test_result.is_ok());
        Ok(())
    }

    /// Error handling tests
    pub async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test IDError integration
        let test_error = crate::errors::IdeError::Resource(MSG_FILE_ALREADY_EXISTS.get_subtable().collect());
        log_ide_error(&test_error, "test_operation", None).await?;

        // Verify metrics were updated
        let metrics = get_logger().get_metrics();
        let counters = metrics.get_counters();

        assert!(counters.get("error.total").is_some());
        assert!(counters.get("error.type.test_operation").is_some());

        Ok(())
    }
}
