use std::sync::Arc;
use tokio::sync::mpsc;

use crate::error::{OrchestrationError, OrchestrationResult};
use crate::service_registry::ServiceRegistry;
use crate::types::{MessageType, ServiceId, ServiceMessage};

/// Message router for inter-service communication
#[derive(Debug, Clone)]
pub struct MessageRouter {
    service_registry: Arc<ServiceRegistry>,
    message_sender: mpsc::Sender<ServiceMessage>,
    message_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<ServiceMessage>>>,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(service_registry: Arc<ServiceRegistry>) -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            service_registry,
            message_sender: tx,
            message_receiver: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }

    /// Send a message to a service
    pub async fn send_message(&self, message: ServiceMessage) -> OrchestrationResult<()> {
        tracing::debug!("Routing message: {}", message.message_id);
        self.message_sender
            .send(message)
            .await
            .map_err(|_| OrchestrationError::MessageRoutingError("Channel is full".to_string()))?;
        Ok(())
    }

    /// Receive the next available message
    pub async fn receive_message(&self) -> ServiceMessage {
        let mut receiver = self.message_receiver.lock().await;
        receiver
            .recv()
            .await
            .expect("Message router channel closed")
    }

    /// Create a request-response message pair
    pub fn create_request_response(
        &self,
        source: ServiceId,
        target: Option<ServiceId>,
        command: String,
        payload: serde_json::Value,
    ) -> ServiceMessage {
        let message_id = format!("msg_{}_{}", source, chrono::Utc::now().timestamp_millis());
        let correlation_id = Some(uuid::Uuid::new_v4().to_string());

        ServiceMessage {
            message_id,
            message_type: MessageType::Request,
            source_service: source,
            target_service: target,
            command,
            payload,
            timestamp: chrono::Utc::now(),
            correlation_id,
            priority: crate::types::ServicePriority::Normal,
            immediate: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let registry = Arc::new(ServiceRegistry::default());
        let router = MessageRouter::new(registry);

        let message = router.create_request_response(
            "test-service".to_string(),
            None,
            "test_command".to_string(),
            serde_json::json!({"test": "data"}),
        );

        assert_eq!(message.source_service, "test-service");
        assert_eq!(message.command, "test_command");
    }
}
