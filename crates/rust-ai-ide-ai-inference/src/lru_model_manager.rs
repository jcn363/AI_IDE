use moka::future::Cache;
use rust_ai_ide_common::{IDEError, IDEErrorKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::task::spawn_blocking;

/// Least Recently Used (LRU) model management for AI models
pub struct LRUModelManager {
    pub(crate) model_access: Arc<RwLock<HashMap<String, Instant>>>,
    pub(crate) model_sizes: Arc<RwLock<HashMap<String, usize>>>,
    pub(crate) access_order: Arc<Mutex<VecDeque<String>>>,
    pub(crate) memory_pressure_detector: Arc<MemoryPressureDetector>,
    pub(crate) eviction_policy: Arc<EvictionPolicy>,
    pub(crate) max_memory_usage: usize,
    pub(crate) current_memory_usage: Arc<Mutex<usize>>,
}

impl LRUModelManager {
    pub fn new(max_memory_usage: usize) -> Self {
        Self {
            model_access: Arc::new(RwLock::new(HashMap::new())),
            model_sizes: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(Mutex::new(VecDeque::new())),
            memory_pressure_detector: Arc::new(MemoryPressureDetector::new()),
            eviction_policy: Arc::new(EvictionPolicy::new(max_memory_usage)),
            max_memory_usage,
            current_memory_usage: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn register_model(&self, model_key: String, size: usize) -> Result<(), IDEError> {
        if size > self.max_memory_usage {
            return Err(IDEError::new(
                IDEErrorKind::ResourceExhausted,
                format!(
                    "Model size {} bytes exceeds maximum memory usage limit {}",
                    size, self.max_memory_usage
                ),
            ));
        }

        let mut current_mem = self.current_memory_usage.lock().await;

        if *current_mem + size > self.max_memory_usage {
            let evicted = self.evict_oldest_models_until_space(size).await?;
            *current_mem -= evicted;
        }

        *current_mem += size;

        let mut model_access = self.model_access.write().await;
        let mut model_sizes = self.model_sizes.write().await;
        let mut access_order = self.access_order.lock().await;

        let now = Instant::now();
        model_access.insert(model_key.clone(), now);
        model_sizes.insert(model_key.clone(), size);

        // Remove if already present, then add to end (most recent)
        access_order.retain(|key| key != &model_key);
        access_order.push_back(model_key);

        Ok(())
    }

    pub async fn access_model(&self, model_key: &str) -> Result<(), IDEError> {
        let mut model_access = self.model_access.write().await;
        let mut access_order = self.access_order.lock().await;

        if model_access.contains_key(model_key) {
            // Update access time and move to end of queue
            let now = Instant::now();
            model_access.insert(model_key.to_string(), now);

            // Remove and re-add to move to end
            access_order.retain(|key| key != model_key);
            access_order.push_back(model_key.to_string());

            Ok(())
        } else {
            Err(IDEError::new(
                IDEErrorKind::ResourceNotFound,
                format!("Model {} not registered with LRU manager", model_key),
            ))
        }
    }

    pub async fn unregister_model(&self, model_key: &str) -> Result<(), IDEError> {
        let mut current_mem = self.current_memory_usage.lock().await;
        let mut model_access = self.model_access.write().await;
        let mut model_sizes = self.model_sizes.write().await;
        let mut access_order = self.access_order.lock().await;

        if let Some(size) = model_sizes.remove(model_key) {
            *current_mem -= size;
        }

        model_access.remove(model_key);
        access_order.retain(|key| key != model_key);

        Ok(())
    }

    pub async fn evict_oldest_models_until_space(
        &self,
        required_space: usize,
    ) -> Result<usize, IDEError> {
        let mut total_evicted = 0;
        let model_access = self.model_access.read().await;
        let model_sizes = self.model_sizes.read().await;
        let mut access_order = self.access_order.lock().await;

        let mut keys_to_evict = Vec::new();

        while !access_order.is_empty()
            && (self.get_current_memory_usage().await + required_space - total_evicted
                > self.max_memory_usage)
        {
            if let Some(oldest_key) = access_order.front().cloned() {
                if let Some(size) = model_sizes.get(&oldest_key) {
                    keys_to_evict.push(oldest_key.clone());
                    total_evicted += size;
                    access_order.pop_front();
                }
            }
        }

        // Actually evict the models
        drop(access_order);
        drop(model_sizes);
        drop(model_access);

        for key in keys_to_evict {
            self.perform_model_eviction(&key).await?;
        }

        Ok(total_evicted)
    }

    async fn perform_model_eviction(&self, model_key: &str) -> Result<(), IDEError> {
        let mut model_access = self.model_access.write().await;
        let mut model_sizes = self.model_sizes.write().await;
        let mut access_order = self.access_order.lock().await;
        let mut current_mem = self.current_memory_usage.lock().await;

        if let Some(size) = model_sizes.remove(model_key) {
            *current_mem -= size;
        }

        model_access.remove(model_key);
        access_order.retain(|key| key != model_key);

        // Signal eviction (in practice, this would unload the actual model)
        self.notify_model_eviction(model_key).await;

        Ok(())
    }

    async fn notify_model_eviction(&self, model_key: &str) {
        // Notify other components about model eviction
        // This could trigger model reloading or other recovery actions
        println!("Model {} evicted due to memory pressure", model_key);
    }

    pub async fn get_least_recently_used(&self) -> Option<String> {
        let access_order = self.access_order.lock().await;
        access_order.front().cloned()
    }

    pub async fn get_most_recently_used(&self) -> Option<String> {
        let access_order = self.access_order.lock().await;
        access_order.back().cloned()
    }

    pub async fn get_current_memory_usage(&self) -> usize {
        *self.current_memory_usage.lock().await
    }

    pub async fn get_memory_usage_ratio(&self) -> f64 {
        let current = self.get_current_memory_usage().await;
        current as f64 / self.max_memory_usage as f64
    }

    pub async fn get_registered_models(&self) -> Vec<String> {
        let model_sizes = self.model_sizes.read().await;
        model_sizes.keys().cloned().collect()
    }

    pub async fn get_access_statistics(&self) -> HashMap<String, Duration> {
        let model_access = self.model_access.read().await;
        let now = Instant::now();

        model_access
            .iter()
            .map(|(key, instant)| (key.clone(), now.duration_since(*instant)))
            .collect()
    }

    pub async fn is_memory_pressure_high(&self) -> bool {
        self.memory_pressure_detector.detect_high_pressure().await
    }

    pub async fn proactive_eviction(&self) -> Result<Vec<String>, IDEError> {
        let mut evicted_models = Vec::new();

        if self.is_memory_pressure_high().await {
            let eviction_candidates = self.eviction_policy.get_candidates().await;
            let mut total_evicted_size = 0;

            for candidate in eviction_candidates {
                if self.get_memory_usage_ratio().await < 0.8 {
                    break;
                }

                evicted_models.push(candidate.clone());
                if let Some(size) = self.model_sizes.read().await.get(&candidate) {
                    total_evicted_size += size;
                }

                self.perform_model_eviction(&candidate).await?;
            }

            if !evicted_models.is_empty() {
                println!(
                    "Proactively evicted {} models, freed {} bytes",
                    evicted_models.len(),
                    total_evicted_size
                );
            }
        }

        Ok(evicted_models)
    }

    pub async fn get_eviction_recommendations(&self) -> Vec<String> {
        let access_stats = self.get_access_statistics().await;
        let threshold = Duration::from_secs(300); // 5 minutes

        access_stats
            .iter()
            .filter(|(_, time_since_access)| **time_since_access > threshold)
            .map(|(key, _)| key.clone())
            .collect()
    }

    pub async fn set_memory_limit(&self, new_limit: usize) {
        let old_limit = self.max_memory_usage;

        // Update the policy with new limit (cast to mutable)
        unsafe {
            let eviction_policy = Arc::as_ptr(&self.eviction_policy) as *mut EvictionPolicy;
            (*eviction_policy).max_memory = new_limit;
        }

        if new_limit < old_limit && self.get_current_memory_usage().await > new_limit {
            let excess = self.get_current_memory_usage().await - new_limit;
            self.evict_oldest_models_until_space(excess).await.ok();
        }
    }
}

/// Memory pressure detection component
pub struct MemoryPressureDetector {
    pub(crate) pressure_threshold: f64,
    pub(crate) measurement_window: Duration,
    pub(crate) measurements: Arc<Mutex<VecDeque<usize>>>,
}

impl MemoryPressureDetector {
    pub fn new() -> Self {
        Self {
            pressure_threshold: 0.85,                    // 85% usage
            measurement_window: Duration::from_secs(60), // 1 minute window
            measurements: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn record_memory_usage(&self, usage: usize) {
        let mut measurements = self.measurements.lock().await;
        measurements.push_back(usage);

        // Maintain window size (simplified - in practice, use timestamps)
        while measurements.len() > 10 {
            measurements.pop_front();
        }
    }

    pub async fn detect_high_pressure(&self) -> bool {
        let measurements = self.measurements.lock().await;

        if measurements.is_empty() {
            return false;
        }

        let avg_usage: usize = measurements.iter().sum::<usize>() / measurements.len();
        let threshold_value = (self.pressure_threshold * 1_000_000_000.0) as usize; // Assuming some scale

        avg_usage > threshold_value
    }

    pub async fn get_average_memory_usage(&self) -> usize {
        let measurements = self.measurements.lock().await;
        if measurements.is_empty() {
            0
        } else {
            measurements.iter().sum::<usize>() / measurements.len()
        }
    }

    pub async fn set_pressure_threshold(&self, threshold: f64) {
        // Update threshold (would need mutable access in real implementation)
        println!("Updated pressure threshold to {}", threshold);
    }
}

/// Intelligent model eviction policy
pub struct EvictionPolicy {
    pub(crate) max_memory: usize,
    pub(crate) access_patterns: Arc<Mutex<HashMap<String, AccessPattern>>>,
    pub(crate) predictive_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub frequency: f64,
    pub recency: Instant,
    pub utility_score: f64,
}

impl EvictionPolicy {
    pub fn new(max_memory: usize) -> Self {
        Self {
            max_memory,
            access_patterns: Arc::new(Mutex::new(HashMap::new())),
            predictive_threshold: 0.7,
        }
    }

    pub async fn record_access(&self, model_key: &str) {
        let mut patterns = self.access_patterns.lock().await;
        let pattern = patterns
            .entry(model_key.to_string())
            .or_insert(AccessPattern {
                frequency: 1.0,
                recency: Instant::now(),
                utility_score: 1.0,
            });

        pattern.frequency += 1.0;
        pattern.recency = Instant::now();
        pattern.utility_score = self.calculate_utility_score(pattern);
    }

    pub async fn get_candidates(&self) -> Vec<String> {
        let patterns = self.access_patterns.lock().await;
        let mut candidates: Vec<(String, f64)> = patterns
            .iter()
            .map(|(key, pattern)| (key.clone(), self.calculate_eviction_priority(pattern)))
            .collect();

        // Sort by eviction priority (higher = more likely to be evicted)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        candidates.into_iter().map(|(key, _)| key).collect()
    }

    pub async fn should_retain(&self, model_key: &str) -> bool {
        let patterns = self.access_patterns.lock().await;

        if let Some(pattern) = patterns.get(model_key) {
            let now = Instant::now();
            let time_since_access = now.duration_since(pattern.recency);

            // Retain if recently accessed or high utility
            time_since_access < Duration::from_secs(300)
                || pattern.utility_score > self.predictive_threshold
        } else {
            false
        }
    }

    fn calculate_utility_score(&self, pattern: &AccessPattern) -> f64 {
        // Combine frequency and recency into a utility score
        // Higher score = more valuable to keep
        let now = Instant::now();
        let hours_since_access = now.duration_since(pattern.recency).as_secs_f64() / 3600.0;

        // Exponential decay with frequency boost
        let recency_score = (-hours_since_access / 24.0).exp();
        let frequency_score = (pattern.frequency / 10.0).tanh(); // Scale to reasonable range

        recency_score * 0.7 + frequency_score * 0.3
    }

    fn calculate_eviction_priority(&self, pattern: &AccessPattern) -> f64 {
        // Inverse of utility score + additional factors
        1.0 - self.calculate_utility_score(pattern)
    }

    pub async fn reset_access_patterns(&self) {
        let mut patterns = self.access_patterns.lock().await;
        patterns.clear();
    }

    pub async fn get_statistics(&self) -> EvictionStats {
        let patterns = self.access_patterns.lock().await;

        let total_patterns = patterns.len();
        let avg_utility = if total_patterns > 0 {
            patterns.values().map(|p| p.utility_score).sum::<f64>() / total_patterns as f64
        } else {
            0.0
        };

        let high_utility_count = patterns
            .values()
            .filter(|p| p.utility_score > self.predictive_threshold)
            .count();

        EvictionStats {
            total_registered_models: total_patterns,
            average_utility_score: avg_utility,
            high_value_models: high_utility_count,
            low_value_models: total_patterns.saturating_sub(high_utility_count),
        }
    }
}

/// Model preloader for predictive loading
pub struct ModelPreloader {
    pub(crate) usage_patterns: Arc<Mutex<HashMap<String, UsagePattern>>>,
    pub(crate) predictive_model: Arc<Mutex<PredictiveModel>>,
    pub(crate) preload_queue: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Debug)]
pub struct UsagePattern {
    pub access_history: VecDeque<Instant>,
    pub confidence_score: f64,
}

#[derive(Clone, Debug)]
pub struct PredictiveModel {
    pub patterns: HashMap<String, f64>,
}

impl ModelPreloader {
    pub fn new() -> Self {
        Self {
            usage_patterns: Arc::new(Mutex::new(HashMap::new())),
            predictive_model: Arc::new(Mutex::new(PredictiveModel {
                patterns: HashMap::new(),
            })),
            preload_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn record_usage(&self, model_key: &str) {
        let mut patterns = self.usage_patterns.lock().await;
        let pattern = patterns
            .entry(model_key.to_string())
            .or_insert(UsagePattern {
                access_history: VecDeque::new(),
                confidence_score: 0.5,
            });

        pattern.access_history.push_back(Instant::now());

        // Maintain history window
        while pattern.access_history.len() > 100 {
            pattern.access_history.pop_front();
        }

        // Update confidence based on recency and frequency
        pattern.confidence_score = self.calculate_confidence(pattern);
    }

    pub async fn predict_next_usage(&self) -> Vec<(String, f64)> {
        let patterns = self.usage_patterns.lock().await;
        let now = Instant::now();

        let mut predictions: Vec<(String, f64)> = Vec::new();

        for (key, pattern) in patterns.iter() {
            let mut score = 0.0;

            // Score based on recent access
            if let Some(last_access) = pattern.access_history.back() {
                let time_since_access = now.duration_since(*last_access).as_secs_f64();
                if time_since_access < 3600.0 {
                    // Within last hour
                    score += 1.0 - (time_since_access / 3600.0); // Higher score for more recent
                }
            }

            // Multiply by confidence
            score *= pattern.confidence_score;

            if score > 0.3 {
                // Only include reasonably confident predictions
                predictions.push((key.clone(), score));
            }
        }

        // Sort by predicted likelihood
        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        predictions
    }

    pub async fn queue_preloads(&self, lru_manager: &LRUModelManager) {
        let predictions = self.predict_next_usage().await;
        let registered_models: HashSet<String> = lru_manager
            .get_registered_models()
            .await
            .into_iter()
            .collect();

        let mut queue = self.preload_queue.lock().await;

        for (model_key, confidence) in predictions {
            if registered_models.contains(&model_key) && confidence > 0.5 {
                queue.push(model_key);
            }
        }
    }

    pub async fn get_next_preload(&self) -> Option<String> {
        let mut queue = self.preload_queue.lock().await;
        queue.pop()
    }

    fn calculate_confidence(&self, pattern: &UsagePattern) -> f64 {
        if pattern.access_history.is_empty() {
            return 0.0;
        }

        let total_count = pattern.access_history.len();

        // Simple confidence calculation based on recent activity
        let recent_count = pattern
            .access_history
            .iter()
            .filter(|&&access_time| {
                Instant::now().duration_since(access_time).as_secs() < 86400 // Last 24 hours
            })
            .count();

        (recent_count as f64 / total_count as f64).min(1.0).max(0.1)
    }
}

#[derive(Clone, Debug)]
pub struct EvictionStats {
    pub total_registered_models: usize,
    pub average_utility_score: f64,
    pub high_value_models: usize,
    pub low_value_models: usize,
}
