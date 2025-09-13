use serde::{Deserialize, Serialize};

/// Message structure for service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id:          Option<String>,
    pub content:     String,
    pub author:      String,
    pub channel:     String,
    pub timestamp:   chrono::DateTime<chrono::Utc>,
    pub attachments: Vec<Attachment>,
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename:     String,
    pub content_type: String,
    pub data:         Vec<u8>,
}

/// Channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id:           String,
    pub name:         String,
    pub channel_type: ChannelType,
}

/// Channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Text,
    Voice,
    Direct,
}

/// Service event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    MessageReceived(Message),
    UserJoined {
        user_id:    String,
        channel_id: String,
    },
    UserLeft {
        user_id:    String,
        channel_id: String,
    },
    ChannelCreated(Channel),
    Error {
        message: String,
    },
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub service_name:     String,
    pub token:            String,
    pub webhook_url:      Option<String>,
    pub channel_mappings: std::collections::HashMap<String, String>,
}
