use serde::{Deserialize, Serialize};

/// Common result type for integration commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationCommandResult {
    pub success: bool,
    pub message: String,
    pub data:    serde_json::Value,
}

/// Status information for integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub cloud_integrations_available: bool,
    pub webhook_system_active:        bool,
    pub connector_services_count:     usize,
    pub marketplace_connected:        bool,
}

/// Cloud deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDeploymentConfig {
    pub provider:      String,
    pub resource_type: String,
    pub parameters:    serde_json::Value,
}

/// Webhook registration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRegistration {
    pub provider: String,
    pub url:      String,
    pub secret:   Option<String>,
    pub events:   Vec<String>,
}

/// Connector message configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorMessage {
    pub connector:   String,
    pub channel:     String,
    pub content:     String,
    pub attachments: Option<Vec<serde_json::Value>>,
}

/// Plugin installation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallRequest {
    pub plugin_id:       String,
    pub version:         Option<String>,
    pub marketplace_url: Option<String>,
}
