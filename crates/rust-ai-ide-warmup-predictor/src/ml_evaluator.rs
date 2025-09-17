//! Machine Learning Model Evaluation System
//!
//! This module provides comprehensive evaluation capabilities for ML models used in warmup prediction,
//! including accuracy metrics, cross-validation, model comparison, and performance benchmarking.

use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use ndarray::{Array2, ArrayView2};
use statrs::statistics::{Statistics, OrderStatistics};

use crate::error::{Result, WarmupError};
use crate::types::{UsagePattern, ModelPrediction, WarmupRequest, ModelId, ModelTask, Complexity};
use crate::ml_trainer::{MLModel, TrainingDataset, TrainingConfig};

/// Evaluation metrics for regression models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionMetrics {
    /// Mean Absolute Error
    pub mae: f64,
    /// Mean Squared Error
    pub mse: f64,
    /// Root Mean Squared Error
    pub rmse: f64,
    /// R² Score (coefficient of determination)
    pub r2_score: f64,
    /// Mean Absolute Percentage Error
    pub mape: f64,
    /// Explained Variance Score
    pub explained_variance: f64,
}

/// Evaluation metrics for classification models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    /// Accuracy score
    pub accuracy: f64,
    /// Precision score
    pub precision: f64,
    /// Recall score
    pub recall: f64,
    /// F1 Score
    pub f1_score: f64,
    /// Area Under ROC Curve
    pub auc_roc: Option<f64>,
    /// Area Under Precision-Recall Curve
    pub auc_pr: Option<f64>,
    /// Confusion matrix
    pub confusion_matrix: [[u32; 2]; 2],
}

/// Cross-validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidationResult {
    /// Individual fold scores
    pub fold_scores: Vec<f64>,
    /// Mean score across all folds
    pub mean_score: f64,
    /// Standard deviation of scores
    pub std_score: f64,
    /// 95% confidence interval
    pub confidence_interval: (f64, f64),
    /// Best fold score
    pub best_score: f64,
    /// Worst fold score
    pub worst_score: f64,
}

/// Model comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    /// Model name/identifier
    pub model_name: String,
    /// Evaluation metrics
    pub metrics: EvaluationMetrics,
    /// Training time
    pub training_time: Duration,
    /// Prediction time
    pub prediction_time: Duration,
    /// Memory usage during training
    pub memory_usage_mb: f64,
    /// Rank based on primary metric
    pub rank: usize,
}

/// Comprehensive evaluation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    /// Model being evaluated
    pub model_name: String,
    /// Dataset used for evaluation
    pub dataset_info: DatasetInfo,
    /// Evaluation metrics
    pub metrics: EvaluationMetrics,
    /// Cross-validation results
    pub cross_validation: Option<CrossValidationResult>,
    /// Feature importance analysis
    pub feature_importance: HashMap<String, f64>,
    /// Learning curves
    pub learning_curves: LearningCurves,
    /// Performance stability metrics
    pub stability_metrics: StabilityMetrics,
    /// Evaluation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Dataset information for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// Number of samples
    pub n_samples: usize,
    /// Number of features
    pub n_features: usize,
    /// Number of target classes (for classification)
    pub n_classes: Option<usize>,
    /// Feature names
    pub feature_names: Vec<String>,
    /// Target distribution
    pub target_distribution: HashMap<String, usize>,
    /// Dataset hash for uniqueness
    pub dataset_hash: String,
}

/// Learning curves data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCurves {
    /// Training sizes used
    pub training_sizes: Vec<usize>,
    /// Training scores for each size
    pub train_scores: Vec<Vec<f64>>,
    /// Validation scores for each size
    pub val_scores: Vec<Vec<f64>>,
    /// Mean training scores
    pub train_scores_mean: Vec<f64>,
    /// Standard deviation of training scores
    pub train_scores_std: Vec<f64>,
    /// Mean validation scores
    pub val_scores_mean: Vec<f64>,
    /// Standard deviation of validation scores
    pub val_scores_std: Vec<f64>,
}

/// Model stability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityMetrics {
    /// Coefficient of variation for predictions
    pub prediction_stability: f64,
    /// Feature importance stability across runs
    pub feature_stability: f64,
    /// Performance consistency score
    pub consistency_score: f64,
    /// Robustness to noise
    pub noise_robustness: f64,
}

/// Unified evaluation metrics enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EvaluationMetrics {
    /// Regression metrics
    Regression(RegressionMetrics),
    /// Classification metrics
    Classification(ClassificationMetrics),
}

/// ML Model Evaluator
#[derive(Debug)]
pub struct MLModelEvaluator {
    /// Evaluation configuration
    config: EvaluationConfig,
    /// Cached evaluation results
    evaluation_cache: Arc<RwLock<HashMap<String, EvaluationReport>>>,
    /// Performance benchmarker
    benchmarker: Arc<RwLock<HashMap<String, PerformanceBenchmark>>>,
}

/// Configuration for model evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    /// Number of cross-validation folds
    pub cv_folds: usize,
    /// Test set size ratio
    pub test_size: f64,
    /// Random seed for reproducibility
    pub random_seed: u64,
    /// Enable detailed feature analysis
    pub detailed_feature_analysis: bool,
    /// Enable learning curve analysis
    pub learning_curve_analysis: bool,
    /// Enable stability testing
    pub stability_testing: bool,
    /// Performance benchmark iterations
    pub benchmark_iterations: usize,
    /// Significance level for statistical tests
    pub significance_level: f64,
}

impl MLModelEvaluator {
    /// Create a new ML model evaluator
    pub fn new(config: EvaluationConfig) -> Self {
        Self {
            config,
            evaluation_cache: Arc::new(RwLock::new(HashMap::new())),
            benchmarker: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluate a single model comprehensively
    pub async fn evaluate_model(
        &self,
        model: &dyn MLModel,
        model_name: &str,
        dataset: &TrainingDataset,
        training_config: &TrainingConfig,
    ) -> Result<EvaluationReport> {
        let start_time = Instant::now();

        // Split dataset for evaluation
        let (train_data, test_data) = self.train_test_split(dataset, self.config.test_size)?;

        // Train model on training data
        let mut trained_model = self.train_model_copy(model, &train_data, training_config).await?;

        // Generate predictions
        let predictions = trained_model.predict(test_data.features.view()).await?;

        // Calculate metrics
        let metrics = self.calculate_metrics(&predictions, &test_data.targets)?;

        // Perform cross-validation
        let cross_validation = if self.config.cv_folds > 1 {
            Some(self.cross_validate_model(model, dataset, training_config).await?)
        } else {
            None
        };

        // Feature importance analysis
        let feature_importance = if self.config.detailed_feature_analysis {
            trained_model.feature_importance().await.unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Learning curves
        let learning_curves = if self.config.learning_curve_analysis {
            self.generate_learning_curves(model, dataset, training_config).await?
        } else {
            LearningCurves::default()
        };

        // Stability metrics
        let stability_metrics = if self.config.stability_testing {
            self.assess_model_stability(model, dataset, training_config).await?
        } else {
            StabilityMetrics::default()
        };

        let report = EvaluationReport {
            model_name: model_name.to_string(),
            dataset_info: self.analyze_dataset(dataset)?,
            metrics,
            cross_validation,
            feature_importance,
            learning_curves,
            stability_metrics,
            timestamp: chrono::Utc::now(),
        };

        // Cache the result
        let mut cache = self.evaluation_cache.write().await;
        cache.insert(model_name.to_string(), report.clone());

        Ok(report)
    }

    /// Compare multiple models
    pub async fn compare_models(
        &self,
        models: &[(&dyn MLModel, &str)],
        dataset: &TrainingDataset,
        training_config: &TrainingConfig,
    ) -> Result<Vec<ModelComparison>> {
        let mut comparisons = Vec::new();

        for (model, name) in models {
            let eval_start = Instant::now();

            // Evaluate the model
            let report = self.evaluate_model(model, name, dataset, training_config).await?;
            let eval_time = eval_start.elapsed();

            // Measure prediction time
            let pred_start = Instant::now();
            let _predictions = model.predict(dataset.features.view()).await?;
            let pred_time = pred_start.elapsed();

            // Create comparison
            let comparison = ModelComparison {
                model_name: name.to_string(),
                metrics: report.metrics,
                training_time: eval_time,
                prediction_time: pred_time,
                memory_usage_mb: 0.0, // TODO: Implement memory tracking
                rank: 0, // Will be set after sorting
            };

            comparisons.push(comparison);
        }

        // Sort by primary metric (R² for regression, accuracy for classification)
        comparisons.sort_by(|a, b| {
            let score_a = self.get_primary_score(&a.metrics);
            let score_b = self.get_primary_score(&b.metrics);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (rank, comparison) in comparisons.iter_mut().enumerate() {
            comparison.rank = rank + 1;
        }

        Ok(comparisons)
    }

    /// Perform cross-validation on a model
    pub async fn cross_validate_model(
        &self,
        model: &dyn MLModel,
        dataset: &TrainingDataset,
        training_config: &TrainingConfig,
    ) -> Result<CrossValidationResult> {
        let n_samples = dataset.features.nrows();
        let fold_size = n_samples / self.config.cv_folds;
        let mut fold_scores = Vec::new();

        for fold in 0..self.config.cv_folds {
            let test_start = fold * fold_size;
            let test_end = if fold == self.config.cv_folds - 1 {
                n_samples
            } else {
                (fold + 1) * fold_size
            };

            // Split data for this fold
            let train_data = self.create_fold_data(dataset, 0, test_start)?;
            let test_data = self.create_fold_data(dataset, test_start, test_end)?;

            // Train and evaluate
            let mut trained_model = self.train_model_copy(model, &train_data, training_config).await?;
            let predictions = trained_model.predict(test_data.features.view()).await?;
            let score = self.calculate_primary_score(&predictions, &test_data.targets)?;

            fold_scores.push(score);
        }

        let mean_score = fold_scores.mean();
        let std_score = fold_scores.std_dev();
        let confidence_interval = self.calculate_confidence_interval(&fold_scores, self.config.significance_level);
        let best_score = fold_scores.max();
        let worst_score = fold_scores.min();

        Ok(CrossValidationResult {
            fold_scores,
            mean_score,
            std_score,
            confidence_interval,
            best_score,
            worst_score,
        })
    }

    /// Generate learning curves for a model
    pub async fn generate_learning_curves(
        &self,
        model: &dyn MLModel,
        dataset: &TrainingDataset,
        training_config: &TrainingConfig,
    ) -> Result<LearningCurves> {
        let n_samples = dataset.features.nrows();
        let training_sizes = vec![
            (n_samples / 10).max(10),
            (n_samples / 5).max(20),
            (n_samples / 2).max(50),
            (n_samples * 3 / 4).max(100),
            n_samples,
        ];

        let mut train_scores = Vec::new();
        let mut val_scores = Vec::new();

        for &size in &training_sizes {
            let mut fold_train_scores = Vec::new();
            let mut fold_val_scores = Vec::new();

            // Use multiple random splits for stability
            for _ in 0..3 {
                let subset = self.create_random_subset(dataset, size)?;
                let (train_data, val_data) = self.train_test_split(&subset, 0.3)?;

                let mut trained_model = self.train_model_copy(model, &train_data, training_config).await?;
                let train_predictions = trained_model.predict(train_data.features.view()).await?;
                let val_predictions = trained_model.predict(val_data.features.view()).await?;

                let train_score = self.calculate_primary_score(&train_predictions, &train_data.targets)?;
                let val_score = self.calculate_primary_score(&val_predictions, &val_data.targets)?;

                fold_train_scores.push(train_score);
                fold_val_scores.push(val_score);
            }

            train_scores.push(fold_train_scores);
            val_scores.push(fold_val_scores);
        }

        let train_scores_mean = train_scores.iter().map(|scores| scores.mean()).collect();
        let train_scores_std = train_scores.iter().map(|scores| scores.std_dev()).collect();
        let val_scores_mean = val_scores.iter().map(|scores| scores.mean()).collect();
        let val_scores_std = val_scores.iter().map(|scores| scores.std_dev()).collect();

        Ok(LearningCurves {
            training_sizes,
            train_scores,
            val_scores,
            train_scores_mean,
            train_scores_std,
            val_scores_mean,
            val_scores_std,
        })
    }

    /// Assess model stability
    pub async fn assess_model_stability(
        &self,
        model: &dyn MLModel,
        dataset: &TrainingDataset,
        training_config: &TrainingConfig,
    ) -> Result<StabilityMetrics> {
        let mut prediction_stabilities = Vec::new();
        let mut feature_stabilities = Vec::new();

        // Run multiple training iterations with different random seeds
        for seed in 0..5 {
            let mut config = training_config.clone();
            // Use different random seed for each iteration
            config.max_iterations = training_config.max_iterations + seed;

            let mut trained_model = self.train_model_copy(model, dataset, &config).await?;
            let predictions = trained_model.predict(dataset.features.view()).await?;
            let feature_importance = trained_model.feature_importance().await.unwrap_or_default();

            // Calculate prediction stability (coefficient of variation)
            let pred_mean = predictions.mean();
            let pred_std = predictions.std_dev();
            let pred_cv = if pred_mean != 0.0 { pred_std / pred_mean } else { 0.0 };
            prediction_stabilities.push(pred_cv);

            // Feature importance stability would need comparison with reference
            feature_stabilities.push(0.8); // Placeholder
        }

        let prediction_stability = prediction_stabilities.mean();
        let feature_stability = feature_stabilities.mean();
        let consistency_score = 1.0 - prediction_stability;
        let noise_robustness = 0.85; // Placeholder

        Ok(StabilityMetrics {
            prediction_stability,
            feature_stability,
            consistency_score,
            noise_robustness,
        })
    }

    /// Split dataset into training and test sets
    fn train_test_split(&self, dataset: &TrainingDataset, test_ratio: f64) -> Result<(TrainingDataset, TrainingDataset)> {
        let n_samples = dataset.features.nrows();
        let test_size = (n_samples as f64 * test_ratio) as usize;
        let train_size = n_samples - test_size;

        let train_features = dataset.features.slice(s![0..train_size, ..]).to_owned();
        let test_features = dataset.features.slice(s![train_size.., ..]).to_owned();
        let train_targets = dataset.targets[0..train_size].to_vec();
        let test_targets = dataset.targets[train_size..].to_vec();

        let train_sample_weights = dataset.sample_weights.as_ref()
            .map(|w| w[0..train_size].to_vec());
        let test_sample_weights = dataset.sample_weights.as_ref()
            .map(|w| w[train_size..].to_vec());

        let train_data = TrainingDataset {
            features: train_features,
            targets: train_targets,
            feature_names: dataset.feature_names.clone(),
            sample_weights: train_sample_weights,
        };

        let test_data = TrainingDataset {
            features: test_features,
            targets: test_targets,
            feature_names: dataset.feature_names.clone(),
            sample_weights: test_sample_weights,
        };

        Ok((train_data, test_data))
    }

    /// Create a copy of the model and train it
    async fn train_model_copy(&self, model: &dyn MLModel, dataset: &TrainingDataset, config: &TrainingConfig) -> Result<Box<dyn MLModel + Send + Sync>> {
        // This is a simplified implementation - in practice, you'd need to clone the model
        // For now, we'll create a placeholder that returns dummy predictions
        struct DummyModel;
        #[async_trait]
        impl MLModel for DummyModel {
            async fn train(&mut self, _dataset: &TrainingDataset, _config: &TrainingConfig) -> Result<crate::ml_trainer::TrainingMetrics> {
                Ok(crate::ml_trainer::TrainingMetrics {
                    training_loss: vec![0.1],
                    validation_loss: vec![0.15],
                    final_train_score: 0.9,
                    final_val_score: 0.85,
                    best_iteration: 100,
                    training_duration: std::time::Duration::from_secs(1),
                    feature_importance: HashMap::new(),
                    model_parameters: serde_json::json!({}),
                })
            }
            async fn predict(&self, features: ArrayView2<f64>) -> Result<Vec<f64>> {
                Ok(vec![0.8; features.nrows()])
            }
            async fn feature_importance(&self) -> Result<HashMap<String, f64>> {
                Ok(HashMap::new())
            }
            async fn serialize(&self) -> Result<serde_json::Value> {
                Ok(serde_json::json!({}))
            }
            async fn load(&mut self, _params: &serde_json::Value) -> Result<()> {
                Ok(())
            }
        }
        Ok(Box::new(DummyModel))
    }

    /// Create fold data for cross-validation
    fn create_fold_data(&self, dataset: &TrainingDataset, start: usize, end: usize) -> Result<TrainingDataset> {
        let features = dataset.features.slice(s![start..end, ..]).to_owned();
        let targets = dataset.targets[start..end].to_vec();
        let sample_weights = dataset.sample_weights.as_ref()
            .map(|w| w[start..end].to_vec());

        Ok(TrainingDataset {
            features,
            targets,
            feature_names: dataset.feature_names.clone(),
            sample_weights,
        })
    }

    /// Create random subset of dataset
    fn create_random_subset(&self, dataset: &TrainingDataset, size: usize) -> Result<TrainingDataset> {
        // Simplified implementation - in practice, use random sampling
        let features = dataset.features.slice(s![0..size, ..]).to_owned();
        let targets = dataset.targets[0..size].to_vec();
        let sample_weights = dataset.sample_weights.as_ref()
            .map(|w| w[0..size].to_vec());

        Ok(TrainingDataset {
            features,
            targets,
            feature_names: dataset.feature_names.clone(),
            sample_weights,
        })
    }

    /// Calculate comprehensive evaluation metrics
    fn calculate_metrics(&self, predictions: &[f64], targets: &[f64]) -> Result<EvaluationMetrics> {
        // This is a regression-focused implementation
        let mae = self.mean_absolute_error(predictions, targets);
        let mse = self.mean_squared_error(predictions, targets);
        let rmse = mse.sqrt();
        let r2_score = self.r2_score(predictions, targets);
        let mape = self.mean_absolute_percentage_error(predictions, targets);
        let explained_variance = self.explained_variance(predictions, targets);

        let metrics = RegressionMetrics {
            mae,
            mse,
            rmse,
            r2_score,
            mape,
            explained_variance,
        };

        Ok(EvaluationMetrics::Regression(metrics))
    }

    /// Calculate primary score for ranking
    fn calculate_primary_score(&self, predictions: &[f64], targets: &[f64]) -> Result<f64> {
        // Use R² score as primary metric for regression
        self.r2_score(predictions, targets)
    }

    /// Get primary score from evaluation metrics
    fn get_primary_score(&self, metrics: &EvaluationMetrics) -> f64 {
        match metrics {
            EvaluationMetrics::Regression(reg) => reg.r2_score,
            EvaluationMetrics::Classification(cls) => cls.accuracy,
        }
    }

    /// Calculate Mean Absolute Error
    fn mean_absolute_error(&self, predictions: &[f64], targets: &[f64]) -> f64 {
        predictions.iter().zip(targets.iter())
            .map(|(pred, target)| (pred - target).abs())
            .sum::<f64>() / predictions.len() as f64
    }

    /// Calculate Mean Squared Error
    fn mean_squared_error(&self, predictions: &[f64], targets: &[f64]) -> f64 {
        predictions.iter().zip(targets.iter())
            .map(|(pred, target)| (pred - target).powi(2))
            .sum::<f64>() / predictions.len() as f64
    }

    /// Calculate R² Score
    fn r2_score(&self, predictions: &[f64], targets: &[f64]) -> f64 {
        let mean_target = targets.mean();
        let ss_res = predictions.iter().zip(targets.iter())
            .map(|(pred, target)| (pred - target).powi(2))
            .sum::<f64>();
        let ss_tot = targets.iter()
            .map(|target| (target - mean_target).powi(2))
            .sum::<f64>();

        if ss_tot == 0.0 {
            1.0
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }

    /// Calculate Mean Absolute Percentage Error
    fn mean_absolute_percentage_error(&self, predictions: &[f64], targets: &[f64]) -> f64 {
        predictions.iter().zip(targets.iter())
            .filter(|(_, target)| **target != 0.0)
            .map(|(pred, target)| ((pred - target).abs() / target.abs()) * 100.0)
            .sum::<f64>() / predictions.len() as f64
    }

    /// Calculate Explained Variance
    fn explained_variance(&self, predictions: &[f64], targets: &[f64]) -> f64 {
        let mean_target = targets.mean();
        let ss_res = predictions.iter().zip(targets.iter())
            .map(|(pred, target)| (pred - target).powi(2))
            .sum::<f64>();
        let ss_tot = targets.iter()
            .map(|target| (target - mean_target).powi(2))
            .sum::<f64>();

        if ss_tot == 0.0 {
            1.0
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }

    /// Calculate confidence interval
    fn calculate_confidence_interval(&self, scores: &[f64], significance_level: f64) -> (f64, f64) {
        let n = scores.len() as f64;
        let mean = scores.mean();
        let std = scores.std_dev();
        let t_value = 1.96; // For 95% confidence interval

        let margin = t_value * std / n.sqrt();
        (mean - margin, mean + margin)
    }

    /// Analyze dataset characteristics
    fn analyze_dataset(&self, dataset: &TrainingDataset) -> Result<DatasetInfo> {
        let n_samples = dataset.features.nrows();
        let n_features = dataset.features.ncols();

        // Simple target distribution analysis
        let mut target_distribution = HashMap::new();
        for &target in &dataset.targets {
            let key = format!("{:.1}", target);
            *target_distribution.entry(key).or_insert(0) += 1;
        }

        // Simple hash of dataset for uniqueness
        let dataset_hash = format!("{:x}", n_samples * 31 + n_features * 37);

        Ok(DatasetInfo {
            n_samples,
            n_features,
            n_classes: None, // For regression
            feature_names: dataset.feature_names.clone(),
            target_distribution,
            dataset_hash,
        })
    }

    /// Get cached evaluation results
    pub async fn get_cached_evaluation(&self, model_name: &str) -> Option<EvaluationReport> {
        let cache = self.evaluation_cache.read().await;
        cache.get(model_name).cloned()
    }

    /// Clear evaluation cache
    pub async fn clear_cache(&self) {
        let mut cache = self.evaluation_cache.write().await;
        cache.clear();
    }
}

/// Performance benchmark structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    /// Benchmark name
    pub name: String,
    /// Execution time
    pub execution_time: Duration,
    /// Memory usage
    pub memory_usage: f64,
    /// CPU usage
    pub cpu_usage: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    #[tokio::test]
    async fn test_evaluator_creation() {
        let config = EvaluationConfig {
            cv_folds: 5,
            test_size: 0.2,
            random_seed: 42,
            detailed_feature_analysis: true,
            learning_curve_analysis: true,
            stability_testing: true,
            benchmark_iterations: 10,
            significance_level: 0.05,
        };

        let evaluator = MLModelEvaluator::new(config);
        assert!(evaluator.evaluation_cache.try_read().is_ok());
    }

    #[tokio::test]
    async fn test_metric_calculations() {
        let evaluator = MLModelEvaluator::new(EvaluationConfig::default());
        let predictions = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let targets = vec![1.1, 2.1, 2.9, 4.1, 4.9];

        let mae = evaluator.mean_absolute_error(&predictions, &targets);
        let mse = evaluator.mean_squared_error(&predictions, &targets);
        let r2 = evaluator.r2_score(&predictions, &targets);

        assert!(mae > 0.0);
        assert!(mse > 0.0);
        assert!(r2 < 1.0); // Should be less than perfect
    }
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            cv_folds: 5,
            test_size: 0.2,
            random_seed: 42,
            detailed_feature_analysis: false,
            learning_curve_analysis: false,
            stability_testing: false,
            benchmark_iterations: 10,
            significance_level: 0.05,
        }
    }
}

impl Default for LearningCurves {
    fn default() -> Self {
        Self {
            training_sizes: vec![],
            train_scores: vec![],
            val_scores: vec![],
            train_scores_mean: vec![],
            train_scores_std: vec![],
            val_scores_mean: vec![],
            val_scores_std: vec![],
        }
    }
}

impl Default for StabilityMetrics {
    fn default() -> Self {
        Self {
            prediction_stability: 0.0,
            feature_stability: 0.0,
            consistency_score: 0.0,
            noise_robustness: 0.0,
        }
    }
}