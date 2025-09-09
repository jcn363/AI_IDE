// Experiment Tracker for Model Versioning and Rollback
import type { FineTuningRequest, FineTuneJobInfo, ModelInfo } from '../types';

export interface ModelVersionEntry {
  versionId: string;
  modelId: string;
  baseModel: string;
  createdAt: string;
  hyperparameters?: Record<string, any>;
  metrics?: Record<string, any>;
  experimentId: string;
  tags?: string[];
  notes?: string;
  isActive: boolean;
}

export interface Experiment {
  id: string;
  name: string;
  description?: string;
  baseModel: string;
  startDate: string;
  endDate?: string;
  status: 'running' | 'completed' | 'failed';
  hyperparameters: Record<string, any>;
  metrics?: Record<string, any>;
  versions: ModelVersionEntry[];
}

export interface RollbackRequest {
  modelId: string;
  targetVersion: string;
  reason?: string;
  createBackup: boolean;
}

export class ExperimentTracker {
  private experiments: Map<string, Experiment> = new Map();

  /**
   * Start a new experiment
   */
  startExperiment(
    name: string,
    baseModel: string,
    hyperparameters: Record<string, any>,
    description?: string
  ): string {
    const experimentId = `exp_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    const experiment: Experiment = {
      id: experimentId,
      name,
      description,
      baseModel,
      startDate: new Date().toISOString(),
      status: 'running',
      hyperparameters: { ...hyperparameters },
      versions: [],
    };

    this.experiments.set(experimentId, experiment);
    console.log(`Started experiment: ${experimentId}`);
    return experimentId;
  }

  /**
   * Add a version to an experiment
   */
  addModelVersion(
    experimentId: string,
    modelId: string,
    baseModel: string,
    hyperparameters: Record<string, any>,
    metrics?: Record<string, any>,
    tags?: string[],
    notes?: string
  ): ModelVersionEntry {
    const experiment = this.experiments.get(experimentId);
    if (!experiment) {
      throw new Error(`Experiment ${experimentId} not found`);
    }

    const versionId = `v${experiment.versions.length + 1}_${Date.now()}`;
    const versionEntry: ModelVersionEntry = {
      versionId,
      modelId,
      baseModel,
      createdAt: new Date().toISOString(),
      hyperparameters: { ...hyperparameters },
      metrics,
      experimentId,
      tags,
      notes,
      isActive: true,
    };

    experiment.versions.push(versionEntry);

    // Mark previous versions as not active
    experiment.versions.forEach((v, index) => {
      if (index < experiment.versions.length - 1) {
        v.isActive = false;
      }
    });

    console.log(`Added model version: ${versionId} to experiment: ${experimentId}`);
    return versionEntry;
  }

  /**
   * End an experiment
   */
  endExperiment(experimentId: string, status: 'completed' | 'failed' = 'completed'): void {
    const experiment = this.experiments.get(experimentId);
    if (!experiment) {
      throw new Error(`Experiment ${experimentId} not found`);
    }

    experiment.status = status;
    experiment.endDate = new Date().toISOString();
    console.log(`Ended experiment: ${experimentId} with status: ${status}`);
  }

  /**
   * Rollback to a specific model version
   */
  async rollbackModel(request: RollbackRequest): Promise<ModelInfo> {
    console.log(`Rolling back model ${request.modelId} to version ${request.targetVersion}`);

    if (request.createBackup) {
      await this.createBackup(request.modelId);
    }

    try {
      const rollbackResult = await globalThis.__TAURI_INTERNALS__
        //@ts-ignore
        .invoke<ModelInfo>('rollback_model', {
          modelId: request.modelId,
          targetVersion: request.targetVersion,
          reason: request.reason || 'Rollback requested',
        });

      console.log(`Successfully rolled back model ${request.modelId} to version ${request.targetVersion}`);
      return rollbackResult;
    } catch (error) {
      console.error('Failed to rollback model:', error);
      throw new Error(`Failed to rollback model: ${error}`);
    }
  }

  /**
   * Get experiment details
   */
  getExperiment(experimentId: string): Experiment | undefined {
    return this.experiments.get(experimentId);
  }

  /**
   * List all experiments
   */
  listExperiments(): Experiment[] {
    return Array.from(this.experiments.values());
  }

  /**
   * Get versions for a model
   */
  getModelVersions(modelId: string): ModelVersionEntry[] {
    const allVersions: ModelVersionEntry[] = [];
    for (const experiment of this.experiments.values()) {
      allVersions.push(...experiment.versions.filter(v => v.modelId === modelId));
    }
    return allVersions.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
  }

  /**
   * Get active version for a model
   */
  getActiveVersion(modelId: string): ModelVersionEntry | undefined {
    return this.getModelVersions(modelId).find(v => v.isActive);
  }

  /**
   * Create backup before rollback
   */
  private async createBackup(modelId: string): Promise<void> {
    try {
      const backupId = await globalThis.__TAURI_INTERNALS__
        //@ts-ignore
        .invoke<string>('create_model_backup', { modelId });

      console.log(`Created backup ${backupId} for model ${modelId}`);
    } catch (error) {
      console.warn('Failed to create backup before rollback:', error);
      throw error;
    }
  }

  /**
   * Compare model versions
   */
  compareVersions(modelId: string, versionId1: string, versionId2: string): {
    version1: ModelVersionEntry;
    version2: ModelVersionEntry;
    differences: string[];
  } {
    const versions = this.getModelVersions(modelId);
    const version1 = versions.find(v => v.versionId === versionId1);
    const version2 = versions.find(v => v.versionId === versionId2);

    if (!version1 || !version2) {
      throw new Error('One or both versions not found');
    }

    const differences: string[] = [];

    // Compare hyperparameters
    const hp1 = version1.hyperparameters || {};
    const hp2 = version2.hyperparameters || {};

    for (const key of new Set([...Object.keys(hp1), ...Object.keys(hp2)])) {
      if (hp1[key] !== hp2[key]) {
        differences.push(`Hyperparameter ${key}: ${hp1[key] || 'undefined'} -> ${hp2[key] || 'undefined'}`);
      }
    }

    // Compare metrics
    const metrics1 = version1.metrics || {};
    const metrics2 = version2.metrics || {};

    for (const key of new Set([...Object.keys(metrics1), ...Object.keys(metrics2)])) {
      if (metrics1[key] !== metrics2[key]) {
        differences.push(`Metric ${key}: ${metrics1[key] || 'undefined'} -> ${metrics2[key] || 'undefined'}`);
      }
    }

    return { version1, version2, differences };
  }
}