import { useEffect, useState, useCallback } from 'react';
import { languageServerClient } from './client';

declare global {
  interface ErrorConstructor {
    new (message?: string): Error;
    (message?: string): Error;
    readonly prototype: Error;
  }

  interface PromiseConstructor {
    new <T>(
      executor: (
        resolve: (value: T | PromiseLike<T>) => void,
        reject: (reason?: any) => void
      ) => void
    ): Promise<T>;
    all<T>(values: Iterable<T | PromiseLike<T>>): Promise<T[]>;
    // Add other Promise methods as needed
  }

  var Promise: PromiseConstructor;
}

interface LanguageServerState {
  isReady: boolean;
  error: Error | null;
  restart: () => Promise<boolean>;
}

type ServerState = {
  isReady: boolean;
  error: Error | null;
};

export function useLanguageServer(): LanguageServerState {
  const [state, setState] = useState<ServerState>({
    isReady: false,
    error: null,
  });

  const startServer = useCallback(async () => {
    try {
      if (!languageServerClient.isRunning()) {
        await languageServerClient.start();
      }
      setState({ isReady: true, error: null });
      return true;
    } catch (err) {
      console.error('Failed to start language server:', err);
      const error = err instanceof Error ? err : new Error('Failed to start language server');
      setState({ isReady: false, error });
      return false;
    }
  }, []);

  const restartServer = useCallback(async () => {
    try {
      await languageServerClient.stop();
      return await startServer();
    } catch (err) {
      console.error('Failed to restart language server:', err);
      const error = err instanceof Error ? err : new Error('Failed to restart language server');
      setState((prev) => ({ ...prev, error }));
      return false;
    }
  }, [startServer]);

  useEffect(() => {
    let isMounted = true;

    startServer();

    return () => {
      isMounted = false;
      // Client cleanup is handled by the client itself
    };
  }, [startServer]);

  return {
    isReady: state.isReady,
    error: state.error,
    restart: restartServer,
  };
}
