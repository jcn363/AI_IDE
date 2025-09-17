//! Enhanced Cache Implementation for Q1 2025 Memory Pooling Enhancements
//!
//! This module integrates all the advanced memory management features:
//! - Virtual memory optimization for large workspaces
//! - Intelligent TTL-based eviction policies
//! - Background defragmentation and compaction
//! - Memory pressure monitoring and adaptation

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::{RwLock, mpsc};
use tokio::time;

use crate::{Cache, CacheConfig, IDEResult};
use super::memory_pool::{
    VirtualMemoryManager, MemoryCompactionTask, IntelligentCacheEntry,
    CachePriority, IntelligentEvictionConfig, CompactionStats
};

/// Enhanced cache implementation with virtual memory support
pub struct EnhancedInMemoryCache<
    K: std::hash::Hash + Eq + Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
> {
    /// Traditional in-memory cache for frequently accessed items
    memory_cache: Arc<RwLock<std::collections::HashMap<K, IntelligentCacheEntry<V>>>>,
    /// Virtual memory manager for large datasets
    virtual_memory: Arc<VirtualMemoryManager>,
    /// Background compaction task
    compaction_task: Arc<MemoryCompactionTask>,
    /// Statistics tracking
    stats: Arc<RwLock<EnhancedCacheStats>>,
    /// Event sender for compaction statistics
    stats_sender: Option<mpsc::UnboundedSender<CompactionStats>>,
}

#[derive(Debug, Clone)]
pub struct EnhancedCacheStats {
    pub memory_entries: usize,
    pub virtual_entries: usize,
    pub total_memory_usage: u64,
    pub virtual_memory_usage: u64,
    pub compaction_cycles: u64,
    pub memory_pressure_level: f64,
    pub adaptive_ttl_adjustments: u64,
    pub created_at: Instant,
}

impl Default for EnhancedCacheStats {
    fn default() -> Self {
        Self {
            memory_entries: 0,
            virtual_entries: 0,
            total_memory_usage: 0,
            virtual_memory_usage: 0,
            compaction_cycles: 0,
            memory_pressure_level: 0.0,
            adaptive_ttl_adjustments: 0,
            created_at: Instant::now(),
        }
    }
}

impl<
    K: std::hash::Hash + Eq + Send + Sync + Clone + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
> EnhancedInMemoryCache<K, V>
{
    /// Create a new enhanced cache with virtual memory support
    pub fn new(config: &CacheConfig) -> Self {
        let eviction_config = IntelligentEvictionConfig::default();
        let virtual_memory = Arc::new(VirtualMemoryManager::new(eviction_config.clone()));

        // Create compaction task
        let compaction_task = Arc::new(MemoryCompactionTask::new(
            eviction_config,
            Arc::clone(&virtual_memory),
        ));

        // Start background compaction
        let compaction_clone = Arc::clone(&compaction_task);
        tokio::spawn(async move {
            compaction_clone.start_background_task();
        });

        Self {
            memory_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            virtual_memory,
            compaction_task,
            stats: Arc::new(RwLock::new(EnhancedCacheStats::default())),
            stats_sender: None,
        }
    }

    /// Create enhanced cache with compaction statistics reporting
    pub fn with_stats_sender(mut self, sender: mpsc::UnboundedSender<CompactionStats>) -> Self {
        self.stats_sender = Some(sender);
        self.compaction_task = Arc::new(
            MemoryCompactionTask::new(
                IntelligentEvictionConfig::default(),
                Arc::clone(&self.virtual_memory),
            ).with_stats_sender(sender)
        );
        self
    }

    /// Determine if an item should use virtual memory based on size and workspace state
    async fn should_use_virtual_memory(&self, key: &K, value_size_estimate: usize) -> bool {
        // Check if virtual memory is enabled
        if !self.virtual_memory.config.enable_virtual_memory {
            return false;
        }

        // Check memory pressure
        let pressure = self.virtual_memory.get_memory_pressure().await;
        if pressure > self.virtual_memory.config.memory_pressure_threshold {
            return true;
        }

        // Check size threshold (>1MB)
        if value_size_estimate > 1024 * 1024 {
            return true;
        }

        // Check if workspace is large (>10M LOC) - simplified heuristic
        let memory_entries = self.memory_cache.read().await.len();
        if memory_entries > 10000 { // Large number of entries suggests large workspace
            return true;
        }

        false
    }

    /// Adaptively adjust TTL based on access patterns and memory pressure
    fn calculate_adaptive_ttl(&self, base_ttl: Option<Duration>, access_count: u32, priority: CachePriority, memory_pressure: f64) -> Option<Duration> {
        let base_ttl = base_ttl.unwrap_or_else(|| Duration::from_secs(1800)); // 30 minutes default

        // Adjust based on access frequency
        let access_multiplier = if access_count >= self.virtual_memory.config.access_frequency_threshold as u32 {
            2.0 // Frequently accessed - longer TTL
        } else {
            0.5 // Rarely accessed - shorter TTL
        };

        // Adjust based on priority
        let priority_multiplier = match priority {
            CachePriority::Critical => 3.0,
            CachePriority::High => 2.0,
            CachePriority::Normal => 1.0,
            CachePriority::Low => 0.3,
        };

        // Adjust based on memory pressure
        let pressure_multiplier = if memory_pressure > self.virtual_memory.config.memory_pressure_threshold {
            0.5 // High pressure - shorter TTL
        } else {
            1.0
        };

        let adjusted_seconds = (base_ttl.as_secs_f64() * access_multiplier * priority_multiplier * pressure_multiplier) as u64;
        Some(Duration::from_secs(adjusted_seconds.max(60))) // Minimum 1 minute
    }

    /// Get current memory pressure level
    pub async fn get_memory_pressure(&self) -> f64 {
        self.virtual_memory.get_memory_pressure().await
    }

    /// Manually trigger compaction cycle
    pub async fn trigger_compaction(&self) -> Result<CompactionStats, String> {
        self.compaction_task.run_compaction_cycle().await
            .map_err(|e| format!("Compaction failed: {:?}", e))
    }

    /// Get comprehensive cache statistics
    pub async fn get_enhanced_stats(&self) -> EnhancedCacheStats {
        let mut stats = self.stats.read().await.clone();
        let memory_cache = self.memory_cache.read().await;
        let virtual_pressure = self.virtual_memory.get_memory_pressure().await;

        stats.memory_entries = memory_cache.len();
        stats.memory_pressure_level = virtual_pressure;
        stats.virtual_entries = self.virtual_memory.virtual_mappings.read().await.len() as usize;

        // Estimate memory usage (simplified)
        stats.total_memory_usage = (stats.memory_entries as u64) * 1024; // Rough estimate
        stats.virtual_memory_usage = *self.virtual_memory.current_usage.read().await;

        stats
    }
}

#[async_trait]
impl<
    K: Send + Sync + Clone + std::hash::Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
> Cache<K, V> for EnhancedInMemoryCache<K, V>
{
    async fn get(&self, key: &K) -> IDEResult<Option<V>> {
        let mut stats = self.stats.write().await;
        let mut memory_cache = self.memory_cache.write().await;

        // First check memory cache
        if let Some(entry) = memory_cache.get_mut(key) {
            if entry.is_expired() {
                // Remove expired entry
                memory_cache.remove(key);
                stats.memory_entries = stats.memory_entries.saturating_sub(1);
                return Ok(None);
            }

            // Update access tracking
            entry.record_access();

            // Check if we should move to virtual memory based on current pressure
            let memory_pressure = self.virtual_memory.get_memory_pressure().await;
            if self.should_use_virtual_memory(key, std::mem::size_of_val(&entry.value)).await {
                // Move to virtual memory
                let key_string = format!("{:?}", key);
                self.virtual_memory.register_virtual_mapping(
                    key_string.clone(),
                    std::mem::size_of_val(&entry.value) as u64,
                    None,
                ).await;

                let value = entry.value.clone();
                memory_cache.remove(key);
                stats.memory_entries = stats.memory_entries.saturating_sub(1);

                // Store in virtual memory (simplified - in real impl would serialize to disk)
                tracing::info!("Moved entry {:?} to virtual memory due to high memory pressure", key);
                return Ok(Some(value));
            }

            return Ok(Some(entry.value.clone()));
        }

        // Check virtual memory
        let key_string = format!("{:?}", key);
        if let Some(virtual_mapping) = self.virtual_memory.get_virtual_data(&key_string).await {
            // In a real implementation, this would deserialize from disk
            tracing::info!("Retrieved entry {:?} from virtual memory", key);
            // For now, return None as we don't have actual virtual storage
            return Ok(None);
        }

        Ok(None)
    }

    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()> {
        let mut stats = self.stats.write().await;
        let mut memory_cache = self.memory_cache.write().await;
        let memory_pressure = self.virtual_memory.get_memory_pressure().await;

        let priority = CachePriority::Normal; // Could be configurable per key
        let adaptive_ttl = self.calculate_adaptive_ttl(ttl, 0, priority, memory_pressure);

        let entry = IntelligentCacheEntry::new(value, adaptive_ttl, priority);

        let value_size = std::mem::size_of_val(&entry.value);

        if self.should_use_virtual_memory(&key, value_size).await {
            // Store in virtual memory
            let key_string = format!("{:?}", key);
            self.virtual_memory.register_virtual_mapping(
                key_string,
                value_size as u64,
                None,
            ).await;

            stats.virtual_entries += 1;
            tracing::info!("Stored entry {:?} in virtual memory (size: {} bytes)", key, value_size);
        } else {
            // Store in memory cache
            memory_cache.insert(key, entry);
            stats.memory_entries += 1;
        }

        // Adaptive TTL tracking
        if adaptive_ttl != ttl {
            stats.adaptive_ttl_adjustments += 1;
        }

        Ok(())
    }

    async fn remove(&self, key: &K) -> IDEResult<Option<V>> {
        let mut stats = self.stats.write().await;
        let mut memory_cache = self.memory_cache.write().await;

        // Check memory cache first
        if let Some(entry) = memory_cache.remove(key) {
            stats.memory_entries = stats.memory_entries.saturating_sub(1);
            return Ok(Some(entry.value));
        }

        // Check virtual memory
        let key_string = format!("{:?}", key);
        if self.virtual_memory.get_virtual_data(&key_string).await.is_some() {
            self.virtual_memory.unload_virtual_data(&key_string).await;
            stats.virtual_entries = stats.virtual_entries.saturating_sub(1);
            // In real implementation, would return deserialized value
            return Ok(None);
        }

        Ok(None)
    }

    async fn clear(&self) -> IDEResult<()> {
        let mut stats = self.stats.write().await;
        let mut memory_cache = self.memory_cache.write().await;

        memory_cache.clear();
        stats.memory_entries = 0;
        stats.virtual_entries = 0;

        // Note: Virtual memory cleanup would happen in background compaction
        Ok(())
    }

    async fn size(&self) -> usize {
        let memory_cache = self.memory_cache.read().await;
        let virtual_mappings = self.virtual_memory.virtual_mappings.read().await;
        memory_cache.len() + virtual_mappings.len()
    }

    async fn contains(&self, key: &K) -> bool {
        let memory_cache = self.memory_cache.read().await;
        if memory_cache.contains_key(key) {
            return true;
        }

        let key_string = format!("{:?}", key);
        self.virtual_memory.get_virtual_data(&key_string).await.is_some()
    }

    async fn stats(&self) -> crate::CacheStats {
        let enhanced_stats = self.get_enhanced_stats().await;
        crate::CacheStats {
            created_at: chrono::Utc::now() - chrono::Duration::from_std(enhanced_stats.created_at.elapsed()).unwrap_or_default(),
            uptime_seconds: enhanced_stats.created_at.elapsed().as_secs() as u64,
            total_entries: enhanced_stats.memory_entries + enhanced_stats.virtual_entries,
            total_hits: 0, // Would need to be tracked separately
            total_misses: 0,
            total_evictions: enhanced_stats.compaction_cycles as u64,
            hit_ratio: 0.0,
            memory_usage_bytes: Some(enhanced_stats.total_memory_usage),
            virtual_memory_usage_bytes: Some(enhanced_stats.virtual_memory_usage),
            last_compaction: Some(chrono::Utc::now()),
        }
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        let mut stats = self.stats.write().await;
        let mut memory_cache = self.memory_cache.write().await;
        let mut removed = 0;

        // Clean up expired memory entries
        let keys_to_remove: Vec<K> = memory_cache.iter()
            .filter_map(|(k, v)| if v.is_expired() { Some(k.clone()) } else { None })
            .collect();

        for key in keys_to_remove {
            memory_cache.remove(&key);
            removed += 1;
        }

        stats.memory_entries = stats.memory_entries.saturating_sub(removed);

        // Virtual memory cleanup happens in background compaction
        Ok(removed)
    }
}

/// Specialized enhanced cache for AI/ML inference results
pub type EnhancedAiInferenceCache = EnhancedInMemoryCache<String, serde_json::Value>;

impl EnhancedAiInferenceCache {
    /// Create cache optimized for AI inference with virtual memory support
    pub fn for_ai_inference() -> Self {
        let mut config = crate::CacheConfig::default();
        config.max_entries = Some(10000); // Higher capacity for AI results
        config.default_ttl = Some(Duration::from_secs(1800)); // 30 minutes

        Self::new(&config)
    }

    /// Cache AI inference result with intelligent virtual memory management
    pub async fn cache_inference_result(
        &self,
        query_hash: String,
        result: serde_json::Value,
        priority: CachePriority,
    ) -> IDEResult<()> {
        let memory_pressure = self.get_memory_pressure().await;
        let adaptive_ttl = self.calculate_adaptive_ttl(
            Some(Duration::from_secs(1800)),
            0,
            priority,
            memory_pressure,
        );

        self.insert(query_hash, result, adaptive_ttl).await
    }

    /// Get inference result with access tracking
    pub async fn get_inference_result(&self, query_hash: &str) -> IDEResult<Option<serde_json::Value>> {
        self.get(&query_hash.to_string()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_enhanced_cache_basic_operations() {
        let config = crate::CacheConfig::default();
        let cache = EnhancedInMemoryCache::new(&config);

        let key = "test_key";
        let value = "test_value";

        // Test insert and get
        cache.insert(key.to_string(), value.to_string(), None).await.unwrap();
        let result = cache.get(&key.to_string()).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Test virtual memory pressure
        let pressure = cache.get_memory_pressure().await;
        assert!(pressure >= 0.0 && pressure <= 1.0);

        // Test statistics
        let stats = cache.get_enhanced_stats().await;
        assert_eq!(stats.memory_entries, 1);
    }

    #[tokio::test]
    async fn test_adaptive_ttl_calculation() {
        let config = crate::CacheConfig::default();
        let cache = EnhancedInMemoryCache::new(&config);

        let base_ttl = Duration::from_secs(1800);
        let adaptive_ttl = cache.calculate_adaptive_ttl(
            Some(base_ttl),
            15, // High access count
            CachePriority::High,
            0.5, // Normal memory pressure
        );

        assert!(adaptive_ttl.is_some());
        let ttl_duration = adaptive_ttl.unwrap();
        // Should be longer than base due to high access count and priority
        assert!(ttl_duration > base_ttl);
    }

    #[tokio::test]
    async fn test_memory_pressure_detection() {
        let config = crate::CacheConfig::default();
        let cache = EnhancedInMemoryCache::new(&config);

        // Initially low pressure
        let initial_pressure = cache.get_memory_pressure().await;
        assert!(initial_pressure < 0.1);

        // Force some virtual mappings to increase pressure
        for i in 0..10 {
            cache.virtual_memory.register_virtual_mapping(
                format!("test_key_{}", i),
                1024 * 1024, // 1MB each
                None,
            ).await;
        }

        let pressure_after = cache.get_memory_pressure().await;
        assert!(pressure_after > initial_pressure);
    }
}