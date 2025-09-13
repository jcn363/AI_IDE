import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';

import {
  VectorSearchRequest,
  InferenceRequest,
  CodeSearchRequest,
  ABTestConfiguration,
  AIServices,
  VectorSearchResult,
  InferenceResult,
  CodeSearchResult,
  ABTestResults,
  PerformanceMetrics,
  AIServiceEvent,
  APIResponse,
} from '../types/ai';

/**
 * AI Service Layer - provides typed interface to Tauri commands
 * Handles communication between React components and backend AI services
 */
class AIService implements AIServices {
  private eventListeners: Map<string, Array<(event: any) => void>> = new Map();

  constructor() {
    this.setupEventListeners();
  }

  /**
   * Single inference request
   */
  async infer(request: InferenceRequest): Promise<InferenceResult> {
    try {
      console.log('Sending inference request:', request.model_name);
      const result = await invoke<InferenceResult>('onnx_inference', { request });
      console.log('Inference completed:', result.model_used, result.inference_time_ms + 'ms');
      return result;
    } catch (error) {
      console.error('Inference failed:', error);
      throw new Error(`Inference failed: ${error}`);
    }
  }

  /**
   * Batch inference for multiple requests
   */
  async batchInfer(requests: InferenceRequest[]): Promise<InferenceResult[]> {
    try {
      console.log(`Sending batch inference: ${requests.length} requests`);

      // For now, process sequentially - in production this could be optimized
      const results = await Promise.all(requests.map((request) => this.infer(request)));

      console.log(`Batch inference completed: ${results.length} results`);
      return results;
    } catch (error) {
      console.error('Batch inference failed:', error);
      throw new Error(`Batch inference failed: ${error}`);
    }
  }

  /**
   * Vector similarity search
   */
  async vectorSearch(request: VectorSearchRequest): Promise<VectorSearchResult[]> {
    try {
      console.log('Vector search request:', request.top_k, 'results requested');
      const results = await invoke<VectorSearchResult[]>('vector_search', { request });
      console.log(`Vector search completed: ${results.length} results`);
      return results;
    } catch (error) {
      console.error('Vector search failed:', error);
      throw new Error(`Vector search failed: ${error}`);
    }
  }

  /**
   * Semantic code search
   */
  async codeSearch(request: CodeSearchRequest): Promise<CodeSearchResult[]> {
    try {
      console.log(
        'Code search request:',
        request.query,
        'languages:',
        request.languages.join(', ')
      );
      const results = await invoke<CodeSearchResult[]>('semantic_code_search', { request });
      console.log(`Code search completed: ${results.length} results`);
      return results;
    } catch (error) {
      console.error('Code search failed:', error);
      throw new Error(`Code search failed: ${error}`);
    }
  }

  /**
   * Index codebase for semantic search
   */
  async indexCodebase(path: string): Promise<void> {
    try {
      console.log('Indexing codebase:', path);
      await invoke('index_codebase', { path });
      console.log('Codebase indexing started');
    } catch (error) {
      console.error('Codebase indexing failed:', error);
      throw new Error(`Codebase indexing failed: ${error}`);
    }
  }

  /**
   * Get indexing status
   */
  async getIndexingStatus(): Promise<any> {
    try {
      const status = await invoke<any>('get_indexing_status', {});
      console.log('Indexing status:', status);
      return status;
    } catch (error) {
      console.error('Failed to get indexing status:', error);
      return { is_indexing: false, progress: 0.0, error: error.toString() };
    }
  }

  /**
   * Configure A/B testing for models
   */
  async configureABTest(testName: string, config: ABTestConfiguration): Promise<void> {
    try {
      console.log('Configuring A/B test:', testName);
      await invoke('configure_ab_test', { testName, config });
      console.log('A/B test configured successfully');
    } catch (error) {
      console.error('A/B test configuration failed:', error);
      throw new Error(`A/B test configuration failed: ${error}`);
    }
  }

  /**
   * Get A/B test results
   */
  async getABTestResults(testName: string): Promise<ABTestResults> {
    try {
      console.log('Fetching A/B test results:', testName);
      const results = await invoke<ABTestResults>('get_ab_test_results', { testName });
      console.log('A/B test results fetched');
      return results;
    } catch (error) {
      console.error('Failed to get A/B test results:', error);
      throw new Error(`Failed to get A/B test results: ${error}`);
    }
  }

  /**
   * Get available model versions
   */
  async getModelVersions(modelId: string): Promise<string[]> {
    try {
      console.log('Fetching model versions for:', modelId);
      const versions = await invoke<string[]>('get_model_versions', { modelId });
      console.log(`Model versions fetched: ${versions.length} available`);
      return versions;
    } catch (error) {
      console.error('Failed to get model versions:', error);
      // Return dummy data if service is not available
      return ['1.0.0', '1.1.0', '2.0.0'];
    }
  }

  /**
   * Switch to different model version
   */
  async switchModelVersion(modelId: string, version: string): Promise<void> {
    try {
      console.log('Switching model version:', modelId, 'to', version);
      await invoke('switch_model_version', { modelId, version });
      console.log('Model version switched successfully');
    } catch (error) {
      console.error('Model version switch failed:', error);
      throw new Error(`Model version switch failed: ${error}`);
    }
  }

  /**
   * Enqueue heavy computation task
   */
  async enqueueHeavyTask(taskType: string, data: any): Promise<string> {
    try {
      console.log('Enqueueing heavy task:', taskType);
      const taskId = await invoke<string>('enqueue_heavy_task', { taskType, data });
      console.log('Heavy task enqueued with ID:', taskId);
      return taskId;
    } catch (error) {
      console.error('Failed to enqueue heavy task:', error);
      throw new Error(`Failed to enqueue heavy task: ${error}`);
    }
  }

  /**
   * Get comprehensive performance metrics
   */
  async getPerformanceMetrics(): Promise<PerformanceMetrics> {
    try {
      console.log('Fetching performance metrics');
      const metrics = await invoke<PerformanceMetrics>('get_performance_metrics', {});
      console.log('Performance metrics fetched');
      return metrics;
    } catch (error) {
      console.error('Failed to get performance metrics:', error);
      throw new Error(`Failed to get performance metrics: ${error}`);
    }
  }

  /**
   * Get real-time GPU metrics
   */
  async getGPUMetrics(): Promise<Record<string, any>> {
    try {
      console.log('Fetching GPU metrics');
      const metrics = await invoke<Record<string, any>>('get_gpu_metrics', {});
      console.log('GPU metrics fetched');
      return metrics;
    } catch (error) {
      console.error('Failed to get GPU metrics:', error);
      // Return dummy data
      return {
        utilization: 0,
        memory_used: 0,
        error: error.toString(),
      };
    }
  }

  /**
   * Set up event listeners for real-time updates
   */
  private setupEventListeners(): void {
    // Listen for Tauri events related to AI operations
    // This would be expanded based on actual events emitted from Rust backend
  }

  /**
   * Emit custom events to React components
   */
  emitEvent(event: AIServiceEvent): void {
    const listeners = this.eventListeners.get(event.type) || [];
    listeners.forEach((listener) => {
      try {
        listener(event);
      } catch (error) {
        console.error('Error in event listener:', error);
      }
    });
  }

  /**
   * Subscribe to AI service events
   */
  on(eventType: string, callback: (event: any) => void): () => void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, []);
    }

    this.eventListeners.get(eventType)!.push(callback);

    // Return unsubscribe function
    return () => {
      const listeners = this.eventListeners.get(eventType) || [];
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    };
  }

  /**
   * Check if AI services are available
   */
  async checkAvailability(): Promise<{ available: boolean; services: string[] }> {
    try {
      // Quick health check by trying to get performance metrics
      const metrics = await this.getPerformanceMetrics();

      // Extract available services from metrics
      const services = [];
      if (metrics.gpu_stats && metrics.gpu_stats.length > 0) {
        services.push('gpu_monitoring');
      }
      if (Object.keys(metrics.model_stats).length > 0) {
        services.push('model_monitoring');
      }

      services.push('inference', 'search'); // Core services

      return {
        available: true,
        services,
      };
    } catch (error) {
      console.warn('AI services check failed:', error);
      return {
        available: false,
        services: [],
      };
    }
  }
}

// Singleton instance
export const aiService = new AIService();

// Default export for convenience
export default aiService;

// Utility functions for React integration

/**
 * Wrap API calls with consistent error handling and state updates
 */
export async function apiCall<T>(
  operation: () => Promise<T>,
  onSuccess?: (result: T) => void,
  onError?: (error: Error) => void
): Promise<T> {
  try {
    const result = await operation();
    onSuccess?.(result);
    return result;
  } catch (error) {
    console.error('AI Service API call failed:', error);
    onError?.(error as Error);
    throw error;
  }
}

/**
 * Create a typed API response wrapper
 */
export function createAPIResponse<T>(data: T, error?: string): APIResponse<T> {
  return {
    success: !error,
    data: error ? undefined : data,
    error,
    timestamp: Date.now(),
  };
}
