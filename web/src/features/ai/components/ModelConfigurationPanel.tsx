import React, { useState, useEffect } from 'react';
import ModelService from '../services/ModelService';
import { ModelInfo, ModelLoadingRequest, ModelSize, Quantization, DeviceType } from '../types';

interface ModelConfigurationPanelProps {
  onModelChange?: (model: ModelInfo | null) => void;
}

export const ModelConfigurationPanel: React.FC<ModelConfigurationPanelProps> = ({
  onModelChange,
}) => {
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [loadedModels, setLoadedModels] = useState<ModelInfo[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadingModels, setLoadingModels] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    try {
      const [available, loaded] = await Promise.all([
        ModelService.listAvailableModels(),
        ModelService.getLoadedModels(),
      ]);
      setAvailableModels(available);
      setLoadedModels(loaded);
    } catch (error) {
      console.error('Failed to load models:', error);
    }
  };

  const handleLoadModel = async (model: ModelInfo) => {
    if (loading) return;

    setLoadingModels((prev) => new Set(prev).add(model.id));

    try {
      const request: ModelLoadingRequest = {
        modelPath: model.model_path || `/models/${model.model_type}-${model.model_size}`,
        modelType: model.model_type,
        modelSize: model.model_size,
        quantization: model.quantization || 'Int4',
        loraAdapters: model.lora_adapters || [],
        device: 'Auto',
      };

      const loadedModel = await ModelService.loadModel(request);
      setLoadedModels((prev) => [...prev, loadedModel]);
      setSelectedModel(loadedModel);
      onModelChange?.(loadedModel);
    } catch (error) {
      console.error('Failed to load model:', error);
      alert(`Failed to load model: ${error}`);
    } finally {
      setLoadingModels((prev) => {
        const newSet = new Set(prev);
        newSet.delete(model.id);
        return newSet;
      });
    }
  };

  const handleUnloadModel = async (modelId: string) => {
    try {
      await ModelService.unloadModel(modelId);
      setLoadedModels((prev) => prev.filter((m) => m.id !== modelId));
      if (selectedModel?.id === modelId) {
        setSelectedModel(null);
        onModelChange?.(null);
      }
    } catch (error) {
      console.error('Failed to unload model:', error);
      alert(`Failed to unload model: ${error}`);
    }
  };

  const getAvailableModels = () => {
    return availableModels.filter(
      (model) => !loadedModels.some((loaded) => loaded.id === model.id)
    );
  };

  return (
    <div className="model-configuration-panel bg-gray-900 text-white p-6 rounded-lg">
      <h2 className="text-xl font-bold mb-4">AI Model Configuration</h2>

      {/* Current Model Status */}
      {selectedModel && (
        <div className="mb-6 p-4 bg-blue-900 rounded">
          <h3 className="text-sm font-semibold mb-2">Active Model</h3>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div>
              <span className="text-gray-400">Name:</span> {selectedModel.model_type}{' '}
              {selectedModel.model_size}
            </div>
            <div>
              <span className="text-gray-400">Size:</span> {selectedModel.model_size}
            </div>
            <div>
              <span className="text-gray-400">Quantization:</span> {selectedModel.quantization}
            </div>
            <div>
              <span className="text-gray-400">Memory:</span>{' '}
              {selectedModel.memory_usage_mb || 'Unknown'}MB
            </div>
          </div>
          <button
            onClick={() => handleUnloadModel(selectedModel.id)}
            className="mt-2 px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm"
          >
            Unload Model
          </button>
        </div>
      )}

      {/* Available Models */}
      <div className="mb-6">
        <h3 className="text-lg font-semibold mb-3">Available Models</h3>
        <div className="space-y-2 max-h-64 overflow-y-auto">
          {getAvailableModels().map((model) => (
            <div
              key={model.id}
              className="flex items-center justify-between p-3 bg-gray-800 rounded"
            >
              <div className="flex-1">
                <div className="font-medium">
                  {model.model_type} {model.model_size}
                </div>
                <div className="text-sm text-gray-400">
                  ~{model.memory_usage_mb || 'Unknown'}MB • Supports fine-tuning:{' '}
                  {model.supports_fine_tuning ? 'Yes' : 'No'}
                </div>
              </div>
              <button
                onClick={() => handleLoadModel(model)}
                disabled={loadingModels.has(model.id)}
                className="px-3 py-1 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 rounded text-sm"
              >
                {loadingModels.has(model.id) ? 'Loading...' : 'Load'}
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* Loaded Models */}
      {loadedModels.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold mb-3">Loaded Models</h3>
          <div className="space-y-2">
            {loadedModels.map((model) => (
              <div
                key={model.id}
                className="flex items-center justify-between p-3 bg-green-900 rounded"
              >
                <div className="flex-1">
                  <div className="font-medium">
                    {model.model_type} {model.model_size}
                  </div>
                  <div className="text-sm text-gray-300">
                    {model.memory_usage_mb}MB • Ready for inference
                  </div>
                </div>
                <div className="space-x-2">
                  <button
                    onClick={() => {
                      setSelectedModel(model);
                      onModelChange?.(model);
                    }}
                    className={`px-2 py-1 rounded text-sm ${
                      selectedModel?.id === model.id
                        ? 'bg-blue-600'
                        : 'bg-gray-600 hover:bg-gray-500'
                    }`}
                  >
                    {selectedModel?.id === model.id ? 'Active' : 'Select'}
                  </button>
                  <button
                    onClick={() => handleUnloadModel(model.id)}
                    className="px-2 py-1 bg-red-600 hover:bg-red-700 rounded text-sm"
                  >
                    Unload
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {availableModels.length === 0 && (
        <div className="text-center py-8 text-gray-400">
          <div className="mb-4">
            <svg
              className="mx-auto h-12 w-12 text-gray-500"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
              />
            </svg>
          </div>
          <p>No models available</p>
          <p className="text-sm mt-1">Models will appear here once installed</p>
        </div>
      )}
    </div>
  );
};
