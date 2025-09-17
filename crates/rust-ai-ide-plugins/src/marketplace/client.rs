//! Plugin marketplace client for interacting with plugin repositories.
//!
//! This module provides functionality for plugin discovery, download, and installation
//! from remote marketplaces, with built-in signature verification support.

use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::registry::PluginRegistry;
use crate::interfaces::{PluginError, PluginMetadata};

/// Plugin marketplace server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceServer {
    /// Server URL
    pub url: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

/// Marketplace search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchResult {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Download URL
    pub download_url: String,
    /// SHA256 hash for verification
    pub hash: String,
    /// Digital signature
    pub signature: Option<String>,
    /// Plugin rating (out of 5 stars)
    pub rating: Option<f32>,
    /// Number of reviews
    pub review_count: Option<u32>,
    /// Download count
    pub download_count: Option<u32>,
    /// Last updated date
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Plugin categories
    pub categories: Vec<String>,
    /// Featured status
    pub featured: bool,
}

/// Plugin categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCategory {
    LanguageSupport,
    BuildTool,
    Testing,
    Refactoring,
    VersionControl,
    Theme,
    ThirdPartyIntegration,
    Debugging,
    Performance,
    Other,
}

/// Enhanced plugin review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub review_id: String,
    pub plugin_id: String,
    pub user_id: String,
    pub rating: f32,
    pub comment: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Marketplace analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceAnalytics {
    pub total_plugins: u32,
    pub active_installations: u32,
    pub downloads_this_month: u32,
    pub popular_plugins: Vec<PopularPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularPlugin {
    pub plugin_id: String,
    pub name: String,
    pub downloads: u32,
    pub rating: f32,
}

/// Marketplace plugin installation result
pub type InstallResult = Result<InstalledPlugin, PluginError>;

/// Information about an installed plugin
#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    /// Plugin ID
    pub plugin_id: String,
    /// Installed version
    pub version: String,
    /// Installation path
    pub install_path: String,
    /// Installation timestamp
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

/// Plugin marketplace client
pub struct MarketplaceClient {
    /// HTTP client
    client: Client,
    /// Marketplace servers
    servers: Vec<MarketplaceServer>,
    /// Plugin registry for signature verification
    registry: Arc<dyn PluginRegistry>,
}

impl MarketplaceClient {
    /// Create a new marketplace client
    pub fn new(registry: Arc<dyn PluginRegistry>) -> Result<Self, PluginError> {
        let client = Client::new();

        Ok(Self {
            client,
            servers: Vec::new(),
            registry,
        })
    }

    /// Add a marketplace server
    pub fn add_server(&mut self, server: MarketplaceServer) {
        self.servers.push(server);
    }

    /// Search for plugins in the marketplace
    pub async fn search_plugins(
        &self,
        query: &str,
        category: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<MarketplaceSearchResult>, PluginError> {
        let mut results = Vec::new();

        // Query all configured servers
        for server in &self.servers {
            let server_results = self
                .search_single_server(server, query, category, limit)
                .await?;
            results.extend(server_results);
        }

        Ok(results)
    }

    /// Get detailed plugin information
    pub async fn get_plugin_info(
        &self,
        plugin_id: &uuid::Uuid,
    ) -> Result<MarketplaceSearchResult, PluginError> {
        // Search all servers for the specific plugin
        for server in &self.servers {
            match self
                .get_plugin_from_server(server, plugin_id.to_string().as_str())
                .await
            {
                Ok(info) => return Ok(info),
                Err(_) => continue, // Try next server
            }
        }

        Err(PluginError::Other(format!(
            "Plugin {} not found in marketplace",
            plugin_id
        )))
    }

    /// Download and verify a plugin
    pub async fn download_plugin(
        &self,
        plugin_id: &uuid::Uuid,
        target_path: &std::path::Path,
    ) -> InstallResult {
        // Get plugin information
        let plugin_info = self.get_plugin_info(plugin_id).await?;

        // Download the plugin
        let plugin_data = self.download_plugin_data(&plugin_info.download_url).await?;

        // Verify hash
        if self.verify_hash(&plugin_data, &plugin_info.hash).await? {
            return Err(PluginError::Other(
                "Plugin hash verification failed".to_string(),
            ));
        }

        // Verify signature if available
        if let Some(signature) = &plugin_info.signature {
            let plugin_uuid = Uuid::parse_str(&plugin_info.plugin_id)
                .map_err(|e| PluginError::Other(format!("Invalid plugin ID: {}", e)))?;

            self.registry
                .verify_signature(plugin_uuid, signature, &plugin_data)
                .await?;
        }

        // Save to target path
        tokio::fs::write(target_path, plugin_data).await?;

        // Create installation record
        let installed = InstalledPlugin {
            plugin_id: plugin_info.plugin_id,
            version: plugin_info.metadata.version.to_string(),
            install_path: target_path.to_string_lossy().to_string(),
            installed_at: chrono::Utc::now(),
        };

        Ok(installed)
    }

    /// Get featured plugins
    pub async fn get_featured_plugins(&self) -> Result<Vec<MarketplaceSearchResult>, PluginError> {
        let mut results = Vec::new();

        for server in &self.servers {
            let server_results = self.get_featured_from_server(server).await?;
            results.extend(server_results);
        }

        Ok(results)
    }

    /// Get popular plugins
    pub async fn get_popular_plugins(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<PopularPlugin>, PluginError> {
        let results = self.get_popular_from_default_server(limit).await?;
        Ok(results)
    }

    /// Search plugins by category
    pub async fn search_by_category(
        &self,
        category: PluginCategory,
    ) -> Result<Vec<MarketplaceSearchResult>, PluginError> {
        let category_str = format!("{:?}", category);
        self.search_plugins("", Some(&category_str), None).await
    }

    /// Submit plugin review
    pub async fn submit_review(&self, review: PluginReview) -> Result<(), PluginError> {
        for server in &self.servers {
            match self.submit_review_to_server(server, &review).await {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }

        Err(PluginError::Other(
            "Failed to submit review to all servers".to_string(),
        ))
    }

    /// Get plugin reviews
    pub async fn get_reviews(
        &self,
        plugin_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<PluginReview>, PluginError> {
        let mut all_reviews = Vec::new();

        for server in &self.servers {
            let server_reviews = self
                .get_reviews_from_server(server, plugin_id, limit)
                .await?;
            all_reviews.extend(server_reviews);
        }

        Ok(all_reviews)
    }

    /// Get marketplace analytics
    pub async fn get_analytics(&self) -> Result<MarketplaceAnalytics, PluginError> {
        self.get_analytics_from_server(&self.servers.first().unwrap())
            .await
    }

    // Helper methods for new functionality

    async fn get_featured_from_server(
        &self,
        server: &MarketplaceServer,
    ) -> Result<Vec<MarketplaceSearchResult>, PluginError> {
        let url = format!("{}/api/plugins/featured", server.url);
        let mut request = self.client.get(&url);

        if let Some(api_key) = &server.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(PluginError::Other(
                "Failed to get featured plugins".to_string(),
            ));
        }

        let plugins: Vec<MarketplaceSearchResult> = response.json().await?;
        Ok(plugins)
    }

    async fn get_popular_from_default_server(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<PopularPlugin>, PluginError> {
        if let Some(server) = self.servers.first() {
            let mut url = format!("{}/api/analytics/popular", server.url);
            if let Some(lim) = limit {
                url.push_str(&format!("?limit={}", lim));
            }

            let mut request = self.client.get(&url);

            if let Some(api_key) = &server.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }

            let response = request.send().await?;
            if !response.status().is_success() {
                return Err(PluginError::Other(
                    "Failed to get popular plugins".to_string(),
                ));
            }

            let popular: Vec<PopularPlugin> = response.json().await?;
            Ok(popular)
        } else {
            Err(PluginError::Other(
                "No marketplace servers configured".to_string(),
            ))
        }
    }

    async fn submit_review_to_server(
        &self,
        server: &MarketplaceServer,
        review: &PluginReview,
    ) -> Result<(), PluginError> {
        let url = format!("{}/api/plugins/{}/reviews", server.url, review.plugin_id);
        let mut request = self.client.post(&url).json(review);

        if let Some(api_key) = &server.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(PluginError::Other("Failed to submit review".to_string()));
        }

        Ok(())
    }

    async fn get_reviews_from_server(
        &self,
        server: &MarketplaceServer,
        plugin_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<PluginReview>, PluginError> {
        let mut url = format!("{}/api/plugins/{}/reviews", server.url, plugin_id);
        if let Some(lim) = limit {
            url.push_str(&format!("?limit={}", lim));
        }

        let mut request = self.client.get(&url);

        if let Some(api_key) = &server.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(PluginError::Other("Failed to get reviews".to_string()));
        }

        let reviews: Vec<PluginReview> = response.json().await?;
        Ok(reviews)
    }

    async fn get_analytics_from_server(
        &self,
        server: &MarketplaceServer,
    ) -> Result<MarketplaceAnalytics, PluginError> {
        let url = format!("{}/api/analytics/overview", server.url);
        let mut request = self.client.get(&url);

        if let Some(api_key) = &server.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(PluginError::Other("Failed to get analytics".to_string()));
        }

        let analytics: MarketplaceAnalytics = response.json().await?;
        Ok(analytics)
    }

    /// Search a single server
    async fn search_single_server(
        &self,
        server: &MarketplaceServer,
        query: &str,
        category: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<MarketplaceSearchResult>, PluginError> {
        let mut url = format!(
            "{}/api/plugins/search?q={}",
            server.url,
            urlencoding::encode(query)
        );

        if let Some(cat) = category {
            url.push_str(&format!("&category={}", urlencoding::encode(cat)));
        }

        if let Some(lim) = limit {
            url.push_str(&format!("&limit={}", lim));
        }

        let mut request = self.client.get(&url);

        // Add authentication if available
        if let Some(api_key) = &server.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(PluginError::Other(format!(
                "Marketplace search failed: {}",
                response.status()
            )));
        }

        let results: Vec<MarketplaceSearchResult> = response.json().await?;
        Ok(results)
    }

    /// Get plugin info from a specific server
    async fn get_plugin_from_server(
        &self,
        server: &MarketplaceServer,
        plugin_id: &str,
    ) -> Result<MarketplaceSearchResult, PluginError> {
        let url = format!("{}/api/plugins/{}", server.url, plugin_id);

        let mut request = self.client.get(&url);

        if let Some(api_key) = &server.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(PluginError::Other(format!(
                "Plugin info request failed: {}",
                response.status()
            )));
        }

        let plugin_info: MarketplaceSearchResult = response.json().await?;
        Ok(plugin_info)
    }

    /// Download plugin data
    async fn download_plugin_data(&self, download_url: &str) -> Result<Vec<u8>, PluginError> {
        let response = self.client.get(download_url).send().await?;

        if !response.status().is_success() {
            return Err(PluginError::Other(format!(
                "Download failed: {}",
                response.status()
            )));
        }

        let data = response.bytes().await?;
        Ok(data.to_vec())
    }

    /// Verify hash of downloaded data
    async fn verify_hash(&self, data: &[u8], expected_hash: &str) -> Result<bool, PluginError> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let actual_hash = hex::encode(result);

        Ok(actual_hash == expected_hash)
    }
}

impl Default for MarketplaceClient {
    fn default() -> Self {
        panic!("MarketplaceClient requires a PluginRegistry");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::marketplace::registry::PluginRegistryImpl;

    #[tokio::test]
    async fn test_marketplace_client_creation() {
        // This would require a mock registry in a full test suite
        let registry = Arc::new(PluginRegistryImpl::new()) as Arc<dyn PluginRegistry>;

        let client = MarketplaceClient::new(registry);
        assert!(client.is_ok());
    }
}
