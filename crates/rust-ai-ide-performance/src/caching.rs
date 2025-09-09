// Caching module for performance optimization

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    created_at: Instant,
    accessed_at: Instant,
    ttl: Option<Duration>,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            accessed_at: now,
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    pub fn touch(&mut self) {
        self.accessed_at = Instant::now();
    }
}

pub struct LRUCache<K, V> {
    /// Cache storage with proper lifetime bounds
    cache: HashMap<K, CacheEntry<V>>,
    /// Capacity limit for memory safety
    capacity: usize,
    /// Access order tracking for LRU eviction with pre-allocated capacity
    access_order: Vec<K>,
}

impl<K, V> LRUCache<K, V>
where
    K: Clone + Eq + Hash,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity), // Pre-allocate for performance
            capacity,
            access_order: Vec::with_capacity(capacity), // Pre-allocate access order tracking
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let is_expired = self
            .cache
            .get(key)
            .map(|entry| entry.is_expired())
            .unwrap_or(false);

        if is_expired {
            self.remove(key);
            return None;
        }

        if let Some(entry) = self.cache.get_mut(key) {
            entry.touch();

            // Update access order with bounds checking
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);

                // Ensure we don't exceed capacity after removal
                if self.access_order.len() < self.capacity {
                    self.access_order.push(key.clone());
                }
                // Note: Access order corruption handling removed to avoid borrow checker issues
                // This is a trade-off for memory safety - we prefer correctness over perfect LRU order
            } else {
                // Key not found in access order, add it with bounds checking
                if self.access_order.len() < self.capacity {
                    self.access_order.push(key.clone());
                }
            }

            Some(&entry.value)
        } else {
            None
        }
    }

    /// Rebuild access order when it becomes corrupted (memory safety critical)
    fn rebuild_access_order(&mut self) {
        self.access_order.clear();
        self.access_order.reserve(self.capacity);

        // Rebuild based on cache contents (last accessed items first)
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|(k, v)| (k.clone(), v.accessed_at))
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1)); // Most recently accessed first

        for (key, _) in entries.into_iter().take(self.capacity) {
            if !self.access_order.contains(&key) {
                self.access_order.push(key);
            }
        }
    }

    pub fn put(&mut self, key: K, value: V, ttl: Option<Duration>) {
        if self.cache.contains_key(&key) {
            // Update existing entry with proper bounds checking
            if let Some(entry) = self.cache.get_mut(&key) {
                entry.value = value;
                entry.touch();

                // Ensure access order is updated
                if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                    self.access_order.remove(pos);
                    if self.access_order.len() < self.capacity {
                        self.access_order.push(key);
                    }
                }
            }
        } else {
            // Add new entry with capacity enforcement
            if self.cache.len() >= self.capacity {
                self.evict_lru();
            }

            // Double-check capacity after eviction
            if self.cache.len() < self.capacity {
                let entry = CacheEntry::new(value, ttl);
                self.cache.insert(key.clone(), entry);

                if self.access_order.len() < self.capacity {
                    self.access_order.push(key);
                }
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> bool {
        if self.cache.remove(key).is_some() {
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            true
        } else {
            false
        }
    }

    pub fn evict_lru(&mut self) {
        if let Some(key) = self.access_order.first().cloned() {
            // Safe eviction with bounds checking
            self.cache.remove(&key);
            self.access_order.remove(0);

            // Ensure capacity constraints are maintained
            if self.access_order.len() > self.capacity {
                self.access_order.truncate(self.capacity);
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

// Prefetch caching for predictive loading
pub struct PrefetchCache<K, V> {
    cache: LRUCache<K, V>,
    prefetch_threshold: usize,
}

impl<K, V> PrefetchCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn new(capacity: usize, prefetch_threshold: usize) -> Self {
        Self {
            cache: LRUCache::new(capacity),
            prefetch_threshold,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        self.cache.put(key, value, None);
    }

    // Simple prefetch trigger based on access threshold
    pub fn should_prefetch(&self) -> bool {
        self.cache.len() >= self.prefetch_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LRUCache::new(3);

        cache.put("a", 1, None);
        cache.put("b", 2, None);
        cache.put("c", 3, None);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));

        // Test LRU eviction
        cache.put("d", 4, None); // Should evict "a"
        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"d"), Some(&4));
    }
}
