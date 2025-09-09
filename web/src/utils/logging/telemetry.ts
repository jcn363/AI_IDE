import { LogLevel, TelemetryAdapter, LogContext } from './types';

/**
 * Default telemetry adapter that can be extended or replaced
 * with a service-specific implementation (e.g., Sentry, Application Insights)
 */
class DefaultTelemetryAdapter implements TelemetryAdapter {
  captureException(error: Error, context?: LogContext): void {
    // In a real implementation, this would send to your telemetry service
    console.error('Telemetry - Exception:', error, context);
  }

  captureMessage(message: string, level: LogLevel, context?: LogContext): void {
    // In a real implementation, this would send to your telemetry service
    const logMessage = `Telemetry - ${level.toUpperCase()}: ${message}`;
    if (level === 'error' || level === 'critical') {
      console.error(logMessage, context);
    } else if (level === 'warn') {
      console.warn(logMessage, context);
    } else {
      console.log(logMessage, context);
    }
  }

  captureEvent(eventName: string, context?: LogContext): void {
    // In a real implementation, this would send to your telemetry service
    console.log(`Telemetry - Event: ${eventName}`, context);
  }
}

// Export a singleton instance
export const defaultTelemetryAdapter = new DefaultTelemetryAdapter();

/**
 * Creates a telemetry adapter that can be used with the logger
 * @param customAdapter Optional custom telemetry adapter
 * @returns TelemetryAdapter instance
 */
export function createTelemetryAdapter(customAdapter?: TelemetryAdapter): TelemetryAdapter {
  return customAdapter || defaultTelemetryAdapter;
}
