import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ModelInfo, ModelType, ModelSize } from '../../features/ai/types';

interface ModelSelectorProps {
  onModelSelect?: (modelId: string) => void;
  selectedModelId?: string;
  filterBySupportedFineTuning?: boolean;
}

interface ModelOption {
  id: string;
  name: string;
  type: ModelType;
  size: ModelSize;
  isLoaded: boolean;
  supportsFineTuning: boolean;
  memoryUsage?: number;
  lastUsed?: string;
}

const MODEL_TYPE_LABELS: Record<ModelType, string> = {
  CodeLlama: 'CodeLlama',
  StarCoder: 'StarCoder',
  Custom: 'Custom',
};

const MODEL_SIZE_LABELS: Record<ModelSize, string> = {
  Small: 'Small',
  Medium: 'Medium',
  Large: 'Large',
};

const MODEL_TYPE_COLORS: Record<ModelType, string> = {
  CodeLlama: '#3182ce',
  StarCoder: '#dd6b20',
  Custom: '#38a169',
};

export const ModelSelector: React.FC<ModelSelectorProps> = ({
  onModelSelect,
  selectedModelId,
  filterBySupportedFineTuning = true,
}) => {
  const [models, setModels] = useState<ModelOption[]>([]);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState<{
    type?: ModelType;
    size?: ModelSize;
    loaded?: boolean;
  }>({});

  useEffect(() => {
    loadAvailableModels();
  }, []);

  const loadAvailableModels = async () => {
    setLoading(true);
    try {
      // This would normally call the backend to get available models
      // For now, we'll simulate some models
      const mockModels: ModelOption[] = [
        {
          id: 'codellama-7b',
          name: 'CodeLlama 7B',
          type: 'CodeLlama',
          size: 'Large',
          isLoaded: true,
          supportsFineTuning: true,
          memoryUsage: 13.5,
          lastUsed: new Date().toISOString(),
        },
        {
          id: 'codellama-13b',
          name: 'CodeLlama 13B',
          type: 'CodeLlama',
          size: 'Large',
          isLoaded: false,
          supportsFineTuning: true,
          memoryUsage: 27,
        },
        {
          id: 'starcoder-7b',
          name: 'StarCoder 7B',
          type: 'StarCoder',
          size: 'Large',
          isLoaded: true,
          supportsFineTuning: true,
          memoryUsage: 13.8,
          lastUsed: new Date(Date.now() - 86400000).toISOString(),
        },
        {
          id: 'codellama-3b',
          name: 'CodeLlama 3B',
          type: 'CodeLlama',
          size: 'Medium',
          isLoaded: true,
          supportsFineTuning: true,
          memoryUsage: 6.8,
        },
      ];

      // Actually load models from backend
      const availableModels: ModelInfo[] = await invoke('get_available_models');
      const formattedModels = availableModels.map((model) => ({
        id: model.id,
        name: `${MODEL_TYPE_LABELS[model.modelType]} ${model.modelSize}`,
        type: model.modelType,
        size: model.modelSize,
        isLoaded: model.isLoaded,
        supportsFineTuning: model.supportsFineTuning,
        memoryUsage: model.memoryUsageMB,
        lastUsed: model.lastUsedAt,
      }));

      setModels(formattedModels);
    } catch (error) {
      console.error('Failed to load models:', error);
      setModels([]);
    } finally {
      setLoading(false);
    }
  };

  const filteredModels = models.filter((model) => {
    if (filterBySupportedFineTuning && !model.supportsFineTuning) return false;
    if (filter.type && model.type !== filter.type) return false;
    if (filter.size && model.size !== filter.size) return false;
    if (filter.loaded !== undefined && model.isLoaded !== filter.loaded) return false;
    return true;
  });

  const handleModelClick = (model: ModelOption) => {
    if (!model.isLoaded) {
      // Optionally trigger loading the model
      console.log(`Loading model: ${model.id}`);
    }
    onModelSelect?.(model.id);
  };

  const formatMemoryUsage = (mb?: number) => {
    if (!mb) return 'Unknown';
    return mb < 1000 ? `${mb.toFixed(1)} MB` : `${(mb / 1024).toFixed(1)} GB`;
  };

  const formatLastUsed = (isoString?: string) => {
    if (!isoString) return 'Never used';
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = diffMs / (1000 * 60 * 60);

    if (diffHours < 24) {
      return `${Math.floor(diffHours)}h ago`;
    } else if (diffHours < 168) {
      return `${Math.floor(diffHours / 24)}d ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  return (
    <div className="model-selector">
      <div className="selector-header">
        <h3>Select Model</h3>
        <button onClick={loadAvailableModels} disabled={loading}>
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      <div className="model-filters">
        <div className="filter-group">
          <label>Type:</label>
          <select
            value={filter.type || ''}
            onChange={(e) =>
              setFilter((prev) => ({
                ...prev,
                type: (e.target.value as ModelType) || undefined,
              }))
            }
          >
            <option value="">All Types</option>
            {Object.entries(MODEL_TYPE_LABELS).map(([key, label]) => (
              <option key={key} value={key}>
                {label}
              </option>
            ))}
          </select>
        </div>

        <div className="filter-group">
          <label>Size:</label>
          <select
            value={filter.size || ''}
            onChange={(e) =>
              setFilter((prev) => ({
                ...prev,
                size: (e.target.value as ModelSize) || undefined,
              }))
            }
          >
            <option value="">All Sizes</option>
            {Object.entries(MODEL_SIZE_LABELS).map(([key, label]) => (
              <option key={key} value={key}>
                {label}
              </option>
            ))}
          </select>
        </div>

        <div className="filter-group">
          <label>Status:</label>
          <select
            value={filter.loaded !== undefined ? filter.loaded.toString() : ''}
            onChange={(e) =>
              setFilter((prev) => ({
                ...prev,
                loaded: e.target.value ? e.target.value === 'true' : undefined,
              }))
            }
          >
            <option value="">All</option>
            <option value="true">Loaded</option>
            <option value="false">Not Loaded</option>
          </select>
        </div>
      </div>

      <div className="model-grid">
        {filteredModels.map((model) => (
          <div
            key={model.id}
            className={`model-card ${selectedModelId === model.id ? 'selected' : ''}`}
            onClick={() => handleModelClick(model)}
          >
            <div className="model-header">
              <div
                className="model-type-badge"
                style={{
                  backgroundColor: MODEL_TYPE_COLORS[model.type],
                }}
              >
                {MODEL_TYPE_LABELS[model.type]}
              </div>
              <div className={`model-status ${model.isLoaded ? 'loaded' : 'not-loaded'}`}>
                {model.isLoaded ? 'Loaded' : 'Not Loaded'}
              </div>
            </div>

            <div className="model-content">
              <h4>{model.name}</h4>
              <div className="model-details">
                <span className="detail-item">Size: {MODEL_SIZE_LABELS[model.size]}</span>
                <span className="detail-item">Memory: {formatMemoryUsage(model.memoryUsage)}</span>
                <span className="detail-item">Last Used: {formatLastUsed(model.lastUsed)}</span>
              </div>

              {model.isLoaded && (
                <div className="model-memory-usage">
                  <div className="memory-bar">
                    <div
                      className="memory-fill"
                      style={{
                        width: model.memoryUsage
                          ? `${Math.min((model.memoryUsage / 32) * 100, 100)}%`
                          : '20%',
                      }}
                    />
                  </div>
                  <span className="memory-text">Used: {formatMemoryUsage(model.memoryUsage)}</span>
                </div>
              )}

              {!model.isLoaded && (
                <button
                  className="load-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    // Handle model loading
                    console.log(`Loading model: ${model.id}`);
                  }}
                >
                  Load Model
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      {filteredModels.length === 0 && (
        <div className="no-models">
          <p>No models match your filters</p>
        </div>
      )}

      <style jsx>{`
        .model-selector {
          padding: 20px;
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          background: white;
        }

        .selector-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
        }

        .selector-header h3 {
          margin: 0;
          color: #2d3748;
        }

        .selector-header button {
          padding: 6px 12px;
          background: #3182ce;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .selector-header button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .model-filters {
          display: flex;
          gap: 16px;
          margin-bottom: 20px;
          flex-wrap: wrap;
        }

        .filter-group {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .filter-group label {
          font-weight: 500;
          color: #4a5568;
        }

        .filter-group select {
          padding: 6px;
          border: 1px solid #e1e5e9;
          border-radius: 4px;
          cursor: pointer;
        }

        .model-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
          gap: 16px;
        }

        .model-card {
          border: 2px solid #e1e5e9;
          border-radius: 8px;
          padding: 16px;
          cursor: pointer;
          transition: all 0.2s ease;
          background: #f8fafc;
        }

        .model-card:hover {
          border-color: #3182ce;
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .model-card.selected {
          border-color: #3182ce;
          background: #ebf8ff;
        }

        .model-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 12px;
        }

        .model-type-badge {
          padding: 4px 8px;
          border-radius: 12px;
          color: white;
          font-size: 12px;
          font-weight: 500;
        }

        .model-status {
          padding: 2px 6px;
          border-radius: 8px;
          font-size: 11px;
          font-weight: 500;
          text-transform: uppercase;
        }

        .model-status.loaded {
          background: #c6f6d5;
          color: #276749;
        }

        .model-status.not-loaded {
          background: #f7fafc;
          color: #4a5568;
        }

        .model-content h4 {
          margin: 0 0 8px 0;
          color: #2d3748;
          font-size: 16px;
        }

        .model-details {
          display: flex;
          flex-direction: column;
          gap: 4px;
          margin-bottom: 12px;
        }

        .detail-item {
          font-size: 13px;
          color: #718096;
        }

        .model-memory-usage {
          margin-top: 12px;
        }

        .memory-bar {
          height: 6px;
          background: #e1e5e9;
          border-radius: 3px;
          overflow: hidden;
          margin-bottom: 4px;
        }

        .memory-fill {
          height: 100%;
          background: #38a169;
          transition: width 0.3s ease;
        }

        .memory-text {
          font-size: 12px;
          color: #4a5568;
        }

        .load-button {
          margin-top: 8px;
          padding: 6px 12px;
          background: #e2e8f0;
          color: #4a5568;
          border: 1px solid #cbd5e0;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          width: 100%;
        }

        .load-button:hover {
          background: #cbd5e0;
        }

        .no-models {
          text-align: center;
          padding: 40px;
          color: #718096;
        }

        .no-models p {
          margin: 0;
          font-size: 16px;
        }

        @media (max-width: 768px) {
          .model-filters {
            flex-direction: column;
            align-items: stretch;
          }

          .model-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};

export default ModelSelector;
