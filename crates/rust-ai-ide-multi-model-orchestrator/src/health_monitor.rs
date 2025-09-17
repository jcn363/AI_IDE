//! Advanced Model Health Monitor
//!
//! This module provides real-time health monitoring and diagnostics for the multi-model
//! orchestration system with advanced failover awareness.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

use rust_ai_ide_command_templates::spawn_background_task;

use crate::types::{HealthEvent, ModelId, ModelMetrics, ModelStatus};
use crate::{OrchestrationError, Result};

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    pub health_check_interval_secs: u64,
    pub failure_threshold: u32,
    pub recovery_attempts: u32,
    pub timeout_secs: u64,
    pub event_history_size: usize,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            health_check_interval_secs: 30,
            failure_threshold: 3,
            recovery_attempts: 5,
            timeout_secs: 10,
            event_history_size: 1000,
        }
    }
}

/// Real-time health checker for models with failover awareness
#[derive(Debug)]
pub struct ModelHealthMonitor {
    config: HealthMonitorConfig,
    health_states: Arc<RwLock<HashMap<ModelId, ModelHealthState>>>,
    event_history: Arc<RwLock<VecDeque<HealthEvent>>>,
    shutdown_tx: mpsc::Sender<()>,
    _shutdown_rx: Arc<Mutex<mpsc::Receiver<()>>>,
}

#[derive(Debug, Clone)]
pub struct ModelHealthState {
    pub status: ModelStatus,
    pub last_health_check: Instant,
    pub consecutive_failures: u32,
    pub recovery_attempts: u32,
    pub metrics: ModelMetrics,
    pub failure_pattern: FailurePattern,
    pub last_failure_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub recent_failures: VecDeque<Instant>,
    pub failure_rate_per_hour: f64,
    pub is_cascading_failure: bool,
    pub correlated_failures: Vec<ModelId>,
}

impl ModelHealthMonitor {
    /// Create new health monitor with configuration
    pub fn new(config: HealthMonitorConfig) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            config,
            health_states: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(VecDeque::new())),
            shutdown_tx,
            _shutdown_rx: Arc::new(Mutex::new(shutdown_rx)),
        }
    }

    /// Start real-time health monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let health_states = self.health_states.clone();
        let event_history = self.event_history.clone();
        let event_bus = self.event_bus.clone();
        let config = self.config.clone();
        let mut shutdown_rx = self._shutdown_rx.lock().await;

        spawn_background_task!(async move {
            let mut interval = interval(Duration::from_secs(config.health_check_interval_secs));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::perform_health_checks(&health_states, &event_history, &config).await {
                            error!("Health check cycle failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Health monitor shutting down");
                        break;
                    }
                }
            }
        });

        info!("Started real-time model health monitoring");
        Ok(())
    }

    /// Stop health monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        self.shutdown_tx.send(()).await
            .map_err(|_| OrchestrationError::HealthMonitoringError("Failed to send shutdown signal".to_string()))?;
        Ok(())
    }

    /// Register a model for health monitoring
    pub async fn register_model(&self, model_id: ModelId, initial_metrics: ModelMetrics) -> Result<()> {
        let mut states = self.health_states.write().await;
        states.insert(model_id.clone(), ModelHealthState {
            status: ModelStatus::Available,
            last_health_check: Instant::now(),
            consecutive_failures: 0,
            recovery_attempts: 0,
            metrics: initial_metrics,
            failure_pattern: FailurePattern {
                recent_failures: VecDeque::new(),
                failure_rate_per_hour: 0.0,
                is_cascading_failure: false,
                correlated_failures: Vec::new(),
            },
            last_failure_time: None,
        });

        self.record_health_event(HealthEvent::ModelAvailable(model_id)).await?;
        Ok(())
    }

    /// Get current health status for a model
    pub async fn get_health_status(&self, model_id: &ModelId) -> Result<ModelHealthState> {
        let states = self.health_states.read().await;
        states.get(model_id).cloned()
            .ok_or_else(|| OrchestrationError::HealthMonitoringError(format!("Model {} not registered for health monitoring", model_id.0)))
    }

    /// Check if model is healthy for failover decisions
    pub async fn is_model_healthy(&self, model_id: &ModelId) -> bool {
        match self.get_health_status(model_id).await {
            Ok(state) => matches!(state.status, ModelStatus::Available) && !state.failure_pattern.is_cascading_failure,
            Err(_) => false,
        }
    }

    /// Get failover candidates based on health patterns
    pub async fn get_failover_candidates(&self, exclude_models: &[ModelId]) -> Vec<ModelId> {
        let states = self.health_states.read().await;
        states.iter()
            .filter(|(id, state)| {
                !exclude_models.contains(id) &&
                matches!(state.status, ModelStatus::Available) &&
                !state.failure_pattern.is_cascading_failure &&
                state.consecutive_failures == 0
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Record a health check failure
    pub async fn record_failure(&self, model_id: &ModelId, error: &str) -> Result<()> {
        let mut states = self.health_states.write().await;

        if let Some(state) = states.get_mut(model_id) {
            state.consecutive_failures += 1;
            state.last_failure_time = Some(Instant::now());
            state.failure_pattern.recent_failures.push_back(Instant::now());

            // Keep only recent failures (last hour)
            let one_hour_ago = Instant::now() - Duration::from_secs(3600);
            while let Some(failure_time) = state.failure_pattern.recent_failures.front() {
                if *failure_time < one_hour_ago {
                    state.failure_pattern.recent_failures.pop_front();
                } else {
                    break;
                }
            }

            // Update failure rate
            state.failure_pattern.failure_rate_per_hour = state.failure_pattern.recent_failures.len() as f64;

            // Check for cascading failure pattern
            state.failure_pattern.is_cascading_failure = state.consecutive_failures >= self.config.failure_threshold;

            if state.consecutive_failures >= self.config.failure_threshold {
                state.status = ModelStatus::Unhealthy;
                self.record_health_event(HealthEvent::ModelUnavailable(model_id.clone())).await?;
                warn!("Model {} marked as unhealthy after {} consecutive failures", model_id.0, state.consecutive_failures);
            }

            self.record_health_event(HealthEvent::PerformanceDegraded {
                model_id: model_id.clone(),
                metric_type: "health_check".to_string(),
                current_value: 0.0,
                threshold: 1.0,
            }).await?;
        }

        Ok(())
    }

    /// Record a successful health check
    pub async fn record_success(&self, model_id: &ModelId, metrics: ModelMetrics) -> Result<()> {
        let mut states = self.health_states.write().await;

        if let Some(state) = states.get_mut(model_id) {
            if state.consecutive_failures > 0 {
                state.consecutive_failures = 0;
                state.recovery_attempts = 0;
                state.status = ModelStatus::Available;
                self.record_health_event(HealthEvent::ModelRecovered(model_id.clone())).await?;
                info!("Model {} recovered and marked as available", model_id.0);
            }

            state.last_health_check = Instant::now();
            state.metrics = metrics;
            state.failure_pattern.failure_rate_per_hour *= 0.9; // Decay failure rate
        }

        Ok(())
    }

    /// Perform health checks for all registered models
    async fn perform_health_checks(
        health_states: &Arc<RwLock<HashMap<ModelId, ModelHealthState>>>,
        event_history: &Arc<RwLock<VecDeque<HealthEvent>>>,
        config: &HealthMonitorConfig,
    ) -> Result<()> {
        let model_ids: Vec<ModelId> = {
            let states = health_states.read().await;
            states.keys().cloned().collect()
        };

        for model_id in model_ids {
            let health_check_result = timeout(
                Duration::from_secs(config.timeout_secs),
                Self::perform_single_health_check(&model_id)
            ).await;

            match health_check_result {
                Ok(Ok(metrics)) => {
                    // Health check passed
                    let mut states = health_states.write().await;
                    if let Some(state) = states.get_mut(&model_id) {
                        state.last_health_check = Instant::now();
                        if state.consecutive_failures > 0 {
                            state.consecutive_failures = 0;
                            state.recovery_attempts = 0;
                            state.status = ModelStatus::Available;
                            Self::record_event_async(event_history, event_bus, HealthEvent::ModelRecovered(model_id.clone())).await;
                        }
                        state.metrics = metrics;
                    }
                }
                _ => {
                    // Health check failed
                    let mut states = health_states.write().await;
                    if let Some(state) = states.get_mut(&model_id) {
                        state.consecutive_failures += 1;
                        state.last_failure_time = Some(Instant::now());

                        if state.consecutive_failures >= config.failure_threshold {
                            state.status = ModelStatus::Unhealthy;
                            let mut history = event_history.write().await;
                            history.push_back(HealthEvent::ModelUnavailable(model_id.clone()));
                            if history.len() > config.event_history_size {
                                history.pop_front();
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Perform a single health check (placeholder - would integrate with LSP service)
    async fn perform_single_health_check(_model_id: &ModelId) -> Result<ModelMetrics> {
        // Placeholder: In real implementation, this would:
        // 1. Query LSP service for model health
        // 2. Check memory usage, response times, etc.
        // 3. Return comprehensive metrics

        // Simulate health check with some variability
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(ModelMetrics::new(_model_id.clone()))
    }


    /// Record health event (public interface)
    pub async fn record_health_event(&self, event: HealthEvent) -> Result<()> {
        Self::record_event_async(&self.event_history, &self.event_bus, event).await;
        Ok(())
    }

    /// Get recent health events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<HealthEvent> {
        let history = self.event_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for ModelHealthMonitor {
    fn default() -> Self {
        Self::new(HealthMonitorConfig::default())
    }
}
