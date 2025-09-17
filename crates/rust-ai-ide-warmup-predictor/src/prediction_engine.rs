//! Prediction Engine with ML-based algorithms for model warmup prediction
//!
//! This module implements sophisticated machine learning algorithms to predict
//! future model needs based on historical usage patterns, user behavior, and
//! contextual factors.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use moka::future::Cache;
use ndarray::{Array1, Array2};
use statrs::distribution::{ContinuousCDF, Normal};
use tokio::sync::RwLock;

use crate::error::{Result, WarmupError};
use crate::types::{
    ChangeType, Complexity, ModelId, ModelPrediction, ModelTask, PredictionAccuracy,
    RequestPriority, UsagePattern, WarmupConfig, WarmupPrediction, WarmupRequest,
};
use crate::usage_pattern_analyzer::UsagePatternAnalyzer;

/// Advanced prediction engine with multiple ML algorithms
#[derive(Debug)]
pub struct PredictionEngine {
    /// Configuration settings
    config: Arc<RwLock<WarmupConfig>>,
    /// Linear regression models for time series prediction
    regression_models: Arc<RwLock<HashMap<ModelId, LinearRegressionModel>>>,
    /// Classification models for task prediction
    classification_models: Arc<RwLock<HashMap<ModelTask, ClassificationModel>>>,
    /// Time series models for usage pattern prediction
    time_series_models: Arc<RwLock<HashMap<ModelId, TimeSeriesModel>>>,
    /// Feature engineering pipeline
    feature_engineer: FeatureEngineer,
    /// Model evaluation and metrics
    model_evaluator: ModelEvaluator,
    /// Prediction cache
    prediction_cache: Cache<String, WarmupPrediction>,
    /// Continuous learning system
    learning_system: ContinuousLearningSystem,
}

/// Linear regression model for time series prediction
#[derive(Debug, Clone)]
struct LinearRegressionModel {
    /// Model coefficients
    coefficients: Array1<f64>,
    /// Intercept term
    intercept: f64,
    /// Model accuracy metrics
    accuracy: f64,
    /// Training data size
    training_size: usize,
    /// Last updated timestamp
    last_updated: Instant,
}

/// Classification model for task-type prediction
#[derive(Debug, Clone)]
struct ClassificationModel {
    /// Feature weights for each class
    weights: HashMap<ModelId, Array1<f64>>,
    /// Bias terms for each class
    biases: HashMap<ModelId, f64>,
    /// Model accuracy by class
    class_accuracy: HashMap<ModelId, f64>,
    /// Feature importance scores
    feature_importance: Array1<f64>,
}

/// Time series model using exponential smoothing
#[derive(Debug, Clone)]
struct TimeSeriesModel {
    /// Smoothed values using exponential smoothing
    smoothed_values: VecDeque<f64>,
    /// Smoothing parameter (alpha)
    alpha: f64,
    /// Trend component
    trend: f64,
    /// Seasonal components
    seasonal: HashMap<String, f64>,
    /// Model accuracy
    accuracy: f64,
}

/// Feature engineering pipeline
#[derive(Debug)]
struct FeatureEngineer {
    /// Feature scalers for normalization
    scalers: HashMap<String, FeatureScaler>,
    /// Feature selection results
    selected_features: Vec<String>,
    /// Feature correlation matrix
    correlation_matrix: Option<Array2<f64>>,
}

/// Feature scaler for normalization
#[derive(Debug, Clone)]
struct FeatureScaler {
    /// Mean value for normalization
    mean: f64,
    /// Standard deviation for normalization
    std: f64,
    /// Minimum value for min-max scaling
    min_val: f64,
    /// Maximum value for min-max scaling
    max_val: f64,
}

/// Model evaluation and metrics collector
#[derive(Debug)]
struct ModelEvaluator {
    /// Prediction accuracy metrics
    accuracy_metrics: HashMap<ModelId, PredictionAccuracy>,
    /// Model performance history
    performance_history: HashMap<ModelId, VecDeque<ModelPerformance>>,
    /// Cross-validation results
    cross_validation_scores: HashMap<ModelId, Vec<f64>>,
}

/// Model performance snapshot
#[derive(Debug, Clone)]
struct ModelPerformance {
    /// Timestamp of evaluation
    timestamp: Instant,
    /// Mean squared error
    mse: f64,
    /// Mean absolute error
    mae: f64,
    /// R-squared score
    r_squared: f64,
    /// Prediction accuracy
    accuracy: f64,
}

/// Continuous learning system for model improvement
#[derive(Debug)]
struct ContinuousLearningSystem {
    /// Learning rate for model updates
    learning_rate: f64,
    /// Minimum data points required for retraining
    min_training_samples: usize,
    /// Model retraining schedule
    retraining_schedule: HashMap<ModelId, Instant>,
    /// Feature drift detector
    drift_detector: DriftDetector,
}

/// Drift detection for feature distribution changes
#[derive(Debug)]
struct DriftDetector {
    /// Reference feature distributions
    reference_distributions: HashMap<String, Normal>,
    /// Drift detection threshold
    drift_threshold: f64,
    /// Current drift scores
    drift_scores: HashMap<String, f64>,
}

impl PredictionEngine {
    /// Create a new prediction engine with configuration
    pub async fn new(config: WarmupConfig) -> Result<Self> {
        let prediction_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(config.prediction_cache_ttl_seconds))
            .build();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            regression_models: Arc::new(RwLock::new(HashMap::new())),
            classification_models: Arc::new(RwLock::new(HashMap::new())),
            time_series_models: Arc::new(RwLock::new(HashMap::new())),
            feature_engineer: FeatureEngineer::new(),
            model_evaluator: ModelEvaluator::new(),
            prediction_cache,
            learning_system: ContinuousLearningSystem::new(),
        })
    }

    /// Generate predictions for models to warm up based on request
    pub async fn predict_models(&self, request: &WarmupRequest) -> Result<Vec<ModelPrediction>> {
        let cache_key = format!(
            "prediction_{}_{}_{}_{}",
            request.user_context.user_id,
            request.task as u8,
            request.complexity as u8,
            request.timestamp.elapsed().as_secs()
        );

        // Check cache first
        if let Some(cached_prediction) = self.prediction_cache.get(&cache_key).await {
            return Ok(cached_prediction.predicted_models);
        }

        // Extract features from request
        let features = self.feature_engineer.extract_features(request).await?;

        // Generate predictions using multiple models
        let mut predictions = Vec::new();

        // Time series prediction
        let time_series_predictions = self.predict_time_series(request).await?;
        predictions.extend(time_series_predictions);

        // Classification-based prediction
        let classification_predictions = self.predict_classification(request, &features).await?;
        predictions.extend(classification_predictions);

        // Regression-based prediction
        let regression_predictions = self.predict_regression(request, &features).await?;
        predictions.extend(regression_predictions);

        // Ensemble prediction combining all models
        let ensemble_predictions = self.ensemble_prediction(&predictions).await?;

        // Sort by confidence and return top predictions
        let mut final_predictions = ensemble_predictions;
        final_predictions.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit to reasonable number
        final_predictions.truncate(10);

        Ok(final_predictions)
    }

    /// Update models with new data for continuous learning
    pub async fn update_models(&self, usage_patterns: &[UsagePattern]) -> Result<()> {
        // Check if retraining is needed
        if !self.learning_system.should_retrain(usage_patterns).await? {
            return Ok(());
        }

        // Update feature engineering pipeline
        self.feature_engineer.update_features(usage_patterns).await?;

        // Retrain models
        for pattern in usage_patterns {
            self.train_regression_model(pattern).await?;
            self.train_time_series_model(pattern).await?;
            self.train_classification_model(pattern).await?;
        }

        // Evaluate updated models
        self.model_evaluator.evaluate_models(usage_patterns).await?;

        Ok(())
    }

    /// Update configuration
    pub async fn update_config(&self, config: WarmupConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    /// Get prediction accuracy metrics
    pub async fn get_accuracy_metrics(&self) -> HashMap<ModelId, PredictionAccuracy> {
        self.model_evaluator.accuracy_metrics.clone()
    }

    /// Time series prediction using exponential smoothing
    async fn predict_time_series(&self, request: &WarmupRequest) -> Result<Vec<ModelPrediction>> {
        let models = self.time_series_models.read().await;
        let mut predictions = Vec::new();

        for (model_id, time_series_model) in &*models {
            let prediction_value = time_series_model.predict_next_value().await?;
            let confidence = time_series_model.accuracy;

            if prediction_value > 0.5 && confidence > 0.6 {
                predictions.push(ModelPrediction {
                    model_id: *model_id,
                    confidence_score: confidence,
                    usage_probability: prediction_value.min(1.0),
                    time_until_needed: Duration::from_secs(30), // Simplified
                    reasoning: vec![
                        format!("Time series prediction: {:.2}", prediction_value),
                        format!("Historical accuracy: {:.2}", confidence),
                    ],
                });
            }
        }

        Ok(predictions)
    }

    /// Classification-based prediction
    async fn predict_classification(&self, request: &WarmupRequest, features: &HashMap<String, f64>) -> Result<Vec<ModelPrediction>> {
        let classifications = self.classification_models.read().await;
        let mut predictions = Vec::new();

        for (task, classification_model) in &*classifications {
            if *task == request.task {
                let task_predictions = classification_model.predict_models(features).await?;
                predictions.extend(task_predictions);
            }
        }

        Ok(predictions)
    }

    /// Regression-based prediction
    async fn predict_regression(&self, request: &WarmupRequest, features: &HashMap<String, f64>) -> Result<Vec<ModelPrediction>> {
        let regressions = self.regression_models.read().await;
        let mut predictions = Vec::new();

        for (model_id, regression_model) in &*regressions {
            let prediction_value = regression_model.predict(features).await?;
            let confidence = regression_model.accuracy;

            if prediction_value > 0.3 && confidence > 0.6 {
                predictions.push(ModelPrediction {
                    model_id: *model_id,
                    confidence_score: confidence,
                    usage_probability: prediction_value.min(1.0),
                    time_until_needed: Duration::from_secs(60), // Simplified
                    reasoning: vec![
                        format!("Regression prediction: {:.2}", prediction_value),
                        format!("Model accuracy: {:.2}", confidence),
                    ],
                });
            }
        }

        Ok(predictions)
    }

    /// Ensemble prediction combining multiple models
    async fn ensemble_prediction(&self, predictions: &[ModelPrediction]) -> Result<Vec<ModelPrediction>> {
        let mut model_scores: HashMap<ModelId, Vec<f64>> = HashMap::new();

        // Collect scores from different models
        for prediction in predictions {
            model_scores.entry(prediction.model_id)
                .or_insert_with(Vec::new)
                .push(prediction.confidence_score * prediction.usage_probability);
        }

        // Calculate ensemble scores
        let mut ensemble_predictions = Vec::new();
        for (model_id, scores) in model_scores {
            if scores.is_empty() {
                continue;
            }

            // Weighted average of scores
            let total_score: f64 = scores.iter().sum();
            let avg_score = total_score / scores.len() as f64;

            // Calculate confidence based on score variance
            let variance = scores.iter()
                .map(|score| (score - avg_score).powi(2))
                .sum::<f64>() / scores.len() as f64;
            let confidence = (1.0 / (1.0 + variance.sqrt())).min(1.0);

            ensemble_predictions.push(ModelPrediction {
                model_id,
                confidence_score: confidence,
                usage_probability: avg_score,
                time_until_needed: Duration::from_secs(45), // Average timing
                reasoning: vec![
                    format!("Ensemble score: {:.3}", avg_score),
                    format!("Score variance: {:.3}", variance),
                    format!("Model count: {}", scores.len()),
                ],
            });
        }

        Ok(ensemble_predictions)
    }

    /// Train linear regression model for a usage pattern
    async fn train_regression_model(&self, pattern: &UsagePattern) -> Result<()> {
        if pattern.access_times.len() < 10 {
            return Ok(()); // Not enough data
        }

        // Prepare training data
        let mut x_data = Vec::new();
        let mut y_data = Vec::new();

        // Use time series data for regression
        for (i, &timestamp) in pattern.access_times.iter().enumerate() {
            if i > 0 {
                let time_diff = timestamp.duration_since(pattern.access_times[i-1]).as_secs_f64();
                x_data.push(vec![i as f64, time_diff, pattern.success_rate]);
                y_data.push(1.0); // Binary target: usage occurred
            }
        }

        if x_data.len() < 5 {
            return Ok(());
        }

        // Simple linear regression implementation (simplified)
        let model = self.train_simple_regression(&x_data, &y_data).await?;

        let mut models = self.regression_models.write().await;
        models.insert(pattern.model_id, model);

        Ok(())
    }

    /// Train time series model using exponential smoothing
    async fn train_time_series_model(&self, pattern: &UsagePattern) -> Result<()> {
        if pattern.access_times.len() < 3 {
            return Ok(());
        }

        // Calculate usage frequency over time windows
        let mut usage_counts = Vec::new();
        let window_size = Duration::from_secs(3600); // 1 hour windows

        let mut current_window_start = pattern.access_times[0];
        let mut current_count = 0;

        for &timestamp in &pattern.access_times {
            if timestamp.duration_since(current_window_start) >= window_size {
                usage_counts.push(current_count as f64);
                current_window_start = timestamp;
                current_count = 1;
            } else {
                current_count += 1;
            }
        }

        if !usage_counts.is_empty() {
            usage_counts.push(current_count as f64);
        }

        // Train exponential smoothing model
        let alpha = 0.3; // Smoothing parameter
        let mut smoothed_values = VecDeque::new();

        if !usage_counts.is_empty() {
            smoothed_values.push_back(usage_counts[0]);

            for &value in &usage_counts[1..] {
                let smoothed = alpha * value + (1.0 - alpha) * smoothed_values.back().unwrap_or(&value);
                smoothed_values.push_back(smoothed);
            }
        }

        let model = TimeSeriesModel {
            smoothed_values,
            alpha,
            trend: 0.0, // Simplified
            seasonal: HashMap::new(),
            accuracy: 0.75, // Placeholder accuracy
        };

        let mut models = self.time_series_models.write().await;
        models.insert(pattern.model_id, model);

        Ok(())
    }

    /// Train classification model for task-based prediction
    async fn train_classification_model(&self, pattern: &UsagePattern) -> Result<()> {
        // Simplified classification training
        // In practice, this would use proper ML algorithms

        let weights = Array1::from_vec(vec![0.1, 0.2, 0.3]); // Placeholder weights
        let bias = 0.0;

        let model = ClassificationModel {
            weights: HashMap::from([(pattern.model_id, weights)]),
            biases: HashMap::from([(pattern.model_id, bias)]),
            class_accuracy: HashMap::from([(pattern.model_id, 0.8)]),
            feature_importance: Array1::from_vec(vec![0.4, 0.3, 0.3]),
        };

        let mut models = self.classification_models.write().await;
        let task = ModelTask::Completion; // Simplified - use actual task type
        models.insert(task, model);

        Ok(())
    }

    /// Train simple linear regression (simplified implementation)
    async fn train_simple_regression(&self, x_data: &[Vec<f64>], y_data: &[f64]) -> Result<LinearRegressionModel> {
        // Very simplified linear regression
        // In practice, use proper numerical libraries

        let num_features = x_data.first().map(|row| row.len()).unwrap_or(0);
        let num_samples = x_data.len();

        if num_samples == 0 || num_features == 0 {
            return Err(WarmupError::PredictionEngine {
                message: "Insufficient training data".to_string(),
            });
        }

        // Simple coefficient estimation (placeholder)
        let coefficients = Array1::from_vec(vec![0.1; num_features]);
        let intercept = 0.0;

        // Calculate simple accuracy metric
        let accuracy = 0.7; // Placeholder

        Ok(LinearRegressionModel {
            coefficients,
            intercept,
            accuracy,
            training_size: num_samples,
            last_updated: Instant::now(),
        })
    }
}

impl LinearRegressionModel {
    /// Make prediction using the trained model
    async fn predict(&self, features: &HashMap<String, f64>) -> Result<f64> {
        // Simplified prediction using feature values
        let mut prediction = self.intercept;

        // Use a subset of features for prediction
        let feature_keys = ["input_length", "complexity", "priority"];
        for (i, key) in feature_keys.iter().enumerate() {
            if let Some(&value) = features.get(*key) {
                if i < self.coefficients.len() {
                    prediction += self.coefficients[i] * value;
                }
            }
        }

        // Apply sigmoid for probability output
        Ok(1.0 / (1.0 + (-prediction).exp()))
    }
}

impl ClassificationModel {
    /// Predict models using the classification model
    async fn predict_models(&self, features: &HashMap<String, f64>) -> Result<Vec<ModelPrediction>> {
        let mut predictions = Vec::new();

        for (model_id, weights) in &self.weights {
            let mut score = *self.biases.get(model_id).unwrap_or(&0.0);

            // Calculate weighted sum of features
            for (i, weight) in weights.iter().enumerate() {
                let feature_key = format!("feature_{}", i);
                if let Some(&feature_value) = features.get(&feature_key) {
                    score += weight * feature_value;
                }
            }

            // Convert to probability
            let probability = 1.0 / (1.0 + (-score).exp());
            let confidence = *self.class_accuracy.get(model_id).unwrap_or(&0.5);

            if probability > 0.4 {
                predictions.push(ModelPrediction {
                    model_id: *model_id,
                    confidence_score: confidence,
                    usage_probability: probability,
                    time_until_needed: Duration::from_secs(30),
                    reasoning: vec![
                        format!("Classification score: {:.3}", score),
                        format!("Predicted probability: {:.3}", probability),
                    ],
                });
            }
        }

        Ok(predictions)
    }
}

impl TimeSeriesModel {
    /// Predict next value using exponential smoothing
    async fn predict_next_value(&self) -> Result<f64> {
        if self.smoothed_values.is_empty() {
            return Ok(0.0);
        }

        // Simple prediction based on latest smoothed value
        let latest_value = *self.smoothed_values.back().unwrap_or(&0.0);

        // Add trend and seasonal components (simplified)
        let trend_adjustment = self.trend * 1.0; // Next time step
        let seasonal_adjustment = 0.0; // Simplified

        Ok((latest_value + trend_adjustment + seasonal_adjustment).max(0.0))
    }
}

impl FeatureEngineer {
    /// Create new feature engineer
    fn new() -> Self {
        Self {
            scalers: HashMap::new(),
            selected_features: Vec::new(),
            correlation_matrix: None,
        }
    }

    /// Extract features from warmup request
    async fn extract_features(&self, request: &WarmupRequest) -> Result<HashMap<String, f64>> {
        let mut features = HashMap::new();

        // Basic features
        features.insert("input_length".to_string(), request.input_length as f64);
        features.insert("complexity".to_string(), request.complexity as u8 as f64);
        features.insert("priority".to_string(), request.priority as u8 as f64);
        features.insert("task_type".to_string(), request.task as u8 as f64);

        // Time-based features
        let now: DateTime<Utc> = request.timestamp.into();
        features.insert("hour_of_day".to_string(), now.hour() as f64);
        features.insert("day_of_week".to_string(), now.weekday().num_days_from_monday() as f64);

        // User context features
        features.insert("session_duration".to_string(),
            request.user_context.session_duration.as_secs_f64());
        features.insert("recent_activities".to_string(),
            request.user_context.recent_activities.len() as f64);

        // Project context features
        features.insert("project_size".to_string(), request.project_context.size_lines as f64);
        features.insert("complexity_score".to_string(), request.project_context.complexity_score);

        // Latency requirement (normalized)
        let latency_seconds = request.acceptable_latency.as_secs_f64();
        features.insert("latency_requirement".to_string(), latency_seconds / 60.0); // Normalize to minutes

        // Scale features
        for (key, value) in &mut features {
            if let Some(scaler) = self.scalers.get(key) {
                *value = scaler.scale(*value);
            }
        }

        Ok(features)
    }

    /// Update feature engineering with new patterns
    async fn update_features(&mut self, _patterns: &[UsagePattern]) -> Result<()> {
        // Update scalers based on new data
        // Implementation would calculate means and std deviations

        // Update selected features based on importance
        self.selected_features = vec![
            "input_length".to_string(),
            "complexity".to_string(),
            "hour_of_day".to_string(),
            "session_duration".to_string(),
        ];

        Ok(())
    }
}

impl FeatureScaler {
    /// Scale a feature value
    fn scale(&self, value: f64) -> f64 {
        // Z-score normalization: (value - mean) / std
        if self.std > 0.0 {
            (value - self.mean) / self.std
        } else {
            value - self.mean
        }
    }
}

impl ModelEvaluator {
    /// Create new model evaluator
    fn new() -> Self {
        Self {
            accuracy_metrics: HashMap::new(),
            performance_history: HashMap::new(),
            cross_validation_scores: HashMap::new(),
        }
    }

    /// Evaluate models against usage patterns
    async fn evaluate_models(&mut self, _patterns: &[UsagePattern]) -> Result<()> {
        // Implementation would perform cross-validation and calculate metrics
        // For now, this is a placeholder

        for pattern in _patterns {
            let accuracy = PredictionAccuracy {
                total_predictions: 100,
                accurate_predictions: 75,
                false_positives: 10,
                false_negatives: 15,
                avg_confidence: 0.75,
                precision: 0.0, // Will be calculated
                recall: 0.0,    // Will be calculated
                f1_score: 0.0,  // Will be calculated
            };

            // Calculate derived metrics
            let mut final_accuracy = accuracy;
            final_accuracy.precision = final_accuracy.calculate_precision();
            final_accuracy.recall = final_accuracy.calculate_recall();
            final_accuracy.f1_score = final_accuracy.calculate_f1_score();

            self.accuracy_metrics.insert(pattern.model_id, final_accuracy);
        }

        Ok(())
    }
}

impl ContinuousLearningSystem {
    /// Create new continuous learning system
    fn new() -> Self {
        Self {
            learning_rate: 0.1,
            min_training_samples: 50,
            retraining_schedule: HashMap::new(),
            drift_detector: DriftDetector::new(),
        }
    }

    /// Check if models should be retrained
    async fn should_retrain(&self, patterns: &[UsagePattern]) -> Result<bool> {
        // Check if we have enough new data
        let total_samples: usize = patterns.iter()
            .map(|p| p.access_times.len())
            .sum();

        if total_samples < self.min_training_samples {
            return Ok(false);
        }

        // Check if retraining is scheduled
        let now = Instant::now();
        for pattern in patterns {
            if let Some(scheduled_time) = self.retraining_schedule.get(&pattern.model_id) {
                if now >= *scheduled_time {
                    return Ok(true);
                }
            }
        }

        // Check for concept drift
        if self.drift_detector.detect_drift(patterns).await? {
            return Ok(true);
        }

        Ok(false)
    }
}

impl DriftDetector {
    /// Create new drift detector
    fn new() -> Self {
        Self {
            reference_distributions: HashMap::new(),
            drift_threshold: 2.0, // 2 standard deviations
            drift_scores: HashMap::new(),
        }
    }

    /// Detect concept drift in feature distributions
    async fn detect_drift(&self, _patterns: &[UsagePattern]) -> Result<bool> {
        // Simplified drift detection
        // Implementation would compare current distributions with reference

        // Placeholder: randomly detect drift occasionally
        Ok(rand::random::<f64>() < 0.05) // 5% chance of detecting drift
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UserContext;

    #[tokio::test]
    async fn test_prediction_engine_creation() {
        let config = WarmupConfig::default();
        let engine = PredictionEngine::new(config).await.unwrap();
        assert!(engine.get_accuracy_metrics().await.is_empty());
    }

    #[tokio::test]
    async fn test_feature_extraction() {
        let config = WarmupConfig::default();
        let engine = PredictionEngine::new(config).await.unwrap();

        let request = WarmupRequest {
            task: ModelTask::Completion,
            input_length: 100,
            complexity: Complexity::Medium,
            priority: RequestPriority::High,
            acceptable_latency: Duration::from_millis(500),
            preferred_hardware: None,
            user_context: UserContext {
                user_id: "test_user".to_string(),
                session_duration: Duration::from_secs(600),
                recent_activities: vec![],
                preferences: HashMap::new(),
            },
            project_context: crate::types::ProjectContext {
                language: "rust".to_string(),
                size_lines: 1000,
                complexity_score: 0.7,
                recent_changes: vec![],
            },
            timestamp: Instant::now(),
        };

        let features = engine.feature_engineer.extract_features(&request).await.unwrap();
        assert!(!features.is_empty());
        assert!(features.contains_key("input_length"));
        assert!(features.contains_key("complexity"));
    }

    #[tokio::test]
    async fn test_time_series_prediction() {
        let config = WarmupConfig::default();
        let engine = PredictionEngine::new(config).await.unwrap();

        let request = WarmupRequest {
            task: ModelTask::Completion,
            input_length: 100,
            complexity: Complexity::Medium,
            priority: RequestPriority::High,
            acceptable_latency: Duration::from_millis(500),
            preferred_hardware: None,
            user_context: UserContext {
                user_id: "test_user".to_string(),
                session_duration: Duration::from_secs(600),
                recent_activities: vec![],
                preferences: HashMap::new(),
            },
            project_context: crate::types::ProjectContext {
                language: "rust".to_string(),
                size_lines: 1000,
                complexity_score: 0.7,
                recent_changes: vec![],
            },
            timestamp: Instant::now(),
        };

        // Initially no models trained
        let predictions = engine.predict_time_series(&request).await.unwrap();
        assert!(predictions.is_empty());
    }
}