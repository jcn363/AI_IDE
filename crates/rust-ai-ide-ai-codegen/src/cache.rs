//! Caching system for AI code generation

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use moka::future::Cache;
use tokio::sync::Mutex;

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// Cached data
    pub data:         T,
    /// Creation timestamp
    pub created_at:   chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub accessed_at:  chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// Time-to-live in seconds
    pub ttl_seconds:  Option<u64>,
}

/// Cache key for generated code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeCacheKey {
    pub spec_hash:    String,
    pub language:     String,
    pub context_hash: String,
}

/// Cache key for completion suggestions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompletionCacheKey {
    pub prefix_hash:  String,
    pub suffix_hash:  String,
    pub language:     String,
    pub context_hash: String,
}

/// Main cache system for code generation
pub struct CodegenCache {
    /// Cache for generated code
    code_cache:       Cache<CodeCacheKey, crate::types::GeneratedCode>,
    /// Cache for completion suggestions
    completion_cache: Cache<CompletionCacheKey, Vec<crate::types::CompletionSuggestion>>,
    /// Statistics for monitoring
    stats:            Arc<Mutex<CacheStats>>,
}

impl CodegenCache {
    /// Create a new cache system
    pub fn new() -> Self {
        let code_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(std::time::Duration::from_secs(3600)) // 1 hour TTL
            .build();

        let completion_cache = Cache::builder()
            .max_capacity(5000)
            .time_to_live(std::time::Duration::from_secs(1800)) // 30 minutes TTL
            .build();

        Self {
            code_cache,
            completion_cache,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        }
    }

    /// Get cached generated code
    pub async fn get(&self, spec: &str) -> Option<crate::types::GeneratedCode> {
        let key = Self::create_code_key(spec);
        let result = self.code_cache.get(&key).await;

        if result.is_some() {
            let mut stats = self.stats.lock().await;
            stats.code_cache_hits += 1;
        } else {
            let mut stats = self.stats.lock().await;
            stats.code_cache_misses += 1;
        }

        result
    }

    /// Put generated code in cache
    pub async fn put(&self, spec: String, code: crate::types::GeneratedCode) {
        let key = Self::create_code_key(&spec);
        self.code_cache.insert(key, code).await;

        let mut stats = self.stats.lock().await;
        stats.code_cache_inserts += 1;
    }

    /// Get cached completion suggestions
    pub async fn get_completion_suggestions(&self, key_str: &str) -> Option<Vec<crate::types::CompletionSuggestion>> {
        let key = Self::create_completion_key(key_str);
        let result = self.completion_cache.get(&key).await;

        if result.is_some() {
            let mut stats = self.stats.lock().await;
            stats.completion_cache_hits += 1;
        } else {
            let mut stats = self.stats.lock().await;
            stats.completion_cache_misses += 1;
        }

        result
    }

    /// Put completion suggestions in cache
    pub async fn put_completion_suggestions(
        &self,
        key_str: String,
        suggestions: Vec<crate::types::CompletionSuggestion>,
    ) {
        let key = Self::create_completion_key(&key_str);
        self.completion_cache.insert(key, suggestions).await;

        let mut stats = self.stats.lock().await;
        stats.completion_cache_inserts += 1;
    }

    /// Clear all caches
    pub async fn clear(&self) {
        self.code_cache.invalidate_all();
        self.completion_cache.invalidate_all();

        let mut stats = self.stats.lock().await;
        stats.clears += 1;
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.lock().await.clone()
    }

    /// Create cache key for code generation
    fn create_code_key(spec: &str) -> CodeCacheKey {
        use std::hash::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        spec.hash(&mut hasher);
        let spec_hash = format!("{:x}", hasher.finish());

        CodeCacheKey {
            spec_hash,
            language: "rust".to_string(),        // Default to Rust, should be parameterized
            context_hash: "default".to_string(), // Should include actual context
        }
    }

    /// Create cache key for completion suggestions
    fn create_completion_key(key_str: &str) -> CompletionCacheKey {
        use std::hash::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        key_str.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());

        CompletionCacheKey {
            prefix_hash:  hash.clone(),
            suffix_hash:  hash.clone(),
            language:     "rust".to_string(), // Default to Rust, should be parameterized
            context_hash: "default".to_string(), // Should include actual context
        }
    }

    /// Warm up cache with common patterns
    pub async fn warmup(&self) -> crate::error::Result<()> {
        // This would be implemented with common code generation patterns
        // For now, it's a placeholder
        Ok(())
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Code cache hits
    pub code_cache_hits:          u64,
    /// Code cache misses
    pub code_cache_misses:        u64,
    /// Code cache inserts
    pub code_cache_inserts:       u64,
    /// Completion cache hits
    pub completion_cache_hits:    u64,
    /// Completion cache misses
    pub completion_cache_misses:  u64,
    /// Completion cache inserts
    pub completion_cache_inserts: u64,
    /// Number of cache clears
    pub clears:                   u64,
}

impl CacheStats {
    /// Calculate hit rate for code cache
    pub fn code_cache_hit_rate(&self) -> f64 {
        let total = self.code_cache_hits + self.code_cache_misses;
        if total == 0 {
            0.0
        } else {
            self.code_cache_hits as f64 / total as f64
        }
    }

    /// Calculate hit rate for completion cache
    pub fn completion_cache_hit_rate(&self) -> f64 {
        let total = self.completion_cache_hits + self.completion_cache_misses;
        if total == 0 {
            0.0
        } else {
            self.completion_cache_hits as f64 / total as f64
        }
    }

    /// Get total cache operations
    pub fn total_operations(&self) -> u64 {
        self.code_cache_hits
            + self.code_cache_misses
            + self.code_cache_inserts
            + self.completion_cache_hits
            + self.completion_cache_misses
            + self.completion_cache_inserts
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum capacity for code cache
    pub code_cache_capacity:          u64,
    /// Time-to-live for code cache entries (seconds)
    pub code_cache_ttl_seconds:       u64,
    /// Maximum capacity for completion cache
    pub completion_cache_capacity:    u64,
    /// Time-to-live for completion cache entries (seconds)
    pub completion_cache_ttl_seconds: u64,
    /// Enable statistics collection
    pub enable_stats:                 bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            code_cache_capacity:          1000,
            code_cache_ttl_seconds:       3600, // 1 hour
            completion_cache_capacity:    5000,
            completion_cache_ttl_seconds: 1800, // 30 minutes
            enable_stats:                 true,
        }
    }
}

/// Smart cache with eviction policies
pub struct SmartCache<T: Clone + Send + Sync> {
    /// Underlying cache
    cache:  Cache<String, CacheEntry<T>>,
    /// Configuration
    config: CacheConfig,
    /// Statistics
    stats:  Arc<Mutex<CacheStats>>,
}

impl<T: Clone + Send + Sync + 'static> SmartCache<T> {
    /// Create a new smart cache
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.code_cache_capacity)
            .time_to_live(std::time::Duration::from_secs(
                config.code_cache_ttl_seconds,
            ))
            .build();

        Self {
            cache,
            config,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        }
    }

    /// Get an item from the cache
    pub async fn get(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.cache.get(key).await {
            // Update access statistics
            if self.config.enable_stats {
                let mut stats = self.stats.lock().await;
                stats.code_cache_hits += 1;
            }

            // Check if entry has expired based on TTL
            if let Some(ttl) = entry.ttl_seconds {
                let elapsed = chrono::Utc::now().signed_duration_since(entry.created_at);
                if elapsed.num_seconds() > ttl as i64 {
                    self.cache.invalidate(key).await;
                    return None;
                }
            }

            // Update access time (this would be better with a custom cache implementation)
            Some(entry.data.clone())
        } else {
            if self.config.enable_stats {
                let mut stats = self.stats.lock().await;
                stats.code_cache_misses += 1;
            }
            None
        }
    }

    /// Put an item in the cache
    pub async fn put(&self, key: String, data: T, ttl_seconds: Option<u64>) {
        let entry = CacheEntry {
            data,
            created_at: chrono::Utc::now(),
            accessed_at: chrono::Utc::now(),
            access_count: 1,
            ttl_seconds,
        };

        self.cache.insert(key, entry).await;

        if self.config.enable_stats {
            let mut stats = self.stats.lock().await;
            stats.code_cache_inserts += 1;
        }
    }

    /// Remove an item from the cache
    pub async fn remove(&self, key: &str) -> bool {
        let was_present = self.cache.contains_key(key);
        self.cache.invalidate(key).await;

        if was_present && self.config.enable_stats {
            let mut stats = self.stats.lock().await;
            stats.clears += 1;
        }

        was_present
    }

    /// Clear all items from the cache
    pub async fn clear(&self) {
        self.cache.invalidate_all();

        if self.config.enable_stats {
            let mut stats = self.stats.lock().await;
            stats.clears += 1;
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        if self.config.enable_stats {
            self.stats.lock().await.clone()
        } else {
            CacheStats::default()
        }
    }

    /// Check if cache contains a key
    pub async fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Get the number of items in the cache
    pub async fn len(&self) -> u64 {
        self.cache.entry_count()
    }
}
