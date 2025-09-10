//! Model Health Monitor
//!
//! This module provides real-time health monitoring and diagnostics for the multi-model orchestration system.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::{ModelId, HealthEvent, ModelStatus, ModelMetrics};
use crate::{Result, OrchestrationError};

/// Real-time health checker for models
#[derive(Debug)]
pub struct ModelHealthMonitor {
    health_states: Arc<RwLock<HashMap<ModelId, ModelHealthState>>>,
    event_history: Arc<RwLock<Vec<HealthEvent>>>,
}

#[derive(Debug, Clone)]
pub struct ModelHealthState {
    pub status: ModelStatus,
    pub last_healthy_check: std::time::Instant,
    pub consecutive_failures: u32,
    pub metrics: ModelMetrics,
}

impl ModelHealthMonitor {
    pub async fn check_model_health(&self, model_id: &ModelId) -> Result<ModelStatus> {
        let health_states = self.health_states.read().await;
        if let Some(state) = health_states.get(model_id) {
            Ok(state.status.clone())
        } else {
            Ok(ModelStatus::Unhealthy) // Default if unknown
        }
    }

    pub async fn record_health_event(&self, event: HealthEvent) -> Result<()> {
        let mut history = self.event_history.write().await;
        history.push(event);
        // Keep history bounded
        if history.len() > 1000 {
            history.remove(0);
        }
        Ok(())
    }
}

impl Default for ModelHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelHealthMonitor {
    pub fn new() -> Self {
        Self {
            health_states: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
}