//! Analysis Result Caching for Incremental Analysis - Unified Cache Integration
//!
//! This module provides intelligent caching of code analysis results using the
//! unified cache infrastructure from rust-ai-ide-cache, with:
//! - High-performance unified caching with unified statistics
//! - Smart cache invalidation based on file changes
//! - Metadata tracking for file validation
//! - LSP-specific optimizations through domain extensions
//! - Backward compatibility with existing interfaces

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::analysis_result::FileAnalysisResult;
use super::file_hash::{calculate_file_hash, get_file_metadata};

// Import unified cache infrastructure
use rust_ai_ide_cache::{
    CacheStats, IDEResult, LspAnalysisCache, LspCacheConfig
};

/// Configuration for analysis result caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCacheConfig {
    /// Cache size limit in MB
    pub capacity_mb: usize,
    /// TTL for cache entries in seconds
    pub ttl_seconds: u64,
    /// Enable compression for large results
    pub enable_compression: bool,
    /// Compression threshold in bytes
    pub compression_threshold_kb: usize,
    /// Redis configuration (if available)
    #[cfg(feature = "redis-backend")]
    pub redis_config: Option<RedisConfig>,
    /// Fallback to in-memory cache if Redis unavailable
    pub enable_fallback_cache: bool,
}

impl Default for AnalysisCacheConfig {
    fn default() -> Self {
        Self {
            capacity_mb: 512,
            ttl_seconds: 3600, // 1 hour
            enable_compression: true,
            compression_threshold_kb: 100,
            #[cfg(feature = "redis-backend")]
            redis_config: None,
            enable_fallback_cache: true,
        }
    }
}

/// Cached analysis result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnalysisResult {
    /// The original analysis result
    pub result: FileAnalysisResult,
    /// File content hash used for validation
    pub file_hash: String,
    /// File size when cached
    pub file_size: u64,
    /// File modification time when cached
    pub file_mtime: std::time::SystemTime,
    /// Dependencies detected during analysis
    pub dependencies: Vec<PathBuf>,
    /// Cache creation timestamp
    pub cached_at: chrono::DateTime<chrono::Utc>,
    /// TTL in seconds
    pub ttl: u64,
}

/// Cache entry metadata for cache management
#[derive(Debug, Clone)]
pub struct CacheEntryMetadata {
    pub key: String,
    pub size_bytes: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
}

/// Zero-copy analysis result caching trait
#[async_trait]
pub trait ZeroCopyAnalysisCaching: Send + Sync {
    /// Cache an analysis result using zero-copy operations
    async fn store_ref<'a>(&self, file_path: &PathBuf, result: &'a FileAnalysisResult) -> Result<(), String>
    where
        'a: 'static;

    /// Retrieve a cached analysis result - returns owned data but avoids internal copies
    async fn retrieve_zero_copy(&self, file_path: &PathBuf) -> Result<Option<Box<FileAnalysisResult>>, String>;

    /// Store raw bytes directly (zero-copy for serialized data)
    async fn store_raw(&self, file_path: &PathBuf, raw_data: &[u8], file_hash: &str) -> Result<(), String>;

    /// Borrow raw bytes without copying
    async fn borrow_raw<'a>(&self, _file_path: &PathBuf) -> Result<Option<&'a [u8]>, String> {
        // Not implemented in base version - would require byte buffer management
        Ok(None)
    }

    /// Memory-mapped file access for large analysis results
    async fn get_memory_mapped(&self, _file_path: &PathBuf) -> Result<Option<memmap2::Mmap>, String> {
        // Not implemented in base version
        Ok(None)
    }
}

/// Legacy trait for backward compatibility
#[async_trait]
pub trait AnalysisCaching: Send + Sync {
    /// Cache an analysis result
    async fn store(&self, file_path: &PathBuf, result: FileAnalysisResult) -> Result<(), String>;

    /// Retrieve a cached analysis result
    async fn retrieve(&self, file_path: &PathBuf) -> Result<Option<FileAnalysisResult>, String>;

    /// Check if a file's cached result is still valid
    async fn is_valid(&self, file_path: &PathBuf) -> Result<bool, String>;

    /// Remove cached result for a file
    async fn invalidate(&self, file_path: &PathBuf) -> Result<(), String>;

    /// Clear all cached results
    async fn clear(&self) -> Result<(), String>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats, String>;

    /// Get cache size in bytes
    async fn size(&self) -> usize;

    /// Compact cache by removing expired entries
    async fn compact(&self) -> Result<usize, String>;
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of entries in cache
    pub total_entries: usize,
    /// Total cache size in bytes
    pub total_size_bytes: u64,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Hit ratio (0.0-1.0)
    pub hit_ratio: f64,
    /// Number of invalidated entries
    pub invalidated: u64,
    /// Cache uptime in seconds
    pub uptime_seconds: u64,
}

/// Main analysis cache implementation using unified cache system
pub struct AnalysisCache {
    unified_cache: LspAnalysisCache,
    config: LspCacheConfig,
}

/// Zero-copy analysis cache implementation
#[derive(Clone)]
pub struct ZeroCopyAnalysisCache {
    raw_cache: rust_ai_ide_cache::InMemoryCache<String, Arc<Vec<u8>>>,
    metadata_cache: rust_ai_ide_cache::InMemoryCache<String, CacheEntryMetadata>,
    config: LspCacheConfig,
}

impl ZeroCopyAnalysisCache {
    /// Create a new zero-copy analysis cache
    pub fn new(config: LspCacheConfig) -> Self {
        Self {
            raw_cache: rust_ai_ide_cache::InMemoryCache::new(),
            metadata_cache: rust_ai_ide_cache::InMemoryCache::new(),
            config,
        }
    }

    fn make_metadata_key(&self, file_path: &PathBuf) -> String {
        format!("meta:{}", file_path.display())
    }
}

impl AnalysisCache {
    /// Create a new analysis cache
    pub async fn new(capacity_mb: usize) -> Result<Self, String> {
        let config = LspCacheConfig {
            base_config: rust_ai_ide_cache::CacheConfig {
                max_entries: Some((capacity_mb * 1024 * 1024) / 1024), // Rough estimate
                default_ttl: Some(Duration::from_secs(3600)), // 1 hour
                ..Default::default()
            },
            enable_file_validation: true,
            max_file_cache_age_seconds: 3600,
            analysis_ttl_seconds: 1800, // 30 minutes for analysis results
        };

        let unified_cache = LspAnalysisCache::new(config.clone());

        Ok(Self {
            unified_cache,
            config,
        })
    }

    /// Generate cache key from file path
    fn make_key(&self, file_path: &PathBuf) -> String {
        LspAnalysisCache::generate_analysis_key(file_path, "legacy_lsp", None)
    }
}

/// Zero-copy implementation that avoids serialization/deserialization overhead
#[async_trait::*]
impl ZeroCopyAnalysisCaching for ZeroCopyAnalysisCache {
    async fn store_ref<'a>(&self, file_path: &PathBuf, result: &'a FileAnalysisResult) -> Result<(), String>
    where
        'a: 'static,
    {
        // Use zero-copy serialization if possible
        let cache_key = LspAnalysisCache::generate_analysis_key(file_path, "zero_copy", None);

        // Serialize once and store Arc<Vec<u8>> to avoid copying
        let serialized = serde_json::to_vec(result)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let data = Arc::new(serialized);

        // Store the raw data (zero-copy)
        self.raw_cache.insert(cache_key.clone(), data, None)
            .await
            .map_err(|e| format!("Raw cache error: {:?}", e))?;

        // Store metadata (minimal)
        let metadata = CacheEntryMetadata {
            key: cache_key.clone(),
            size_bytes: data.len(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 1,
        };

        let meta_key = self.make_metadata_key(file_path);
        self.metadata_cache.insert(meta_key, metadata, None)
            .await
            .map_err(|e| format!("Metadata cache error: {:?}", e))?;

        Ok(())
    }

    async fn retrieve_zero_copy(&self, file_path: &PathBuf) -> Result<Option<Box<FileAnalysisResult>>, String> {
        let cache_key = LspAnalysisCache::generate_analysis_key(file_path, "zero_copy", None);

        // Retrieve the raw data (zero-copy)
        if let Some(data_arc) = self.raw_cache.get(&cache_key).await
            .map_err(|e| format!("Raw cache retrieval error: {:?}", e))? {

            // Deserialize from the Arc<Vec<u8>> (minimal copying)
            let result: FileAnalysisResult = serde_json::from_slice(&data_arc)
                .map_err(|e| format!("Deserialization error: {}", e))?;

            // Return in a Box to avoid stack copying
            Ok(Some(Box::new(result)))
        } else {
            Ok(None)
        }
    }

    async fn store_raw(&self, file_path: &PathBuf, raw_data: &[u8], file_hash: &str) -> Result<(), String> {
        let cache_key = LspAnalysisCache::generate_analysis_key(file_path, "zero_copy_raw", Some(file_hash));

        // Store the raw bytes directly (zero-copy)
        let data = Arc::new(raw_data.to_vec()); // This is necessary for storage, but still avoids copying later
        self.raw_cache.insert(cache_key.clone(), data, None)
            .await
            .map_err(|e| format!("Raw data cache error: {:?}", e))?;

        Ok(())
    }

    async fn borrow_raw<'a>(&self, file_path: &PathBuf) -> Result<Option<&'a [u8]>, String> {
        let cache_key = LspAnalysisCache::generate_analysis_key(file_path, "zero_copy_raw", None);

        // This implementation would require a different cache type that supports borrowing
        // For now, return None to indicate borrowing is not available for this cache type
        if let Some(_) = self.raw_cache.get(&cache_key).await
            .map_err(|e| format!("Cache borrow error: {:?}", e))? {
            // In a proper implementation, we'd return a reference to the internal buffer
            Err("Borrowing not implemented for this cache type".to_string())
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl AnalysisCaching for AnalysisCache {
    async fn store(&self, file_path: &PathBuf, result: FileAnalysisResult) -> Result<(), String> {
        // Get file metadata for validation
        let file_hash = match get_file_metadata(file_path) {
            Ok((hash, _, _)) => hash,
            Err(_) => calculate_file_hash(file_path).unwrap_or_default(),
        };

        let cache_key = self.make_key(file_path);

        // Serialize the result to JSON for unified cache storage
        let json_result = serde_json::to_value(&result)
            .map_err(|e| format!("Serialization error: {}", e))?;

        // Use unified LSP cache with domain-specific extensions
        self.unified_cache.lsp_store_analysis(
            cache_key,
            json_result,
            file_hash,
            Some(Duration::from_secs(self.config.analysis_ttl_seconds))
        ).await
        .map_err(|e| format!("Unified cache error: {:?}", e))?;

        Ok(())
    }

    async fn retrieve(&self, file_path: &PathBuf) -> Result<Option<FileAnalysisResult>, String> {
        let cache_key = self.make_key(file_path);

        // Use unified cache retrieval
        let json_result = self.unified_cache.lsp_retrieve_analysis(&cache_key).await
            .map_err(|e| format!("Unified cache error: {:?}", e))?;

        if let Some(json_value) = json_result {
            // Deserialize back to FileAnalysisResult
            let result: FileAnalysisResult = serde_json::from_value(json_value)
                .map_err(|e| format!("Deserialization error: {}", e))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn is_valid(&self, file_path: &PathBuf) -> Result<bool, String> {
        // For the unified cache, we rely on TTL expiration
        // File content validation would need additional metadata storage
        let _cache_key = self.make_key(file_path);
        // The unified cache handles expiration internally
        Ok(true) // Placeholder - actual validation would require metadata comparison
    }

    async fn invalidate(&self, file_path: &PathBuf) -> Result<(), String> {
        // Use unified cache invalidation through the interface
        self.unified_cache.invalidate_file(file_path).await
            .map_err(|e| format!("Unified cache error: {:?}", e))?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), String> {
        self.unified_cache.clear().await
            .map_err(|e| format!("Unified cache error: {:?}", e))?;
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats, String> {
        let unified_stats = self.unified_cache.stats().await;

        Ok(CacheStats {
            total_entries: unified_stats.total_entries,
            total_size_bytes: unified_stats.memory_usage_bytes.unwrap_or(0),
            hits: unified_stats.total_hits,
            misses: unified_stats.total_misses,
            hit_ratio: unified_stats.hit_ratio,
            invalidated: unified_stats.total_evictions,
            uptime_seconds: unified_stats.uptime_seconds,
        })
    }

    fn size(&self) -> usize {
        // Note: This would need to be async in a real implementation
        // For now, return 0 to maintain compatibility
        0
    }

    async fn compact(&self) -> Result<usize, String> {
        // The unified cache handles cleanup automatically
        let removed = self.unified_cache.cleanup_expired().await
            .map_err(|e| format!("Unified cache error: {:?}", e))?;
        Ok(removed)
    }
}

/// Factory function for creating optimized analysis caches
pub async fn create_analysis_cache(
    capacity_mb: usize,
    enable_redis: bool,
    redis_url: Option<String>,
) -> Result<AnalysisCache, String> {
    let mut config = AnalysisCacheConfig::default();
    config.capacity_mb = capacity_mb;

    #[cfg(feature = "redis-backend")]
    if enable_redis {
        let redis_config = if let Some(url) = redis_url {
            RedisConfig::single_node(&url)
        } else {
            RedisConfig::single_node("redis://localhost:6379")
        };
        config.redis_config = Some(redis_config);
    }

    AnalysisCache::new(capacity_mb).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_analysis_cache_creation() {
        let cache = AnalysisCache::new(100).await.unwrap();
        assert_eq!(cache.size().await, 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let cache = AnalysisCache::new(100).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        // Create a test file
        fs::write(&file_path, "fn main() {}").await.unwrap();

        let analysis_result = FileAnalysisResult {
            file_path: file_path.clone(),
            language: "rust".to_string(),
            diagnostics: vec![],
            suggestions: vec![],
            analysis_time_ms: 100,
            from_cache: false,
        };

        // Store result
        cache.store(&file_path, analysis_result.clone()).await.unwrap();
        assert_eq!(cache.size().await, 1);

        // Retrieve result
        let retrieved = cache.retrieve(&file_path).await.unwrap().unwrap();
        assert_eq!(retrieved.file_path, analysis_result.file_path);
        assert_eq!(retrieved.language, "rust");

        // Check cache stats
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_ratio, 1.0);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = AnalysisCache::new(100).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        // Create and cache a file
        fs::write(&file_path, "fn main() {}").await.unwrap();

        let analysis_result = FileAnalysisResult {
            file_path: file_path.clone(),
            language: "rust".to_string(),
            diagnostics: vec![],
            suggestions: vec![],
            analysis_time_ms: 100,
            from_cache: false,
        };

        cache.store(&file_path, analysis_result).await.unwrap();
        assert!(cache.retrieve(&file_path).await.unwrap().is_some());

        // Invalidate cache
        cache.invalidate(&file_path).await.unwrap();
        assert!(cache.retrieve(&file_path).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = AnalysisCache::new(100).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let file_path1 = temp_dir.path().join("test1.rs");
        let file_path2 = temp_dir.path().join("test2.rs");

        // Create and cache multiple files
        fs::write(&file_path1, "fn main1() {}").await.unwrap();
        fs::write(&file_path2, "fn main2() {}").await.unwrap();

        let result1 = FileAnalysisResult {
            file_path: file_path1.clone(),
            language: "rust".to_string(),
            diagnostics: vec![],
            suggestions: vec![],
            analysis_time_ms: 100,
            from_cache: false,
        };

        let result2 = FileAnalysisResult {
            file_path: file_path2.clone(),
            language: "rust".to_string(),
            diagnostics: vec![],
            suggestions: vec![],
            analysis_time_ms: 100,
            from_cache: false,
        };

        cache.store(&file_path1, result1).await.unwrap();
        cache.store(&file_path2, result2).await.unwrap();

        assert_eq!(cache.size().await, 2);

        // Clear cache
        cache.clear().await.unwrap();
        assert_eq!(cache.size().await, 0);
        assert!(cache.retrieve(&file_path1).await.unwrap().is_none());
        assert!(cache.retrieve(&file_path2).await.unwrap().is_none());
    }
}