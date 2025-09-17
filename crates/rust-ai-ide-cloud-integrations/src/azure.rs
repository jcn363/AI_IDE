use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use azure_identity::{DefaultAzureCredential, TokenCredentialOptions};
use azure_storage::blob::{BlobServiceClient, ContainerService, CopyBlobOptions};
use serde::{Deserialize, Serialize};

use crate::types::{CloudAuth, CloudResource};
use crate::CloudProvider;

/// Azure-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub account_name: String,
    pub account_key: String,
    pub endpoint: Option<String>,
}

/// Azure client wrapper
pub struct AzureClient {
    blob_client: BlobServiceClient,
    config: AzureConfig,
}

impl AzureClient {
    /// Create a new Azure client
    pub async fn new(config: AzureConfig) -> Result<Self> {
        let blob_client = BlobServiceClient::new_access_key(
            config.account_name.clone(),
            config.account_key.clone(),
        );

        Ok(Self {
            blob_client,
            config,
        })
    }
}

#[async_trait]
impl CloudProvider for AzureClient {
    type Config = AzureConfig;
    type Client = Self;

    async fn new(config: Self::Config) -> Result<Self> {
        AzureClient::new(config).await
    }

    async fn list_resources(&self, resource_type: &str) -> Result<Vec<CloudResource>> {
        match resource_type {
            "storage" => self.list_storage_containers().await,
            _ => Ok(vec![]),
        }
    }

    async fn deploy_resource(&self, resource: &CloudResource) -> Result<String> {
        match resource.resource_type.as_str() {
            "storage" => self.create_storage_container(&resource.name).await,
            _ => Err(anyhow::anyhow!(
                "Unsupported Azure resource type: {}",
                resource.resource_type
            )),
        }
    }

    async fn get_resource_status(&self, resource_id: &str) -> Result<serde_json::Value> {
        let result = serde_json::json!({
            "resource_id": resource_id,
            "status": "active",
            "provider": "azure"
        });
        Ok(result)
    }

    async fn delete_resource(&self, resource_id: &str) -> Result<()> {
        if resource_id.starts_with("azure://") {
            let container = &resource_id[9..]; // Remove azure:// prefix
            self.delete_storage_container(container).await
        } else {
            Err(anyhow::anyhow!(
                "Unsupported Azure resource type for deletion: {}",
                resource_id
            ))
        }
    }
}

impl AzureClient {
    /// List storage containers (blob containers)
    async fn list_storage_containers(&self) -> Result<Vec<CloudResource>> {
        let client = self.blob_client.clone();
        let containers = client
            .list_containers()
            .include_metadata(true)
            .execute()
            .await?;

        let mut resources = Vec::new();

        for container in containers.containers {
            let resource = CloudResource {
                id: format!("azure://{}", container.name),
                name: container.name,
                resource_type: "storage".to_string(),
                region: "azure-global".to_string(), // Azure uses regions, but simplified here
                status: "active".to_string(),
                properties: HashMap::new(),
                created_at: None, // Azure API might not provide creation time directly
            };
            resources.push(resource);
        }

        Ok(resources)
    }

    /// Create storage container
    async fn create_storage_container(&self, container_name: &str) -> Result<String> {
        let client = self.blob_client.clone();
        let container_client = client.container_client(container_name);

        container_client
            .create()
            .execute()
            .await
            .context("Failed to create Azure storage container")?;

        Ok(format!("azure://{}", container_name))
    }

    /// Delete storage container
    async fn delete_storage_container(&self, container_name: &str) -> Result<()> {
        let client = self.blob_client.clone();
        let container_client = client.container_client(container_name);

        container_client
            .delete()
            .execute()
            .await
            .context("Failed to delete Azure storage container")?;

        Ok(())
    }
}
