use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cloud authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAuth {
    pub access_key: String,
    pub secret_key: Option<String>,
    pub region: String,
    pub account_id: Option<String>,
    pub token: Option<String>,
}

/// Cloud provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub provider: String,
    pub auth: CloudAuth,
    pub settings: HashMap<String, String>,
}

/// Cloud resource representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudResource {
    pub id: String,
    pub name: String,
    pub resource_type: String,
    pub region: String,
    pub status: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cloud deployment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSpec {
    pub template: String,
    pub parameters: HashMap<String, String>,
    pub outputs: Vec<String>,
}

/// Cloud service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudServiceType {
    Compute,
    Storage,
    Database,
    Networking,
    AI,
    Security,
    Monitoring,
}

/// Cloud operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudOperationResult {
    pub success: bool,
    pub message: String,
    pub resource_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// Cloud quota information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudQuota {
    pub service: String,
    pub limit: u64,
    pub used: u64,
    pub available: u64,
}

impl CloudOperationResult {
    pub fn success(message: impl Into<String>, resource_id: Option<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            message: message.into(),
            resource_id,
            data,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            resource_id: None,
            data: None,
        }
    }
}