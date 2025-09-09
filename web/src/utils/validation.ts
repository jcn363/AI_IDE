/**
 * Common validation utilities for forms and data processing
 */

/**
 * Validates a Cargo package name
 */
export function isValidPackageName(name: string): boolean {
  // Cargo package names must follow specific rules
  const packageNameRegex = /^[a-zA-Z][a-zA-Z0-9_-]*$/;
  return packageNameRegex.test(name) && name.length <= 64 && name.length >= 2;
}

/**
 * Validates a semantic version string
 */
export function isValidVersion(version: string): boolean {
  // Simple semver validation (could use a proper semver library in production)
  const semverRegex = /^\d+\.\d+\.\d+(-[\w\d]+)?(\+[\w\d]+)?$/;
  return semverRegex.test(version);
}

/**
 * Validates a file path for dataset files
 */
export function isValidDatasetPath(path: string): boolean {
  if (!path) return false;
  // Allow common dataset formats
  return path.endsWith('.jsonl') || path.endsWith('.json') || path.endsWith('.csv');
}

/**
 * Validates learning rate range
 */
export function isValidLearningRate(rate: number): boolean {
  return rate > 0 && rate <= 1e-2;
}

/**
 * Validates batch size range
 */
export function isValidBatchSize(size: number): boolean {
  return Number.isInteger(size) && size >= 1 && size <= 64;
}

/**
 * Validates maximum epochs
 */
export function isValidMaxEpochs(epochs: number): boolean {
  return Number.isInteger(epochs) && epochs >= 1 && epochs <= 20;
}

/**
 * Validates sequence length
 */
export function isValidSequenceLength(length: number): boolean {
  return Number.isInteger(length) && length >= 512 && length <= 8192;
}

/**
 * Validates LoRA rank
 */
export function isValidLoraRank(rank: number): boolean {
  return Number.isInteger(rank) && rank >= 4 && rank <= 64;
}

/**
 * Comprehensive form validation for fine-tuning requests
 */
export interface ValidationResult {
  isValid: boolean;
  errors: Record<string, string>;
}

export function validateFineTuneForm(data: {
  jobName: string;
  description?: string;
  datasetPath: string;
  learningRate: number;
  batchSize: number;
  maxEpochs: number;
  maxSeqLength: number;
  loraRank?: number;
}): ValidationResult {
  const errors: Record<string, string> = {};

  // Job name validation
  if (!data.jobName?.trim()) {
    errors.jobName = 'Job name is required';
  } else if (!isValidPackageName(data.jobName)) {
    errors.jobName = 'Job name must start with a letter and contain only alphanumeric characters, hyphens, or underscores';
  }

  // Description validation (optional but shouldn't be empty if provided)
  if (data.description !== undefined && !data.description.trim()) {
    errors.description = 'Description cannot be empty if provided';
  }

  // Dataset path validation
  if (!data.datasetPath?.trim()) {
    errors.datasetPath = 'Dataset path is required';
  } else if (!isValidDatasetPath(data.datasetPath)) {
    errors.datasetPath = 'Dataset must be in JSONL, JSON, or CSV format';
  }

  // Learning rate validation
  if (!isValidLearningRate(data.learningRate)) {
    errors.learningRate = 'Learning rate must be between 0 and 0.01';
  }

  // Batch size validation
  if (!isValidBatchSize(data.batchSize)) {
    errors.batchSize = 'Batch size must be between 1 and 64';
  }

  // Max epochs validation
  if (!isValidMaxEpochs(data.maxEpochs)) {
    errors.maxEpochs = 'Maximum epochs must be between 1 and 20';
  }

  // Sequence length validation
  if (!isValidSequenceLength(data.maxSeqLength)) {
    errors.maxSeqLength = 'Maximum sequence length must be between 512 and 8192';
  }

  // LoRA rank validation (if provided)
  if (data.loraRank !== undefined && !isValidLoraRank(data.loraRank)) {
    errors.loraRank = 'LoRA rank must be between 4 and 64';
  }

  return {
    isValid: Object.keys(errors).length === 0,
    errors
  };
}

/**
 * Validates configuration compatibility
 */
export function validateModelConfigCompatibility(baseModel: string, batchSize: number): string | null {
  // Example compatibility validation
  if (baseModel.includes('codellama-13b') && batchSize > 4) {
    return 'For CodeLlama 13B, batch size should not exceed 4 due to memory constraints';
  }
  if (baseModel.includes('codellama-7b') && batchSize > 8) {
    return 'For CodeLlama 7B, batch size should not exceed 8 due to memory constraints';
  }
  if (baseModel.includes('starcoder') && batchSize > 8) {
    return 'For StarCoder models, batch size should not exceed 8 due to memory constraints';
  }
  return null;
}

export default {
  isValidPackageName,
  isValidVersion,
  isValidDatasetPath,
  isValidLearningRate,
  isValidBatchSize,
  isValidMaxEpochs,
  isValidSequenceLength,
  isValidLoraRank,
  validateFineTuneForm,
  validateModelConfigCompatibility,
};