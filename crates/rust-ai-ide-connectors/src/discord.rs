use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::types::{Channel, Message, ServiceEvent};
use crate::ServiceConnector;

/// Discord API endpoints
const DISCORD_API_BASE: &str = "https://discord.com/api/v10";
const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";

/// Discord connector configuration
#[derive(Debug, Clone)]
pub struct DiscordConnector {
    client:       Client,
    token:        String,
    is_connected: std::sync::atomic::AtomicBool,
    event_sender: Option<mpsc::UnboundedSender<ServiceEvent>>,
}

impl DiscordConnector {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
            is_connected: std::sync::atomic::AtomicBool::new(false),
            event_sender: None,
        }
    }

    /// Set event sender for WebSocket events
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<ServiceEvent>) {
        self.event_sender = Some(sender);
    }
}

#[async_trait]
impl ServiceConnector for DiscordConnector {
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Test authentication
        let test_result = self.test_connection().await?;
        if test_result {
            self.is_connected
                .store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        } else {
            Err("Failed to authenticate with Discord API".into())
        }
    }

    async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_connected
            .store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn send_message(&self, channel: &str, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        if !self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("Not connected to Discord".into());
        }

        let payload = serde_json::json!({ "content": message });

        let response = self
            .client
            .post(&format!(
                "{}/channels/{}/messages",
                DISCORD_API_BASE, channel
            ))
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Discord API error: {}", error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;
        let message_id = response_json["id"].as_str().unwrap_or("unknown");

        Ok(message_id.to_string())
    }

    async fn listen_events(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Connect to Discord Gateway
        match connect_async(GATEWAY_URL).await {
            Ok((ws_stream, _)) => {
                self.handle_gateway_connection(ws_stream).await?;
                Ok(())
            }
            Err(e) => Err(format!("Failed to connect to Discord Gateway: {}", e).into()),
        }
    }

    async fn get_status(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let status = if self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            "connected"
        } else {
            "disconnected"
        };

        Ok(serde_json::json!({
            "service": "discord",
            "status": status,
            "token_configured": !self.token.is_empty(),
            "websocket_available": true
        }))
    }

    fn get_service_name(&self) -> &str {
        "discord"
    }
}

impl DiscordConnector {
    /// Test connection to Discord API
    async fn test_connection(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/users/@me", DISCORD_API_BASE))
            .bearer_auth(&self.token)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Get channels for a guild
    pub async fn get_guild_channels(&self, guild_id: &str) -> Result<Vec<Channel>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!(
                "{}/guilds/{}/channels",
                DISCORD_API_BASE, guild_id
            ))
            .bearer_auth(&self.token)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let mut channels = Vec::new();

        if let Some(channels_array) = json.as_array() {
            for channel in channels_array {
                let channel_type = channel["type"].as_u64().unwrap_or(0);
                let channel_type = match channel_type {
                    0 | 2 => crate::types::ChannelType::Text,
                    1 => crate::types::ChannelType::Direct,
                    3 => crate::types::ChannelType::Voice,
                    _ => crate::types::ChannelType::Text,
                };

                let channel = Channel {
                    id: channel["id"].as_str().unwrap_or("").to_string(),
                    name: channel["name"].as_str().unwrap_or("").to_string(),
                    channel_type,
                };
                channels.push(channel);
            }
        }

        Ok(channels)
    }

    /// Handle Discord Gateway WebSocket connection
    async fn handle_gateway_connection(
        &self,
        mut ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Connected to Discord Gateway");

        // Identify with Discord
        let identify_payload = serde_json::json!({
            "op": 2,
            "d": {
                "token": self.token,
                "intents": 1 << 9, // MESSAGE_CREATE intent
                "properties": {
                    "$os": "linux",
                    "$browser": "rust-ai-ide",
                    "$device": "rust-ai-ide"
                }
            }
        });

        let payload_str = serde_json::to_string(&identify_payload)?;
        ws_stream.send(payload_str.into()).await?;

        // Listen for messages
        while let Some(message) = ws_stream.next().await {
            match message {
                Ok(msg) =>
                    if let Err(e) = self.handle_gateway_message(msg).await {
                        tracing::error!("Error handling gateway message: {}", e);
                    },
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        tracing::info!("Disconnected from Discord Gateway");
        Ok(())
    }

    /// Handle incoming WebSocket messages from Discord Gateway
    async fn handle_gateway_message(
        &self,
        message: tokio_tungstenite::tungstenite::Message,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                let json: serde_json::Value = serde_json::from_str(&text)?;
                if let Some(op_code) = json["op"].as_u64() {
                    match op_code {
                        0 => {
                            // Dispatch
                            if let Some(event_type) = json["t"].as_str() {
                                tracing::debug!("Received Discord event: {}", event_type);

                                if let Some(event_sender) = &self.event_sender {
                                    // Handle MESSAGE_CREATE event specifically
                                    if event_type == "MESSAGE_CREATE" {
                                        if let Some(d) = json["d"].as_object() {
                                            if let Ok(message) = self.parse_discord_message(d) {
                                                let _ = event_sender.send(ServiceEvent::MessageReceived(message));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        11 => { // Heartbeat ACK - ignore
                        }
                        _ => tracing::debug!("Received op code: {}", op_code),
                    }
                }
            }
            _ => tracing::debug!("Received non-text message from Discord Gateway"),
        }

        Ok(())
    }

    /// Parse Discord message from Gateway payload
    fn parse_discord_message(
        &self,
        data: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let id = data["id"].as_str().unwrap_or("").to_string();
        let content = data["content"].as_str().unwrap_or("").to_string();
        let author = data["author"]["username"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let channel_id = data["channel_id"].as_str().unwrap_or("").to_string();

        Ok(Message {
            id: Some(id),
            content,
            author,
            channel: channel_id,
            timestamp: chrono::Utc::now(),
            attachments: Vec::new(), // TODO: Parse attachments
        })
    }
}
