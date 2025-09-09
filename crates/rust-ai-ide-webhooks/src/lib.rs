pub mod handlers;
pub mod middleware;
pub mod server;
pub mod types;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::net::SocketAddr;
use crate::types::{WebhookConfig, WebhookPayload, IntegrationEvent};
use crate::server::WebhookServer;

/// Webhook provider trait for handling different platform integrations
#[async_trait]
pub trait WebhookProvider: Send + Sync {
    async fn validate_signature(&self, payload: &[u8], signature: &str) -> bool;
    async fn process_payload(&self, payload: WebhookPayload) -> Result<(), Box<dyn std::error::Error>>;
    fn get_event_type(&self) -> String;
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookProviderConfig {
    pub provider: String,
    pub secret: String,
    pub enabled_events: Vec<String>,
}

/// Webhook registry managing all registered webhooks
pub struct WebhookRegistry {
    providers: RwLock<HashMap<String, Arc<dyn WebhookProvider>>>,
    server: WebhookServer,
    configs: RwLock<HashMap<String, WebhookProviderConfig>>,
}

impl WebhookRegistry {
    /// Create a new webhook registry
    pub async fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let server = WebhookServer::new(port).await?;

        Ok(Self {
            providers: RwLock::new(HashMap::new()),
            server,
            configs: RwLock::new(HashMap::new()),
        })
    }

    /// Register a webhook provider
    pub async fn register_provider(&self, name: String, provider: Arc<dyn WebhookProvider>) {
        let mut providers = self.providers.write().await;
        providers.insert(name.clone(), provider.clone());

        self.server.register_handler(name, provider).await;
    }

    /// Configure a webhook provider
    pub async fn configure_provider(&self, name: &str, config: WebhookProviderConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(name.to_string(), config);
    }

    /// Start the webhook server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.server.start().await
    }

    /// Stop the webhook server
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.server.stop().await
    }

    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.keys().cloned().collect()
    }

    /// Test webhook payload
    pub async fn test_webhook(&self, provider: &str, payload: serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
        let providers = self.providers.read().await;
        if let Some(provider_instance) = providers.get(provider) {
            let webhook_payload = WebhookPayload {
                id: uuid::Uuid::new_v4().to_string(),
                event: "test".to_string(),
                payload,
                headers: HashMap::new(),
                signature: None,
            };

            provider_instance.process_payload(webhook_payload).await?;
            Ok("Test webhook processed successfully".to_string())
        } else {
            Err(format!("Provider {} not found", provider).into())
        }
    }
}

/// API Integration framework for external services
pub struct APIIntegrationManager {
    integrations: RwLock<HashMap<String, Arc<dyn APIIntegration>>>,
}

#[async_trait]
pub trait APIIntegration: Send + Sync {
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_status(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
    async fn execute_action(&self, action: &str, params: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl APIIntegrationManager {
    pub fn new() -> Self {
        Self {
            integrations: RwLock::new(HashMap::new()),
        }
    }

    /// Register an API integration
    pub async fn register_integration(&self, name: String, integration: Arc<dyn APIIntegration>) {
        let mut integrations = self.integrations.write().await;
        integrations.insert(name, integration);
    }

    /// Initialize all registered integrations
    pub async fn initialize_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let integrations = self.integrations.read().await;

        for (name, integration) in integrations.iter() {
            integration.initialize().await?;
            tracing::info!("Initialized API integration: {}", name);
        }

        Ok(())
    }

    /// Execute action on a specific integration
    pub async fn execute_action(&self, integration_name: &str, action: &str, params: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let integrations = self.integrations.read().await;

        if let Some(integration) = integrations.get(integration_name) {
            integration.execute_action(action, params).await
        } else {
            Err(format!("Integration {} not found", integration_name).into())
        }
    }

    /// Get status of all integrations
    pub async fn get_status_all(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let integrations = self.integrations.read().await;
        let mut status_map = serde_json::Map::new();

        for (name, integration) in integrations.iter() {
            let status = integration.get_status().await?;
            status_map.insert(name.clone(), status);
        }

        Ok(status_map.into())
    }
}

/// Initialize the complete webhook and API integration system
pub async fn init_webhook_system(port: u16) -> Result<(WebhookRegistry, APIIntegrationManager), Box<dyn std::error::Error>> {
    // Initialize webhook registry
    let webhook_registry = WebhookRegistry::new(port).await?;

    // Initialize API integration manager
    let api_manager = APIIntegrationManager::new();

    // TODO: Register default providers (GitHub, GitLab, etc.)
    // Can be extended based on user configuration

    Ok((webhook_registry, api_manager))
}