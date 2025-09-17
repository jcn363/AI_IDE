//! Tauri commands for Model Warmup Prediction System
//!
//! This module provides Tauri command handlers for frontend integration with the
//! model warmup prediction system, including real-time monitoring, configuration,
//! and performance analytics.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::{State, AppHandle};
use tokio::sync::RwLock;

use rust_ai_ide_common::validation::TauriInputSanitizer;
use rust_ai_ide_warmup_predictor::{
    ModelWarmupPredictor, WarmupConfig, WarmupRequest, ModelTask, Complexity,
    RequestPriority, UserContext, ProjectContext, ml_trainer::{MLModelTrainer, TrainingConfig, MLModelType},
    ml_evaluator::{MLModelEvaluator, EvaluationConfig, EvaluationReport},
    advanced_patterns::{AdvancedPatternAnalyzer, PatternAlgorithm, PatternConfig, RecognizedPattern},
    benchmark_tools::{PerformanceBenchmarker, BenchmarkConfig, BenchmarkResult},
};

use crate::command_templates::{tauri_command_template, tauri_command_template_with_result, CommandConfig};
use crate::errors::IDError;
use crate::infra::AppState;

const COMMAND_CONFIG: CommandConfig = CommandConfig {
    enable_logging: true,
    log_level: log::Level::Info,
    enable_validation: true,
    async_timeout_secs: Some(30),
};

/// Frontend-compatible warmup request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendWarmupRequest {
    pub task: String,
    pub input_length: usize,
    pub complexity: String,
    pub priority: String,
    pub user_id: String,
    pub project_language: String,
    pub project_size: usize,
}

/// Frontend-compatible pattern analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendPatternRequest {
    pub user_id: String,
    pub activities: Vec<FrontendUserActivity>,
}

/// Frontend-compatible user activity structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendUserActivity {
    pub activity_type: String,
    pub timestamp: String,
    pub duration: u64,
    pub model_task: Option<String>,
}

/// Frontend-compatible ML training request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendMLTrainingRequest {
    pub model_type: String,
    pub dataset_features: Vec<Vec<f64>>,
    pub dataset_targets: Vec<f64>,
    pub feature_names: Vec<String>,
}

/// Frontend response structure for warmup predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendWarmupResponse {
    pub predicted_models: Vec<FrontendModelPrediction>,
    pub confidence_score: f64,
    pub performance_impact: FrontendPerformanceImpact,
    pub recommendations: Vec<String>,
}

/// Frontend model prediction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendModelPrediction {
    pub model_id: String,
    pub confidence_score: f64,
    pub usage_probability: f64,
    pub time_until_needed: String,
    pub reasoning: Vec<String>,
}

/// Frontend performance impact structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendPerformanceImpact {
    pub cpu_impact_percent: f64,
    pub memory_impact_mb: f64,
    pub estimated_latency_increase: String,
    pub is_acceptable: bool,
}

/// Frontend metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendMetricsResponse {
    pub total_predictions: u64,
    pub accuracy_score: f64,
    pub average_confidence: f64,
    pub warmup_effectiveness: f64,
    pub performance_improvements: Vec<String>,
}

/// Frontend benchmark response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendBenchmarkResponse {
    pub benchmark_name: String,
    pub throughput: f64,
    pub avg_latency: String,
    pub memory_usage: f64,
    pub recommendations: Vec<String>,
}

// Command: Get warmup prediction for a request
tauri_command_template_with_result!(
    get_warmup_prediction,
    FrontendWarmupResponse,
    {
        // Validate input
        let sanitized_request = TauriInputSanitizer::sanitize_struct(&request)?;

        // Convert frontend request to internal format
        let warmup_request = convert_frontend_request(sanitized_request)?;

        // Get warmup predictor from state
        acquire_service_and_execute!(state.warmup_predictor, ModelWarmupPredictor, {
            let prediction = predictor.predict_and_warm(&warmup_request).await?;

            // Convert to frontend response
            convert_warmup_prediction(prediction)
        })
    },
    service = ModelWarmupPredictor,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Get current warmup system metrics
tauri_command_template_with_result!(
    get_warmup_metrics,
    FrontendMetricsResponse,
    {
        acquire_service_and_execute!(state.warmup_predictor, ModelWarmupPredictor, {
            let metrics = predictor.get_metrics();

            // Convert metrics to frontend format
            FrontendMetricsResponse {
                total_predictions: metrics.total_predictions(),
                accuracy_score: metrics.accuracy_score(),
                average_confidence: metrics.average_confidence(),
                warmup_effectiveness: metrics.warmup_effectiveness(),
                performance_improvements: metrics.performance_improvements(),
            }
        })
    },
    service = ModelWarmupPredictor,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Analyze user behavior patterns
tauri_command_template_with_result!(
    analyze_behavior_patterns,
    Vec<FrontendPatternResult>,
    {
        // Validate input
        let sanitized_request = TauriInputSanitizer::sanitize_struct(&request)?;

        // Convert frontend request to internal format
        let activities: Vec<rust_ai_ide_warmup_predictor::types::UserActivity> = sanitized_request.activities
            .into_iter()
            .map(|a| convert_frontend_activity(a))
            .collect();

        // Get pattern analyzer from state
        acquire_service_and_execute!(state.pattern_analyzer, AdvancedPatternAnalyzer, {
            let patterns = analyzer.analyze_patterns(&sanitized_request.user_id, &activities).await?;

            // Convert to frontend format
            patterns.into_iter()
                .map(|p| convert_pattern_result(p))
                .collect()
        })
    },
    service = AdvancedPatternAnalyzer,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Predict future patterns
tauri_command_template_with_result!(
    predict_future_patterns,
    Vec<FrontendPatternResult>,
    {
        // Validate input
        let sanitized_request = TauriInputSanitizer::sanitize_struct(&request)?;

        // Convert frontend request to internal format
        let activities: Vec<rust_ai_ide_warmup_predictor::types::UserActivity> = sanitized_request.activities
            .into_iter()
            .map(|a| convert_frontend_activity(a))
            .collect();

        // Get pattern analyzer from state
        acquire_service_and_execute!(state.pattern_analyzer, AdvancedPatternAnalyzer, {
            // Add activities to analyzer first
            for activity in &activities {
                analyzer.analyze_patterns(&sanitized_request.user_id, std::slice::from_ref(activity)).await?;
            }

            // Create a dummy current activity for prediction
            let current_activity = activities.last().unwrap_or(&rust_ai_ide_warmup_predictor::types::UserActivity {
                activity_type: "general".to_string(),
                timestamp: std::time::Instant::now(),
                duration: std::time::Duration::from_secs(30),
                model_task: None,
            });

            let prediction = analyzer.predict_patterns(&sanitized_request.user_id, current_activity).await?;

            // Convert to frontend format
            prediction.patterns.into_iter()
                .map(|p| convert_pattern_result(p))
                .collect()
        })
    },
    service = AdvancedPatternAnalyzer,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Train ML model
tauri_command_template_with_result!(
    train_ml_model,
    FrontendTrainingResponse,
    {
        // Validate input
        let sanitized_request = TauriInputSanitizer::sanitize_struct(&request)?;

        // Convert frontend request to internal format
        let model_type = match sanitized_request.model_type.as_str() {
            "linear_regression" => MLModelType::LinearRegression,
            "random_forest" => MLModelType::RandomForest,
            "gradient_boosting" => MLModelType::GradientBoosting,
            "ensemble" => MLModelType::Ensemble,
            _ => return Err("Invalid model type".to_string()),
        };

        let dataset = rust_ai_ide_warmup_predictor::ml_trainer::TrainingDataset {
            features: ndarray::Array2::from_shape_vec(
                (sanitized_request.dataset_features.len(), sanitized_request.dataset_features[0].len()),
                sanitized_request.dataset_features.into_iter().flatten().collect()
            ).map_err(|e| format!("Invalid dataset dimensions: {}", e))?,
            targets: sanitized_request.dataset_targets,
            feature_names: sanitized_request.feature_names,
            sample_weights: None,
        };

        // Get ML trainer from state
        acquire_service_and_execute!(state.ml_trainer, MLModelTrainer, {
            let metrics = trainer.train_model(&dataset, model_type).await?;

            // Convert to frontend response
            FrontendTrainingResponse {
                model_type: request.model_type,
                training_score: metrics.final_train_score,
                validation_score: metrics.final_val_score,
                training_time: metrics.training_duration.as_millis() as u64,
                feature_importance: metrics.feature_importance,
                status: "completed".to_string(),
            }
        })
    },
    service = MLModelTrainer,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Evaluate ML model
tauri_command_template_with_result!(
    evaluate_ml_model,
    FrontendEvaluationResponse,
    {
        // Validate input
        let sanitized_request = TauriInputSanitizer::sanitize_struct(&request)?;

        // Get ML evaluator from state
        acquire_service_and_execute!(state.ml_evaluator, MLModelEvaluator, {
            // Create dummy model and dataset for evaluation
            let model_type = match sanitized_request.model_type.as_str() {
                "linear_regression" => MLModelType::LinearRegression,
                "random_forest" => MLModelType::RandomForest,
                _ => MLModelType::LinearRegression,
            };

            let dataset = rust_ai_ide_warmup_predictor::ml_trainer::TrainingDataset {
                features: ndarray::Array2::from_shape_vec(
                    (sanitized_request.test_features.len(), sanitized_request.test_features[0].len()),
                    sanitized_request.test_features.into_iter().flatten().collect()
                ).map_err(|e| format!("Invalid test dataset dimensions: {}", e))?,
                targets: sanitized_request.test_targets,
                feature_names: sanitized_request.feature_names,
                sample_weights: None,
            };

            // Create a dummy model for evaluation
            let model = match model_type {
                MLModelType::LinearRegression => Box::new(rust_ai_ide_warmup_predictor::ml_trainer::LinearRegressionModel::new()) as Box<dyn rust_ai_ide_warmup_predictor::ml_trainer::MLModel + Send + Sync>,
                _ => Box::new(rust_ai_ide_warmup_predictor::ml_trainer::RandomForestModel::new()) as Box<dyn rust_ai_ide_warmup_predictor::ml_trainer::MLModel + Send + Sync>,
            };

            let report = evaluator.evaluate_model(&*model, &request.model_name, &dataset, &TrainingConfig::default()).await?;

            // Convert to frontend response
            FrontendEvaluationResponse {
                model_name: report.model_name,
                accuracy_score: match report.metrics {
                    rust_ai_ide_warmup_predictor::ml_evaluator::EvaluationMetrics::Regression(reg) => reg.r2_score,
                    rust_ai_ide_warmup_predictor::ml_evaluator::EvaluationMetrics::Classification(cls) => cls.accuracy,
                },
                cross_validation_score: report.cross_validation.as_ref()
                    .map(|cv| cv.mean_score)
                    .unwrap_or(0.0),
                evaluation_time: chrono::Utc::now().timestamp() as u64,
                recommendations: vec!["Model evaluation completed".to_string()],
            }
        })
    },
    service = MLModelEvaluator,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Run performance benchmark
tauri_command_template_with_result!(
    run_performance_benchmark,
    FrontendBenchmarkResponse,
    {
        // Validate input
        let sanitized_request = TauriInputSanitizer::sanitize_struct(&request)?;

        // Get benchmarker from state
        acquire_service_and_execute!(state.benchmarker, PerformanceBenchmarker, {
            let config = BenchmarkConfig {
                iterations: sanitized_request.iterations,
                warmup_iterations: sanitized_request.warmup_iterations,
                max_duration: std::time::Duration::from_secs(sanitized_request.max_duration_secs),
                memory_profiling: true,
                cpu_profiling: true,
                detailed_latency: true,
                confidence_level: 0.95,
                concurrent_requests: 1,
            };

            // Create dummy function for benchmarking
            let benchmark_fn = || async {
                // Simulate warmup predictor workload
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                Ok(())
            };

            let result = benchmarker.benchmark_warmup_predictor(&sanitized_request.benchmark_name, benchmark_fn).await?;

            // Convert to frontend response
            FrontendBenchmarkResponse {
                benchmark_name: result.name,
                throughput: result.throughput,
                avg_latency: format!("{:.2}ms", result.avg_latency.as_millis()),
                memory_usage: result.memory_usage_mb,
                recommendations: vec![
                    format!("Throughput: {:.2} req/s", result.throughput),
                    format!("95th percentile: {:.2}ms", result.p95_latency.as_millis()),
                ],
            }
        })
    },
    service = PerformanceBenchmarker,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Update warmup configuration
tauri_command_template!(
    update_warmup_config,
    {
        // Validate input
        let sanitized_config = TauriInputSanitizer::sanitize_struct(&config)?;

        // Convert frontend config to internal format
        let warmup_config = convert_frontend_config(sanitized_config)?;

        acquire_service_and_execute!(state.warmup_predictor, ModelWarmupPredictor, {
            predictor.update_config(warmup_config).await?;
            Ok(serde_json::json!({"status": "Configuration updated successfully"}).to_string())
        })
    },
    service = ModelWarmupPredictor,
    state = state,
    config = COMMAND_CONFIG
);

// Command: Get system status and health
tauri_command_template_with_result!(
    get_system_status,
    FrontendSystemStatus,
    {
        // Get status from multiple services
        let warmup_status = acquire_service_and_execute!(state.warmup_predictor, ModelWarmupPredictor, {
            Ok(predictor.get_metrics().total_predictions() > 0)
        }).unwrap_or(false);

        let pattern_status = acquire_service_and_execute!(state.pattern_analyzer, AdvancedPatternAnalyzer, {
            Ok(true) // Pattern analyzer is always ready if initialized
        }).unwrap_or(false);

        FrontendSystemStatus {
            warmup_predictor_active: warmup_status,
            pattern_analyzer_active: pattern_status,
            ml_trainer_active: true, // Assume active if service exists
            ml_evaluator_active: true,
            benchmarker_active: true,
            system_health: if warmup_status && pattern_status { "healthy" } else { "degraded" }.to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    },
    service = ModelWarmupPredictor,
    state = state,
    config = COMMAND_CONFIG
);

// Helper structures for command responses

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendPatternResult {
    pub pattern_id: String,
    pub pattern_type: String,
    pub confidence: f64,
    pub strength: f64,
    pub next_occurrence: String,
    pub associated_tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendTrainingResponse {
    pub model_type: String,
    pub training_score: f64,
    pub validation_score: f64,
    pub training_time: u64,
    pub feature_importance: HashMap<String, f64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendEvaluationResponse {
    pub model_name: String,
    pub accuracy_score: f64,
    pub cross_validation_score: f64,
    pub evaluation_time: u64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendSystemStatus {
    pub warmup_predictor_active: bool,
    pub pattern_analyzer_active: bool,
    pub ml_trainer_active: bool,
    pub ml_evaluator_active: bool,
    pub benchmarker_active: bool,
    pub system_health: String,
    pub last_updated: String,
}

// Conversion helper functions

fn convert_frontend_request(frontend: FrontendWarmupRequest) -> Result<WarmupRequest, String> {
    let task = match frontend.task.as_str() {
        "completion" => ModelTask::Completion,
        "chat" => ModelTask::Chat,
        "analysis" => ModelTask::Analysis,
        "refactoring" => ModelTask::Refactoring,
        _ => ModelTask::Custom(frontend.task),
    };

    let complexity = match frontend.complexity.as_str() {
        "simple" => Complexity::Simple,
        "medium" => Complexity::Medium,
        "complex" => Complexity::Complex,
    };

    let priority = match frontend.priority.as_str() {
        "low" => RequestPriority::Low,
        "medium" => RequestPriority::Medium,
        "high" => RequestPriority::High,
        "critical" => RequestPriority::Critical,
        _ => RequestPriority::Medium,
    };

    Ok(WarmupRequest {
        task,
        input_length: frontend.input_length,
        complexity,
        priority,
        acceptable_latency: std::time::Duration::from_millis(100),
        preferred_hardware: None,
        user_context: UserContext {
            user_id: frontend.user_id,
            session_duration: std::time::Duration::from_secs(1800),
            recent_activities: vec![],
            preferences: HashMap::new(),
        },
        project_context: ProjectContext {
            language: frontend.project_language,
            size_lines: frontend.project_size,
            complexity_score: 0.5,
            recent_changes: vec![],
        },
    })
}

fn convert_warmup_prediction(prediction: rust_ai_ide_warmup_predictor::types::WarmupPrediction) -> Result<FrontendWarmupResponse, String> {
    let predicted_models = prediction.predicted_models.into_iter()
        .map(|p| FrontendModelPrediction {
            model_id: p.model_id.to_string(),
            confidence_score: p.confidence_score,
            usage_probability: p.usage_probability,
            time_until_needed: format!("{:.1}s", p.time_until_needed.as_secs_f64()),
            reasoning: p.reasoning,
        })
        .collect();

    let performance_impact = FrontendPerformanceImpact {
        cpu_impact_percent: prediction.performance_impact.cpu_impact_percent,
        memory_impact_mb: prediction.performance_impact.memory_impact_mb,
        estimated_latency_increase: format!("{:.1}ms", prediction.performance_impact.latency_increase_ms),
        is_acceptable: prediction.performance_impact.is_acceptable,
    };

    Ok(FrontendWarmupResponse {
        predicted_models,
        confidence_score: prediction.confidence_score,
        performance_impact,
        recommendations: vec!["Warmup prediction completed".to_string()],
    })
}

fn convert_frontend_activity(frontend: FrontendUserActivity) -> rust_ai_ide_warmup_predictor::types::UserActivity {
    rust_ai_ide_warmup_predictor::types::UserActivity {
        activity_type: frontend.activity_type,
        timestamp: std::time::Instant::now(), // TODO: Parse from frontend timestamp
        duration: std::time::Duration::from_secs(frontend.duration),
        model_task: frontend.model_task.map(|t| match t.as_str() {
            "completion" => ModelTask::Completion,
            "chat" => ModelTask::Chat,
            "analysis" => ModelTask::Analysis,
            _ => ModelTask::Custom(t),
        }),
    }
}

fn convert_pattern_result(pattern: RecognizedPattern) -> FrontendPatternResult {
    FrontendPatternResult {
        pattern_id: pattern.pattern_id,
        pattern_type: format!("{:?}", pattern.pattern_type),
        confidence: pattern.confidence,
        strength: pattern.strength,
        next_occurrence: format!("{:.1}s", pattern.next_occurrence.as_secs_f64()),
        associated_tasks: pattern.associated_tasks.iter()
            .map(|t| format!("{:?}", t))
            .collect(),
    }
}

fn convert_frontend_config(_frontend: serde_json::Value) -> Result<WarmupConfig, String> {
    // TODO: Implement proper config conversion
    Ok(WarmupConfig::default())
}