import { invoke } from '@tauri-apps/api/core';

/**
 * Code generation request types
 */
export interface GenerateFunctionRequest {
  original_function?: string;
  function_purpose: string;
  target_language:
    | 'Rust'
    | 'Python'
    | 'TypeScript'
    | 'JavaScript'
    | 'Go'
    | 'Java'
    | 'C++'
    | 'SQL'
    | 'HTML'
    | 'CSS';
  parameters?: string[];
  return_type?: string;
  similar_functions?: string[];
  error_handling?: boolean;
  performance_requirements?: string;
  safety_requirements?: string;
}

export interface ValidateCodeRequest {
  code: string;
  language:
    | 'Rust'
    | 'Python'
    | 'TypeScript'
    | 'JavaScript'
    | 'Go'
    | 'Java'
    | 'C++'
    | 'SQL'
    | 'HTML'
    | 'CSS';
}

export interface BatchGenerateRequest {
  requests: GenerateFunctionRequest[];
}

export interface GenerationTemplate {
  name: string;
  version: string;
  language_support: string[];
  description: string;
}

/**
 * Generated function response types
 */
export interface GeneratedFunction {
  name: string;
  signature: string;
  body: string;
  imports: string[];
  documentation: string;
  tests: string[];
  complexity: number;
  confidence_score: number;
  language?: string;
  code: string;
  parameters: string[];
  return_type?: string;
}

export interface ValidationResult {
  success: boolean;
  readability_score: number;
  maintainability_score: number;
  performance_score: number;
  security_score: number;
  compliance_score: number;
  overall_score: number;
  issues: ValidationIssue[];
  timestamp: number;
}

export interface ValidationIssue {
  category: string;
  severity: string;
  message: string;
  suggestion: string;
}

export interface GenerationResult {
  success: boolean;
  generated_function?: GeneratedFunction;
  error?: string;
  timestamp: number;
}

export interface BatchGenerationResult {
  success: boolean;
  total_requests: number;
  successful: number;
  errors: number;
  results: BatchGenerationItem[];
  timestamp: number;
}

export interface BatchGenerationItem {
  index: number;
  success: boolean;
  generated_function?: GeneratedFunction;
  error?: string;
  original_request?: GenerateFunctionRequest;
}

export interface SupportedLanguagesResult {
  success: boolean;
  supported_languages: string[];
  generator_info: {
    name: string;
    version: string;
    description: string;
    author: string;
  };
  timestamp: number;
}

export interface GenerationTemplatesResult {
  success: boolean;
  language: string;
  templates: GenerationTemplate[];
  message?: string;
  timestamp: number;
}

/**
 * AI Code Generation Service
 * Provides typed interface to Tauri code generation commands
 */
class CodegenService {
  private eventListeners: Map<string, Array<(event: any) => void>> = new Map();

  constructor() {
    this.setupEventListeners();
  }

  /**
   * Generate a single function using AI
   */
  async generateFunction(request: GenerateFunctionRequest): Promise<GenerationResult> {
    try {
      console.log('Generating function:', request.function_purpose, 'in', request.target_language);
      const result = await invoke<GenerationResult>('generate_function', { request });
      console.log('Function generation completed:', result.success ? 'success' : 'failed');
      return result;
    } catch (error) {
      console.error('Function generation failed:', error);
      throw new Error(`Function generation failed: ${error}`);
    }
  }

  /**
   * Validate generated code quality
   */
  async validateCode(request: ValidateCodeRequest): Promise<ValidationResult> {
    try {
      console.log('Validating code for language:', request.language);
      const result = await invoke<ValidationResult>('validate_generated_code', {
        request: {
          code: request.code,
          language: request.language,
        },
      });
      console.log('Code validation completed with score:', result.overall_score);
      return result;
    } catch (error) {
      console.error('Code validation failed:', error);
      throw new Error(`Code validation failed: ${error}`);
    }
  }

  /**
   * Get supported programming languages
   */
  async getSupportedLanguages(): Promise<SupportedLanguagesResult> {
    try {
      console.log('Fetching supported languages');
      const result = await invoke<SupportedLanguagesResult>('get_supported_languages');
      console.log(
        'Supported languages fetched:',
        result.supported_languages?.length || 0,
        'languages'
      );
      return result;
    } catch (error) {
      console.error('Failed to get supported languages:', error);
      throw new Error(`Failed to get supported languages: ${error}`);
    }
  }

  /**
   * Get generation templates for a specific language
   */
  async getGenerationTemplates(language: string): Promise<GenerationTemplatesResult> {
    try {
      console.log('Getting generation templates for:', language);
      const result = await invoke<GenerationTemplatesResult>('get_generation_templates', {
        request: { language },
      });
      console.log(
        'Templates fetched for',
        language,
        ':',
        result.templates?.length || 0,
        'templates'
      );
      return result;
    } catch (error) {
      console.error('Failed to get generation templates:', error);
      throw new Error(`Failed to get generation templates: ${error}`);
    }
  }

  /**
   * Batch generate multiple functions
   */
  async batchGenerateFunctions(request: BatchGenerateRequest): Promise<BatchGenerationResult> {
    try {
      console.log('Batch generating functions:', request.requests.length, 'requests');
      const result = await invoke<BatchGenerationResult>('batch_generate_functions', { request });
      console.log(
        'Batch generation completed:',
        result.successful,
        'successful,',
        result.errors,
        'errors'
      );
      return result;
    } catch (error) {
      console.error('Batch generation failed:', error);
      throw new Error(`Batch generation failed: ${error}`);
    }
  }

  /**
   * Set up event listeners for real-time updates
   */
  private setupEventListeners(): void {
    // Listen for Tauri events related to code generation operations
    // This would be expanded based on actual events emitted from Rust backend
  }

  /**
   * Emit custom events to React components
   */
  emitEvent(event: any): void {
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
   * Subscribe to code generation service events
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
   * Check if code generation services are available
   */
  async checkAvailability(): Promise<{ available: boolean; error?: string }> {
    try {
      // Quick health check by trying to get supported languages
      await this.getSupportedLanguages();
      return { available: true };
    } catch (error) {
      console.warn('Code generation services check failed:', error);
      return {
        available: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }
}

// Singleton instance
export const codegenService = new CodegenService();

// Default export for convenience
export default codegenService;

/**
 * Utility functions for React integration
 */

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
    console.error('Codegen Service API call failed:', error);
    onError?.(error as Error);
    throw error;
  }
}

/**
 * Create a typed API response wrapper
 */
export function createAPIResponse<T>(data: T, error?: string) {
  return {
    success: !error,
    data: error ? undefined : data,
    error,
    timestamp: Date.now(),
  };
}

/**
 * Helper function to format validation score as percentage
 */
export function formatValidationScore(score: number): string {
  return `${Math.round(score * 100)}%`;
}

/**
 * Helper function to get validation score color
 */
export function getValidationScoreColor(score: number): string {
  if (score >= 0.8) return 'text-green-600';
  if (score >= 0.6) return 'text-yellow-600';
  return 'text-red-600';
}

/**
 * Helper function to get issue severity color
 */
export function getIssueSeverityColor(severity: string): string {
  switch (severity.toLowerCase()) {
    case 'high':
    case 'critical':
      return 'text-red-600';
    case 'medium':
    case 'warning':
      return 'text-yellow-600';
    case 'low':
    case 'info':
      return 'text-blue-600';
    default:
      return 'text-gray-600';
  }
}
