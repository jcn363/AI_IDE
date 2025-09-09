export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'critical';

export interface LogContext {
  [key: string]: any;
  component?: string;
  timestamp?: string;
  sessionId?: string;
  userId?: string;
  [key: `x-${string}`]: any;
}

export interface LogEntry {
  level: LogLevel;
  message: string;
  context?: LogContext;
  timestamp: string;
  error?: Error;
}

export interface LoggerOptions {
  minLevel?: LogLevel;
  serviceName?: string;
  environment?: 'development' | 'staging' | 'production' | 'test';
  enableConsole?: boolean;
  enableTelemetry?: boolean;
  context?: LogContext;
}

export interface TelemetryAdapter {
  captureException(error: Error, context?: LogContext): void;
  captureMessage(message: string, level: LogLevel, context?: LogContext): void;
  captureEvent(eventName: string, context?: LogContext): void;
}
