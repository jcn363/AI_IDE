// Performance monitoring service for collaborative editing
// Implements threshold-based alerting for performance benchmarking

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time;

use crate::websocket::CollaborationWebSocketServer;

/// Performance metrics for collaborative editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMetrics {
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub active_users: usize,
    pub operations_per_second: f64,
    pub average_response_time_ms: f64,
    pub memory_usage_mb: f64,
    pub websocket_connections: usize,
    pub conflict_resolution_time_ms: f64,
    pub sync_operations_count: u64,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_response_time_ms: f64,
    pub max_memory_usage_mb: f64,
    pub min_operations_per_second: f64,
    pub max_conflict_resolution_time_ms: f64,
    pub max_websocket_connections: usize,
    pub alert_cooldown_seconds: u64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 100.0,           // 100ms max response time
            max_memory_usage_mb: 512.0,            // 512MB max memory
            min_operations_per_second: 10.0,       // 10 ops/sec minimum
            max_conflict_resolution_time_ms: 50.0, // 50ms max conflict resolution
            max_websocket_connections: 1000,       // 1000 max connections
            alert_cooldown_seconds: 300,           // 5 minute cooldown
        }
    }
}

/// Alert types for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlert {
    HighResponseTime {
        session_id: String,
        response_time_ms: f64,
        threshold_ms: f64,
    },
    HighMemoryUsage {
        session_id: String,
        memory_usage_mb: f64,
        threshold_mb: f64,
    },
    LowOperationsRate {
        session_id: String,
        operations_per_second: f64,
        threshold_ops: f64,
    },
    HighConflictResolutionTime {
        session_id: String,
        resolution_time_ms: f64,
        threshold_ms: f64,
    },
    ConnectionLimitExceeded {
        session_id: String,
        connection_count: usize,
        threshold_count: usize,
    },
    SystemOverload {
        session_id: String,
        metrics: CollaborationMetrics,
    },
}

/// Performance monitoring service
pub struct CollaborationPerformanceMonitor {
    metrics_cache: Cache<String, Vec<CollaborationMetrics>>,
    thresholds: PerformanceThresholds,
    alert_sender: mpsc::UnboundedSender<PerformanceAlert>,
    alert_receiver: mpsc::UnboundedReceiver<PerformanceAlert>,
    session_stats: Arc<RwLock<HashMap<String, SessionStats>>>,
    last_alert_times: Arc<RwLock<HashMap<String, Instant>>>,
}

#[derive(Debug, Clone)]
struct SessionStats {
    operation_count: u64,
    total_response_time: Duration,
    conflict_resolution_times: Vec<Duration>,
    start_time: Instant,
    last_measurement: Instant,
}

impl CollaborationPerformanceMonitor {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();

        Self {
            metrics_cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(3600)) // 1 hour TTL
                .build(),
            thresholds,
            alert_sender,
            alert_receiver,
            session_stats: Arc::new(RwLock::new(HashMap::new())),
            last_alert_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the performance monitoring service
    pub async fn start_monitoring(&self) {
        let monitor = self.clone();

        tokio::spawn(async move {
            monitor.monitoring_loop().await;
        });

        let alert_handler = self.clone();
        tokio::spawn(async move {
            alert_handler.alert_handling_loop().await;
        });
    }

    /// Record operation performance metrics
    pub async fn record_operation(
        &self,
        session_id: &str,
        response_time: Duration,
        operation_type: &str,
    ) {
        let mut stats = self.session_stats.write().await;
        let session_stat = stats
            .entry(session_id.to_string())
            .or_insert_with(|| SessionStats {
                operation_count: 0,
                total_response_time: Duration::default(),
                conflict_resolution_times: Vec::new(),
                start_time: Instant::now(),
                last_measurement: Instant::now(),
            });

        session_stat.operation_count += 1;
        session_stat.total_response_time += response_time;
        session_stat.last_measurement = Instant::now();

        // Record conflict resolution time if applicable
        if operation_type.contains("conflict") {
            session_stat.conflict_resolution_times.push(response_time);
            // Keep only last 100 measurements
            if session_stat.conflict_resolution_times.len() > 100 {
                session_stat.conflict_resolution_times.remove(0);
            }
        }

        // Check thresholds and send alerts
        self.check_thresholds(session_id, session_stat).await;
    }

    /// Record websocket connection metrics
    pub async fn record_connection_count(&self, session_id: &str, count: usize) {
        if count > self.thresholds.max_websocket_connections {
            let alert = PerformanceAlert::ConnectionLimitExceeded {
                session_id: session_id.to_string(),
                connection_count: count,
                threshold_count: self.thresholds.max_websocket_connections,
            };
            let _ = self.alert_sender.send(alert);
        }
    }

    /// Get current performance metrics for a session
    pub async fn get_metrics(&self, session_id: &str) -> Option<CollaborationMetrics> {
        let stats = self.session_stats.read().await;
        stats.get(session_id).map(|session_stat| {
            let elapsed = session_stat
                .last_measurement
                .duration_since(session_stat.start_time);
            let operations_per_second = if elapsed.as_secs_f64() > 0.0 {
                session_stat.operation_count as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            let average_response_time_ms = if session_stat.operation_count > 0 {
                session_stat.total_response_time.as_millis() as f64
                    / session_stat.operation_count as f64
            } else {
                0.0
            };

            let avg_conflict_time_ms = if !session_stat.conflict_resolution_times.is_empty() {
                let total: Duration = session_stat.conflict_resolution_times.iter().sum();
                total.as_millis() as f64 / session_stat.conflict_resolution_times.len() as f64
            } else {
                0.0
            };

            CollaborationMetrics {
                session_id: session_id.to_string(),
                timestamp: chrono::Utc::now(),
                active_users: 0, // Would be populated from websocket server
                operations_per_second,
                average_response_time_ms,
                memory_usage_mb: self.get_memory_usage(),
                websocket_connections: 0, // Would be populated from websocket server
                conflict_resolution_time_ms: avg_conflict_time_ms,
                sync_operations_count: session_stat.operation_count,
            }
        })
    }

    /// Get performance history for a session
    pub async fn get_metrics_history(&self, session_id: &str) -> Vec<CollaborationMetrics> {
        self.metrics_cache.get(session_id).await.unwrap_or_default()
    }

    /// Update performance thresholds
    pub fn update_thresholds(&mut self, new_thresholds: PerformanceThresholds) {
        self.thresholds = new_thresholds;
    }

    /// Main monitoring loop
    async fn monitoring_loop(&self) {
        let mut interval = time::interval(Duration::from_secs(60)); // Collect metrics every minute

        loop {
            interval.tick().await;

            // Collect metrics for all active sessions
            let session_ids: Vec<String> = {
                let stats = self.session_stats.read().await;
                stats.keys().cloned().collect()
            };

            for session_id in session_ids {
                if let Some(metrics) = self.get_metrics(&session_id).await {
                    // Store in cache
                    let mut history = self
                        .metrics_cache
                        .get(&session_id)
                        .await
                        .unwrap_or_default();
                    history.push(metrics.clone());

                    // Keep only last 60 measurements (1 hour of data)
                    if history.len() > 60 {
                        history.remove(0);
                    }

                    self.metrics_cache.insert(session_id, history).await;

                    // Check for system overload
                    self.check_system_overload(&metrics).await;
                }
            }
        }
    }

    /// Alert handling loop
    async fn alert_handling_loop(&self) {
        while let Some(alert) = self.alert_receiver.recv().await {
            self.handle_alert(alert).await;
        }
    }

    /// Check performance thresholds and send alerts
    async fn check_thresholds(&self, session_id: &str, stats: &SessionStats) {
        let elapsed = stats.last_measurement.duration_since(stats.start_time);
        let operations_per_second = if elapsed.as_secs_f64() > 0.0 {
            stats.operation_count as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let average_response_time_ms = if stats.operation_count > 0 {
            stats.total_response_time.as_millis() as f64 / stats.operation_count as f64
        } else {
            0.0
        };

        // Check response time threshold
        if average_response_time_ms > self.thresholds.max_response_time_ms {
            self.send_alert_with_cooldown(PerformanceAlert::HighResponseTime {
                session_id: session_id.to_string(),
                response_time_ms: average_response_time_ms,
                threshold_ms: self.thresholds.max_response_time_ms,
            })
            .await;
        }

        // Check operations per second threshold
        if operations_per_second < self.thresholds.min_operations_per_second
            && stats.operation_count > 10
        {
            self.send_alert_with_cooldown(PerformanceAlert::LowOperationsRate {
                session_id: session_id.to_string(),
                operations_per_second,
                threshold_ops: self.thresholds.min_operations_per_second,
            })
            .await;
        }

        // Check conflict resolution time
        if !stats.conflict_resolution_times.is_empty() {
            let avg_conflict_time: Duration = stats.conflict_resolution_times.iter().sum();
            let avg_conflict_time_ms =
                avg_conflict_time.as_millis() as f64 / stats.conflict_resolution_times.len() as f64;

            if avg_conflict_time_ms > self.thresholds.max_conflict_resolution_time_ms {
                self.send_alert_with_cooldown(PerformanceAlert::HighConflictResolutionTime {
                    session_id: session_id.to_string(),
                    resolution_time_ms: avg_conflict_time_ms,
                    threshold_ms: self.thresholds.max_conflict_resolution_time_ms,
                })
                .await;
            }
        }

        // Check memory usage
        let memory_usage = self.get_memory_usage();
        if memory_usage > self.thresholds.max_memory_usage_mb {
            self.send_alert_with_cooldown(PerformanceAlert::HighMemoryUsage {
                session_id: session_id.to_string(),
                memory_usage_mb: memory_usage,
                threshold_mb: self.thresholds.max_memory_usage_mb,
            })
            .await;
        }
    }

    /// Check for system overload conditions
    async fn check_system_overload(&self, metrics: &CollaborationMetrics) {
        let overload_conditions = metrics.average_response_time_ms
            > self.thresholds.max_response_time_ms * 2.0
            && metrics.memory_usage_mb > self.thresholds.max_memory_usage_mb * 0.9
            && metrics.operations_per_second < self.thresholds.min_operations_per_second * 0.5;

        if overload_conditions {
            let alert = PerformanceAlert::SystemOverload {
                session_id: metrics.session_id.clone(),
                metrics: metrics.clone(),
            };
            let _ = self.alert_sender.send(alert);
        }
    }

    /// Send alert with cooldown to prevent alert spam
    async fn send_alert_with_cooldown(&self, alert: PerformanceAlert) {
        let alert_key = match &alert {
            PerformanceAlert::HighResponseTime { session_id, .. } => {
                format!("response_time_{}", session_id)
            }
            PerformanceAlert::HighMemoryUsage { session_id, .. } => {
                format!("memory_{}", session_id)
            }
            PerformanceAlert::LowOperationsRate { session_id, .. } => {
                format!("ops_rate_{}", session_id)
            }
            PerformanceAlert::HighConflictResolutionTime { session_id, .. } => {
                format!("conflict_time_{}", session_id)
            }
            PerformanceAlert::ConnectionLimitExceeded { session_id, .. } => {
                format!("connections_{}", session_id)
            }
            PerformanceAlert::SystemOverload { session_id, .. } => {
                format!("overload_{}", session_id)
            }
        };

        let mut last_alerts = self.last_alert_times.write().await;
        let now = Instant::now();

        if let Some(last_time) = last_alerts.get(&alert_key) {
            if now.duration_since(*last_time).as_secs() < self.thresholds.alert_cooldown_seconds {
                return; // Still in cooldown period
            }
        }

        last_alerts.insert(alert_key, now);
        let _ = self.alert_sender.send(alert);
    }

    /// Handle performance alerts (log, notify, take action)
    async fn handle_alert(&self, alert: PerformanceAlert) {
        match &alert {
            PerformanceAlert::HighResponseTime {
                session_id,
                response_time_ms,
                threshold_ms,
            } => {
                log::warn!(
                    "PERFORMANCE ALERT: High response time in session {}: {:.2}ms (threshold: {:.2}ms)",
                    session_id, response_time_ms, threshold_ms
                );
                // Could trigger auto-scaling, rate limiting, etc.
            }
            PerformanceAlert::HighMemoryUsage {
                session_id,
                memory_usage_mb,
                threshold_mb,
            } => {
                log::warn!(
                    "PERFORMANCE ALERT: High memory usage in session {}: {:.2}MB (threshold: {:.2}MB)",
                    session_id, memory_usage_mb, threshold_mb
                );
                // Could trigger garbage collection, session cleanup, etc.
            }
            PerformanceAlert::LowOperationsRate {
                session_id,
                operations_per_second,
                threshold_ops,
            } => {
                log::warn!(
                    "PERFORMANCE ALERT: Low operation rate in session {}: {:.2} ops/sec (threshold: {:.2} ops/sec)",
                    session_id, operations_per_second, threshold_ops
                );
            }
            PerformanceAlert::HighConflictResolutionTime {
                session_id,
                resolution_time_ms,
                threshold_ms,
            } => {
                log::warn!(
                    "PERFORMANCE ALERT: High conflict resolution time in session {}: {:.2}ms (threshold: {:.2}ms)",
                    session_id, resolution_time_ms, threshold_ms
                );
            }
            PerformanceAlert::ConnectionLimitExceeded {
                session_id,
                connection_count,
                threshold_count,
            } => {
                log::warn!(
                    "PERFORMANCE ALERT: Connection limit exceeded in session {}: {} connections (threshold: {})",
                    session_id, connection_count, threshold_count
                );
            }
            PerformanceAlert::SystemOverload {
                session_id,
                metrics,
            } => {
                log::error!(
                    "SYSTEM OVERLOAD ALERT: Session {} experiencing critical performance degradation",
                    session_id
                );
                log::error!("Metrics: {:?}", metrics);
                // Could trigger emergency measures, session termination, etc.
            }
        }

        // Here you could integrate with external monitoring systems,
        // send notifications, trigger auto-scaling, etc.
    }

    /// Get current memory usage (simplified implementation)
    fn get_memory_usage(&self) -> f64 {
        // In a real implementation, this would use system monitoring APIs
        // For now, return a placeholder value
        128.0 // MB
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_performance_monitoring() {
        let monitor = CollaborationPerformanceMonitor::new(PerformanceThresholds::default());

        // Record some operations
        monitor
            .record_operation("session1", Duration::from_millis(50), "insert")
            .await;
        monitor
            .record_operation("session1", Duration::from_millis(75), "delete")
            .await;
        monitor
            .record_operation("session1", Duration::from_millis(30), "conflict_resolution")
            .await;

        // Get metrics
        if let Some(metrics) = monitor.get_metrics("session1").await {
            assert_eq!(metrics.session_id, "session1");
            assert!(metrics.average_response_time_ms > 0.0);
            assert!(metrics.sync_operations_count >= 3);
        } else {
            panic!("Metrics should be available");
        }

        // Test connection count alerting
        monitor.record_connection_count("session1", 1200).await; // Should trigger alert
    }

    #[test]
    fn test_performance_thresholds() {
        let thresholds = PerformanceThresholds::default();

        assert_eq!(thresholds.max_response_time_ms, 100.0);
        assert_eq!(thresholds.max_memory_usage_mb, 512.0);
        assert_eq!(thresholds.min_operations_per_second, 10.0);
        assert_eq!(thresholds.alert_cooldown_seconds, 300);
    }
}
