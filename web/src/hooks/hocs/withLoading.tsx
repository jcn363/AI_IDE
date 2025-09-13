import React, { ComponentType, ReactNode } from 'react';
import { Box, CircularProgress, Skeleton, Typography, Alert, Button } from '@mui/material';

/**
 * Loading configuration options
 */
export interface LoadingConfig {
  /** Type of loading indicator */
  type?: 'spinner' | 'skeleton' | 'text' | 'custom';
  /** Custom loading message */
  message?: string;
  /** Whether loading state blocks interaction */
  blocking?: boolean;
  /** Size of spinner (if type is spinner) */
  size?: number | 'small' | 'medium' | 'large';
  /** Custom component for loading state */
  customComponent?: ReactNode;
  /** Number of skeleton lines (if type is skeleton) */
  skeletonLines?: number;
  /** Failback component when there are errors */
  errorFallback?: ReactNode;
}

/**
 * Loading state HOC options
 */
export interface WithLoadingOptions {
  /** Configuration for the loading state */
  loadingConfig?: LoadingConfig;
  /** Whether to show retry button on errors */
  showRetryButton?: boolean;
  /** Custom retry handler */
  onRetry?: () => void;
  /** Custom error message formatter */
  formatErrorMessage?: (error: string) => string;
}

/**
 * Props that components wrapped with withLoading receive
 */
export interface WithLoadingProps {
  /** Whether the component is loading */
  loading: boolean;
  /** Error message to display */
  error?: string;
  /** Function to clear the error */
  clearError?: () => void;
}

/**
 * Higher-Order Component for loading states
 *
 * Wraps a component with consistent loading, error, and retry UI patterns.
 *
 * @param WrappedComponent - Component to enhance with loading states
 * @param options - Configuration options
 * @returns Enhanced component with loading capabilities
 *
 * @example
 * ```tsx
 * const UserProfile = ({ loading, error, clearError }) => {
 *   if (loading) return 'Loading...'; // This will be replaced by HOC
 *   if (error) return `Error: ${error}`;
 *
 *   return <div>User data...</div>;
 * };
 *
 * const EnhancedUserProfile = withLoading(UserProfile, {
 *   loadingConfig: {
 *     type: 'spinner',
 *     message: 'Loading user profile...'
 *   },
 *   showRetryButton: true
 * });
 * ```
 */
export function withLoading<P extends object>(
  WrappedComponent: ComponentType<P & WithLoadingProps>,
  options: WithLoadingOptions = {}
): ComponentType<P> {
  const {
    loadingConfig: config = {},
    showRetryButton = false,
    onRetry,
    formatErrorMessage,
  } = options;

  const EnhancedComponent: React.FC<P> = (props) => {
    const loadingProps: WithLoadingProps = {
      loading: false,
      error: '',
      clearError: () => {},
    };

    // In a real implementation, you would get these from props or context
    // For now, this is a placeholder demonstrating the pattern

    const { loading, error } = loadingProps;

    // Show loading state
    if (loading) {
      return <LoadingIndicator config={config} />;
    }

    // Show error state
    if (error) {
      return (
        <ErrorIndicator
          error={error}
          config={config}
          showRetryButton={showRetryButton}
          onRetry={onRetry}
          formatErrorMessage={formatErrorMessage}
          clearError={loadingProps.clearError || (() => {})}
        />
      );
    }

    // Show normal component
    return <WrappedComponent {...props} {...loadingProps} />;
  };

  EnhancedComponent.displayName = `withLoading(${WrappedComponent.displayName || WrappedComponent.name})`;

  return EnhancedComponent;
}

/**
 * Loading indicator component
 */
const LoadingIndicator: React.FC<{ config: LoadingConfig }> = ({ config }) => {
  const {
    type = 'spinner',
    message = 'Loading...',
    customComponent,
    size = 'medium',
    skeletonLines = 3,
  } = config;

  if (customComponent) {
    return <>{customComponent}</>;
  }

  switch (type) {
    case 'spinner':
      return (
        <Box
          display="flex"
          flexDirection="column"
          alignItems="center"
          justifyContent="center"
          p={3}
        >
          <CircularProgress size={size === 'small' ? 20 : size === 'large' ? 60 : 40} />
          <Typography variant="body2" sx={{ mt: 2 }}>
            {message}
          </Typography>
        </Box>
      );

    case 'skeleton':
      return (
        <Box p={3}>
          {Array.from({ length: skeletonLines }, (_, index) => (
            <Skeleton key={index} height={40} sx={{ mb: 2 }} />
          ))}
        </Box>
      );

    case 'text':
      return (
        <Box display="flex" alignItems="center" justifyContent="center" p={3}>
          <Typography variant="body2" sx={{ mr: 1 }}>
            {message}
          </Typography>
          <CircularProgress size="small" />
        </Box>
      );

    default:
      return <CircularProgress />;
  }
};

/**
 * Error indicator component
 */
const ErrorIndicator: React.FC<{
  error: string;
  config: LoadingConfig;
  showRetryButton: boolean;
  onRetry?: () => void;
  formatErrorMessage?: (error: string) => string;
  clearError: () => void;
}> = ({ error, config, showRetryButton, onRetry, formatErrorMessage, clearError }) => {
  const { errorFallback } = config;
  const formattedError = formatErrorMessage ? formatErrorMessage(error) : error;

  if (errorFallback) {
    return <>{errorFallback}</>;
  }

  return (
    <Box p={3}>
      <Alert
        severity="error"
        action={
          showRetryButton && onRetry ? (
            <Button color="inherit" size="small" onClick={onRetry}>
              Retry
            </Button>
          ) : clearError ? (
            <Button color="inherit" size="small" onClick={clearError}>
              Dismiss
            </Button>
          ) : undefined
        }
      >
        {formattedError}
      </Alert>
    </Box>
  );
};

/**
 * Hook for managing loading states
 *
 * @param initialLoading - Initial loading state
 * @returns Loading state management functions
 */
export function useLoadingState(initialLoading = false) {
  const [loading, setLoading] = React.useState(initialLoading);
  const [error, setError] = React.useState<string>('');
  const [retryCount, setRetryCount] = React.useState(0);

  const startLoading = React.useCallback(() => {
    setLoading(true);
    setError('');
  }, []);

  const stopLoading = React.useCallback(() => {
    setLoading(false);
  }, []);

  const setErrorState = React.useCallback((errorMessage: string) => {
    setError(errorMessage);
    setLoading(false);
  }, []);

  const clearError = React.useCallback(() => {
    setError('');
  }, []);

  const retry = React.useCallback(() => {
    setRetryCount((prev) => prev + 1);
    startLoading();
  }, [startLoading]);

  return {
    loading,
    error,
    retryCount,
    startLoading,
    stopLoading,
    setError: setErrorState,
    clearError,
    retry,
  };
}

/**
 * Synchronous loading HOC for direct use
 */
export function withSyncLoading<P extends object>(
  WrappedComponent: ComponentType<P>,
  loading: boolean,
  error?: string,
  config: LoadingConfig = {}
): ComponentType<P> {
  const LoadingComponent: React.FC<P> = (props) => {
    if (loading) {
      return <LoadingIndicator config={config} />;
    }

    if (error && config.errorFallback) {
      return <>{config.errorFallback}</>;
    }

    if (error) {
      return (
        <ErrorIndicator
          error={error}
          config={config}
          showRetryButton={false}
          clearError={() => {}}
        />
      );
    }

    return <WrappedComponent {...props} />;
  };

  LoadingComponent.displayName = `withSyncLoading(${WrappedComponent.displayName || WrappedComponent.name})`;

  return LoadingComponent;
}

/**
 * Wrapper component for consistent loading patterns
 */
export const LoadingWrapper: React.FC<{
  loading: boolean;
  error?: string;
  children: ReactNode;
  config?: LoadingConfig;
  retry?: () => void;
}> = ({ loading, error, children, config, retry }) => {
  if (loading) {
    return <LoadingIndicator config={config || {}} />;
  }

  if (error) {
    return (
      <ErrorIndicator
        error={error}
        config={config || {}}
        showRetryButton={!!retry}
        onRetry={retry}
        clearError={() => {}}
      />
    );
  }

  return <>{children}</>;
};
