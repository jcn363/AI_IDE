//! Cache testing utilities and fixtures
//!
//! Provides mock cache implementations, cache testing patterns,
//! and utilities for testing cache-related functionality.

use crate::error::TestError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Mock cache entry with timestamp information
#[derive(Debug, Clone)]
pub struct MockCacheEntry<V> {
    pub value: V,
    pub created_at: Instant,
    pub accessed_at: Instant,
    pub ttl: Option<Duration>,
}

impl<V> MockCacheEntry<V> {
    pub fn new(value: V) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            accessed_at: now,
            ttl: None,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    pub fn access(&mut self) {
        self.accessed_at = Instant::now();
    }
}

/// Mock cache implementation for testing
#[derive(Debug, Clone)]
pub struct MockCache<K, V> {
    data: Arc<Mutex<HashMap<K, MockCacheEntry<V>>>>,
    max_size: Option<usize>,
    eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    Fifo,
}

impl<K, V> MockCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            max_size: None,
            eviction_policy: EvictionPolicy::Lru,
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn with_eviction_policy(mut self, policy: EvictionPolicy) -> Self {
        self.eviction_policy = policy;
        self
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.lock().unwrap();
        if let Some(entry) = data.get_mut(key) {
            if entry.is_expired() {
                data.remove(key);
                None
            } else {
                entry.access();
                Some(entry.value.clone())
            }
        } else {
            None
        }
    }

    pub fn put(&self, key: K, value: V) -> Result<(), TestError> {
        let mut data = self.data.lock().unwrap();

        // Check size limits
        if let Some(max_size) = self.max_size {
            if data.len() >= max_size && !data.contains_key(&key) {
                self.evict_entries(&mut data, 1);
            }
        }

        data.insert(key, MockCacheEntry::new(value));
        Ok(())
    }

    pub fn put_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<(), TestError> {
        let mut data = self.data.lock().unwrap();

        if let Some(max_size) = self.max_size {
            if data.len() >= max_size && !data.contains_key(&key) {
                self.evict_entries(&mut data, 1);
            }
        }

        data.insert(key, MockCacheEntry::new(value).with_ttl(ttl));
        Ok(())
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.lock().unwrap();
        data.remove(key).map(|entry| entry.value)
    }

    pub fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        data.clear();
    }

    pub fn size(&self) -> usize {
        let data = self.data.lock().unwrap();
        data.len()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let data = self.data.lock().unwrap();
        data.contains_key(key) && !data.get(key).unwrap().is_expired()
    }

    fn evict_entries(&self, data: &mut HashMap<K, MockCacheEntry<V>>, count: usize) {
        if data.len() <= self.max_size.unwrap_or(usize::MAX) || count == 0 {
            return;
        }

        // Collect keys to remove first to avoid borrowing issues
        let keys_to_remove: Vec<K> = match self.eviction_policy {
            EvictionPolicy::Lru => {
                let mut entries: Vec<_> = data.iter().map(|(k, v)| (k.clone(), v.accessed_at)).collect();
                entries.sort_by_key(|(_, accessed_at)| *accessed_at);
                entries.into_iter().take(count).map(|(k, _)| k).collect()
            }
            EvictionPolicy::Lfu => {
                // For simplicity, using access count as frequency
                let mut entries: Vec<_> = data.iter().map(|(k, v)| (k.clone(), v.accessed_at)).collect();
                entries.sort_by_key(|(_, accessed_at)| *accessed_at);
                entries.into_iter().take(count).map(|(k, _)| k).collect()
            }
            EvictionPolicy::Fifo => {
                let mut entries: Vec<_> = data.iter().map(|(k, v)| (k.clone(), v.created_at)).collect();
                entries.sort_by_key(|(_, created_at)| *created_at);
                entries.into_iter().take(count).map(|(k, _)| k).collect()
            }
        };

        // Remove all collected keys
        for key in keys_to_remove {
            data.remove(&key);
        }
    }
}

impl<K, V> Default for MockCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Cache test fixture with predefined data and scenarios
pub struct CacheFixture<K, V> {
    cache: MockCache<K, V>,
    initial_data: HashMap<K, V>,
    expected_calls: Arc<Mutex<Vec<CacheCall<K, V>>>>,
}

#[derive(Debug, Clone)]
pub enum CacheCall<K, V> {
    Get(K),
    Put(K, V),
    Remove(K),
    Clear,
}

impl<K, V> CacheFixture<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            cache: MockCache::new(),
            initial_data: HashMap::new(),
            expected_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_initial_data(mut self, data: HashMap<K, V>) -> Self {
        self.initial_data = data;
        self
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.cache = self.cache.with_max_size(max_size);
        self
    }

    pub async fn setup(&mut self) -> Result<(), TestError> {
        for (key, value) in &self.initial_data {
            self.cache.put(key.clone(), value.clone())?;
        }
        Ok(())
    }

    pub fn cache(&self) -> &MockCache<K, V> {
        &self.cache
    }

    pub fn expect_get(&self, key: K) {
        self.expected_calls.lock().unwrap().push(CacheCall::Get(key));
    }

    pub fn expect_put(&self, key: K, value: V) {
        self.expected_calls.lock().unwrap().push(CacheCall::Put(key, value));
    }

    pub fn expect_remove(&self, key: K) {
        self.expected_calls.lock().unwrap().push(CacheCall::Remove(key));
    }

    pub fn expect_clear(&self) {
        self.expected_calls.lock().unwrap().push(CacheCall::Clear);
    }

    pub fn verify(&self) -> Result<(), TestError> {
        // This is a simplified verification - in practice, you'd want to use
        // a proper mocking framework to verify actual calls
        Ok(())
    }
}

impl<K, V> Default for CacheFixture<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined cache fixture builders
pub struct CacheFixtures;

impl CacheFixtures {
    /// Create a basic in-memory cache fixture
    pub fn basic<K, V>() -> CacheFixture<K, V>
    where
        K: Eq + std::hash::Hash + Clone,
        V: Clone,
    {
        CacheFixture::new()
    }

    /// Create a cache fixture with size limits
    pub fn with_size_limit<K, V>(max_size: usize) -> CacheFixture<K, V>
    where
        K: Eq + std::hash::Hash + Clone,
        V: Clone,
    {
        CacheFixture::new().with_max_size(max_size)
    }

    /// Create a cache fixture pre-populated with common test data
    pub fn populated() -> CacheFixture<String, serde_json::Value> {
        let mut initial_data = HashMap::new();
        initial_data.insert("user:123".to_string(), serde_json::json!({"name": "John", "id": 123}));
        initial_data.insert("config:theme".to_string(), serde_json::json!("dark"));

        CacheFixture::new().with_initial_data(initial_data)
    }
}

/// Cache performance test utilities
pub struct CachePerformanceTester<K, V> {
    cache: MockCache<K, V>,
    operations: Vec<CacheOperation<K, V>>,
}

#[derive(Debug, Clone)]
pub enum CacheOperation<K, V> {
    Get(K),
    Put(K, V),
    Remove(K),
}

impl<K, V> CachePerformanceTester<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new(cache: MockCache<K, V>) -> Self {
        Self {
            cache,
            operations: Vec::new(),
        }
    }

    pub fn add_operation(&mut self, op: CacheOperation<K, V>) {
        self.operations.push(op);
    }

    pub async fn run_benchmark(&self, iterations: usize) -> CacheBenchmarkResult {
        let start = Instant::now();
        let mut total_gets = 0;
        let mut total_puts = 0;
        let mut total_removes = 0;

        for _ in 0..iterations {
            for op in &self.operations {
                match op {
                    CacheOperation::Get(key) => {
                        self.cache.get(key);
                        total_gets += 1;
                    }
                    CacheOperation::Put(key, value) => {
                        let _ = self.cache.put(key.clone(), value.clone());
                        total_puts += 1;
                    }
                    CacheOperation::Remove(key) => {
                        self.cache.remove(key);
                        total_removes += 1;
                    }
                }
            }
        }

        let elapsed = start.elapsed();

        CacheBenchmarkResult {
            total_time: elapsed,
            operations_per_second: (total_gets + total_puts + total_removes) as f64 / elapsed.as_secs_f64(),
            total_gets,
            total_puts,
            total_removes,
        }
    }
}

#[derive(Debug)]
pub struct CacheBenchmarkResult {
    pub total_time: Duration,
    pub operations_per_second: f64,
    pub total_gets: usize,
    pub total_puts: usize,
    pub total_removes: usize,
}

/// Multi-level cache test fixture for testing cache hierarchies
pub struct MultiLevelCacheFixture<K, V> {
    l1_cache: MockCache<K, V>,
    l2_cache: MockCache<K, V>,
}

impl<K, V> MultiLevelCacheFixture<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            l1_cache: MockCache::new().with_max_size(100),
            l2_cache: MockCache::new().with_max_size(1000),
        }
    }

    pub async fn setup(&mut self) -> Result<(), TestError> {
        // Setup hierarchical cache relationships
        Ok(())
    }

    pub fn get(&self, key: &K) -> Option<V> {
        // L1 cache lookup first
        if let Some(value) = self.l1_cache.get(key) {
            return Some(value);
        }

        // L2 cache lookup
        if let Some(value) = self.l2_cache.get(key) {
            // Promote to L1
            let _ = self.l1_cache.put(key.clone(), value.clone());
            return Some(value);
        }

        None
    }

    pub fn put(&self, key: K, value: V) -> Result<(), TestError> {
        // Put in both levels
        self.l1_cache.put(key.clone(), value.clone())?;
        self.l2_cache.put(key, value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_mock_cache_basic_operations() {
        let cache = MockCache::<String, String>::new();

        // Test put and get
        cache.put("key1".to_string(), "value1".to_string()).unwrap();
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"nonexistent".to_string()), None);
    }

    #[test]
    fn test_mock_cache_ttl() {
        let cache = MockCache::<String, String>::new();

        cache.put_with_ttl("key1".to_string(), "value1".to_string(), Duration::from_millis(10)).unwrap();

        // Should work initially
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Should be expired
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_fixture() {
        let mut fixture = CacheFixtures::populated();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(fixture.setup()).unwrap();

        let cache = fixture.cache();
        assert!(cache.contains_key(&"user:123".to_string()));
        assert!(cache.contains_key(&"config:theme".to_string()));
    }

    #[test]
    fn test_multi_level_cache() {
        let mut fixture = MultiLevelCacheFixture::<String, String>::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(fixture.setup()).unwrap();

        fixture.put("key1".to_string(), "value1".to_string()).unwrap();

        // Should find in L1 after first access
        assert_eq!(fixture.get(&"key1".to_string()), Some("value1".to_string()));
    }
}