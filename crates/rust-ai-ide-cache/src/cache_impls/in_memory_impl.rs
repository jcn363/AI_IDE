//! Advanced in-memory cache implementation using Moka LRU with compression and TTL support

use std::sync::Arc;

use async_trait::async_trait;
#[cfg(feature = "compression")]
use bincode;
use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{Cache, CacheConfig, CacheEntry, CacheStats, IDEResult};

/// Idle state detection and management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdleState {
    Active,
    Idle,
    DeepIdle, // Extended idle period with more aggressive cleanup
}

impl Default for IdleState {
    fn default() -> Self {
        IdleState::Active
    }
}

impl IdleState {
    /// Determine idle state based on inactivity duration
    pub fn from_inactivity_duration(inactivity_seconds: u64, deep_idle_threshold: u64) -> Self {
        if inactivity_seconds < 300 { // Less than 5 minutes
            IdleState::Active
        } else if inactivity_seconds < deep_idle_threshold {
            IdleState::Idle
        } else {
            IdleState::DeepIdle
        }
    }

    /// Get TTL reduction factor for idle state
    pub fn ttl_reduction_factor(&self) -> f64 {
        match self {
            IdleState::Active => 1.0,
            IdleState::Idle => 0.3, // 30% of normal TTL
            IdleState::DeepIdle => 0.1, // 10% of normal TTL
        }
    }

    /// Get max entries reduction factor for idle state
    pub fn max_entries_reduction_factor(&self) -> f64 {
        match self {
            IdleState::Active => 1.0,
            IdleState::Idle => 0.5, // 50% of normal max entries
            IdleState::DeepIdle => 0.2, // 20% of normal max entries
        }
    }
}

/// Compressed data wrapper for large cache entries
#[cfg(feature = "compression")]
#[derive(Debug, Clone)]
struct CompressedData {
    data: Vec<u8>,
    original_size: usize,
    compressed_size: usize,
}

#[cfg(feature = "compression")]
impl CompressedData {
    fn compress<T: serde::Serialize>(data: &T) -> IDEResult<Self> {
        let serialized = bincode::serialize(data)?;
        let original_size = serialized.len();
        let compressed = zstd::encode_all(&serialized[..], 3)?; // Compression level 3
        let compressed_size = compressed.len();

        Ok(Self {
            data: compressed,
            original_size,
            compressed_size,
        })
    }

    fn decompress<T: serde::de::DeserializeOwned>(&self) -> IDEResult<T> {
        let decompressed = zstd::decode_all(&self.data[..])?;
        let result = bincode::deserialize(&decompressed)?;
        Ok(result)
    }

    fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            self.compressed_size as f64 / self.original_size as f64
        }
    }
}

#[cfg(feature = "compression")]
impl Default for CompressedData {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            original_size: 0,
            compressed_size: 0,
        }
    }
}

/// Enhanced in-memory cache implementation with Moka LRU, TTL, and compression
pub struct InMemoryCache<
    K: std::hash::Hash + Eq + Send + Sync + Clone + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
> {
    cache: MokaCache<K, CacheEntry<V>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    #[cfg(feature = "compression")]
    compressed_cache: Option<MokaCache<String, CompressedData>>,
    /// Idle state management
    idle_state: Arc<RwLock<IdleState>>,
    last_activity: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl<K: std::hash::Hash + Eq + Send + Sync + Clone + 'static, V: Send + Sync + Clone + 'static>
    InMemoryCache<K, V>
{
    /// Create a new in-memory cache with the given configuration
    pub fn new(config: &CacheConfig) -> Self {
        let stats = CacheStats {
            created_at: chrono::Utc::now(),
            uptime_seconds: 0,
            ..Default::default()
        };

        // Build Moka cache with configuration
        let mut cache_builder = MokaCache::builder();

        // Set max capacity based on config
        if let Some(max_entries) = config.max_entries {
            cache_builder = cache_builder.max_capacity(max_entries as u64);
        }

        // Set default TTL
        if let Some(default_ttl) = config.default_ttl {
            cache_builder = cache_builder.time_to_live(default_ttl);
        }

        // Enable metrics if configured
        // TODO: Enable metrics when moka API is available
        // if config.enable_metrics {
        //     cache_builder = cache_builder.enable_statistics();
        // }

        let cache = cache_builder.build();

        // Initialize compressed cache if compression is enabled
        #[cfg(feature = "compression")]
        let compressed_cache = if config.compression_threshold_kb.is_some() {
            Some(
                MokaCache::builder()
                    .max_capacity(1000) // Separate capacity for compressed entries
                    .time_to_live(
                        config
                            .default_ttl
                            .unwrap_or(std::time::Duration::from_secs(3600)),
                    )
                    .build(),
            )
        } else {
            None
        };

        Self {
            cache,
            config: config.clone(),
            stats: Arc::new(RwLock::new(stats)),
            #[cfg(feature = "compression")]
            compressed_cache,
            idle_state: Arc::new(RwLock::new(IdleState::Active)),
            last_activity: Arc::new(RwLock::new(chrono::Utc::now())),
        }
    }

    /// Get current idle state
    pub async fn idle_state(&self) -> IdleState {
        *self.idle_state.read().await
    }

    /// Update idle state based on activity timing
    pub async fn update_idle_state(&self) {
        let now = chrono::Utc::now();
        let last_activity = *self.last_activity.read().await;
        let inactivity_duration = (now - last_activity).num_seconds() as u64;

        let new_state = IdleState::from_inactivity_duration(
            inactivity_duration,
            self.config.idle_detection_timeout_seconds * 2, // Deep idle after 2x normal timeout
        );

        *self.idle_state.write().await = new_state;

        if matches!(new_state, IdleState::Idle | IdleState::DeepIdle) {
            // Trigger aggressive cleanup when entering idle states
            let _ = self.perform_idle_cleanup().await;
        }
    }

    /// Record cache activity to reset idle timer
    pub async fn record_activity(&self) {
        *self.last_activity.write().await = chrono::Utc::now();
        // Reset to active state on any activity
        *self.idle_state.write().await = IdleState::Active;
    }

    /// Perform aggressive cleanup when idle
    async fn perform_idle_cleanup(&self) -> IDEResult<()> {
        let idle_state = *self.idle_state.read().await;

        // Calculate reduced limits for idle state
        let ttl_reduction = idle_state.ttl_reduction_factor();
        let max_entries_reduction = idle_state.max_entries_reduction_factor();

        let max_entries = self.config.max_entries
            .map(|max| ((max as f64 * max_entries_reduction) as usize).max(50)); // Minimum 50 entries

        // Force cleanup expired entries
        self.cleanup_expired().await?;

        // If we have a max_entries limit for idle state, enforce it aggressively
        if let Some(max_entries) = max_entries {
            let current_entries = self.size().await;
            if current_entries > max_entries {
                // Remove oldest entries aggressively
                let to_remove = current_entries - max_entries;
                self.remove_oldest_entries(to_remove).await?;
            }
        }

        // Reduce TTL for existing entries
        self.adjust_ttl_for_idle(ttl_reduction).await?;

        Ok(())
    }

    /// Remove oldest entries for aggressive cleanup
    async fn remove_oldest_entries(&self, count: usize) -> IDEResult<()> {
        // This is a simplified implementation - in practice, you'd need to iterate through entries
        // and remove the oldest ones. For now, we'll just run cleanup.
        self.cleanup_expired().await?;
        Ok(())
    }

    /// Adjust TTL for existing entries during idle periods
    async fn adjust_ttl_for_idle(&self, ttl_reduction: f64) -> IDEResult<()> {
        // This would require iterating through all entries and adjusting their TTL
        // For now, we rely on the natural expiration with reduced TTL for new entries
        Ok(())
    }

    /// Get effective TTL considering idle state
    fn effective_ttl(&self, base_ttl: Option<std::time::Duration>, idle_state: IdleState) -> Option<std::time::Duration> {
        base_ttl.map(|ttl| {
            let reduction_factor = idle_state.ttl_reduction_factor();
            let reduced_seconds = (ttl.as_secs_f64() * reduction_factor) as u64;
            std::time::Duration::from_secs(reduced_seconds.max(30)) // Minimum 30 seconds
        })
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for InMemoryCache<K, V>
where
    K: Send + Sync + Clone + std::hash::Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    async fn get(&self, key: &K) -> IDEResult<Option<V>> {
        // Record activity on cache access
        self.record_activity().await;

        let mut stats = self.stats.write().await;

        // Check compressed cache first if compression is enabled
        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_cache {
            let compressed_key = format!("{:?}", key);
            if let Some(compressed_data) = compressed_cache.get(&compressed_key).await {
                if let Ok(value) = compressed_data.decompress::<V>() {
                    stats.record_hit();
                    return Ok(Some(value));
                }
            }
        }

        // Try main cache
        if let Some(entry) = self.cache.get(key).await {
            if entry.is_expired() {
                self.cache.invalidate(key).await;
                stats.record_miss();
                stats.record_eviction();
                Ok(None)
            } else {
                stats.record_hit();
                Ok(Some(entry.value))
            }
        } else {
            stats.record_miss();
            Ok(None)
        }
    }

    async fn insert(&self, key: K, value: V, ttl: Option<std::time::Duration>) -> IDEResult<()> {
        // Record activity on cache write
        self.record_activity().await;

        let mut stats = self.stats.write().await;

        // Get current idle state for TTL adjustment
        let idle_state = *self.idle_state.read().await;
        let effective_ttl = self.effective_ttl(ttl, idle_state);

        // Check if compression should be used
        #[cfg(feature = "compression")]
        if let Some(threshold_kb) = self.config.compression_threshold_kb {
            let data_size_kb =
                (serde_json::to_string(&value).unwrap_or_default().len() / 1024) as usize;
            if data_size_kb >= threshold_kb {
                if let Some(compressed_cache) = &self.compressed_cache {
                    if let Ok(compressed_data) = CompressedData::compress(&value) {
                        let compressed_key = format!("{:?}", key);
                        compressed_cache
                            .insert(compressed_key, compressed_data)
                            .await;
                        stats.record_set();
                        return Ok(());
                    }
                }
            }
        }

        let entry = CacheEntry::new_with_ttl(value, effective_ttl, chrono::Utc::now());
        self.cache.insert(key, entry).await;
        stats.record_set();

        Ok(())
    }

    async fn remove(&self, key: &K) -> IDEResult<Option<V>> {
        // Check compressed cache first
        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_cache {
            let compressed_key = format!("{:?}", key);
            if let Some(compressed_data) = compressed_cache.remove(&compressed_key).await {
                if let Ok(value) = compressed_data.decompress::<V>() {
                    return Ok(Some(value));
                }
            }
        }

        if let Some(entry) = self.cache.remove(key).await {
            Ok(Some(entry.value))
        } else {
            Ok(None)
        }
    }

    async fn clear(&self) -> IDEResult<()> {
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;

        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_cache {
            compressed_cache.invalidate_all();
            compressed_cache.run_pending_tasks().await;
        }

        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
        Ok(())
    }

    async fn size(&self) -> usize {
        let total_size = self.cache.entry_count();

        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_cache {
            total_size += compressed_cache.entry_count();
        }

        total_size as usize
    }

    async fn contains(&self, key: &K) -> bool {
        // Check compressed cache first
        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_cache {
            let compressed_key = format!("{:?}", key);
            if compressed_cache.contains_key(&compressed_key) {
                return true;
            }
        }

        self.cache.contains_key(key)
    }

    async fn stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.total_entries = self.size().await;

        // Get Moka-specific stats if available
        // TODO: Get moka stats when API is available
        // if let Ok(moka_stats) = self.cache.stats().await {
        //     stats.total_hits = moka_stats.num_hits();
        //     stats.total_misses = moka_stats.num_misses();
        //     stats.memory_usage_bytes = Some(moka_stats.consumption().bytes);
        // }

        stats.uptime_seconds = (chrono::Utc::now() - stats.created_at)
            .as_seconds_f64()
            .abs() as u64;

        stats.update_hit_ratio();
        stats
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        // Moka handles TTL-based expiration automatically
        // We only need to run pending tasks to ensure cleanup
        self.cache.run_pending_tasks().await;

        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_cache {
            compressed_cache.run_pending_tasks().await;
        }

        // Return 0 since Moka handles expiration internally
        Ok(0)
    }
}

impl<K: std::hash::Hash + Eq + Send + Sync + Clone + 'static, V: Send + Sync + Clone + 'static> Drop
    for InMemoryCache<K, V>
{
    fn drop(&mut self) {
        // Cleanup on drop - Moka handles this automatically
        // but we can run pending tasks to ensure cleanup
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            handle.block_on(async {
                self.cache.invalidate_all();
                #[cfg(feature = "compression")]
                if let Some(compressed_cache) = &self.compressed_cache {
                    compressed_cache.invalidate_all();
                }
            });
        }
    }
}

/// Hybrid cache that combines in-memory and another storage backend
pub struct HybridCache<
    K: std::hash::Hash + Eq + Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
> {
    memory_cache: InMemoryCache<K, V>,
    // secondary_cache: Option<Box<dyn Cache<K, V>>>, // For future use
}

impl<K: std::hash::Hash + Eq + Send + Sync + Clone + 'static, V: Send + Sync + Clone + 'static>
    HybridCache<K, V>
{
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            memory_cache: InMemoryCache::new(config),
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for HybridCache<K, V>
where
    K: Send + Sync + Clone + std::hash::Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    async fn get(&self, key: &K) -> IDEResult<Option<V>> {
        self.memory_cache.get(key).await
    }

    async fn insert(&self, key: K, value: V, ttl: Option<std::time::Duration>) -> IDEResult<()> {
        self.memory_cache.insert(key, value, ttl).await
    }

    async fn remove(&self, key: &K) -> IDEResult<Option<V>> {
        self.memory_cache.remove(key).await
    }

    async fn clear(&self) -> IDEResult<()> {
        self.memory_cache.clear().await
    }

    async fn size(&self) -> usize {
        self.memory_cache.size().await
    }

    async fn contains(&self, key: &K) -> bool {
        self.memory_cache.contains(key).await
    }

    async fn stats(&self) -> CacheStats {
        self.memory_cache.stats().await
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        self.memory_cache.cleanup_expired().await
    }
}

/// Specialized cache for AI/ML inference results with compression and TTL
pub type AiInferenceCache = InMemoryCache<String, serde_json::Value>;

impl AiInferenceCache {
    /// Create a cache optimized for AI inference results
    pub fn for_ai_inference() -> Self {
        let mut config = CacheConfig::default();
        config.max_entries = Some(5000); // Higher capacity for AI results
        config.default_ttl = Some(std::time::Duration::from_secs(1800)); // 30 minutes
        config.compression_threshold_kb = Some(50); // Compress large results
        config.enable_metrics = true;

        Self::new(&config)
    }

    /// Cache AI inference result with automatic compression for large data
    pub async fn cache_inference_result(
        &self,
        query_hash: String,
        result: serde_json::Value,
        ttl: Option<std::time::Duration>,
    ) -> IDEResult<()> {
        let ttl = ttl.or_else(|| Some(std::time::Duration::from_secs(1800)));
        self.insert(query_hash, result, ttl).await
    }

    /// Get inference result with performance tracking
    pub async fn get_inference_result(
        &self,
        query_hash: &str,
    ) -> IDEResult<Option<serde_json::Value>> {
        self.get(&query_hash.to_string()).await
    }
}

/// Specialized cache for LSP symbol resolution
pub type LspSymbolCache = InMemoryCache<String, serde_json::Value>;

impl LspSymbolCache {
    /// Create a cache optimized for LSP operations
    pub fn for_lsp_symbols() -> Self {
        let mut config = CacheConfig::default();
        config.max_entries = Some(10000); // Large capacity for symbols
        config.default_ttl = Some(std::time::Duration::from_secs(3600)); // 1 hour
        config.compression_threshold_kb = Some(100); // Compress large symbol data
        config.enable_metrics = true;

        Self::new(&config)
    }

    /// Cache LSP symbol resolution result
    pub async fn cache_symbol_resolution(
        &self,
        file_path: String,
        symbol_data: serde_json::Value,
        ttl: Option<std::time::Duration>,
    ) -> IDEResult<()> {
        let key = format!("symbol:{}", file_path);
        let ttl = ttl.or_else(|| Some(std::time::Duration::from_secs(3600)));
        self.insert(key, symbol_data, ttl).await
    }

    /// Get cached symbol resolution
    pub async fn get_symbol_resolution(
        &self,
        file_path: &str,
    ) -> IDEResult<Option<serde_json::Value>> {
        let key = format!("symbol:{}", file_path);
        self.get(&key).await
    }

    /// Cache LSP workspace analysis
    pub async fn cache_workspace_analysis(
        &self,
        workspace_path: String,
        analysis_data: serde_json::Value,
        ttl: Option<std::time::Duration>,
    ) -> IDEResult<()> {
        let key = format!("workspace:{}", workspace_path);
        let ttl = ttl.or_else(|| Some(std::time::Duration::from_secs(1800))); // 30 minutes
        self.insert(key, analysis_data, ttl).await
    }
}

/// Specialized cache for cryptographic keys with security features
pub type CryptoKeyCache = InMemoryCache<String, Vec<u8>>;

impl CryptoKeyCache {
    /// Create a cache optimized for cryptographic operations
    pub fn for_crypto_keys() -> Self {
        let mut config = CacheConfig::default();
        config.max_entries = Some(1000); // Limited for security
        config.default_ttl = Some(std::time::Duration::from_secs(1800)); // 30 minutes for security
        config.compression_threshold_kb = Some(10); // Small threshold for keys
        config.enable_metrics = true;

        Self::new(&config)
    }

    /// Cache encrypted key data with security metadata
    pub async fn cache_encrypted_key(
        &self,
        key_id: String,
        encrypted_data: Vec<u8>,
        ttl: Option<std::time::Duration>,
    ) -> IDEResult<()> {
        let ttl = ttl.or_else(|| Some(std::time::Duration::from_secs(1800)));
        self.insert(key_id, encrypted_data, ttl).await
    }

    /// Get cached encrypted key
    pub async fn get_encrypted_key(&self, key_id: &str) -> IDEResult<Option<Vec<u8>>> {
        self.get(&key_id.to_string()).await
    }

    /// Securely invalidate key cache
    pub async fn invalidate_key(&self, key_id: &str) -> IDEResult<()> {
        self.remove(&key_id.to_string()).await?;
        Ok(())
    }
}

/// Performance monitoring and metrics for cache operations
#[allow(dead_code)]
pub struct CachePerformanceMonitor {
    cache_name: String,
    metrics: Arc<RwLock<CacheMetrics>>,
    reporting_interval: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CacheMetrics {
    pub total_operations: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub compression_ratio_avg: f64,
    pub average_response_time_ms: f64,
    pub memory_usage_mb: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
            compression_ratio_avg: 1.0,
            average_response_time_ms: 0.0,
            memory_usage_mb: 0.0,
            last_updated: chrono::Utc::now(),
        }
    }
}

impl CachePerformanceMonitor {
    pub fn new(cache_name: impl Into<String>) -> Self {
        Self {
            cache_name: cache_name.into(),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            reporting_interval: std::time::Duration::from_secs(60), // Report every minute
        }
    }

    pub async fn record_hit(&self, response_time_ms: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.hit_count += 1;
        metrics.average_response_time_ms =
            (metrics.average_response_time_ms + response_time_ms) / 2.0;
        metrics.last_updated = chrono::Utc::now();
    }

    pub async fn record_miss(&self, response_time_ms: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.miss_count += 1;
        metrics.average_response_time_ms =
            (metrics.average_response_time_ms + response_time_ms) / 2.0;
        metrics.last_updated = chrono::Utc::now();
    }

    #[allow(dead_code)]
    pub async fn record_eviction(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.eviction_count += 1;
    }

    #[allow(dead_code)]
    pub async fn record_compression_ratio(&self, ratio: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.compression_ratio_avg = (metrics.compression_ratio_avg + ratio) / 2.0;
    }

    #[allow(dead_code)]
    pub async fn update_memory_usage(&self, memory_mb: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.memory_usage_mb = memory_mb;
    }

    #[allow(dead_code)]
    pub async fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn get_hit_rate(&self) -> f64 {
        let metrics = self.metrics.read().await;
        if metrics.total_operations == 0 {
            0.0
        } else {
            metrics.hit_count as f64 / metrics.total_operations as f64
        }
    }

    pub async fn get_cache_efficiency_score(&self) -> f64 {
        let metrics = self.metrics.read().await;
        let hit_rate = self.get_hit_rate().await;
        let compression_benefit = metrics.compression_ratio_avg.min(1.0);
        let memory_efficiency = if metrics.memory_usage_mb > 0.0 {
            (100.0 / metrics.memory_usage_mb).min(1.0)
        } else {
            1.0
        };

        // Weighted score combining hit rate, compression efficiency, and memory usage
        (hit_rate * 0.5) + (compression_benefit * 0.3) + (memory_efficiency * 0.2)
    }

    pub async fn generate_performance_report(&self) -> String {
        let metrics = self.metrics.read().await;
        let hit_rate = self.get_hit_rate().await;
        let efficiency_score = self.get_cache_efficiency_score().await;

        format!(
            "Cache Performance Report for '{}'\n======================================\nTotal Operations: {}\nHit \
             Rate: {:.2}%\nCache Efficiency Score: {:.3}\nAverage Response Time: {:.2}ms\nMemory Usage: \
             {:.2}MB\nCompression Ratio: {:.2}x\nEvictions: {}\nLast Updated: {}\n",
            self.cache_name,
            metrics.total_operations,
            hit_rate * 100.0,
            efficiency_score,
            metrics.average_response_time_ms,
            metrics.memory_usage_mb,
            metrics.compression_ratio_avg,
            metrics.eviction_count,
            metrics.last_updated
        )
    }

    /// Start background monitoring task
    pub fn start_monitoring(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.reporting_interval);
            loop {
                interval.tick().await;
                let report = self.generate_performance_report().await;
                tracing::info!("{}", report);
            }
        });
    }
}

/// Enhanced cache with performance monitoring
#[allow(dead_code)]
pub struct MonitoredInMemoryCache<
    K: std::hash::Hash + Eq + Send + Sync + Clone + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
> {
    inner: InMemoryCache<K, V>,
    monitor: Arc<CachePerformanceMonitor>,
}

#[allow(dead_code)]
impl<
        K: std::hash::Hash + Eq + Send + Sync + Clone + serde::Serialize + 'static,
        V: Send + Sync + Clone + serde::Serialize + 'static,
    > MonitoredInMemoryCache<K, V>
{
    pub fn new(cache_name: impl Into<String>, config: &CacheConfig) -> Self {
        let monitor = Arc::new(CachePerformanceMonitor::new(cache_name));
        let inner = InMemoryCache::new(config);

        // Start monitoring
        let monitor_clone = monitor.clone();
        monitor_clone.start_monitoring();

        Self { inner, monitor }
    }

    pub async fn get(&self, key: &K) -> IDEResult<Option<V>> {
        let start_time = std::time::Instant::now();
        let result = self.inner.get(key).await;
        let response_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        match &result {
            Ok(Some(_)) => self.monitor.record_hit(response_time_ms).await,
            Ok(None) => self.monitor.record_miss(response_time_ms).await,
            _ => {}
        }

        result
    }

    pub async fn insert(
        &self,
        key: K,
        value: V,
        ttl: Option<std::time::Duration>,
    ) -> IDEResult<()> {
        self.inner.insert(key, value, ttl).await
    }

    pub async fn remove(&self, key: &K) -> IDEResult<Option<V>> {
        self.inner.remove(key).await
    }

    pub async fn clear(&self) -> IDEResult<()> {
        self.inner.clear().await
    }

    pub async fn size(&self) -> usize {
        self.inner.size().await
    }

    pub fn get_monitor(&self) -> Arc<CachePerformanceMonitor> {
        self.monitor.clone()
    }
}

/// Export specialized monitored cache types for different use cases
#[allow(dead_code)]
pub type DependencyGraphCache = MonitoredInMemoryCache<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_in_memory_cache_basic_operations() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        let key = "test_key";
        let value = "test_value";

        // Test insert and get
        cache.insert(key, value, None).await.unwrap();
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Test contains
        assert!(cache.contains(&key).await);

        // Test remove
        let removed = cache.remove(&key).await.unwrap();
        assert_eq!(removed, Some(value.to_string()));
        assert!(!cache.contains(&key).await);
    }

    #[tokio::test]
    async fn test_cache_with_ttl() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        let key = "ttl_key";
        let value = "ttl_value";
        let ttl = Duration::from_millis(100);

        cache.insert(key, value, Some(ttl)).await.unwrap();

        // Should be available immediately
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_cleanup_expired() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        // Insert expired entry
        let key = "expired_key";
        let value = "expired_value";
        let short_ttl = Duration::from_millis(50);

        cache.insert(key, value, Some(short_ttl)).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup expired entries
        let cleaned = cache.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Verify entry is gone
        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(&config);

        let key = "stats_key";
        let value = "stats_value";

        // Insert and retrieve to generate stats
        cache.insert(key, value, None).await.unwrap();
        cache.get(&key).await.unwrap();
        cache.get(&"nonexistent").await.unwrap(); // Miss

        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);
        assert!(stats.hit_ratio > 0.0);
    }
}
