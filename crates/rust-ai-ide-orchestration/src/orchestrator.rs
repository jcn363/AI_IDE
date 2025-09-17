use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::Duration;

use crate::error::{OrchestrationError, OrchestrationResult};
use crate::service_registry::ServiceRegistry;
use crate::types::ServiceId;

/// Main service orchestrator that coordinates all orchestration layer components
#[derive(Debug)]
pub struct ServiceOrchestrator {
    service_registry: Arc<ServiceRegistry>,
    // Message router will be added in message_router.rs
    message_router: Arc<tokio::sync::RwLock<Option<crate::message_router::MessageRouter>>>,
    // Health monitor will be added in health_monitor.rs
    health_monitor: Arc<tokio::sync::RwLock<Option<crate::health_monitor::HealthMonitor>>>,
    // Lifecycle manager will be added in lifecycle_manager.rs
    lifecycle_manager: Arc<tokio::sync::RwLock<Option<crate::lifecycle_manager::LifecycleManager>>>,
    initialized: Arc<RwLock<bool>>,
}

impl ServiceOrchestrator {
    /// Create a new service orchestrator
    pub fn new() -> Self {
        Self {
            service_registry: Arc::new(ServiceRegistry::default()),
            message_router: Arc::new(RwLock::new(None)),
            health_monitor: Arc::new(RwLock::new(None)),
            lifecycle_manager: Arc::new(RwLock::new(None)),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the orchestrator with all components
    pub async fn initialize(&self) -> OrchestrationResult<()> {
        if *self.initialized.read().await {
            return Err(OrchestrationError::ValidationError(
                "Orchestrator is already initialized".to_string(),
            ));
        }

        info!("Initializing Service Orchestrator...");

        // Initialize message router
        let message_router =
            crate::message_router::MessageRouter::new(self.service_registry.clone());
        *self.message_router.write().await = Some(message_router);

        // Initialize health monitor
        let health_monitor =
            crate::health_monitor::HealthMonitor::new(self.service_registry.clone());
        *self.health_monitor.write().await = Some(health_monitor);

        // Initialize lifecycle manager
        let lifecycle_manager =
            crate::lifecycle_manager::LifecycleManager::new(self.service_registry.clone());
        *self.lifecycle_manager.write().await = Some(lifecycle_manager);

        *self.initialized.write().await = true;

        info!("Service Orchestrator initialized successfully");
        Ok(())
    }

    /// Shutdown the orchestrator and clean up resources
    pub async fn shutdown(&self) -> OrchestrationResult<()> {
        info!("Shutting down Service Orchestrator...");

        if let Some(ref lifecycle_manager) = *self.lifecycle_manager.read().await {
            lifecycle_manager.shutdown().await?;
        }

        if let Some(ref health_monitor) = *self.health_monitor.read().await {
            health_monitor.stop_monitoring().await?;
        }

        *self.initialized.write().await = false;

        info!("Service Orchestrator shutdown complete");
        Ok(())
    }

    /// Check if orchestrator is initialized
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }

    /// Get service registry reference
    pub fn service_registry(&self) -> Arc<ServiceRegistry> {
        self.service_registry.clone()
    }

    /// Get message router reference
    pub async fn message_router(&self) -> Option<crate::message_router::MessageRouter> {
        self.message_router.read().await.clone()
    }

    /// Get health monitor reference
    pub async fn health_monitor(&self) -> Option<crate::health_monitor::HealthMonitor> {
        self.health_monitor.read().await.clone()
    }

    /// Get lifecycle manager reference
    pub async fn lifecycle_manager(&self) -> Option<crate::lifecycle_manager::LifecycleManager> {
        self.lifecycle_manager.read().await.clone()
    }

    /// Get orchestrator status
    pub async fn get_status(&self) -> OrchestratorStatus {
        let initialized = self.is_initialized().await;
        let service_count = if initialized {
            self.service_registry.get_statistics().await.total_services
        } else {
            0
        };

        OrchestratorStatus {
            initialized,
            service_count,
        }
    }
}

/// Orchard status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrchestratorStatus {
    pub initialized: bool,
    pub service_count: usize,
}

impl Default for ServiceOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceOrchestrator {
    /// Register a service with validation
    pub async fn register_service(
        &self,
        registration: crate::types::ServiceRegistration,
    ) -> OrchestrationResult<()> {
        if !self.is_initialized().await {
            return Err(OrchestrationError::ValidationError(
                "Orchestrator is not initialized".to_string(),
            ));
        }

        self.service_registry.register_service(registration).await
    }

    /// Unregister a service
    pub async fn unregister_service(&self, service_id: &ServiceId) -> OrchestrationResult<()> {
        if !self.is_initialized().await {
            return Err(OrchestrationError::ValidationError(
                "Orchestrator is not initialized".to_string(),
            ));
        }

        self.service_registry.unregister_service(service_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        ServiceCapabilities, ServicePriority, ServiceRegistration, ServiceStatus, ServiceVersion,
    };

    #[tokio::test]
    async fn test_orchestrator_initialization() {
        let orchestrator = ServiceOrchestrator::new();

        // Check initial state
        assert!(!orchestrator.is_initialized().await);

        // Initialize
        assert!(orchestrator.initialize().await.is_ok());
        assert!(orchestrator.is_initialized().await);

        // Check status
        let status = orchestrator.get_status().await;
        assert!(status.initialized);
        assert_eq!(status.service_count, 0);

        // Shutdown
        assert!(orchestrator.shutdown().await.is_ok());
        assert!(!orchestrator.is_initialized().await);
    }

    #[tokio::test]
    async fn test_service_registration() {
        let orchestrator = ServiceOrchestrator::new();
        orchestrator.initialize().await.unwrap();

        let registration = ServiceRegistration {
            id: "test-service".to_string(),
            name: "Test Service".to_string(),
            description: "A test service for orchestrator".to_string(),
            version: ServiceVersion::new(1, 0, 0),
            status: ServiceStatus::Ready,
            capabilities: ServiceCapabilities {
                supported_operations: vec!["test_operation".to_string()],
                max_concurrent_requests: Some(10),
                rate_limits: None,
                dependencies: vec![],
                provides: vec!["test_capability".to_string()],
            },
            health_check_endpoint: None,
            priority: ServicePriority::Normal,
            tags: vec!["test".to_string()],
        };

        // Register service
        assert!(orchestrator.register_service(registration).await.is_ok());

        // Check status updated
        let status = orchestrator.get_status().await;
        assert_eq!(status.service_count, 1);

        // Cleanup
        orchestrator.shutdown().await.unwrap();
    }
}
