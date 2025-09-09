export * from './types';
export * from './logger';
export * from './telemetry';

// Create a default logger instance
import { Logger } from './logger';

export const logger = new Logger({
  serviceName: 'rust-ai-ide',
  environment: process.env.NODE_ENV === 'production' ? 'production' : 'development',
  minLevel: process.env.NODE_ENV === 'production' ? 'info' : 'debug',
});

// Export common logger methods for convenience
export const debug = logger.debug.bind(logger);
export const info = logger.info.bind(logger);
export const warn = logger.warn.bind(logger);
export const error = logger.error.bind(logger);
export const critical = logger.critical.bind(logger);
export const trackEvent = logger.trackEvent.bind(logger);

/**
 * Creates a scoped logger with additional context
 * @example
 * const log = createScopedLogger('module:name', { userId: '123' });
 * log.info('User action performed');
 */
export function createScopedLogger(component: string, context: Record<string, any> = {}) {
  return logger.child({ component, ...context });
}

// Export the Logger class as default
export default logger;
