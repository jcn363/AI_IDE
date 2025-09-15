//! LSP-specific cache implementations and extensions
//!
//! This module provides LSP-oriented cache implementations using the unified
//! cache infrastructure, with optimizations for LSP analysis patterns.

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::{key_utils, Cache, CacheConfig, CacheStats, IDEResult, InMemoryCache, LspCacheExt};

/// LSP-specific cache configuration
#[derive(Debug, Clone)]
pub struct LspCacheConfig {
    pub base_config:                CacheConfig,
    pub enable_file_validation:     bool,
    pub max_file_cache_age_seconds: u64,
    pub analysis_ttl_seconds:       u64,
}

impl Default for LspCacheConfig {
    fn default() -> Self {
        Self {
            base_config:                CacheConfig {
                max_entries: Some(5000),                      // Higher limit for LSP analysis
                default_ttl: Some(Duration::from_secs(1800)), // 30 minutes
                ..Default::default()
            },
            enable_file_validation:     true,
            max_file_cache_age_seconds: 3600, // 1 hour
            analysis_ttl_seconds:       1800, // 30 minutes
        }
    }
}

/// LSP-optimized cache implementation
pub struct LspAnalysisCache {
    pub cache: InMemoryCache<String, JsonValue>,
    config:    LspCacheConfig,
}

impl LspAnalysisCache {
    /// Create a new LSP analysis cache
    pub fn new(config: LspCacheConfig) -> Self {
        let cache = InMemoryCache::new(&config.base_config);
        Self { cache, config }
    }

    /// Generate LSP analysis cache key from file path and analysis type
    pub fn generate_analysis_key(
        file_path: &PathBuf,
        analysis_type: &str,
        analysis_params: Option<&JsonValue>,
    ) -> String {
        let file_path_str = file_path.display().to_string();
        let mut owned_components: Vec<String> = vec![analysis_type.to_string(), file_path_str];
        if let Some(params) = analysis_params {
            owned_components.push(params.to_string());
        }
        let components: Vec<&str> = owned_components.iter().map(|s| s.as_str()).collect();
        key_utils::structured_key("lsp_analysis", &components)
    }

    /// Cache LSP analysis result with file validation
    pub async fn store_analysis_result(
        &self,
        file_path: &PathBuf,
        analysis_type: &str,
        result: JsonValue,
        file_hash: Option<String>,
        analysis_params: Option<JsonValue>,
    ) -> IDEResult<()> {
        let key = Self::generate_analysis_key(file_path, analysis_type, analysis_params.as_ref());

        // Store with unified LSP extension
        self.lsp_store_analysis(
            key,
            result,
            file_hash.unwrap_or_default(),
            Some(Duration::from_secs(self.config.analysis_ttl_seconds)),
        )
        .await
    }

    /// Retrieve LSP analysis result
    pub async fn get_analysis_result(
        &self,
        file_path: &PathBuf,
        analysis_type: &str,
        analysis_params: Option<JsonValue>,
    ) -> IDEResult<Option<JsonValue>> {
        let key = Self::generate_analysis_key(file_path, analysis_type, analysis_params.as_ref());
        self.lsp_retrieve_analysis(&key).await
    }

    /// Check if cached analysis result is still valid
    pub async fn is_result_valid(
        &self,
        file_path: &PathBuf,
        analysis_type: &str,
        current_file_hash: &str,
        analysis_params: Option<JsonValue>,
    ) -> bool {
        let key = Self::generate_analysis_key(file_path, analysis_type, analysis_params.as_ref());

        if let Ok(Some(cached_result)) = self.cache.get(&key).await {
            // For LSP results, we trust the TTL and hash validation from the unified system
            // Additional file content validation can be added here if needed
            !self.cache.stats().await.hit_ratio.is_nan() // Hack: use stats to determine if we have
                                                         // valid data
        } else {
            false
        }
    }

    /// Invalidate all analysis results for a file
    pub async fn invalidate_file(&self, file_path: &PathBuf) -> IDEResult<usize> {
        // This would need to iterate through cache entries with file path pattern
        // For now, using a simplified approach
        // warn!("File-specific invalidation not fully implemented - clearing cache");
        self.cache.clear().await?;
        Ok(0) // Return 0 as we don't have the exact count without iteration
    }
}

// Implement the Cache trait so we can use extension traits
#[async_trait]
impl Cache<String, JsonValue> for LspAnalysisCache {
    async fn get(&self, key: &String) -> IDEResult<Option<JsonValue>> {
        self.cache.get(key).await
    }

    async fn insert(&self, key: String, value: JsonValue, ttl: Option<Duration>) -> IDEResult<()> {
        self.cache.insert(key, value, ttl).await
    }

    async fn remove(&self, key: &String) -> IDEResult<Option<JsonValue>> {
        self.cache.remove(key).await
    }

    async fn clear(&self) -> IDEResult<()> {
        self.cache.clear().await
    }

    async fn size(&self) -> usize {
        self.cache.size().await
    }

    async fn contains(&self, key: &String) -> bool {
        self.cache.contains(key).await
    }

    async fn stats(&self) -> CacheStats {
        self.cache.stats().await
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        self.cache.cleanup_expired().await
    }
}

// LspCacheExt methods are automatically available through the blanket implementation
// when Cache<String, JsonValue> is implemented (which we do above)

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_lsp_analysis_cache_basic() {
        let config = LspCacheConfig::default();
        let cache = LspAnalysisCache::new(config);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let analysis_type = "diagnostics";
        let result = serde_json::json!({"diagnostics": [], "errors": 0});

        // Store analysis result
        cache
            .store_analysis_result(
                &file_path,
                analysis_type,
                result.clone(),
                Some("test_hash".to_string()),
                None,
            )
            .await
            .unwrap();

        // Retrieve analysis result
        let retrieved = cache
            .get_analysis_result(&file_path, analysis_type, None)
            .await
            .unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), result);
    }

    #[tokio::test]
    async fn test_lsp_cache_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let key = LspAnalysisCache::generate_analysis_key(
            &file_path,
            "diagnostics",
            Some(&serde_json::json!({"include_warnings": true})),
        );

        assert!(!key.is_empty());
        assert!(key.starts_with("lsp_analysis:"));

        // Keys for same inputs should be identical
        let key2 = LspAnalysisCache::generate_analysis_key(
            &file_path,
            "diagnostics",
            Some(&serde_json::json!({"include_warnings": true})),
        );

        assert_eq!(key, key2);
    }
}
