use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client as S3Client, Config as S3Config, Error as S3Error};
use serde::{Deserialize, Serialize};

use crate::types::{CloudAuth, CloudResource};
use crate::CloudProvider;

/// AWS-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub access_key: String,
    pub secret_key: String,
    pub region:     String,
}

/// AWS client wrapper
pub struct AwsClient {
    s3_client: S3Client,
    config:    AwsConfig,
}

#[async_trait]
impl CloudProvider for AwsClient {
    type Config = AwsConfig;
    type Client = Self;

    async fn new(config: Self::Config) -> Result<Self> {
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .load()
            .await;

        let s3_config = S3Config::builder().defaults(shared_config).build();

        let s3_client = S3Client::from_conf(s3_config);

        Ok(Self { s3_client, config })
    }

    async fn list_resources(&self, resource_type: &str) -> Result<Vec<CloudResource>> {
        match resource_type {
            "s3" => self.list_s3_buckets().await,
            _ => Ok(vec![]),
        }
    }

    async fn deploy_resource(&self, resource: &CloudResource) -> Result<String> {
        match resource.resource_type.as_str() {
            "s3" => self.create_s3_bucket(&resource.name).await,
            _ => Err(anyhow::anyhow!(
                "Unsupported AWS resource type: {}",
                resource.resource_type
            )),
        }
    }

    async fn get_resource_status(&self, resource_id: &str) -> Result<serde_json::Value> {
        // Parse resource ARN or parse bucket info
        let result = serde_json::json!({
            "resource_id": resource_id,
            "status": "active",
            "provider": "aws"
        });
        Ok(result)
    }

    async fn delete_resource(&self, resource_id: &str) -> Result<()> {
        // Parse resource type and delete
        if resource_id.starts_with("s3://") {
            let bucket = &resource_id[5..]; // Remove s3:// prefix
            self.delete_s3_bucket(bucket).await
        } else {
            Err(anyhow::anyhow!(
                "Unsupported AWS resource type for deletion: {}",
                resource_id
            ))
        }
    }
}

impl AwsClient {
    /// List S3 buckets
    async fn list_s3_buckets(&self) -> Result<Vec<CloudResource>> {
        let resp = self.s3_client.list_buckets().send().await?;
        let mut resources = Vec::new();

        if let Some(buckets) = resp.buckets {
            for bucket in buckets {
                if let Some(name) = bucket.name {
                    let created_at = bucket.creation_date.and_then(|dt| {
                        Some(chrono::DateTime::<chrono::Utc>::from_utc(
                            dt.to_chrono(),
                            chrono::Utc,
                        ))
                    });
                    let resource = CloudResource {
                        id: format!("s3://{}", name),
                        name,
                        resource_type: "s3".to_string(),
                        region: self.config.region.clone(),
                        status: "active".to_string(),
                        properties: HashMap::new(),
                        created_at,
                    };
                    resources.push(resource);
                }
            }
        }

        Ok(resources)
    }

    /// Create S3 bucket
    async fn create_s3_bucket(&self, bucket_name: &str) -> Result<String> {
        self.s3_client
            .create_bucket()
            .bucket(bucket_name)
            .send()
            .await
            .context("Failed to create S3 bucket")?;

        Ok(format!("s3://{}", bucket_name))
    }

    /// Delete S3 bucket
    async fn delete_s3_bucket(&self, bucket_name: &str) -> Result<()> {
        self.s3_client
            .delete_bucket()
            .bucket(bucket_name)
            .send()
            .await
            .context("Failed to delete S3 bucket")?;

        Ok(())
    }
}
