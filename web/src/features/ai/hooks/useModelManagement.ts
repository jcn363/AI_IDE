import { useState, useEffect, useCallback } from 'react';
import ModelService, { AutoUnloadConfig } from '../services/ModelService';
import type { ModelInfo, ResourceStatus } from '../types';

interface UseModelManagementReturn {
  // Model state
  availableModels: ModelInfo[];
  loadedModels: ModelInfo[];
  selectedModel: ModelInfo | null;
  resourceStatus: ResourceStatus | null;

  // Auto-unload configuration
  autoUnloadConfig: AutoUnloadConfig;

  // Loading states
  loading: boolean;
  error: string | null;

  // Actions
  loadModel: (modelId: string) => Promise<void>;
  unloadModel: (modelId: string) => Promise<void>;
  selectModel: (model: ModelInfo | null) => void;
  refreshModels: () => Promise<void>;
  updateAutoUnloadConfig: (config: Partial<AutoUnloadConfig>) => void;
  triggerManualCleanup: () => Promise<void>;
  recordModelUsage: (modelId: string) => void;

  // Computed values
  totalMemoryUsage: number;
  activeModelCount: number;
  isMonitoringActive: boolean;
}

/**
 * Custom hook for AI model management
 */
export const useModelManagement = (): UseModelManagementReturn => {
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [loadedModels, setLoadedModels] = useState<ModelInfo[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelInfo | null>(null);
  const [resourceStatus, setResourceStatus] = useState<ResourceStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [autoUnloadConfig, setAutoUnloadConfig] = useState<AutoUnloadConfig>(ModelService.getAutoUnloadConfig());

  // Load initial data
  const loadInitialData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const [available, loaded, resources] = await Promise.all([
        ModelService.listAvailableModels(),
        ModelService.getLoadedModels(),
        ModelService.getResourceStatus(),
      ]);

      setAvailableModels(available);
      setLoadedModels(loaded);
      setResourceStatus(resources);

      // Auto-select first loaded model if available
      if (loaded.length > 0 && !selectedModel) {
        setSelectedModel(loaded[0]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load models');
      console.error('Model management initialization failed:', err);
    } finally {
      setLoading(false);
    }
  }, [selectedModel]);

  useEffect(() => {
    loadInitialData();
  }, [loadInitialData]);

  // Load a specific model
  const loadModel = useCallback(async (modelId: string) => {
    try {
      setLoading(true);
      setError(null);

      const availableModel = availableModels.find(m => m.id === modelId);
      if (!availableModel) {
        throw new Error(`Model ${modelId} not found in available models`);
      }

      // Check if already loaded
      if (loadedModels.some(m => m.id === modelId)) {
        console.log(`Model ${modelId} is already loaded`);
        const loadedModel = loadedModels.find(m => m.id === modelId);
        if (loadedModel) {
          setSelectedModel(loadedModel);
        }
        return;
      }

      console.log(`Loading model: ${modelId}`);
      const loadedModel = await ModelService.loadModel({
        modelPath: availableModel.modelPath || `/models/${availableModel.modelType.toLowerCase()}-${availableModel.modelSize.toLowerCase()}`,
        modelType: availableModel.modelType,
        modelSize: availableModel.modelSize,
        quantization: availableModel.quantization || 'Int4',
        loraAdapters: availableModel.loraAdapters,
        device: 'Auto',
      });

      setLoadedModels(prev => [...prev, loadedModel]);
      setSelectedModel(loadedModel);

      // Refresh resource status
      const resources = await ModelService.getResourceStatus();
      setResourceStatus(resources);

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load model';
      setError(errorMessage);
      console.error('Failed to load model:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [availableModels, loadedModels]);

  // Unload a model
  const unloadModel = useCallback(async (modelId: string) => {
    try {
      setLoading(true);
      setError(null);

      console.log(`Unloading model: ${modelId}`);
      await ModelService.unloadModel(modelId);

      setLoadedModels(prev => prev.filter(m => m.id !== modelId));

      // Clear selection if unloaded model was selected
      if (selectedModel?.id === modelId) {
        setSelectedModel(null);
      }

      // Refresh resource status
      const resources = await ModelService.getResourceStatus();
      setResourceStatus(resources);

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to unload model';
      setError(errorMessage);
      console.error('Failed to unload model:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [selectedModel]);

  // Select a model (for using without loading)
  const selectModel = useCallback((model: ModelInfo | null) => {
    setSelectedModel(model);
  }, []);

  // Update auto-unload configuration
  const updateAutoUnloadConfig = useCallback((config: Partial<AutoUnloadConfig>) => {
    ModelService.updateAutoUnloadConfig(config);
    setAutoUnloadConfig(ModelService.getAutoUnloadConfig());
  }, []);

  // Trigger manual cleanup
  const triggerManualCleanup = useCallback(async () => {
    try {
      setLoading(true);
      await ModelService.triggerManualCleanup();

      // Refresh resource status after cleanup
      const resources = await ModelService.getResourceStatus();
      setResourceStatus(resources);

      // Refresh loaded models
      const loaded = await ModelService.getLoadedModels();
      setLoadedModels(loaded);

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to trigger cleanup';
      setError(errorMessage);
      console.error('Failed to trigger cleanup:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  // Record model usage
  const recordModelUsage = useCallback((modelId: string) => {
    try {
      ModelService.recordModelActivity(modelId);
    } catch (err) {
      console.error('Failed to record model usage:', err);
    }
  }, []);

  // Refresh models data
  const refreshModels = useCallback(async () => {
    await loadInitialData();
  }, [loadInitialData]);

  // Computed values
  const totalMemoryUsage = loadedModels.reduce((total, model) => total + (model.memoryUsageMB || 0), 0);
  const activeModelCount = loadedModels.length;
  const isMonitoringActive = ModelService.isMonitoringActive();

  return {
    availableModels,
    loadedModels,
    selectedModel,
    resourceStatus,
    autoUnloadConfig,
    loading,
    error,
    loadModel,
    unloadModel,
    selectModel,
    refreshModels,
    updateAutoUnloadConfig,
    triggerManualCleanup,
    recordModelUsage,
    totalMemoryUsage,
    activeModelCount,
    isMonitoringActive,
  };
};