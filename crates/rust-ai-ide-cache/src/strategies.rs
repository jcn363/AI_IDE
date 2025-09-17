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
        let metadata_size = entry
            .metadata
            .iter()
            .map(|(k, v)| {
                k.len()
                    + serde_json::from_str::<serde_json::Value>(v)
                        .map_or(v.len(), |value| self.estimate_json_size(&value))
            })
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
            serde_json::Value::Array(arr) => arr
                .iter()
                .map(|v| self.estimate_json_size(v))
                .sum::<usize>(),
            serde_json::Value::Object(obj) => obj
                .iter()
                .map(|(k, v)| k.len() + self.estimate_json_size(v))
                .sum::<usize>(),
            _ => 16, // Small fixed size for primitives
        }
    }

    fn calculate_eviction_score(
        &self,
        entry: &CacheEntry<impl serde::Serialize>,
        access_count: u64,
    ) -> f64 {
        let priority_weight = entry
            .metadata
            .get("priority")
            .and_then(|v| Some(v.as_str()))
            .and_then(|p| self.priority_weights.read().unwrap().get(p).copied())
            .unwrap_or(1.0);

        let size_weight =
            (self.estimate_entry_size(entry) as f64 / self.max_memory_usage as f64).min(1.0);
        let freshness_weight =
            chrono::Utc::now().timestamp() as f64 - entry.last_accessed.timestamp() as f64;
        let usage_weight = if access_count > 0 {
            1.0 / (access_count as f64 + 1.0)
        } else {
            2.0
        };

        // Higher score means more likely to be evicted
        (size_weight * 0.4 + freshness_weight * 0.3 * priority_weight as f64 + usage_weight * 0.3)
            .clamp(0.0, 1.0)
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
        let current_usage: usize = entries
            .iter()
            .map(|(_, entry)| self.estimate_entry_size(entry))
            .sum();

        // Update current memory (atomic operation)
        self.current_memory
            .store(current_usage, std::sync::atomic::Ordering::Relaxed);

        // If we're under memory limit, don't evict
        if current_usage <= self.max_memory_usage {
            return Vec::new();
        }

        // Calculate eviction scores
        let mut scored_entries: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(idx, (_, entry))| {
                let score = self.calculate_eviction_score(entry, entry.access_count);
                (idx, score, current_usage - self.estimate_entry_size(entry))
            })
            .collect();

        // Sort by score (highest first) and then by memory savings (lowest first)
        scored_entries.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.2.cmp(&b.2))
        });

        // Select entries to evict until we reach target memory usage or count
        let target_memory = (self.max_memory_usage as f64 * 0.8) as usize; // Target 80% of max
        let mut selected = Vec::new();
        let mut running_memory = current_usage;

        for (idx, _score, reduced_memory) in scored_entries {
            if selected.len() >= target_count || running_memory <= target_memory {
                break;
            }

            running_memory = reduced_memory;
            selected.push(entries[idx].0.clone());
        }

        selected
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
            .map(|(idx, (_, entry))| {
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
                let priority_weight = entry
                    .metadata
                    .get("priority")
                    .and_then(|v| Some(v.as_str()))
                    .map(|p| match p {
                        "high" => 0.3,
                        "medium" => 0.6,
                        "low" => 0.9,
                        _ => 0.7,
                    })
                    .unwrap_or(0.7);

                // Adaptive scoring
                let base_score = if self
                    .adaptive_mode
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    // Use adaptive weights based on historical performance
                    adaptive_scoring(recent_score, frequency_score, size_score, entry, self)
                } else {
                    // Use configured weights
                    recent_score * self.weight_recent
                        + frequency_score * self.weight_frequency
                        + size_score * self.weight_size
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
            .filter(|(_idx, _score)| {
                let entry = entries[*_idx].1;
                // Don't evict entries accessed within last minute if they have high access count
                let minutes_since_access =
                    ((now.timestamp() - entry.last_accessed.timestamp()) as f64) / 60.0;
                !(minutes_since_access < 1.0 && entry.access_count > 5)
            })
            .map(|(idx, _)| entries[idx].0.clone())
            .collect();

        result
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
        let mut scored_entries: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(idx, (_, entry))| {
                let hash = self.hash_key(entry);
                let frequency = self.estimate_frequency(hash);

                // Record access for future decisions
                self.record_access(hash, now);

                // Calculate score based on frequency and recency
                let recency_factor = 1.0
                    / ((now as f64) - entry.last_accessed.timestamp() as f64)
                        .abs()
                        .max(1.0);
                let frequency_factor = frequency.min(100) as f32 / 100.0; // Cap frequency

                // W-TinyLFU score (higher = more likely to keep)
                let wtinylfu_score = frequency_factor * 0.7 + recency_factor as f32 * 0.3;

                (idx, wtinylfu_score)
            })
            .collect();

        // Sort by score (lower scores evicted first)
        scored_entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_entries
            .into_iter()
            .take(target_count)
            .map(|(idx, _)| entries[idx].0.clone())
            .collect()
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
            let _entry = entries[hand].1;
            // Use timestamp as simple proxy for clock hand
            let key_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::Hasher;
                let mut hasher = DefaultHasher::new();
                entries[hand].0.hash(&mut hasher);
                hasher.finish()
            };

            let ref_bit = *self
                .reference_bits
                .read()
                .unwrap()
                .get(&key_hash)
                .unwrap_or(&false);

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

        self.clock_hand
            .store(hand, std::sync::atomic::Ordering::Relaxed);
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
    use std::time::Duration;

    use chrono::Utc;

    /// Adaptive eviction strategy with configurable weights
    #[derive(Debug, Clone)]
    pub struct AdaptiveStrategy {
        pub weight_recent:     f32,
        pub weight_frequency:  f32,
        pub weight_size:       f32,
        pub adaptive_mode:     std::sync::atomic::AtomicBool,
        pub prediction_enabled: bool,
        pub frequency_map:     std::sync::RwLock<std::collections::HashMap<u64, u32>>,
    }
    
    /// W-TinyLFU eviction strategy implementation
    #[derive(Debug, Clone)]
    pub struct WTinyLFU {
        pub window_size: usize,
        pub reset_threshold: u64,
        pub frequency_map: std::sync::RwLock<std::collections::HashMap<u64, u32>>,
    }
    
    impl Default for AdaptiveStrategy {
        fn default() -> Self {
            Self {
                weight_recent: 0.3,
                weight_frequency: 0.4,
                weight_size: 0.3,
                adaptive_mode: std::sync::atomic::AtomicBool::new(false),
                prediction_enabled: false,
                frequency_map: std::sync::RwLock::new(std::collections::HashMap::new()),
            }
        }
    }
    
    impl Default for WTinyLFU {
        fn default() -> Self {
            Self {
                window_size: 10000,
                reset_threshold: 100000,
                frequency_map: std::sync::RwLock::new(std::collections::HashMap::new()),
            }
        }
    }
    
    impl WTinyLFU {
        fn hash_key(&self, entry: &CacheEntry<impl serde::Serialize>) -> u64 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::Hasher;
            let mut hasher = DefaultHasher::new();
            entry.created_at.timestamp().hash(&mut hasher);
            hasher.finish()
        }
    
        fn estimate_frequency(&self, hash: u64) -> u32 {
            *self.frequency_map.read().unwrap().get(&hash).unwrap_or(&0)
        }
    
        fn record_access(&self, hash: u64, timestamp: u64) {
            let mut map = self.frequency_map.write().unwrap();
            let count = map.entry(hash).or_insert(0);
            *count += 1;
    
            // Reset periodically to prevent unbounded growth
            if timestamp % self.reset_threshold == 0 {
                map.clear();
            }
        }
    }
    
    fn adaptive_scoring(recent_score: f32, frequency_score: f32, size_score: f32, entry: &CacheEntry<impl serde::Serialize>, strategy: &AdaptiveStrategy) -> f32 {
        // Adaptive scoring based on historical performance
        // This is a simplified implementation
        recent_score * strategy.weight_recent + frequency_score * strategy.weight_frequency + size_score * strategy.weight_size
    }
    
    fn predict_future_access(entry: &CacheEntry<impl serde::Serialize>, now: chrono::DateTime<chrono::Utc>) -> f32 {
        // Simple prediction based on access patterns
        // This is a placeholder implementation
        let time_since_access = (now - entry.last_accessed).num_seconds() as f32;
        if time_since_access < 60.0 {
            0.8 // High chance of being accessed soon
        } else if time_since_access < 3600.0 {
            0.5 // Moderate chance
        } else {
            0.1 // Low chance
        }
    }
    
    use super::*;

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
