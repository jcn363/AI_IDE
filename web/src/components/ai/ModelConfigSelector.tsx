/**
 * Component for selecting and configuring models for fine-tuning
 * Handles model selection, configuration management, and display
 */

import React from 'react';

export interface ModelConfig {
  baseModel: string;
  supportsMixedPrecision: boolean;
  maxSeqLength: number;
  recommendedBatchSize: number;
  supportsLoRA: boolean;
}

const MODEL_CONFIGS: Record<string, ModelConfig> = {
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
};

interface ModelConfigSelectorProps {
  selectedModelKey: string;
  onModelChange: (modelKey: string, config: ModelConfig) => void;
  disabled?: boolean;
}

/**
 * Component for selecting a model configuration from available options
 */
export const ModelConfigSelector: React.FC<ModelConfigSelectorProps> = ({
  selectedModelKey,
  onModelChange,
  disabled = false,
}) => {
  const handleModelClick = (modelKey: string) => {
    if (disabled) return;

    const config = MODEL_CONFIGS[modelKey];
    if (config) {
      onModelChange(modelKey, config);
    }
  };

  return (
    <div className="model-selector">
      <label>Base Model</label>
      <div className="model-options">
        {Object.entries(MODEL_CONFIGS).map(([key, config]) => (
          <div
            key={key}
            className={`model-option ${
              selectedModelKey === key ? 'selected' : ''
            } ${disabled ? 'disabled' : ''}`}
            onClick={() => handleModelClick(key)}
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
  );
};

/**
 * Gets available model configurations
 */
export const getAvailableModelConfigs = (): Record<string, ModelConfig> => {
  return MODEL_CONFIGS;
};

/**
 * Gets the default model configuration
 */
export const getDefaultModelConfig = (): ModelConfig => {
  return MODEL_CONFIGS.codellama7b;
};

export default ModelConfigSelector;
