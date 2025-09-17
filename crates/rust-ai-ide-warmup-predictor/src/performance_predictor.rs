//! Performance predictor for assessing warmup impact on system performance

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use sysinfo::{System, SystemExt};

use crate::error::{Result, WarmupError};
use crate::types::{ModelId, PerformanceImpact, ResourceRequirements, WarmupConfig, WarmupSchedule};

#[derive(Debug)]
pub struct PerformancePredictor {
    system: Arc<RwLock<System>>,
    config: Arc<RwLock<WarmupConfig>>,
    performance_history: Arc<RwLock<HashMap<ModelId, Vec<f64>>>>,
}

impl PerformancePredictor {
    pub async fn new(config: WarmupConfig) -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        Ok(Self {
            system: Arc::new(RwLock::new(system)),
            config: Arc::new(RwLock::new(config)),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn assess_impact(&self, schedule: &WarmupSchedule) -> Result<PerformanceImpact> {
        let config = self.config.read().await;
        let mut system = self.system.write().await;
        system.refresh_all();

        let total_memory_mb = system.total_memory() / 1024 / 1024;
        let available_memory_mb = system.available_memory() / 1024 / 1024;
        let total_memory_impact = schedule.resource_requirements.memory_mb as f64 / total_memory_mb as f64;

        let cpu_impact = schedule.resource_requirements.cpu_percent.min(100.0);
        let memory_impact_mb = schedule.resource_requirements.memory_mb;
        let network_impact = schedule.resource_requirements.network_bandwidth_mbps.unwrap_or(0.0);
        let latency_increase = self.calculate_latency_impact(schedule).await;
        let responsiveness_impact = self.calculate_responsiveness_impact(schedule).await;

        let is_acceptable = total_memory_impact <= config.performance_impact_threshold
            && cpu_impact <= config.max_cpu_percent * 0.8
            && latency_increase <= Duration::from_millis(100);

        Ok(PerformanceImpact {
            cpu_impact_percent: cpu_impact,
            memory_impact_mb,
            network_impact_mbps: network_impact,
            latency_increase_ms: latency_increase.as_millis() as f64,
            responsiveness_impact,
            is_acceptable,
        })
    }

    async fn calculate_latency_impact(&self, schedule: &WarmupSchedule) -> Duration {
        let base_parallel_time = schedule.resource_requirements.cpu_percent / 10.0;
        Duration::from_millis((base_parallel_time * 50.0) as u64)
    }

    async fn calculate_responsiveness_impact(&self, schedule: &WarmupSchedule) -> f64 {
        let cpu_factor = schedule.resource_requirements.cpu_percent / 100.0;
        let memory_factor = schedule.resource_requirements.memory_mb as f64 / 1024.0; // GB
        (cpu_factor + memory_factor).min(1.0)
    }

    pub async fn update_config(&self, config: WarmupConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }
}