import { performance } from 'node:perf_hooks';

interface Task<T = any> {
  id: string;
  name?: string;
  execute: () => Promise<T>;
  priority?: number;
}

interface TaskResult<T> {
  task: Task<T>;
  result?: T;
  error?: Error;
  duration: number;
  startTime: number;
  endTime: number;
}

interface BatchOptions {
  maxConcurrency?: number;
  timeout?: number;
  continueOnError?: boolean;
}

interface RateLimitOptions {
  maxConcurrent: number;
  queueLimit?: number;
}

/**
 * Execute tasks in batches with controlled concurrency
 */
export class BatchExecutor<T = any> {
  private tasks: Task<T>[] = [];
  private options: BatchOptions;

  constructor(options: BatchOptions = {}) {
    this.options = {
      maxConcurrency: 5,
      timeout: 30000, // 30s
      continueOnError: true,
      ...options
    };
  }

  /**
   * Add a task to the batch
   */
  addTask(task: Task<T>): void {
    this.tasks.push(task);
  }

  /**
   * Add multiple tasks to the batch
   */
  addTasks(tasks: Task<T>[]): void {
    this.tasks.push(...tasks);
  }

  /**
   * Execute all tasks in the batch
   */
  async execute(): Promise<TaskResult<T>[]> {
    const results: TaskResult<T>[] = [];
    const concurrency = Math.min(this.options.maxConcurrency!, this.tasks.length);

    // Execute tasks in chunks
    for (let i = 0; i < this.tasks.length; i += concurrency) {
      const chunk = this.tasks.slice(i, i + concurrency);

      const chunkPromises = chunk.map(async (task) => {
        const startTime = performance.now();

        try {
          const result = await this.withTimeout(task.execute(), this.options.timeout!);
          const endTime = performance.now();

          return {
            task,
            result,
            duration: endTime - startTime,
            startTime,
            endTime
          } as TaskResult<T>;
        } catch (error) {
          const endTime = performance.now();
          const taskError = error instanceof Error ? error : new Error(String(error));

          return {
            task,
            error: taskError,
            duration: endTime - startTime,
            startTime,
            endTime
          } as TaskResult<T>;
        }
      });

      try {
        const chunkResults = await Promise.all(chunkPromises);
        results.push(...chunkResults);
      } catch (error) {
        if (!this.options.continueOnError!) {
          throw error;
        }
        console.error('Batch execution failed:', error);
      }
    }

    return results;
  }

  private withTimeout<T>(promise: Promise<T>, timeout: number): Promise<T> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new Error(`Operation timed out after ${timeout}ms`));
      }, timeout);

      promise
        .then(resolve)
        .catch(reject)
        .finally(() => clearTimeout(timer));
    });
  }

  /**
   * Get pending task count
   */
  getTaskCount(): number {
    return this.tasks.length;
  }

  /**
   * Clear all tasks
   */
  clear(): void {
    this.tasks = [];
  }
}

/**
 * Rate limiter for controlling concurrent executions
 */
export class RateLimiter {
  private queue: Array<{ resolve: (value: any) => void; reject: (reason?: any) => void; fn: () => Promise<any>; }> = [];
  private activeCount = 0;
  private options: RateLimitOptions;

  constructor(options: RateLimitOptions) {
    this.options = {
      queueLimit: Infinity,
      ...options
    };
  }

  /**
   * Execute a function with rate limiting
   */
  execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.activeCount >= this.options.maxConcurrent) {
      if (this.options.queueLimit != null && this.queue.length >= this.options.queueLimit) {
        throw new Error('Queue limit exceeded');
      }

      return new Promise<T>((resolve, reject) => {
        this.queue.push({ resolve, reject, fn });
      });
    }

    return this.doExecute(fn);
  }

  private async doExecute<T>(fn: () => Promise<T>): Promise<T> {
    this.activeCount++;
    try {
      const result = await fn();
      return result;
    } finally {
      this.activeCount--;
      this.processQueue();
    }
  }

  private processQueue(): void {
    if (this.queue.length > 0 && this.activeCount < this.options.maxConcurrent) {
      const { resolve, reject, fn } = this.queue.shift()!;
      (this.doExecute(fn) as Promise<any>).then(resolve, reject);
    }
  }

  /**
   * Get current active count
   */
  getActiveCount(): number {
    return this.activeCount;
  }

  /**
   * Get queue length
   */
  getQueueLength(): number {
    return this.queue.length;
  }
}

/**
 * Parallel execution utilities
 */
export class ParallelExecutor {
  private maxConcurrency: number;
  private retryAttempts: number;

  constructor(maxConcurrency: number = 10, retryAttempts: number = 0) {
    this.maxConcurrency = maxConcurrency;
    this.retryAttempts = retryAttempts;
  }

  /**
   * Execute functions in parallel with controlled concurrency
   */
  async execute<T>(functions: (() => Promise<T>)[], options: BatchOptions = {}): Promise<T[]> {
    const tasks: Task<T>[] = functions.map((fn, index) => ({
      id: `task-${index}`,
      execute: fn
    }));

    const executor = new BatchExecutor<T>({
      maxConcurrency: this.maxConcurrency,
      continueOnError: true,
      ...options
    });

    executor.addTasks(tasks);
    const results = await executor.execute();

    return results.map(result => {
      if (result.error) {
        throw result.error;
      }
      return result.result!;
    });
  }

  /**
   * Execute with retry logic
   */
  async executeWithRetry<T>(
    functions: (() => Promise<T>)[],
    retryOptions: { maxRetries?: number; retryDelay?: number } = {}
  ): Promise<T[]> {
    const { maxRetries = this.retryAttempts, retryDelay = 1000 } = retryOptions;

    const executeWithRetry = (fn: () => Promise<T>): () => Promise<T> => {
    return async () => {
      for (let attempt = 0; attempt <= maxRetries; attempt++) {
        try {
          return await fn();
        } catch (error) {
          if (attempt === maxRetries) {
            throw error;
          }
          await this.delay(retryDelay);
        }
      }
      throw new Error('Should not reach here');
    };
  };

    const retriedFunctions = functions.map(executeWithRetry);
    return this.execute(retriedFunctions);
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Utility function for batched processing
 */
export function batchProcess<T, R>(
  items: T[],
  processor: (item: T) => Promise<R>,
  options: BatchOptions = {}
): () => Promise<TaskResult<R>[]> {
  return async () => {
    const executor = new BatchExecutor<R>(options);

    items.forEach((item, index) => {
      executor.addTask({
        id: `batch-item-${index}`,
        execute: () => processor(item)
      });
    });

    return executor.execute();
  };
}

/**
 * Utility function for rate-limited execution
 */
export function rateLimit<T>(
  fn: () => Promise<T>,
  rateLimiter: RateLimiter
): () => Promise<T> {
  return () => rateLimiter.execute(fn);
}

export type { Task, TaskResult, BatchOptions, RateLimitOptions };