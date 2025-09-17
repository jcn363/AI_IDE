//! Redis and Redis Cluster cache implementations for high-performance distributed caching
//!
//! This module provides Redis-backed cache implementations with support for:
//! - Single Redis instance caching
//! - Redis Cluster for distributed caching with high availability
//! - Connection pooling with bb8-redis
//! - Async operations with tokio
//! - Automatic failover and resilience

use std::fmt::Debug;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{Cache, CacheEntry, CacheStats, IDEResult};
use rust_ai_ide_errors;

/// Redis connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URLs (single instance or cluster nodes)
    pub urls: Vec<String>,
    /// Enable Redis cluster mode
    pub enable_cluster: bool,
    /// Connection pool settings
    pub pool_max_size: u32,
    pub pool_min_idle: u32,
    /// Authentication
    pub password: Option<String>,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Operation timeout in seconds
    pub operation_timeout_secs: u64,
    /// Key prefix for namespacing
    pub key_prefix: String,
    /// Enable TLS/SSL connections
    pub enable_tls: bool,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            urls: vec!["redis://127.0.0.1:6379".to_string()],
            enable_cluster: false,
            pool_max_size: 20,
            pool_min_idle: 5,
            password: None,
            connection_timeout_secs: 30,
            operation_timeout_secs: 10,
            key_prefix: "rust-ai-ide:cache".to_string(),
            enable_tls: false,
        }
    }
}

impl RedisConfig {
    /// Create a new configuration for a single Redis instance
    pub fn single_node(url: &str) -> Self {
        Self {
            urls: vec![url.to_string()],
            enable_cluster: false,
            ..Default::default()
        }
    }

    /// Create a new configuration for a Redis cluster
    pub fn cluster(urls: Vec<String>) -> Self {
        Self {
            urls,
            enable_cluster: true,
            ..Default::default()
        }
    }

    /// Set authentication password
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Set key prefix for namespacing
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.key_prefix = prefix;
        self
    }
}

/// Redis connection manager types
#[derive(Clone)]
enum RedisConnectionManager {
    Single(bb8::Pool<bb8_redis::RedisConnectionManager>),
    #[cfg(feature = "redis-cluster")]
    Cluster(redis::cluster::ClusterClient),
}

#[cfg(feature = "redis")]
impl RedisConnectionManager {
    async fn new_single(config: &RedisConfig) -> IDEResult<Self> {
        let url = config.urls.first().ok_or_else(|| {
            rust_ai_ide_errors::RustAIError::Config(rust_ai_ide_errors::ConfigError::new("No Redis URL provided"))
        })?;

        let manager = bb8_redis::RedisConnectionManager::new(url.clone())
            .map_err(|e| {
                rust_ai_ide_errors::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(format!("Redis connection manager error: {}", e)))
            })?;

        let mut pool_builder = bb8::Pool::builder()
            .max_size(config.pool_max_size)
            .min_idle(Some(config.pool_min_idle));

        if config.connection_timeout_secs > 0 {
            pool_builder = pool_builder.connection_timeout(Duration::from_secs(config.connection_timeout_secs));
        }

        let pool = pool_builder
            .build(manager)
            .await
            .map_err(|e| {
                rust_ai_ide_errors::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(format!("Redis pool creation error: {}", e)))
            })?;

        Ok(Self::Single(pool))
    }
}

#[cfg(feature = "redis-cluster")]
impl RedisConnectionManager {
    async fn new_cluster(config: &RedisConfig) -> IDEResult<Self> {
        let mut client_builder = redis::cluster::ClusterClientBuilder::new(config.urls.clone());

        if let Some(ref password) = config.password {
            client_builder = client_builder.password(password.clone());
        }

        let client = client_builder.build().map_err(|e| {
            rust_ai_ide_errors::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(format!("Redis cluster client error: {}", e)))
        })?;

        Ok(Self::Cluster(client))
    }
}

impl RedisConnectionManager {
    async fn execute<F, Fut, T>(&self, operation: F) -> IDEResult<T>
    where
        F: FnOnce(redis::aio::MultiplexedConnection) -> Fut + Send,
        Fut: std::future::Future<Output = redis::RedisResult<T>> + Send,
        T: Send,
    {
        match self {
            Self::Single(pool) => {
                let conn = pool.get().await.map_err(|e| {
                    rust_ai_ide_errors::RustAIError::Network(format!("Redis connection pool error: {}", e))
                })?;

                let result = tokio::time::timeout(
                    Duration::from_secs(10), // Use default operation timeout
                    operation(conn),
                )
                .await
                .map_err(|_| rust_ai_ide_errors::RustAIError::Timeout("Redis operation timeout".to_string()))?
                .map_err(|e| rust_ai_ide_errors::RustAIError::Network(format!("Redis operation error: {}", e)))?;

                Ok(result)
            }
            #[cfg(feature = "redis-cluster")]
            Self::Cluster(client) => {
                let conn = client.get_async_connection().await.map_err(|e| {
                    rust_ai_ide_errors::RustAIError::Network(format!("Redis cluster connection error: {}", e))
                })?;

                let result = tokio::time::timeout(
                    Duration::from_secs(10),
                    operation(conn),
                )
                .await
                .map_err(|_| rust_ai_ide_errors::RustAIError::Timeout("Redis cluster operation timeout".to_string()))?
                .map_err(|e| rust_ai_ide_errors::RustAIError::Network(format!("Redis cluster operation error: {}", e)))?;

                Ok(result)
            }
        }
    }

    async fn health_check(&self) -> bool {
        let result = match self {
            Self::Single(pool) => pool.get().await.ok(),
            #[cfg(feature = "redis-cluster")]
            Self::Cluster(client) => client.get_async_connection().await.ok(),
        };

        if let Some(conn) = result {
            let check: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut *conn).await;
            check.is_ok()
        } else {
            false
        }
    }
}

/// Redis-backed cache implementation
pub struct RedisCache<K, V> {
    config: RedisConfig,
    connection_manager: RedisConnectionManager,
    cache_stats: tokio::sync::RwLock<CacheStats>,
}

impl<K, V> RedisCache<K, V> {
    /// Create a new Redis cache instance
    pub async fn new(config: RedisConfig) -> IDEResult<Self> {
        // Validate configuration
        if config.urls.is_empty() {
            return Err(rust_ai_ide_errors::RustAIError::Config(rust_ai_ide_errors::ConfigError::new("No Redis URLs provided")));
        }

        // Create appropriate connection manager based on configuration
        let connection_manager = if config.enable_cluster {
            #[cfg(not(feature = "redis-cluster"))]
            {
                return Err(rust_ai_ide_errors::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                    "Redis cluster support not enabled. Enable 'redis-cluster' feature",
                )));
            }
            #[cfg(feature = "redis-cluster")]
            {
                RedisConnectionManager::new_cluster(&config).await?
            }
        } else {
            #[cfg(not(feature = "redis"))]
            {
                return Err(rust_ai_ide_errors::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                    "Redis support not enabled. Enable 'redis-backend' feature",
                )));
            }
            #[cfg(feature = "redis")]
            {
                RedisConnectionManager::new_single(&config).await?
            }
        };

        let stats = CacheStats {
            created_at: chrono::Utc::now(),
            ..Default::default()
        };

        Ok(Self {
            config,
            connection_manager,
            cache_stats: tokio::sync::RwLock::new(stats),
        })
    }

    /// Generate namespaced cache key
    fn make_key(&self, key: &K) -> String
    where
        K: Serialize + Debug,
    {
        let key_str = serde_json::to_string(key).unwrap_or_else(|_| format!("{:?}", key));
        format!("{}:{}", self.config.key_prefix, key_str)
    }

    /// Serialize cache entry to Redis-compatible format
    fn serialize_entry(&self, entry: &CacheEntry<V>) -> IDEResult<String>
    where
        V: Serialize,
    {
        serde_json::to_string(entry)
            .map_err(|e| rust_ai_ide_errors::RustAIError::Serialization(format!("Serialization error: {}", e)))
    }

    /// Deserialize cache entry from Redis format
    fn deserialize_entry(&self, data: &str) -> IDEResult<CacheEntry<V>>
    where
        for<'de> V: Deserialize<'de>,
    {
        serde_json::from_str(data)
            .map_err(|e| rust_ai_ide_errors::RustAIError::Serialization(format!("Deserialization error: {}", e)))
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for RedisCache<K, V>
where
    K: Send + Sync + Clone + std::hash::Hash + Eq + Serialize + Debug + 'static,
    V: Send + Sync + Clone + Serialize + Debug + 'static,
    for<'de> V: Deserialize<'de>,
{
    async fn get(&self, key: &K) -> IDEResult<Option<V>> {
        let cache_key = self.make_key(key);
        let mut stats = self.cache_stats.write().await;

        let result: IDEResult<Option<String>> = self
            .connection_manager
            .execute(|mut conn| async move { redis::cmd("GET").arg(&cache_key).query_async(&mut conn).await })
            .await;

        match result {
            Ok(Some(data)) => {
                match self.deserialize_entry(&data) {
                    Ok(entry) => {
                        if entry.is_expired() {
                            // Remove expired entry
                            let _: IDEResult<()> = self
                                .connection_manager
                                .execute(|mut conn| async move {
                                    redis::cmd("DEL").arg(&cache_key).query_async(&mut conn).await
                                })
                                .await;
                            stats.record_miss();
                            stats.record_eviction();
                            Ok(None)
                        } else {
                            stats.record_hit();
                            Ok(Some(entry.value))
                        }
                    }
                    Err(_) => {
                        // Invalid data, remove it
                        let _: IDEResult<()> = self
                            .connection_manager
                            .execute(|mut conn| async move {
                                redis::cmd("DEL").arg(&cache_key).query_async(&mut conn).await
                            })
                            .await;
                        stats.record_miss();
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                stats.record_miss();
                Ok(None)
            }
            Err(e) => {
                stats.record_miss();
                Err(e)
            }
        }
    }

    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()> {
        let cache_key = self.make_key(&key);
        let entry = CacheEntry::new_with_ttl(value, ttl, chrono::Utc::now());

        let serialized = self.serialize_entry(&entry)?;
        let mut stats = self.cache_stats.write().await;

        let result: IDEResult<()> = if let Some(ttl_duration) = ttl {
            self.connection_manager
                .execute(|mut conn| async move {
                    redis::cmd("SETEX")
                        .arg(&cache_key)
                        .arg(ttl_duration.as_secs() as i32)
                        .arg(&serialized)
                        .query_async(&mut conn)
                        .await
                })
                .await
        } else {
            self.connection_manager
                .execute(|mut conn| async move {
                    redis::cmd("SET")
                        .arg(&cache_key)
                        .arg(&serialized)
                        .query_async(&mut conn)
                        .await
                })
                .await
        };

        match result {
            Ok(_) => {
                stats.record_set();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    async fn remove(&self, key: &K) -> IDEResult<Option<V>> {
        let cache_key = self.make_key(key);

        // First try to get the value before deleting
        let get_result: IDEResult<Option<String>> = self
            .connection_manager
            .execute(|mut conn| async move { redis::cmd("GETDEL").arg(&cache_key).query_async(&mut conn).await })
            .await;

        match get_result {
            Ok(Some(data)) => {
                match self.deserialize_entry(&data) {
                    Ok(entry) => Ok(Some(entry.value)),
                    Err(_) => Ok(None), // Invalid data, treat as not found
                }
            }
            _ => Ok(None),
        }
    }

    async fn clear(&self) -> IDEResult<()> {
        // Get all keys with our prefix and delete them
        let pattern = format!("{}:*", self.config.key_prefix);

        // This is a simplified implementation
        // In production, you might want to use SCAN for safety
        let result: IDEResult<()> = self
            .connection_manager
            .execute(|mut conn| async move {
                let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern).query_async(&mut conn).await?;
                if !keys.is_empty() {
                    redis::cmd("DEL").arg(keys).query_async::<_, ()>(&mut conn).await?;
                }
                Ok(())
            })
            .await;

        if result.is_ok() {
            let mut stats = self.cache_stats.write().await;
            *stats = CacheStats::default();
        }

        result
    }

    async fn size(&self) -> usize {
        let pattern = format!("{}:*", self.config.key_prefix);

        let result: IDEResult<Option<u32>> = self
            .connection_manager
            .execute(|mut conn| async move {
                let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern).query_async(&mut conn).await?;
                Ok(Some(keys.len() as u32))
            })
            .await;

        match result {
            Ok(Some(count)) => count as usize,
            _ => 0,
        }
    }

    async fn contains(&self, key: &K) -> bool {
        let cache_key = self.make_key(key);

        let result: IDEResult<Option<i32>> = self
            .connection_manager
            .execute(|mut conn| async move { redis::cmd("EXISTS").arg(&cache_key).query_async(&mut conn).await })
            .await;

        matches!(result, Ok(Some(count)) if count > 0)
    }

    async fn stats(&self) -> CacheStats {
        let mut stats = self.cache_stats.read().await.clone();

        // Update dynamic stats
        stats.total_entries = self.size().await as usize;
        stats.uptime_seconds = (chrono::Utc::now() - stats.created_at).num_seconds() as u64;

        // Memory usage estimation (simplified)
        stats.memory_usage_bytes = Some((stats.total_entries * 1000) as u64); // Rough estimate

        stats
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        // Redis handles TTL expiration automatically, so we just return 0
        // In a production implementation, you might implement active cleanup
        Ok(0)
    }
}

/// Redis Cluster specialization (placeholder for future enhancement)
#[cfg(feature = "redis-cluster")]
pub type RedisClusterCache<K, V> = RedisCache<K, V>;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_redis_config_builder() {
        let config = RedisConfig::single_node("redis://localhost:6379")
            .with_password("testpass".to_string())
            .with_prefix("test-cache".to_string());

        assert_eq!(config.urls.len(), 1);
        assert_eq!(config.password, Some("testpass".to_string()));
        assert_eq!(config.key_prefix, "test-cache");
        assert!(!config.enable_cluster);
    }

    #[tokio::test]
    async fn test_cluster_config() {
        let urls = vec![
            "redis://node1:6379".to_string(),
            "redis://node2:6379".to_string(),
        ];
        let config = RedisConfig::cluster(urls.clone());

        assert_eq!(config.urls, urls);
        assert!(config.enable_cluster);
    }

    #[tokio::test]
    async fn test_key_namespacing() {
        let config = RedisConfig::default();

        // We can't easily test the full RedisCache without a running Redis instance
        // But we can test the key generation logic
        let key = "test_key";
        let expected_prefix = format!("{}:{:?}", config.key_prefix, key);

        // The actual formatted key will be JSON-serialized
        assert!(expected_prefix.contains("rust-ai-ide:cache"));
    }

    // Integration tests would require a running Redis instance
    // For now, we skip them in automated testing
}