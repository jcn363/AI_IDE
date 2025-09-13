/**
 * Capability unification utilities
 * Unifies capability shapes between UI and backend for consistent access patterns
 */
import type { BackendCapabilities, BackendFeatures } from '../../../types/refactoring';

/**
 * Backend capabilities response with snake_case keys (matches backend API format)
 */
export interface BackendCapabilitiesResponse {
  supported_refactorings: string[];
  supported_file_types: string[];
  features: BackendFeaturesSnake;
  performance_metrics: Record<string, number>;
  configuration_options: string[];
}

/**
 * Backend features with snake_case keys (matches backend API format)
 */
export interface BackendFeaturesSnake {
  batch_operations: boolean;
  analysis: boolean;
  backup_recovery: boolean;
  test_generation: boolean;
  ai_analysis: boolean;
  lsp_integration: boolean;
  git_integration: boolean;
  cross_language_support: boolean;
  parallel_processing: boolean;
}

/**
 * Feature support summary for UI consumption
 */
export interface FeatureSupportSummary {
  batchOperations: boolean;
  testGeneration: boolean;
  analysis: boolean;
  performanceMetrics: boolean;
  advancedFeatures: boolean;
  basicOperations: readonly string[];
}

/**
 * Utility class for unifying capability shapes between UI and backend
 */
export class CapabilityUnifier {
  /**
   * Convert backend capabilities from snake_case to camelCase for UI consistency
   */
  static convertBackendCapabilitiesToCamelCase(
    backendResponse: BackendCapabilitiesResponse
  ): BackendCapabilities {
    return {
      supportedRefactorings: backendResponse.supported_refactorings,
      supportedFileTypes: backendResponse.supported_file_types,
      features: this.convertFeaturesToCamelCase(backendResponse.features),
      performanceMetrics: backendResponse.performance_metrics,
      configurationOptions: backendResponse.configuration_options,
    };
  }

  /**
   * Convert backend features from snake_case to camelCase
   */
  static convertFeaturesToCamelCase(snakeFeatures: BackendFeaturesSnake): BackendFeatures {
    return {
      batchOperations: snakeFeatures.batch_operations,
      analysis: snakeFeatures.analysis,
      backupRecovery: snakeFeatures.backup_recovery,
      testGeneration: snakeFeatures.test_generation,
      aiAnalysis: snakeFeatures.ai_analysis,
      lspIntegration: snakeFeatures.lsp_integration,
      gitIntegration: snakeFeatures.git_integration,
      crossLanguageSupport: snakeFeatures.cross_language_support,
      parallelProcessing: snakeFeatures.parallel_processing,
    };
  }

  /**
   * Get feature support summary with unified naming
   */
  static getFeatureSupportSummary(capabilities: BackendCapabilities): FeatureSupportSummary {
    const features = capabilities.features;
    return {
      batchOperations: features.batchOperations,
      testGeneration: features.testGeneration,
      analysis: features.analysis,
      performanceMetrics: capabilities.performanceMetrics !== undefined,
      advancedFeatures:
        features.aiAnalysis || features.lspIntegration || features.crossLanguageSupport,
      basicOperations:
        features.analysis === false
          ? (['rename', 'extract-function', 'extract-variable'] as const)
          : ([] as const),
    };
  }

  /**
   * Validate that capabilities response has all required fields
   */
  static validateCapabilities(capabilities: BackendCapabilitiesResponse): {
    valid: boolean;
    missingFields: string[];
  } {
    const requiredFields = [
      'supported_refactorings',
      'supported_file_types',
      'features',
      'performance_metrics',
      'configuration_options',
    ];
    const missingFields: string[] = [];

    for (const field of requiredFields) {
      if (!(capabilities as any)[field]) {
        missingFields.push(field);
      }
    }

    return {
      valid: missingFields.length === 0,
      missingFields,
    };
  }

  /**
   * Merge capabilities with fallback values
   */
  static mergeWithFallbacks(capabilities: BackendCapabilitiesResponse | null): BackendCapabilities {
    if (!capabilities) {
      return this.getDefaultCapabilities();
    }

    const camelCaseCapabilities = this.convertBackendCapabilitiesToCamelCase(capabilities);

    // Merge with defaults to ensure all fields are present
    return {
      supportedRefactorings: camelCaseCapabilities.supportedRefactorings || [
        'rename',
        'extract-function',
      ],
      supportedFileTypes: camelCaseCapabilities.supportedFileTypes || ['rs', 'ts', 'js', 'py'],
      features: camelCaseCapabilities.features || this.getDefaultFeatures(),
      performanceMetrics: camelCaseCapabilities.performanceMetrics || {},
      configurationOptions: camelCaseCapabilities.configurationOptions || [],
    };
  }

  /**
   * Get default capabilities when backend is unavailable
   */
  static getDefaultCapabilities(): BackendCapabilities {
    return {
      supportedRefactorings: ['rename', 'extract-function', 'extract-variable'],
      supportedFileTypes: ['rs', 'ts', 'js', 'py'],
      features: this.getDefaultFeatures(),
      performanceMetrics: {},
      configurationOptions: [],
    };
  }

  /**
   * Get default features when backend is unavailable
   */
  private static getDefaultFeatures(): BackendFeatures {
    return {
      batchOperations: false,
      analysis: true,
      backupRecovery: false,
      testGeneration: false,
      aiAnalysis: false,
      lspIntegration: false,
      gitIntegration: false,
      crossLanguageSupport: false,
      parallelProcessing: false,
    };
  }

  /**
   * Check if a specific feature is available
   */
  static isFeatureAvailable(
    capabilities: BackendCapabilities,
    feature: keyof BackendFeatures
  ): boolean {
    return capabilities?.features[feature] || false;
  }

  /**
   * Get unified feature set for UI rendering decisions
   */
  static getUIFeatureSet(capabilities: BackendCapabilities): {
    showBatchOperations: boolean;
    showTestGeneration: boolean;
    showAnalysis: boolean;
    showPerformanceMetrics: boolean;
    showAdvancedFeatures: boolean;
  } {
    if (!capabilities) {
      return {
        showBatchOperations: false,
        showTestGeneration: false,
        showAnalysis: false,
        showPerformanceMetrics: false,
        showAdvancedFeatures: false,
      };
    }

    const features = capabilities.features;
    return {
      showBatchOperations: features.batchOperations,
      showTestGeneration: features.testGeneration,
      showAnalysis: features.analysis,
      showPerformanceMetrics: capabilities.performanceMetrics !== undefined,
      showAdvancedFeatures:
        features.aiAnalysis || features.lspIntegration || features.crossLanguageSupport,
    };
  }
}
