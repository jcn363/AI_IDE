import { invoke } from '@tauri-apps/api/core';
import type {
  DatasetFilters,
  DatasetPreparationRequest,
  FineTuneJobInfo,
  FineTuningRequest,
  ModelInfo,
  ModelLoadingRequest,
  ModelStatus,
  ResourceStatus,
  TrainingStatus,
  QuantizationConfig,
  QuantizationOption,
  ValidationResult,
} from '../types';

// Import enhanced services
import { ExperimentTracker, type ModelVersionEntry } from './ExperimentTracker';
import { HyperparameterTuningService } from './HyperparameterTuningService';
import { FederatedLearningManager } from './FederatedLearningManager';

/**
 * Service for managing AI model operations
 * Handles loading, unloading, downloading, and fine-tuning of models
 */
export interface AutoUnloadConfig {
  // Memory pressure thresholds (in MB)
  memoryThresholdMB: number;
  memoryWarningThresholdMB: number;

  // Inactivity timeout (in minutes)
  inactivityTimeoutMinutes: number;

  // LRU settings
  enableLRU: boolean;
  maxSimultaneousModels: number;

  // Monitoring interval (in minutes)
  monitoringIntervalMinutes: number;

  // Auto-unload enabled flag
  enabled: boolean;
}

export interface ModelActivity {
  modelId: string;
  loadedAt: number;
  lastUsedAt: number;
  useCount: number;
}

export class ModelService {
  private static instance: ModelService;
  private loadedModels: Map<string, ModelInfo> = new Map();
  private modelActivity: Map<string, ModelActivity> = new Map();
  private autoUnloadConfig: AutoUnloadConfig = {
    memoryThresholdMB: 2048, // 2GB
    memoryWarningThresholdMB: 1024, // 1GB
    inactivityTimeoutMinutes: 30,
    enableLRU: true,
    maxSimultaneousModels: 2,
    monitoringIntervalMinutes: 5,
    enabled: true,
  };
  private monitoringIntervalId?: ReturnType<typeof setInterval>;
  private cleanupInProgress: boolean = false;

  // Model versioning and experimentation tracking
  private modelVersions: Map<string, ModelVersionEntry[]> = new Map();
  private experimentTracker: ExperimentTracker = new ExperimentTracker();

  // Hyperparameter tuning
  private tuningService: HyperparameterTuningService = new HyperparameterTuningService();

  // Privacy-preserving training
  private federatedLearningManager: FederatedLearningManager = new FederatedLearningManager();

  private constructor() {
    this.startMonitoring();
  }

  static getInstance(): ModelService {
    if (!ModelService.instance) {
      ModelService.instance = new ModelService();
    }
    return ModelService.instance;
  }

  /**
   * List all available models
   */
  async listAvailableModels(): Promise<ModelInfo[]> {
    try {
      const models = await invoke<ModelInfo[]>('list_available_models');
      return models.map((model) => ({
        ...model,
        status: this.mapStatus(model.status),
      }));
    } catch (error) {
      console.error('Failed to list available models:', error);
      throw new Error(`Failed to list models: ${error}`);
    }
  }

  /**
   * List downloaded models
   */
  async listDownloadedModels(): Promise<ModelInfo[]> {
    try {
      const models = await invoke<ModelInfo[]>('list_downloaded_models');
      return models.map((model) => ({
        ...model,
        status: this.mapStatus(model.status),
      }));
    } catch (error) {
      console.error('Failed to list downloaded models:', error);
      throw new Error(`Failed to list downloaded models: ${error}`);
    }
  }

  /**
   * Get currently loaded models
   */
  async getLoadedModels(): Promise<ModelInfo[]> {
    try {
      const models = await invoke<ModelInfo[]>('get_loaded_models');
      // Normalize models and update cache
      const normalizedModels = models.map((model) => ({
        ...model,
        status: this.mapStatus(model.status),
      }));

      // Update local cache with normalized models
      this.loadedModels.clear();
      normalizedModels.forEach((model) => {
        this.loadedModels.set(model.id, model);
      });

      // Initialize activity tracking for models that don't have it
      normalizedModels.forEach((model) => {
        if (!this.modelActivity.has(model.id)) {
          this.recordModelActivity(model.id, model);
        }
      });

      return normalizedModels;
    } catch (error) {
      console.error('Failed to get loaded models:', error);
      throw new Error(`Failed to get loaded models: ${error}`);
    }
  }

  /**
   * Load a model
   */
  async loadModel(request: ModelLoadingRequest): Promise<ModelInfo> {
    try {
      const model = await invoke<ModelInfo>('load_model', { request });

      if ((model as any).is_loaded) {
        this.loadedModels.set(model.id, { ...model, status: this.mapStatus(model.status) });
        // Record activity for automatic unloading
        this.recordModelActivity(model.id);
      }

      console.log(`Model loaded: ${model.id} (${(model as any).memory_usage_mb || 0}MB)`);
      return { ...model, status: this.mapStatus(model.status) };
    } catch (error) {
      console.error('Failed to load model:', error);
      throw new Error(`Failed to load model: ${error}`);
    }
  }

  /**
   * Unload a model
   */
  async unloadModel(modelId: string): Promise<void> {
    try {
      await invoke('unload_model', { modelId });
      this.loadedModels.delete(modelId);
      this.modelActivity.delete(modelId);
      console.log(`Model unloaded: ${modelId}`);
    } catch (error) {
      console.error('Failed to unload model:', error);
      throw new Error(`Failed to unload model: ${error}`);
    }
  }

  /**
   * Record model usage for activity tracking
   */
  recordModelActivity(modelId: string, model?: ModelInfo): void {
    const now = Date.now();
    const activity = this.modelActivity.get(modelId);

    if (activity) {
      activity.lastUsedAt = now;
      activity.useCount += 1;
    } else {
      // Use current time for initial loaded timestamp since ModelInfo doesn't have this property
      this.modelActivity.set(modelId, {
        modelId,
        loadedAt: now,
        lastUsedAt: now,
        useCount: 1,
      });
    }
  }

  /**
   * Start background monitoring for automatic unloading
   */
  startMonitoring(): void {
    if (!this.autoUnloadConfig.enabled) {
      return;
    }

    // Validate monitoring interval and clamp to minimum of 1 minute
    const validatedIntervalMinutes = Math.max(1, this.autoUnloadConfig.monitoringIntervalMinutes);

    // Clear any existing interval
    if (this.monitoringIntervalId) {
      clearInterval(this.monitoringIntervalId);
    }

    const intervalMs = validatedIntervalMinutes * 60 * 1000;

    this.monitoringIntervalId = setInterval(() => {
      this.performAutomaticCleanup();
    }, intervalMs);

    console.log(
      `Automatic model unloading monitoring started (${validatedIntervalMinutes} minute intervals)`
    );
  }

  /**
   * Stop background monitoring
   */
  stopMonitoring(): void {
    if (this.monitoringIntervalId) {
      clearInterval(this.monitoringIntervalId);
      this.monitoringIntervalId = undefined;
      console.log('Automatic model unloading monitoring stopped');
    }
  }

  /**
   * Perform automatic cleanup based on configured policies
   */
  private async performAutomaticCleanup(): Promise<void> {
    // Prevent reentrancy
    if (this.cleanupInProgress) {
      return;
    }

    try {
      this.cleanupInProgress = true;
      const resources = await this.getResourceStatus();
      const loadedModelIds = Array.from(this.loadedModels.keys());

      // Check memory pressure
      const memoryUsageMb = (resources as any).memory_usage_gb * 1024;
      if (memoryUsageMb > this.autoUnloadConfig.memoryThresholdMB) {
        await this.handleMemoryPressureCleanup(loadedModelIds);
      }

      // Check inactivity timeout
      await this.handleInactivityCleanup();

      // Check LRU policy
      if (
        this.autoUnloadConfig.enableLRU &&
        loadedModelIds.length > this.autoUnloadConfig.maxSimultaneousModels
      ) {
        await this.handleLRUCleanup();
      }
    } catch (error) {
      console.error('Error during automatic cleanup:', error);
    } finally {
      this.cleanupInProgress = false;
    }
  }

  /**
   * Manually trigger cleanup of models
   */
  async triggerManualCleanup(): Promise<void> {
    console.log('Starting manual cleanup triggered by user...');
    await this.performAutomaticCleanup();
    console.log('Manual cleanup completed');
  }

  /**
   * Check if monitoring is currently active
   */
  isMonitoringActive(): boolean {
    return this.monitoringIntervalId !== undefined;
  }

  /**
   * Handle cleanup due to memory pressure
   */
  private async handleMemoryPressureCleanup(loadedModelIds: string[]): Promise<void> {
    console.log('Memory pressure detected, initiating cleanup...');

    // Sort models by last used time (LRU)
    const sortedModels = loadedModelIds
      .map((id) => ({
        id,
        activity: this.modelActivity.get(id),
      }))
      .sort((a, b) => (a.activity?.lastUsedAt || 0) - (b.activity?.lastUsedAt || 0));

    // Unload oldest models until memory pressure is relieved
    for (const model of sortedModels) {
      if (!(await this.shouldUnloadModel(model.id))) {
        continue;
      }

      console.log(`Auto-unloading model due to memory pressure: ${model.id}`);
      await this.unloadModel(model.id);

      // Check if memory pressure is relieved
      const resources = await this.getResourceStatus();
      const memoryUsageMb = (resources as any).memory_usage_gb * 1024;
      if (memoryUsageMb <= this.autoUnloadConfig.memoryWarningThresholdMB) {
        break;
      }
    }
  }

  /**
   * Handle cleanup of inactive models
   */
  private async handleInactivityCleanup(): Promise<void> {
    const now = Date.now();
    const timeoutMs = this.autoUnloadConfig.inactivityTimeoutMinutes * 60 * 1000;

    for (const [modelId, activity] of Array.from(this.modelActivity.entries())) {
      if (now - activity.lastUsedAt > timeoutMs) {
        if (await this.shouldUnloadModel(modelId)) {
          console.log(
            `Auto-unloading inactive model: ${modelId} (${this.autoUnloadConfig.inactivityTimeoutMinutes} min timeout)`
          );
          await this.unloadModel(modelId);
        }
      }
    }
  }

  /**
   * Handle LRU cleanup
   */
  private async handleLRUCleanup(): Promise<void> {
    console.log('LRU cleanup triggered...');

    const sortedModels = Array.from(this.modelActivity.entries())
      .sort(([, a], [, b]) => a.lastUsedAt - b.lastUsedAt)
      .slice(0, -this.autoUnloadConfig.maxSimultaneousModels);

    for (const [modelId] of sortedModels) {
      if (await this.shouldUnloadModel(modelId)) {
        console.log(`Auto-unloading model via LRU policy: ${modelId}`);
        await this.unloadModel(modelId);
      }
    }
  }

  /**
   * Check if a model should be unloaded (safety check)
   */
  private async shouldUnloadModel(modelId: string): Promise<boolean> {
    const model = this.loadedModels.get(modelId);
    return !(!model || !(model as any).is_loaded);
  }

  /**
   * Update auto-unload configuration
   */
  updateAutoUnloadConfig(config: Partial<AutoUnloadConfig>): void {
    const configToMerge = { ...config };

    // Validate and clamp maxSimultaneousModels to minimum of 1
    if (configToMerge.maxSimultaneousModels !== undefined) {
      configToMerge.maxSimultaneousModels = Math.max(1, configToMerge.maxSimultaneousModels);
    }

    this.autoUnloadConfig = { ...this.autoUnloadConfig, ...configToMerge };

    // Restart monitoring if config changed
    if (this.autoUnloadConfig.enabled) {
      this.startMonitoring();
    } else {
      this.stopMonitoring();
    }

    console.log('Auto-unload configuration updated:', configToMerge);
  }

  /**
   * Get current auto-unload configuration
   */
  getAutoUnloadConfig(): AutoUnloadConfig {
    return { ...this.autoUnloadConfig };
  }

  /**
   * Get model status
   */
  async getModelStatus(modelId: string): Promise<ModelInfo> {
    try {
      const model = await invoke<ModelInfo>('get_model_status', { modelId });
      const status = this.mapStatus(model.status);

      if ((model as any).is_loaded) {
        this.loadedModels.set(modelId, { ...model, status });
      } else {
        this.loadedModels.delete(modelId);
      }

      return { ...model, status };
    } catch (error) {
      console.error('Failed to get model status:', error);
      throw new Error(`Failed to get model status: ${error}`);
    }
  }

  /**
   * Download a model
   */
  async downloadModel(
    modelType: 'CodeLlama' | 'StarCoder',
    modelSize: 'Small' | 'Medium' | 'Large',
    destinationPath?: string,
    forceDownload = false
  ): Promise<string> {
    try {
      const request = {
        modelType,
        modelSize,
        destinationPath,
        forceDownload,
      };

      const downloadId = await invoke<string>('download_model', { request });
      console.log(`Started model download: ${downloadId}`);

      return downloadId;
    } catch (error) {
      console.error('Failed to start model download:', error);
      throw new Error(`Failed to download model: ${error}`);
    }
  }

  /**
   * Validate model configuration
   */
  async validateModelConfig(request: ModelLoadingRequest): Promise<string[]> {
    try {
      const warnings = await invoke<string[]>('validate_model_config', { request });
      return warnings;
    } catch (error) {
      console.error('Failed to validate model config:', error);
      throw new Error(`Failed to validate model config: ${error}`);
    }
  }

  /**
   * Start fine-tuning job
   */
  async startFineTuneJob(request: FineTuningRequest): Promise<string> {
    try {
      const jobId = await invoke<string>('start_finetune_job', { request });
      console.log(`Started fine-tuning job: ${jobId}`);
      return jobId;
    } catch (error) {
      console.error('Failed to start fine-tuning job:', error);
      throw new Error(`Failed to start fine-tuning job: ${error}`);
    }
  }

  /**
   * Get fine-tuning job progress
   */
  async getFineTuneProgress(jobId: string): Promise<FineTuneJobInfo> {
    try {
      const job = await invoke<FineTuneJobInfo>('get_finetune_progress', { jobId });
      return {
        ...job,
        status: this.mapTrainingStatus(job.status),
      };
    } catch (error) {
      console.error('Failed to get fine-tuning progress:', error);
      throw new Error(`Failed to get fine-tuning progress: ${error}`);
    }
  }

  /**
   * Cancel fine-tuning job
   */
  async cancelFineTuneJob(jobId: string): Promise<void> {
    try {
      await invoke('cancel_finetune_job', { jobId });
      console.log(`Cancelled fine-tuning job: ${jobId}`);
    } catch (error) {
      console.error('Failed to cancel fine-tuning job:', error);
      throw new Error(`Failed to cancel fine-tuning job: ${error}`);
    }
  }

  /**
   * List fine-tuning jobs
   */
  async listFineTuneJobs(): Promise<FineTuneJobInfo[]> {
    try {
      const jobs = await invoke<FineTuneJobInfo[]>('list_finetune_jobs');
      return jobs.map((job) => ({
        ...job,
        status: this.mapTrainingStatus(job.status),
      }));
    } catch (error) {
      console.error('Failed to list fine-tuning jobs:', error);
      throw new Error(`Failed to list fine-tuning jobs: ${error}`);
    }
  }

  /**
   * Prepare dataset for fine-tuning
   */
  async prepareDataset(
    sourcePaths: string[],
    outputPath: string,
    taskType: 'CodeCompletion' | 'ErrorCorrection' | 'Documentation',
    filters?: Partial<DatasetFilters>
  ): Promise<string> {
    try {
      const defaultFilters: DatasetFilters = {
        minFileSize: 100,
        maxFileSize: 100000,
        allowedExtensions: ['rs'],
        qualityThreshold: 0.8,
        includeTests: false,
        maxSamples: undefined,
        ...filters,
      };

      const request: DatasetPreparationRequest = {
        sourcePaths,
        outputPath,
        taskType,
        filters: defaultFilters,
      };

      const taskId = await invoke<string>('prepare_dataset', { request });
      console.log(`Started dataset preparation: ${taskId}`);

      return taskId;
    } catch (error) {
      console.error('Failed to prepare dataset:', error);
      throw new Error(`Failed to prepare dataset: ${error}`);
    }
  }

  /**
   * Get resource status
   */
  async getResourceStatus(): Promise<ResourceStatus> {
    try {
      const status = await invoke<ResourceStatus>('get_resource_status');
      return status;
    } catch (error) {
      console.error('Failed to get resource status:', error);
      throw new Error(`Failed to get resource status: ${error}`);
    }
  }

  /**
   * Check if a model is loaded
   */
  isModelLoaded(modelId: string): boolean {
    const model = this.loadedModels.get(modelId);
    return model !== undefined && (model as any).is_loaded && model.status === 'Loaded';
  }

  /**
   * Get loaded model info
   */
  getLoadedModel(modelId: string): ModelInfo | undefined {
    return this.loadedModels.get(modelId);
  }

  /**
   * Get all loaded models
   */
  getAllLoadedModels(): ModelInfo[] {
    return Array.from(this.loadedModels.values()).filter((model) => (model as any).is_loaded);
  }

  /**
   * Get total memory usage
   */
  getTotalMemoryUsage(): number {
    return Array.from(this.loadedModels.values())
      .filter((model) => (model as any).is_loaded)
      .reduce((total, model) => total + ((model as any).memory_usage_mb || 0), 0);
  }

  // =============== MODEL QUANTIZATION METHODS ===============

  /**
   * Quantize a model for efficiency
   */
  async quantizeModel(
    modelId: string,
    quantizationConfig: QuantizationConfig,
    outputPath?: string
  ): Promise<ModelInfo> {
    try {
      console.log(
        `Starting quantization of model ${modelId} to ${quantizationConfig.targetPrecision}`
      );

      const request = {
        modelId,
        quantizationConfig,
        outputPath,
      };

      const quantizedModel = await invoke<ModelInfo>('quantize_model', { request });

      // Update memory usage estimates
      const originalModel = this.loadedModels.get(modelId);
      if (originalModel) {
        const memoryReduction = this.calculateQuantizationMemoryReduction(
          originalModel,
          quantizationConfig
        );
        console.log(`Estimated memory reduction: ${memoryReduction}%`);
      }

      console.log(`Successfully quantized model: ${quantizedModel.id}`);
      return { ...quantizedModel, status: this.mapStatus(quantizedModel.status) };
    } catch (error) {
      console.error('Failed to quantize model:', error);
      throw new Error(`Failed to quantize model: ${error}`);
    }
  }

  /**
   * Get supported quantization options
   */
  async getQuantizationOptions(modelId: string): Promise<QuantizationOption[]> {
    try {
      const options = await invoke<QuantizationOption[]>('get_quantization_options', { modelId });
      return options;
    } catch (error) {
      console.error('Failed to get quantization options:', error);
      throw new Error(`Failed to get quantization options: ${error}`);
    }
  }

  /**
   * Validate quantization configuration
   */
  async validateQuantizationConfig(
    modelId: string,
    quantizationConfig: QuantizationConfig
  ): Promise<ValidationResult> {
    try {
      const validation = await invoke<ValidationResult>('validate_quantization_config', {
        modelId,
        quantizationConfig,
      });
      return validation;
    } catch (error) {
      console.error('Failed to validate quantization config:', error);
      throw new Error(`Failed to validate quantization config: ${error}`);
    }
  }

  /**
   * Calculate expected memory reduction from quantization
   */
  private calculateQuantizationMemoryReduction(
    model: ModelInfo,
    config: QuantizationConfig
  ): number {
    const originalMemoryMb = (model as any).memory_usage_mb || 0;

    if (!originalMemoryMb || !model.modelSize) {
      return 0;
    }

    // Estimate reduction based on precision
    let reductionFactor: number;

    switch (config.targetPrecision) {
      case 'Int4':
        reductionFactor = 0.75; // 4-bit vs 32-bit
        break;
      case 'Int8':
        reductionFactor = 0.875; // 8-bit vs 32-bit
        break;
      case 'GPTQ':
        reductionFactor = 0.6; // GPTQ often achieves higher compression
        break;
      default:
        reductionFactor = 1.0;
    }

    const reducedMemoryMb = originalMemoryMb * reductionFactor;
    const reductionPercentage = Math.round(
      ((originalMemoryMb - reducedMemoryMb) / originalMemoryMb) * 100
    );

    return reductionPercentage;
  }

  // =============== MODEL VERSIONING METHODS ===============

  /**
   * Start experiment for model versioning
   */
  startExperiment(
    name: string,
    baseModel: string,
    hyperparameters: Record<string, any>,
    description?: string
  ): string {
    return this.experimentTracker.startExperiment(name, baseModel, hyperparameters, description);
  }

  /**
   * Add model version to experiment
   */
  addModelVersion(
    experimentId: string,
    modelId: string,
    baseModel: string,
    hyperparameters: Record<string, any>,
    metrics?: Record<string, any>,
    tags?: string[],
    notes?: string
  ): void {
    this.experimentTracker.addModelVersion(
      experimentId,
      modelId,
      baseModel,
      hyperparameters,
      metrics,
      tags,
      notes
    );
  }

  /**
   * End experiment
   */
  endExperiment(experimentId: string, status: 'completed' | 'failed' = 'completed'): void {
    this.experimentTracker.endExperiment(experimentId, status);
  }

  /**
   * Rollback to model version
   */
  async rollbackModel(request: any): Promise<ModelInfo> {
    return this.experimentTracker.rollbackModel(request);
  }

  /**
   * Get experiment details
   */
  getExperiment(experimentId: string): any {
    return this.experimentTracker.getExperiment(experimentId);
  }

  /**
   * Get model versions
   */
  getModelVersions(modelId: string): any[] {
    return this.experimentTracker.getModelVersions(modelId);
  }

  /**
   * Compare model versions
   */
  compareVersions(modelId: string, versionId1: string, versionId2: string): any {
    return this.experimentTracker.compareVersions(modelId, versionId1, versionId2);
  }

  // =============== HYPERPARAMETER TUNING METHODS ===============

  /**
   * Start hyperparameter tuning
   */
  async startTuning(config: any): Promise<string> {
    return this.tuningService.startTuning(config);
  }

  /**
   * Get tuning results
   */
  getTuningResult(tuningId: string): any {
    return this.tuningService.getTuningResult(tuningId);
  }

  /**
   * Stop tuning process
   */
  stopTuning(tuningId: string): void {
    this.tuningService.stopTuning(tuningId);
  }

  // =============== FEDERATED LEARNING METHODS ===============

  /**
   * Start federated learning session
   */
  async startFederatedSession(
    modelId: string,
    datasetIds: string[],
    config?: any
  ): Promise<string> {
    return this.federatedLearningManager.startFederatedSession(modelId, datasetIds, config);
  }

  /**
   * Receive client update for federated learning
   */
  async receiveClientUpdate(clientId: string, modelUpdate: any, localMetrics?: any): Promise<void> {
    return this.federatedLearningManager.receiveClientUpdate(clientId, modelUpdate, localMetrics);
  }

  /**
   * Get federated learning status
   */
  getFederatedStatus(sessionId: string): any {
    return this.federatedLearningManager.getFederatedStatus(sessionId);
  }

  /**
   * Stop federated learning session
   */
  async stopFederatedSession(sessionId: string): Promise<void> {
    return this.federatedLearningManager.stopFederatedSession(sessionId);
  }

  /**
   * Configure federated learning
   */
  updateFederatedConfig(config: any): void {
    this.federatedLearningManager.updateFederatedConfig(config);
  }

  /**
   * Export federated learning results
   */
  exportFederatedResults(sessionId: string): any {
    return this.federatedLearningManager.exportFederatedResults(sessionId);
  }

  /**
   * Map string status to enum
   */
  private mapStatus(status: string): ModelStatus {
    switch (status) {
      case 'Available':
        return 'Available';
      case 'Downloading':
        return 'Downloading';
      case 'Downloaded':
        return 'Downloaded';
      case 'Loading':
        return 'Loading';
      case 'Loaded':
        return 'Loaded';
      case 'Unloading':
        return 'Unloading';
      case 'Error':
        return 'Error';
      default:
        return 'Error';
    }
  }

  /**
   * Map string training status to enum
   */
  private mapTrainingStatus(status: string): TrainingStatus {
    switch (status) {
      case 'Created':
        return 'Created';
      case 'Initializing':
        return 'Initializing';
      case 'PreparingData':
        return 'PreparingData';
      case 'Training':
        return 'Training';
      case 'Evaluating':
        return 'Evaluating';
      case 'Saving':
        return 'Saving';
      case 'Completed':
        return 'Completed';
      case 'Failed':
        return 'Failed';
      case 'Cancelled':
        return 'Cancelled';
      default:
        return 'Failed';
    }
  }
}

// Singleton instance
const modelService = ModelService.getInstance();

export default modelService;
