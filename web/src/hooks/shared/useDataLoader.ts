import { useState, useCallback, useEffect, useRef } from 'react';

/**
 * Configuration options for data loader hook
 */
export interface UseDataLoaderOptions<T> {
  /** Function to fetch data */
  fetchFn: () => Promise<T>;
  /** Whether to fetch data immediately */
  immediate?: boolean;
  /** Auto-refresh interval in milliseconds */
  refreshInterval?: number;
  /** Whether to cache the result */
  enableCache?: boolean;
  /** Cache key for this data */
  cacheKey?: string;
  /** Transform function for data */
  transform?: (data: T) => T;
  /** Dependencies for re-fetching */
  deps?: any[];
}

/**
 * Return type for useDataLoader hook
 */
export interface UseDataLoaderReturn<T> {
  /** Current data */
  data: T | null;
  /** Loading state */
  loading: boolean;
  /** Error state */
  error: string | null;
  /** Function to manually refresh data */
  refresh: () => Promise<T | null>;
  /** Function to clear data */
  clear: () => void;
  /** Last fetch timestamp */
  lastFetch: number | null;
}

/**
 * Cache for data loader hooks
 */
const cache = new Map<string, { data: any; timestamp: number; ttl: number }>();

/**
 * Hook for managing data loading patterns with caching and auto-refresh
 *
 * @param options - Configuration options
 * @returns Data loading state and control functions
 *
 * @example
 * ```tsx
 * const { data, loading, error, refresh } = useDataLoader({
 *   fetchFn: () => api.getUsers(),
 *   immediate: true,
 *   enableCache: true,
 *   cacheKey: 'users',
 *   refreshInterval: 30000 // 30 seconds
 * });
 *
 * if (loading) return <CircularProgress />;
 * if (error) return <Alert severity="error">{error}</Alert>;
 * return <UserList users={data || []} onRefresh={refresh} />;
 * ```
 */
export function useDataLoader<T>(
  options: UseDataLoaderOptions<T>
): UseDataLoaderReturn<T> {
  const {
    fetchFn,
    immediate = false,
    refreshInterval,
    enableCache = false,
    cacheKey,
    transform,
    deps = []
  } = options;

  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastFetch, setLastFetch] = useState<number | null>(null);

  const refreshTimerRef = useRef<NodeJS.Timeout | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  // Load from cache if available
  const loadFromCache = useCallback(() => {
    if (!enableCache || !cacheKey) return null;

    const cached = cache.get(cacheKey);
    if (cached) {
      const now = Date.now();
      if (now - cached.timestamp < cached.ttl) {
        return cached.data;
      } else {
        cache.delete(cacheKey);
      }
    }
    return null;
  }, [enableCache, cacheKey]);

  // Save to cache
  const saveToCache = useCallback((data: T, ttl = 5 * 60 * 1000) => {
    if (enableCache && cacheKey) {
      cache.set(cacheKey, {
        data,
        timestamp: Date.now(),
        ttl
      });
    }
  }, [enableCache, cacheKey]);

  const fetchData = useCallback(async (): Promise<T | null> => {
    // Cancel any existing request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    abortControllerRef.current = new AbortController();

    setLoading(true);
    setError(null);

    try {
      // Check cache first
      const cachedData = loadFromCache();
      if (cachedData !== null) {
        setData(transform ? transform(cachedData) : cachedData);
        setLoading(false);
        setLastFetch(Date.now());
        return cachedData;
      }

      // Fetch new data
      const result = await fetchFn();

      // Apply transformation
      const transformedData = transform ? transform(result) : result;

      // Update state
      setData(transformedData);
      setLastFetch(Date.now());

      // Save to cache
      saveToCache(transformedData);

      return transformedData;
    } catch (err) {
      if (err instanceof Error && err.name !== 'AbortError') {
        const errorMessage = err.message || 'Failed to fetch data';
        setError(errorMessage);
        console.error('useDataLoader error:', err);
      }
      return null;
    } finally {
      setLoading(false);
    }
  }, [fetchFn, loadFromCache, saveToCache, transform]);

  const refresh = useCallback(async (): Promise<T | null> => {
    // Clear cache to force fresh fetch
    if (enableCache && cacheKey) {
      cache.delete(cacheKey);
    }
    return fetchData();
  }, [fetchData, enableCache, cacheKey]);

  const clear = useCallback(() => {
    setData(null);
    setError(null);
    setLastFetch(null);
    if (enableCache && cacheKey) {
      cache.delete(cacheKey);
    }

    // Cancel any ongoing request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    // Clear refresh timer
    if (refreshTimerRef.current) {
      clearInterval(refreshTimerRef.current);
      refreshTimerRef.current = null;
    }
  }, [enableCache, cacheKey]);

  // Setup auto-refresh
  useEffect(() => {
    if (refreshInterval && refreshInterval > 0) {
      refreshTimerRef.current = setInterval(refresh, refreshInterval);
      return () => {
        if (refreshTimerRef.current) {
          clearInterval(refreshTimerRef.current);
        }
      };
    }
  }, [refreshInterval, refresh]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
      if (refreshTimerRef.current) {
        clearInterval(refreshTimerRef.current);
      }
    };
  }, []);

  // Initial fetch
  useEffect(() => {
    if (immediate) {
      fetchData();
    } else {
      // Try to load from cache even if not immediate
      const cachedData = loadFromCache();
      if (cachedData !== null) {
        setData(transform ? transform(cachedData) : cachedData);
        setLastFetch(Date.now());
      }
    }
    // Note: We can't include fetchData and loadFromCache in deps as they change on every render
    // We rely on the deps array provided by the caller to know when to refetch
  }, [immediate, ...deps]);

  return {
    data,
    loading,
    error,
    refresh,
    clear,
    lastFetch
  };
}

/**
 * Hook for managing multiple data loaders
 */
export function useMultipleDataLoaders<T>(
  loaders: Array<{ key: string; options: UseDataLoaderOptions<T> }>
) {
  const results = loaders.map(loader =>
    useDataLoader(loader.options)
  );

  const loading = results.some(r => r.loading);
  const error = results.find(r => r.error)?.error || null;
  const lastFetch = Math.max(...results.map(r => r.lastFetch || 0));

  const refreshAll = useCallback(async () => {
    const promises = results.map(r => r.refresh());
    await Promise.all(promises);
  }, [results]);

  const clearAll = useCallback(() => {
    results.forEach(r => r.clear());
  }, [results]);

  return {
    loaders: Object.fromEntries(
      loaders.map((loader, index) => [loader.key, results[index]])
    ),
    loading,
    error,
    lastFetch: lastFetch > 0 ? lastFetch : null,
    refreshAll,
    clearAll
  };
}