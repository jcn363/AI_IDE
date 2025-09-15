use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

use crate::error::{OrchestrationError, OrchestrationResult};
use crate::types::{ServiceId, ServiceRegistration, ServiceStatus, ServiceVersion};

/// Service registry that manages service discovery and registration
#[derive(Debug)]
pub struct ServiceRegistry {
    services:                 Arc<RwLock<HashMap<ServiceId, ServiceRegistration>>>,
    discovery_event_sender:   tokio::sync::mpsc::Sender<DiscoveryEvent>,
    discovery_event_receiver: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<DiscoveryEvent>>>,
    service_timeout:          Duration,
    max_services:             usize,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(max_services: usize, service_timeout: Duration) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            discovery_event_sender: tx,
            discovery_event_receiver: Arc::new(tokio::sync::Mutex::new(rx)),
            service_timeout,
            max_services,
        }
    }

    /// Register a new service
    pub async fn register_service(&self, registration: ServiceRegistration) -> OrchestrationResult<()> {
        let mut services = self.services.write().await;

        // Check capacity
        if services.len() >= self.max_services {
            return Err(OrchestrationError::ResourceExhaustion(
                "Maximum number of services reached".to_string(),
            ));
        }

        // Check if service already exists
        if services.contains_key(&registration.id) {
            return Err(OrchestrationError::ValidationError(format!(
                "Service {} is already registered",
                registration.id
            )));
        }

        // Clone for the event
        let service_id = registration.id.clone();
        services.insert(registration.id.clone(), registration);

        // Notify discovery
        let event = DiscoveryEvent::ServiceRegistered(service_id);
        if let Err(e) = self.discovery_event_sender.send(event).await {
            warn!("Failed to send discovery event: {}", e);
        }

        Ok(())
    }

    /// Unregister a service
    pub async fn unregister_service(&self, service_id: &ServiceId) -> OrchestrationResult<()> {
        let mut services = self.services.write().await;

        if services.remove(service_id).is_none() {
            return Err(OrchestrationError::UnknownService(format!(
                "Service {} not found",
                service_id
            )));
        }

        // Notify discovery
        let event = DiscoveryEvent::ServiceUnregistered(service_id.clone());
        if let Err(e) = self.discovery_event_sender.send(event).await {
            warn!("Failed to send discovery event: {}", e);
        }

        Ok(())
    }

    /// Update service status
    pub async fn update_service_status(
        &self,
        service_id: &ServiceId,
        status: ServiceStatus,
    ) -> OrchestrationResult<()> {
        let mut services = self.services.write().await;

        let service = services
            .get_mut(service_id)
            .ok_or_else(|| OrchestrationError::UnknownService(format!("Service {} not found", service_id)))?;

        service.status = status.clone();

        // Notify discovery if status changed significantly
        let event = DiscoveryEvent::ServiceStatusChanged(service_id.clone(), status);
        if let Err(e) = self.discovery_event_sender.send(event).await {
            warn!("Failed to send discovery event: {}", e);
        }

        Ok(())
    }

    /// Get service information by ID
    pub async fn get_service(&self, service_id: &ServiceId) -> OrchestrationResult<ServiceRegistration> {
        let services = self.services.read().await;

        services
            .get(service_id)
            .cloned()
            .ok_or_else(|| OrchestrationError::UnknownService(format!("Service {} not found", service_id)))
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<ServiceRegistration> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// List services by status
    pub async fn list_services_by_status(&self, status: &ServiceStatus) -> Vec<ServiceRegistration> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|service| &service.status == status)
            .map(|service| service.clone())
            .collect()
    }

    /// Discover services by capability
    pub async fn discover_services_by_capability(&self, capability: &str) -> Vec<ServiceRegistration> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|service| {
                service
                    .capabilities
                    .provides
                    .contains(&capability.to_string())
                    || service
                        .capabilities
                        .supported_operations
                        .contains(&capability.to_string())
            })
            .map(|service| service.clone())
            .collect()
    }

    /// Check if a service is available
    pub async fn is_service_available(&self, service_id: &ServiceId) -> bool {
        let services = self.services.read().await;
        services
            .get(service_id)
            .map(|service| service.status.is_available())
            .unwrap_or(false)
    }

    /// Get the next discovery event
    pub async fn next_discovery_event(&self) -> Option<DiscoveryEvent> {
        let mut receiver = self.discovery_event_receiver.lock().await;
        tokio::time::timeout(Duration::from_millis(100), receiver.recv())
            .await
            .ok()
            .flatten()
    }

    /// Clean up expired services
    pub async fn cleanup_expired_services(&self) -> OrchestrationResult<()> {
        // Implementation would check service health and remove failed services
        // For now, this is a placeholder
        Ok(())
    }

    /// Get statistics about the registry
    pub async fn get_statistics(&self) -> RegistryStatistics {
        let services = self.services.read().await;
        let total_services = services.len();
        let available_services = services
            .values()
            .filter(|s| s.status.is_available())
            .count();
        let failed_services = services
            .values()
            .filter(|s| s.status == ServiceStatus::Failed)
            .count();

        RegistryStatistics {
            total_services,
            available_services,
            failed_services,
        }
    }
}

/// Discovery events
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    ServiceRegistered(ServiceId),
    ServiceUnregistered(ServiceId),
    ServiceStatusChanged(ServiceId, ServiceStatus),
}

/// Registry statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryStatistics {
    pub total_services:     usize,
    pub available_services: usize,
    pub failed_services:    usize,
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(300)) // 5 minutes timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ServiceCapabilities, ServicePriority};

    #[tokio::test]
    async fn test_service_registration() {
        let registry = ServiceRegistry::default();

        let service_id = "test-service".to_string();
        let registration = ServiceRegistration {
            id:                    service_id.clone(),
            name:                  "Test Service".to_string(),
            description:           "A test service".to_string(),
            version:               ServiceVersion::new(1, 0, 0),
            status:                ServiceStatus::Ready,
            capabilities:          ServiceCapabilities {
                supported_operations:    vec!["test".to_string()],
                max_concurrent_requests: Some(10),
                rate_limits:             None,
                dependencies:            vec![],
                provides:                vec!["test".to_string()],
            },
            health_check_endpoint: None,
            priority:              ServicePriority::Normal,
            tags:                  vec![],
        };

        // Register service
        assert!(registry.register_service(registration).await.is_ok());

        // Get service
        let retrieved = registry.get_service(&service_id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test Service");

        // Check availability
        assert!(registry.is_service_available(&service_id).await);

        // Update status
        assert!(registry
            .update_service_status(&service_id, ServiceStatus::Busy)
            .await
            .is_ok());

        // Unregister service
        assert!(registry.unregister_service(&service_id).await.is_ok());
    }
}
