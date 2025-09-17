//! Context Prediction Engine for Intelligent Model Selection
//!
//! This module implements predictive analytics for model selection based on
//! request patterns, user behavior, and historical performance data.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use moka::future::Cache;

use crate::types::{ModelId, ModelTask, Complexity, RequestContext, RequestPriority};
use crate::{OrchestrationError, Result};

/// Historical request pattern for predictive analysis
#[derive(Debug, Clone)]
pub struct RequestPattern {
    pub timestamp: Instant,
    pub task_type: ModelTask,
    pub complexity: Complexity,
    pub priority: RequestPriority,
    pub input_length: usize,
    pub selected_model: ModelId,
    pub processing_time: Duration,
    pub success: bool,
}

/// Context prediction result
#[derive(Debug, Clone)]
pub struct ContextPrediction {
    pub predicted_task_type: ModelTask,
    pub predicted_complexity: Complexity,
    pub predicted_input_length: usize,
    pub confidence_score: f64,
    pub recommended_models: Vec<ModelId>,
}

/// Intelligent context predictor
#[derive(Debug)]
pub struct ContextPredictor {
    /// Historical request patterns for analysis
    request_history: Arc<RwLock<VecDeque<RequestPattern>>>,
    /// Pattern analysis cache with TTL
    pattern_cache: Cache<String, ContextPrediction>,
    /// Transition probabilities between contexts
    context_transitions: Arc<RwLock<HashMap<String, HashMap<String, f64>>>>,
    /// Maximum history size to maintain
    max_history_size: usize,
}

impl ContextPredictor {
    /// Create a new context predictor
    pub fn new(max_history_size: usize) -> Self {
        let pattern_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(600)) // 10 minute TTL
            .build();

        Self {
            request_history: Arc::new(RwLock::new(VecDeque::new())),
            pattern_cache,
            context_transitions: Arc::new(RwLock::new(HashMap::new())),
            max_history_size,
        }
    }

    /// Record a request pattern for future predictions
    pub async fn record_request_pattern(&self, pattern: RequestPattern) -> Result<()> {
        let mut history = self.request_history.write().await;

        // Add new pattern
        history.push_back(pattern);

        // Maintain size limit
        while history.len() > self.max_history_size {
            history.pop_front();
        }

        // Update transition probabilities
        self.update_transitions(&history).await;

        Ok(())
    }

    /// Predict context based on current request and history
    pub async fn predict_context(&self, partial_context: &RequestContext) -> Result<ContextPrediction> {
        // Create cache key from partial context
        let cache_key = self.create_cache_key(partial_context);

        // Check cache first
        if let Some(prediction) = self.pattern_cache.get(&cache_key).await {
            return Ok(prediction);
        }

        // Perform prediction
        let prediction = self.perform_prediction(partial_context).await?;

        // Cache the result
        self.pattern_cache.insert(cache_key, prediction.clone()).await;

        Ok(prediction)
    }

    /// Get context transition probability
    pub async fn get_transition_probability(&self, from_context: &str, to_context: &str) -> f64 {
        let transitions = self.context_transitions.read().await;
        if let Some(from_transitions) = transitions.get(from_context) {
            from_transitions.get(to_context).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Analyze recent patterns for trend detection
    pub async fn analyze_trends(&self) -> Result<ContextTrends> {
        let history = self.request_history.read().await;

        if history.is_empty() {
            return Ok(ContextTrends::default());
        }

        let mut task_distribution = HashMap::new();
        let mut complexity_distribution = HashMap::new();
        let mut priority_distribution = HashMap::new();
        let mut avg_processing_times = HashMap::new();
        let mut success_rates = HashMap::new();

        // Analyze last hour of data
        let one_hour_ago = Instant::now() - Duration::from_secs(3600);
        let recent_patterns: Vec<&RequestPattern> = history
            .iter()
            .filter(|p| p.timestamp > one_hour_ago)
            .collect();

        for pattern in recent_patterns {
            // Task distribution
            *task_distribution.entry(pattern.task_type).or_insert(0) += 1;

            // Complexity distribution
            *complexity_distribution.entry(pattern.complexity).or_insert(0) += 1;

            // Priority distribution
            *priority_distribution.entry(pattern.priority).or_insert(0) += 1;

            // Processing times per task type
            let avg_time = avg_processing_times.entry(pattern.task_type).or_insert(Vec::new());
            avg_time.push(pattern.processing_time);

            // Success rates
            let success_count = success_rates.entry(pattern.task_type).or_insert((0, 0));
            success_count.1 += 1; // total
            if pattern.success {
                success_count.0 += 1; // successful
            }
        }

        // Calculate averages and rates
        let avg_processing_times: HashMap<ModelTask, Duration> = avg_processing_times
            .into_iter()
            .map(|(task, times)| {
                let total: Duration = times.iter().sum();
                let avg = total / times.len() as u32;
                (task, avg)
            })
            .collect();

        let success_rates: HashMap<ModelTask, f64> = success_rates
            .into_iter()
            .map(|(task, (success, total))| {
                let rate = if total > 0 { success as f64 / total as f64 } else { 0.0 };
                (task, rate)
            })
            .collect();

        Ok(ContextTrends {
            task_distribution,
            complexity_distribution,
            priority_distribution,
            avg_processing_times,
            success_rates,
            analysis_timestamp: Instant::now(),
        })
    }

    /// Create cache key from partial context
    fn create_cache_key(&self, context: &RequestContext) -> String {
        format!(
            "{:?}_{:?}_{}_{}",
            context.task_type, context.expected_complexity, context.priority, context.input_length
        )
    }

    /// Perform the actual prediction logic
    async fn perform_prediction(&self, partial_context: &RequestContext) -> Result<ContextPrediction> {
        let history = self.request_history.read().await;

        if history.is_empty() {
            // Return default prediction if no history
            return Ok(ContextPrediction {
                predicted_task_type: partial_context.task_type,
                predicted_complexity: partial_context.expected_complexity,
                predicted_input_length: partial_context.input_length,
                confidence_score: 0.5,
                recommended_models: Vec::new(),
            });
        }

        // Find similar historical patterns
        let similar_patterns: Vec<&RequestPattern> = history
            .iter()
            .filter(|p| self.pattern_similarity(p, partial_context) > 0.7)
            .collect();

        if similar_patterns.is_empty() {
            // No similar patterns found
            return Ok(ContextPrediction {
                predicted_task_type: partial_context.task_type,
                predicted_complexity: partial_context.expected_complexity,
                predicted_input_length: partial_context.input_length,
                confidence_score: 0.3,
                recommended_models: Vec::new(),
            });
        }

        // Calculate predictions from similar patterns
        let mut task_weights = HashMap::new();
        let mut complexity_weights = HashMap::new();
        let mut input_lengths = Vec::new();
        let mut model_recommendations = HashMap::new();

        for pattern in similar_patterns {
            // Task type prediction
            *task_weights.entry(pattern.task_type).or_insert(0.0) += 1.0;

            // Complexity prediction
            *complexity_weights.entry(pattern.complexity).or_insert(0.0) += 1.0;

            // Input length prediction
            input_lengths.push(pattern.input_length);

            // Model recommendations
            *model_recommendations.entry(pattern.selected_model).or_insert(0) += 1;
        }

        // Determine most likely task type
        let predicted_task = task_weights
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(task, _)| task)
            .unwrap_or(partial_context.task_type);

        // Determine most likely complexity
        let predicted_complexity = complexity_weights
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(complexity, _)| complexity)
            .unwrap_or(partial_context.expected_complexity);

        // Calculate average predicted input length
        let predicted_input_length = if input_lengths.is_empty() {
            partial_context.input_length
        } else {
            (input_lengths.iter().sum::<usize>() as f64 / input_lengths.len() as f64) as usize
        };

        // Get top recommended models
        let mut model_counts: Vec<(ModelId, usize)> = model_recommendations.into_iter().collect();
        model_counts.sort_by(|a, b| b.1.cmp(&a.1));
        let recommended_models = model_counts
            .into_iter()
            .take(3)
            .map(|(model, _)| model)
            .collect();

        // Calculate confidence based on pattern similarity and sample size
        let confidence_score = (similar_patterns.len() as f64 / 10.0).min(1.0) * 0.8 + 0.2;

        Ok(ContextPrediction {
            predicted_task_type: predicted_task,
            predicted_complexity: predicted_complexity,
            predicted_input_length,
            confidence_score,
            recommended_models,
        })
    }

    /// Calculate similarity between a pattern and partial context
    fn pattern_similarity(&self, pattern: &RequestPattern, context: &RequestContext) -> f64 {
        let mut similarity = 0.0;
        let mut total_weight = 0.0;

        // Task type similarity (high weight)
        if pattern.task_type == context.task_type {
            similarity += 1.0;
        }
        total_weight += 1.0;

        // Complexity similarity (medium weight)
        if pattern.complexity == context.expected_complexity {
            similarity += 0.8;
        }
        total_weight += 0.8;

        // Priority similarity (medium weight)
        if pattern.priority == context.priority {
            similarity += 0.7;
        }
        total_weight += 0.7;

        // Input length similarity (low weight)
        let length_diff = (pattern.input_length as f64 - context.input_length as f64).abs();
        let length_similarity = (1.0 - (length_diff / 1000.0).min(1.0)) * 0.5;
        similarity += length_similarity;
        total_weight += 0.5;

        similarity / total_weight
    }

    /// Update context transition probabilities
    async fn update_transitions(&self, history: &VecDeque<RequestPattern>) {
        let mut transitions = self.context_transitions.write().await;

        // Clear existing transitions
        transitions.clear();

        // Calculate transitions from recent history
        let recent_patterns: Vec<&RequestPattern> = history
            .iter()
            .rev()
            .take(100) // Last 100 patterns
            .collect();

        for window in recent_patterns.windows(2) {
            let from = &window[1];
            let to = &window[0];

            let from_key = format!("{:?}_{:?}", from.task_type, from.complexity);
            let to_key = format!("{:?}_{:?}", to.task_type, to.complexity);

            let from_transitions = transitions.entry(from_key).or_insert_with(HashMap::new);
            *from_transitions.entry(to_key).or_insert(0.0) += 1.0;
        }

        // Normalize probabilities
        for from_transitions in transitions.values_mut() {
            let total: f64 = from_transitions.values().sum();
            if total > 0.0 {
                for prob in from_transitions.values_mut() {
                    *prob /= total;
                }
            }
        }
    }
}

/// Context trend analysis result
#[derive(Debug, Clone, Default)]
pub struct ContextTrends {
    pub task_distribution: HashMap<ModelTask, usize>,
    pub complexity_distribution: HashMap<Complexity, usize>,
    pub priority_distribution: HashMap<RequestPriority, usize>,
    pub avg_processing_times: HashMap<ModelTask, Duration>,
    pub success_rates: HashMap<ModelTask, f64>,
    pub analysis_timestamp: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_context_prediction() {
        let predictor = ContextPredictor::new(1000);

        // Add some historical patterns
        let pattern1 = RequestPattern {
            timestamp: Instant::now(),
            task_type: ModelTask::Completion,
            complexity: Complexity::Medium,
            priority: RequestPriority::Medium,
            input_length: 100,
            selected_model: ModelId::new(),
            processing_time: Duration::from_millis(50),
            success: true,
        };

        predictor.record_request_pattern(pattern1).await.unwrap();

        // Test prediction
        let partial_context = RequestContext {
            task_type: ModelTask::Completion,
            input_length: 120,
            priority: RequestPriority::Medium,
            expected_complexity: Complexity::Medium,
            acceptable_latency: Duration::from_millis(100),
            preferred_hardware: None,
        };

        let prediction = predictor.predict_context(&partial_context).await.unwrap();
        assert_eq!(prediction.predicted_task_type, ModelTask::Completion);
        assert!(prediction.confidence_score > 0.0);
    }

    #[tokio::test]
    async fn test_trend_analysis() {
        let predictor = ContextPredictor::new(1000);

        // Add patterns
        for i in 0..5 {
            let pattern = RequestPattern {
                timestamp: Instant::now() - Duration::from_secs(i * 60),
                task_type: ModelTask::Completion,
                complexity: Complexity::Medium,
                priority: RequestPriority::High,
                input_length: 100 + i * 10,
                selected_model: ModelId::new(),
                processing_time: Duration::from_millis(40 + i as u64),
                success: i % 2 == 0,
            };
            predictor.record_request_pattern(pattern).await.unwrap();
        }

        let trends = predictor.analyze_trends().await.unwrap();
        assert!(trends.task_distribution.contains_key(&ModelTask::Completion));
        assert!(trends.avg_processing_times.contains_key(&ModelTask::Completion));
    }
}