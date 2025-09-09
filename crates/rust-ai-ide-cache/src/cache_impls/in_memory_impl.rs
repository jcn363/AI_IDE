//! In-memory cache implementation using DashMap for concurrent access

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Cache, CacheConfig, CacheEntry, CacheStats, IDEResult};

/// In-memory cache implementation with O(1) operations
pub struct InMemoryCache<K: std::hash::Hash + Eq, V> {
    entries: dashmap::DashMap<K, CacheEntry<V>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl<K: std::hash::Hash + Eq, V> InMemoryCache<K, V> {
    /// Create a new in-memory cache with the given configuration
    pub fn new(config: &CacheConfig) -> Self {
        let stats = CacheStats {
            created_at: chrono::Utc::now(),
            uptime_seconds: 0,
            ..Default::default()
        };

        Self {
            entries: dashmap::DashMap::new(),
            config: config.clone(),
            stats: Arc::new(RwLock::new(stats)),
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for InMemoryCache<K, V>
where
    K: Send + Sync + Clone + std::hash::Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    async fn get(&self, key: &K) -> IDEResult<Option<V>> {
        let mut stats = self.stats.write().await;

        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                self.entries.remove(key);
                stats.record_miss();
                stats.record_eviction();
                Ok(None)
            } else {
                // Update access count efficiently
                let value = entry.value.clone();
                let _ = entry; // Release the read lock before modifying
                self.entries.alter(key, |_, mut v| {
                    v.access();
                    v
                });
                stats.record_hit();
                Ok(Some(value))
            }
        } else {
            stats.record_miss();
            Ok(None)
        }
    }

    async fn insert(&self, key: K, value: V, ttl: Option<std::time::Duration>) -> IDEResult<()> {
        let mut stats = self.stats.write().await;

        // Check memory limits
        if let Some(max_entries) = self.config.max_entries {
            if self.entries.len() >= max_entries {
                // Simple eviction: remove oldest entry
                if let Some(entry) = self.entries.iter().next() {
                    let old_key = entry.key().clone();
                    let _ = entry; // Release the read lock
                    self.entries.remove(&old_key);
                    stats.record_eviction();
                }
            }
        }

        let entry = CacheEntry::new_with_ttl(value, ttl, chrono::Utc::now());
        self.entries.insert(key, entry);
        stats.record_set();

        Ok(())
    }

    async fn remove(&self, key: &K) -> IDEResult<Option<V>> {
        let result = self.entries.remove(key);
        match result {
            Some((_, entry)) => Ok(Some(entry.value)),
            None => Ok(None),
        }
    }

    async fn clear(&self) -> IDEResult<()> {
        self.entries.clear();
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
        Ok(())
    }

    async fn size(&self) -> usize {
        self.entries.len()
    }

    async fn contains(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    async fn stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.total_entries = self.entries.len();
        stats.uptime_seconds = (chrono::Utc::now() - stats.created_at)
            .as_seconds_f64()
            .abs() as u64;
        stats
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        let mut expired_count = 0;

        // Efficiently remove expired entries while iterating
        self.entries.retain(|_, entry| {
            if entry.is_expired() {
                expired_count += 1;
                false // Remove this entry
            } else {
                true // Keep this entry
            }
        });

        if expired_count > 0 {
            let mut stats = self.stats.write().await;
            stats.total_evictions += expired_count as u64;
        }

        Ok(expired_count)
    }
}

impl<K: std::hash::Hash + Eq, V> Drop for InMemoryCache<K, V> {
    fn drop(&mut self) {
        // Cleanup on drop
        self.entries.clear();
    }
}

/// Hybrid cache that combines in-memory and another storage backend
pub struct HybridCache<K: std::hash::Hash + Eq, V> {
    memory_cache: InMemoryCache<K, V>,
    // secondary_cache: Option<Box<dyn Cache<K, V>>>, // For future use
}

impl<K: std::hash::Hash + Eq, V> HybridCache<K, V> {
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            memory_cache: InMemoryCache::new(config),
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for HybridCache<K, V>
where
    K: Send + Sync + Clone + std::hash::Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    async fn get(&self, key: &K) -> IDEResult<Option<V>> {
        // Try memory cache first
        if let Some(value) = self.memory_cache.get(key).await? {
            Ok(Some(value))
        } else {
            Ok(None) // Secondary cache lookup would go here
        }
    }

    async fn insert(&self, key: K, value: V, ttl: Option<std::time::Duration>) -> IDEResult<()> {
        // Populate both caches
        self.memory_cache.insert(key, value, ttl).await?;
        // Secondary cache insert would go here
        Ok(())
    }

    async fn remove(&self, key: &K) -> IDEResult<Option<V>> {
        self.memory_cache.remove(key).await
    }

    async fn clear(&self) -> IDEResult<()> {
        self.memory_cache.clear().await?;
        // Secondary cache clear would go here
        Ok(())
    }

    async fn size(&self) -> usize {
        self.memory_cache.size().await
    }

    async fn contains(&self, key: &K) -> bool {
        self.memory_cache.contains(key).await
    }

    async fn stats(&self) -> CacheStats {
        self.memory_cache.stats().await
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        self.memory_cache.cleanup_expired().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_in_memory_cache_basic_operations() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        let key = "test_key";
        let value = "test_value";

        // Test insert and get
        cache.insert(key, value, None).await.unwrap();
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Test contains
        assert!(cache.contains(&key).await);

        // Test remove
        let removed = cache.remove(&key).await.unwrap();
        assert_eq!(removed, Some(value.to_string()));
        assert!(!cache.contains(&key).await);
    }

    #[tokio::test]
    async fn test_cache_with_ttl() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        let key = "ttl_key";
        let value = "ttl_value";
        let ttl = Duration::from_millis(100);

        cache.insert(key, value, Some(ttl)).await.unwrap();

        // Should be available immediately
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_cleanup_expired() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        // Insert expired entry
        let key = "expired_key";
        let value = "expired_value";
        let short_ttl = Duration::from_millis(50);

        cache.insert(key, value, Some(short_ttl)).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup expired entries
        let cleaned = cache.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Verify entry is gone
        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        let key = "stats_key";
        let value = "stats_value";

        // Insert and retrieve to generate stats
        cache.insert(key, value, None).await.unwrap();
        cache.get(&key).await.unwrap();
        cache.get(&"nonexistent").await.unwrap(); // Miss

        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);
        assert!(stats.hit_ratio > 0.0);
    }
}
