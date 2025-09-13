/**
 * Main form component for creating fine-tuning jobs
 * Integrates model selection, validation, and form submission
 */

import React, { useState } from 'react';
import type { FineTuningRequest, TrainingConfigInfo } from '../../features/ai/types';
import { ModelConfigSelector } from './ModelConfigSelector';
import type { ModelConfig } from './ModelConfigSelector';
import { validateFineTuningJob, ValidationError } from './TrainingValidators';

interface FormData {
  jobName: string;
  description: string;
  baseModelConfig: ModelConfig;
  datasetPath: string;
  learningRate: number;
  batchSize: number;
  maxEpochs: number;
  mixedPrecision: boolean;
  maxSeqLength: number;
  enableLoRA: boolean;
  loraRank: number;
}

interface FineTuneFormProps {
  onSubmit: (request: FineTuningRequest) => Promise<void>;
  onClose: () => void;
  isSubmitting: boolean;
}

/**
 * Form component for configuring and submitting fine-tuning jobs
 */
export const FineTuneForm: React.FC<FineTuneFormProps> = ({ onSubmit, onClose, isSubmitting }) => {
  const [formData, setFormData] = useState<FormData>({
    jobName: '',
    description: '',
    baseModelConfig: {
      baseModel: 'codellama-7b',
      supportsMixedPrecision: true,
      maxSeqLength: 2048,
      recommendedBatchSize: 8,
      supportsLoRA: true,
    },
    datasetPath: '',
    learningRate: 5e-5,
    batchSize: 8,
    maxEpochs: 3,
    mixedPrecision: true,
    maxSeqLength: 2048,
    enableLoRA: false,
    loraRank: 8,
  });

  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});

  const handleModelChange = (modelKey: string, config: ModelConfig) => {
    setFormData((prev) => ({
      ...prev,
      baseModelConfig: config,
      batchSize: config.recommendedBatchSize,
      maxSeqLength: config.maxSeqLength,
      mixedPrecision: config.supportsMixedPrecision,
      enableLoRA: config.supportsLoRA,
    }));
  };

  const handleFormChange = (field: keyof FormData, value: any) => {
    setFormData((prev) => ({ ...prev, [field]: value }));

    // Clear validation error for this field
    if (validationErrors[field]) {
      setValidationErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[field];
        return newErrors;
      });
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validate form
    const validation = validateFineTuningJob({
      jobName: formData.jobName,
      description: formData.description,
      datasetPath: formData.datasetPath,
      learningRate: formData.learningRate,
      batchSize: formData.batchSize,
      maxEpochs: formData.maxEpochs,
    });

    const errors: Record<string, string> = {};
    validation.forEach((v) => {
      errors[v.field] = v.message;
    });
    setValidationErrors(errors);

    if (Object.keys(errors).length === 0) {
      const request: FineTuningRequest = {
        jobName: formData.jobName.trim(),
        description: formData.description.trim(),
        baseModel: formData.baseModelConfig.baseModel,
        datasetPath: formData.datasetPath.trim(),
        config: {
          learningRate: formData.learningRate,
          batchSize: formData.batchSize,
          maxEpochs: formData.maxEpochs,
          mixedPrecision: formData.mixedPrecision,
          maxSeqLength: formData.maxSeqLength,
          loraRank: formData.enableLoRA ? formData.loraRank : undefined,
        },
        enableMonitoring: true,
      };

      try {
        await onSubmit(request);

        // Reset form
        setFormData({
          jobName: '',
          description: '',
          baseModelConfig: {
            baseModel: 'codellama-7b',
            supportsMixedPrecision: true,
            maxSeqLength: 2048,
            recommendedBatchSize: 8,
            supportsLoRA: true,
          },
          datasetPath: '',
          learningRate: 5e-5,
          batchSize: 8,
          maxEpochs: 3,
          mixedPrecision: true,
          maxSeqLength: 2048,
          enableLoRA: false,
          loraRank: 8,
        });
        setValidationErrors({});
        onClose();
      } catch (error) {
        console.error('Failed to submit:', error);
        // Error is handled by parent component
      }
    }
  };

  return (
    <form onSubmit={handleSubmit} className="fine-tune-form">
      {/* Form fields would be implemented in JSX */}
      {/* This is a placeholder structure */}
      <div className="form-section">
        <h3>Job Information</h3>
        <input
          type="text"
          placeholder="Job Name"
          value={formData.jobName}
          onChange={(e) => handleFormChange('jobName', e.target.value)}
          disabled={isSubmitting}
        />
        {validationErrors.jobName && <span className="error">{validationErrors.jobName}</span>}
      </div>

      <ModelConfigSelector
        selectedModelKey="codellama7b" // Would need to be computed from config
        onModelChange={(key, config) => handleModelChange(key, config)}
        disabled={isSubmitting}
      />

      <div className="form-section">
        <h3>Training Configuration</h3>
        {/* Training config fields */}
        <input
          type="number"
          step="1e-6"
          placeholder="Learning Rate"
          value={formData.learningRate}
          onChange={(e) => handleFormChange('learningRate', parseFloat(e.target.value))}
          disabled={isSubmitting}
        />
        {validationErrors.learningRate && (
          <span className="error">{validationErrors.learningRate}</span>
        )}
      </div>

      <div className="form-actions">
        <button type="button" onClick={onClose} disabled={isSubmitting}>
          Cancel
        </button>
        <button type="submit" disabled={isSubmitting}>
          {isSubmitting ? 'Creating...' : 'Create Fine-Tuning Job'}
        </button>
      </div>
    </form>
  );
};

export default FineTuneForm;
