use dashmap::{mapref::one::Ref, DashMap};
use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
///! Caching infrastructure for the Rust AI IDE
///!
///! Provides a unified caching system with TTL-based expiration,
///! thread safety, and hit rate monitoring.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Cache statistics for monitoring performance
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
}

impl CacheStats {
    /// Calculate hit rate as hits / (hits + misses)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Cache entry with expiration time
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    value: V,
    expires_at: Option<Instant>,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|ttl| Instant::now() + ttl);
        Self { value, expires_at }
    }

    fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| Instant::now() > exp)
    }
}

/// Thread-safe cache trait
#[async_trait::async_trait]
pub trait Cache<K, V>: Send + Sync + std::fmt::Debug {
    /// Get a value from the cache
    async fn get(&self, key: &K) -> Option<V>
    where
        K: Clone,
        V: Clone;

    /// Insert a value into the cache with optional TTL
    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> bool;

    /// Remove a value from the cache
    async fn remove(&self, key: &K) -> Option<V>;

    /// Clear the entire cache
    async fn clear(&self);

    /// Check if a key exists (may not consider expiry)
    async fn contains_key(&self, key: &K) -> bool;

    /// Get cache statistics
    fn stats(&self) -> CacheStats;

    /// Get the number of entries in the cache
    fn len(&self) -> usize;

    /// Check if the cache is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory cache implementation with TTL support
#[derive(Debug)]
pub struct MemoryCache<K, V>
where
    K: Eq + std::hash::Hash + std::fmt::Debug,
    V: std::fmt::Debug,
{
    storage: Arc<DashMap<K, CacheEntry<V>>>,
    stats: Arc<std::sync::RwLock<CacheStats>>,
}

impl<K, V> MemoryCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug,
    V: Clone + Send + Sync + std::fmt::Debug,
{
    /// Create a new in-memory cache
    pub fn new() -> Self {
        Self {
            storage: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Create a new in-memory cache with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: Arc::new(DashMap::with_capacity(capacity)),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get a reference to the entry if it exists and is not expired
    fn get_entry(&self, key: &K) -> Option<Ref<'_, K, CacheEntry<V>>> {
        if let Some(entry) = self.storage.get(key) {
            if entry.is_expired() {
                self.storage.remove(key);
                self.increment_evictions();
                None
            } else {
                Some(entry)
            }
        } else {
            None
        }
    }

    fn increment_hits(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.hits += 1;
        } else {
            warn!("Failed to acquire stats write lock for hit increment");
        }
    }

    fn increment_misses(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.misses += 1;
        } else {
            warn!("Failed to acquire stats write lock for miss increment");
        }
    }

    fn increment_evictions(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.evictions += 1;
        } else {
            warn!("Failed to acquire stats write lock for eviction increment");
        }
    }

    fn increment_insertions(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.insertions += 1;
        } else {
            warn!("Failed to acquire stats write lock for insertion increment");
        }
    }
}

#[async_trait::async_trait]
impl<K, V> Cache<K, V> for MemoryCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug,
    V: Clone + Send + Sync + std::fmt::Debug,
{
    async fn get(&self, key: &K) -> Option<V> {
        if let Some(entry) = self.get_entry(key) {
            self.increment_hits();
            Some(entry.value.clone())
        } else {
            self.increment_misses();
            None
        }
    }

    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> bool {
        let entry = CacheEntry::new(value, ttl);
        if self.storage.insert(key, entry).is_some() {
            self.increment_evictions(); // Existing entry replaced
            false
        } else {
            self.increment_insertions();
            true
        }
    }

    async fn remove(&self, key: &K) -> Option<V> {
        self.storage.remove(key).map(|(_, entry)| entry.value)
    }

    async fn clear(&self) {
        self.storage.clear();
    }

    async fn contains_key(&self, key: &K) -> bool {
        self.get_entry(key).is_some()
    }

    fn stats(&self) -> CacheStats {
        self.stats
            .read()
            .expect("Failed to acquire stats read lock - cache may be corrupted")
            .clone()
    }

    fn len(&self) -> usize {
        self.storage.len()
    }
}

impl<K, V> Default for MemoryCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug,
    V: Clone + Send + Sync + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}
