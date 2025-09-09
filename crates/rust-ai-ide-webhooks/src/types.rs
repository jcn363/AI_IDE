use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Webhook payload with headers and body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    pub event: String,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
    pub signature: Option<String>,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub retries: u32,
    pub timeout_seconds: u32,
}

/// Integration event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationEvent {
    Webhook(WebhookPayload),
    API(serde_json::Value),
    Custom(String, serde_json::Value),
}

/// Event handler response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandlerResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Webhook delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebhookDeliveryStatus {
    Pending,
    Delivered,
    Failed {
        reason: String,
        retry_count: u32,
        next_retry: Option<chrono::DateTime<chrono::Utc>>,
    },
    TimedOut,
}

impl WebhookDeliveryStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, WebhookDeliveryStatus::Delivered)
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, WebhookDeliveryStatus::Failed { .. } | WebhookDeliveryStatus::TimedOut)
    }
}

/// Webhook delivery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub payload_id: String,
    pub status: WebhookDeliveryStatus,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> APIResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Webhook statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookStats {
    pub total_deliveries: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub average_response_time_ms: f64,
    pub last_delivery: Option<chrono::DateTime<chrono::Utc>>,
}

/// Provider-specific webhook handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    GitHub,
    GitLab,
    Bitbucket,
    Discord,
    Slack,
    Custom { name: String, signature_header: String },
}

impl Provider {
    pub fn signature_header(&self) -> &str {
        match self {
            Provider::GitHub => "X-Hub-Signature-256",
            Provider::GitLab => "X-Gitlab-Token",
            Provider::Bitbucket => "X-Hub-Signature-256",
            Provider::Discord => "X-Signature-Ed25519",
            Provider::Slack => "X-Slack-Signature",
            Provider::Custom { signature_header, .. } => signature_header,
        }
    }
}