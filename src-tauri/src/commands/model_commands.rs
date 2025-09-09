//! Model management and fine-tuning commands for Tauri
//!
//! This module provides Tauri commands for managing AI models, fine-tuning jobs,
//! and related operations for the Rust AI IDE.

use crate::state::AppState;
use crate::commands::ai::services::{AIAnalysisConfig, AIServiceState};
use rust_ai_ide_ai::finetune;
use rust_ai_ide_ai::model_loader;
use rust_ai_ide_ai::inference;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Model information returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub model_type: ModelType,
    pub model_size: ModelSize,
    pub model_path: Option<PathBuf>,
    pub quantization: Option<Quantization>,
    pub lora_adapters: Vec<String>,
    pub status: ModelStatus,
    pub memory_usage_mb: Option<u64>,
    pub is_loaded: bool,
    pub supports_fine_tuning: bool,
}

/// Model types for display
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelType {
    CodeLlama,
    StarCoder,
    Custom,
}

/// Model sizes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelSize {
    Small,
    Medium,
    Large,
}

/// Quantization options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Quantization {
    None,
    Int8,
    Int4,
    GPTQ,
}

/// Model status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelStatus {
    Available,
    Downloading,
    Downloaded,
    Loading,
    Loaded,
    Unloading,
    Error,
}

/// Fine-tuning job information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FineTuneJobInfo {
    pub job_id: String,
    pub name: String,
    pub description: Option<String>,
    pub base_model: String,
    pub model_type: ModelType,
    pub status: TrainingStatus,
    pub progress: TrainingProgress,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub output_path: Option<PathBuf>,
    pub metrics: Option<TrainingMetrics>,
    pub error_message: Option<String>,
    pub config: TrainingConfigInfo,
}

/// Training status (frontend version)
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

/// Training progress
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingProgress {
    pub epoch: usize,
    pub total_epochs: usize,
    pub step: usize,
    pub total_steps: usize,
    pub loss: Option<f32>,
    pub learning_rate: Option<f32>,
    pub estimated_time_remaining: Option<f64>, // in seconds
    pub memory_usage_mb: Option<u64>,
    pub gpu_utilization: Option<f32>,
}

/// Training metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingMetrics {
    pub final_loss: f32,
    pub training_time_seconds: u64,
    pub peak_memory_usage_mb: u64,
    pub samples_per_second: f32,
    pub validation_loss: Option<f32>,
    pub perplexity: Option<f32>,
    pub bleu_score: Option<f32>,
    pub code_bleu_score: Option<f32>,
}

/// Training configuration info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingConfigInfo {
    pub learning_rate: f32,
    pub batch_size: usize,
    pub max_epochs: usize,
    pub lora_rank: Option<usize>,
    pub mixed_precision: bool,
    pub max_seq_length: usize,
    pub dataset_size: Option<usize>,
}

/// Model download request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadRequest {
    pub model_type: ModelType,
    pub model_size: ModelSize,
    pub model_version: Option<String>,
    pub destination_path: Option<PathBuf>,
    pub force_download: bool,
}

/// Model loading request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLoadingRequest {
    pub model_path: PathBuf,
    pub model_type: ModelType,
    pub model_size: ModelSize,
    pub quantization: Option<Quantization>,
    pub lora_adapters: Vec<String>,
    pub device: DeviceType,
    pub endpoint: Option<String>,
}

/// Device types for model loading
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceType {
    Cpu,
    Cuda,
    Auto,
}

/// Fine-tuning request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FineTuningRequest {
    pub job_name: String,
    pub description: Option<String>,
    pub base_model: String,
    pub dataset_path: PathBuf,
    pub config: TrainingConfigInfo,
    pub output_path: Option<PathBuf>,
    pub enable_monitoring: bool,
}

/// Dataset preparation request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetPreparationRequest {
    pub source_paths: Vec<PathBuf>,
    pub output_path: PathBuf,
    pub task_type: TaskType,
    pub filters: DatasetFilters,
}

/// Task types for dataset preparation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskType {
    CodeCompletion,
    ErrorCorrection,
    Documentation,
    CodeReview,
}

/// Dataset filters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetFilters {
    pub min_file_size: usize,
    pub max_file_size: usize,
    pub allowed_extensions: Vec<String>,
    pub quality_threshold: f32,
    pub include_tests: bool,
    pub max_samples: Option<usize>,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStatus {
    pub memory_usage_gb: f64,
    pub memory_limit_gb: f64,
    pub gpu_usage_percent: f32,
    pub gpu_memory_usage_gb: f64,
    pub gpu_memory_limit_gb: f64,
    pub active_jobs: usize,
    pub available_models: usize,
    pub system_load: f32,
}

/// Global state for model management
#[derive(Debug)]
pub struct ModelManagementState {
    pub available_models: Arc<Mutex<HashMap<String, ModelInfo>>>,
    pub fine_tune_jobs: Arc<Mutex<HashMap<String, FineTuneJobInfo>>>,
    pub orchestrator: Option<Arc<Mutex<finetune::TrainingOrchestrator>>>,
    pub model_loader: Option<Arc<Mutex<model_loader::ModelLoader>>>,
    pub inference_engine: Option<Arc<Mutex<dyn inference::InferenceEngine + Send + Sync>>>,
}

/// Get list of available models
#[tauri::command]
pub async fn list_available_models(
    _state: State<'_, AppState>,
) -> Result<Vec<ModelInfo>, String> {
    // Get models from model registry
    let registry = finetune::create_orchestrator()
        .map_err(|e| format!("Failed to create orchestrator: {}", e))?;

    // Convert to ModelInfo format
    let models = vec![
        ModelInfo {
            id: "codellama-7b".to_string(),
            model_type: ModelType::CodeLlama,
            model_size: ModelSize::Medium,
            model_path: None,
            quantization: Some(Quantization::Int4),
            lora_adapters: vec![],
            status: ModelStatus::Available,
            memory_usage_mb: Some(7168),
            is_loaded: false,
            supports_fine_tuning: true,
        },
        ModelInfo {
            id: "codellama-13b".to_string(),
            model_type: ModelType::CodeLlama,
            model_size: ModelSize::Large,
            model_path: None,
            quantization: Some(Quantization::Int8),
            lora_adapters: vec![],
            status: ModelStatus::Available,
            memory_usage_mb: Some(14000),
            is_loaded: false,
            supports_fine_tuning: true,
        },
        ModelInfo {
            id: "starcoder-7b".to_string(),
            model_type: ModelType::StarCoder,
            model_size: ModelSize::Medium,
            model_path: None,
            quantization: Some(Quantization::Int4),
            lora_adapters: vec![],
            status: ModelStatus::Available,
            memory_usage_mb: Some(7738),
            is_loaded: false,
            supports_fine_tuning: true,
        },
    ];

    Ok(models)
}

/// Get list of downloaded models
#[tauri::command]
pub async fn list_downloaded_models(
    _state: State<'_, AppState>,
) -> Result<Vec<ModelInfo>, String> {
    // List models in the local model directory
    let models_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".rust-ai-ide")
        .join("models");

    if !models_dir.exists() {
        return Ok(vec![]);
    }

    let mut models = Vec::new();

    for entry in std::fs::read_dir(&models_dir)
        .map_err(|e| format!("Failed to read models directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(model_name) = path.file_name().and_then(|n| n.to_str()) {
                let (model_type, model_size) = parse_model_name(model_name);

                models.push(ModelInfo {
                    id: model_name.to_string(),
                    model_type,
                    model_size,
                    model_path: Some(path),
                    quantization: None,
                    lora_adapters: vec![],
                    status: ModelStatus::Downloaded,
                    memory_usage_mb: None,
                    is_loaded: false,
                    supports_fine_tuning: true,
                });
            }
        }
    }

    Ok(models)
}

/// Get loaded models
#[tauri::command]
pub async fn get_loaded_models(
    _state: State<'_, AppState>,
) -> Result<Vec<ModelInfo>, String> {
    // Get loaded models from model loader
    // Placeholder implementation
    Ok(vec![])
}

/// Load a model
#[tauri::command]
pub async fn load_model(
    _state: State<'_, AppState>,
    request: ModelLoadingRequest,
) -> Result<ModelInfo, String> {
    log::info!("Loading model from {:?}", request.model_path);

    // Validate request parameters
    if !request.model_path.exists() {
        return Err(format!("Model path does not exist: {:?}", request.model_path));
    }

    // Create model config
    let config = model_loader::ModelLoadConfig {
        quantization: request.quantization.map(|q| match q {
            Quantization::None => rust_ai_ide_ai::Quantization::None,
            Quantization::Int8 => rust_ai_ide_ai::Quantization::Int8,
            Quantization::Int4 => rust_ai_ide_ai::Quantization::Int4,
            Quantization::GPTQ => rust_ai_ide_ai::Quantization::GPTQ,
        }),
        lora_adapters: request.lora_adapters,
        memory_limit_mb: None,
        device: match request.device {
            DeviceType::Cpu => model_loader::ModelDevice::Cpu,
            DeviceType::Cuda => model_loader::ModelDevice::Gpu,
            DeviceType::Auto => model_loader::ModelDevice::Auto,
        },
        lazy_loading: true,
        enable_cache: true,
    };

    // Placeholder: In real implementation, this would load the actual model
    // For now, return success

    Ok(ModelInfo {
        id: format!("{}_{:?}", request.model_path.display(), request.model_size),
        model_type: request.model_type,
        model_size: request.model_size,
        model_path: Some(request.model_path),
        quantization: request.quantization,
        lora_adapters: request.lora_adapters,
        status: ModelStatus::Loaded,
        memory_usage_mb: Some(4096),
        is_loaded: true,
        supports_fine_tuning: true,
    })
}

/// Unload a model
#[tauri::command]
pub async fn unload_model(
    _state: State<'_, AppState>,
    model_id: String,
) -> Result<(), String> {
    log::info!("Unloading model: {}", model_id);

    // Placeholder: In real implementation, this would unload the model
    Ok(())
}

/// Get model status
#[tauri::command]
pub async fn get_model_status(
    _state: State<'_, AppState>,
    model_id: String,
) -> Result<ModelInfo, String> {
    // Placeholder implementation
    Ok(ModelInfo {
        id: model_id,
        model_type: ModelType::CodeLlama,
        model_size: ModelSize::Medium,
        model_path: None,
        quantization: Some(Quantization::Int4),
        lora_adapters: vec![],
        status: ModelStatus::Loaded,
        memory_usage_mb: Some(4096),
        is_loaded: true,
        supports_fine_tuning: true,
    })
}

/// Start fine-tuning job
#[tauri::command]
pub async fn start_finetune_job(
    _state: State<'_, AppState>,
    request: FineTuningRequest,
) -> Result<String, String> {
    log::info!("Starting fine-tuning job: {}", request.job_name);

    // Validate dataset path
    if !request.dataset_path.exists() {
        return Err(format!("Dataset path does not exist: {:?}", request.dataset_path));
    }

    // Create fine-tuning job
    let job_id = format!("ft_{}", uuid::Uuid::new_v4());

    let config = finetune::TrainingConfig {
        learning_rate: request.config.learning_rate,
        batch_size: request.config.batch_size,
        max_epochs: request.config.max_epochs,
        warmup_ratio: 0.1,
        weight_decay: 0.01,
        max_grad_norm: 1.0,
        save_steps: 500,
        eval_steps: 500,
        logging_steps: 100,
        gradient_accumulation_steps: 4,
        max_seq_length: request.config.max_seq_length,
        lora_rank: request.config.lora_rank,
        lora_alpha: request.config.lora_rank.map(|r| r as f32 * 2.0),
        lora_dropout: Some(0.1),
        quantization_config: None,
        dataloader_num_workers: 4,
        dataloader_pin_memory: true,
        mixed_precision: if request.config.mixed_precision {
            Some(finetune::MixedPrecision::Fp16)
        } else {
            None
        },
        gradient_checkpointing: true,
        early_stopping_patience: Some(3),
        label_smoothing: None,
        distributed_config: None,
        model_specific_config: None,
        evaluation_config: None,
        training_hooks: vec![],
    };

    let job = finetune::FineTuneJob {
        id: job_id.clone(),
        name: request.job_name,
        description: request.description,
        base_model: request.base_model,
        model_type: match parse_model_name(&request.base_model).0 {
            ModelType::CodeLlama => finetune::ModelType::CodeLlama,
            ModelType::StarCoder => finetune::ModelType::StarCoder,
            _ => finetune::ModelType::CodeLlama,
        },
        dataset_path: request.dataset_path,
        config,
        status: finetune::TrainingStatus::Created,
        progress: finetune::TrainingProgress {
            epoch: 0,
            total_epochs: request.config.max_epochs,
            step: 0,
            total_steps: 1000, // Placeholder
            loss: None,
            learning_rate: Some(request.config.learning_rate),
            estimated_time_remaining: Some(3600.0),
            memory_usage_mb: Some(4096),
            gpu_utilization: Some(0.0),
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        output_path: request.output_path,
        metrics: None,
        error_message: None,
    };

    // Placeholder: In real implementation, this would start the actual training
    log::info!("Fine-tuning job created with ID: {}", job_id);

    Ok(job_id)
}

/// Get fine-tuning job status
#[tauri::command]
pub async fn get_finetune_progress(
    _state: State<'_, AppState>,
    job_id: String,
) -> Result<FineTuneJobInfo, String> {
    // Placeholder implementation
    Ok(FineTuneJobInfo {
        job_id: job_id.clone(),
        name: "Sample Training Job".to_string(),
        description: Some("A sample fine-tuning job".to_string()),
        base_model: "codellama-7b".to_string(),
        model_type: ModelType::CodeLlama,
        status: TrainingStatus::Training,
        progress: TrainingProgress {
            epoch: 1,
            total_epochs: 3,
            step: 250,
            total_steps: 1000,
            loss: Some(1.8),
            learning_rate: Some(5e-5),
            estimated_time_remaining: Some(2700.0),
            memory_usage_mb: Some(6144),
            gpu_utilization: Some(75.0),
        },
        created_at: chrono::Utc::now() - chrono::Duration::hours(2),
        updated_at: chrono::Utc::now(),
        output_path: None,
        metrics: None,
        error_message: None,
        config: TrainingConfigInfo {
            learning_rate: 5e-5,
            batch_size: 8,
            max_epochs: 3,
            lora_rank: Some(8),
            mixed_precision: true,
            max_seq_length: 2048,
            dataset_size: Some(10000),
        },
    })
}

/// Cancel fine-tuning job
#[tauri::command]
pub async fn cancel_finetune_job(
    _state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    log::info!("Cancelling fine-tuning job: {}", job_id);

    // Placeholder: In real implementation, this would stop the actual training
    Ok(())
}

/// List fine-tuning jobs
#[tauri::command]
pub async fn list_finetune_jobs(
    _state: State<'_, AppState>,
) -> Result<Vec<FineTuneJobInfo>, String> {
    // Placeholder implementation
    Ok(vec![FineTuneJobInfo {
        job_id: "ft_sample_1".to_string(),
        name: "Sample Training Job".to_string(),
        description: Some("A sample fine-tuning job".to_string()),
        base_model: "codellama-7b".to_string(),
        model_type: ModelType::CodeLlama,
        status: TrainingStatus::Completed,
        progress: TrainingProgress {
            epoch: 3,
            total_epochs: 3,
            step: 1000,
            total_steps: 1000,
            loss: Some(0.8),
            learning_rate: Some(1e-5),
            estimated_time_remaining: Some(0.0),
            memory_usage_mb: Some(6144),
            gpu_utilization: Some(0.0),
        },
        created_at: chrono::Utc::now() - chrono::Duration::hours(4),
        updated_at: chrono::Utc::now() - chrono::Duration::hours(1),
        output_path: Some(PathBuf::from("/models/codellama-finetuned")),
        metrics: Some(TrainingMetrics {
            final_loss: 0.8,
            training_time_seconds: 10800,
            peak_memory_usage_mb: 7168,
            samples_per_second: 100.0,
            validation_loss: Some(0.9),
            perplexity: Some(12.0),
            bleu_score: Some(0.8),
            code_bleu_score: Some(0.75),
        }),
        error_message: None,
        config: TrainingConfigInfo {
            learning_rate: 5e-5,
            batch_size: 8,
            max_epochs: 3,
            lora_rank: Some(8),
            mixed_precision: true,
            max_seq_length: 2048,
            dataset_size: Some(10000),
        },
    }])
}

/// Prepare dataset for fine-tuning
#[tauri::command]
pub async fn prepare_dataset(
    _state: State<'_, AppState>,
    request: DatasetPreparationRequest,
) -> Result<String, String> {
    log::info!("Preparing dataset for task: {:?}", request.task_type);

    // Validate source paths
    for path in &request.source_paths {
        if !path.exists() {
            return Err(format!("Source path does not exist: {:?}", path));
        }
    }

    // Validate output path
    if let Some(parent) = request.output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }
    }

    // Placeholder: In real implementation, this would create the dataset
    let task_id = format!("prep_{}", uuid::Uuid::new_v4());
    log::info!("Dataset preparation started with task ID: {}", task_id);

    Ok(task_id)
}

/// Get resource usage status
#[tauri::command]
pub async fn get_resource_status(
    _state: State<'_, AppState>,
) -> Result<ResourceStatus, String> {
    // Placeholder implementation
    Ok(ResourceStatus {
        memory_usage_gb: 4.2,
        memory_limit_gb: 16.0,
        gpu_usage_percent: 45.0,
        gpu_memory_usage_gb: 3.6,
        gpu_memory_limit_gb: 8.0,
        active_jobs: 1,
        available_models: 3,
        system_load: 0.7,
    })
}

/// Validate model configuration
#[tauri::command]
pub async fn validate_model_config(
    _state: State<'_, AppState>,
    request: ModelLoadingRequest,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    // Check model path
    if !request.model_path.exists() {
        return Err(format!("Model path does not exist: {:?}", request.model_path));
    }

    // Check memory requirements
    let estimated_memory = match (request.model_type, request.model_size, request.quantization) {
        (ModelType::CodeLlama, ModelSize::Large, Some(Quantization::Int8)) => 14.0,
        (ModelType::CodeLlama, ModelSize::Large, Some(Quantization::Int4)) => 7.0,
        (ModelType::CodeLlama, ModelSize::Medium, Some(Quantization::Int4)) => 4.0,
        (ModelType::CodeLlama, ModelSize::Small, Some(Quantization::Int4)) => 2.0,
        (ModelType::StarCoder, ModelSize::Large, Some(Quantization::Int4)) => 8.0,
        (ModelType::StarCoder, ModelSize::Medium, Some(Quantization::Int4)) => 4.0,
        _ => 8.0, // Default
    };

    warnings.push(format!("Estimated memory usage: {:.1}GB", estimated_memory));

    if estimated_memory > 8.0 {
        warnings.push("High memory usage may require GPU with sufficient VRAM".to_string());
    }

    // Check LoRA adapters
    for adapter in &request.lora_adapters {
        if !adapter.ends_with(".bin") && !adapter.ends_with(".safetensors") {
            warnings.push(format!("LoRA adapter {} may have incorrect format", adapter));
        }
    }

    // Device-specific warnings
    match request.device {
        DeviceType::Cpu => {
            if estimated_memory > 4.0 {
                warnings.push("CPU mode with high memory requirement may be slow".to_string());
            }
        }
        DeviceType::Cuda => {
            warnings.push("CUDA mode requires compatible NVIDIA GPU".to_string());
        }
        DeviceType::Auto => {
            warnings.push("Auto mode will select best available device".to_string());
        }
    }

    Ok(warnings)
}

/// Download model from Hugging Face or other sources
#[tauri::command]
pub async fn download_model(
    _state: State<'_, AppState>,
    request: ModelDownloadRequest,
) -> Result<String, String> {
    log::info!("Downloading model: {:?} {:?}", request.model_type, request.model_size);

    // Determine model identifier
    let model_identifier = match (request.model_type, request.model_size) {
        (ModelType::CodeLlama, ModelSize::Medium) => "codellama/CodeLlama-7b-hf",
        (ModelType::CodeLlama, ModelSize::Large) => "codellama/CodeLlama-13b-hf",
        (ModelType::StarCoder, ModelSize::Medium) => "bigcode/starcoder",
        (ModelType::StarCoder, ModelSize::Large) => "bigcode/starcoder2-7b",
        _ => return Err("Unsupported model combination".to_string()),
    };

    // Determine destination path
    let destination_path = request.destination_path.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".rust-ai-ide")
            .join("models")
            .join(format!("{:?}_{:?}", request.model_type, request.model_size).to_lowercase())
    });

    // Placeholder: In real implementation, this would download the model
    log::info!("Would download model {} to {:?}", model_identifier, destination_path);

    let download_id = format!("dl_{}", uuid::Uuid::new_v4());

    Ok(download_id)
}

/// Helper function to parse model name and extract type/size
fn parse_model_name(model_name: &str) -> (ModelType, ModelSize) {
    let lower_name = model_name.to_lowercase();

    let model_type = if lower_name.contains("codellama") {
        ModelType::CodeLlama
    } else if lower_name.contains("starcoder") {
        ModelType::StarCoder
    } else {
        ModelType::Custom
    };

    let model_size = if lower_name.contains("large") || lower_name.contains("13b") {
        ModelSize::Large
    } else if lower_name.contains("small") || lower_name.contains("1b") || lower_name.contains("3b") {
        ModelSize::Small
    } else {
        ModelSize::Medium
    };

    (model_type, model_size)
}