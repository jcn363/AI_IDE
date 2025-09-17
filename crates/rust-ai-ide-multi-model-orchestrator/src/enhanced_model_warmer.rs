//! Enhanced Model Warmer with Pre-warmed Model Pools
//!
//! This module implements advanced model warming with predictive pre-loading,
//! background warming tasks, and intelligent cache management for <50ms target latency.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use moka::future::Cache;

use crate::types::{ModelId, ModelTask, Complexity, RequestContext};
use crate::{OrchestrationError, Result};

/// Performance metrics for the model warmer
#[derive(Debug, Clone)]
pub struct ModelWarmerMetrics {
    pub total_warm_requests: u64,
    pub successful_warms: u64,
    pub failed_warms: u64,
    pub average_warm_time_ms: f64,
    pub cache_hit_rate: f64,
    pub predictive_accuracy: f64,
    pub last_updated: Instant,
}

impl ModelWarmerMetrics {
    pub fn new() -> Self {
        Self {
            total_warm_requests: 0,
            successful_warms: 0,
            failed_warms: 0,
            average_warm_time_ms: 0.0,
            cache_hit_rate: 0.0,
            predictive_accuracy: 0.0,
            last_updated: Instant::now(),
        }
    }

    pub fn record_warm_attempt(&mut self, success: bool, duration_ms: f64) {
        self.total_warm_requests += 1;
        if success {
            self.successful_warms += 1;
        } else {
            self.failed_warms += 1;
        }

        // Update average warm time using exponential moving average
        let alpha = 0.1; // Smoothing factor
        self.average_warm_time_ms = alpha * duration_ms + (1.0 - alpha) * self.average_warm_time_ms;

        // Update cache hit rate
        if self.total_warm_requests > 0 {
            self.cache_hit_rate = self.successful_warms as f64 / self.total_warm_requests as f64;
        }

        self.last_updated = Instant::now();
    }

    pub fn record_prediction(&mut self, accurate: bool) {
        if accurate {
            self.predictive_accuracy = 0.9 * self.predictive_accuracy + 0.1 * 1.0;
        } else {
            self.predictive_accuracy = 0.9 * self.predictive_accuracy + 0.1 * 0.0;
        }
    }
}

/// Enhanced Model warmer for pre-loading frequently used models with predictive warming
#[derive(Debug)]
pub struct EnhancedModelWarmer {
    /// Cache of pre-warmed models: (warm_time, is_ready, last_access)
    warm_cache: Arc<RwLock<HashMap<ModelId, (Instant, bool, Instant)>>>,
    /// Model usage patterns for predictive warming: model_id -> list of access times
    usage_patterns: Arc<RwLock<HashMap<ModelId, Vec<Instant>>>>,
    /// Background warming task for asynchronous pre-loading
    background_warmer: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Maximum number of models to keep warm simultaneously
    max_warm_models: usize,
    /// Cache for predictive warming scores with TTL eviction
    prediction_cache: Cache<ModelId, f64>,
    /// Performance metrics for warming decisions and monitoring
    metrics: Arc<ModelWarmerMetrics>,
    /// Warming queue for background processing
    warming_queue: Arc<RwLock<Vec<ModelId>>>,
}

impl EnhancedModelWarmer {
    /// Create a new enhanced model warmer with specified capacity
    pub fn new(max_warm_models: usize) -> Self {
        let prediction_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300)) // 5 minute TTL
            .build();

        Self {
            warm_cache: Arc::new(RwLock::new(HashMap::new())),
            usage_patterns: Arc::new(RwLock::new(HashMap::new())),
            background_warmer: Arc::new(RwLock::new(None)),
            max_warm_models,
            prediction_cache,
            metrics: Arc::new(ModelWarmerMetrics::new()),
            warming_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if a model is warm and ready for immediate use
    pub async fn is_model_warm(&self, model_id: &ModelId) -> bool {
        let cache = self.warm_cache.read().await;
        if let Some((_, is_ready, _)) = cache.get(model_id) {
            *is_ready
        } else {
            false
        }
    }

    /// Get the warm-up time for a model (returns 0 if already warm)
    pub async fn get_warm_time(&self, model_id: &ModelId) -> Duration {
        let cache = self.warm_cache.read().await;
        if let Some((warm_time, is_ready, _)) = cache.get(model_id) {
            if *is_ready {
                Duration::from_millis(0) // Already warm
            } else {
                warm_time.elapsed() // Time spent warming so far
            }
        } else {
            Duration::from_millis(50) // Estimated warm time for cold model
        }
    }

    /// Warm up a model synchronously (for immediate use)
    pub async fn warm_model_sync(&self, model_id: ModelId) -> Result<()> {
        let start_time = Instant::now();

        // Simulate model warming (in real implementation, this would load the model)
        tokio::time::sleep(Duration::from_millis(25)).await;

        let duration_ms = start_time.elapsed().as_millis() as f64;

        // Update cache
        let mut cache = self.warm_cache.write().await;
        cache.insert(model_id, (Instant::now(), true, Instant::now()));

        // Maintain cache size limit
        if cache.len() > self.max_warm_models {
            self.evict_lru(&mut cache).await;
        }

        // Record metrics
        let mut metrics = self.metrics.as_ref().clone();
        metrics.record_warm_attempt(true, duration_ms);
        *Arc::get_mut(&mut self.metrics).unwrap() = metrics;

        Ok(())
    }

    /// Queue a model for background warming
    pub async fn queue_background_warm(&self, model_id: ModelId) -> Result<()> {
        let mut queue = self.warming_queue.write().await;
        if !queue.contains(&model_id) {
            queue.push(model_id);

            // Start background warmer if not running
            let warmer = self.background_warmer.read().await;
            if warmer.is_none() {
                drop(warmer);
                self.start_background_warmer().await;
            }
        }
        Ok(())
    }

    /// Predict which models should be warmed based on usage patterns and context
    pub async fn predict_models_to_warm(&self, context: &RequestContext) -> Vec<ModelId> {
        let patterns = self.usage_patterns.read().await;
        let mut predictions = Vec::new();

        // Simple prediction based on recent usage patterns
        for (model_id, access_times) in patterns.iter() {
            if access_times.len() < 2 {
                continue;
            }

            // Calculate access frequency in the last hour
            let one_hour_ago = Instant::now() - Duration::from_secs(3600);
            let recent_accesses = access_times.iter()
                .filter(|&&time| time > one_hour_ago)
                .count();

            if recent_accesses > 2 {
                // Check if this model matches the current context
                if self.model_matches_context(model_id, context).await {
                    predictions.push(*model_id);
                }
            }
        }

        // Sort by prediction score (simplified)
        predictions.sort_by(|a, b| {
            let score_a = self.prediction_cache.get(a).unwrap_or(0.5);
            let score_b = self.prediction_cache.get(b).unwrap_or(0.5);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        predictions.into_iter().take(3).collect() // Return top 3 predictions
    }

    /// Record model usage for predictive warming
    pub async fn record_model_usage(&self, model_id: ModelId) {
        let mut patterns = self.usage_patterns.write().await;
        let access_times = patterns.entry(model_id).or_insert_with(Vec::new);
        access_times.push(Instant::now());

        // Keep only recent accesses (last 24 hours)
        let one_day_ago = Instant::now() - Duration::from_secs(86400);
        access_times.retain(|&time| time > one_day_ago);

        // Keep max 100 entries per model
        if access_times.len() > 100 {
            access_times.remove(0);
        }
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> ModelWarmerMetrics {
        (*self.metrics).clone()
    }

    /// Get currently warm models
    pub async fn get_warm_models(&self) -> Vec<ModelId> {
        let cache = self.warm_cache.read().await;
        cache.iter()
            .filter(|(_, (_, is_ready, _))| *is_ready)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Start the background warming task
    async fn start_background_warmer(&self) {
        let warming_queue = Arc::clone(&self.warming_queue);
        let warm_cache = Arc::clone(&self.warm_cache);
        let metrics = Arc::clone(&self.metrics);

        let handle = tokio::spawn(async move {
            loop {
                // Get next model to warm
                let model_id = {
                    let mut queue = warming_queue.write().await;
                    if queue.is_empty() {
                        break; // Exit if no more models to warm
                    }
                    queue.remove(0)
                };

                // Check if already warm
                let cache = warm_cache.read().await;
                let already_warm = cache.contains_key(&model_id) &&
                    cache.get(&model_id).map(|(_, ready, _)| *ready).unwrap_or(false);
                drop(cache);

                if !already_warm {
                    // Warm the model
                    let start_time = Instant::now();
                    tokio::time::sleep(Duration::from_millis(25)).await; // Simulate warming
                    let duration_ms = start_time.elapsed().as_millis() as f64;

                    // Update cache
                    let mut cache = warm_cache.write().await;
                    cache.insert(model_id, (Instant::now(), true, Instant::now()));

                    // Record metrics
                    let mut metrics_clone = (*metrics).clone();
                    metrics_clone.record_warm_attempt(true, duration_ms);
                    *Arc::get_mut(&mut metrics).unwrap() = metrics_clone;
                }

                // Small delay between warming operations
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        let mut warmer = self.background_warmer.write().await;
        *warmer = Some(handle);
    }

    /// Evict least recently used models from cache
    async fn evict_lru(&self, cache: &mut HashMap<ModelId, (Instant, bool, Instant)>) {
        // Find LRU model
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by(|a, b| {
            let (_, _, last_access_a) = a.1;
            let (_, _, last_access_b) = b.1;
            last_access_a.cmp(last_access_b) // Sort by last access time (oldest first)
        });

        if let Some((&key, _)) = entries.first() {
            cache.remove(&key);
        }
    }

    /// Check if a model matches the current request context (simplified)
    async fn model_matches_context(&self, _model_id: &ModelId, _context: &RequestContext) -> bool {
        // In a real implementation, this would check model capabilities against context
        // For now, return true for demonstration
        true
    }
}

/// Public interface for the enhanced model warmer
pub type ModelWarmer = EnhancedModelWarmer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RequestPriority;

    #[tokio::test]
    async fn test_model_warmer_sync_warm() {
        let warmer = EnhancedModelWarmer::new(5);

        let model_id = ModelId::new();
        let result = warmer.warm_model_sync(model_id).await;
        assert!(result.is_ok());

        let is_warm = warmer.is_model_warm(&model_id).await;
        assert!(is_warm);

        let warm_time = warmer.get_warm_time(&model_id).await;
        assert_eq!(warm_time, Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_background_warming() {
        let warmer = EnhancedModelWarmer::new(5);

        let model_id = ModelId::new();
        let result = warmer.queue_background_warm(model_id).await;
        assert!(result.is_ok());

        // Wait a bit for background warming
        tokio::time::sleep(Duration::from_millis(50)).await;

        let is_warm = warmer.is_model_warm(&model_id).await;
        assert!(is_warm);
    }

    #[tokio::test]
    async fn test_predictive_warming() {
        let warmer = EnhancedModelWarmer::new(5);

        // Record usage patterns
        let model_id = ModelId::new();
        warmer.record_model_usage(model_id).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
        warmer.record_model_usage(model_id).await;

        let context = RequestContext {
            task_type: ModelTask::Completion,
            input_length: 100,
            priority: RequestPriority::Medium,
            expected_complexity: Complexity::Medium,
            acceptable_latency: Duration::from_millis(50),
            preferred_hardware: None,
        };

        let predictions = warmer.predict_models_to_warm(&context).await;
        assert!(predictions.is_empty()); // No recent usage pattern established yet
    }
}