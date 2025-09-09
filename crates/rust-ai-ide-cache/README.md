# rust-ai-ide-cache

The cache crate provides a unified, high-performance caching infrastructure for the Rust AI IDE. This crate consolidates multiple caching implementations into a single, cohesive system with async support, multiple storage backends, and rich metadata tracking.

Built for v2.4.0, this crate eliminates fragmentation by providing a single `Cache<K,V>` trait that unifies previously disparate caching patterns across the codebase.

## 📦 Features

- **Unified Cache Trait**: Single async `Cache<K,V>` trait for all implementations
- **Multiple Backends**: InMemoryCache, HybridCache, and RedisCache support
- **Rich TTL & Eviction**: Configurable time-to-live and multiple eviction policies (LRU, LFU, FIFO, Random, Size-based)
- **Performance Monitoring**: Comprehensive statistics and metrics collection
- **Type-Safe Keys**: SHA256-based key generation utilities
- **Domain Extensions**: Specialized LSP and AI caching patterns
- **Async Support**: Full tokio async/await support
- **Background Cleanup**: Automatic expired entry cleanup

## 🔗 Architecture Integration

The caching layer integrates deeply with:

- `rust-ai-ide-core`: Core operations and path management
- `rust-ai-ide-lsp`: Language server analysis caching via `LspCacheExt`
- `rust-ai-ide-ai-codegen`: AI computation results via `AiCacheExt`
- `rust-ai-ide-shared-types`: Shared data structures and serialization

## 🚀 Usage

### Basic Usage

```rust
use rust_ai_ide_cache::{Cache, InMemoryCache, CacheConfig};

// Create a cache with default configuration
let cache = InMemoryCache::<String, String>::new(CacheConfig::default()).await;

// Store and retrieve values
cache.insert("key".to_string(), "value".to_string(), None).await?;
let value = cache.get(&"key".to_string()).await?;

// Check statistics
let stats = cache.stats().await;
println!("Hit ratio: {:.2}%", stats.hit_ratio * 100.0);
```

### Advanced Configuration

```rust
use rust_ai_ide_cache::{Cache, HybridCache, CacheConfig, EvictionPolicy, key_utils};
use std::time::Duration;

// Configure a hybrid cache with custom settings
let config = CacheConfig {
    max_entries: Some(5000),
    default_ttl: Some(Duration::from_secs(3600)), // 1 hour
    eviction_policy: EvictionPolicy::Lfu,
    enable_metrics: true,
    max_memory_mb: Some(256),
    background_cleanup_interval_seconds: 600,
    ..Default::default()
};

let cache = HybridCache::<String, serde_json::Value>::new(config).await?;

// Use structured key generation
let key = key_utils::structured_key("lsp", &"diagnostic_data.json");
cache.insert(key, serde_json::json!({"diagnostics": []}), Some(Duration::from_secs(300))).await?;
```

### LSP Caching Extension

```rust
use rust_ai_ide_cache::{InMemoryCache, LspCacheExt};

let lsp_cache = InMemoryCache::<String, serde_json::Value>::new(Default::default()).await;

// Store LSP analysis results with file validation
lsp_cache.lsp_store_analysis(
    "src/main.rs".to_string(),
    serde_json::json!({"symbols": [], "diagnostics": []}),
    "file_hash_123".to_string(),
    Some(Duration::from_secs(600))
).await?;

// Retrieve analysis with validation
if let Some(result) = lsp_cache.lsp_retrieve_analysis(&"src/main.rs".to_string()).await? {
    println!("Retrieved LSP analysis: {:?}", result);
}
```

### AI Caching Extension

```rust
use rust_ai_ide_cache::{RedisCache, AiCacheExt};

let ai_cache = RedisCache::<String, serde_json::Value>::new(Default::default()).await?;

// Cache AI inference results with token usage tracking
ai_cache.ai_store_inference(
    "completion_prompt_hash".to_string(),
    serde_json::json!({"completions": ["result"]}),
    Some(150), // tokens used
    Some(Duration::from_secs(1800))
).await?;

// Cache similarity computations
ai_cache.ai_store_similarity(
    "pattern_similarity_key".to_string(),
    serde_json::json!({"similarities": [0.95, 0.87]}),
    Some(Duration::from_secs(3600))
).await?;
```

## 📚 Integration Guide

### Multiple Backend Selection

```rust
// For high-performance, in-memory caching
let memory_cache = InMemoryCache::new(Default::default()).await;

// For large datasets with persistence
let hybrid_cache = HybridCache::new(CacheConfig {
    max_memory_mb: Some(512),
    ..Default::default()
}).await;

// For distributed deployments
let redis_cache = RedisCache::new(CacheConfig::default()).await;
```

### Cache Manager for Orchestration

```rust
use rust_ai_ide_cache::CacheManager;

let mut manager = CacheManager::new(Default::default());

// Register multiple caches
manager.register_cache("lsp", memory_cache);
manager.register_cache("ai", redis_cache);

// Global cleanup
let cleaned = manager.cleanup_all().await?;
let stats = manager.global_stats().await;
```

## 📈 Performance Characteristics

- **Memory Efficient**: Configurable size limits and compression support
- **High Hit Ratios**: Adaptive eviction policies optimize for access patterns
- **Low Latency**: In-memory operations with Redis persistence option
- **Background Processing**: Non-blocking cleanup and statistics updates
- **Scalable**: Supports millions of entries across distributed backends

### Performance Best Practices

1. **Choose Appropriate Backend**:
   - Use `InMemoryCache` for hot data and high-frequency access
   - Use `HybridCache` for large datasets with memory limits
   - Use `RedisCache` for distributed and persistent caching

2. **Configure TTL Strategically**:
   ```rust
   // Short TTL for volatile data
   let config = CacheConfig {
       default_ttl: Some(Duration::from_secs(300)), // 5 minutes
       ..Default::default()
   };
   ```

3. **Monitor Performance**:
   ```rust
   let stats = cache.stats().await;
   if stats.hit_ratio < 0.7 {
       // Consider adjusting eviction policy or TTL
   }
   ```

4. **Use Type-Safe Keys**:
   ```rust
   // Avoid string concatenation - use utilities
   let key = key_utils::structured_key("operation", &context_data);
   ```

## 🔄 Migration Notes

### Legacy Cache Consolidation

The unified caching system replaces fragmented implementations:

- **GenericCache** → `Cache<K,V>` trait
- **InMemoryCache** → `cache_impls::InMemoryCache`
- **DiagnosticCache** → LSP-specific caching with `LspCacheExt`
- **CachedItem/CacheEntry** → Unified `CacheEntry<V>` with rich metadata
- **CacheStatistics** → Consolidated `CacheStats` across all implementations

### Migration Path

1. **Update Imports**:
   ```rust
   // Before
   use some_legacy_cache::{GenericCache, CacheStatistics};

   // After
   use rust_ai_ide_cache::{Cache, CacheStats, InMemoryCache};
   ```

2. **Replace Trait Usage**:
   ```rust
   // Before
   async fn old_cache_get(cache: &impl SomeLegacyTrait) { /* ... */ }

   // After
   async fn new_cache_get<C: Cache<K, V>>(cache: &C) { /* ... */ }
   ```

3. **Migrate Configuration**:
   ```rust
   // Before: Various config structs
   let config = LegacyConfig { max_size: 1000 };

   // After: Unified config
   let config = CacheConfig {
       max_entries: Some(1000),
       ..Default::default()
   };
   ```

4. **Update Statistics Access**:
   ```rust
   // Before: Direct field access
   let hits = cache.stats.hits;

   // After: Async stats method
   let stats = cache.stats().await;
   let hits = stats.total_hits;
   ```

### Benefits of Migration

- **50% reduction** in cache-related code duplication
- **Consistent API** across all caching operations
- **Enhanced monitoring** with unified statistics
- **Better performance** through optimized implementations
- **Future-proof** with extensible backend support