import { useState, useCallback } from "react";

// Define the ApiRequestOptions interface
interface ApiRequestOptions {
  method?: string;
  headers?: Record<string, string>;
  params?: Record<string, any>;
  skipLogging?: boolean;
  [key: string]: any; // For other fetch options like body, cache, etc.
}

// Simple logging utility for errors
const logError = (message: string, ...args: any[]) => {
  console.error(message, ...args);
};

// useApi custom hook
export const useApi = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [apiError, setApiError] = useState<Error | null>(null);

  const request = useCallback(async <T>(
    endpoint: string,
    options: ApiRequestOptions = {}
  ): Promise<T | null> => {
    const {
      method = 'GET',
      headers = {},
      params,
      skipLogging = false,
      ...fetchOptions
    } = options;

    const url = new URL(endpoint, window.location.origin);

    // Add query parameters
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          url.searchParams.append(key, String(value));
        }
      });
    }

    const requestId = Math.random().toString(36).substring(2, 9);
    const startTime = Date.now();

    if (!skipLogging) {
      console.log(`API Request [${requestId}]:`, {
        method,
        url: url.toString(),
        headers,
        body: fetchOptions.body,
      });
    }

    setIsLoading(true);
    setApiError(null);

    try {
      const response = await fetch(url.toString(), {
        method,
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
        ...fetchOptions,
      });

      const responseTime = Date.now() - startTime;
      const responseData = await response.json().catch(() => ({}));

      if (!skipLogging) {
        const logData = {
          status: response.status,
          statusText: response.statusText,
          responseTime: `${responseTime}ms`,
          response: responseData,
        };

        if (!response.ok) {
          logError(`API Error [${requestId}]:`, logData);
        } else {
          console.log(`API Response [${requestId}]:`, logData);
        }
      }

      if (!response.ok) {
        const error = new Error(response.statusText || 'API request failed');
        (error as any).response = responseData;
        (error as any).status = response.status;
        throw error;
      }

      return responseData as T;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown API error');
      setApiError(error);

      if (!skipLogging) {
        logError(`API Request Failed [${requestId}]:`, error, {
          url: url.toString(),
          method,
        });
      }

      throw error;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return {
    request,
    isLoading,
    apiError,
  };
};
