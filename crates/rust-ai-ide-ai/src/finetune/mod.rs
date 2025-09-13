//! Fine-tuning pipeline for AI/ML enhancements
//!
//! This module provides comprehensive fine-tuning capabilities for CodeLlama and StarCoder
//! models, specifically optimized for Rust development tasks.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
pub use config::TrainingConfig;
/// Re-export key fine-tuning types for convenient access
pub use dataset::DatasetBuilder;
pub use orchestrator::TrainingOrchestrator;
use serde::{Deserialize, Serialize};

/// Fine-tuning job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FineTuneJob {
    pub id:            String,
    pub name:          String,
    pub description:   Option<String>,
    pub base_model:    String,
    pub model_type:    ModelType,
    pub dataset_path:  PathBuf,
    pub config:        TrainingConfig,
    pub status:        TrainingStatus,
    pub progress:      TrainingProgress,
    pub created_at:    chrono::DateTime<chrono::Utc>,
    pub updated_at:    chrono::DateTime<chrono::Utc>,
    pub output_path:   Option<PathBuf>,
    pub metrics:       Option<TrainingMetrics>,
    pub error_message: Option<String>,
}

/// Model types supported for fine-tuning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelType {
    CodeLlama,
    StarCoder,
}

/// Training status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrainingStatus {
    Created,
    Initializing,
    PreparingData,
    Training,
    Evaluating,
    Saving,
    Completed,
    Failed,
    Cancelled,
}

/// Training progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingProgress {
    pub epoch:                    usize,
    pub total_epochs:             usize,
    pub step:                     usize,
    pub total_steps:              usize,
    pub loss:                     Option<f32>,
    pub learning_rate:            Option<f32>,
    pub estimated_time_remaining: Option<std::time::Duration>,
    pub memory_usage_mb:          Option<u64>,
    pub gpu_utilization:          Option<f32>,
}

/// Training metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingMetrics {
    pub final_loss:            f32,
    pub training_time_seconds: u64,
    pub peak_memory_usage_mb:  u64,
    pub samples_per_second:    f32,
    pub validation_loss:       Option<f32>,
    pub perplexity:            Option<f32>,
    pub bleu_score:            Option<f32>,
    pub code_bleu_score:       Option<f32>,
}

pub mod config;
/// Re-exports from submodules
pub mod dataset;
pub mod orchestrator;

/// Common constants and utilities for fine-tuning
pub mod constants {
    pub const DEFAULT_LEARNING_RATE: f32 = 2e-5;
    pub const DEFAULT_BATCH_SIZE: usize = 8;
    pub const DEFAULT_MAX_EPOCHS: usize = 3;
    pub const DEFAULT_WARMUP_RATIO: f32 = 0.1;
    pub const DEFAULT_WEIGHT_DECAY: f32 = 0.01;
    pub const DEFAULT_MAX_GRAD_NORM: f32 = 1.0;
    pub const DEFAULT_SAVE_STEPS: usize = 500;
    pub const DEFAULT_EVAL_STEPS: usize = 500;
    pub const DEFAULT_LOGGING_STEPS: usize = 100;

    // Model-specific parameters
    pub const CODELLAMA_CONTEXT_LENGTH: usize = 16384;
    pub const STARCODER_CONTEXT_LENGTH: usize = 8192;
    pub const CODELLAMA_VOCAB_SIZE: usize = 32016;
    pub const STARCODER_VOCAB_SIZE: usize = 49152;
}

/// Quality validation for fine-tuning results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelValidationResult {
    pub passed:            bool,
    pub score:             f32,
    pub issues:            Vec<String>,
    pub recommendations:   Vec<String>,
    pub benchmark_results: Option<BenchmarkResults>,
}

/// Benchmark results for model evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResults {
    pub humaneval_pass_at_1:     Option<f32>,
    pub humaneval_pass_at_10:    Option<f32>,
    pub rust_specific_pass_rate: Option<f32>,
    pub completion_accuracy:     Option<f32>,
    pub response_time_ms:        Option<f32>,
    pub memory_efficiency:       Option<f32>,
}

/// Training event types for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TrainingEvent {
    JobStarted {
        job_id:    String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    JobProgress {
        job_id:    String,
        progress:  TrainingProgress,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    JobCompleted {
        job_id:    String,
        metrics:   TrainingMetrics,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    JobFailed {
        job_id:    String,
        error:     String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    CheckpointSaved {
        job_id:    String,
        step:      usize,
        path:      PathBuf,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    EvaluationCompleted {
        job_id:    String,
        metrics:   HashMap<String, f32>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Fine-tuning pipeline interface
#[async_trait::async_trait]
pub trait TrainingPipeline {
    /// Initialize the training pipeline
    async fn initialize(&mut self) -> Result<()>;

    /// Start training with the given configuration
    async fn start_training(&mut self, job: FineTuneJob) -> Result<String>;

    /// Get training progress
    async fn get_progress(&self, job_id: &str) -> Result<TrainingProgress>;

    /// Cancel training
    async fn cancel_training(&self, job_id: &str) -> Result<()>;

    /// Get final metrics
    async fn get_metrics(&self, job_id: &str) -> Result<TrainingMetrics>;

    /// Validate trained model
    async fn validate_model(&self, model_path: &PathBuf) -> Result<ModelValidationResult>;

    /// Cleanup resources
    async fn cleanup(&self, job_id: &str) -> Result<()>;
}

/// Create a default fine-tuning configuration for CodeLlama
pub fn default_codellama_config() -> TrainingConfig {
    TrainingConfig {
        learning_rate:               constants::DEFAULT_LEARNING_RATE,
        batch_size:                  constants::DEFAULT_BATCH_SIZE,
        max_epochs:                  constants::DEFAULT_MAX_EPOCHS,
        warmup_ratio:                constants::DEFAULT_WARMUP_RATIO,
        weight_decay:                constants::DEFAULT_WEIGHT_DECAY,
        max_grad_norm:               constants::DEFAULT_MAX_GRAD_NORM,
        save_steps:                  constants::DEFAULT_SAVE_STEPS,
        eval_steps:                  constants::DEFAULT_EVAL_STEPS,
        logging_steps:               constants::DEFAULT_LOGGING_STEPS,
        gradient_accumulation_steps: 1,
        max_seq_length:              constants::CODELLAMA_CONTEXT_LENGTH,
        lora_rank:                   Some(8),
        lora_alpha:                  Some(16.0),
        lora_dropout:                Some(0.1),
        quantization_config:         None,
        dataloader_num_workers:      4,
        dataloader_pin_memory:       true,
        mixed_precision:             Some(MixedPrecision::Fp16),
        gradient_checkpointing:      false,
        early_stopping_patience:     Some(3),
        label_smoothing:             Some(0.1),
    }
}

/// Create a default fine-tuning configuration for StarCoder
pub fn default_starcoder_config() -> TrainingConfig {
    TrainingConfig {
        learning_rate:               constants::DEFAULT_LEARNING_RATE,
        batch_size:                  constants::DEFAULT_BATCH_SIZE,
        max_epochs:                  constants::DEFAULT_MAX_EPOCHS,
        warmup_ratio:                constants::DEFAULT_WARMUP_RATIO,
        weight_decay:                constants::DEFAULT_WEIGHT_DECAY,
        max_grad_norm:               constants::DEFAULT_MAX_GRAD_NORM,
        save_steps:                  constants::DEFAULT_SAVE_STEPS,
        eval_steps:                  constants::DEFAULT_EVAL_STEPS,
        logging_steps:               constants::DEFAULT_LOGGING_STEPS,
        gradient_accumulation_steps: 1,
        max_seq_length:              constants::STARCODER_CONTEXT_LENGTH,
        lora_rank:                   Some(8),
        lora_alpha:                  Some(16.0),
        lora_dropout:                Some(0.1),
        quantization_config:         None,
        dataloader_num_workers:      4,
        dataloader_pin_memory:       true,
        mixed_precision:             Some(MixedPrecision::Fp16),
        gradient_checkpointing:      false,
        early_stopping_patience:     Some(3),
        label_smoothing:             Some(0.1),
    }
}

// Re-export from config module
pub use config::{MixedPrecision, QuantizationConfig};
