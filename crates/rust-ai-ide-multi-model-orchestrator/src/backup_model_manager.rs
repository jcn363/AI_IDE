//! Backup Model Manager
//!
//! This module manages backup and standby model instances for rapid failover
//! activation during system failures.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, info, warn};

use rust_ai_ide_command_templates::spawn_background_task;

use crate::types::{ModelId, ModelInfo, ModelStatus, ModelTask};
use crate::{OrchestrationError, Result};

/// Backup model configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Maximum number of standby models to maintain
    pub max_standby_models: usize,
    /// Warm-up time for standby models (seconds)
    pub warmup_time_secs: u64,
    /// Health check interval for standby models
    pub health_check_interval_secs: u64,
    /// Maximum time to wait for model activation
    pub activation_timeout_secs: u64,
    /// Standby models per primary model
    pub standby_ratio: f64,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            max_standby_models: 3,
            warmup_time_secs: 30,
            health_check_interval_secs: 60,
            activation_timeout_secs: 10,
            standby_ratio: 0.5, // 1 standby per 2 primary models
        }
    }
}

/// Standby model instance
#[derive(Debug, Clone)]
pub struct StandbyModel {
    pub model_id: ModelId,
    pub primary_model_id: ModelId,
    pub status: StandbyStatus,
    pub last_health_check: Instant,
    pub warmup_started_at: Option<Instant>,
    pub activation_attempts: u32,
    pub capabilities: Vec<ModelTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StandbyStatus {
    Inactive,
    WarmingUp,
    Ready,
    Activating,
    Active,
    Failed,
}

/// Backup model metrics
#[derive(Debug, Clone)]
pub struct BackupMetrics {
    pub total_standby_models: usize,
    pub ready_standby_models: usize,
    pub failed_standby_models: usize,
    pub average_warmup_time_secs: f64,
    pub average_activation_time_secs: f64,
    pub standby_coverage_ratio: f64,
}

/// Backup Model Manager
#[derive(Debug)]
pub struct BackupModelManager {
    config: BackupConfig,
    standby_models: Arc<RwLock<HashMap<ModelId, StandbyModel>>>,
    primary_to_standby: Arc<RwLock<HashMap<ModelId, Vec<ModelId>>>>,
    metrics: Arc<RwLock<BackupMetrics>>,
    shutdown_tx: mpsc::Sender<()>,
    _shutdown_rx: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl BackupModelManager {
    /// Create new backup model manager
    pub fn new(config: BackupConfig) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            config,
            standby_models: Arc::new(RwLock::new(HashMap::new())),
            primary_to_standby: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(BackupMetrics {
                total_standby_models: 0,
                ready_standby_models: 0,
                failed_standby_models: 0,
                average_warmup_time_secs: 0.0,
                average_activation_time_secs: 0.0,
                standby_coverage_ratio: 0.0,
            })),
            shutdown_tx,
            _shutdown_rx: Arc::new(Mutex::new(shutdown_rx)),
        }
    }

    /// Start backup model management
    pub async fn start_management(&self) -> Result<()> {
        let standby_models = self.standby_models.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        let mut shutdown_rx = self._shutdown_rx.lock().await;

        spawn_background_task!(async move {
            let mut interval = interval(Duration::from_secs(config.health_check_interval_secs));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::manage_standby_models(&standby_models, &metrics, &config).await {
                            warn!("Standby model management failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Backup model manager shutting down");
                        break;
                    }
                }
            }
        });

        info!("Started backup model management");
        Ok(())
    }

    /// Stop backup model management
    pub async fn stop_management(&self) -> Result<()> {
        self.shutdown_tx.send(()).await
            .map_err(|_| OrchestrationError::ServiceUnavailable("Failed to send shutdown signal".to_string()))?;
        Ok(())
    }

    /// Register a primary model and create standby instances
    pub async fn register_primary_model(&self, primary_model: ModelInfo) -> Result<()> {
        let standby_count = (self.config.standby_ratio * 10.0) as usize; // Simplified calculation

        let mut standby_ids = Vec::new();
        for i in 0..standby_count.min(self.config.max_standby_models) {
            let standby_id = ModelId::new();
            let standby = StandbyModel {
                model_id: standby_id,
                primary_model_id: primary_model.id,
                status: StandbyStatus::Inactive,
                last_health_check: Instant::now(),
                warmup_started_at: None,
                activation_attempts: 0,
                capabilities: primary_model.capability.supported_tasks.clone(),
            };

            // Store standby model
            {
                let mut standbys = self.standby_models.write().await;
                standbys.insert(standby_id, standby);
            }

            standby_ids.push(standby_id);

            // Start warmup process
            self.start_warmup(standby_id).await?;
        }

        // Map primary to standbys
        {
            let mut mapping = self.primary_to_standby.write().await;
            mapping.insert(primary_model.id, standby_ids);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_standby_models += standby_count;
            self.update_metrics(&mut metrics).await;
        }

        info!("Registered primary model {} with {} standby instances", primary_model.id.0, standby_count);
        Ok(())
    }

    /// Get available standby model for a primary model
    pub async fn get_available_standby(&self, primary_model_id: &ModelId) -> Option<ModelId> {
        let mapping = self.primary_to_standby.read().await;
        let standbys = self.standby_models.read().await;

        if let Some(standby_ids) = mapping.get(primary_model_id) {
            for standby_id in standby_ids {
                if let Some(standby) = standbys.get(standby_id) {
                    if standby.status == StandbyStatus::Ready {
                        return Some(*standby_id);
                    }
                }
            }
        }

        None
    }

    /// Activate a standby model to replace a failed primary
    pub async fn activate_standby(&self, standby_id: &ModelId) -> Result<()> {
        let timeout_duration = Duration::from_secs(self.config.activation_timeout_secs);

        match timeout(timeout_duration, self.perform_activation(standby_id)).await {
            Ok(result) => result,
            Err(_) => {
                warn!("Standby activation timed out for model {}", standby_id.0);
                self.mark_standby_failed(standby_id).await?;
                Err(OrchestrationError::Timeout("Standby activation timeout".to_string()))
            }
        }
    }

    /// Deactivate a standby model (return to ready state)
    pub async fn deactivate_standby(&self, standby_id: &ModelId) -> Result<()> {
        let mut standbys = self.standby_models.write().await;

        if let Some(standby) = standbys.get_mut(standby_id) {
            standby.status = StandbyStatus::Ready;
            standby.activation_attempts += 1;
            info!("Deactivated standby model {}", standby_id.0);
        }

        Ok(())
    }

    /// Get backup metrics
    pub async fn get_metrics(&self) -> BackupMetrics {
        self.metrics.read().await.clone()
    }

    /// Get standby models for a primary
    pub async fn get_standby_models(&self, primary_model_id: &ModelId) -> Vec<StandbyModel> {
        let mapping = self.primary_to_standby.read().await;
        let standbys = self.standby_models.read().await;

        if let Some(standby_ids) = mapping.get(primary_model_id) {
            standby_ids
                .iter()
                .filter_map(|id| standbys.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Start warmup process for a standby model
    async fn start_warmup(&self, standby_id: ModelId) -> Result<()> {
        let mut standbys = self.standby_models.write().await;

        if let Some(standby) = standbys.get_mut(&standby_id) {
            standby.status = StandbyStatus::WarmingUp;
            standby.warmup_started_at = Some(Instant::now());

            // Spawn warmup task
            let standby_models = self.standby_models.clone();
            let config = self.config.clone();

            spawn_background_task!(async move {
                if let Err(e) = Self::perform_warmup(standby_id, standby_models, config).await {
                    warn!("Warmup failed for standby model: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Perform warmup process
    async fn perform_warmup(
        standby_id: ModelId,
        standby_models: Arc<RwLock<HashMap<ModelId, StandbyModel>>>,
        config: BackupConfig,
    ) -> Result<()> {
        // Simulate warmup time
        tokio::time::sleep(Duration::from_secs(config.warmup_time_secs)).await;

        // Check if warmup succeeded (simulate 90% success rate)
        let success = rand::random::<f64>() < 0.9;

        let mut standbys = standby_models.write().await;
        if let Some(standby) = standbys.get_mut(&standby_id) {
            if success {
                standby.status = StandbyStatus::Ready;
                debug!("Warmup completed for standby model {}", standby_id.0);
            } else {
                standby.status = StandbyStatus::Failed;
                warn!("Warmup failed for standby model {}", standby_id.0);
            }
        }

        Ok(())
    }

    /// Perform standby activation
    async fn perform_activation(&self, standby_id: &ModelId) -> Result<()> {
        let mut standbys = self.standby_models.write().await;

        if let Some(standby) = standbys.get_mut(standby_id) {
            standby.status = StandbyStatus::Activating;
            standby.activation_attempts += 1;

            debug!("Activating standby model {}", standby_id.0);

            // Simulate activation time
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Assume activation succeeds
            standby.status = StandbyStatus::Active;

            info!("Standby model {} activated successfully", standby_id.0);
        }

        Ok(())
    }

    /// Mark standby as failed
    async fn mark_standby_failed(&self, standby_id: &ModelId) -> Result<()> {
        let mut standbys = self.standby_models.write().await;

        if let Some(standby) = standbys.get_mut(standby_id) {
            standby.status = StandbyStatus::Failed;
            warn!("Standby model {} marked as failed", standby_id.0);
        }

        Ok(())
    }

    /// Manage standby models (health checks, cleanup)
    async fn manage_standby_models(
        standby_models: &Arc<RwLock<HashMap<ModelId, StandbyModel>>>,
        metrics: &Arc<RwLock<BackupMetrics>>,
        config: &BackupConfig,
    ) -> Result<()> {
        let mut standbys = standby_models.write().await;
        let mut failed_count = 0;
        let mut ready_count = 0;

        for (id, standby) in standbys.iter_mut() {
            match standby.status {
                StandbyStatus::Ready => {
                    ready_count += 1;
                    // Perform health check
                    standby.last_health_check = Instant::now();
                }
                StandbyStatus::Failed => {
                    failed_count += 1;
                }
                StandbyStatus::WarmingUp => {
                    // Check if warmup timed out
                    if let Some(started_at) = standby.warmup_started_at {
                        if started_at.elapsed() > Duration::from_secs(config.warmup_time_secs * 2) {
                            standby.status = StandbyStatus::Failed;
                            failed_count += 1;
                            warn!("Warmup timed out for standby model {}", id.0);
                        }
                    }
                }
                _ => {}
            }
        }

        // Update metrics
        let mut metrics_guard = metrics.write().await;
        metrics_guard.ready_standby_models = ready_count;
        metrics_guard.failed_standby_models = failed_count;
        metrics_guard.total_standby_models = standbys.len();

        if metrics_guard.total_standby_models > 0 {
            metrics_guard.standby_coverage_ratio = ready_count as f64 / metrics_guard.total_standby_models as f64;
        }

        Ok(())
    }

    /// Update derived metrics
    async fn update_metrics(&self, metrics: &mut BackupMetrics) {
        // Calculate averages (simplified - would track actual times in real implementation)
        if metrics.total_standby_models > 0 {
            metrics.average_warmup_time_secs = self.config.warmup_time_secs as f64;
            metrics.average_activation_time_secs = 2.0; // Simulated activation time
        }
    }
}

impl Default for BackupModelManager {
    fn default() -> Self {
        Self::new(BackupConfig::default())
    }
}