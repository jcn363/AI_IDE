//! Caching layer for performance optimization

use std::collections::HashMap;

use moka::future::Cache;
use tokio::sync::RwLock;

use crate::errors::*;
use crate::types::*;

#[derive(Debug)]
pub struct CacheManager {
    forecast_cache: Cache<String, MaintenanceForecast>,
    cost_cache:     Cache<String, CostEstimation>,
    impact_cache:   Cache<String, ImpactAnalysis>,
    config:         MaintenanceConfig,
}

impl CacheManager {
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            forecast_cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(std::time::Duration::from_secs(config.cache_ttl_seconds))
                .build(),
            cost_cache:     Cache::builder()
                .max_capacity(100)
                .time_to_live(std::time::Duration::from_secs(config.cache_ttl_seconds))
                .build(),
            impact_cache:   Cache::builder()
                .max_capacity(100)
                .time_to_live(std::time::Duration::from_secs(config.cache_ttl_seconds))
                .build(),
            config:         config.clone(),
        })
    }
}
