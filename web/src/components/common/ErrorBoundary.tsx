import React, { Component, ErrorInfo, ReactNode } from 'react';
import logger from '@/utils/logging';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  context?: Record<string, any>;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    // Log the error with proper typing
    const context: Record<string, any> = {
      ...this.props.context,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
    };

    logger.error('Error in React component', error, context);

    this.setState({ error, errorInfo });
  }

  render() {
    if (this.state.hasError) {
      // Render fallback UI
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return (
        <div style={{ padding: '1rem' }}>
          <h2>Something went wrong</h2>
          <details style={{ whiteSpace: 'pre-wrap' }}>
            {this.state.error?.toString()}
            <br />
            {this.state.errorInfo?.componentStack}
          </details>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;

// Higher-Order Component for error boundaries
export function withErrorBoundary<T>(
  Component: React.ComponentType<T>,
  options: Omit<Props, 'children'> = {}
): React.FC<T> {
  return (props: T) => (
    <ErrorBoundary {...options}>
      <Component {...(props as any)} />
    </ErrorBoundary>
  );
}
