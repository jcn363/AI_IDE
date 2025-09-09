import { useState, useCallback } from 'react';

/**
 * Hook for managing async operation state including loading, error, and data states
 */
export function useAsyncState<T = any>() {
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<T | null>(null);

  const executeAsync = useCallback(async <TArgs extends any[]>(
    asyncFn: (...args: TArgs) => Promise<T>,
    ...args: TArgs
  ): Promise<T | null> => {
    setIsLoading(true);
    setError(null);
    setData(null);

    try {
      const result = await asyncFn(...args);
      setData(result as T);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const reset = useCallback(() => {
    setIsLoading(false);
    setError(null);
    setData(null);
  }, []);

  return {
    isLoading,
    error,
    data,
    executeAsync,
    reset,
    setData,
    setError,
  };
}

/**
 * Hook for simple loading/error state management
 */
export function useSimpleAsyncState() {
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const startAsync = useCallback(async (asyncFn: () => Promise<void>): Promise<boolean> => {
    setIsLoading(true);
    setError(null);

    try {
      await asyncFn();
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const reset = useCallback(() => {
    setIsLoading(false);
    setError(null);
  }, []);

  return {
    isLoading,
    error,
    startAsync,
    reset,
    setError,
  };
}