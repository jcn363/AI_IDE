//! Resource Manager for intelligent resource allocation and monitoring
//!
//! This module manages system resources allocated to warmup operations,
//! ensuring optimal resource utilization and preventing resource exhaustion.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use sysinfo::{System, SystemExt, CpuExt, MemoryExt};

use crate::error::{Result, WarmupError};
use crate::types::{ResourceAvailability, ResourceRequirements, WarmupConfig};

/// Resource manager for monitoring and allocating system resources
#[derive(Debug)]
pub struct ResourceManager {
    /// System information monitor
    system: Arc<RwLock<System>>,
    /// Resource allocation tracker
    allocation_tracker: Arc<RwLock<ResourceTracker>>,
    /// Configuration settings
    config: Arc<RwLock<WarmupConfig>>,
    /// Resource monitoring task
    monitor_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

/// Resource allocation tracker
#[derive(Debug, Clone)]
pub struct ResourceTracker {
    /// Memory allocated (MB)
    allocated_memory_mb: u64,
    /// CPU allocated (percentage)
    allocated_cpu_percent: f64,
    /// Network bandwidth allocated (Mbps)
    allocated_network_mbps: f64,
    /// Storage allocated (MB)
    allocated_storage_mb: u64,
    /// Last updated timestamp
    last_updated: Instant,
    /// Current system metrics
    system_metrics: SystemMetrics,
}

/// Current system resource metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Total system memory (MB)
    total_memory_mb: u64,
    /// Available memory (MB)
    available_memory_mb: u64,
    /// CPU usage percentage
    cpu_usage_percent: f64,
    /// Network bandwidth available (Mbps)
    network_available_mbps: f64,
    /// Storage available (MB)
    storage_available_mb: u64,
    /// System load average
    load_average: f64,
}

impl ResourceManager {
    /// Create new resource manager
    pub async fn new(config: WarmupConfig) -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let system_metrics = Self::collect_system_metrics(&system);

        let allocation_tracker = ResourceTracker {
            allocated_memory_mb: 0,
            allocated_cpu_percent: 0.0,
            allocated_network_mbps: 0.0,
            allocated_storage_mb: 0,
            last_updated: Instant::now(),
            system_metrics,
        };

        let manager = Self {
            system: Arc::new(RwLock::new(system)),
            allocation_tracker: Arc::new(RwLock::new(allocation_tracker)),
            config: Arc::new(RwLock::new(config)),
            monitor_task: Arc::new(RwLock::new(None)),
        };

        manager.start_monitoring().await?;
        Ok(manager)
    }

    /// Get current resource availability
    pub async fn get_available_resources(&self) -> Result<ResourceAvailability> {
        let tracker = self.allocation_tracker.read().await;
        let config = self.config.read().await;

        // Calculate available resources considering allocations and limits
        let available_memory = tracker.system_metrics.available_memory_mb
            .saturating_sub(tracker.allocated_memory_mb);

        let available_cpu = (100.0 - tracker.system_metrics.cpu_usage_percent)
            .max(0.0)
            .min(config.max_cpu_percent - tracker.allocated_cpu_percent);

        let available_network = tracker.system_metrics.network_available_mbps
            .saturating_sub(tracker.allocated_network_mbps);

        let available_storage = tracker.system_metrics.storage_available_mb
            .saturating_sub(tracker.allocated_storage_mb);

        Ok(ResourceAvailability {
            available_memory_mb: available_memory,
            available_cpu_percent: available_cpu,
            available_network_mbps: available_network,
            available_storage_mb: available_storage,
            system_load: tracker.system_metrics.load_average,
        })
    }

    /// Allocate resources for a warmup task
    pub async fn allocate_resources(&self, requirements: &ResourceRequirements) -> Result<()> {
        let mut tracker = self.allocation_tracker.write().await;
        let config = self.config.read().await;

        // Check if allocation is possible
        if !self.can_allocate(requirements).await? {
            return Err(WarmupError::ResourceExhausted {
                resource_type: "insufficient_resources".to_string(),
            });
        }

        // Allocate resources
        tracker.allocated_memory_mb += requirements.memory_mb;
        tracker.allocated_cpu_percent += requirements.cpu_percent;
        tracker.allocated_storage_mb += requirements.storage_mb;

        if let Some(network) = requirements.network_bandwidth_mbps {
            tracker.allocated_network_mbps += network;
        }

        tracker.last_updated = Instant::now();

        Ok(())
    }

    /// Deallocate resources from a completed task
    pub async fn deallocate_resources(&self, requirements: &ResourceRequirements) -> Result<()> {
        let mut tracker = self.allocation_tracker.write().await;

        tracker.allocated_memory_mb = tracker.allocated_memory_mb.saturating_sub(requirements.memory_mb);
        tracker.allocated_cpu_percent = tracker.allocated_cpu_percent - requirements.cpu_percent;
        tracker.allocated_storage_mb = tracker.allocated_storage_mb.saturating_sub(requirements.storage_mb);

        if let Some(network) = requirements.network_bandwidth_mbps {
            tracker.allocated_network_mbps = tracker.allocated_network_mbps - network;
        }

        tracker.last_updated = Instant::now();

        Ok(())
    }

    /// Check if resources can be allocated
    pub async fn can_allocate(&self, requirements: &ResourceRequirements) -> Result<bool> {
        let tracker = self.allocation_tracker.read().await;
        let config = self.config.read().await;

        // Check memory
        let available_memory = tracker.system_metrics.available_memory_mb
            .saturating_sub(tracker.allocated_memory_mb);
        if available_memory < requirements.memory_mb {
            return Ok(false);
        }

        // Check CPU
        let available_cpu = (100.0 - tracker.system_metrics.cpu_usage_percent)
            .max(0.0) - tracker.allocated_cpu_percent;
        if available_cpu < requirements.cpu_percent {
            return Ok(false);
        }

        // Check within configured limits
        if tracker.allocated_memory_mb + requirements.memory_mb > config.max_memory_mb {
            return Ok(false);
        }

        if tracker.allocated_cpu_percent + requirements.cpu_percent > config.max_cpu_percent {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get resource utilization metrics
    pub async fn get_resource_utilization(&self) -> Result<ResourceUtilization> {
        let tracker = self.allocation_tracker.read().await;

        let memory_utilization = if tracker.system_metrics.total_memory_mb > 0 {
            (tracker.allocated_memory_mb as f64 / tracker.system_metrics.total_memory_mb as f64) * 100.0
        } else {
            0.0
        };

        let cpu_utilization = tracker.allocated_cpu_percent;

        Ok(ResourceUtilization {
            memory_utilization_percent: memory_utilization,
            cpu_utilization_percent: cpu_utilization,
            network_utilization_mbps: tracker.allocated_network_mbps,
            storage_utilization_mb: tracker.allocated_storage_mb,
            total_allocated_resources: tracker.allocated_memory_mb + tracker.allocated_storage_mb,
        })
    }

    /// Update configuration
    pub async fn update_config(&self, config: WarmupConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    /// Collect current system metrics
    fn collect_system_metrics(system: &System) -> SystemMetrics {
        // Refresh system information
        let mut sys = system.clone();
        sys.refresh_all();

        let total_memory_mb = sys.total_memory() / 1024 / 1024;
        let available_memory_mb = sys.available_memory() / 1024 / 1024;

        let cpu_usage_percent = sys.cpus().iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .sum::<f64>() / sys.cpus().len() as f64;

        // Estimate network bandwidth (simplified)
        let network_available_mbps = 100.0; // Placeholder

        // Estimate storage (simplified)
        let storage_available_mb = 1024 * 10; // 10GB placeholder

        // Get load average
        let load_average = sys.load_average().one;

        SystemMetrics {
            total_memory_mb,
            available_memory_mb,
            cpu_usage_percent,
            network_available_mbps,
            storage_available_mb,
            load_average: load_average as f64,
        }
    }

    /// Start background resource monitoring
    async fn start_monitoring(&self) -> Result<()> {
        let system = Arc::clone(&self.system);
        let allocation_tracker = Arc::clone(&self.allocation_tracker);

        let handle = tokio::spawn(async move {
            loop {
                // Update system metrics
                {
                    let mut sys = system.write().await;
                    sys.refresh_all();

                    let metrics = Self::collect_system_metrics(&sys);

                    let mut tracker = allocation_tracker.write().await;
                    tracker.system_metrics = metrics;
                    tracker.last_updated = Instant::now();
                }

                // Sleep for monitoring interval
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        let mut monitor = self.monitor_task.write().await;
        *monitor = Some(handle);

        Ok(())
    }
}

/// Resource utilization metrics
#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    /// Memory utilization percentage
    pub memory_utilization_percent: f64,
    /// CPU utilization percentage
    pub cpu_utilization_percent: f64,
    /// Network utilization (Mbps)
    pub network_utilization_mbps: f64,
    /// Storage utilization (MB)
    pub storage_utilization_mb: u64,
    /// Total allocated resources (MB)
    pub total_allocated_resources: u64,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            memory_mb: 100,
            cpu_percent: 5.0,
            network_bandwidth_mbps: None,
            storage_mb: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_manager_creation() {
        let config = WarmupConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let availability = manager.get_available_resources().await.unwrap();
        assert!(availability.available_memory_mb > 0);
    }

    #[tokio::test]
    async fn test_resource_allocation() {
        let config = WarmupConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let requirements = ResourceRequirements {
            memory_mb: 50,
            cpu_percent: 2.0,
            network_bandwidth_mbps: Some(1.0),
            storage_mb: 5,
        };

        // Should be able to allocate
        assert!(manager.can_allocate(&requirements).await.unwrap());

        // Allocate resources
        manager.allocate_resources(&requirements).await.unwrap();

        // Check utilization
        let utilization = manager.get_resource_utilization().await.unwrap();
        assert!(utilization.memory_utilization_percent >= 0.0);

        // Deallocate resources
        manager.deallocate_resources(&requirements).await.unwrap();
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let mut config = WarmupConfig::default();
        config.max_memory_mb = 100;
        config.max_cpu_percent = 10.0;

        let manager = ResourceManager::new(config).await.unwrap();

        let large_requirements = ResourceRequirements {
            memory_mb: 200, // Exceeds limit
            cpu_percent: 5.0,
            network_bandwidth_mbps: None,
            storage_mb: 10,
        };

        // Should not be able to allocate
        assert!(!manager.can_allocate(&large_requirements).await.unwrap());
    }
}