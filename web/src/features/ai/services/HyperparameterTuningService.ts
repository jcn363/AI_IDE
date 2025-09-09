// Hyperparameter Tuning Service for automated model optimization
import type { FineTuningRequest, TrainingMetrics } from '../types';

export interface HyperparameterSpace {
  learningRate?: { min: number; max: number; type: 'log' | 'linear' };
  batchSize?: { values: number[] };
  epochs?: { min: number; max: number; type: 'int' };
  weightDecay?: { min: number; max: number; type: 'log' };
  warmupSteps?: { min: number; max: number; type: 'int' };
  maxGradNorm?: { min: number; max: number; type: 'linear' };
  loraRank?: { values: number[] };
  loraAlpha?: { min: number; max: number; type: 'linear' };
}

export interface TuningConfiguration {
  modelId: string;
  hyperparameterSpace: HyperparameterSpace;
  optimizationTarget?: 'loss' | 'accuracy' | 'f1' | 'precision' | 'recall';
  maxTrials: number;
  concurrentTrials: number;
  timeoutMinutes?: number;
  budgetHours?: number;
}

export interface TuningTrial {
  trialId: string;
  config: TuningConfiguration;
  hyperparameters: Record<string, any>;
  status: 'pending' | 'running' | 'completed' | 'failed';
  startTime?: string;
  endTime?: string;
  metrics?: TrainingMetrics;
  score?: number;
  errorMessage?: string;
}

export interface TuningResult {
  bestTrial: TuningTrial;
  allTrials: TuningTrial[];
  optimizationTarget: string;
  improvement: number;
  recommendations: string[];
}

export class HyperparameterTuningService {
  private activeTunings: Map<string, TuningConfiguration> = new Map();
  private trials: Map<string, TuningTrial[]> = new Map();

  /**
   * Start hyperparameter tuning for a model
   */
  async startTuning(config: TuningConfiguration): Promise<string> {
    const tuningId = `tuning_${config.modelId}_${Date.now()}`;

    console.log(`Starting hyperparameter tuning for model ${config.modelId}`);

    // Generate initial trial suggestions
    const initialTrials = this.generateInitialTrials(config);

    // Store tuning configuration
    this.activeTunings.set(tuningId, config);
    this.trials.set(tuningId, initialTrials);

    // Start execution of trials
    await this.executeTrials(tuningId);

    return tuningId;
  }

  /**
   * Generate initial hyperparameter trials
   */
  private generateInitialTrials(config: TuningConfiguration): TuningTrial[] {
    const trials: TuningTrial[] = [];

    // Generate trial using grid search, random search, or Bayesian optimization
    for (let i = 0; i < config.maxTrials; i++) {
      const hyperparameters = this.sampleHyperparameters(config.hyperparameterSpace);

      const trial: TuningTrial = {
        trialId: `trial_${i + 1}`,
        config,
        hyperparameters,
        status: 'pending',
      };

      trials.push(trial);
    }

    return trials;
  }

  /**
   * Sample hyperparameters from the search space
   */
  private sampleHyperparameters(space: HyperparameterSpace): Record<string, any> {
    const params: Record<string, any> = {};

    if (space.learningRate) {
      params.learningRate = this.sampleContinuous(space.learningRate.min, space.learningRate.max, space.learningRate.type);
    }

    if (space.batchSize) {
      params.batchSize = this.sampleDiscrete(space.batchSize.values);
    }

    if (space.epochs) {
      params.maxEpochs = this.sampleInteger(space.epochs.min, space.epochs.max);
    }

    if (space.weightDecay) {
      params.weightDecay = this.sampleContinuous(space.weightDecay.min, space.weightDecay.max, space.weightDecay.type);
    }

    if (space.warmupSteps) {
      params.warmupSteps = this.sampleInteger(space.warmupSteps.min, space.warmupSteps.max);
    }

    if (space.maxGradNorm) {
      params.maxGradNorm = this.sampleContinuous(space.maxGradNorm.min, space.maxGradNorm.max, space.maxGradNorm.type);
    }

    if (space.loraRank) {
      params.loraRank = this.sampleDiscrete(space.loraRank.values);
    }

    if (space.loraAlpha) {
      params.loraAlpha = this.sampleContinuous(space.loraAlpha.min, space.loraAlpha.max, space.loraAlpha.type);
    }

    return params;
  }

  /**
   * Sample continuous value
   */
  private sampleContinuous(min: number, max: number, scale: 'log' | 'linear' = 'linear'): number {
    const random = Math.random();

    if (scale === 'log') {
      // Logarithmic sampling
      const logMin = Math.log10(min);
      const logMax = Math.log10(max);
      const sampledLog = logMin + random * (logMax - logMin);
      return Math.pow(10, sampledLog);
    }

    // Linear sampling
    return min + random * (max - min);
  }

  /**
   * Sample discrete value
   */
  private sampleDiscrete(values: number[]): number {
    const index = Math.floor(Math.random() * values.length);
    return values[index];
  }

  /**
   * Sample integer value
   */
  private sampleInteger(min: number, max: number): number {
    return Math.floor(min + Math.random() * (max - min + 1));
  }

  /**
   * Execute trials concurrently
   */
  private async executeTrials(tuningId: string): Promise<void> {
    const config = this.activeTunings.get(tuningId);
    const trialList = this.trials.get(tuningId);

    if (!config || !trialList) {
      throw new Error(`Tuning ${tuningId} not found`);
    }

    const concurrentLimit = config.concurrentTrials;
    const executing: Promise<void>[] = [];

    for (const trial of trialList) {
      while (executing.length >= concurrentLimit) {
        await Promise.race(executing);
        executing.splice(executing.findIndex(p => p === Promise.race(executing)), 1);
      }

      const trialPromise = this.executeSingleTrial(tuningId, trial);
      executing.push(trialPromise);

      // Check if we need to wait before starting next trial
      if (executing.length >= concurrentLimit) {
        await Promise.race(executing);
      }
    }

    // Wait for remaining trials
    await Promise.all(executing);
  }

  /**
   * Execute a single trial
   */
  private async executeSingleTrial(tuningId: string, trial: TuningTrial): Promise<void> {
    trial.status = 'running';
    trial.startTime = new Date().toISOString();

    try {
      // Prepare fine-tuning request
      const fineTuneRequest: FineTuningRequest = {
        jobName: `${tuningId}_${trial.trialId}`,
        description: `Hyperparameter tuning trial`,
        baseModel: trial.config.modelId,
        datasetPath: '', // This should be provided in the tuning configuration
        config: {
          learningRate: trial.hyperparameters.learningRate,
          batchSize: trial.hyperparameters.batchSize,
          maxEpochs: trial.hyperparameters.maxEpochs,
          mixedPrecision: true,
          maxSeqLength: 512,
          loraRank: trial.hyperparameters.loraRank,
        },
        enableMonitoring: true,
      };

      // Start the fine-tuning job (delegate to backend)
      await this.startFineTuningJob(fineTuneRequest);

      // Monitor progress and wait for completion
      await this.monitorTrialProgress(trial);

      // Evaluate results
      await this.evaluateTrial(trial);

      trial.status = 'completed';
      trial.endTime = new Date().toISOString();

    } catch (error) {
      trial.status = 'failed';
      trial.errorMessage = error instanceof Error ? error.message : 'Unknown error';
      trial.endTime = new Date().toISOString();
      console.error(`Trial ${trial.trialId} failed:`, error);
    }
  }

  /**
   * Start fine-tuning job via backend
   */
  private async startFineTuningJob(request: FineTuningRequest): Promise<string> {
    try {
      // This would be a backend call to actually start the fine-tuning
      return await globalThis.__TAURI_INTERNALS__
        //@ts-ignore
        .invoke<string>('start_finetune_job', { request });
    } catch (error) {
      console.error('Failed to start fine-tuning job:', error);
      throw error;
    }
  }

  /**
   * Monitor trial progress
   */
  private async monitorTrialProgress(trial: TuningTrial): Promise<void> {
    const maxWaitTime = 30 * 60 * 1000; // 30 minutes
    const startTime = Date.now();

    while (Date.now() - startTime < maxWaitTime) {
      try {
        // Check if the job is still running
        const status = await globalThis.__TAURI_INTERNALS__
          //@ts-ignore
          .invoke('get_finetune_progress', { jobId: `${trial.config.modelId}_${trial.trialId}` });

        if (status.status === 'Completed' || status.status === 'Failed') {
          break;
        }

        // Wait before checking again
        await new Promise(resolve => setTimeout(resolve, 5000)); // 5 seconds
      } catch (error) {
        console.warn('Error checking trial progress:', error);
        break;
      }
    }
  }

  /**
   * Evaluate trial results and assign score
   */
  private async evaluateTrial(trial: TuningTrial): Promise<void> {
    try {
      // Get training metrics from backend
      const metrics = await globalThis.__TAURI_INTERNALS__
        //@ts-ignore
        .invoke<TrainingMetrics>('get_trial_metrics', {
          trialId: trial.trialId,
          jobName: `${trial.config.modelId}_${trial.trialId}`
        });

      trial.metrics = metrics;

      // Calculate score based on optimization target
      const target = trial.config.optimizationTarget || 'loss';
      switch (target) {
        case 'loss':
          trial.score = -metrics.finalLoss; // Negative because lower is better for loss
          break;
        case 'accuracy':
          trial.score = metrics.bleuScore || 0; // Use BLEU score as accuracy proxy
          break;
        case 'f1':
        case 'precision':
        case 'recall':
          // These would require more specific metrics
          trial.score = metrics.codeBleuScore || 0;
          break;
        default:
          trial.score = -metrics.finalLoss;
      }

    } catch (error) {
      console.error(`Failed to evaluate trial ${trial.trialId}:`, error);
      // Assign a default low score for failed evaluation
      trial.score = -999;
    }
  }

  /**
   * Get tuning status and results
   */
  getTuningResult(tuningId: string): TuningResult | null {
    const config = this.activeTunings.get(tuningId);
    const trialList = this.trials.get(tuningId);

    if (!config || !trialList) {
      return null;
    }

    const completedTrials = trialList.filter(t => t.status === 'completed' && t.score !== undefined);
    if (completedTrials.length === 0) {
      return null;
    }

    const bestTrial = completedTrials.reduce((best, current) =>
      (current.score || 0) > (best.score || 0) ? current : best
    );

    const averageScore = completedTrials.reduce((sum, t) => sum + (t.score || 0), 0) / completedTrials.length;
    const improvement = (bestTrial.score || 0) - averageScore;

    const recommendations = this.generateRecommendations(bestTrial, completedTrials);

    return {
      bestTrial,
      allTrials: trialList,
      optimizationTarget: config.optimizationTarget || 'loss',
      improvement,
      recommendations,
    };
  }

  /**
   * Generate recommendations based on tuning results
   */
  private generateRecommendations(bestTrial: TuningTrial, allTrials: TuningTrial[]): string[] {
    const recommendations: string[] = [];

    // Recommend optimal hyperparameters
    recommendations.push('Use the following hyperparameters for production:');
    for (const [key, value] of Object.entries(bestTrial.hyperparameters || {})) {
      recommendations.push(`  ${key}: ${value}`);
    }

    // Analyze parameter importance
    const importantParams = this.analyzeParameterImportance(allTrials);
    if (importantParams.length > 0) {
      recommendations.push('\nMost important parameters:');
      importantParams.forEach(param => {
        recommendations.push(`  ${param.name}: affects performance by ~${param.impact}%`);
      });
    }

    return recommendations;
  }

  /**
   * Analyze which parameters have the most impact on performance
   */
  private analyzeParameterImportance(trials: TuningTrial[]): Array<{ name: string; impact: number }> {
    const params: Record<string, Array<{ value: any; score: number }>> = {};

    trials.forEach(trial => {
      if (trial.score !== undefined) {
        for (const [key, value] of Object.entries(trial.hyperparameters || {})) {
          if (!params[key]) {
            params[key] = [];
          }
          params[key].push({ value, score: trial.score });
        }
      }
    });

    const importanceScores: Array<{ name: string; impact: number }> = [];

    for (const [paramName, values] of Object.entries(params)) {
      if (values.length < 3) continue; // Need at least 3 samples

      // Calculate correlation between parameter value and score
      // This is a simplified implementation
      const correlation = this.calculateCorrelation(values);
      importanceScores.push({
        name: paramName,
        impact: Math.abs(correlation) * 100,
      });
    }

    return importanceScores.sort((a, b) => b.impact - a.impact).slice(0, 5);
  }

  /**
   * Calculate correlation coefficient between values and scores
   */
  private calculateCorrelation(data: Array<{ value: any; score: number }>): number {
    const n = data.length;

    // Convert continuous values to numbers for correlation
    const numericValues = data.map(d => {
      if (typeof d.value === 'number') {
        return d.value;
      }
      // For categorical/discrete values, use index
      return data.findIndex(item => item.value === d.value);
    });

    const scores = data.map(d => d.score);

    const sumX = numericValues.reduce((a, b) => a + b, 0);
    const sumY = scores.reduce((a, b) => a + b, 0);
    const sumXY = numericValues.reduce((sum, x, i) => sum + x * scores[i], 0);
    const sumX2 = numericValues.reduce((sum, x) => sum + x * x, 0);
    const sumY2 = scores.reduce((sum, y) => sum + y * y, 0);

    const numerator = n * sumXY - sumX * sumY;
    const denominator = Math.sqrt((n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY));

    return denominator === 0 ? 0 : numerator / denominator;
  }

  /**
   * Stop tuning process
   */
  stopTuning(tuningId: string): void {
    console.log(`Stopping hyperparameter tuning: ${tuningId}`);
    this.activeTunings.delete(tuningId);
    // Implementation would also need to cancel running trials
  }
}