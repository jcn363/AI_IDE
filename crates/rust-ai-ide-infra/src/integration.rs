//! Integration module for connecting defragmentation system with existing memory management

use std::sync::Arc;
use crate::tracker::MemoryBlockTracker;
use crate::metrics::FragmentationMetricsCollector;
use crate::guard::PerformanceGuard;
use crate::coordinator::BackgroundDefragmentationCoordinator;
use crate::InfraResult;

/// Integration manager for connecting defragmentation with existing memory systems
#[derive(Debug)]
pub struct DefragmentationIntegrationManager {
    /// Memory block tracker
    tracker: Arc<MemoryBlockTracker>,

    /// Metrics collector
    metrics: Arc<FragmentationMetricsCollector>,

    /// Performance guard
    guard: Arc<PerformanceGuard>,

    /// Background coordinator
    coordinator: Arc<BackgroundDefragmentationCoordinator>,

    /// Integration state
    state: Arc<std::sync::RwLock<IntegrationState>>,
}

#[derive(Debug)]
struct IntegrationState {
    /// Whether integration is active
    active: bool,

    /// Connected memory pools
    connected_pools: Vec<String>,

    /// Connected cache systems
    connected_caches: Vec<String>,

    /// Connected GC coordinators
    connected_gc_coordinators: Vec<String>,

    /// Connected virtual memory interfaces
    connected_virtual_memory: Vec<String>,
}

impl DefragmentationIntegrationManager {
    /// Create a new integration manager
    pub fn new(
        tracker: Arc<MemoryBlockTracker>,
        metrics: Arc<FragmentationMetricsCollector>,
        guard: Arc<PerformanceGuard>,
        coordinator: Arc<BackgroundDefragmentationCoordinator>,
    ) -> Self {
        Self {
            tracker,
            metrics,
            guard,
            coordinator,
            state: Arc::new(std::sync::RwLock::new(IntegrationState {
                active: false,
                connected_pools: Vec::new(),
                connected_caches: Vec::new(),
                connected_gc_coordinators: Vec::new(),
                connected_virtual_memory: Vec::new(),
            })),
        }
    }

    /// Initialize integration with memory pool manager
    pub async fn integrate_memory_pool_manager(&self, pool_manager: &rust_ai_ide_lazy_loading::MemoryPoolManager) -> InfraResult<()> {
        let mut state = self.state.write().unwrap();

        // Register pool manager with tracker
        for pool_id in pool_manager.get_pool_ids().await {
            state.connected_pools.push(pool_id.clone());

            // Set up block tracking callbacks
            self.setup_pool_tracking(&pool_id, pool_manager).await?;
        }

        tracing::info!("Integrated with MemoryPoolManager: {} pools connected", state.connected_pools.len());
        Ok(())
    }

    /// Initialize integration with enhanced cache
    pub async fn integrate_enhanced_cache(&self, cache: &rust_ai_ide_cache::cache_impls::enhanced_cache::EnhancedInMemoryCache<String, serde_json::Value>) -> InfraResult<()> {
        let mut state = self.state.write().unwrap();
        state.connected_caches.push("enhanced_cache".to_string());

        // Set up cache entry tracking
        self.setup_cache_tracking(cache).await?;

        tracing::info!("Integrated with EnhancedInMemoryCache");
        Ok(())
    }

    /// Initialize integration with garbage collection coordinator
    pub async fn integrate_gc_coordinator(&self, gc_coordinator: &rust_ai_ide_advanced_memory::garbage_collection::GarbageCollectionCoordinator) -> InfraResult<()> {
        let mut state = self.state.write().unwrap();
        state.connected_gc_coordinators.push("gc_coordinator".to_string());

        // Set up GC event monitoring
        self.setup_gc_event_monitoring(gc_coordinator).await?;

        tracing::info!("Integrated with GarbageCollectionCoordinator");
        Ok(())
    }

    /// Initialize integration with virtual memory interface
    pub async fn integrate_virtual_memory(&self, vm_interface: &rust_ai_ide_advanced_memory::virtual_memory::VirtualMemoryInterface) -> InfraResult<()> {
        let mut state = self.state.write().unwrap();
        state.connected_virtual_memory.push("virtual_memory".to_string());

        // Set up virtual memory monitoring
        self.setup_virtual_memory_monitoring(vm_interface).await?;

        tracing::info!("Integrated with VirtualMemoryInterface");
        Ok(())
    }

    /// Start the integrated defragmentation system
    pub async fn start_integration(&self) -> InfraResult<()> {
        let mut state = self.state.write().unwrap();

        if state.active {
            return Ok(()); // Already active
        }

        // Start the background coordinator
        self.coordinator.start().await?;

        // Start metrics collection
        self.metrics.start_collection().await?;

        state.active = true;

        tracing::info!("Defragmentation integration started successfully");
        Ok(())
    }

    /// Stop the integrated defragmentation system
    pub async fn stop_integration(&self) -> InfraResult<()> {
        let mut state = self.state.write().unwrap();

        if !state.active {
            return Ok(()); // Already stopped
        }

        // Stop the background coordinator
        self.coordinator.stop().await?;

        state.active = false;

        tracing::info!("Defragmentation integration stopped");
        Ok(())
    }

    /// Get integration status
    pub fn get_integration_status(&self) -> IntegrationStatus {
        let state = self.state.read().unwrap();

        IntegrationStatus {
            active: state.active,
            connected_pools: state.connected_pools.len(),
            connected_caches: state.connected_caches.len(),
            connected_gc_coordinators: state.connected_gc_coordinators.len(),
            connected_virtual_memory_interfaces: state.connected_virtual_memory.len(),
            total_integrations: state.connected_pools.len()
                + state.connected_caches.len()
                + state.connected_gc_coordinators.len()
                + state.connected_virtual_memory.len(),
        }
    }

    /// Perform health check on all integrations
    pub async fn health_check(&self) -> InfraResult<HealthCheckResult> {
        let state = self.state.read().unwrap();

        let mut issues = Vec::new();

        // Check coordinator health
        let coordinator_status = self.coordinator.get_status().await;
        if !coordinator_status.running {
            issues.push("BackgroundDefragmentationCoordinator is not running".to_string());
        }

        // Check metrics health
        let current_metrics = self.metrics.get_current_metrics().await;
        if current_metrics.stats.total_memory == 0 {
            issues.push("FragmentationMetricsCollector has no memory data".to_string());
        }

        // Check guard health
        if !self.guard.is_healthy().await {
            issues.push("PerformanceGuard is unhealthy".to_string());
        }

        // Check tracker health
        let tracker_stats = self.tracker.get_fragmentation_stats().await;
        if tracker_stats.allocated_blocks == 0 && tracker_stats.free_blocks == 0 {
            issues.push("MemoryBlockTracker has no block data".to_string());
        }

        let healthy = issues.is_empty();

        Ok(HealthCheckResult {
            healthy,
            issues,
            integrations_checked: state.connected_pools.len()
                + state.connected_caches.len()
                + state.connected_gc_coordinators.len()
                + state.connected_virtual_memory.len(),
        })
    }

    /// Export integration status for monitoring
    pub async fn export_status(&self) -> serde_json::Value {
        let status = self.get_integration_status();
        let coordinator_status = self.coordinator.export_status().await;
        let guard_status = self.guard.export_status().await;

        serde_json::json!({
            "integration": {
                "active": status.active,
                "connected_pools": status.connected_pools,
                "connected_caches": status.connected_caches,
                "connected_gc_coordinators": status.connected_gc_coordinators,
                "connected_virtual_memory_interfaces": status.connected_virtual_memory_interfaces,
                "total_integrations": status.total_integrations,
            },
            "coordinator": coordinator_status,
            "performance_guard": guard_status,
        })
    }

    /// Setup tracking for a memory pool
    async fn setup_pool_tracking(&self, pool_id: &str, pool_manager: &rust_ai_ide_lazy_loading::MemoryPoolManager) -> InfraResult<()> {
        // In a real implementation, this would set up callbacks for pool events
        // For now, we'll simulate by registering some initial blocks

        let pool_stats = pool_manager.get_pool_stats(pool_id).await?;

        // Register existing blocks with the tracker
        for (size, address) in &pool_stats.block_sizes {
            let block_id = self.tracker.register_block(pool_id.to_string(), *size, Some(*address as usize)).await;
            tracing::debug!("Registered existing pool block: {} (size: {})", block_id, size);
        }

        Ok(())
    }

    /// Setup tracking for cache entries
    async fn setup_cache_tracking(&self, cache: &rust_ai_ide_cache::cache_impls::enhanced_cache::EnhancedInMemoryCache<String, serde_json::Value>) -> InfraResult<()> {
        // In a real implementation, this would set up cache event monitoring
        // For now, we'll simulate by registering cache memory usage

        let cache_stats = cache.get_stats().await?;

        // Register cache memory as a tracked block
        let cache_size = cache_stats.memory_usage as usize;
        if cache_size > 0 {
            let block_id = self.tracker.register_block("cache".to_string(), cache_size, None).await;
            tracing::debug!("Registered cache memory block: {} (size: {})", block_id, cache_size);
        }

        Ok(())
    }

    /// Setup GC event monitoring
    async fn setup_gc_event_monitoring(&self, gc_coordinator: &rust_ai_ide_advanced_memory::garbage_collection::GarbageCollectionCoordinator) -> InfraResult<()> {
        // In a real implementation, this would subscribe to GC events
        // For now, we'll just log the integration

        let gc_stats = gc_coordinator.get_stats().await?;
        tracing::debug!("GC Coordinator stats: {} collections, {} objects freed", gc_stats.collection_count, gc_stats.objects_freed);

        Ok(())
    }

    /// Setup virtual memory monitoring
    async fn setup_virtual_memory_monitoring(&self, vm_interface: &rust_ai_ide_advanced_memory::virtual_memory::VirtualMemoryInterface) -> InfraResult<()> {
        // In a real implementation, this would monitor virtual memory usage
        // For now, we'll just log the integration

        let vm_stats = vm_interface.get_stats().await?;
        tracing::debug!("Virtual Memory stats: {} mappings, {} bytes used", vm_stats.mapping_count, vm_stats.total_mapped_bytes);

        Ok(())
    }
}

/// Integration status information
#[derive(Debug, Clone)]
pub struct IntegrationStatus {
    /// Whether integration is active
    pub active: bool,

    /// Number of connected memory pools
    pub connected_pools: usize,

    /// Number of connected cache systems
    pub connected_caches: usize,

    /// Number of connected GC coordinators
    pub connected_gc_coordinators: usize,

    /// Number of connected virtual memory interfaces
    pub connected_virtual_memory_interfaces: usize,

    /// Total number of integrations
    pub total_integrations: usize,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Whether all integrations are healthy
    pub healthy: bool,

    /// List of issues found
    pub issues: Vec<String>,

    /// Number of integrations checked
    pub integrations_checked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefragmentationConfig;
    use crate::tracker::MemoryBlockTracker;
    use crate::metrics::FragmentationMetricsCollector;
    use crate::guard::PerformanceGuard;
    use crate::coordinator::BackgroundDefragmentationCoordinator;

    #[tokio::test]
    async fn test_integration_manager_creation() {
        let config = DefragmentationConfig::default();
        let tracker = Arc::new(MemoryBlockTracker::new());
        let metrics = Arc::new(FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(PerformanceGuard::new(crate::guard::GuardConfig::default()));
        let coordinator = Arc::new(BackgroundDefragmentationCoordinator::new(
            config,
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
        ));

        let manager = DefragmentationIntegrationManager::new(
            tracker,
            metrics,
            guard,
            coordinator,
        );

        let status = manager.get_integration_status();
        assert!(!status.active);
        assert_eq!(status.total_integrations, 0);
    }

    #[tokio::test]
    async fn test_health_check_initial_state() {
        let config = DefragmentationConfig::default();
        let tracker = Arc::new(MemoryBlockTracker::new());
        let metrics = Arc::new(FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(PerformanceGuard::new(crate::guard::GuardConfig::default()));
        let coordinator = Arc::new(BackgroundDefragmentationCoordinator::new(
            config,
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
        ));

        let manager = DefragmentationIntegrationManager::new(
            tracker,
            metrics,
            guard,
            coordinator,
        );

        let health = manager.health_check().await.unwrap();
        assert!(!health.healthy); // Should not be healthy when coordinator is not running
        assert!(!health.issues.is_empty());
    }
}