import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { FineTuningRequest, TrainingConfigInfo } from '../../features/ai/types';

interface CreateFineTuneModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (request: FineTuningRequest) => Promise<void>;
  availableModels: Array<{ id: string; name: string; }>;
}

const MODEL_CONFIGS = {
  codellama7b: {
    baseModel: 'codellama-7b',
    supportsMixedPrecision: true,
    maxSeqLength: 2048,
    recommendedBatchSize: 8,
    supportsLoRA: true,
  },
  codellama13b: {
    baseModel: 'codellama-13b',
    supportsMixedPrecision: true,
    maxSeqLength: 2048,
    recommendedBatchSize: 4,
    supportsLoRA: true,
  },
  starcoder7b: {
    baseModel: 'starcoder-7b',
    supportsMixedPrecision: true,
    maxSeqLength: 4096,
    recommendedBatchSize: 8,
    supportsLoRA: true,
  },
  starcoder15b: {
    baseModel: 'starcoder-15b',
    supportsMixedPrecision: true,
    maxSeqLength: 4096,
    recommendedBatchSize: 4,
    supportsLoRA: true,
  },
} as const;

type ModelConfig = typeof MODEL_CONFIGS[keyof typeof MODEL_CONFIGS];
type ModelKey = keyof typeof MODEL_CONFIGS;

// Update the state type to use this config
interface FormState {
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

export const CreateFineTuneModal: React.FC<CreateFineTuneModalProps> = ({
  isOpen,
  onClose,
  onSubmit,
  availableModels,
}) => {
    const [formData, setFormData] = useState<FormState>({
        jobName: '',
        description: '',
        baseModelConfig: MODEL_CONFIGS.codellama7b,
        datasetPath: '',
        learningRate: 5e-5,
        batchSize: 8,
        maxEpochs: 3,
        mixedPrecision: true,
        maxSeqLength: 2048,
        enableLoRA: false,
        loraRank: 8,
      });

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});

  const handleModelChange = (modelKey: ModelKey) => {
    const config = MODEL_CONFIGS[modelKey];
    if (config) {
      setFormData(prev => ({
        ...prev,
        baseModelConfig: config,
        batchSize: config.recommendedBatchSize,
        maxSeqLength: config.maxSeqLength,
        mixedPrecision: config.supportsMixedPrecision,
        enableLoRA: config.supportsLoRA,
        loraRank: config.supportsLoRA ? prev.loraRank : 0,
      }));
    }
  };

  const validateForm = () => {
    const errors: Record<string, string> = {};

    if (!formData.jobName.trim()) {
      errors.jobName = 'Job name is required';
    } else if (formData.jobName.length < 3) {
      errors.jobName = 'Job name must be at least 3 characters';
    }

    if (!formData.description.trim()) {
      errors.description = 'Description is required';
    }

    if (!formData.datasetPath.trim()) {
      errors.datasetPath = 'Dataset path is required';
    } else {
      // Basic validation for dataset path
      const path = formData.datasetPath.trim();
      if (!path.endsWith('.jsonl') && !path.endsWith('.json')) {
        errors.datasetPath = 'Dataset must be in JSONL or JSON format';
      }
    }

    if (formData.learningRate <= 0 || formData.learningRate > 1e-2) {
      errors.learningRate = 'Learning rate must be between 0 and 0.01';
    }

    if (formData.batchSize < 1 || formData.batchSize > 64) {
      errors.batchSize = 'Batch size must be between 1 and 64';
    }

    if (formData.maxEpochs < 1 || formData.maxEpochs > 20) {
      errors.maxEpochs = 'Maximum epochs must be between 1 and 20';
    }

    setValidationErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const getRecommendedConfig = (): TrainingConfigInfo => {
    return {
      learningRate: formData.learningRate,
      batchSize: formData.batchSize,
      maxEpochs: formData.maxEpochs,
      mixedPrecision: formData.mixedPrecision,
      maxSeqLength: formData.maxSeqLength,
      loraRank: formData.enableLoRA ? formData.loraRank : undefined,
    };
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsSubmitting(true);

    try {
      const request: FineTuningRequest = {
        jobName: formData.jobName.trim(),
        description: formData.description.trim(),
        baseModel: formData.baseModelConfig.baseModel,
        datasetPath: formData.datasetPath.trim(),
        config: getRecommendedConfig(),
        enableMonitoring: true,
      };

      await onSubmit(request);

      // Reset form
      setFormData({
        jobName: '',
        description: '',
        baseModelConfig: MODEL_CONFIGS.codellama7b,
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
      console.error('Failed to submit fine-tuning request:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content create-job-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Create New Fine-Tuning Job</h2>
          <button onClick={onClose} className="close-btn">&times;</button>
        </div>

        <form onSubmit={handleSubmit} className="job-creation-form">
          <div className="form-section">
            <h3>Job Information</h3>
            <div className="form-group">
              <label htmlFor="jobName">Job Name *</label>
              <input
                type="text"
                id="jobName"
                value={formData.jobName}
                onChange={(e) => setFormData(prev => ({ ...prev, jobName: e.target.value }))}
                placeholder="e.g., rust-code-analysis-v1"
                disabled={isSubmitting}
                className={validationErrors.jobName ? 'error' : ''}
              />
              {validationErrors.jobName && (
                <span className="error-text">{validationErrors.jobName}</span>
              )}
            </div>

            <div className="form-group">
              <label htmlFor="description">Description *</label>
              <textarea
                id="description"
                value={formData.description}
                onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Describe what this fine-tuning job is for..."
                rows={3}
                disabled={isSubmitting}
                className={validationErrors.description ? 'error' : ''}
              />
              {validationErrors.description && (
                <span className="error-text">{validationErrors.description}</span>
              )}
            </div>
          </div>

          <div className="form-section">
            <h3>Model Selection</h3>
            <div className="model-selector">
              <label>Base Model</label>
              <div className="model-options">
                {Object.entries(MODEL_CONFIGS).map(([key, config]) => (
                  <div
                    key={key}
                    className={`model-option ${
                      formData.baseModelConfig.baseModel === config.baseModel ? 'selected' : ''
                    }`}
                    onClick={() => handleModelChange(key as ModelKey)}
                  >
                    <h4>{config.baseModel}</h4>
                    <div className="model-features">
                      <span>{config.maxSeqLength} tokens</span>
                      {config.supportsMixedPrecision && <span>Mixed Precision</span>}
                      {config.supportsLoRA && <span>LoRA Support</span>}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          <div className="form-section">
            <h3>Dataset</h3>
            <div className="form-group">
              <label htmlFor="datasetPath">Dataset Path *</label>
              <input
                type="text"
                id="datasetPath"
                value={formData.datasetPath}
                onChange={(e) => setFormData(prev => ({ ...prev, datasetPath: e.target.value }))}
                placeholder="/path/to/training/data.jsonl"
                disabled={isSubmitting}
                className={validationErrors.datasetPath ? 'error' : ''}
              />
              {validationErrors.datasetPath && (
                <span className="error-text">{validationErrors.datasetPath}</span>
              )}
            </div>
          </div>

          <div className="form-section">
            <h3>Training Configuration</h3>
            <div className="config-grid">
              <div className="form-group">
                <label htmlFor="learningRate">Learning Rate</label>
                <input
                  type="number"
                  id="learningRate"
                  step="1e-6"
                  min="1e-6"
                  max="1e-2"
                  value={formData.learningRate}
                  onChange={(e) => setFormData(prev => ({
                    ...prev,
                    learningRate: parseFloat(e.target.value) || 0
                  }))}
                  disabled={isSubmitting}
                  className={validationErrors.learningRate ? 'error' : ''}
                />
                {validationErrors.learningRate && (
                  <span className="error-text">{validationErrors.learningRate}</span>
                )}
              </div>

              <div className="form-group">
                <label htmlFor="batchSize">Batch Size</label>
                <input
                  type="number"
                  id="batchSize"
                  min="1"
                  max="64"
                  value={formData.batchSize}
                  onChange={(e) => setFormData(prev => ({
                    ...prev,
                    batchSize: parseInt(e.target.value) || 1
                  }))}
                  disabled={isSubmitting}
                  className={validationErrors.batchSize ? 'error' : ''}
                />
                {validationErrors.batchSize && (
                  <span className="error-text">{validationErrors.batchSize}</span>
                )}
              </div>

              <div className="form-group">
                <label htmlFor="maxEpochs">Max Epochs</label>
                <input
                  type="number"
                  id="maxEpochs"
                  min="1"
                  max="20"
                  value={formData.maxEpochs}
                  onChange={(e) => setFormData(prev => ({
                    ...prev,
                    maxEpochs: parseInt(e.target.value) || 1
                  }))}
                  disabled={isSubmitting}
                  className={validationErrors.maxEpochs ? 'error' : ''}
                />
                {validationErrors.maxEpochs && (
                  <span className="error-text">{validationErrors.maxEpochs}</span>
                )}
              </div>

              <div className="form-group">
                <label htmlFor="maxSeqLength">Max Sequence Length</label>
                <input
                  type="number"
                  id="maxSeqLength"
                  min="512"
                  max="8192"
                  step="512"
                  value={formData.maxSeqLength}
                  onChange={(e) => setFormData(prev => ({
                    ...prev,
                    maxSeqLength: parseInt(e.target.value) || 2048
                  }))}
                  disabled={isSubmitting}
                />
              </div>
            </div>

            <div className="checkbox-group">
              <label>
                <input
                  type="checkbox"
                  checked={formData.mixedPrecision}
                  onChange={(e) => setFormData(prev => ({
                    ...prev,
                    mixedPrecision: e.target.checked
                  }))}
                  disabled={isSubmitting || !formData.baseModelConfig.supportsMixedPrecision}
                />
                Enable Mixed Precision Training
              </label>

              {formData.baseModelConfig.supportsLoRA && (
                <>
                  <label>
                    <input
                      type="checkbox"
                      checked={formData.enableLoRA}
                      onChange={(e) => setFormData(prev => ({
                        ...prev,
                        enableLoRA: e.target.checked
                      }))}
                      disabled={isSubmitting}
                    />
                    Enable LoRA (Parameter Efficient Fine-Tuning)
                  </label>

                  {formData.enableLoRA && (
                    <div className="form-group">
                      <label htmlFor="loraRank">LoRA Rank</label>
                      <input
                        type="number"
                        id="loraRank"
                        min="4"
                        max="64"
                        step="4"
                        value={formData.loraRank}
                        onChange={(e) => setFormData(prev => ({
                          ...prev,
                          loraRank: parseInt(e.target.value) || 8
                        }))}
                        disabled={isSubmitting}
                      />
                    </div>
                  )}
                </>
              )}
            </div>
          </div>

          <div className="form-actions">
            <button
              type="button"
              onClick={onClose}
              className="cancel-btn"
              disabled={isSubmitting}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="submit-btn"
              disabled={isSubmitting}
            >
              {isSubmitting ? 'Creating Job...' : 'Create Fine-Tuning Job'}
            </button>
          </div>
        </form>

        <style jsx>{`
          .modal-overlay {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.6);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 1000;
          }

          .modal-content {
            background: white;
            border-radius: 12px;
            padding: 0;
            max-width: 800px;
            width: 90%;
            max-height: 90vh;
            overflow-y: auto;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
          }

          .modal-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 24px;
            border-bottom: 1px solid #e1e5e9;
          }

          .modal-header h2 {
            margin: 0;
            font-size: 24px;
            color: #2d3748;
          }

          .close-btn {
            background: none;
            border: none;
            font-size: 28px;
            cursor: pointer;
            color: #718096;
            padding: 0;
          }

          .close-btn:hover {
            color: #2d3748;
          }

          .job-creation-form {
            padding: 24px;
          }

          .form-section {
            margin-bottom: 32px;
            border-bottom: 1px solid #f1f5f9;
            padding-bottom: 20px;
          }

          .form-section:last-child {
            border-bottom: none;
          }

          .form-section h3 {
            margin: 0 0 16px 0;
            color: #2d3748;
            font-size: 18px;
          }

          .form-group {
            margin-bottom: 16px;
          }

          .form-group label {
            display: block;
            margin-bottom: 6px;
            font-weight: 500;
            color: #4a5568;
          }

          .form-group input,
          .form-group select,
          .form-group textarea {
            width: 100%;
            padding: 8px 12px;
            border: 1px solid #e1e5e9;
            border-radius: 6px;
            font-size: 14px;
            transition: border-color 0.2s;
          }

          .form-group input:focus,
          .form-group select:focus,
          .form-group textarea:focus {
            outline: none;
            border-color: #3182ce;
          }

          .form-group.input {
            border-color: #fc8181;
          }

          .form-group.input input {
            border-color: #fc8181;
          }

          .error-text {
            color: #e53e3e;
            font-size: 12px;
            margin-top: 4px;
            display: block;
          }

          .model-selector {
            margin-top: 16px;
          }

          .model-options {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 12px;
            margin-top: 12px;
          }

          .model-option {
            padding: 16px;
            border: 2px solid #e1e5e9;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.2s;
          }

          .model-option:hover {
            border-color: #3182ce;
          }

          .model-option.selected {
            border-color: #3182ce;
            background: #ebf8ff;
          }

          .model-option h4 {
            margin: 0 0 8px 0;
            color: #2d3748;
          }

          .model-features {
            display: flex;
            gap: 8px;
            flex-wrap: wrap;
          }

          .model-features span {
            background: #f1f5f9;
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 12px;
            color: #4a5568;
          }

          .config-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
          }

          .checkbox-group {
            margin-top: 16px;
          }

          .checkbox-group label {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 12px;
            cursor: pointer;
          }

          .checkbox-group input[type="checkbox"] {
            width: auto;
          }

          .form-actions {
            display: flex;
            justify-content: flex-end;
            gap: 12px;
            padding-top: 24px;
            border-top: 1px solid #e1e5e9;
          }

          .cancel-btn,
          .submit-btn {
            padding: 12px 24px;
            border-radius: 6px;
            font-weight: 500;
            cursor: pointer;
            transition: background-color 0.2s;
          }

          .cancel-btn {
            background: #e2e8f0;
            color: #4a5568;
            border: 1px solid #cbd5e0;
          }

          .cancel-btn:hover:not(:disabled) {
            background: #cbd5e0;
          }

          .submit-btn {
            background: #3182ce;
            color: white;
            border: none;
          }

          .submit-btn:hover:not(:disabled) {
            background: #2c5282;
          }

          .cancel-btn:disabled,
          .submit-btn:disabled {
            opacity: 0.6;
            cursor: not-allowed;
          }
        `}</style>
      </div>
    </div>
  );
};

export default CreateFineTuneModal;