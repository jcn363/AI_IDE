//! Dummy cache implementations for when caching is disabled

pub use super::dummy_cache_impl::{DummyCache, DummyCacheEntry};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_dummy_cache() {
        let cache: DummyCache<String, String> = DummyCache::new();

        // Should always return None since we're not actually caching
        assert!(cache.get(&"key".to_string()).is_none());

        // Insert should be a no-op
        cache.insert("key".to_string(), "value".to_string());
        assert!(cache.get(&"key".to_string()).is_none());

        // Clear should be a no-op
        cache.clear();
    }

    #[test]
    fn test_dummy_cache_entry() {
        let entry = DummyCacheEntry::new(42);
        assert_eq!(entry.into_inner(), 42);

        let entry_with_ttl = DummyCacheEntry::with_ttl(42, Duration::from_secs(60));
        assert_eq!(entry_with_ttl.into_inner(), 42);
    }
}
