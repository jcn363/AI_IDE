import { performance } from 'node:perf_hooks';

interface AsyncHookContext {
  name: string;
  startTime: number;
  metadata: Record<string, any>;
}

interface HookFunction<Result = void, Context = AsyncHookContext> {
  (context: Context): Promise<Result> | Result;
}

interface HookOptions {
  timeout?: number;
  continueOnError?: boolean;
  priority?: number;
}

/**
 * Base hook manager class for managing async lifecycle hooks
 */
export class HookManager {
  private beforeHooks: Array<{ fn: HookFunction<any>; options: HookOptions }> = [];
  private afterHooks: Array<{ fn: HookFunction<any>; options: HookOptions }> = [];
  private errorHooks: Array<{ fn: HookFunction<void>; options: HookOptions }> = [];

  /**
   * Register a hook to run before the main operation
   */
  addBeforeHook<T>(hook: HookFunction<T>, options: HookOptions = {}): void {
    this.beforeHooks.push({ fn: hook, options: { timeout: 5000, continueOnError: true, priority: 0, ...options } });
    this.beforeHooks.sort((a, b) => (b.options.priority || 0) - (a.options.priority || 0));
  }

  /**
   * Register a hook to run after the main operation
   */
  addAfterHook<T>(hook: HookFunction<T>, options: HookOptions = {}): void {
    this.afterHooks.push({ fn: hook, options: { timeout: 5000, continueOnError: true, priority: 0, ...options } });
    this.afterHooks.sort((a, b) => (b.options.priority || 0) - (a.options.priority || 0));
  }

  /**
   * Register a hook to run when an error occurs
   */
  addErrorHook(hook: HookFunction<void>, options: HookOptions = {}): void {
    this.errorHooks.push({ fn: hook, options: { timeout: 5000, continueOnError: true, priority: 0, ...options } });
    this.errorHooks.sort((a, b) => (b.options.priority || 0) - (a.options.priority || 0));
  }

  /**
   * Execute all before hooks
   */
  async executeBeforeHooks<T = void>(
    operationName: string,
    metadata: Record<string, any> = {}
  ): Promise<T[]> {
    const context: AsyncHookContext = {
      name: operationName,
      startTime: performance.now(),
      metadata
    };

    return this.executeHooks<T>(this.beforeHooks as Array<{ fn: HookFunction<T>; options: HookOptions }>, context);
  }

  /**
   * Execute all after hooks
   */
  async executeAfterHooks<T = void>(
    operationName: string,
    result: T,
    metadata: Record<string, any> = {}
  ): Promise<T[]> {
    const context: AsyncHookContext & { result: T } = {
      name: operationName,
      startTime: performance.now(),
      metadata: { ...metadata, result },
      result
    };

    return this.executeHooks<T>(this.afterHooks as Array<{ fn: HookFunction<T>; options: HookOptions }>, context);
  }

  /**
   * Execute all error hooks
   */
  async executeErrorHooks(
    operationName: string,
    error: Error,
    metadata: Record<string, any> = {}
  ): Promise<void[]> {
    const context: AsyncHookContext & { error: Error } = {
      name: operationName,
      startTime: performance.now(),
      metadata: { ...metadata, error },
      error
    };

    return this.executeHooks<void>(this.errorHooks, context);
  }

  /**
   * Run a function with before/after hooks
   */
  async withHooks<R>(
    operationName: string,
    operation: () => Promise<R>,
    metadata: Record<string, any> = {}
  ): Promise<R> {
    let result: R;

    try {
      // Execute before hooks
      await this.executeBeforeHooks(operationName, metadata);

      // Execute main operation
      result = await operation();

      // Execute after hooks
      await this.executeAfterHooks(operationName, result, metadata);

      return result;
    } catch (error) {
      const hookError = error instanceof Error ? error : new Error(String(error));

      // Execute error hooks
      await this.executeErrorHooks(operationName, hookError, metadata);
      throw hookError;
    }
  }

  private async executeHooks<Result>(
    hooks: Array<{ fn: HookFunction<Result>; options: HookOptions }>,
    context: AsyncHookContext
  ): Promise<Result[]> {
    if (hooks.length === 0) return [];

    const results: Result[] = [];

    for (const hook of hooks) {
      try {
        const result = await this.withTimeout(
          hook.fn(context),
          hook.options.timeout!
        );
        results.push(result);
      } catch (error) {
        if (!hook.options.continueOnError!) {
          throw error;
        }
        console.error(`Hook execution failed:`, error);
      }
    }

    return results;
  }

  private withTimeout<T>(promise: Promise<T> | T, timeout: number): Promise<T> {
    if (!(promise instanceof Promise)) {
      return Promise.resolve(promise);
    }

    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new Error(`Hook timed out after ${timeout}ms`));
      }, timeout);

      promise
        .then(resolve)
        .catch(reject)
        .finally(() => clearTimeout(timer));
    });
  }

  /**
   * Get the number of registered hooks
   */
  getHookCount(): { before: number; after: number; error: number } {
    return {
      before: this.beforeHooks.length,
      after: this.afterHooks.length,
      error: this.errorHooks.length
    };
  }

  /**
   * Clear all hooks
   */
  clear(): void {
    this.beforeHooks = [];
    this.afterHooks = [];
    this.errorHooks = [];
  }
}

/**
 * Global hook registry for managing application-wide hooks
 */
class GlobalHookRegistry {
  private registries = new Map<string, HookManager>();

  /**
   * Get or create a hook manager for a specific namespace
   */
  getManager(namespace: string): HookManager {
    if (!this.registries.has(namespace)) {
      this.registries.set(namespace, new HookManager());
    }
    return this.registries.get(namespace)!;
  }

  /**
    * Register a global hook
    */
  addGlobalHook<T = void>(
    namespace: string,
    type: 'before' | 'after' | 'error',
    hook: HookFunction<T>,
    options?: HookOptions
  ): void {
    const manager = this.getManager(namespace);

    switch (type) {
      case 'before':
        manager.addBeforeHook(hook as HookFunction<any>, options);
        break;
      case 'after':
        manager.addAfterHook(hook as HookFunction<any>, options);
        break;
      case 'error':
        manager.addErrorHook(hook as HookFunction<void>, options);
        break;
    }
  }

  /**
   * Execute hooks in multiple namespaces
   */
  async executeMultiNamespace<R>(
    namespaces: string[],
    type: 'before' | 'after' | 'error',
    operationName: string,
    params?: any
  ): Promise<void> {
    const promises = namespaces.map(namespace => {
      const manager = this.registries.get(namespace);
      if (!manager) return Promise.resolve();

      switch (type) {
        case 'before':
          return manager.executeBeforeHooks(operationName, params);
        case 'after':
          return manager.executeAfterHooks(operationName, params);
        case 'error':
          return manager.executeErrorHooks(operationName, params);
        default:
          return Promise.resolve();
      }
    });

    await Promise.all(promises);
  }
}

export const globalHookRegistry = new GlobalHookRegistry();

/**
 * Decorator for adding hooks to methods
 */
export function withHooks(
  operationName: string,
  metadata?: Record<string, any>
) {
  return function (
    target: any,
    propertyName: string,
    descriptor: PropertyDescriptor
  ) {
    const method = descriptor.value;
    const hookManager = new HookManager();

    descriptor.value = async function (...args: any[]) {
      return hookManager.withHooks(
        operationName,
        () => method.apply(this, args),
        { ...metadata, target: target.constructor.name, method: propertyName, args }
      );
    };
  };
}

/**
 * Middleware-style hooks
 */
export class AsyncMiddleware {
  private middlewares: Array<(context: AsyncHookContext) => Promise<void> | void> = [];

  add(middleware: (context: AsyncHookContext) => Promise<void> | void): void {
    this.middlewares.push(middleware);
  }

  async execute(context: AsyncHookContext): Promise<void> {
    for (const middleware of this.middlewares) {
      await middleware(context);
    }
  }
}

export type { AsyncHookContext, HookFunction, HookOptions };