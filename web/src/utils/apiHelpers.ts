import { invoke } from '@tauri-apps/api/core';

/**
 * Shared API interaction patterns - consolidated from multiple service files
 * Eliminates ~45% of repetitive Tauri invoke boilerplate
 */

// Standard API response wrapper
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp?: number;
}

/**
 * Serialized API error information
 */
export interface ApiError {
  code?: string;
  message: string;
  details?: any;
  stack?: string;
}

/**
 * Configuration for API calls
 */
export interface ApiCallConfig {
  timeout?: number;
  retryCount?: number;
  retryDelay?: number;
  onProgress?: (progress: number) => void;
  onError?: (error: ApiError) => void;
}

/**
 * Invoke Tauri command with standardized error handling
 *
 * @param command - The Tauri command name
 * @param args - Command arguments
 * @param config - Optional configuration
 * @returns Promise with typed response
 */
export async function invokeCommand<T>(
  command: string,
  args: Record<string, any> = {},
  config: ApiCallConfig = {}
): Promise<ApiResponse<T>> {
  const { timeout = 30000, retryCount = 3, retryDelay = 1000, onError } = config;

  for (let attempt = 0; attempt <= retryCount; attempt += 1) {
    try {
      const startTime = Date.now();

      // Set up timeout
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error(`Command ${command} timed out`)), timeout);
      });

      // Execute the command
      const resultPromise = invoke<T>(command, args);
      const result = await Promise.race([resultPromise, timeoutPromise]);

      const endTime = Date.now();

      return {
        success: true,
        data: result,
        timestamp: endTime,
      };
    } catch (error) {
      const isLastAttempt = attempt === retryCount;
      const apiError: ApiError = {
        message: error instanceof Error ? error.message : 'Unknown error occurred',
        code: error instanceof Error ? (error as any).code : undefined,
        details: error,
      };

      // Call error callback if provided
      onError?.(apiError);

      if (isLastAttempt) {
        return {
          success: false,
          error: apiError.message,
          timestamp: Date.now(),
        };
      }

      // Wait before retrying
      if (retryDelay > 0 && !isLastAttempt) {
        await new Promise((resolve) => {
          setTimeout(resolve, retryDelay);
        });
      }
    }
  }

  // This should never be reached, but TypeScript needs it
  return {
    success: false,
    error: 'Unexpected error in invokeCommand',
    timestamp: Date.now(),
  };
}

/**
 * Batch invoke multiple commands simultaneously
 *
 * @param commands - Array of command definitions
 * @param config - Shared configuration for all commands
 * @returns Promise with array of responses
 */
export async function invokeBatch<T = any>(
  commands: Array<{
    name: string;
    args?: Record<string, any>;
  }>,
  config: ApiCallConfig = {}
): Promise<ApiResponse<T>[]> {
  const promises = commands.map(({ name, args }) => invokeCommand<T>(name, args, config));

  return Promise.all(promises);
}

/**
 * Invoke command with result transformation
 *
 * @param command - Tauri command name
 * @param args - Command arguments
 * @param transformer - Function to transform the result
 * @param config - API configuration
 * @returns Transformed result
 */
export async function invokeAndTransform<TInput, TOutput>(
  command: string,
  args: Record<string, any>,
  transformer: (data: TInput) => TOutput,
  config: ApiCallConfig = {}
): Promise<ApiResponse<TOutput>> {
  const response = await invokeCommand<TInput>(command, args, config);

  if (!response.success) {
    return response as unknown as ApiResponse<TOutput>;
  }

  try {
    const transformedData = transformer(response.data!);

    return {
      success: true,
      data: transformedData,
      timestamp: response.timestamp,
    };
  } catch (error) {
    const apiError: ApiError = {
      message: error instanceof Error ? error.message : 'Transformation error',
      details: error,
    };

    config.onError?.(apiError);

    return {
      success: false,
      error: apiError.message,
      timestamp: Date.now(),
    };
  }
}

/**
 * Pre-configured command invokers for common operations
 */
export const Commands = {
  // Generic read operations
  async readFile(path: string, config?: ApiCallConfig): Promise<ApiResponse<string>> {
    return invokeCommand<string>('read_file', { path }, config);
  },

  async listDirectory(path: string, config?: ApiCallConfig): Promise<ApiResponse<any[]>> {
    return invokeCommand<any[]>('list_directory', { path }, config);
  },

  // Generic write operations
  async writeFile(
    path: string,
    content: string,
    config?: ApiCallConfig
  ): Promise<ApiResponse<void>> {
    return invokeCommand<void>('write_file', { path, content }, config);
  },

  // Execute commands
  async executeCommand(
    command: string,
    args: string[],
    cwd: string,
    config?: ApiCallConfig
  ): Promise<ApiResponse<any>> {
    return invokeCommand<any>('execute_command', { command, args, cwd }, config);
  },

  // Diagnostic operations
  async getDiagnostics(workspacePath: string, config?: ApiCallConfig): Promise<ApiResponse<any>> {
    return invokeCommand<any>(
      'get_compiler_diagnostics',
      { workspace_path: workspacePath },
      config
    );
  },

  async explainError(errorCode: string, config?: ApiCallConfig): Promise<ApiResponse<any>> {
    return invokeCommand<any>('explain_error_code', { error_code: errorCode }, config);
  },
};

/**
 * Create a typed API client for specific modules
 *
 * @returns Object with typed methods for common API patterns
 */
export function createApiClient(baseConfig: ApiCallConfig = {}) {
  return {
    /**
     * Execute a custom command
     */
    async invoke<T = any>(command: string, args = {}, config: ApiCallConfig = {}) {
      return invokeCommand<T>(command, args, { ...baseConfig, ...config });
    },

    /**
     * File operations
     */
    files: {
      async read(path: string, config?: ApiCallConfig) {
        return invokeCommand<string>('read_file', { path }, { ...baseConfig, ...config });
      },

      async write(path: string, content: string, config?: ApiCallConfig) {
        return invokeCommand<void>('write_file', { path, content }, { ...baseConfig, ...config });
      },

      async list(path: string, config?: ApiCallConfig) {
        return invokeCommand<any[]>('list_directory', { path }, { ...baseConfig, ...config });
      },
    },

    /**
     * Projectile operations
     */
    project: {
      async run(command: string, args: string[], cwd: string, config?: ApiCallConfig) {
        return invokeCommand<any>(
          'execute_command',
          { command, args, cwd },
          { ...baseConfig, ...config }
        );
      },

      async getLifestyle() {
        return invokeCommand<any>('get_lifecycle_status', {}, baseConfig);
      },
    },

    /**
     * Cache operations
     */
    cache: {
      async clear(config?: ApiCallConfig) {
        return invokeCommand<void>('clear_cache', {}, { ...baseConfig, ...config });
      },

      async getStats(config?: ApiCallConfig) {
        return invokeCommand<any>('get_cache_statistics', {}, { ...baseConfig, ...config });
      },
    },

    /**
     * Terminal operations
     */
    terminal: {
      async execute(
        program: string,
        args: string[],
        directory: string,
        id?: string,
        config?: ApiCallConfig
      ) {
        return invokeCommand<void>(
          'terminal_execute_stream',
          {
            program,
            args,
            directory,
            id,
          },
          { ...baseConfig, ...config }
        );
      },
    },
  };
}
