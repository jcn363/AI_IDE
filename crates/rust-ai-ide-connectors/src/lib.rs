pub mod discord;
pub mod slack;
pub mod types;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Connector service trait for third-party integrations
#[async_trait]
pub trait ServiceConnector: Send + Sync {
    /// Connect to the service
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Disconnect from the service
    async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Send a message
    async fn send_message(
        &self,
        channel: &str,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Listen for events
    async fn listen_events(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Get service status
    async fn get_status(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;

    /// Get service name
    fn get_service_name(&self) -> &str;
}

/// Connector manager for managing multiple service connections
pub struct ConnectorManager {
    connectors: RwLock<std::collections::HashMap<String, Arc<dyn ServiceConnector>>>,
}

impl ConnectorManager {
    pub fn new() -> Self {
        Self {
            connectors: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Register a connector
    pub async fn register_connector(&self, name: String, connector: Arc<dyn ServiceConnector>) {
        let mut connectors = self.connectors.write().await;
        connectors.insert(name, connector);
    }

    /// Get a connector by name
    pub async fn get_connector(&self, name: &str) -> Option<Arc<dyn ServiceConnector>> {
        let connectors = self.connectors.read().await;
        connectors.get(name).cloned()
    }

    /// List all registered connectors
    pub async fn list_connectors(&self) -> Vec<String> {
        let connectors = self.connectors.read().await;
        connectors.keys().cloned().collect()
    }

    /// Send message via specific connector
    pub async fn send_message(
        &self,
        connector_name: &str,
        channel: &str,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(connector) = self.get_connector(connector_name).await {
            connector.send_message(channel, message).await
        } else {
            Err(format!("Connector {} not found", connector_name).into())
        }
    }

    /// Initialize all connectors
    pub async fn initialize_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let connectors = self.connectors.read().await;

        for (name, connector) in connectors.iter() {
            connector.connect().await?;
            tracing::info!("Initialized connector: {}", name);
        }

        Ok(())
    }

    /// Get status of all connectors
    pub async fn get_status_all(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let connectors = self.connectors.read().await;
        let mut status_map = serde_json::Map::new();

        for (name, connector) in connectors.iter() {
            let status = connector.get_status().await?;
            status_map.insert(name.clone(), status);
        }

        Ok(status_map.into())
    }
}

/// Factory for creating service connectors
pub struct ConnectorFactory;

impl ConnectorFactory {
    /// Create Slack connector
    pub fn create_slack_connector(token: String) -> Box<dyn ServiceConnector> {
        Box::new(slack::SlackConnector::new(token))
    }

    /// Create Discord connector
    pub fn create_discord_connector(token: String) -> Box<dyn ServiceConnector> {
        Box::new(discord::DiscordConnector::new(token))
    }
}

/// Initialize the connector system
pub async fn init_connector_system() -> ConnectorManager {
    ConnectorManager::new()
}
