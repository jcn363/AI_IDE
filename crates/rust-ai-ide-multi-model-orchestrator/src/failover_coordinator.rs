//! Advanced Failover Coordinator
//!
//! This module implements intelligent failover decision making and execution
//! for multi-model orchestration with zero-downtime capabilities.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

use rust_ai_ide_command_templates::spawn_background_task;

use crate::health_monitor::{ModelHealthMonitor, ModelHealthState};
use crate::types::{ModelId, ModelStatus, RequestContext, LoadDecision};
use crate::load_balancer::LoadBalancer;
use crate::{OrchestrationError, Result};

/// Failover strategy configuration
#[derive(Debug, Clone)]
pub struct FailoverStrategy {
    /// Maximum time to wait for failover completion
    pub failover_timeout_secs: u64,
    /// Minimum models required for degraded operation
    pub min_models_for_operation: usize,
    /// Whether to allow graceful degradation
    pub allow_graceful_degradation: bool,
    /// Recovery strategy (immediate, gradual, scheduled)
    pub recovery_strategy: RecoveryStrategy,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Cooldown period between failover attempts
    pub failover_cooldown_secs: u64,
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Immediate,
    Gradual { step_size: usize, interval_secs: u64 },
    Scheduled { cron_expression: String },
}

impl Default for FailoverStrategy {
    fn default() -> Self {
        Self {
            failover_timeout_secs: 30,
            min_models_for_operation: 1,
            allow_graceful_degradation: true,
            recovery_strategy: RecoveryStrategy::Immediate,
            circuit_breaker_threshold: 5,
            failover_cooldown_secs: 60,
        }
    }
}

/// Failover decision context
#[derive(Debug, Clone)]
pub struct FailoverDecision {
    pub failed_model: ModelId,
    pub replacement_model: Option<ModelId>,
    pub decision_reason: String,
    pub expected_downtime: Duration,
    pub risk_assessment: FailoverRisk,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum FailoverRisk {
    Low,
    Medium,
    High,
    Critical,
}

/// Circuit breaker states
#[derive(Debug, Clone)]
pub enum CircuitBreakerState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

/// Advanced Failover Coordinator
#[derive(Debug)]
pub struct FailoverCoordinator {
    strategy: FailoverStrategy,
    health_monitor: Arc<ModelHealthMonitor>,
    load_balancer: Arc<LoadBalancer>,
    circuit_breakers: Arc<RwLock<HashMap<ModelId, CircuitBreakerState>>>,
    active_failovers: Arc<RwLock<HashMap<ModelId, FailoverDecision>>>,
    failover_history: Arc<RwLock<VecDeque<FailoverDecision>>>,
    last_failover_time: Arc<RwLock<Option<Instant>>>,
    shutdown_tx: mpsc::Sender<()>,
    _shutdown_rx: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl FailoverCoordinator {
    /// Create new failover coordinator
    pub fn new(
        strategy: FailoverStrategy,
        health_monitor: Arc<ModelHealthMonitor>,
        load_balancer: Arc<LoadBalancer>,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            strategy,
            health_monitor,
            load_balancer,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            active_failovers: Arc::new(RwLock::new(HashMap::new())),
            failover_history: Arc::new(RwLock::new(VecDeque::new())),
            last_failover_time: Arc::new(RwLock::new(None)),
            shutdown_tx,
            _shutdown_rx: Arc::new(Mutex::new(shutdown_rx)),
        }
    }

    /// Start failover coordination
    pub async fn start_coordination(&self) -> Result<()> {
        let circuit_breakers = self.circuit_breakers.clone();
        let strategy = self.strategy.clone();
        let mut shutdown_rx = self._shutdown_rx.lock().await;

        spawn_background_task!(async move {
            let mut interval = interval(Duration::from_secs(10)); // Check every 10 seconds

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::manage_circuit_breakers(&circuit_breakers, &strategy).await {
                            error!("Circuit breaker management failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Failover coordinator shutting down");
                        break;
                    }
                }
            }
        });

        info!("Started advanced failover coordination");
        Ok(())
    }

    /// Stop failover coordination
    pub async fn stop_coordination(&self) -> Result<()> {
        self.shutdown_tx.send(()).await
            .map_err(|_| OrchestrationError::ServiceUnavailable("Failed to send shutdown signal".to_string()))?;
        Ok(())
    }

    /// Handle model failure and initiate failover
    pub async fn handle_model_failure(&self, failed_model: &ModelId, failure_reason: &str) -> Result<FailoverDecision> {
        // Check cooldown period
        if let Some(last_failover) = *self.last_failover_time.read().await {
            let cooldown_remaining = Duration::from_secs(self.strategy.failover_cooldown_secs)
                .saturating_sub(last_failover.elapsed());

            if cooldown_remaining > Duration::ZERO {
                warn!("Failover cooldown active, remaining: {:?}", cooldown_remaining);
                return Err(OrchestrationError::ServiceUnavailable(
                    "Failover cooldown active".to_string()
                ));
            }
        }

        // Assess current system health
        let system_health = self.assess_system_health().await?;

        // Determine failover decision
        let decision = self.make_failover_decision(failed_model, &system_health, failure_reason).await?;

        // Record active failover
        {
            let mut active_failovers = self.active_failovers.write().await;
            active_failovers.insert(failed_model.clone(), decision.clone());
        }

        // Update last failover time
        {
            let mut last_time = self.last_failover_time.write().await;
            *last_time = Some(Instant::now());
        }

        // Record in history
        {
            let mut history = self.failover_history.write().await;
            history.push_back(decision.clone());
            if history.len() > 1000 {
                history.pop_front();
            }
        }

        // Execute failover
        self.execute_failover(&decision).await?;

        Ok(decision)
    }

    /// Get current failover status
    pub async fn get_failover_status(&self) -> Result<HashMap<ModelId, FailoverDecision>> {
        let active = self.active_failovers.read().await;
        Ok(active.clone())
    }

    /// Check if system can operate with current model availability
    pub async fn can_operate(&self) -> bool {
        let healthy_models = self.health_monitor.get_failover_candidates(&[]).await.len();
        healthy_models >= self.strategy.min_models_for_operation
    }

    /// Assess overall system health for failover decisions
    async fn assess_system_health(&self) -> Result<SystemHealthAssessment> {
        let circuit_breakers = self.circuit_breakers.read().await;
        let active_failovers = self.active_failovers.read().await;

        let open_circuits = circuit_breakers.values()
            .filter(|state| matches!(state, CircuitBreakerState::Open { .. }))
            .count();

        Ok(SystemHealthAssessment {
            available_models: self.health_monitor.get_failover_candidates(&[]).await.len(),
            active_failovers: active_failovers.len(),
            open_circuit_breakers: open_circuits,
            overall_health_score: self.calculate_health_score().await,
        })
    }

    /// Make intelligent failover decision
    async fn make_failover_decision(
        &self,
        failed_model: &ModelId,
        system_health: &SystemHealthAssessment,
        failure_reason: &str,
    ) -> Result<FailoverDecision> {
        // Get failover candidates
        let candidates = self.health_monitor.get_failover_candidates(&[failed_model.clone()]).await;

        let (replacement_model, risk_assessment) = if candidates.is_empty() {
            // No candidates available
            (None, FailoverRisk::Critical)
        } else {
            // Select best replacement based on load balancer
            let replacement = self.select_best_replacement(&candidates).await;
            let risk = self.assess_failover_risk(&replacement, system_health);
            (Some(replacement), risk)
        };

        let expected_downtime = self.calculate_expected_downtime(&replacement_model, system_health);

        let decision_reason = format!(
            "Model {} failed ({}). System health: {} available models, {} active failovers. Selected replacement: {:?}",
            failed_model.0,
            failure_reason,
            system_health.available_models,
            system_health.active_failovers,
            replacement_model.as_ref().map(|id| id.0.clone())
        );

        Ok(FailoverDecision {
            failed_model: failed_model.clone(),
            replacement_model,
            decision_reason,
            expected_downtime,
            risk_assessment,
            timestamp: Instant::now(),
        })
    }

    /// Execute the failover operation
    async fn execute_failover(&self, decision: &FailoverDecision) -> Result<()> {
        let timeout_duration = Duration::from_secs(self.strategy.failover_timeout_secs);

        match timeout(timeout_duration, async {
            if let Some(replacement) = &decision.replacement_model {
                // Notify load balancer of the change
                self.load_balancer.handle_model_failover(&decision.failed_model, replacement).await?;

                // Update circuit breaker for failed model
                self.trip_circuit_breaker(&decision.failed_model).await?;

                info!("Failover completed: {} -> {}", decision.failed_model.0, replacement.0);
                Ok(())
            } else {
                // No replacement available - graceful degradation
                if self.strategy.allow_graceful_degradation {
                    warn!("Entering graceful degradation mode - no replacement available for {}", decision.failed_model.0);
                    Ok(())
                } else {
                    Err(OrchestrationError::ServiceUnavailable("No replacement model available".to_string()))
                }
            }
        }).await {
            Ok(result) => result,
            Err(_) => {
                error!("Failover timed out for model {}", decision.failed_model.0);
                Err(OrchestrationError::ServiceUnavailable("Failover timeout".to_string()))
            }
        }
    }

    /// Select best replacement model from candidates
    async fn select_best_replacement(&self, candidates: &[ModelId]) -> ModelId {
        // Use load balancer to select best model based on current load
        // For simplicity, return first candidate - in real implementation,
        // this would consult the load balancer for optimal selection
        candidates[0].clone()
    }

    /// Assess risk level of a failover operation
    fn assess_failover_risk(&self, replacement: &ModelId, system_health: &SystemHealthAssessment) -> FailoverRisk {
        if system_health.available_models < self.strategy.min_models_for_operation {
            FailoverRisk::Critical
        } else if system_health.active_failovers > 2 {
            FailoverRisk::High
        } else if system_health.open_circuit_breakers > 0 {
            FailoverRisk::Medium
        } else {
            FailoverRisk::Low
        }
    }

    /// Calculate expected downtime for failover
    fn calculate_expected_downtime(&self, replacement: &Option<ModelId>, system_health: &SystemHealthAssessment) -> Duration {
        match replacement {
            Some(_) => Duration::from_millis(500), // Typical failover time
            None => {
                if system_health.available_models >= self.strategy.min_models_for_operation {
                    Duration::ZERO // Graceful degradation
                } else {
                    Duration::from_secs(30) // Service interruption
                }
            }
        }
    }

    /// Calculate overall system health score
    async fn calculate_health_score(&self) -> f64 {
        let healthy_count = self.health_monitor.get_failover_candidates(&[]).await.len() as f64;
        let total_models = 10.0; // Placeholder - would get from registry
        (healthy_count / total_models).min(1.0)
    }

    /// Trip circuit breaker for a model
    async fn trip_circuit_breaker(&self, model_id: &ModelId) -> Result<()> {
        let mut circuit_breakers = self.circuit_breakers.write().await;

        let entry = circuit_breakers.entry(model_id.clone()).or_insert(CircuitBreakerState::Closed);

        match entry {
            CircuitBreakerState::Closed => {
                *entry = CircuitBreakerState::Open { opened_at: Instant::now() };
                warn!("Circuit breaker tripped for model {}", model_id.0);
            }
            CircuitBreakerState::HalfOpen => {
                *entry = CircuitBreakerState::Open { opened_at: Instant::now() };
                warn!("Circuit breaker re-tripped for model {} (was half-open)", model_id.0);
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, update timestamp
                *entry = CircuitBreakerState::Open { opened_at: Instant::now() };
            }
        }

        Ok(())
    }

    /// Manage circuit breaker states (called periodically)
    async fn manage_circuit_breakers(
        circuit_breakers: &Arc<RwLock<HashMap<ModelId, CircuitBreakerState>>>,
        strategy: &FailoverStrategy,
    ) -> Result<()> {
        let mut breakers = circuit_breakers.write().await;
        let mut to_update = Vec::new();

        for (model_id, state) in breakers.iter() {
            match state {
                CircuitBreakerState::Open { opened_at } => {
                    // Check if timeout has passed for half-open attempt
                    if opened_at.elapsed() > Duration::from_secs(strategy.failover_cooldown_secs) {
                        to_update.push((model_id.clone(), CircuitBreakerState::HalfOpen));
                    }
                }
                CircuitBreakerState::HalfOpen => {
                    // In real implementation, would test with a canary request
                    // For now, assume success and close
                    to_update.push((model_id.clone(), CircuitBreakerState::Closed));
                }
                CircuitBreakerState::Closed => {
                    // Nothing to do
                }
            }
        }

        for (model_id, new_state) in to_update {
            breakers.insert(model_id.clone(), new_state);
        }

        Ok(())
    }

    /// Get failover history
    pub async fn get_failover_history(&self, limit: usize) -> Vec<FailoverDecision> {
        let history = self.failover_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
}

/// System health assessment for decision making
#[derive(Debug, Clone)]
struct SystemHealthAssessment {
    available_models: usize,
    active_failovers: usize,
    open_circuit_breakers: usize,
    overall_health_score: f64,
}