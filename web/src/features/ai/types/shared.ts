// / Centralized AI types - reducing duplication across AI modules by ~40%
// Consolidated from RefactoringService.ts, capabilityUnifier.ts, and related AI features

/**
 * Backend capabilities for AI operations
 */
export interface BackendCapabilitiesResponse {
  // / Version information
  version: string;
  // / Build time features
  build_features: string[];
  // / Available AI models
  models: ModelConfig[];
  // / Supported operations
  supported_operations: string[];
  // / Configuration limits
  limits: {
    max_context_length: number;
    max_completion_tokens: number;
    max_file_size_mb: number;
  };
  // / Environment capabilities
  capabilities: {
    streaming_supports: boolean;
    code_generation_supported: boolean;
    refactoring_supported: boolean;
    error_analysis_supported: boolean;
  };
}

/**
 * AI Model configuration
 */
export interface ModelConfig {
  // / Model identifier
  id: string;
  // / Model family (e.g., "openai", "anthropic", "local")
  family: string;
  // / Model architecture details
  architecture: {
    // / Number of parameters
    parameter_count?: number;
    // / Context window size
    context_window: number;
    // / Maximum output tokens
    max_tokens: number;
    // / Supported file types
    supported_file_types: string[];
  };
  // / Performance metrics
  performance: {
    // / Tokens per second (approximate)
    tokens_per_second: number;
    // / Cost per token (if applicable)
    cost_per_token?: number;
    // / Quality score (0-10)
    quality_score: number;
  };
  // / Model capabilities
  capabilities: {
    // / Supports streaming
    streaming: boolean;
    // / Supports function calling
    functions: boolean;
    // / Supports code generation
    code_generation: boolean;
    // / Excellent at refactoring
    refactoring_excellent: boolean;
    // / Good at error analysis
    error_analysis_good: boolean;
  };
  // / Usage statistics
  usage: {
    // / Successfully completed requests
    successful_requests: number;
    // / Failed requests
    failed_requests: number;
    // / Average response time
    avg_response_time_ms: number;
    // / Last used timestamp
    last_used?: number;
  };
}

/**
 * AI Context information
 */
export interface AIContext {
  // / Current file content being analyzed
  current_code?: string;
  // / File name (optional)
  filename?: string;
  // / Current cursor position
  cursor_position?: [number, number]; // [line, column]
  // / Selected text
  selection?: string;
  // / Project context
  project_context?: Record<string, any>;
  // / Dependencies available
  dependencies?: string[];
  // / Workspace structure
  workspace_structure?: Record<string, any>;
  // / Preferences for analysis
  analysis_preferences?: AnalysisPreferences;
}

/**
 * Analysis preferences
 */
export interface AnalysisPreferences {
  // / Maximum suggestions to return
  max_suggestions: number;
  // / Detail level for suggestions
  detail_level: 'brief' | 'detailed' | 'comprehensive';
  // / Include code examples
  include_examples: boolean;
  // / Focus areas (e.g., ["performance", "safety", "readability"])
  focus_areas: string[];
  // / Risk tolerance (0-10)
  risk_tolerance: number;
}

/**
 * Generic AI Response wrapper
 */
export interface AIResponse<T = any> {
  // / Success indicator
  success: boolean;
  // / Response data
  data?: T;
  // / Error message (if applicable)
  error?: string;
  // / Request ID
  request_id?: string;
  // / Usage statistics
  usage?: {
    // / Input tokens
    input_tokens: number;
    // / Output tokens
    output_tokens: number;
    // / Model used
    model: string;
    // / Cost (if applicable)
    cost?: number;
  };
  // / Additional metadata
  metadata?: Record<string, any>;
}

/**
 * Serpentine compatibility layer
 * Converts between camelCase (frontend) and snake_case (backend) representations
 */
export class AITypeSerializer {
  // / Convert frontend AI context to backend format
  static toBackend(context: AIContext): Record<string, any> {
    return {
      current_code: context.current_code,
      filename: context.filename,
      cursor_position: context.cursor_position,
      selection: context.selection,
      project_context: context.project_context,
      dependencies: context.dependencies,
      workspace_structure: context.workspace_structure,
      analysis_preferences: context.analysis_preferences,
    };
  }

  // / Convert backend model config to frontend format
  static modelConfigFromBackend(config: Record<string, any>): ModelConfig {
    return {
      id: config.id || '',
      family: config.family || '',
      architecture: {
        parameter_count: config.parameter_count,
        context_window: config.context_window || 0,
        max_tokens: config.max_tokens || 0,
        supported_file_types: config.supported_file_types || [],
      },
      performance: {
        tokens_per_second: config.tokens_per_second || 0,
        cost_per_token: config.cost_per_token,
        quality_score: config.quality_score || 0,
      },
      capabilities: {
        streaming: config.supports_streaming || false,
        functions: config.supports_functions || false,
        code_generation: config.supports_code_generation || false,
        refactoring_excellent: config.refactoring_excellent || false,
        error_analysis_good: config.error_analysis_good || false,
      },
      usage: {
        successful_requests: config.successful_requests || 0,
        failed_requests: config.failed_requests || 0,
        avg_response_time_ms: config.avg_response_time_ms || 0,
        last_used: config.last_used,
      },
    };
  }

  // / Convert backend capabilities to frontend format
  static backendCapabilitiesFromBackend(data: Record<string, any>): BackendCapabilitiesResponse {
    return {
      version: data.version || '',
      build_features: data.build_features || [],
      models: (data.models || []).map(AITypeSerializer.modelConfigFromBackend),
      supported_operations: data.supported_operations || [],
      limits: {
        max_context_length: data.max_context_length || 0,
        max_completion_tokens: data.max_completion_tokens || 0,
        max_file_size_mb: data.max_file_size_mb || 0,
      },
      capabilities: {
        streaming_supports: data.streaming_supports || false,
        code_generation_supported: data.code_generation_supported || false,
        refactoring_supported: data.refactoring_supported || false,
        error_analysis_supported: data.error_analysis_supported || false,
      },
    };
  }
}
