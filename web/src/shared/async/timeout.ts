// Standardized timeout utilities with cancellation support

export interface TimeoutOptions {
  timeout?: number;
  signal?: AbortSignal;
}

export interface TimeoutResult<T> {
  result: T | undefined;
  timedOut: boolean;
}

/**
 * Executes an async operation with optional timeout and cancellation support
 * Consolidates timeout patterns found throughout the codebase
 */
export async function withTimeout<T>(
  operation: () => Promise<T>,
  timeoutMs: number,
  signal?: AbortSignal,
): Promise<TimeoutResult<T>> {
  const timeoutPromise = new Promise<never>((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error('Operation timed out'));
    }, timeoutMs);

    signal?.addEventListener('abort', () => {
      clearTimeout(timeoutId);
      reject(new Error('Operation cancelled'));
    }, { once: true });
  });

  try {
    const result = await Promise.race([operation(), timeoutPromise]);
    return { result, timedOut: false };
  } catch (error) {
    if (error instanceof Error && error.message === 'Operation timed out') {
      return { result: undefined, timedOut: true };
    }
    throw error;
  }
}

/**
 * Creates a promise that resolves after specified delay
 * Standard utility for async delays
 */
export function delay(ms: number): Promise<void> {
  return new Promise(resolve => {
    setTimeout(resolve, ms);
  });
}

/**
 * Creates a periodic timer that can be cancelled
 */
export interface TimerOptions {
  interval: number;
  maxDuration?: number;
  onTick?: () => void | Promise<void>;
  signal?: AbortSignal;
}

export async function periodicTimer(options: TimerOptions): Promise<void> {
  const { interval, maxDuration, onTick, signal } = options;

  const startTime = Date.now();
  const maxTime = maxDuration ? startTime + maxDuration : Infinity;

  const controller = new AbortController();
  const combinedSignal = signal ? combineSignals(signal, controller.signal) : controller.signal;

  return new Promise((resolve, reject) => {
    const timer = setInterval(async () => {
      if (Date.now() >= maxTime || combinedSignal.aborted) {
        clearInterval(timer);
        resolve();
        return;
      }

      if (onTick) {
        try {
          await onTick();
        } catch (error) {
          clearInterval(timer);
          reject(error instanceof Error ? error : new Error(String(error)));
        }
      }
    }, interval);

    combinedSignal.addEventListener('abort', () => {
      clearInterval(timer);
      resolve();
    });
  });
}

/**
 * Combines multiple AbortSignals into one
 * Useful for complex cancellation scenarios
 */
export function combineSignals(...signals: AbortSignal[]): AbortSignal {
  const controller = new AbortController();
  const combinedSignal = controller.signal;

  signals.forEach(signal => {
    if (signal.aborted) {
      controller.abort();
      return;
    }

    signal.addEventListener('abort', () => {
      controller.abort();
    }, { once: true });
  });

  return combinedSignal;
}

/**
 * Creates a cancellable timeout promise
 */
export function cancellableDelay(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      resolve();
    }, ms);

    signal.addEventListener('abort', () => {
      clearTimeout(timeoutId);
      reject(new Error('Delayed operation cancelled'));
    }, { once: true });
  });
}