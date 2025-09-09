use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::types::{Message, Channel};
use crate::ServiceConnector;

/// Slack API endpoints
const SLACK_API_BASE: &str = "https://slack.com/api";

/// Slack connector configuration
#[derive(Debug, Clone)]
pub struct SlackConnector {
    client: Client,
    token: String,
    is_connected: std::sync::atomic::AtomicBool,
}

impl SlackConnector {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
            is_connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl ServiceConnector for SlackConnector {
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Test authentication with a simple API call
        let test_result = self.test_connection().await?;
        if test_result {
            self.is_connected.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        } else {
            Err("Failed to authenticate with Slack API".into())
        }
    }

    async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_connected.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn send_message(&self, channel: &str, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        if !self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("Not connected to Slack".into());
        }

        let payload = serde_json::json!({
            "channel": channel,
            "text": message
        });

        let response = self.client
            .post(&format!("{}/chat.postMessage", SLACK_API_BASE))
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Slack API error: {}", error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;
        let timestamp = response_json["ts"].as_str().unwrap_or("unknown");

        Ok(timestamp.to_string())
    }

    async fn listen_events(&self) -> Result<(), Box<dyn std::error::Error>> {
        // For simplicity, we'll poll for events
        // In a production implementation, this would use WebSockets or webhooks

        tracing::info!("Starting Slack event listener");
        Ok(())
    }

    async fn get_status(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let status = if self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            "connected"
        } else {
            "disconnected"
        };

        Ok(serde_json::json!({
            "service": "slack",
            "status": status,
            "token_configured": !self.token.is_empty()
        }))
    }

    fn get_service_name(&self) -> &str {
        "slack"
    }
}

impl SlackConnector {
    /// Test connection to Slack API
    async fn test_connection(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("{}/auth.test", SLACK_API_BASE))
            .bearer_auth(&self.token)
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            Ok(json["ok"].as_bool().unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    /// Get channels list
    pub async fn get_channels(&self) -> Result<Vec<Channel>, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("{}/conversations.list", SLACK_API_BASE))
            .bearer_auth(&self.token)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let mut channels = Vec::new();

        if let Some(channels_array) = json["channels"].as_array() {
            for channel in channels_array {
                let channel = Channel {
                    id: channel["id"].as_str().unwrap_or("").to_string(),
                    name: channel["name"].as_str().unwrap_or("").to_string(),
                    channel_type: crate::types::ChannelType::Text, // Default for now
                };
                channels.push(channel);
            }
        }

        Ok(channels)
    }

    /// Send file attachment
    pub async fn send_file(&self, channel: &str, filename: &str, content: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        // Implementation for sending files would require multipart upload
        tracing::info!("File upload not implemented yet for Slack connector");
        Ok("file_upload_placeholder".to_string())
    }
}