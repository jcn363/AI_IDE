use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::debug;

use crate::FeatureFlagError;

/// Trait for feature flag providers
#[async_trait]
pub trait FeatureFlagProvider: Send + Sync {
    /// Fetch feature flags from the provider
    async fn fetch_flags(&self) -> Result<HashMap<String, String>, FeatureFlagError>;

    /// Check if provider is healthy
    async fn is_healthy(&self) -> bool;
}

/// Kubernetes ConfigMap provider
pub struct K8sConfigMapProvider {
    namespace: String,
    configmap_name: String,
    k8s_client: Arc<RwLock<Option<k8s_openapi::api::core::v1::ConfigMap>>>,
}

impl K8sConfigMapProvider {
    pub fn new(namespace: String, configmap_name: String) -> Self {
        Self {
            namespace,
            configmap_name,
            k8s_client: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl FeatureFlagProvider for K8sConfigMapProvider {
    async fn fetch_flags(&self) -> Result<HashMap<String, String>, FeatureFlagError> {
        // In a real implementation, use kube crate to fetch ConfigMap
        // For now, simulate by returning an empty map
        debug!(
            "Fetching flags from ConfigMap {}/{}",
            self.namespace, self.configmap_name
        );
        Ok(HashMap::new())
    }

    async fn is_healthy(&self) -> bool {
        // Simulate health check
        true
    }
}

/// Environment variable provider
pub struct EnvVarProvider {
    prefix: String,
}

impl EnvVarProvider {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

#[async_trait]
impl FeatureFlagProvider for EnvVarProvider {
    async fn fetch_flags(&self) -> Result<HashMap<String, String>, FeatureFlagError> {
        let mut flags = HashMap::new();

        for (key, value) in std::env::vars() {
            if key.starts_with(&self.prefix) {
                let flag_name = key.trim_start_matches(&self.prefix).to_lowercase();
                flags.insert(flag_name, value);
            }
        }

        debug!("Fetched {} flags from environment variables", flags.len());
        Ok(flags)
    }

    async fn is_healthy(&self) -> bool {
        true
    }
}

/// HTTP provider for remote flag service
pub struct HttpProvider {
    url: String,
    client: reqwest::Client,
}

impl HttpProvider {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl FeatureFlagProvider for HttpProvider {
    async fn fetch_flags(&self) -> Result<HashMap<String, String>, FeatureFlagError> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| FeatureFlagError::NetworkError(e.to_string()))?;

        let flags: HashMap<String, String> = response
            .json()
            .await
            .map_err(|e| FeatureFlagError::ParseError(e.to_string()))?;

        debug!("Fetched {} flags from HTTP provider", flags.len());
        Ok(flags)
    }

    async fn is_healthy(&self) -> bool {
        match self.client.get(&self.url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

/// Composite provider that combines multiple providers
pub struct CompositeProvider {
    providers: Vec<Arc<dyn FeatureFlagProvider>>,
}

impl CompositeProvider {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn FeatureFlagProvider>) -> Self {
        self.providers.push(provider);
        self
    }
}

#[async_trait]
impl FeatureFlagProvider for CompositeProvider {
    async fn fetch_flags(&self) -> Result<HashMap<String, String>, FeatureFlagError> {
        let mut combined_flags = HashMap::new();

        for provider in &self.providers {
            if provider.is_healthy().await {
                match provider.fetch_flags().await {
                    Ok(flags) => {
                        combined_flags.extend(flags);
                    }
                    Err(e) => {
                        debug!("Provider error (continuing): {}", e);
                    }
                }
            }
        }

        debug!(
            "Fetched {} flags from {} providers",
            combined_flags.len(),
            self.providers.len()
        );
        Ok(combined_flags)
    }

    async fn is_healthy(&self) -> bool {
        self.providers
            .iter()
            .any(|p| futures::executor::block_on(p.is_healthy()))
    }
}

/// Cached provider with refresh capability
pub struct CachedProvider<P: FeatureFlagProvider + 'static> {
    inner: P,
    cache: Arc<RwLock<Option<HashMap<String, String>>>>,
    ttl: std::time::Duration,
    last_refresh: Arc<RwLock<std::time::Instant>>,
}

impl<P: FeatureFlagProvider + 'static> CachedProvider<P> {
    pub fn new(inner: P, ttl: std::time::Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(None)),
            ttl,
            last_refresh: Arc::new(RwLock::new(std::time::Instant::now() - ttl)),
        }
    }
}

#[async_trait]
impl<P: FeatureFlagProvider + 'static> FeatureFlagProvider for CachedProvider<P> {
    async fn fetch_flags(&self) -> Result<HashMap<String, String>, FeatureFlagError> {
        let now = std::time::Instant::now();
        let last_refresh = *self.last_refresh.read().await;

        // Check if cache is still valid
        if now.duration_since(last_refresh) < self.ttl {
            if let Some(cached) = self.cache.read().await.as_ref() {
                debug!("Returning cached flags");
                return Ok(cached.clone());
            }
        }

        // Refresh cache
        let fresh_flags = self.inner.fetch_flags().await?;
        *self.cache.write().await = Some(fresh_flags.clone());
        *self.last_refresh.write().await = now;

        debug!("Refreshed cached flags");
        Ok(fresh_flags)
    }

    async fn is_healthy(&self) -> bool {
        self.inner.is_healthy().await
    }
}
