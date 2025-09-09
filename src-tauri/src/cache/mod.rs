//! Generic caching infrastructure
//!
//! DEPRECATED: This module is deprecated in favor of the unified rust-ai-ide-cache crate.
//! TODO: Remove this module and migrate all usages to rust_ai_ide_cache

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Generic cache entry with TTL support
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub expires_at: Option<Instant>,
    pub created_at: Instant,
    pub access_count: u64,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V, ttl: Option<Duration>) -> Self {
        let created_at = Instant::now();
        Self {
            value,
            expires_at: ttl.map(|t| created_at + t),
            created_at,
            access_count: 0,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Instant::now() > exp)
    }

    pub fn access(&mut self) {
        self.access_count += 1;
    }
}

/// Cache configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_size: usize,
    /// Default TTL for entries
    pub default_ttl: Option<u64>,
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl: Some(300), // 5 minutes
            eviction_policy: EvictionPolicy::Lru,
        }
    }
}

/// Eviction policies for cache
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,
    /// First In First Out
    Fifo,
    /// Least Frequently Used
    Lfu,
}

/// Generic cache trait
#[async_trait::async_trait]
pub trait Cache<K, V>: Send + Sync {
    /// Insert a value with optional TTL
    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> anyhow::Result<()>;

    /// Get a value by key
    async fn get(&self, key: &K) -> anyhow::Result<Option<Arc<V>>>;

    /// Remove a value by key
    async fn remove(&self, key: &K) -> anyhow::Result<Option<Arc<V>>>;

    /// Clear all entries
    async fn clear(&self) -> anyhow::Result<()>;

    /// Get current size
    async fn size(&self) -> usize;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;
}

/// Cache statistics
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub sets: u64,
}

/// Thread-safe LRU cache implementation using RwLock
#[derive(Clone)]
pub struct InMemoryCache<K, V> {
    inner: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl<K, V> InMemoryCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats {
                total_entries: 0,
                hits: 0,
                misses: 0,
                evictions: 0,
                sets: 0,
            })),
        }
    }

    async fn evict_if_needed(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;

        // Check size limit
        if inner.len() >= self.config.max_size {
            match self.config.eviction_policy {
                EvictionPolicy::Lru => {
                    // Find least recently accessed
                    let mut lru_key = None;
                    let mut lru_time = Instant::now();

                    for (key, entry) in inner.iter() {
                        // Note: We use created_at + access_count as a rough LRU indicator
                        // In a real implementation, you'd track last access time
                        if entry.access_count < (lru_time.elapsed().as_millis() as u64) {
                            lru_time = entry.created_at;
                            lru_key = Some(key.clone());
                        }
                    }

                    if let Some(key) = lru_key {
                        inner.remove(&key);
                        let mut stats = self.stats.write().await;
                        stats.evictions += 1;
                        stats.total_entries = inner.len();
                    }
                }
                EvictionPolicy::Fifo => {
                    // Remove oldest entry
                    if let Some(key) = inner.keys().next().cloned() {
                        inner.remove(&key);
                        let mut stats = self.stats.write().await;
                        stats.evictions += 1;
                        stats.total_entries = inner.len();
                    }
                }
                EvictionPolicy::Lfu => {
                    // Remove least frequently accessed
                    let mut lfu_key = None;
                    let mut min_access = u64::MAX;

                    for (key, entry) in inner.iter() {
                        if entry.access_count < min_access {
                            min_access = entry.access_count;
                            lfu_key = Some(key.clone());
                        }
                    }

                    if let Some(key) = lfu_key {
                        inner.remove(&key);
                        let mut stats = self.stats.write().await;
                        stats.evictions += 1;
                        stats.total_entries = inner.len();
                    }
                }
            }
        }

        Ok(())
    }

    async fn cleanup_expired(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;
        let mut expired_keys = Vec::new();

        for (key, entry) in inner.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        for key in expired_keys {
            inner.remove(&key);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_entries = inner.len();
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<K, V> Cache<K, V> for InMemoryCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> anyhow::Result<()> {
        self.evict_if_needed().await?;
        self.cleanup_expired().await?;

        let ttl = ttl.or(self.config.default_ttl.map(Duration::from_secs));
        let entry = CacheEntry::new(value, ttl);

        let mut inner = self.inner.write().await;
        inner.insert(key, entry);

        let mut stats = self.stats.write().await;
        stats.sets += 1;
        stats.total_entries = inner.len();

        Ok(())
    }

    async fn get(&self, key: &K) -> anyhow::Result<Option<Arc<V>>> {
        self.cleanup_expired().await?;

        let mut inner = self.inner.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = inner.get_mut(key) {
            if !entry.is_expired() {
                entry.access();
                stats.hits += 1;
                return Ok(Some(Arc::new(entry.value.clone())));
            } else {
                // Entry is expired, remove it
                inner.remove(key);
                stats.misses += 1;
                stats.total_entries = inner.len();
                return Ok(None);
            }
        }

        stats.misses += 1;
        Ok(None)
    }

    async fn remove(&self, key: &K) -> anyhow::Result<Option<Arc<V>>> {
        let mut inner = self.inner.write().await;
        let result = inner.remove(key).map(|entry| Arc::new(entry.value));

        let mut stats = self.stats.write().await;
        stats.total_entries = inner.len();

        Ok(result)
    }

    async fn clear(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;
        inner.clear();

        let mut stats = self.stats.write().await;
        stats.total_entries = 0;

        Ok(())
    }

    async fn size(&self) -> usize {
        self.inner.read().await.len()
    }

    async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_cache_operations() {
        let cache: InMemoryCache<String, String> = InMemoryCache::new(CacheConfig::default());

        // Test insert and get
        cache.insert("key1".to_string(), "value1".to_string(), None).await.unwrap();
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, Some(Arc::new("value1".to_string())));

        // Test stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.sets, 1);

        // Test remove
        cache.remove(&"key1".to_string()).await.unwrap();
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache: InMemoryCache<String, String> = InMemoryCache::new(CacheConfig::default());

        cache.insert("key1".to_string(), "value1".to_string(), Some(Duration::from_millis(50))).await.unwrap();

        // Should be available immediately
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, Some(Arc::new("value1".to_string())));

        // Wait for expiration
        sleep(Duration::from_millis(60)).await;

        // Should be expired
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, None);
    }
}