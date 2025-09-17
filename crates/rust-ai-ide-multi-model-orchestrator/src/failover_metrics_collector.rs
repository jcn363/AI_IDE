//! Failover Metrics Collector
//!
//! This module collects and analyzes metrics related to failover events,
//! providing insights into system reliability and performance.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::types::{ModelId, HealthEvent};
use crate::{OrchestrationError, Result};

/// Metrics collection configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// How long to keep metrics data
    pub retention_period_secs: u64,
    /// Maximum number of events to keep
    pub max_events: usize,
    /// Metrics reporting interval
    pub report_interval_secs: u64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            retention_period_secs: 86400, // 24 hours
            max_events: 10000,
            report_interval_secs: 300, // 5 minutes
        }
    }
}

/// Comprehensive failover metrics
#[derive(Debug, Clone)]
pub struct FailoverMetrics {
    /// Total failover events
    pub total_failovers: u64,
    /// Successful failovers
    pub successful_failovers: u64,
    /// Failed failovers
    pub failed_failovers: u64,
    /// Average failover time
    pub average_failover_time_ms: f64,
    /// Failover success rate (0.0 to 1.0)
    pub failover_success_rate: f64,
    /// Models by failure frequency
    pub model_failure_counts: HashMap<ModelId, u64>,
    /// Peak concurrent failovers
    pub peak_concurrent_failovers: u32,
    /// System availability percentage
    pub system_availability_percent: f64,
    /// Last updated timestamp
    pub last_updated: Instant,
}

/// Individual failover event record
#[derive(Debug, Clone)]
pub struct FailoverEventRecord {
    pub model_id: ModelId,
    pub event_type: FailoverEventType,
    pub timestamp: Instant,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum FailoverEventType {
    ModelFailure,
    FailoverInitiated,
    FailoverCompleted,
    FailoverFailed,
    RecoveryStarted,
    RecoveryCompleted,
    CircuitBreakerTripped,
}

/// Failover Metrics Collector
#[derive(Debug)]
pub struct FailoverMetricsCollector {
    config: MetricsConfig,
    metrics: Arc<RwLock<FailoverMetrics>>,
    event_history: Arc<RwLock<VecDeque<FailoverEventRecord>>>,
    current_failovers: Arc<RwLock<HashMap<ModelId, Instant>>>,
}

impl FailoverMetricsCollector {
    /// Create new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(FailoverMetrics {
                total_failovers: 0,
                successful_failovers: 0,
                failed_failovers: 0,
                average_failover_time_ms: 0.0,
                failover_success_rate: 1.0,
                model_failure_counts: HashMap::new(),
                peak_concurrent_failovers: 0,
                system_availability_percent: 100.0,
                last_updated: Instant::now(),
            })),
            event_history: Arc::new(RwLock::new(VecDeque::new())),
            current_failovers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a failover event
    pub async fn record_failover_event(&self, event: FailoverEventRecord) -> Result<()> {
        // Add to history
        {
            let mut history = self.event_history.write().await;
            history.push_back(event.clone());
            if history.len() > self.config.max_events {
                history.pop_front();
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;

            match event.event_type {
                FailoverEventType::ModelFailure => {
                    *metrics.model_failure_counts.entry(event.model_id).or_insert(0) += 1;
                }
                FailoverEventType::FailoverInitiated => {
                    metrics.total_failovers += 1;
                    let mut current = self.current_failovers.write().await;
                    current.insert(event.model_id, event.timestamp);
                    metrics.peak_concurrent_failovers = metrics.peak_concurrent_failovers.max(current.len() as u32);
                }
                FailoverEventType::FailoverCompleted if event.success => {
                    metrics.successful_failovers += 1;
                    let mut current = self.current_failovers.write().await;
                    current.remove(&event.model_id);
                }
                FailoverEventType::FailoverFailed => {
                    metrics.failed_failovers += 1;
                    let mut current = self.current_failovers.write().await;
                    current.remove(&event.model_id);
                }
                _ => {}
            }

            // Recalculate derived metrics
            self.update_derived_metrics(&mut metrics).await;

            metrics.last_updated = Instant::now();
        }

        debug!("Recorded failover event: {:?}", event.event_type);
        Ok(())
    }

    /// Record a health event (integration with health monitor)
    pub async fn record_health_event(&self, event: HealthEvent) -> Result<()> {
        let record = match event {
            HealthEvent::ModelAvailable(model_id) => {
                FailoverEventRecord {
                    model_id,
                    event_type: FailoverEventType::RecoveryCompleted,
                    timestamp: Instant::now(),
                    duration_ms: None,
                    success: true,
                    details: "Model became available".to_string(),
                }
            }
            HealthEvent::ModelUnavailable(model_id) => {
                FailoverEventRecord {
                    model_id,
                    event_type: FailoverEventType::ModelFailure,
                    timestamp: Instant::now(),
                    duration_ms: None,
                    success: false,
                    details: "Model became unavailable".to_string(),
                }
            }
            _ => return Ok(()), // Ignore other health events
        };

        self.record_failover_event(record).await
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> FailoverMetrics {
        self.metrics.read().await.clone()
    }

    /// Get recent events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<FailoverEventRecord> {
        let history = self.event_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get events for a specific model
    pub async fn get_model_events(&self, model_id: &ModelId, limit: usize) -> Vec<FailoverEventRecord> {
        let history = self.event_history.read().await;
        history
            .iter()
            .rev()
            .filter(|event| &event.model_id == model_id)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Generate metrics report
    pub async fn generate_report(&self) -> FailoverReport {
        let metrics = self.get_metrics().await;
        let recent_events = self.get_recent_events(50).await;

        let mut failure_patterns = HashMap::new();
        for event in &recent_events {
            if let FailoverEventType::ModelFailure = event.event_type {
                *failure_patterns.entry(event.model_id).or_insert(0) += 1;
            }
        }

        FailoverReport {
            metrics,
            recent_events,
            failure_patterns,
            report_generated_at: Instant::now(),
        }
    }

    /// Clean up old data
    pub async fn cleanup_old_data(&self) -> Result<()> {
        let cutoff = Instant::now() - Duration::from_secs(self.config.retention_period_secs);

        // Clean event history
        {
            let mut history = self.event_history.write().await;
            while let Some(event) = history.front() {
                if event.timestamp < cutoff {
                    history.pop_front();
                } else {
                    break;
                }
            }
        }

        // Clean metrics (reset counters periodically)
        {
            let mut metrics = self.metrics.write().await;
            if metrics.last_updated.elapsed() > Duration::from_secs(self.config.retention_period_secs) {
                // Reset counters but keep structural data
                metrics.total_failovers = 0;
                metrics.successful_failovers = 0;
                metrics.failed_failovers = 0;
                metrics.average_failover_time_ms = 0.0;
                metrics.peak_concurrent_failovers = 0;
            }
        }

        Ok(())
    }

    /// Update derived metrics
    async fn update_derived_metrics(&self, metrics: &mut FailoverMetrics) {
        // Calculate success rate
        let total_completed = metrics.successful_failovers + metrics.failed_failovers;
        if total_completed > 0 {
            metrics.failover_success_rate = metrics.successful_failovers as f64 / total_completed as f64;
        }

        // Estimate system availability (simplified)
        // In a real system, this would be calculated from uptime monitoring
        let total_failures: u64 = metrics.model_failure_counts.values().sum();
        if total_failures > 0 {
            metrics.system_availability_percent = 100.0 - (total_failures as f64 * 0.1).min(50.0);
        }
    }
}

/// Comprehensive failover report
#[derive(Debug, Clone)]
pub struct FailoverReport {
    pub metrics: FailoverMetrics,
    pub recent_events: Vec<FailoverEventRecord>,
    pub failure_patterns: HashMap<ModelId, u32>,
    pub report_generated_at: Instant,
}

impl Default for FailoverMetricsCollector {
    fn default() -> Self {
        Self::new(MetricsConfig::default())
    }
}