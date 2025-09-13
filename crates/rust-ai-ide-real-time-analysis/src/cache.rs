#![allow(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use dashmap::DashMap;
use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};
use sled::{Db, IVec};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::types::{AnalysisResult, AnalysisType, CacheConfig, PerformanceMetrics};

/// Cache errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache key error: {0}")]
    Key(String),
}

/// Cache result type
type CacheResult<T> = Result<T, CacheError>;

/// Cache hit/miss statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Total cache accesses
    pub total_accesses: u64,
    /// Cache hits
    pub hits:           u64,
    /// Cache misses
    pub misses:         u64,
    /// Hit rate percentage
    pub hit_rate:       f32,
    /// Memory cache size
    pub memory_size:    u64,
    /// Disk cache size
    pub disk_size:      u64,
    /// Last eviction timestamp
    pub last_eviction:  Option<i64>,
}

/// Cache key for analysis results
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheKey {
    /// File path
    pub file_path:     PathBuf,
    /// Analysis type
    pub analysis_type: AnalysisType,
    /// File modification time
    pub file_mtime:    i64,
    /// File size
    pub file_size:     u64,
    /// File hash (for content verification)
    pub file_hash:     String,
    /// Analysis configuration hash
    pub config_hash:   String,
}

impl CacheKey {
    /// Create a cache key from file path and analysis type
    pub async fn from_file(file_path: &Path, analysis_type: AnalysisType) -> CacheResult<Self> {
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_mtime = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let file_size = metadata.len();

        // Simple file hash using file size and mtime (in production, use actual content hash)
        let file_hash = format!("{:x}", (file_size as u64).wrapping_mul(file_mtime as u64));

        // Configuration hash (simplified)
        let config_hash = format!("{:?}", analysis_type);

        Ok(Self {
            file_path: file_path.to_path_buf(),
            analysis_type,
            file_mtime,
            file_size,
            file_hash,
            config_hash,
        })
    }

    /// Check if cache key is still valid for a file
    pub async fn is_valid(&self) -> bool {
        if let Ok(metadata) = tokio::fs::metadata(&self.file_path).await {
            let current_mtime = metadata
                .modified()
                .unwrap_or(std::time::UNIX_EPOCH)
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let current_size = metadata.len();

            // Check if file has changed
            current_mtime == self.file_mtime && current_size == self.file_size
        } else {
            false // File doesn't exist, key is invalid
        }
    }

    /// Convert to string key for storage
    pub fn to_string_key(&self) -> String {
        format!(
            "{}:{:?}:{}:{}:{}",
            self.file_path.display(),
            self.analysis_type,
            self.file_mtime,
            self.file_size,
            self.file_hash
        )
    }

    /// Parse from string key
    pub fn from_string_key(key: &str) -> Option<Self> {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() != 6 {
            return None;
        }

        let file_path = PathBuf::from(parts[0]);
        let analysis_type = match parts[1] {
            "Syntax" => AnalysisType::Syntax,
            "Security" => AnalysisType::Security,
            "Performance" => AnalysisType::Performance,
            "Quality" => AnalysisType::Quality,
            "Dependencies" => AnalysisType::Dependencies,
            "AiAssisted" => AnalysisType::AiAssisted,
            _ => return None,
        };

        let file_mtime = parts[2].parse().ok()?;
        let file_size = parts[3].parse().ok()?;
        let file_hash = parts[4].to_string();
        let config_hash = parts[5].to_string();

        Some(Self {
            file_path,
            analysis_type,
            file_mtime,
            file_size,
            file_hash,
            config_hash,
        })
    }
}

/// Cached analysis result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    /// Analysis result data
    pub result:        AnalysisResult,
    /// Cache key (for validation)
    pub key:           CacheKey,
    /// Cached timestamp
    pub cached_at:     i64,
    /// Access count
    pub access_count:  u64,
    /// Last accessed timestamp
    pub last_accessed: i64,
    /// Cache version for invalidation
    pub version:       u32,
}

/// Multi-level analysis cache
#[derive(Clone)]
pub struct AnalysisCache {
    /// Memory cache (fast access)
    memory_cache: Arc<MokaCache<String, CachedResult>>,
    /// Disk cache (persistent storage)
    disk_cache:   Arc<RwLock<Option<Db>>>,
    /// Cache manager for coordination
    manager:      Arc<CacheManager>,
    /// Cache statistics
    statistics:   Arc<CacheStatistics>,
    /// Configuration
    config:       CacheConfig,
}

/// Cache manager for multi-level coordination
pub struct CacheManager {
    /// Dependency tracking
    dependency_tracker:    Arc<DependencyTracker>,
    /// Invalidation listener
    invalidation_listener: Arc<RwLock<Option<Box<dyn Fn(Vec<CacheKey>) + Send + Sync>>>>,
    /// Cache version for invalidation
    current_version:       Arc<RwLock<u32>>,
}

/// Dependency tracking for cache invalidation
#[derive(Debug)]
pub struct DependencyTracker {
    /// File dependencies (reverse mapping)
    file_dependencies:     DashMap<PathBuf, HashSet<CacheKey>>,
    /// Cache invalidation triggers
    invalidation_triggers: DashMap<String, Vec<CacheKey>>,
}

impl AnalysisCache {
    /// Create a new analysis cache with configuration
    pub async fn new(config: CacheConfig) -> CacheResult<Self> {
        info!("Initializing analysis cache with config: {:?}", config);

        // Initialize memory cache
        let memory_cache = MokaCache::builder()
            .max_capacity(config.memory_cache_size)
            .time_to_live(config.cache_ttl)
            .build();

        // Initialize disk cache (optional)
        let disk_cache = if config.disk_cache_size > 0 {
            Self::initialize_disk_cache(&config).await.ok()
        } else {
            None
        };

        // Initialize cache manager
        let manager = CacheManager::new();

        // Initialize statistics
        let statistics = Default::default();

        Ok(Self {
            memory_cache: Arc::new(memory_cache),
            disk_cache: Arc::new(RwLock::new(disk_cache)),
            manager: Arc::new(manager),
            statistics: Arc::new(statistics),
            config,
        })
    }

    /// Get analysis result from cache
    #[instrument(skip(self), err)]
    pub async fn get(&self, key: &CacheKey) -> CacheResult<Option<AnalysisResult>> {
        let string_key = key.to_string_key();
        let start_time = Instant::now();

        // Update access statistics
        self.statistics.total_accesses += 1;

        // Try memory cache first
        if let Some(cached_result) = self.memory_cache.get(&string_key).await {
            if cached_result.key.is_valid().await {
                debug!("Cache hit (memory) for key: {}", string_key);
                self.statistics.hits += 1;

                // Update access metadata
                let mut updated_result = cached_result.clone();
                updated_result.access_count += 1;
                updated_result.last_accessed = chrono::Utc::now().timestamp();

                // Store updated version back in memory cache
                self.memory_cache.insert(string_key, updated_result).await;

                return Ok(Some(cached_result.result));
            } else {
                debug!("Cached result invalid, evicting: {}", string_key);
                self.memory_cache.invalidate(&string_key).await;
            }
        }

        // Try disk cache if available and memory miss
        if let Some(disk_db) = self.disk_cache.read().await.as_ref() {
            if let Ok(Some(data)) = disk_db.get(string_key.as_bytes()) {
                let data_str = String::from_utf8_lossy(&data);
                if let Ok(cached_result) = Self::deserialize_cached_result(&data_str) {
                    if cached_result.key.is_valid().await {
                        debug!("Cache hit (disk) for key: {}", string_key);
                        self.statistics.hits += 1;

                        // Promote to memory cache
                        self.memory_cache
                            .insert(string_key.clone(), cached_result.clone())
                            .await;

                        return Ok(Some(cached_result.result));
                    } else {
                        debug!("Disk cached result invalid, evicting: {}", string_key);
                        let _ = disk_db.remove(string_key.as_bytes());
                    }
                }
            }
        }

        debug!("Cache miss for key: {}", string_key);
        self.statistics.misses += 1;

        // Check access time threshold for metrics
        let access_time = start_time.elapsed();
        if access_time > self.config.hit_time_threshold {
            warn!(
                "Slow cache access: {}ms for key: {}",
                access_time.as_millis(),
                string_key
            );
        }

        Ok(None)
    }

    /// Store analysis result in cache
    #[instrument(skip(self, result), err)]
    pub async fn put(&self, key: CacheKey, result: AnalysisResult) -> CacheResult<()> {
        let string_key = key.to_string_key();
        let now = chrono::Utc::now().timestamp();

        let cached_result = CachedResult {
            result,
            key: key.clone(),
            cached_at: now,
            access_count: 0,
            last_accessed: now,
            version: self.manager.get_current_version().await,
        };

        debug!("Storing result in memory cache: {}", string_key);

        // Store in memory cache
        self.memory_cache
            .insert(string_key.clone(), cached_result.clone())
            .await;

        // Store in disk cache if available
        if let Some(disk_db) = self.disk_cache.write().await.as_ref() {
            let serialized = Self::serialize_cached_result(&cached_result)?;
            if let Err(e) = disk_db.insert(string_key.as_bytes(), serialized.as_bytes()) {
                error!("Failed to store in disk cache: {}", e);
            } else {
                debug!("Stored result in disk cache: {}", string_key);
            }
        }

        // Track dependencies
        self.manager.track_dependencies(key.clone()).await;

        Ok(())
    }

    /// Invalidate cache entries
    #[instrument(skip(self), err)]
    pub async fn invalidate(&self, keys: &[CacheKey]) -> CacheResult<()> {
        debug!("Invalidating {} cache entries", keys.len());

        let mut invalidated_keys = Vec::new();

        for key in keys {
            let string_key = key.to_string_key();

            // Invalidate from memory cache
            self.memory_cache.invalidate(&string_key).await;

            // Invalidate from disk cache if available
            if let Some(disk_db) = self.disk_cache.write().await.as_ref() {
                let _ = disk_db.remove(string_key.as_bytes());
            }

            invalidated_keys.push(key.clone());
        }

        // Notify invalidation listener
        if let Some(listener) = self.manager.invalidation_listener.read().await.as_ref() {
            listener(invalidated_keys);
        }

        Ok(())
    }

    /// Clear all cache entries
    #[instrument(skip(self), err)]
    pub async fn clear(&self) -> CacheResult<()> {
        info!("Clearing all cache entries");

        // Clear memory cache
        self.memory_cache.invalidate_all();
        self.memory_cache.run_pending_tasks().await;

        // Clear disk cache if available
        if let Some(disk_db) = self.disk_cache.write().await.as_ref() {
            disk_db.clear()?;
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn statistics(&self) -> CacheStatistics {
        let mut stats = (*self.statistics).clone();

        // Update hit rate
        if stats.total_accesses > 0 {
            stats.hit_rate = (stats.hits as f32 / stats.total_accesses as f32) * 100.0;
        }

        // Get memory cache size
        stats.memory_size = self.memory_cache.entry_count() as u64;

        // Get disk cache size
        if let Some(disk_db) = self.disk_cache.read().await.as_ref() {
            if let Ok(size_bytes) = disk_db.size_on_disk() {
                stats.disk_size = size_bytes as u64;
            }
        }

        stats
    }

    /// Check if cache contains key
    pub async fn contains(&self, key: &CacheKey) -> bool {
        let string_key = key.to_string_key();
        self.memory_cache.contains_key(&string_key)
    }

    /// Set invalidation listener
    pub async fn set_invalidation_listener(&self, listener: Box<dyn Fn(Vec<CacheKey>) + Send + Sync>) {
        *self.manager.invalidation_listener.write().await = Some(listener);
    }

    /// Perform cache maintenance (eviction, cleanup)
    #[instrument(skip(self))]
    pub async fn perform_maintenance(&self) -> CacheResult<()> {
        debug!("Performing cache maintenance");

        // Run memory cache maintenance
        self.memory_cache.run_pending_tasks().await;

        // Compact disk cache if available
        if let Some(disk_db) = self.disk_cache.write().await.as_ref() {
            if let Err(e) = disk_db.flush() {
                warn!("Failed to flush disk cache: {}", e);
            }
        }

        // Update last eviction timestamp
        {
            let mut stats = unsafe { &mut *(&*self.statistics as *const CacheStatistics as *mut CacheStatistics) };
            stats.last_eviction = Some(chrono::Utc::now().timestamp());
        }

        Ok(())
    }

    /// Initialize disk cache database
    async fn initialize_disk_cache(config: &CacheConfig) -> CacheResult<Db> {
        // Create cache directory in temp or app data
        let cache_dir = std::env::temp_dir().join("rust-ai-ide-real-time-analysis");

        if !cache_dir.exists() {
            tokio::fs::create_dir_all(&cache_dir).await?;
        }

        let db_path = cache_dir.join("analysis_cache.db");

        // Open sled database with size limits
        let db = sled::Config::default()
            .path(db_path)
            .cache_capacity(config.disk_cache_size as usize)
            .open()
            .map_err(|e| CacheError::Database(e.to_string()))?;

        info!("Initialized disk cache at: {:?}", cache_dir);
        Ok(db)
    }

    /// Serialize cached result for storage
    fn serialize_cached_result(cached: &CachedResult) -> CacheResult<String> {
        Ok(serde_json::to_string(cached)?)
    }

    /// Deserialize cached result from storage
    fn deserialize_cached_result(data: &str) -> CacheResult<CachedResult> {
        Ok(serde_json::from_str(data)?)
    }
}

impl CacheManager {
    /// Create a new cache manager
    fn new() -> Self {
        Self {
            dependency_tracker:    Arc::new(DependencyTracker::new()),
            invalidation_listener: Arc::new(RwLock::new(None)),
            current_version:       Arc::new(RwLock::new(1)),
        }
    }

    /// Get current cache version
    async fn get_current_version(&self) -> u32 {
        *self.current_version.read().await
    }

    /// Track dependencies for a cache key
    async fn track_dependencies(&self, key: CacheKey) {
        self.dependency_tracker
            .add_dependency(key.file_path.clone(), key);
    }

    /// Invalidate dependent cache entries
    async fn invalidate_dependencies(&self, file_path: &Path) -> Vec<CacheKey> {
        self.dependency_tracker.get_dependencies(file_path)
    }

    /// Bump cache version for invalidation
    async fn bump_version(&self) -> u32 {
        let mut version = self.current_version.write().await;
        *version += 1;
        *version
    }
}

impl DependencyTracker {
    /// Create a new dependency tracker
    fn new() -> Self {
        Self {
            file_dependencies:     DashMap::new(),
            invalidation_triggers: DashMap::new(),
        }
    }

    /// Add dependency relationship
    fn add_dependency(&self, file_path: PathBuf, key: CacheKey) {
        self.file_dependencies
            .entry(file_path)
            .or_insert_with(HashSet::new)
            .insert(key);
    }

    /// Get all dependencies for a file
    fn get_dependencies(&self, file_path: &Path) -> Vec<CacheKey> {
        if let Some(deps) = self.file_dependencies.get(file_path) {
            deps.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Remove dependencies for a file
    fn remove_dependencies(&self, file_path: &Path) {
        self.file_dependencies.remove(file_path);
    }
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            total_accesses: 0,
            hits:           0,
            misses:         0,
            hit_rate:       0.0,
            memory_size:    0,
            disk_size:      0,
            last_eviction:  None,
        }
    }
}

impl CachedResult {
    /// Check if cached result is expired based on TTL
    pub fn is_expired(&self, ttl: Duration) -> bool {
        let now = chrono::Utc::now().timestamp();
        (now - self.cached_at) > (ttl.as_secs() as i64)
    }

    /// Check if cached result has valid version
    pub fn has_valid_version(&self, current_version: u32) -> bool {
        self.version == current_version
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tempfile::TempDir;

    use super::*;

    fn create_test_cache_key() -> CacheKey {
        CacheKey {
            file_path:     PathBuf::from("test.rs"),
            analysis_type: AnalysisType::Syntax,
            file_mtime:    1234567890,
            file_size:     1024,
            file_hash:     "test_hash".to_string(),
            config_hash:   "config_hash".to_string(),
        }
    }

    fn create_test_analysis_result() -> AnalysisResult {
        use crate::types::{AnalysisMetadata, TaskPriority};

        let metadata = AnalysisMetadata {
            task_id:       "test-task".to_string(),
            file_path:     PathBuf::from("test.rs"),
            analysis_type: AnalysisType::Syntax,
            priority:      TaskPriority::Normal,
            start_time:    std::time::Instant::now(),
            metadata:      HashMap::new(),
        };

        AnalysisResult {
            metadata,
            findings: Vec::new(),
            duration: Duration::from_millis(100),
            success: true,
            error_message: None,
            performance_metrics: PerformanceMetrics::default(),
        }
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("Cargo.toml");

        // Create a test file
        std::fs::write(&test_file, "test content").unwrap();

        let key = CacheKey::from_file(&test_file, AnalysisType::Syntax)
            .await
            .unwrap();

        assert_eq!(key.file_path, test_file);
        assert_eq!(key.analysis_type, AnalysisType::Syntax);
        assert!(key.file_size > 0);
        assert!(!key.file_hash.is_empty());
    }

    #[tokio::test]
    async fn test_memory_cache_operations() {
        let config = CacheConfig {
            memory_cache_size:  1_000_000,
            disk_cache_size:    0, // Disable disk cache
            cache_ttl:          Duration::from_secs(3600),
            hit_time_threshold: Duration::from_millis(10),
        };

        let cache = AnalysisCache::new(config).await.unwrap();
        let key = create_test_cache_key();
        let result = create_test_analysis_result();

        // Test cache miss
        let retrieved = cache.get(&key).await.unwrap();
        assert!(retrieved.is_none());

        // Store result
        cache.put(key.clone(), result.clone()).await.unwrap();

        // Test cache hit
        let retrieved = cache.get(&key).await.unwrap();
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().success);

        // Test statistics
        let stats = cache.statistics().await;
        assert_eq!(stats.total_accesses, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let config = CacheConfig {
            memory_cache_size:  1_000_000,
            disk_cache_size:    0,
            cache_ttl:          Duration::from_secs(3600),
            hit_time_threshold: Duration::from_millis(10),
        };

        let cache = AnalysisCache::new(config).await.unwrap();
        let key = create_test_cache_key();
        let result = create_test_analysis_result();

        // Store result
        cache.put(key.clone(), result).await.unwrap();

        // Verify it's in cache
        assert!(cache.get(&key).await.unwrap().is_some());

        // Invalidate
        cache.invalidate(&[key.clone()]).await.unwrap();

        // Verify it's gone
        assert!(cache.get(&key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let config = CacheConfig {
            memory_cache_size:  1_000_000,
            disk_cache_size:    0,
            cache_ttl:          Duration::from_secs(3600),
            hit_time_threshold: Duration::from_millis(10),
        };

        let cache = AnalysisCache::new(config).await.unwrap();

        // Generate some cache activity
        for i in 0..10 {
            let key = CacheKey {
                file_path:     PathBuf::from(format!("test{}.rs", i)),
                analysis_type: AnalysisType::Syntax,
                file_mtime:    1234567890 + i as i64,
                file_size:     1024 + i as u64,
                file_hash:     format!("hash{}", i),
                config_hash:   "config".to_string(),
            };

            // Put and get to generate statistics
            let result = create_test_analysis_result();
            cache.put(key.clone(), result).await.unwrap();
            let _ = cache.get(&key).await.unwrap();
        }

        let stats = cache.statistics().await;
        assert_eq!(stats.total_accesses, 10);
        assert_eq!(stats.hits, 10);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 100.0);
    }

    #[test]
    fn test_cache_key_string_conversion() {
        let key = CacheKey {
            file_path:     PathBuf::from("/path/to/file.rs"),
            analysis_type: AnalysisType::Syntax,
            file_mtime:    1234567890,
            file_size:     1024,
            file_hash:     "test_hash".to_string(),
            config_hash:   "config_hash".to_string(),
        };

        let string_key = key.to_string_key();
        let parsed_key = CacheKey::from_string_key(&string_key).unwrap();

        assert_eq!(key.file_path, parsed_key.file_path);
        assert_eq!(key.analysis_type, parsed_key.analysis_type);
        assert_eq!(key.file_mtime, parsed_key.file_mtime);
        assert_eq!(key.file_size, parsed_key.file_size);
        assert_eq!(key.file_hash, parsed_key.file_hash);
    }
}
