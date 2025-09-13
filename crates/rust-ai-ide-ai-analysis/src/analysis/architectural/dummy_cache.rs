//! Dummy cache implementation for when caching is disabled

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

/// Dummy cache entry that doesn't actually cache anything
#[derive(Debug, Clone)]
pub struct DummyCacheEntry<T> {
    value:       T,
    _expires_at: Option<std::time::Instant>,
}

impl<T> DummyCacheEntry<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            _expires_at: None,
        }
    }

    pub fn with_ttl(value: T, _ttl: std::time::Duration) -> Self {
        Self {
            value,
            _expires_at: None, // Still don't actually track expiration
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

/// Dummy cache that doesn't actually cache anything
#[derive(Debug, Default)]
pub struct DummyCache<K, V> {
    inner: RwLock<HashMap<K, V>>,
}

impl<K, V> DummyCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, _key: &K) -> Option<V> {
        None // Always return None to indicate cache miss
    }

    pub fn insert(&self, _key: K, _value: V) {
        // Don't actually cache anything
    }

    pub fn remove(&self, _key: &K) -> Option<V> {
        None
    }

    pub fn clear(&self) {
        // No-op
    }
}

impl<K, V> Default for DummyCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
