import { useState, useCallback, useEffect } from 'react';

/**
 * Async state type for managing operation states
 */
export type AsyncState<T> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: T }
  | { status: 'error'; error: string };

/**
 * Configuration options for async operation hook
 */
export interface UseAsyncOptions<T> {
  /** Function to execute on success */
  onSuccess?: (data: T) => void;
  /** Function to execute on error */
  onError?: (error: string) => void;
  /** Whether to run operation immediately */
  immediate?: boolean;
  /** Initial data value */
  initialData?: T;
  /** Custom error handler context */
  errorContext?: string;
}

/**
 * Return type for useAsync hook
 */
export interface UseAsyncReturn<TData, TArgs extends any[] = []> {
  /** Current async state */
  state: AsyncState<TData>;
  /** Function to execute the async operation */
  execute: (...args: TArgs) => Promise<TData | null>;
  /** Function to reset the state */
  reset: () => void;
  /** Function to retry the last operation */
  retry: () => Promise<TData | null>;
  /** Current loading state */
  isLoading: boolean;
  /** Current error state */
  error: string | null;
  /** Current data state */
  data: TData | null;
}

/**
 * Hook for managing async operations with consistent loading/error/success states
 *
 * @param asyncFn - The async function to execute
 * @param options - Configuration options
 * @returns Object with state management functions
 *
 * @example
 * ```tsx
 * const { execute, state, isLoading, error } = useAsync(
 *   (userId: string) => api.getUser(userId),
 *   { onSuccess: (user) => setSelectedUser(user) }
 * );
 *
 * return (
 *   <button onClick={() => execute('123')} disabled={isLoading}>
 *     {isLoading ? 'Loading...' : 'Load User'}
 *   </button>
 * );
 * ```
 */
export function useAsync<TData = any, TArgs extends any[] = []>(
  asyncFn: (...args: TArgs) => Promise<TData>,
  options: UseAsyncOptions<TData> = {}
): UseAsyncReturn<TData, TArgs> {
  const {
    onSuccess,
    onError,
    immediate = false,
    initialData,
    errorContext = 'operation',
  } = options;

  const [state, setState] = useState<AsyncState<TData>>(
    initialData !== undefined ? { status: 'success', data: initialData } : { status: 'idle' }
  );

  const [lastArgs, setLastArgs] = useState<TArgs | null>(null);

  const execute = useCallback(
    async (...args: TArgs): Promise<TData | null> => {
      setLastArgs(args as TArgs);
      setState({ status: 'loading' });

      try {
        const data = await asyncFn(...args);
        setState({ status: 'success', data });
        onSuccess?.(data);
        return data;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'An error occurred';
        setState({ status: 'error', error: errorMessage });
        onError?.(errorMessage);
        return null;
      }
    },
    [asyncFn, onSuccess, onError]
  );

  const reset = useCallback(() => {
    setState(
      initialData !== undefined ? { status: 'success', data: initialData } : { status: 'idle' }
    );
    setLastArgs(null);
  }, [initialData]);

  const retry = useCallback(async (): Promise<TData | null> => {
    if (lastArgs) {
      return execute(...lastArgs);
    }
    return Promise.resolve(null);
  }, [lastArgs, execute]);

  // Execute immediately if requested
  useEffect(() => {
    if (immediate) {
      // Only execute immediately if the function doesn't expect any arguments
      if (asyncFn.length === 0) {
        // Use type assertion to handle the empty arguments case
        (execute as () => Promise<TData | null>)();
      }
    }
  }, [immediate, execute, asyncFn]);

  return {
    state,
    execute,
    reset,
    retry,
    isLoading: state.status === 'loading',
    error: state.status === 'error' ? state.error : null,
    data: state.status === 'success' ? state.data : null,
  };
}

/**
 * Simplified hook for one-time async operations
 */
export function useLazyAsync<TData = any, TArgs extends any[] = []>(
  asyncFn: (...args: TArgs) => Promise<TData>,
  options: Omit<UseAsyncOptions<TData>, 'immediate'> = {}
): UseAsyncReturn<TData, TArgs> & { called: boolean } {
  const hook = useAsync(asyncFn, options);

  return {
    ...hook,
    called: hook.state.status !== 'idle',
  };
}

/**
 * Hook for parallel async operations
 */
export function useAsyncMultiple<TData = any>() {
  const [state, setState] = useState<AsyncState<TData[]>>({ status: 'idle' });

  const execute = useCallback(
    async (operations: Array<() => Promise<TData>>): Promise<TData[] | null> => {
      setState({ status: 'loading' });

      try {
        const results = await Promise.all(operations.map((op) => op()));
        setState({ status: 'success', data: results });
        return results;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'An error occurred';
        setState({ status: 'error', error: errorMessage });
        return null;
      }
    },
    []
  );

  const reset = useCallback(() => {
    setState({ status: 'idle' });
  }, []);

  return {
    state,
    execute,
    reset,
    isLoading: state.status === 'loading',
    error: state.status === 'error' ? state.error : null,
    data: state.status === 'success' ? state.data : null,
  };
}
