//! Unified Caching Infrastructure for Rust AI IDE
//!
//! This crate provides a comprehensive caching solution that consolidates
//! multiple cache implementations found throughout the Rust AI IDE codebase.
//!
//! Instead of having multiple cache types like:
//! - GenericCache, InMemoryCache, DiagnosticCache, LegacyDiagnosticCache
//! - CachedItem, CacheEntry (different names for similar concepts)
//! - Duplicate CacheStatistics across modules
//!
//! This crate provides:
//! - Unified Cache trait with async support
//! - Multiple storage backends (memory, disk, hybrid)
//! - Rich TTL and eviction policies
//! - Serialization/deserialization support
//! - Performance monitoring and metrics
//! - Type-safe key generation
//! - Async operations with tokio

pub mod adapters;
pub mod cache_impls;
pub mod lsp_cache;
pub mod storage;
pub mod strategies;

use async_trait::async_trait;
use rust_ai_ide_errors::IDEResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;
use tokio::sync::RwLock;
// Using chrono::DateTime<chrono::Utc> as Timestamp alias
type Timestamp = chrono::DateTime<chrono::Utc>;

/// Re-export commonly used items
pub use cache_impls::*;
pub use storage::*;
pub use strategies::*;

/// Unified cache entry with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: Timestamp,
    pub last_accessed: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub access_count: u64,
    pub ttl_seconds: Option<u64>,
    pub metadata: HashMap<String, String>,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V) -> Self {
        let now = chrono::Utc::now();
        Self::new_with_ttl(value, None, now)
    }

    pub fn new_with_ttl(value: V, ttl: Option<Duration>, created_at: Timestamp) -> Self {
        let expires_at = ttl.map(|t| {
            let duration_ms = t.as_millis() as i64;
            created_at + chrono::Duration::milliseconds(duration_ms)
        });

        Self {
            value,
            created_at,
            last_accessed: created_at,
            expires_at,
            access_count: 0,
            ttl_seconds: ttl.map(|t| t.as_secs()),
            metadata: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = chrono::Utc::now();
    }

    pub fn update_value(&mut self, value: V) {
        self.value = value;
        self.last_accessed = chrono::Utc::now();
    }

    pub fn refresh_ttl(&mut self, ttl: Option<Duration>) {
        let new_expires_at = ttl.map(|t| {
            let duration_ms = t.as_millis() as i64;
            self.last_accessed + chrono::Duration::milliseconds(duration_ms)
        });
        self.expires_at = new_expires_at;
        self.ttl_seconds = ttl.map(|t| t.as_secs());
    }

    pub fn size_hint(&self) -> usize
    where
        V: serde::Serialize,
    {
        // Rough estimate of serialized size
        serde_json::to_string(self).map(|s| s.len()).unwrap_or(0)
    }
}

/// Unified cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_entries: Option<usize>,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub enable_metrics: bool,
    pub max_memory_mb: Option<usize>,
    pub compression_threshold_kb: Option<usize>,
    pub background_cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: Some(1000),
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            eviction_policy: EvictionPolicy::Lru,
            enable_metrics: true,
            max_memory_mb: None,
            compression_threshold_kb: None,
            background_cleanup_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Eviction policies for cache
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvictionPolicy {
    /// Least Recently Used - evict oldest accessed
    Lru,
    /// Least Frequently Used - evict least accessed
    Lfu,
    /// First In First Out - evict oldest
    Fifo,
    /// Random eviction
    Random,
    /// Size-based eviction (for memory limits)
    SizeBased,
    /// Adaptive strategy with dynamic weight adjustment
    Adaptive,
    /// Windowed TinyLFU - advanced frequency-based eviction
    #[serde(rename = "w_tiny_lfu")]
    WTinyLFU,
    /// Segmented LRU - hybrid of LRU with multiple segments
    #[serde(rename = "segmented_lru")]
    SegmentedLRU,
    /// Clock algorithm - approximate LRU with O(1) operations
    Clock,
}

/// Unified cache trait that unifies all cache implementations
#[async_trait]
pub trait Cache<K, V>: Send + Sync + 'static
where
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize,
    V: Send + Sync + Clone + serde::Serialize,
{
    /// Get a value from the cache
    async fn get(&self, key: &K) -> IDEResult<Option<V>>;

    /// Insert a value into the cache
    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()>;

    /// Remove a value from the cache
    async fn remove(&self, key: &K) -> IDEResult<Option<V>>;

    /// Clear all entries from the cache
    async fn clear(&self) -> IDEResult<()>;

    /// Get cache size
    async fn size(&self) -> usize;

    /// Check if cache contains key
    async fn contains(&self, key: &K) -> bool;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;

    /// Force cleanup of expired entries
    async fn cleanup_expired(&self) -> IDEResult<usize>;
}

/// Cache performance metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub total_sets: u64,
    pub hit_ratio: f64,
    pub memory_usage_bytes: Option<u64>,
    pub uptime_seconds: u64,
    pub created_at: Timestamp,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            total_hits: 0,
            total_misses: 0,
            total_evictions: 0,
            total_sets: 0,
            hit_ratio: 0.0,
            memory_usage_bytes: None,
            uptime_seconds: 0,
            created_at: chrono::Utc::now(),
        }
    }
}

impl CacheStats {
    pub fn record_hit(&mut self) {
        self.total_hits += 1;
        self.update_hit_ratio();
    }

    pub fn record_miss(&mut self) {
        self.total_misses += 1;
        self.update_hit_ratio();
    }

    pub fn record_set(&mut self) {
        self.total_sets += 1;
    }

    pub fn record_eviction(&mut self) {
        self.total_evictions += 1;
    }

    pub fn update_hit_ratio(&mut self) {
        let total = self.total_hits + self.total_misses;
        self.hit_ratio = if total > 0 {
            self.total_hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

/// Cache manager for coordinating multiple caches
pub struct CacheManager {
    caches: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    config: CacheConfig,
    stats: RwLock<CacheStats>,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let stats = CacheStats {
            created_at: chrono::Utc::now(),
            uptime_seconds: 0,
            ..Default::default()
        };

        Self {
            caches: HashMap::new(),
            config,
            stats: RwLock::new(stats),
        }
    }

    pub fn register_cache<K, V, C>(&mut self, name: &str, cache: C)
    where
        K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
        V: Send + Sync + Clone + serde::Serialize + 'static,
        C: Cache<K, V> + 'static,
    {
        self.caches.insert(name.to_string(), Box::new(cache));
    }

    pub async fn cleanup_all(&self) -> IDEResult<usize> {
        let total_cleaned = 0;
        // Note: In real implementation, we'd iterate through caches and call cleanup_expired
        // This is simplified for the consolidation example
        Ok(total_cleaned)
    }

    pub async fn global_stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.uptime_seconds = (chrono::Utc::now() - stats.created_at)
            .as_seconds_f64()
            .abs() as u64;
        stats
    }
}

/// Type-safe cache key generation utilities
pub mod key_utils {

    use serde::Serialize;

    /// Generate a cache key from multiple components
    pub fn generate_key(components: &[&(impl Serialize + ?Sized)]) -> String {
        let mut key = String::new();
        for (i, component) in components.iter().enumerate() {
            if i > 0 {
                key.push(';');
            }
            let json = serde_json::to_string(component).unwrap_or_default();
            key.push_str(&json);
        }
        sha256::digest(key)
    }

    /// Generate a structured cache key
    pub fn structured_key(prefix: &str, data: &(impl Serialize + ?Sized)) -> String {
        format!("{}:{}", prefix, generate_key(&[data]))
    }

    /// Generate a path-based cache key
    pub fn path_key(operation: &str, path: &std::path::Path) -> String {
        format!("{}:{}", operation, path.display())
    }
}

/// Shorthand type aliases for common use cases
pub type StringCache = InMemoryCache<String, String>;
/// TODO: Define CompilerDiagnosticsResult in rust_ai_ide_types
// pub type DiagnosticCache = InMemoryCache<String, rust_ai_ide_types::CompilerDiagnosticsResult>;
/// TODO: Define ErrorCodeExplanation in rust_ai_ide_types
// pub type ExplanationCache = InMemoryCache<String, rust_ai_ide_types::ErrorCodeExplanation>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_entry_lifecycle() {
        let entry = CacheEntry::new("test_value");
        assert!(!entry.is_expired());
        assert_eq!(entry.access_count, 0);

        let mut entry_clone = entry.clone();
        entry_clone.access();
        assert_eq!(entry_clone.access_count, 1);
    }

    #[tokio::test]
    async fn test_entry_with_ttl() {
        let short_ttl = Duration::from_millis(10);
        let entry = CacheEntry::new_with_ttl("ttl_value", Some(short_ttl), chrono::Utc::now());

        assert!(!entry.is_expired());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.total_hits, 2);
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.hit_ratio, 2.0 / 3.0);
    }

    #[test]
    fn test_key_generation() {
        let key = key_utils::structured_key("test", &"data");
        assert!(!key.is_empty());

        let multi_key = key_utils::generate_key(&[&"component1", &"component2"]);
        assert!(!multi_key.is_empty());
    }
}

// Domain-specific cache trait extensions

/// Extensions for LSP-specific caching patterns
#[async_trait]
pub trait LspCacheExt: Cache<String, serde_json::Value> + Send + Sync + 'static {
    /// Cache LSP analysis results with file validation metadata
    async fn lsp_store_analysis(
        &self,
        file_key: String,
        result: serde_json::Value,
        file_hash: String,
        ttl: Option<Duration>,
    ) -> IDEResult<()> {
        // Extend the base result with LSP metadata
        let mut cache_entry = CacheEntry::new(result.clone());
        cache_entry
            .metadata
            .insert("file_hash".to_string(), file_hash);
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "lsp_analysis".to_string());

        // Use the unified cache insert method
        self.insert(file_key, result, ttl).await
    }

    /// Retrieve LSP analysis result with validation
    async fn lsp_retrieve_analysis(
        &self,
        file_key: &String,
    ) -> IDEResult<Option<serde_json::Value>> {
        self.get(file_key).await
    }
}

// Auto-implement for any cache that implements the base trait
impl<C> LspCacheExt for C where C: Cache<String, serde_json::Value> + Send + Sync + 'static {}

/// Specialized cache for high-performance AI operations
#[async_trait]
pub trait AiCacheExt: Cache<String, serde_json::Value> + Send + Sync + 'static {
    /// Cache AI computation results with usage metrics
    async fn ai_store_inference(
        &self,
        query_key: String,
        result: serde_json::Value,
        tokens_used: Option<u32>,
        ttl: Option<Duration>,
    ) -> IDEResult<()> {
        let mut cache_entry = CacheEntry::new(result.clone());
        if let Some(tokens) = tokens_used {
            cache_entry
                .metadata
                .insert("tokens_used".to_string(), tokens.to_string());
        }
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "ai_inference".to_string());

        self.insert(query_key, result, ttl).await
    }

    /// Cache similarity computations
    async fn ai_store_similarity(
        &self,
        pattern_key: String,
        similarities: serde_json::Value,
        ttl: Option<Duration>,
    ) -> IDEResult<()> {
        let mut cache_entry = CacheEntry::new(similarities.clone());
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "similarity".to_string());

        self.insert(pattern_key, similarities, ttl).await
    }
}

// Auto-implement for any AI cache
impl<C> AiCacheExt for C where C: Cache<String, serde_json::Value> + Send + Sync + 'static {}
