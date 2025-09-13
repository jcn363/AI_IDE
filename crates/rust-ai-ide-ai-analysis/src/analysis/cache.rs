//! Caching capabilities for analysis results

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Analysis cache for storing computed results
#[derive(Debug)]
pub struct AnalysisCache {
    inner:    Arc<RwLock<HashMap<String, String>>>,
    max_size: usize,
}

impl AnalysisCache {
    /// Create a new analysis cache
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    /// Get a cached result
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.read().unwrap().get(key).cloned()
    }

    /// Store a result in cache
    pub fn put(&self, key: String, value: String) {
        let mut cache = self.inner.write().unwrap();

        // If at capacity, remove oldest entry
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            // For simplicity, remove first entry
            let oldest_key = cache.keys().next().cloned();
            if let Some(key_to_remove) = oldest_key {
                cache.remove(&key_to_remove);
            }
        }

        cache.insert(key, value);
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Create a cache key for analysis results
pub fn create_cache_key(path: &str, analysis_type: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{}:{}", path, analysis_type).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
