fntest main() {
    use std::time::Duration;
    use rust_ai_ide_cache::{InMemoryCache, CacheConfig, EvictionPolicy};

    let config = CacheConfig {
        max_entries: 1000,
        default_ttl: Some(Duration::from_secs(300)),
        eviction_policy: EvictionPolicy::Lru,
        enable_metrics: true,
        max_memory_mb: Some(50),
        compression_threshold_kb: Some(10),
        background_cleanup_interval_seconds: 300,
    };

    let cache: InMemoryCache<String, String> = InMemoryCache::new(config);

    println!("Cache created successfully!");
}