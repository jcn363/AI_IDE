import { v4 as uuidv4 } from 'uuid';
import { LogLevel, LogContext, LogEntry, LoggerOptions, TelemetryAdapter } from './types';
import { defaultTelemetryAdapter } from './telemetry';

const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
  critical: 4,
};

/**
 * A flexible and extensible logger with telemetry support
 */
export class Logger {
  private readonly options: Required<LoggerOptions>;
  private telemetry: TelemetryAdapter;
  private sessionId: string;
  private readonly context: LogContext;

  constructor(options: LoggerOptions = {}) {
    this.options = {
      minLevel: options.minLevel || 'info',
      serviceName: options.serviceName || 'rust-ai-ide',
      environment: options.environment || 'development',
      enableConsole: options.enableConsole !== false, // enabled by default
      enableTelemetry: options.enableTelemetry !== false, // enabled by default
      context: options.context || {},
    };

    this.telemetry = defaultTelemetryAdapter;
    this.sessionId = uuidv4();
    this.context = {
      service: this.options.serviceName,
      environment: this.options.environment,
      sessionId: this.sessionId,
      ...this.options.context,
    };
  }

  /**
   * Create a child logger with additional context
   */
  public child(additionalContext: LogContext): Logger {
    return new Logger({
      ...this.options,
      context: {
        ...this.context,
        ...additionalContext,
      },
    });
  }

  /**
   * Log a message at debug level
   */
  public debug(message: string, context: LogContext = {}): void {
    this.log('debug', message, context);
  }

  /**
   * Log a message at info level
   */
  public info(message: string, context: LogContext = {}): void {
    this.log('info', message, context);
  }

  /**
   * Log a message at warn level
   */
  public warn(message: string, context: LogContext = {}): void {
    this.log('warn', message, context);
  }

  /**
   * Log a message at error level
   */
  public error(message: string, error?: Error, context: LogContext = {}): void {
    this.log('error', message, { ...context, error });
  }

  /**
   * Log a message at critical level
   */
  public critical(message: string, error?: Error, context: LogContext = {}): void {
    this.log('critical', message, { ...context, error });
  }

  /**
   * Track a custom event
   */
  public trackEvent(eventName: string, context: LogContext = {}): void {
    if (this.options.enableTelemetry) {
      this.telemetry.captureEvent(eventName, {
        ...this.context,
        ...context,
      });
    }
  }

  /**
   * Set the telemetry adapter
   */
  public setTelemetryAdapter(adapter: TelemetryAdapter): void {
    this.telemetry = adapter;
  }

  /**
   * Internal log method that handles all logging
   */
  private log(level: LogLevel, message: string, context: LogContext = {}): void {
    // Check if the log level is enabled
    if (LOG_LEVELS[level] < LOG_LEVELS[this.options.minLevel]) {
      return;
    }

    const timestamp = new Date().toISOString();
    const logEntry: LogEntry = {
      level,
      message,
      context: {
        ...this.context,
        ...context,
      },
      timestamp,
    };

    // Handle errors in the context
    if (context.error) {
      logEntry.error = context.error as Error;
    }

    // Output to console if enabled
    if (this.options.enableConsole) {
      this.consoleLog(level, logEntry);
    }

    // Send to telemetry if enabled and level is error or above
    if (this.options.enableTelemetry && level !== 'debug') {
      try {
        if (logEntry.error) {
          this.telemetry.captureException(logEntry.error, logEntry.context);
        } else {
          this.telemetry.captureMessage(message, level, logEntry.context);
        }
      } catch (telemetryError) {
        // Don't throw if telemetry fails
        console.error('Failed to send log to telemetry:', telemetryError);
      }
    }
  }

  /**
   * Format and output log to console
   */
  private consoleLog(level: LogLevel, entry: LogEntry): void {
    const { message, context, timestamp } = entry;
    const styles = this.getConsoleStyles(level);
    const parts = [
      `%c[${timestamp}] %c${level.toUpperCase()}`,
      'color: #888',
      styles.style,
      message,
    ];

    // Add context if present
    if (context && Object.keys(context).length > 0) {
      parts.push('\n%cContext:', 'font-weight: bold; color: #666;');
      console.log(...parts);
      console.log(context);
      return;
    }

    // Use appropriate console method
    const consoleMethod = this.getConsoleMethod(level);
    // @ts-ignore - TypeScript doesn't understand the spread with console methods
    console[consoleMethod](...parts);
  }

  /**
   * Get the appropriate console method for the log level
   */
  private getConsoleMethod(level: LogLevel): 'debug' | 'info' | 'warn' | 'error' | 'log' {
    switch (level) {
      case 'debug':
        return 'debug';
      case 'info':
        return 'info';
      case 'warn':
        return 'warn';
      case 'error':
      case 'critical':
        return 'error';
      default:
        return 'log';
    }
  }

  /**
   * Get console styles for different log levels
   */
  private getConsoleStyles(level: LogLevel): { style: string } {
    const styles: Record<LogLevel, string> = {
      debug: 'color: #9E9E9E; font-weight: bold;',
      info: 'color: #2196F3; font-weight: bold;',
      warn: 'color: #FF9800; font-weight: bold;',
      error: 'color: #F44336; font-weight: bold;',
      critical: 'color: #B71C1C; font-weight: bold; background: #FFEBEE;',
    };

    return { style: styles[level] };
  }
}
