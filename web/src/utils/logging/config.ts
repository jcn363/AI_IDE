import { Logger } from './logger';
import type { LoggerOptions } from './types';

// Environment-based configuration
const isDevelopment = process.env.NODE_ENV === 'development';
const isTest = process.env.NODE_ENV === 'test';

// Default logger configuration
const defaultConfig: LoggerOptions = {
  minLevel: isDevelopment ? 'debug' : 'info', // More verbose in development
  serviceName: 'rust-ai-ide',
  environment: (isDevelopment ? 'development' : isTest ? 'test' : 'production') as
    | 'development'
    | 'production'
    | 'test',
  enableConsole: !isTest, // Disable console logs in tests
  enableTelemetry: !isTest, // Disable telemetry in tests
  context: {
    appVersion: process.env.REACT_APP_VERSION || '0.1.0',
  },
};

// Create a singleton logger instance
export const logger = new Logger(defaultConfig);

// Export a function to create a scoped logger
export const createScopedLogger = (component: string, context: Record<string, any> = {}) => {
  return logger.child({
    component,
    ...context,
  });
};

// Export common logger methods for convenience
export const { debug, info, warn, error, critical, trackEvent } = logger;

export default logger;
