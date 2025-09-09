import { useState, useCallback } from 'react';

/**
 * Generic async operation state
 */
type AsyncState<T> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: T }
  | { status: 'error'; error: string };

/**
 * Configuration options for async operations
 */
interface AsyncOperationOptions {
  errorContext?: string;
  retryDelay?: number;
  retryCount?: number;
  onLoading?: (loading: boolean) => void;
  onSuccess?: (data: any) => void;
  onError?: (error: string) => void;
}

/**
 * Consolidated async operation hook
 *
 * Provides unified loading states, error handling, and retry logic
 * Eliminates duplicate async state management patterns across components
 */
export function useAsyncOperation<TData = any>(
  asyncFn: () => Promise<TData>,
  options: AsyncOperationOptions = {}
) {
  const {
    errorContext = 'operation',
    retryDelay = 1000,
    retryCount = 3,
    onLoading,
    onSuccess,
    onError,
  } = options;

  const [state, setState] = useState<AsyncState<TData>>({ status: 'idle' });
  const [retryAttempts, setRetryAttempts] = useState(0);

  const createErrorHandler = useCallback((context: string) => {
    return (error: unknown) => {
      console.error(`Error in ${context}:`, error);
      const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred';
      return errorMessage;
    };
  }, []);

  const execute = useCallback(async (): Promise<TData | null> => {
    setState({ status: 'loading' });
    onLoading?.(true);

    let lastError: string | null = null;

    for (let attempt = 0; attempt <= retryCount; attempt++) {
      try {
        const data = await asyncFn();
        setState({ status: 'success', data });
        setRetryAttempts(0);
        onSuccess?.(data);
        onLoading?.(false);
        return data;
      } catch (error) {
        const errorMessage = createErrorHandler(errorContext)(error);
        lastError = errorMessage;

        if (attempt < retryCount) {
          console.log(`Retrying ${errorContext} (attempt ${attempt + 1}/${retryCount + 1}) in ${retryDelay}ms...`);
          await new Promise(resolve => setTimeout(resolve, retryDelay));
          continue;
        }

        setState({ status: 'error', error: errorMessage });
        setRetryAttempts(attempt);
        onError?.(errorMessage);
        onLoading?.(false);
        return null;
      }
    }

    return null;
  }, [asyncFn, errorContext, retryCount, retryDelay, onLoading, onSuccess, onError, createErrorHandler]);

  const reset = useCallback(() => {
    setState({ status: 'idle' });
    setRetryAttempts(0);
  }, []);

  const retry = useCallback(() => {
    if (state.status === 'error' || state.status === 'idle') {
      execute();
    }
  }, [state.status, execute]);

  return {
    ...state,
    execute,
    reset,
    retry,
    retryAttempts,
    isIdle: state.status === 'idle',
    isLoading: state.status === 'loading',
    isSuccess: state.status === 'success',
    isError: state.status === 'error',
    data: state.status === 'success' ? state.data : undefined,
    error: state.status === 'error' ? state.error : undefined,
  };
}

/**
 * Hook for handling multiple concurrent async operations
 */
interface AsyncOperationsState {
  [key: string]: AsyncState<any>;
}

export function useAsyncOperations<TData = any>(
  operations: Record<string, () => Promise<TData>>,
  options: AsyncOperationOptions = {}
) {
  const [states, setStates] = useState<AsyncOperationsState>({});

  const executeAll = useCallback(async () => {
    const promises = Object.entries(operations).map(async ([key, asyncFn]) => {
      setStates(prev => ({ ...prev, [key]: { status: 'loading' } }));

      try {
        const result = await useAsyncOperation(asyncFn, options).execute();
        setStates(prev => ({
          ...prev,
          [key]: result ? { status: 'success', data: result } : { status: 'idle' }
        }));
      } catch (error) {
        setStates(prev => ({
          ...prev,
          [key]: { status: 'error', error: error instanceof Error ? error.message : 'Unknown error' }
        }));
      }
    });

    await Promise.all(promises);
  }, [operations, options]);

  const execute = useCallback(async (key: string) => {
    const asyncFn = operations[key];
    if (asyncFn) {
      setStates(prev => ({ ...prev, [key]: { status: 'loading' } }));

      try {
        const result = await asyncFn();
        setStates(prev => ({
          ...prev,
          [key]: { status: 'success', data: result }
        }));
        options.onSuccess?.(result);
        return result;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error';
        setStates(prev => ({
          ...prev,
          [key]: { status: 'error', error: errorMessage }
        }));
        options.onError?.(errorMessage);
        console.error(`Error in operation ${key}:`, error);
        return null;
      }
    }
    return null;
  }, [operations, options]);

  const reset = useCallback((key?: string) => {
    if (key) {
      setStates(prev => ({ ...prev, [key]: { status: 'idle' } }));
    } else {
      const resetStates: AsyncOperationsState = {};
      Object.keys(states).forEach(k => {
        resetStates[k] = { status: 'idle' };
      });
      setStates(resetStates);
    }
  }, [states]);

  return {
    states,
    execute,
    executeAll,
    reset,
    isAnyLoading: Object.values(states).some(state => state.status === 'loading'),
    isAllSuccess: Object.values(states).every(state => state.status === 'success'),
    hasAnyError: Object.values(states).some(state => state.status === 'error'),
  };
}