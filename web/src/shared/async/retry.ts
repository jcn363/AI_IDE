// Standardized retry policies with configurable backoff
// Consolidates duplicate retry patterns found in services and utilities

export interface RetryOptions {
  /** Maximum number of attempts (including first attempt) */
  maxAttempts: number;
  /** Initial delay in milliseconds */
  initialDelay?: number;
  /** Maximum delay in milliseconds */
  maxDelay?: number;
  /** Backoff multiplier (default: 2 for exponential) */
  backoffMultiplier?: number;
  /** Cancellation signal */
  signal?: AbortSignal;
  /** Function to determine if error should trigger retry */
  shouldRetry?: (error: Error, attempt: number) => boolean;
  /** Callback for each retry attempt */
  onRetry?: (error: Error, attempt: number, nextDelay: number) => void;
}

/**
 * Default retry decision function
 */
function shouldRetryDefault(error: Error): boolean {
  // Don't retry for certain error types
  const nonRetryableErrors = [
    'AbortError',
    'TypeError',
    /fatal/i,
    /unauthorized/i,
    /forbidden/i,
    /not found/i,
  ];

  return !nonRetryableErrors.some((pattern) => {
    if (typeof pattern === 'string') {
      return error.name === pattern || error.message.includes(pattern);
    }
    return pattern.test(error.message);
  });
}

/**
 * Calculates delay for the given attempt
 */
function calculateDelay(
  attempt: number,
  initialDelay: number,
  maxDelay: number,
  backoffMultiplier: number
): number {
  if (attempt <= 1) return 0;

  const exponentialDelay = initialDelay * Math.pow(backoffMultiplier, attempt - 2);
  return Math.min(exponentialDelay, maxDelay);
}

/**
 * Retries an async operation with configurable backoff
 */
export async function retryWithBackoff<T>(
  operation: () => Promise<T>,
  options: RetryOptions
): Promise<T> {
  const {
    maxAttempts,
    initialDelay = 1000,
    maxDelay = 30000,
    backoffMultiplier = 2,
    signal,
    shouldRetry = shouldRetryDefault,
    onRetry,
  } = options;

  let lastError: Error | undefined;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      if (signal?.aborted) {
        throw new Error('Operation cancelled');
      }

      return await operation();
    } catch (error) {
      lastError = error as Error;

      // Check if this was the last attempt
      if (attempt === maxAttempts) {
        if (lastError) {
          throw lastError;
        } else {
          throw new Error('Unknown retry error');
        }
      }

      // Check if we should retry for this error
      if (shouldRetry(lastError, attempt)) {
        const delay = calculateDelay(attempt, initialDelay, maxDelay, backoffMultiplier);

        console.warn(`Attempt ${attempt} failed: ${lastError.message}. Retrying in ${delay}ms`);

        if (onRetry) {
          onRetry(lastError, attempt, delay);
        }

        await cancellableDelay(delay, signal);
      } else {
        throw lastError;
      }
    }
  }

  // This should never be reached, but TypeScript requires it
  throw lastError || new Error('Retry logic error');
}

// Simplified retry functions for common use cases

export interface SimpleRetryOptions {
  maxAttempts?: number;
  signal?: AbortSignal;
  shouldRetry?: (error: Error) => boolean;
}

/**
 * Simple exponential backoff retry for network operations
 */
export async function retryNetwork<T>(
  operation: () => Promise<T>,
  options: SimpleRetryOptions = {}
): Promise<T> {
  const { maxAttempts = 3, signal, shouldRetry } = options;

  return retryWithBackoff(operation, {
    maxAttempts,
    initialDelay: 500,
    maxDelay: 5000,
    signal,
    shouldRetry:
      shouldRetry ||
      ((error) => {
        // Retry network-related errors
        return (
          error.name === 'TypeError' ||
          error.name === 'NetworkError' ||
          error.message.includes('fetch') ||
          (error.message.includes('5') && error.message.includes('server'))
        );
      }),
  });
}

/**
 * Linear backoff retry for rate-limited operations
 */
export async function retryRateLimited<T>(
  operation: () => Promise<T>,
  options: SimpleRetryOptions = {}
): Promise<T> {
  const { maxAttempts = 5, signal, shouldRetry } = options;

  return retryWithBackoff(operation, {
    maxAttempts,
    initialDelay: 1000,
    maxDelay: 10000,
    backoffMultiplier: 1, // Linear
    signal,
    shouldRetry:
      shouldRetry ||
      ((error) => {
        return (
          error.message.includes('rate limit') ||
          error.message.includes('429') ||
          error.message.includes('Too Many Requests')
        );
      }),
  });
}

/**
 * Immediate retry with no backoff for transient errors
 */
export async function retryImmediate<T>(
  operation: () => Promise<T>,
  options: SimpleRetryOptions = {}
): Promise<T> {
  const { maxAttempts = 3, signal, shouldRetry } = options;

  return retryWithBackoff(operation, {
    maxAttempts,
    initialDelay: 100,
    maxDelay: 100,
    backoffMultiplier: 1, // Constant delay
    signal,
    shouldRetry,
  });
}

/**
 * Cancellable delay utility
 */
async function cancellableDelay(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(resolve, ms);

    signal?.addEventListener(
      'abort',
      () => {
        clearTimeout(timeoutId);
        reject(new Error('Delay cancelled'));
      },
      { once: true }
    );
  });
}
