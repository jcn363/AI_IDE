//! Training configuration and hyperparameters for fine-tuning
//!
//! This module defines comprehensive configuration options for fine-tuning
//! CodeLlama and StarCoder models, including hyperparameters, optimization settings,
//! and model-specific configurations.

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

/// Training configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TrainingConfig {
    /// Learning hyperparameters
    #[validate(range(min = 0.0, max = 1.0))]
    pub learning_rate: f32,

    #[validate(range(min = 1, max = 1024))]
    pub batch_size: usize,

    #[validate(range(min = 1, max = 100))]
    pub max_epochs: usize,

    /// Learning rate scheduling
    #[validate(range(min = 0.0, max = 1.0))]
    pub warmup_ratio: f32,

    #[validate(range(min = 0.0))]
    pub weight_decay: f32,

    #[validate(range(min = 0.0))]
    pub max_grad_norm: f32,

    /// Training progress checkpoints
    #[validate(range(min = 1))]
    pub save_steps: usize,

    #[validate(range(min = 1))]
    pub eval_steps: usize,

    #[validate(range(min = 1))]
    pub logging_steps: usize,

    /// Gradient accumulation
    #[validate(range(min = 1, max = 32))]
    pub gradient_accumulation_steps: usize,

    /// Model architecture parameters
    pub max_seq_length: usize,

    /// LoRA parameters (optional)
    pub lora_rank: Option<usize>,

    pub lora_alpha: Option<f32>,

    pub lora_dropout: Option<f32>,

    /// Quantization configuration
    pub quantization_config: Option<QuantizationConfig>,

    /// Data loading configuration
    #[validate(range(min = 1, max = 16))]
    pub dataloader_num_workers: usize,

    pub dataloader_pin_memory: bool,

    /// Mixed precision training
    pub mixed_precision: Option<MixedPrecision>,

    /// Advanced optimization techniques
    pub gradient_checkpointing: bool,

    pub early_stopping_patience: Option<usize>,

    /// Label smoothing
    pub label_smoothing: Option<f32>,

    /// Distributed training configuration
    pub distributed_config: Option<DistributedConfig>,

    /// Model-specific optimizations
    pub model_specific_config: Option<ModelSpecificConfig>,

    /// Evaluation and validation
    pub evaluation_config: Option<EvaluationConfig>,

    /// Custom training hooks and callbacks
    pub training_hooks: Vec<TrainingHook>,
}

/// Mixed precision training options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MixedPrecision {
    Fp16,
    Bf16,
    Int8,
}

/// Quantization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    pub quantization_type: QuantizationType,

    /// Quantization bits
    pub bits: u8,

    /// Symmetric vs asymmetric quantization
    pub symmetric: bool,

    /// Quantization group size
    pub group_size: Option<usize>,

    /// Channels to quantize (by default, all linear layers)
    pub quantize_channels: Option<Vec<String>>,

    /// Skip quantization for these layers
    pub skip_quantize: Vec<String>,
}

/// Quantization types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizationType {
    /// 4-bit quantization (GPTQ style)
    Gptq,

    /// AWQ quantization
    Awq,

    /// Standard quantization
    Standard,

    /// No quantization (for mixed precision)
    None,
}

/// Distributed training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Number of processes/devices
    pub world_size: usize,

    /// Current process rank
    pub rank: usize,

    /// Master node address
    pub master_addr: String,

    /// Master node port
    pub master_port: u16,

    /// Backend (NCCL, Gloo, etc.)
    pub backend: String,

    /// Data parallelism configuration
    pub data_parallelism: DataParallelismConfig,

    /// Tensor parallelism configuration
    pub tensor_parallelism: TensorParallelismConfig,
}

/// Data parallelism configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataParallelismConfig {
    pub enabled: bool,

    /// Gradient accumulation steps across devices
    pub accumulation_steps: usize,
}

/// Tensor parallelism configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorParallelismConfig {
    pub enabled: bool,

    /// Number of tensor parallel groups
    pub tensor_parallel_size: usize,
}

/// Model-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpecificConfig {
    /// CodeLlama-specific settings
    pub codellama: Option<CodeLlamaConfig>,

    /// StarCoder-specific settings
    pub starcoder: Option<StarCoderConfig>,

    /// Custom model configuration
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// CodeLlama-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLlamaConfig {
    /// Instruction template for fine-tuning
    pub instruction_template: Option<String>,

    /// System prompt for code generation
    pub system_prompt: Option<String>,

    /// Special tokens for code completion
    pub completion_tokens: Option<CodeCompletionTokens>,

    /// Code quality filters
    pub quality_filters: Option<CodeQualityFilters>,
}

/// StarCoder-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarCoderConfig {
    /// FIM (Fill-in-the-Middle) configuration
    pub fim_config: Option<FimConfig>,

    /// Multi-query attention settings
    pub multi_query_attention: bool,

    /// Context length adaptation
    pub context_adaptation: Option<ContextAdaptation>,
}

/// Code completion tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompletionTokens {
    pub prefix_token:     String,
    pub suffix_token:     String,
    pub middle_token:     String,
    pub completion_token: String,
}

/// Code quality filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityFilters {
    pub min_line_length:    usize,
    pub max_line_length:    usize,
    pub allowed_extensions: Vec<String>,
    pub block_patterns:     Vec<String>,
    pub required_patterns:  Vec<String>,
}

/// Fill-in-the-Middle configuration for StarCoder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FimConfig {
    /// FIM enabled
    pub enabled: bool,

    /// FIM prefix token
    pub prefix_token: String,

    /// FIM suffix token
    pub suffix_token: String,

    /// FIM middle token
    pub middle_token: String,

    /// FIM probability (0.0 to 1.0)
    pub probability: f32,

    /// Maximum prefix/suffix length
    pub max_prefix_suffix_length: usize,
}

/// Context adaptation for long contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAdaptation {
    /// Rope theta scaling factor
    pub rope_theta_scale: f32,

    /// Position interpolation factor
    pub position_interpolation_factor: f32,
}

/// Evaluation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    /// Evaluation metrics to compute
    pub metrics: Vec<EvaluationMetric>,

    /// Evaluation dataset path
    pub eval_dataset_path: Option<String>,

    /// Evaluation batch size
    pub eval_batch_size: usize,

    /// Evaluation frequency (in steps)
    pub eval_frequency: usize,

    /// Early stopping metric
    pub early_stopping_metric: Option<String>,

    /// Metric thresholds for early stopping
    pub metric_thresholds: HashMap<String, f32>,

    /// Save best checkpoints
    pub save_best_checkpoint: bool,
}

/// Evaluation metrics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationMetric {
    Perplexity,
    Loss,
    Accuracy,
    BleuScore,
    CodeBleu,
    PassAtK,
    RougeScore,
    BertScore,
    Custom(String),
}

/// Training hooks for custom behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHook {
    pub hook_type:  HookType,
    pub name:       String,
    pub enabled:    bool,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Hook types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    PreTraining,
    PostTraining,
    OnEpochStart,
    OnEpochEnd,
    OnStepStart,
    OnStepEnd,
    OnEvaluation,
    OnSaveCheckpoint,
    OnLoadCheckpoint,
    OnEarlyStopping,
}

/// Learning rate scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRateScheduler {
    pub scheduler_type: SchedulerType,

    /// Learning rate parameters
    pub warmup_steps: Option<usize>,

    pub decay_steps: Option<usize>,

    pub decay_factor: Option<f32>,

    pub min_learning_rate: Option<f32>,

    pub cycle_length: Option<usize>,

    pub cycle_mult: Option<f32>,
}

/// Scheduler types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerType {
    Linear,
    Cosine,
    CosineWithRestarts,
    Polynomial,
    Constant,
    ConstantWithWarmup,
}

/// Optimizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    pub optimizer_type: OptimizerType,

    /// Adam/AdamW parameters
    pub beta1: f32,

    pub beta2: f32,

    pub epsilon: f32,

    /// LAMB/Lion parameters
    pub trust_ratio_clipping: Option<f32>,

    /// SGD parameters
    pub momentum: Option<f32>,

    pub nesterov: Option<bool>,

    /// Custom optimizer parameters
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
}

/// Optimizer types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizerType {
    Adam,
    AdamW,
    AdamW8bit,
    Lamb,
    Lion,
    SGD,
    Custom(String),
}

/// Model configuration validation
impl TrainingConfig {
    /// Validate the configuration
    pub fn validate_comprehensive(&self) -> Result<(), ValidationError> {
        // Base validation
        self.validate()?;

        // Custom validation logic

        // Check learning rate is reasonable
        if self.learning_rate < 1e-7 || self.learning_rate > 1e-2 {
            return Err(ValidationError::new(
                "learning rate out of reasonable range",
            ));
        }

        // Check batch size is power of 2 for efficiency
        if self.batch_size & (self.batch_size - 1) != 0 {
            log::warn!(
                "Batch size {} is not a power of 2, may impact performance",
                self.batch_size
            );
        }

        // Check LoRA parameters if provided
        if let Some(rank) = self.lora_rank {
            if let Some(alpha) = self.lora_alpha {
                if alpha / rank as f32 > 2.0 {
                    log::warn!(
                        "LoRA alpha/rank ratio ({}) is quite high, may lead to instability",
                        alpha / rank as f32
                    );
                }
            }
        }

        // Check gradient accumulation doesn't create too large effective batch size
        let effective_batch_size = self.batch_size * self.gradient_accumulation_steps;
        if effective_batch_size > 1024 {
            log::warn!(
                "Effective batch size {} is very large, may cause memory issues",
                effective_batch_size
            );
        }

        // Validate distributed configuration
        if let Some(dist_config) = &self.distributed_config {
            if dist_config.rank >= dist_config.world_size {
                return Err(ValidationError::new("rank must be less than world_size"));
            }
        }

        Ok(())
    }

    /// Get recommended configuration based on hardware
    pub fn get_recommended_for_hardware(&self, hardware: HardwareProfile) -> Result<TrainingConfig> {
        let mut config = self.clone();

        match hardware.gpu_memory_gb {
            0..=8 => {
                config.batch_size = config.batch_size.min(4);
                config.max_seq_length = config.max_seq_length.min(1024);
                config.mixed_precision = Some(MixedPrecision::Fp16);
                config.gradient_checkpointing = true;
            }
            9..=16 => {
                config.batch_size = config.batch_size.min(8);
                config.max_seq_length = config.max_seq_length.min(2048);
                config.mixed_precision = Some(MixedPrecision::Fp16);
            }
            17..=32 => {
                config.batch_size = config.batch_size.min(16);
                config.mixed_precision = Some(MixedPrecision::Bf16);
            }
            _ => {
                config.batch_size = config.batch_size.min(32);
                config.mixed_precision = Some(MixedPrecision::Bf16);
            }
        }

        Ok(config)
    }

    /// Optimize configuration based on dataset characteristics
    pub fn optimize_for_dataset(&self, dataset_stats: &DatasetStats) -> Result<TrainingConfig> {
        let mut config = self.clone();

        // Adjust based on dataset size
        if dataset_stats.sample_count < 1000 {
            config.max_epochs = config.max_epochs.min(5);
            config.learning_rate *= 0.1; // Reduce learning rate for small datasets
        } else if dataset_stats.sample_count > 100000 {
            config.max_epochs = config.max_epochs.max(2);
        }

        // Adjust for average sequence length
        if dataset_stats.avg_sequence_length > config.max_seq_length as f64 * 0.8 {
            config.max_seq_length = (dataset_stats.avg_sequence_length * 1.1) as usize;
        }

        Ok(config)
    }

    /// Get estimated training time
    pub fn estimate_training_time(
        &self,
        dataset_stats: &DatasetStats,
        hardware: &HardwareProfile,
    ) -> TrainingTimeEstimate {
        let samples_per_step = self.batch_size * self.gradient_accumulation_steps;
        let total_steps = (dataset_stats.sample_count as usize * self.max_epochs) / samples_per_step;
        let steps_per_hour = hardware.estimated_performance_steps_per_hour;

        let estimated_hours = total_steps as f64 / steps_per_hour;

        TrainingTimeEstimate {
            total_steps,
            steps_per_hour,
            estimated_hours,
            estimated_days: estimated_hours / 24.0,
            memory_requirement_gb: self.estimate_memory_requirement(hardware),
        }
    }

    /// Estimate memory requirement in GB
    pub fn estimate_memory_requirement(&self, hardware: &HardwareProfile) -> f64 {
        let model_size_gb = match hardware.gpu_memory_gb {
            0..=8 => 7.0,   // Small models
            9..=16 => 14.0, // Medium models
            _ => 30.0,      // Large models
        };

        let batch_memory = model_size_gb * (self.batch_size as f64 / 8.0); // Rough estimate
        let gradient_memory = batch_memory * 2.0; // For backward pass
        let optimizer_memory = gradient_memory * 1.5; // Optimizer state

        optimizer_memory
            * if hardware.has_mixed_precision {
                0.6
            } else {
                1.0
            }
    }

    /// Get optimal batch size for hardware
    pub fn get_optimal_batch_size(&self, hardware: &HardwareProfile) -> usize {
        let memory_per_sample_gb = self.estimate_memory_requirement(hardware) / self.batch_size as f64;

        match hardware.gpu_memory_gb as f64 / memory_per_sample_gb {
            x if x >= 32.0 => 32,
            x if x >= 16.0 => 16,
            x if x >= 8.0 => 8,
            x if x >= 4.0 => 4,
            x if x >= 2.0 => 2,
            _ => 1,
        }
    }
}

/// Hardware profile for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub gpu_count: usize,
    pub gpu_memory_gb: usize,
    pub cpu_cores: usize,
    pub system_memory_gb: usize,
    pub has_tensor_cores: bool,
    pub has_mixed_precision: bool,
    pub estimated_performance_steps_per_hour: f64,
}

/// Dataset statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetStats {
    pub sample_count:          u64,
    pub avg_sequence_length:   f64,
    pub max_sequence_length:   usize,
    pub vocabulary_size:       usize,
    pub language_distribution: HashMap<String, f64>,
    pub quality_scores:        Vec<f32>,
}

/// Training time estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingTimeEstimate {
    pub total_steps:           usize,
    pub steps_per_hour:        f64,
    pub estimated_hours:       f64,
    pub estimated_days:        f64,
    pub memory_requirement_gb: f64,
}

/// Configuration presets for different use cases
pub mod presets {
    use super::*;

    /// Configuration optimized for code completion
    pub fn code_completion() -> TrainingConfig {
        TrainingConfig {
            learning_rate:               5e-5,
            batch_size:                  8,
            max_epochs:                  5,
            warmup_ratio:                0.1,
            weight_decay:                0.01,
            max_grad_norm:               1.0,
            save_steps:                  500,
            eval_steps:                  500,
            logging_steps:               100,
            gradient_accumulation_steps: 4,
            max_seq_length:              2048,
            lora_rank:                   Some(8),
            lora_alpha:                  Some(16.0),
            lora_dropout:                Some(0.05),
            quantization_config:         None,
            dataloader_num_workers:      4,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Fp16),
            gradient_checkpointing:      true,
            early_stopping_patience:     Some(3),
            label_smoothing:             None,
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        }
    }

    /// Configuration optimized for error fixing
    pub fn error_fixing() -> TrainingConfig {
        TrainingConfig {
            learning_rate:               2e-5,
            batch_size:                  4,
            max_epochs:                  10,
            warmup_ratio:                0.05,
            weight_decay:                0.01,
            max_grad_norm:               1.0,
            save_steps:                  1000,
            eval_steps:                  500,
            logging_steps:               50,
            gradient_accumulation_steps: 8,
            max_seq_length:              4096,
            lora_rank:                   Some(16),
            lora_alpha:                  Some(32.0),
            lora_dropout:                Some(0.1),
            quantization_config:         None,
            dataloader_num_workers:      2,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Bf16),
            gradient_checkpointing:      true,
            early_stopping_patience:     Some(5),
            label_smoothing:             Some(0.1),
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        }
    }

    /// Configuration optimized for documentation generation
    pub fn documentation_generation() -> TrainingConfig {
        TrainingConfig {
            learning_rate:               8e-5,
            batch_size:                  12,
            max_epochs:                  3,
            warmup_ratio:                0.15,
            weight_decay:                0.05,
            max_grad_norm:               1.0,
            save_steps:                  200,
            eval_steps:                  200,
            logging_steps:               50,
            gradient_accumulation_steps: 2,
            max_seq_length:              1024,
            lora_rank:                   Some(8),
            lora_alpha:                  Some(16.0),
            lora_dropout:                Some(0.1),
            quantization_config:         None,
            dataloader_num_workers:      6,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Fp16),
            gradient_checkpointing:      false,
            early_stopping_patience:     Some(2),
            label_smoothing:             None,
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        }
    }

    /// Get preset by task type
    pub fn get_preset_by_task(task_type: &str) -> TrainingConfig {
        match task_type.to_lowercase().as_str() {
            "completion" | "code_completion" => code_completion(),
            "error_fixing" | "error_correction" => error_fixing(),
            "documentation" | "doc_generation" => documentation_generation(),
            _ => {
                log::warn!(
                    "Unknown task type '{}', using code completion preset",
                    task_type
                );
                code_completion()
            }
        }
    }
}

/// Training configuration utilities
pub mod utils {
    use super::*;

    /// Load configuration from JSON file
    pub fn load_config_from_file(path: &std::path::Path) -> Result<TrainingConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: TrainingConfig = serde_json::from_str(&content)?;
        config.validate_comprehensive()?;
        Ok(config)
    }

    /// Save configuration to JSON file
    pub fn save_config_to_file(config: &TrainingConfig, path: &std::path::Path) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Merge two configurations (config2 values override config1)
    pub fn merge_configs(config1: TrainingConfig, config2: TrainingConfig) -> TrainingConfig {
        TrainingConfig {
            learning_rate:               config2.learning_rate,
            batch_size:                  config2.batch_size,
            max_epochs:                  config2.max_epochs,
            warmup_ratio:                config2.warmup_ratio,
            weight_decay:                config2.weight_decay,
            max_grad_norm:               config2.max_grad_norm,
            save_steps:                  config2.save_steps,
            eval_steps:                  config2.eval_steps,
            logging_steps:               config2.logging_steps,
            gradient_accumulation_steps: config2.gradient_accumulation_steps,
            max_seq_length:              config2.max_seq_length,
            lora_rank:                   config2.lora_rank.or(config1.lora_rank),
            lora_alpha:                  config2.lora_alpha.or(config1.lora_alpha),
            lora_dropout:                config2.lora_dropout.or(config1.lora_dropout),
            quantization_config:         config2.quantization_config.or(config1.quantization_config),
            dataloader_num_workers:      config2.dataloader_num_workers,
            dataloader_pin_memory:       config2.dataloader_pin_memory,
            mixed_precision:             config2.mixed_precision.or(config1.mixed_precision),
            gradient_checkpointing:      config2.gradient_checkpointing,
            early_stopping_patience:     config2
                .early_stopping_patience
                .or(config1.early_stopping_patience),
            label_smoothing:             config2.label_smoothing.or(config1.label_smoothing),
            distributed_config:          config2.distributed_config.or(config1.distributed_config),
            model_specific_config:       config2
                .model_specific_config
                .or(config1.model_specific_config),
            evaluation_config:           config2.evaluation_config.or(config1.evaluation_config),
            training_hooks:              config2.training_hooks,
        }
    }

    /// Validate configuration for a specific model type
    pub fn validate_for_model(config: &TrainingConfig, model_type: &crate::finetune::ModelType) -> Result<()> {
        match model_type {
            crate::finetune::ModelType::CodeLlama =>
                if config.max_seq_length > 16384 {
                    return Err(anyhow::anyhow!(
                        "CodeLlama context length cannot exceed 16384"
                    ));
                },
            crate::finetune::ModelType::StarCoder =>
                if config.max_seq_length > 8192 {
                    return Err(anyhow::anyhow!(
                        "StarCoder context length cannot exceed 8192"
                    ));
                },
        }

        Ok(())
    }

    /// Get safe default configuration
    pub fn safe_defaults() -> TrainingConfig {
        TrainingConfig {
            learning_rate:               5e-5,
            batch_size:                  4,
            max_epochs:                  3,
            warmup_ratio:                0.1,
            weight_decay:                0.01,
            max_grad_norm:               1.0,
            save_steps:                  500,
            eval_steps:                  500,
            logging_steps:               100,
            gradient_accumulation_steps: 4,
            max_seq_length:              2048,
            lora_rank:                   Some(8),
            lora_alpha:                  Some(16.0),
            lora_dropout:                Some(0.1),
            quantization_config:         None,
            dataloader_num_workers:      2,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Fp16),
            gradient_checkpointing:      true,
            early_stopping_patience:     Some(3),
            label_smoothing:             None,
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        }
    }
}

/// Configuration templates for different scenarios
pub mod templates {
    use super::*;

    /// Template for resource-constrained environments
    pub fn low_resource() -> TrainingConfig {
        TrainingConfig {
            learning_rate:               1e-4,
            batch_size:                  1,
            max_epochs:                  5,
            warmup_ratio:                0.2,
            weight_decay:                0.05,
            max_grad_norm:               1.0,
            save_steps:                  200,
            eval_steps:                  200,
            logging_steps:               50,
            gradient_accumulation_steps: 16,
            max_seq_length:              512,
            lora_rank:                   Some(4),
            lora_alpha:                  Some(8.0),
            lora_dropout:                Some(0.1),
            quantization_config:         Some(QuantizationConfig {
                quantization_type: QuantizationType::Int8,
                bits:              8,
                symmetric:         true,
                group_size:        None,
                quantize_channels: None,
                skip_quantize:     vec!["lm_head".to_string()],
            }),
            dataloader_num_workers:      1,
            dataloader_pin_memory:       false,
            mixed_precision:             Some(MixedPrecision::Fp16),
            gradient_checkpointing:      true,
            early_stopping_patience:     Some(2),
            label_smoothing:             None,
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        }
    }

    /// Template for high-performance computing environments
    pub fn high_performance() -> TrainingConfig {
        TrainingConfig {
            learning_rate:               1e-5,
            batch_size:                  32,
            max_epochs:                  10,
            warmup_ratio:                0.05,
            weight_decay:                0.001,
            max_grad_norm:               0.5,
            save_steps:                  1000,
            eval_steps:                  500,
            logging_steps:               10,
            gradient_accumulation_steps: 1,
            max_seq_length:              4096,
            lora_rank:                   Some(64),
            lora_alpha:                  Some(128.0),
            lora_dropout:                Some(0.05),
            quantization_config:         None,
            dataloader_num_workers:      8,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Bf16),
            gradient_checkpointing:      false,
            early_stopping_patience:     Some(5),
            label_smoothing:             Some(0.1),
            distributed_config:          Some(DistributedConfig {
                world_size:         4,
                rank:               0,
                master_addr:        "127.0.0.1".to_string(),
                master_port:        12345,
                backend:            "nccl".to_string(),
                data_parallelism:   DataParallelismConfig {
                    enabled:            true,
                    accumulation_steps: 1,
                },
                tensor_parallelism: TensorParallelismConfig {
                    enabled:              true,
                    tensor_parallel_size: 2,
                },
            }),
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        }
    }

    /// Experimental template for advanced techniques
    pub fn experimental() -> TrainingConfig {
        TrainingConfig {
            learning_rate:               2e-5,
            batch_size:                  4,
            max_epochs:                  15,
            warmup_ratio:                0.1,
            weight_decay:                0.001,
            max_grad_norm:               0.1,
            save_steps:                  250,
            eval_steps:                  125,
            logging_steps:               25,
            gradient_accumulation_steps: 8,
            max_seq_length:              8192,
            lora_rank:                   Some(128),
            lora_alpha:                  Some(256.0),
            lora_dropout:                Some(0.0),
            quantization_config:         Some(QuantizationConfig {
                quantization_type: QuantizationType::Gptq,
                bits:              4,
                symmetric:         false,
                group_size:        Some(128),
                quantize_channels: None,
                skip_quantize:     vec!["lm_head".to_string(), "embed_tokens".to_string()],
            }),
            dataloader_num_workers:      4,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Bf16),
            gradient_checkpointing:      true,
            early_stopping_patience:     None,
            label_smoothing:             Some(0.2),
            distributed_config:          None,
            model_specific_config:       Some(ModelSpecificConfig {
                codellama: None,
                starcoder: Some(StarCoderConfig {
                    fim_config:            Some(FimConfig {
                        enabled:                  true,
                        prefix_token:             "<prefix>".to_string(),
                        suffix_token:             "<suffix>".to_string(),
                        middle_token:             "<fim-middle><middle>".to_string(),
                        probability:              0.5,
                        max_prefix_suffix_length: 512,
                    }),
                    multi_query_attention: true,
                    context_adaptation:    Some(ContextAdaptation {
                        rope_theta_scale:              2.0,
                        position_interpolation_factor: 1.5,
                    }),
                }),
                custom:    None,
            }),
            evaluation_config:           Some(EvaluationConfig {
                metrics:               vec![
                    EvaluationMetric::Perplexity,
                    EvaluationMetric::PassAtK,
                    EvaluationMetric::CodeBleu,
                ],
                eval_dataset_path:     None,
                eval_batch_size:       4,
                eval_frequency:        100,
                early_stopping_metric: Some("perplexity".to_string()),
                metric_thresholds:     HashMap::from([
                    ("perplexity".to_string(), 15.0),
                    ("code_bleu".to_string(), 0.6),
                ]),
                save_best_checkpoint:  true,
            }),
            training_hooks:              vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = TrainingConfig {
            learning_rate:               5e-5,
            batch_size:                  8,
            max_epochs:                  3,
            warmup_ratio:                0.1,
            weight_decay:                0.01,
            max_grad_norm:               1.0,
            save_steps:                  500,
            eval_steps:                  500,
            logging_steps:               100,
            gradient_accumulation_steps: 4,
            max_seq_length:              2048,
            lora_rank:                   Some(8),
            lora_alpha:                  Some(16.0),
            lora_dropout:                Some(0.1),
            quantization_config:         None,
            dataloader_num_workers:      4,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Fp16),
            gradient_checkpointing:      true,
            early_stopping_patience:     Some(3),
            label_smoothing:             None,
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        };

        assert!(config.validate_comprehensive().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = utils::safe_defaults();
        config.learning_rate = 0.1; // Too high

        assert!(config.validate_comprehensive().is_err());
    }

    #[test]
    fn test_config_presets() {
        let completion_config = presets::code_completion();
        let fixing_config = presets::error_fixing();
        let doc_config = presets::documentation_generation();

        assert!(completion_config.validate_comprehensive().is_ok());
        assert!(fixing_config.validate_comprehensive().is_ok());
        assert!(doc_config.validate_comprehensive().is_ok());

        // Test that configurations are appropriately different
        assert_ne!(completion_config.learning_rate, fixing_config.learning_rate);
        assert_ne!(completion_config.max_epochs, doc_config.max_epochs);
    }

    #[test]
    fn test_config_merge() {
        let config1 = TrainingConfig {
            learning_rate:               1e-4,
            batch_size:                  4,
            max_epochs:                  3,
            warmup_ratio:                0.1,
            weight_decay:                0.01,
            max_grad_norm:               1.0,
            save_steps:                  500,
            eval_steps:                  500,
            logging_steps:               100,
            gradient_accumulation_steps: 1,
            max_seq_length:              1024,
            lora_rank:                   Some(4),
            lora_alpha:                  Some(8.0),
            lora_dropout:                Some(0.1),
            quantization_config:         None,
            dataloader_num_workers:      2,
            dataloader_pin_memory:       false,
            mixed_precision:             Some(MixedPrecision::Fp16),
            gradient_checkpointing:      false,
            early_stopping_patience:     Some(3),
            label_smoothing:             None,
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        };

        let config2 = TrainingConfig {
            learning_rate:               5e-5,
            batch_size:                  8,
            max_epochs:                  5,
            warmup_ratio:                0.05,
            weight_decay:                0.001,
            max_grad_norm:               0.5,
            save_steps:                  1000,
            eval_steps:                  1000,
            logging_steps:               50,
            gradient_accumulation_steps: 2,
            max_seq_length:              2048,
            lora_rank:                   Some(8),
            lora_alpha:                  Some(16.0),
            lora_dropout:                Some(0.05),
            quantization_config:         None,
            dataloader_num_workers:      4,
            dataloader_pin_memory:       true,
            mixed_precision:             Some(MixedPrecision::Bf16),
            gradient_checkpointing:      true,
            early_stopping_patience:     Some(5),
            label_smoothing:             Some(0.1),
            distributed_config:          None,
            model_specific_config:       None,
            evaluation_config:           None,
            training_hooks:              vec![],
        };

        let merged = utils::merge_configs(config1, config2);

        // Config2 values should override config1
        assert_eq!(merged.learning_rate, config2.learning_rate);
        assert_eq!(merged.batch_size, config2.batch_size);
        assert_eq!(merged.lora_rank, config2.lora_rank);

        assert!(merged.validate_comprehensive().is_ok());
    }
}
