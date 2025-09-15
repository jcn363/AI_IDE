//! Configuration cache for performance optimization
//!
//! Provides intelligent caching with TTL, invalidation strategies,
//! and integration with the unified cache system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rust_ai_ide_cache::{Cache, CacheConfig, CacheStats, EvictionPolicy, InMemoryCache};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Configuration-specific cache
pub struct ConfigCache {
    /// Underlying unified cache
    unified_cache:   Arc<dyn Cache<String, serde_json::Value>>,
    /// Cache configuration
    config:          CacheConfig,
    /// Per-configuration metadata
    config_metadata: RwLock<HashMap<String, ConfigCacheMetadata>>,
}

/// Metadata for cached configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCacheMetadata {
    /// Configuration name
    name:            String,
    /// Source file path (if applicable)
    source_path:     Option<String>,
    /// Last load time
    last_load:       chrono::DateTime<chrono::Utc>,
    /// Cache hit count
    hit_count:       u64,
    /// Validation hash
    validation_hash: String,
}

impl ConfigCache {
    /// Create new configuration cache
    pub async fn new() -> crate::IDEResult<Self> {
        let config = CacheConfig {
            max_entries: Some(1000),
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            eviction_policy: EvictionPolicy::Lru,
            enable_metrics: true,
            max_memory_mb: Some(50),
            compression_threshold_kb: Some(10),
            background_cleanup_interval_seconds: 300,
        };

        let unified_cache = Arc::new(InMemoryCache::<String, serde_json::Value>::new(&config));

        Ok(Self {
            unified_cache,
            config,
            config_metadata: RwLock::new(HashMap::new()),
        })
    }

    /// Create cache with custom configuration
    pub async fn with_config(config: CacheConfig) -> crate::IDEResult<Self> {
        let unified_cache = Arc::new(InMemoryCache::<String, serde_json::Value>::new(&config));

        Ok(Self {
            unified_cache,
            config,
            config_metadata: RwLock::new(HashMap::new()),
        })
    }

    /// Create disabled configuration cache
    pub fn disabled() -> Self {
        let disabled_config = CacheConfig {
            max_entries: None,
            default_ttl: None,
            eviction_policy: EvictionPolicy::Lru, // or whatever the appropriate default is
            enable_metrics: false,
            max_memory_mb: None,
            compression_threshold_kb: None,
            background_cleanup_interval_seconds: 0,
        };

        Self {
            unified_cache:   Arc::new(InMemoryCache::<String, serde_json::Value>::new(
                &disabled_config,
            )),
            config:          disabled_config,
            config_metadata: RwLock::new(HashMap::new()),
        }
    }

    /// Get cached configuration
    pub async fn get<C>(&self, name: &str) -> crate::IDEResult<Option<C>>
    where
        C: serde::de::DeserializeOwned + crate::Config,
    {
        if !self.config.enable_metrics {
            return Ok(None);
        }

        let cache_key = self.make_cache_key(name);
        if let Some(json_value) = self.unified_cache.get(&cache_key).await? {
            // Update metadata
            {
                let mut metadata = self.config_metadata.write().await;
                if let Some(meta) = metadata.get_mut(name) {
                    meta.hit_count += 1;
                }
            }

            let config: C = serde_json::from_value(json_value).map_err(|e| {
                crate::RustAIError::Serialization(format!("Failed to deserialize cached config: {}", e))
            })?;

            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    /// Put configuration in cache
    pub async fn put<C>(&self, name: &str, config: C) -> crate::IDEResult<()>
    where
        C: serde::Serialize + crate::Config,
    {
        if !self.config.enable_metrics {
            return Ok(());
        }

        let cache_key = self.make_cache_key(name);
        let json_value = serde_json::to_value(&config)
            .map_err(|e| crate::RustAIError::Serialization(format!("Failed to serialize config for cache: {}", e)))?;

        let ttl = self.config.default_ttl;
        self.unified_cache
            .insert(cache_key, json_value, ttl)
            .await?;

        // Update metadata
        {
            let mut metadata = self.config_metadata.write().await;
            let meta = metadata
                .entry(name.to_string())
                .or_insert_with(|| ConfigCacheMetadata {
                    name:            name.to_string(),
                    source_path:     None,
                    last_load:       chrono::Utc::now(),
                    hit_count:       0,
                    validation_hash: self.calculate_validation_hash(&config),
                });
            meta.last_load = chrono::Utc::now();
            meta.validation_hash = self.calculate_validation_hash(&config);
        }

        Ok(())
    }

    /// Invalidate specific configuration
    pub async fn invalidate_config(&self, name: &str) -> crate::IDEResult<bool> {
        let cache_key = self.make_cache_key(name);
        let removed = self.unified_cache.remove(&cache_key).await?;
        Ok(removed.is_some())
    }

    /// Invalidate by source path
    pub async fn invalidate_by_path(&self, path: &str) -> crate::IDEResult<usize> {
        let mut removed_count = 0;
        let metadata = self.config_metadata.read().await;

        for (name, meta) in metadata.iter() {
            if meta.source_path.as_ref() == Some(&path.to_string()) {
                let cache_key = self.make_cache_key(name);
                if self.unified_cache.remove(&cache_key).await?.is_some() {
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }

    /// Clear all cached configurations
    pub async fn clear(&self) -> crate::IDEResult<()> {
        self.unified_cache.clear().await?;
        self.config_metadata.write().await.clear();
        Ok(())
    }

    /// Run cleanup operations
    pub async fn cleanup(&self) -> crate::IDEResult<usize> {
        let cleaned = self.unified_cache.cleanup_expired().await?;
        Ok(cleaned)
    }

    /// Get cache statistics
    pub async fn stats(&self) -> ConfigCacheStats {
        let unified_stats = self.unified_cache.stats().await;
        let metadata_count = self.config_metadata.read().await.len();

        ConfigCacheStats {
            unified_stats,
            metadata_count,
            cache_config: self.config.clone(),
        }
    }

    /// Get configuration metadata
    pub async fn metadata(&self, name: &str) -> Option<ConfigCacheMetadata> {
        self.config_metadata.read().await.get(name).cloned()
    }

    /// Pre-warm cache with configuration
    pub async fn prewarm<C>(&self, name: &str, config: C, source_path: Option<String>) -> crate::IDEResult<()>
    where
        C: serde::Serialize + crate::Config,
    {
        self.put(name, config).await?;

        // Update source path
        if let Some(path) = source_path {
            let mut metadata = self.config_metadata.write().await;
            if let Some(meta) = metadata.get_mut(name) {
                meta.source_path = Some(path);
            }
        }

        Ok(())
    }

    // Helper methods

    fn make_cache_key(&self, name: &str) -> String {
        format!("config:{}", name)
    }

    fn calculate_validation_hash<C>(&self, config: &C) -> String
    where
        C: serde::Serialize,
    {
        use sha2::{Digest, Sha256};
        let json = serde_json::to_string(config).unwrap_or_default();
        let hash = Sha256::digest(json.as_bytes());
        format!("{:x}", hash)
    }
}

/// Extended cache statistics for configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCacheStats {
    /// Unified cache statistics
    pub unified_stats:  CacheStats,
    /// Number of cached configurations with metadata
    pub metadata_count: usize,
    /// Cache configuration
    pub cache_config:   CacheConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestConfig {
        value:  String,
        number: i32,
    }

    impl crate::Config for TestConfig {
        const FILE_PREFIX: &'static str = "test";
        const DESCRIPTION: &'static str = "Test Configuration";

        fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
            Ok(Vec::new())
        }

        fn default_config() -> Self {
            Self {
                value:  "default".to_string(),
                number: 42,
            }
        }
    }

    #[tokio::test]
    async fn test_config_cache_operations() {
        let cache = ConfigCache::new().await.unwrap();

        let test_config = TestConfig {
            value:  "cached_value".to_string(),
            number: 123,
        };

        // Put config in cache
        cache.put("test_config", test_config.clone()).await.unwrap();

        // Get config from cache
        let retrieved: Option<TestConfig> = cache.get("test_config").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.value, "cached_value");
        assert_eq!(retrieved.number, 123);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = ConfigCache::new().await.unwrap();

        let test_config = TestConfig {
            value:  "test".to_string(),
            number: 42,
        };

        cache.put("test_config", test_config).await.unwrap();

        // Verify it's cached
        let retrieved: Option<TestConfig> = cache.get("test_config").await.unwrap();
        assert!(retrieved.is_some());

        // Invalidate
        let was_invalidated = cache.invalidate_config("test_config").await.unwrap();
        assert!(was_invalidated);

        // Verify it's no longer cached
        let retrieved: Option<TestConfig> = cache.get("test_config").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = ConfigCache::new().await.unwrap();
        let stats = cache.stats().await;

        assert_eq!(stats.metadata_count, 0);
        assert!(stats.cache_config.enable_metrics);
    }

    #[tokio::test]
    async fn test_cache_metadata() {
        let cache = ConfigCache::new().await.unwrap();

        let test_config = TestConfig {
            value:  "test".to_string(),
            number: 42,
        };

        cache.put("test_config", test_config).await.unwrap();

        let metadata = cache.metadata("test_config").await;
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "test_config");
        assert!(metadata.hit_count >= 0);
    }
}
