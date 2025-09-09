/**
 * Validator functions and utilities for training configurations
 * Handles validation of fine-tuning parameters and form data
 */

export interface ValidationError {
  field: string;
  message: string;
}

export interface TrainingConfigValidation {
  jobName?: string;
  description?: string;
  datasetPath?: string;
  learningRate?: number;
  batchSize?: number;
  maxEpochs?: number;
}

/**
 * Validates fine-tuning job configuration
 *
 * @param config - The training configuration to validate
 * @returns Array of validation errors, empty if valid
 */
export function validateFineTuningJob(config: TrainingConfigValidation): ValidationError[] {
  const errors: ValidationError[] = [];

  // Validate job name
  if (!config.jobName?.trim()) {
    errors.push({
      field: 'jobName',
      message: 'Job name is required'
    });
  } else if (config.jobName.length < 3) {
    errors.push({
      field: 'jobName',
      message: 'Job name must be at least 3 characters'
    });
  }

  // Validate description
  if (!config.description?.trim()) {
    errors.push({
      field: 'description',
      message: 'Description is required'
    });
  }

  // Validate dataset path
  if (!config.datasetPath?.trim()) {
    errors.push({
      field: 'datasetPath',
      message: 'Dataset path is required'
    });
  } else {
    // Basic validation for dataset path
    const path = config.datasetPath.trim();
    if (!path.endsWith('.jsonl') && !path.endsWith('.json')) {
      errors.push({
        field: 'datasetPath',
        message: 'Dataset must be in JSONL or JSON format'
      });
    }
  }

  // Validate learning rate
  if (config.learningRate !== undefined && (
    config.learningRate <= 0 || config.learningRate > 1e-2
  )) {
    errors.push({
      field: 'learningRate',
      message: 'Learning rate must be between 0 and 0.01'
    });
  }

  // Validate batch size
  if (config.batchSize !== undefined && (
    config.batchSize < 1 || config.batchSize > 64
  )) {
    errors.push({
      field: 'batchSize',
      message: 'Batch size must be between 1 and 64'
    });
  }

  // Validate max epochs
  if (config.maxEpochs !== undefined && (
    config.maxEpochs < 1 || config.maxEpochs > 20
  )) {
    errors.push({
      field: 'maxEpochs',
      message: 'Maximum epochs must be between 1 and 20'
    });
  }

  return errors;
}

/**
 * Validates if a path is a valid dataset
 *
 * @param path - File path to validate
 * @returns Whether the path is valid
 */
export function validateDatasetPath(path: string): boolean {
  if (!path?.trim()) return false;
  const trimmed = path.trim();
  return trimmed.endsWith('.jsonl') || trimmed.endsWith('.json');
}

/**
 * Validates numeric constraints
 *
 * @param value - Value to validate
 * @param constraints - Min and max constraints
 * @returns Whether the value is valid
 */
export function validateNumeric(value: number, { min, max }: { min: number; max: number }): boolean {
  return value >= min && value <= max;
}