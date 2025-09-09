// Consolidated utilities for frontend components
// Provides common patterns and helpers to reduce duplication

// ==========================================
// ERROR HANDLING UTILITIES
// ==========================================

/**
 * Creates a context-specific error handler function
 * @param context - The operation context for logging
 * @returns Error handler function
 */
export const createErrorHandler = (context: string) => {
  return (error: unknown, customMessage?: string): string => {
    const message = customMessage ||
      (error instanceof Error ? error.message : 'An unknown error occurred');

    // Using console.error as standard practice, could be replaced with logging service
    console.error(`Error in ${context}:`, error);

    return message;
  };
};

/**
 * Formats loading state messages consistently
 * @param isLoading - Whether operation is loading
 * @param customText - Custom loading text
 * @returns Formatted loading message or null
 */
export const formatLoadingState = (isLoading: boolean, customText = 'Processing...'): string | null => {
  return isLoading ? customText : null;
};

/**
 * Handles async operations with consistent error and loading state management
 * @param operation - Async function to execute
 * @param options - Configuration options
 * @returns Promise that resolves when operation completes
 */
export const handleAsyncOperation = async <T>(
  operation: () => Promise<T>,
  options: {
    onLoading?: (loading: boolean) => void;
    onError?: (error: string) => void;
    onSuccess?: (result: T) => void;
    errorContext?: string;
  } = {}
): Promise<T | null> => {
  const { onLoading, onError, onSuccess, errorContext = 'operation' } = options;
  const errorHandler = createErrorHandler(errorContext);

  try {
    onLoading?.(true);
    const result = await operation();
    onSuccess?.(result);
    return result;
  } catch (error) {
    const errorMessage = errorHandler(error);
    onError?.(errorMessage);
    return null;
  } finally {
    onLoading?.(false);
  }
};

// ==========================================
// STATE MANAGEMENT HELPERS
// ==========================================

/**
 * Creates a state updater function for object state management
 * @param setState - React setState function
 * @returns Updater function that merges new data with existing state
 */
export const createStateUpdater = <T extends Record<string, any>>(
  setState: React.Dispatch<React.SetStateAction<T>>
) => {
  return (updates: Partial<T>) => {
    setState(prev => ({ ...prev, ...updates }));
  };
};

/**
 * Creates a toggle handler for boolean state
 * @param setState - React setState function
 * @param field - The boolean field to toggle
 * @returns Toggle function
 */
export const createToggleHandler = <T extends Record<string, any>>(
  setState: React.Dispatch<React.SetStateAction<T>>,
  field: keyof T
) => {
  return () => {
    setState(prev => ({ ...prev, [field]: !prev[field] }));
  };
};

// ==========================================
// FORMATTING UTILITIES
// ==========================================

/**
 * Formats file paths consistently
 * @param path - File path to format
 * @param rootPath - Optional root path to resolve against
 * @returns Formatted path string
 */
export const formatFilePath = (path: string, rootPath?: string): string => {
  if (rootPath && path.startsWith(rootPath)) {
    return path.replace(rootPath, '').replace(/^\/+/, '');
  }
  return path.replace(/\\/g, '/').replace(/^\/+/, '');
};

/**
 * Formats duration in milliseconds to human readable string
 * @param ms - Duration in milliseconds
 * @returns Formatted duration string
 */
export const formatDuration = (ms?: number): string => {
  if (!ms || ms < 0) return 'N/A';

  const seconds = Math.round(ms / 1000);
  if (seconds < 60) return `${seconds}s`;

  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}m ${remainingSeconds}s`;
};

/**
 * Formats large numbers with appropriate units
 * @param num - Number to format
 * @returns Formatted string with units
 */
export const formatLargeNumber = (num?: number): string => {
  if (!num) return '0';

  if (num >= 1e9) return `${(num / 1e9).toFixed(1)}B`;
  if (num >= 1e6) return `${(num / 1e6).toFixed(1)}M`;
  if (num >= 1e3) return `${(num / 1e3).toFixed(1)}K`;
  return num.toString();
};

// ==========================================
// VALIDATION HELPERS
// ==========================================

/**
 * Validates text input for common patterns
 */
export const validators = {
  isNotEmpty: (value: string): boolean => value.trim().length > 0,
  isEmail: (value: string): boolean => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value),
  isValidPath: (value: string): boolean => /^[^\0\n\r\f\v<>*?"|]*$/.test(value),
  hasMinLength: (min: number) => (value: string): boolean => value.length >= min,
};

/**
 * Creates a validation result object
 */
export const createValidationResult = (isValid: boolean, message: string) => ({
  isValid,
  message,
});

// ==========================================
// COMMON CONSTANTS
// ==========================================

/**
 * Standard panel dimensions and spacing
 */
export const PANEL_CONSTANTS = {
  PADDING: 2,
  MAX_HEIGHT: '100%',
  HEADER_HEIGHT: 64,
  TOOLBAR_HEIGHT: 48,
} as const;

/**
 * Standard animation durations
 */
export const ANIMATION_CONSTANTS = {
  FAST: '0.1s',
  NORMAL: '0.2s',
  SLOW: '0.4s',
} as const;

/**
 * Standard breakpoints (matching Material-UI)
 */
export const BREAKPOINT_CONSTANTS = {
  xs: 0,
  sm: 600,
  md: 960,
  lg: 1280,
  xl: 1920,
} as const;

// ==========================================
// TYPE HELPERS
// ==========================================

/**
 * Creates a discriminated union for async states
 */
export type AsyncState<T> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: T }
  | { status: 'error'; error: string };

/**
 * Standard component props interface
 */
export interface BaseComponentProps {
  className?: string;
  children?: React.ReactNode;
  style?: React.CSSProperties;
  'data-testid'?: string;
}

/**
 * Standard panel props interface
 */
export interface BasePanelProps extends BaseComponentProps {
  title?: string;
  headerActions?: React.ReactNode;
  fullHeight?: boolean;
  padded?: boolean;
}