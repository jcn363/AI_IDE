//! Training orchestration and job management for fine-tuning
//!
//! This module provides comprehensive job management, progress tracking, and orchestration
//! capabilities for fine-tuning CodeLlama and StarCoder models.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Training orchestrator for managing fine-tuning jobs
#[derive(Debug)]
pub struct TrainingOrchestrator {
    jobs: Arc<RwLock<HashMap<String, JobState>>>,
    event_sender: mpsc::UnboundedSender<TrainingEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TrainingEvent>>>>,
    max_concurrent_jobs: usize,
    active_jobs: Arc<RwLock<usize>>,
    resource_manager: Arc<ResourceManager>,
}

/// Job state information
#[derive(Debug, Clone)]
struct JobState {
    job: crate::finetune::FineTuneJob,
    status: TrainingStatus,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    last_progress: Option<TrainingProgress>,
    error_message: Option<String>,
    process_handle: Option<std::process::Child>,
}

/// Resource manager for tracking hardware resources
#[derive(Debug)]
pub struct ResourceManager {
    available_memory_gb: f64,
    available_gpu_memory_gb: Vec<f64>,
    current_memory_usage: Arc<RwLock<f64>>,
    current_gpu_usage: Arc<RwLock<Vec<f64>>>,
    job_resource_map: Arc<RwLock<HashMap<String, JobResources>>>,
}

/// Resources allocated to a specific job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResources {
    pub allocated_memory_gb: f64,
    pub allocated_gpu_memory_gb: f64,
    pub gpu_devices: Vec<usize>,
    pub cpu_cores: usize,
}

/// External training process interface
#[async_trait::async_trait]
pub trait TrainingProcess: Send + Sync {
    /// Start training with the given job configuration
    async fn start_training(&self, job: &crate::finetune::FineTuneJob) -> Result<String>;

    /// Check if the training process is still running
    async fn is_running(&self, process_id: &str) -> Result<bool>;

    /// Stop training for the given process
    async fn stop_training(&self, process_id: &str) -> Result<()>;

    /// Get training progress from the process
    async fn get_progress(&self, process_id: &str) -> Result<TrainingProgress>;

    /// Get final training metrics
    async fn get_final_metrics(&self, process_id: &str) -> Result<TrainingMetrics>;
}

// Re-export for convenience
pub use crate::finetune::{FineTuneJob, TrainingMetrics, TrainingProgress, TrainingStatus};

/// Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    pub max_concurrent_jobs: usize,
    pub max_memory_per_job_gb: f64,
    pub max_gpu_memory_per_job_gb: f64,
    pub enable_resource_monitoring: bool,
    pub progress_poll_interval_seconds: u64,
    pub job_timeout_hours: u64,
    pub enable_auto_recovery: bool,
    pub checkpoint_interval_minutes: u64,
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 2,
            max_memory_per_job_gb: 16.0,
            max_gpu_memory_per_job_gb: 8.0,
            enable_resource_monitoring: true,
            progress_poll_interval_seconds: 30,
            job_timeout_hours: 48,
            enable_auto_recovery: true,
            checkpoint_interval_minutes: 30,
        }
    }
}

impl TrainingOrchestrator {
    /// Create a new training orchestrator with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(OrchestrationConfig::default())
    }

    /// Create a new training orchestrator with custom configuration
    pub fn with_config(config: OrchestrationConfig) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            max_concurrent_jobs: config.max_concurrent_jobs,
            active_jobs: Arc::new(RwLock::new(0)),
            resource_manager: Arc::new(ResourceManager::new()?),
        })
    }

    /// Start a fine-tuning job
    pub async fn start_job(&self, job: FineTuneJob) -> Result<String> {
        let job_id = job.id.clone();

        // Check resource availability
        self.check_resource_availability(&job).await?;
        self.check_job_limits().await?;

        // Create job state
        let job_state = JobState {
            job: job.clone(),
            status: TrainingStatus::Created,
            start_time: None,
            end_time: None,
            last_progress: None,
            error_message: None,
            process_handle: None,
        };

        // Store job
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.clone(), job_state);
        }

        // Allocate resources
        self.resource_manager
            .allocate_resources(&job_id, &job)
            .await?;

        // Emit job started event
        self.emit_event(TrainingEvent::JobStarted {
            job_id: job_id.clone(),
            timestamp: Utc::now(),
        })
        .await;

        // Start the training process asynchronously
        self.start_training_process(job_id.clone()).await?;

        Ok(job_id)
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: &str) -> Result<FineTuneJob> {
        let jobs = self.jobs.read().await;
        match jobs.get(job_id) {
            Some(job_state) => {
                let mut job = job_state.job.clone();
                job.status = job_state.status.clone();

                if let Some(progress) = &job_state.last_progress {
                    job.progress = progress.clone();
                }

                Ok(job)
            }
            None => Err(anyhow::anyhow!("Job not found: {}", job_id)),
        }
    }

    /// Cancel a running job
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job_state) = jobs.get_mut(job_id) {
            if job_state.status != TrainingStatus::Completed
                && job_state.status != TrainingStatus::Failed
            {
                job_state.status = TrainingStatus::Cancelled;
                job_state.end_time = Some(Utc::now());

                // Terminate training process if running
                if let Some(mut process) = job_state.process_handle.take() {
                    let _ = process.kill();
                }

                // Release resources
                self.resource_manager.release_resources(job_id).await?;

                // Update active job count
                *self.active_jobs.write().await = self.active_jobs.read().await.saturating_sub(1);

                // Emit cancellation event
                self.emit_event(TrainingEvent::JobFailed {
                    job_id: job_id.to_string(),
                    error: "Job cancelled by user".to_string(),
                    timestamp: Utc::now(),
                })
                .await;

                log::info!("Job {} cancelled successfully", job_id);
            }
        }

        Ok(())
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Result<Vec<FineTuneJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .map(|state| {
                let mut job = state.job.clone();
                job.status = state.status.clone();
                if let Some(progress) = &state.last_progress {
                    job.progress = progress.clone();
                }
                job
            })
            .collect())
    }

    /// Get active jobs
    pub async fn get_active_jobs(&self) -> Result<Vec<FineTuneJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .filter(|state| {
                matches!(
                    state.status,
                    TrainingStatus::Training | TrainingStatus::PreparingData
                )
            })
            .map(|state| {
                let mut job = state.job.clone();
                job.status = state.status.clone();
                if let Some(progress) = &state.last_progress {
                    job.progress = progress.clone();
                }
                job
            })
            .collect())
    }

    /// Get completed jobs
    pub async fn get_completed_jobs(&self) -> Result<Vec<FineTuneJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .filter(|state| matches!(state.status, TrainingStatus::Completed))
            .map(|state| {
                let mut job = state.job.clone();
                job.status = state.status.clone();
                if let Some(progress) = &state.last_progress {
                    job.progress = progress.clone();
                }
                job
            })
            .collect())
    }

    /// Clean up completed jobs
    pub async fn cleanup_completed_jobs(&self, older_than_hours: u64) -> Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(older_than_hours as i64);
        let mut jobs_to_cleanup = Vec::new();

        {
            let jobs = self.jobs.read().await;
            for (job_id, job_state) in jobs.iter() {
                if matches!(
                    job_state.status,
                    TrainingStatus::Completed | TrainingStatus::Failed | TrainingStatus::Cancelled
                ) {
                    if let Some(end_time) = job_state.end_time {
                        if end_time < cutoff_time {
                            jobs_to_cleanup.push(job_id.clone());
                        }
                    }
                }
            }
        }

        if !jobs_to_cleanup.is_empty() {
            let mut jobs = self.jobs.write().await;
            for job_id in &jobs_to_cleanup {
                jobs.remove(job_id);
            }
        }

        log::info!("Cleaned up {} old jobs", jobs_to_cleanup.len());
        Ok(jobs_to_cleanup.len())
    }

    /// Subscribe to training events
    pub async fn subscribe_events(&self) -> mpsc::UnboundedReceiver<TrainingEvent> {
        let mut guard = self.event_receiver.write().await;
        let receiver = guard.take().unwrap_or_else(|| {
            let (_, r) = mpsc::unbounded_channel();
            r
        });
        receiver
    }

    /// Start monitoring the orchestrator
    pub async fn start_monitoring(&self) -> Result<()> {
        let orchestrator = Arc::new(self.clone());
        tokio::spawn(async move {
            orchestrator.monitoring_loop().await;
        });

        log::info!("Training orchestrator monitoring started");
        Ok(())
    }

    /// Internal: Start training process for a job
    async fn start_training_process(&self, job_id: String) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job_state) = jobs.get_mut(&job_id) {
            job_state.status = TrainingStatus::Initializing;
            job_state.start_time = Some(Utc::now());

            // Simulate starting a training process
            // In a real implementation, this would launch the actual training script
            let process_id = format!("process_{}", job_id);

            // Update active job count
            *self.active_jobs.write().await += 1;

            // Start the monitoring task
            self.start_job_monitoring(job_id.clone()).await?;
        }

        Ok(())
    }

    /// Start monitoring a specific job
    async fn start_job_monitoring(&self, job_id: String) -> Result<()> {
        let orchestrator = Arc::new(self.clone());
        let job_id_clone = job_id.clone();

        tokio::spawn(async move {
            if let Err(e) = orchestrator.monitor_job(job_id_clone).await {
                log::error!("Error monitoring job: {}", e);
            }
        });

        Ok(())
    }

    /// Monitor a specific job's progress
    async fn monitor_job(&self, job_id: String) -> Result<()> {
        let mut progress = TrainingProgress {
            epoch: 0,
            total_epochs: 3,
            step: 0,
            total_steps: 1000,
            loss: Some(2.5),
            learning_rate: Some(2e-5),
            estimated_time_remaining: Some(std::time::Duration::from_secs(3600)),
            memory_usage_mb: Some(4096),
            gpu_utilization: Some(85.0),
        };

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;

            // Update job status
            let mut jobs = self.jobs.write().await;
            if let Some(job_state) = jobs.get_mut(&job_id) {
                match job_state.status {
                    TrainingStatus::Training => {
                        // Simulate progress updates
                        progress.step += 10;
                        progress.epoch =
                            progress.step / (progress.total_steps / progress.total_epochs);

                        if progress.step >= progress.total_steps {
                            // Training completed
                            job_state.status = TrainingStatus::Completed;
                            job_state.end_time = Some(Utc::now());
                            job_state.last_progress = Some(progress.clone());

                            // Release resources
                            drop(jobs);
                            self.resource_manager.release_resources(&job_id).await?;
                            *self.active_jobs.write().await =
                                self.active_jobs.read().await.saturating_sub(1);

                            // Emit completion event
                            self.emit_event(TrainingEvent::JobCompleted {
                                job_id: job_id.clone(),
                                metrics: TrainingMetrics {
                                    final_loss: 0.5,
                                    training_time_seconds: 3600,
                                    peak_memory_usage_mb: 4096,
                                    samples_per_second: 100.0,
                                    validation_loss: Some(0.6),
                                    perplexity: Some(15.0),
                                    bleu_score: Some(0.8),
                                    code_bleu_score: Some(0.75),
                                },
                                timestamp: Utc::now(),
                            })
                            .await;

                            break;
                        } else {
                            job_state.last_progress = Some(progress.clone());
                        }
                    }
                    TrainingStatus::Cancelled => {
                        break;
                    }
                    _ => {
                        // Update to training status if still initializing
                        if matches!(job_state.status, TrainingStatus::Initializing) {
                            job_state.status = TrainingStatus::Training;
                            job_state.last_progress = Some(progress.clone());
                        }
                    }
                }
            } else {
                break;
            }

            // Emit progress event
            self.emit_event(TrainingEvent::JobProgress {
                job_id: job_id.clone(),
                progress: progress.clone(),
                timestamp: Utc::now(),
            })
            .await;
        }

        log::info!("Job monitoring completed for: {}", job_id);
        Ok(())
    }

    /// Monitoring loop for the orchestrator
    async fn monitoring_loop(&self) {
        log::info!("Starting training orchestrator monitoring loop");

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            // Check for jobs that have timed out
            if let Err(e) = self.check_for_timeouts().await {
                log::error!("Error checking for timeouts: {}", e);
            }

            // Log resource usage
            if let Ok(usage) = self.resource_manager.get_current_usage().await {
                log::debug!(
                    "Current resource usage - Memory: {:.1}GB, GPU: {:.1}%",
                    usage.memory_gb,
                    usage.gpu_utilization
                );
            }
        }
    }

    /// Check for jobs that have exceeded timeout
    async fn check_for_timeouts(&self) -> Result<()> {
        let mut timed_out_jobs = Vec::new();
        let timeout_duration = chrono::Duration::hours(48); // 48 hours timeout

        {
            let jobs = self.jobs.read().await;
            for (job_id, job_state) in jobs.iter() {
                if matches!(
                    job_state.status,
                    TrainingStatus::Training
                        | TrainingStatus::PreparingData
                        | TrainingStatus::Initializing
                ) {
                    if let Some(start_time) = job_state.start_time {
                        if Utc::now().signed_duration_since(start_time) > timeout_duration {
                            timed_out_jobs.push(job_id.clone());
                        }
                    }
                }
            }
        }

        for job_id in timed_out_jobs {
            log::warn!("Job {} timed out, cancelling", job_id);
            let _ = self.timeout_job(&job_id).await;
        }

        Ok(())
    }

    /// Timeout a job
    async fn timeout_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job_state) = jobs.get_mut(job_id) {
            job_state.status = TrainingStatus::Failed;
            job_state.error_message = Some("Job timed out".to_string());
            job_state.end_time = Some(Utc::now());

            // Terminate process if running
            if let Some(mut process) = job_state.process_handle.take() {
                let _ = process.kill();
            }

            // Release resources
            drop(jobs);
            self.resource_manager.release_resources(job_id).await?;
            *self.active_jobs.write().await = self.active_jobs.read().await.saturating_sub(1);

            // Emit failure event
            self.emit_event(TrainingEvent::JobFailed {
                job_id: job_id.to_string(),
                error: "Job timed out after 48 hours".to_string(),
                timestamp: Utc::now(),
            })
            .await;
        }

        Ok(())
    }

    /// Get resource status
    pub async fn get_resource_status(&self) -> Result<ResourceStatus> {
        self.resource_manager.get_current_usage().await
    }

    /// Get orchestrator statistics
    pub async fn get_statistics(&self) -> Result<OrchestratorStatistics> {
        let jobs = self.jobs.read().await;
        let active_count = *self.active_jobs.read().await;

        let mut status_counts = HashMap::new();
        for job_state in jobs.values() {
            *status_counts.entry(job_state.status.clone()).or_insert(0) += 1;
        }

        let total_jobs = jobs.len();
        let completed_jobs = status_counts
            .get(&TrainingStatus::Completed)
            .copied()
            .unwrap_or(0);
        let failed_jobs = status_counts
            .get(&TrainingStatus::Failed)
            .copied()
            .unwrap_or(0);

        Ok(OrchestratorStatistics {
            total_jobs,
            active_jobs: active_count,
            completed_jobs,
            failed_jobs,
            status_distribution: status_counts,
        })
    }

    /// Export job queue to JSON
    pub async fn export_job_queue(&self) -> Result<String> {
        let jobs = self.jobs.read().await;
        let job_summaries: Vec<_> = jobs
            .values()
            .map(|state| {
                serde_json::json!({
                    "id": state.job.id,
                    "name": state.job.name,
                    "status": state.status,
                    "model_type": state.job.model_type,
                    "created_at": state.job.created_at,
                    "start_time": state.start_time,
                    "progress": state.last_progress,
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&job_summaries)?)
    }

    /// Internal: Emit training event
    async fn emit_event(&self, event: TrainingEvent) {
        if let Err(e) = self.event_sender.send(event) {
            log::error!("Failed to emit training event: {}", e);
        }
    }

    /// Internal: Check resource availability
    async fn check_resource_availability(&self, job: &FineTuneJob) -> Result<()> {
        self.resource_manager.check_availability(job).await
    }

    /// Internal: Check job limits
    async fn check_job_limits(&self) -> Result<()> {
        let active_count = *self.active_jobs.read().await;
        if active_count >= self.max_concurrent_jobs {
            return Err(anyhow::anyhow!(
                "Maximum concurrent jobs ({}) reached. Active: {}",
                self.max_concurrent_jobs,
                active_count
            ));
        }
        Ok(())
    }
}

impl Clone for TrainingOrchestrator {
    fn clone(&self) -> Self {
        Self {
            jobs: self.jobs.clone(),
            event_sender: self.event_sender.clone(),
            event_receiver: self.event_receiver.clone(),
            max_concurrent_jobs: self.max_concurrent_jobs,
            active_jobs: self.active_jobs.clone(),
            resource_manager: self.resource_manager.clone(),
        }
    }
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Result<Self> {
        // Detect available resources
        let available_memory_gb = Self::detect_available_memory_gb()?;
        let available_gpu_memory_gb = Self::detect_gpu_memory()?;

        Ok(Self {
            available_memory_gb,
            available_gpu_memory_gb,
            current_memory_usage: Arc::new(RwLock::new(0.0)),
            current_gpu_usage: Arc::new(RwLock::new(vec![0.0; available_gpu_memory_gb.len()])),
            job_resource_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Allocate resources for a job
    pub async fn allocate_resources(&self, job_id: &str, job: &FineTuneJob) -> Result<()> {
        let mut memory_usage = self.current_memory_usage.write().await;
        let mut gpu_usage = self.current_gpu_usage.write().await;
        let mut job_map = self.job_resource_map.write().await;

        // Determine resource requirements based on model type and size
        let required_memory = match job.model_type {
            crate::finetune::ModelType::CodeLlama => match job.config.model_size {
                crate::ModelSize::Large => 32.0,
                crate::ModelSize::Medium => 16.0,
                crate::ModelSize::Small => 8.0,
            },
            crate::finetune::ModelType::StarCoder => match job.config.model_size {
                crate::ModelSize::Large => 24.0,
                crate::ModelSize::Medium => 12.0,
                crate::ModelSize::Small => 6.0,
            },
        };

        let required_gpu_memory = required_memory * 0.8; // Assume 80% needed on GPU

        // Check availability
        if *memory_usage + required_memory > self.available_memory_gb {
            return Err(anyhow::anyhow!(
                "Insufficient memory. Required: {}GB, Available: {}GB",
                required_memory,
                self.available_memory_gb - *memory_usage
            ));
        }

        // Find available GPU by iterating over all GPUs
        let gpu_index = (0..gpu_usage.len())
            .find(|&idx| gpu_usage[idx] + required_gpu_memory < self.available_gpu_memory_gb[idx])
            .ok_or_else(|| anyhow::anyhow!("No GPU with sufficient memory available"))?;

        // Allocate resources
        *memory_usage += required_memory;
        gpu_usage[gpu_index] += required_gpu_memory;

        let allocation = JobResources {
            allocated_memory_gb: required_memory,
            allocated_gpu_memory_gb: required_gpu_memory,
            gpu_devices: vec![gpu_index],
            cpu_cores: 4, // Allocate 4 CPU cores
        };

        job_map.insert(job_id.to_string(), allocation);

        log::info!(
            "Allocated resources for job {}: {:.1}GB memory, GPU {}",
            job_id,
            required_memory,
            gpu_index
        );
        Ok(())
    }

    /// Release resources for a job
    pub async fn release_resources(&self, job_id: &str) -> Result<()> {
        let mut memory_usage = self.current_memory_usage.write().await;
        let mut gpu_usage = self.current_gpu_usage.write().await;
        let mut job_map = self.job_resource_map.write().await;

        if let Some(resources) = job_map.remove(job_id) {
            *memory_usage -= resources.allocated_memory_gb;
            for &gpu_idx in &resources.gpu_devices {
                if gpu_idx < gpu_usage.len() {
                    gpu_usage[gpu_idx] -= resources.allocated_gpu_memory_gb;
                }
            }

            log::info!("Released resources for job {}", job_id);
        }

        Ok(())
    }

    /// Check resource availability for a job
    pub async fn check_availability(&self, job: &FineTuneJob) -> Result<()> {
        let memory_usage = self.current_memory_usage.read().await;

        let required_memory = match job.model_type {
            crate::finetune::ModelType::CodeLlama => 16.0, // Base requirement
            crate::finetune::ModelType::StarCoder => 12.0,
        };

        if *memory_usage + required_memory > self.available_memory_gb {
            return Err(anyhow::anyhow!("Insufficient memory for job"));
        }

        Ok(())
    }

    /// Get current resource usage
    pub async fn get_current_usage(&self) -> Result<ResourceStatus> {
        let memory_usage = *self.current_memory_usage.read().await;
        let gpu_usage = self.current_gpu_usage.read().await;

        let total_gpu_utilization = if gpu_usage.is_empty() {
            0.0
        } else {
            gpu_usage.iter().sum::<f64>() / gpu_usage.len() as f64
        };

        Ok(ResourceStatus {
            memory_gb: memory_usage,
            total_memory_gb: self.available_memory_gb,
            gpu_count: gpu_usage.len(),
            gpu_utilization: total_gpu_utilization * 100.0,
            available_gpu_memory_gb: self.available_gpu_memory_gb.clone(),
        })
    }

    /// Detect available system memory
    fn detect_available_memory_gb() -> Result<f64> {
        // This is a placeholder - in real implementation, you'd use system APIs
        // For now, assume 64GB available
        Ok(64.0)
    }

    /// Detect available GPU memory
    fn detect_gpu_memory() -> Result<Vec<f64>> {
        // This is a placeholder - in real implementation, you'd detect actual GPUs
        // Assume 2 GPUs with 16GB each
        Ok(vec![16.0, 16.0])
    }
}

/// Resource status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    pub memory_gb: f64,
    pub total_memory_gb: f64,
    pub gpu_count: usize,
    pub gpu_utilization: f32,
    pub available_gpu_memory_gb: Vec<f64>,
}

/// Orchestrator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatistics {
    pub total_jobs: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub status_distribution: HashMap<TrainingStatus, usize>,
}

/// Create a default training orchestrator instance
pub fn create_orchestrator() -> Result<TrainingOrchestrator> {
    TrainingOrchestrator::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_allocation() {
        let manager = ResourceManager::new().unwrap();

        // Test resource allocation
        let job = FineTuneJob {
            id: "test_job".to_string(),
            name: "Test Job".to_string(),
            description: Some("Test job description".to_string()),
            base_model: "codellama-7b".to_string(),
            model_type: crate::finetune::ModelType::CodeLlama,
            dataset_path: PathBuf::from("test_dataset.jsonl"),
            config: crate::finetune::default_codellama_config(),
            status: TrainingStatus::Created,
            progress: TrainingProgress {
                epoch: 0,
                total_epochs: 3,
                step: 0,
                total_steps: 100,
                loss: None,
                learning_rate: None,
                estimated_time_remaining: None,
                memory_usage_mb: None,
                gpu_utilization: None,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            output_path: None,
            metrics: None,
            error_message: None,
        };

        manager.allocate_resources("test_job", &job).await.unwrap();

        let status = manager.get_current_usage().await.unwrap();
        assert!(status.memory_gb > 0.0);
        assert!(status.gpu_utilization > 0.0);

        // Release resources
        manager.release_resources("test_job").await.unwrap();

        let status = manager.get_current_usage().await.unwrap();
        assert_eq!(status.memory_gb, 0.0);
        // GPU might still show some usage due to floating point precision
    }

    #[tokio::test]
    async fn test_orchestrator_job_management() {
        let orchestrator = TrainingOrchestrator::new().unwrap();

        let job = FineTuneJob {
            id: Uuid::new_v4().to_string(),
            name: "Test Training Job".to_string(),
            description: Some("Integration test for orchestrator".to_string()),
            base_model: "codellama-7b".to_string(),
            model_type: crate::finetune::ModelType::CodeLlama,
            dataset_path: PathBuf::from("test_dataset.jsonl"),
            config: crate::finetune::default_codellama_config(),
            status: TrainingStatus::Created,
            progress: TrainingProgress {
                epoch: 0,
                total_epochs: 3,
                step: 0,
                total_steps: 1000,
                loss: None,
                learning_rate: None,
                estimated_time_remaining: None,
                memory_usage_mb: None,
                gpu_utilization: None,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            output_path: None,
            metrics: None,
            error_message: None,
        };

        // Start job
        let job_id = orchestrator.start_job(job).await.unwrap();

        // Get job status
        let job_status = orchestrator.get_job_status(&job_id).await.unwrap();
        assert!(matches!(
            job_status.status,
            TrainingStatus::Initializing | TrainingStatus::Training
        ));

        // Get all jobs
        let all_jobs = orchestrator.get_all_jobs().await.unwrap();
        assert_eq!(all_jobs.len(), 1);

        // Cancel job
        orchestrator.cancel_job(&job_id).await.unwrap();

        // Verify cancellation
        let cancelled_job = orchestrator.get_job_status(&job_id).await.unwrap();
        assert_eq!(cancelled_job.status, TrainingStatus::Cancelled);
    }
}
