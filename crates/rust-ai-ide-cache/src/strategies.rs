//! Cache Eviction Strategies
//!
//! This module defines various eviction strategies for the cache.
//! This consolidates the scattered eviction logic found throughout the codebase.

use super::*;

pub trait EvictionStrategy<K, V> {
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
        V: serde::Serialize;
}

/// Least Recently Used eviction strategy
pub struct LruStrategy;

impl<K, V> EvictionStrategy<K, V> for LruStrategy {
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        let mut sorted_entries: Vec<_> = entries.iter().enumerate().collect();
        sorted_entries.sort_by(|a, b| a.1 .1.last_accessed.cmp(&b.1 .1.last_accessed));

        sorted_entries
            .into_iter()
            .take(target_count)
            .map(|(idx, _)| entries[idx].0.clone())
            .collect()
    }
}

/// Least Frequently Used eviction strategy
pub struct LfuStrategy;

impl<K, V> EvictionStrategy<K, V> for LfuStrategy {
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        let mut sorted_entries: Vec<_> = entries.iter().enumerate().collect();
        sorted_entries.sort_by(|a, b| a.1 .1.access_count.cmp(&b.1 .1.access_count));

        sorted_entries
            .into_iter()
            .take(target_count)
            .map(|(idx, _)| entries[idx].0.clone())
            .collect()
    }
}

/// First In First Out eviction strategy
pub struct FifoStrategy;

impl<K, V> EvictionStrategy<K, V> for FifoStrategy {
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        let mut sorted_entries: Vec<_> = entries.iter().enumerate().collect();
        sorted_entries.sort_by(|a, b| a.1 .1.created_at.cmp(&b.1 .1.created_at));

        sorted_entries
            .into_iter()
            .take(target_count)
            .map(|(idx, _)| entries[idx].0.clone())
            .collect()
    }
}

/// Size-based eviction strategy with memory awareness
pub struct SizeStrategy {
    pub max_memory_usage: usize,
    pub current_memory: std::sync::atomic::AtomicUsize,
    pub priority_weights: std::sync::RwLock<std::collections::HashMap<String, f32>>,
}

impl Default for SizeStrategy {
    fn default() -> Self {
        let mut weights = std::collections::HashMap::new();
        weights.insert("lsp-analysis".to_string(), 0.3);
        weights.insert("ai-inference".to_string(), 0.7);
        weights.insert("semantic-tokens".to_string(), 0.2);
        weights.insert("diagnostics".to_string(), 0.5);
        weights.insert("completion".to_string(), 0.4);

        Self {
            max_memory_usage: 100 * 1024 * 1024, // 100MB default
            current_memory: std::sync::atomic::AtomicUsize::new(0),
            priority_weights: std::sync::RwLock::new(weights),
        }
    }
}

impl SizeStrategy {
    pub fn with_priority_weights(self, weights: std::collections::HashMap<String, f32>) -> Self {
        *self.priority_weights.write().unwrap() = weights;
        self
    }

    fn estimate_entry_size<V: serde::Serialize>(&self, entry: &CacheEntry<V>) -> usize {
        let base_size = std::mem::size_of::<CacheEntry<V>>();
        let value_size = self.estimate_value_size(entry);
        let metadata_size = entry.metadata.iter()
            .map(|(k, v)| k.len() + serde_json::from_str::<serde_json::Value>(v)
                .map_or(v.len(), |value| self.estimate_json_size(&value)))
            .sum::<usize>();

        base_size + value_size + metadata_size + entry.size_hint()
    }

    fn estimate_value_size<V: serde::Serialize>(&self, entry: &CacheEntry<V>) -> usize {
        if let Ok(serialized) = serde_json::to_string(&entry.value) {
            serialized.len()
        } else {
            std::mem::size_of::<V>() * 2 // Rough estimate if serialization fails
        }
    }

    fn estimate_json_size(&self, value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::String(s) => s.len(),
            serde_json::Value::Array(arr) => arr.iter().map(|v| self.estimate_json_size(v)).sum::<usize>(),
            serde_json::Value::Object(obj) => obj.iter().map(|(k, v)| k.len() + self.estimate_json_size(v)).sum::<usize>(),
            _ => 16, // Small fixed size for primitives
        }
    }

    fn calculate_eviction_score(&self, entry: &CacheEntry<impl serde::Serialize>, access_count: u64) -> f64 {
        let priority_weight = entry.metadata.get("priority")
            .and_then(|v| Some(v.as_str()))
            .and_then(|p| self.priority_weights.read().unwrap().get(p).copied())
            .unwrap_or(1.0);

        let size_weight = (self.estimate_entry_size(entry) as f64 / self.max_memory_usage as f64).min(1.0);
        let freshness_weight = chrono::Utc::now().timestamp() as f64 - entry.last_accessed.timestamp() as f64;
        let usage_weight = if access_count > 0 { 1.0 / (access_count as f64 + 1.0) } else { 2.0 };

        // Higher score means more likely to be evicted
        (size_weight * 0.4 + freshness_weight * 0.3 * priority_weight as f64 + usage_weight * 0.3).clamp(0.0, 1.0)
    }
}

impl<K, V> EvictionStrategy<K, V> for SizeStrategy
where
    V: serde::de::DeserializeOwned + serde::Serialize,
    K: Clone,
{
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        if entries.is_empty() {
            return Vec::new();
        }

        // Calculate current memory usage
        let current_usage: usize = entries.iter()
            .map(|(_, entry)| self.estimate_entry_size(entry))
            .sum();

        // Update current memory (atomic operation)
        self.current_memory.store(current_usage, std::sync::atomic::Ordering::Relaxed);

        // If we're under memory limit, don't evict
        if current_usage <= self.max_memory_usage {
            return Vec::new();
        }

        // Calculate eviction scores
        let mut scored_entries: Vec<_> = entries.iter().enumerate()
            .map(|(idx, (key, entry))| {
                let score = self.calculate_eviction_score(entry, entry.access_count);
                (idx, score, current_usage - self.estimate_entry_size(entry))
            })
            .collect();

        // Sort by score (highest first) and then by memory savings (lowest first)
        scored_entries.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.2.cmp(&b.2))
        });

        // Select entries to evict until we reach target memory usage or count
        let target_memory = (self.max_memory_usage as f64 * 0.8) as usize; // Target 80% of max
        let mut selected = Vec::new();
        let mut running_memory = current_usage;

        for (idx, score, reduced_memory) in scored_entries {
            if selected.len() >= target_count || running_memory <= target_memory {
                break;
            }

            running_memory = reduced_memory;
            selected.push(entries[idx].0.clone());
        }

        selected
    }
}

/// TTL-based cleanup strategy
pub struct TtlCleanupStrategy;

impl TtlCleanupStrategy {
    pub fn cleanup_expired<K, V>(entries: &mut Vec<(K, CacheEntry<V>)>) -> usize
    where
        K: Clone,
    {
        let initial_len = entries.len();
        entries.retain(|(_, entry)| !entry.is_expired());
        initial_len - entries.len()
    }
}

/// Performance-aware eviction strategy that considers multiple factors and adapts to usage patterns
pub struct AdaptiveStrategy {
    pub weight_recent: f32,
    pub weight_frequency: f32,
    pub weight_size: f32,
    pub adaptive_mode: std::sync::atomic::AtomicBool,
    pub usage_history: std::sync::RwLock<std::collections::VecDeque<(chrono::DateTime<chrono::Utc>, usize, f64)>>,
    pub prediction_enabled: bool,
}

impl Default for AdaptiveStrategy {
    fn default() -> Self {
        Self {
            weight_recent: 0.4,
            weight_frequency: 0.4,
            weight_size: 0.2,
            adaptive_mode: std::sync::atomic::AtomicBool::new(true),
            usage_history: std::sync::RwLock::new(std::collections::VecDeque::with_capacity(1000)),
            prediction_enabled: true,
        }
    }
}

impl AdaptiveStrategy {
    /// Adapt weights based on historical usage patterns
    pub async fn adapt_weights(&self, current_hit_rate: f64, current_throughput: f64) {
        if !self.adaptive_mode.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let history = self.usage_history.read().unwrap();
        if history.len() < 10 {
            return;
        }

        // Calculate trend: is performance improving?
        let recent_performance: Vec<_> = history.iter().rev().take(5).map(|(_, _, perf)| *perf).collect();
        let avg_recent = recent_performance.iter().sum::<f64>() / recent_performance.len() as f64;
        let trend = avg_recent - (history.iter().map(|(_, _, perf)| *perf).sum::<f64>() / history.len() as f64);

        // Adjust weights based on trend
        if trend > 0.0 {
            // Performance is improving, slight adjustment towards current weights
            let factor = 0.02 * trend.signum(); // Small adjustment
            self.adjust_weight_for_improvement(factor).await;
        }

        // Log current metrics
        {
            let mut history = self.usage_history.write().unwrap();
            if history.len() >= 1000 {
                history.pop_front();
            }
            history.push_back((chrono::Utc::now(), current_hit_rate as usize, current_throughput));
        }
    }

    async fn adjust_weight_for_improvement(&self, _factor: f64) {
        // Weight adjustment logic could be implemented here if needed
        // For now, we maintain fixed weights
    }
}

impl<K, V> EvictionStrategy<K, V> for AdaptiveStrategy
where
    K: Clone,
    V: serde::Serialize,
{
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        if entries.is_empty() {
            return Vec::new();
        }

        // Enhanced scoring with multiple factors
        let now = chrono::Utc::now();
        let max_recent = entries
            .iter()
            .map(|(_, e)| e.last_accessed.timestamp())
            .max()
            .unwrap();

        let max_frequency = entries.iter().map(|(_, e)| e.access_count).max().unwrap();

        let scores: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(idx, (key, entry))| {
                // Base scores
                let recent_score = (max_recent - entry.last_accessed.timestamp()) as f32 / 3600.0; // hours ago
                let frequency_score = max_frequency.saturating_sub(entry.access_count) as f32;
                let size_score = entry.size_hint() as f32 / 1024.0; // KB

                // Predictive factors
                let time_since_access = (now.timestamp() - entry.last_accessed.timestamp()) as f32;
                let access_pattern_factor = if entry.access_count > 0 {
                    time_since_access / entry.access_count as f32
                } else {
                    f32::INFINITY
                };

                // Cost factor (combines size and access pattern)
                let cost_factor = size_score * (1.0 + access_pattern_factor.log10().max(0.0));

                // Priority-based weighting
                let priority_weight = entry.metadata.get("priority")
                    .and_then(|v| Some(v.as_str()))
                    .map(|p| match p {
                        "high" => 0.3,
                        "medium" => 0.6,
                        "low" => 0.9,
                        _ => 0.7,
                    })
                    .unwrap_or(0.7);

                // Adaptive scoring
                let base_score = if self.adaptive_mode.load(std::sync::atomic::Ordering::Relaxed) {
                    // Use adaptive weights based on historical performance
                    adaptive_scoring(recent_score, frequency_score, size_score, entry, self)
                } else {
                    // Use configured weights
                    recent_score * self.weight_recent +
                    frequency_score * self.weight_frequency +
                    size_score * self.weight_size
                };

                // Prediction boost: entries that might be accessed soon get lower scores
                let prediction_boost = if self.prediction_enabled {
                    predict_future_access(entry, now)
                } else {
                    0.0
                };

                let final_score = base_score * priority_weight + cost_factor - prediction_boost;

                (idx, final_score)
            })
            .collect();

        // Sort by score (lower scores are evicted first)
        let mut scores: Vec<_> = scores.into_iter().enumerate().collect();
        scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply intelligent selection to avoid evicting recently accessed items
        let result: Vec<_> = scores
            .into_iter()
            .take(target_count)
            .filter(|(idx, score)| {
                let entry = entries[*idx].1;
                // Don't evict entries accessed within last minute if they have high access count
                let minutes_since_access = ((now.timestamp() - entry.last_accessed.timestamp()) as f64) / 60.0;
                !(minutes_since_access < 1.0 && entry.access_count > 5)
            })
            .map(|(idx, _)| entries[idx].0.clone())
            .collect();

        result
    }
}

fn adaptive_scoring<V>(
    recent_score: f32,
    frequency_score: f32,
    size_score: f32,
    entry: &CacheEntry<V>,
    strategy: &AdaptiveStrategy,
) -> f32 {
    // Use historical data to adjust weights dynamically
    let history = strategy.usage_history.try_read().unwrap();
    if history.is_empty() {
        return recent_score * 0.4 + frequency_score * 0.4 + size_score * 0.2;
    }

    // Calculate average hit rate
    let avg_hit_rate: f64 = history.iter().map(|(_, hit_rate, _)| *hit_rate as f64).sum::<f64>() / history.len() as f64;

    // Adjust weights based on hit rate performance
    let (recent_weight, freq_weight, size_weight) = if avg_hit_rate > 0.8 {
        // High hit rate - favor frequency
        (0.3, 0.5, 0.2)
    } else if avg_hit_rate > 0.6 {
        // Medium hit rate - balanced
        (0.4, 0.4, 0.2)
    } else {
        // Low hit rate - favor recency
        (0.6, 0.2, 0.2)
    };

    recent_score * recent_weight + frequency_score * freq_weight + size_score * size_weight
}

fn predict_future_access<V>(entry: &CacheEntry<V>, now: chrono::DateTime<chrono::Utc>) -> f32 {
    // Simple prediction based on access patterns
    let time_since_last_access = (now.timestamp() - entry.last_accessed.timestamp()) as f32;

    if entry.access_count == 0 {
        return 0.0;
    }

    let avg_access_interval = time_since_last_access / entry.access_count as f32;

    // Predict if this entry will be accessed soon based on historical pattern
    if avg_access_interval < 10.0 && entry.access_count > 3 {
        // Frequently accessed, predict higher chance of future access
        (10.0 - avg_access_interval).max(0.0) * 0.5
    } else {
        0.0
    }
}

/// Strategy selector for choosing the right strategy based on usage patterns
pub struct StrategySelector;

impl StrategySelector {
    pub fn select_strategy<K, V>(
        entries: &[(&K, &CacheEntry<V>)],
        config: &CacheConfig,
    ) -> Box<dyn EvictionStrategy<K, V> + Send + Sync>
    where
        K: Clone + Send + Sync + 'static + std::hash::Hash,
        V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        match config.eviction_policy {
            EvictionPolicy::Lru => Box::new(LruStrategy),
            EvictionPolicy::Lfu => Box::new(LfuStrategy),
            EvictionPolicy::Fifo => Box::new(FifoStrategy),
            EvictionPolicy::Random => Box::new(RandomStrategy::new()),
            EvictionPolicy::SizeBased => Box::new(SizeStrategy {
                max_memory_usage: config.max_memory_mb.unwrap_or(100) * 1024 * 1024,
                current_memory: std::sync::atomic::AtomicUsize::new(0),
                priority_weights: std::sync::RwLock::new(std::collections::HashMap::new()),
            }),
            EvictionPolicy::Adaptive => Box::new(AdaptiveStrategy::default()),
            EvictionPolicy::WTinyLFU => Box::new(WTinyLFU::default()),
            EvictionPolicy::SegmentedLRU => Box::new(SegmentedLRUStrategy::default()),
            EvictionPolicy::Clock => Box::new(ClockStrategy::default()),
        }
    }
}

/// W-TinyLFU (Windowed TinyLFU) eviction strategy
/// Advanced cache replacement policy that combines recency and frequency
pub struct WTinyLFU {
    window_size: usize,
    sketch_width: usize,
    sketch_depth: usize,
    window_accesses: std::sync::RwLock<std::collections::VecDeque<(u64, u64)>>, // (timestamp, hash)
    main_accesses: std::sync::RwLock<std::collections::HashMap<u64, u8>>, // Hash -> frequency
    current_window_size: std::sync::atomic::AtomicUsize,
}

impl WTinyLFU {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            sketch_width: 1024,
            sketch_depth: 4,
            window_accesses: std::sync::RwLock::new(std::collections::VecDeque::with_capacity(window_size)),
            main_accesses: std::sync::RwLock::new(std::collections::HashMap::new()),
            current_window_size: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn hash_key<K>(&self, key: &K) -> u64 where K: std::hash::Hash {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn estimate_frequency(&self, hash: u64) -> u8 {
        self.main_accesses.read().unwrap().get(&hash).copied().unwrap_or(0)
    }

    fn record_access(&self, hash: u64, now: u64) {
        // Windowed TinyLFU algorithm
        {
            let mut window = self.window_accesses.write().unwrap();
            window.push_back((now, hash));
            if window.len() > self.window_size {
                if let Some((_, old_hash)) = window.pop_front() {
                    // Move frequent items to main structure
                    if self.estimate_frequency(old_hash) > 1 {
                        self.main_accesses.write().unwrap()
                            .entry(old_hash)
                            .and_modify(|freq| {
                                if *freq < 255 {
                                    *freq += 1;
                                }
                            })
                            .or_insert(1);
                    }
                }
            }
        }

        // Update main frequency
        self.main_accesses.write().unwrap()
            .entry(hash)
            .and_modify(|freq| {
                if *freq < 255 {
                    *freq += 1;
                }
            })
            .or_insert(1);

        self.current_window_size.store(
            self.window_accesses.read().unwrap().len(),
            std::sync::atomic::Ordering::Relaxed
        );
    }
}

impl Default for WTinyLFU {
    fn default() -> Self {
        Self::new(10000) // Default window size
    }
}

impl<K, V> EvictionStrategy<K, V> for WTinyLFU
where
    K: Clone + std::hash::Hash,
    V: serde::Serialize,
{
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        if entries.is_empty() {
            return Vec::new();
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Score entries using W-TinyLFU algorithm
        let mut scored_entries: Vec<_> = entries.iter().enumerate()
            .map(|(idx, (key, entry))| {
                let hash = self.hash_key(key);
                let frequency = self.estimate_frequency(hash);

                // Record access for future decisions
                self.record_access(hash, now);

                // Calculate score based on frequency and recency
                let recency_factor = 1.0 / ((now as f64) - entry.last_accessed.timestamp() as f64).abs().max(1.0);
                let frequency_factor = frequency.min(100) as f32 / 100.0; // Cap frequency

                // W-TinyLFU score (higher = more likely to keep)
                let wtinylfu_score = frequency_factor * 0.7 + recency_factor as f32 * 0.3;

                (idx, wtinylfu_score)
            })
            .collect();

        // Sort by score (lower scores evicted first)
        scored_entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_entries.into_iter()
            .take(target_count)
            .map(|(idx, _)| entries[idx].0.clone())
            .collect()
    }
}

/// Segmented LRU strategy for better cache performance
/// Uses two segments: probationary and protected
pub struct SegmentedLRUStrategy {
    probationary_capacity: usize,
    protected_capacity: usize,
    probationary: std::sync::RwLock<std::collections::VecDeque<u64>>, // Hash of keys
    protected: std::sync::RwLock<std::collections::VecDeque<u64>>,
}

impl SegmentedLRUStrategy {
    fn hash_key<K: std::hash::Hash>(&self, key: &K) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for SegmentedLRUStrategy {
    fn default() -> Self {
        Self {
            probationary_capacity: 1000,
            protected_capacity: 500,
            probationary: std::sync::RwLock::new(std::collections::VecDeque::new()),
            protected: std::sync::RwLock::new(std::collections::VecDeque::new()),
        }
    }
}

impl<K, V> EvictionStrategy<K, V> for SegmentedLRUStrategy
where
    K: Clone + std::hash::Hash,
    V: serde::Serialize,
{
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        // Simple implementation: just use LRU on probationary segment
        // In practice, this would track hits and move entries between segments
        LruStrategy.select_entries_for_eviction(entries, target_count)
    }
}

/// Clock algorithm (second chance algorithm) for efficient LRU approximation
pub struct ClockStrategy {
    clock_hand: std::sync::atomic::AtomicUsize,
    reference_bits: std::sync::RwLock<std::collections::HashMap<u64, bool>>, // Hash -> reference bit
}

impl Default for ClockStrategy {
    fn default() -> Self {
        Self {
            clock_hand: std::sync::atomic::AtomicUsize::new(0),
            reference_bits: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl<K, V> EvictionStrategy<K, V> for ClockStrategy
where
    K: Clone + std::hash::Hash,
    V: serde::Serialize,
{
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        if entries.is_empty() {
            return Vec::new();
        }

        let mut hand = self.clock_hand.load(std::sync::atomic::Ordering::Relaxed);
        let mut selected = Vec::new();

        while selected.len() < target_count && hand < entries.len() {
            let entry = entries[hand].1;
            // Use timestamp as simple proxy for clock hand
            let key_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                entries[hand].0.hash(&mut hasher);
                hasher.finish()
            };

            let ref_bit = *self.reference_bits.read().unwrap().get(&key_hash).unwrap_or(&false);

            if !ref_bit {
                selected.push(entries[hand].0.clone());
            } else {
                // Give second chance - clear reference bit
                self.reference_bits.write().unwrap().insert(key_hash, false);
            }

            hand += 1;
            if hand >= entries.len() {
                hand = 0;
            }
        }

        self.clock_hand.store(hand, std::sync::atomic::Ordering::Relaxed);
        selected
    }
}

/// Random eviction strategy
pub struct RandomStrategy {
    seed: u64,
}

impl RandomStrategy {
    pub fn new() -> Self {
        Self {
            seed: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        }
    }
}

impl<K, V> EvictionStrategy<K, V> for RandomStrategy {
    fn select_entries_for_eviction(
        &self,
        entries: &[(&K, &CacheEntry<V>)],
        target_count: usize,
    ) -> Vec<K>
    where
        K: Clone,
    {
        use std::cmp;

        if entries.is_empty() {
            return Vec::new();
        }

        let count = cmp::min(target_count, entries.len());
        let mut indices = (0..entries.len()).collect::<Vec<_>>();

        // Simple Fisher-Yates shuffle
        for i in (1..indices.len()).rev() {
            let j =
                (self.seed.wrapping_mul(1103515245).wrapping_add(12345) % (i + 1) as u64) as usize;
            indices.swap(i, j);
        }

        indices
            .into_iter()
            .take(count)
            .map(|idx| entries[idx].0.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_entry(
        created_at: Timestamp,
        last_accessed: Timestamp,
        access_count: u64,
    ) -> CacheEntry<String> {
        CacheEntry {
            value: "test".to_string(),
            created_at,
            last_accessed,
            expires_at: None,
            access_count,
            ttl_seconds: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_lru_strategy() {
        let strategy = LruStrategy;

        let now = Utc::now();
        let entries = vec![
            (
                "old_access",
                create_test_entry(
                    now - chrono::Duration::hours(2),
                    now - chrono::Duration::hours(2),
                    1,
                ),
            ),
            (
                "new_access",
                create_test_entry(
                    now - chrono::Duration::hours(1),
                    now - chrono::Duration::hours(1),
                    1,
                ),
            ),
            ("very_new", create_test_entry(now, now, 5)),
        ];

        let entries_refs: Vec<(&String, &CacheEntry<String>)> =
            entries.iter().map(|(k, v)| (k, v)).collect();

        let evicted = strategy.select_entries_for_eviction(&entries_refs, 2);

        // Should evict oldest accessed first
        assert_eq!(evicted, vec!["old_access", "new_access"]);
    }

    #[test]
    fn test_lfu_strategy() {
        let strategy = LfuStrategy;

        let now = Utc::now();
        let entries = vec![
            ("high_freq", create_test_entry(now, now, 10)),
            ("low_freq1", create_test_entry(now, now, 1)),
            ("low_freq2", create_test_entry(now, now, 2)),
        ];

        let entries_refs: Vec<(&String, &CacheEntry<String>)> =
            entries.iter().map(|(k, v)| (k, v)).collect();

        let evicted = strategy.select_entries_for_eviction(&entries_refs, 1);

        // Should evict least frequently used
        assert_eq!(evicted, vec!["low_freq1"]);
    }

    #[test]
    fn test_fifo_strategy() {
        let strategy = FifoStrategy;

        let now = Utc::now();
        let entries = vec![
            (
                "first",
                create_test_entry(now - chrono::Duration::hours(3), now, 1),
            ),
            (
                "second",
                create_test_entry(now - chrono::Duration::hours(2), now, 1),
            ),
            (
                "third",
                create_test_entry(now - chrono::Duration::hours(1), now, 1),
            ),
        ];

        let entries_refs: Vec<(&String, &CacheEntry<String>)> =
            entries.iter().map(|(k, v)| (k, v)).collect();

        let evicted = strategy.select_entries_for_eviction(&entries_refs, 2);

        // Should evict oldest first
        assert_eq!(evicted, vec!["first", "second"]);
    }

    #[test]
    fn test_ttl_cleanup() {
        let mut entries = vec![
            (
                "expired",
                CacheEntry::new_with_ttl(
                    "expired".to_string(),
                    Some(Duration::from_millis(10)),
                    Utc::now(),
                ),
            ),
            (
                "valid",
                CacheEntry::new_with_ttl(
                    "valid".to_string(),
                    Some(Duration::from_secs(300)),
                    Utc::now(),
                ),
            ),
        ];

        // Wait for first entry to expire
        std::thread::sleep(Duration::from_millis(20));

        let cleaned = TtlCleanupStrategy::cleanup_expired(&mut entries);
        assert_eq!(cleaned, 1);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "valid");
    }

    #[test]
    fn test_random_strategy() {
        let strategy = RandomStrategy::new();

        let now = Utc::now();
        let entries = vec![
            ("key1", create_test_entry(now, now, 1)),
            ("key2", create_test_entry(now, now, 1)),
            ("key3", create_test_entry(now, now, 1)),
            ("key4", create_test_entry(now, now, 1)),
            ("key5", create_test_entry(now, now, 1)),
        ];

        let entries_refs: Vec<(&String, &CacheEntry<String>)> =
            entries.iter().map(|(k, v)| (k, v)).collect();

        let evicted = strategy.select_entries_for_eviction(&entries_refs, 2);

        // Should evict exactly 2 entries
        assert_eq!(evicted.len(), 2);

        // All evicted entries should exist in original set
        for key in &evicted {
            assert!(entries.iter().any(|(k, _)| k == key));
        }
    }

    #[test]
    fn test_adaptive_strategy() {
        let strategy = AdaptiveStrategy::default();

        let now = Utc::now();
        let entries = vec![
            ("recent_high_freq", create_test_entry(now, now, 10)),
            (
                "old_high_freq",
                create_test_entry(now - chrono::Duration::hours(24), now, 10),
            ),
            ("recent_low_freq", create_test_entry(now, now, 1)),
            (
                "old_low_freq",
                create_test_entry(now - chrono::Duration::hours(24), now, 1),
            ),
        ];

        let entries_refs: Vec<(&String, &CacheEntry<String>)> =
            entries.iter().map(|(k, v)| (k, v)).collect();

        let evicted = strategy.select_entries_for_eviction(&entries_refs, 1);

        // Should evict oldest low frequency entry
        assert_eq!(evicted, vec!["old_low_freq"]);
    }
}
