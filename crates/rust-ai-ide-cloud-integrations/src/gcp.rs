use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use google_cloud_storage::client::{Client as StorageClient, ClientConfig};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::types::{CloudAuth, CloudResource};
use crate::CloudProvider;

/// GCP-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpConfig {
    pub project_id: String,
    pub key_file_path: Option<String>,
    pub token: Option<String>,
}

/// GCP client wrapper
pub struct GcpClient {
    storage_client: StorageClient,
    config: GcpConfig,
}

impl GcpClient {
    /// Create a new GCP client
    pub async fn new(config: GcpConfig) -> Result<Self> {
        let client_config = if let Some(key_path) = &config.key_file_path {
            ClientConfig::default().with_auth().await?
        } else {
            ClientConfig::default()
        };

        let storage_client = StorageClient::new(client_config).await;

        Ok(Self {
            storage_client,
            config,
        })
    }
}

#[async_trait]
impl CloudProvider for GcpClient {
    type Config = GcpConfig;
    type Client = Self;

    async fn new(config: Self::Config) -> Result<Self> {
        GcpClient::new(config).await
    }

    async fn list_resources(&self, resource_type: &str) -> Result<Vec<CloudResource>> {
        match resource_type {
            "storage" => self.list_storage_buckets().await,
            _ => Ok(vec![]),
        }
    }

    async fn deploy_resource(&self, resource: &CloudResource) -> Result<String> {
        match resource.resource_type.as_str() {
            "storage" => self.create_storage_bucket(&resource.name).await,
            _ => Err(anyhow::anyhow!(
                "Unsupported GCP resource type: {}",
                resource.resource_type
            )),
        }
    }

    async fn get_resource_status(&self, resource_id: &str) -> Result<serde_json::Value> {
        let result = serde_json::json!({
            "resource_id": resource_id,
            "status": "active",
            "provider": "gcp"
        });
        Ok(result)
    }

    async fn delete_resource(&self, resource_id: &str) -> Result<()> {
        if resource_id.starts_with("gcp://") {
            let bucket = &resource_id[5..]; // Remove gcp:// prefix
            self.delete_storage_bucket(bucket).await
        } else {
            Err(anyhow::anyhow!(
                "Unsupported GCP resource type for deletion: {}",
                resource_id
            ))
        }
    }
}

impl GcpClient {
    /// List storage buckets
    async fn list_storage_buckets(&self) -> Result<Vec<CloudResource>> {
        let project_id = &self.config.project_id;
        let buckets = self.storage_client.list_buckets(project_id).await?;

        let mut resources = Vec::new();

        for bucket in buckets {
            let created_at = bucket.time_created.and_then(|dt| {
                let millis = dt.seconds * 1000 + (dt.nanos / 1_000_000) as i64;
                chrono::DateTime::from_timestamp(
                    millis / 1000,
                    ((millis % 1000) * 1_000_000) as u32,
                )
            });

            let resource = CloudResource {
                id: format!("gcp://{}", bucket.name),
                name: bucket.name,
                resource_type: "storage".to_string(),
                region: bucket.location.unwrap_or_else(|| "global".to_string()),
                status: "active".to_string(),
                properties: HashMap::new(),
                created_at,
            };
            resources.push(resource);
        }

        Ok(resources)
    }

    /// Create storage bucket
    async fn create_storage_bucket(&self, bucket_name: &str) -> Result<String> {
        let project_id = &self.config.project_id;
        let bucket = google_cloud_storage::bucket::Bucket::new(bucket_name, project_id, None);

        self.storage_client
            .create_bucket(project_id, bucket_name.to_string())
            .await
            .context("Failed to create GCP storage bucket")?;

        Ok(format!("gcp://{}", bucket_name))
    }

    /// Delete storage bucket
    async fn delete_storage_bucket(&self, bucket_name: &str) -> Result<()> {
        self.storage_client
            .delete_bucket(bucket_name)
            .await
            .context("Failed to delete GCP storage bucket")?;

        Ok(())
    }
}
