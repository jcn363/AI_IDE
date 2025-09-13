// Async operation tracing utilities with performance metrics and logging

interface TraceContext {
  id: string;
  name: string;
  startTime: number;
  endTime?: number;
  parentId?: string;
  metadata: Record<string, any>;
  children: TraceContext[];
}

interface PerformanceMetrics {
  duration: number;
  memoryUsage?: number;
  cpuUsage?: number;
}

class AsyncTracer {
  private activeTraces = new Map<string, TraceContext>();
  private logger: (level: string, message: string, context: any) => void;

  constructor(logger?: (level: string, message: string, context: any) => void) {
    this.logger =
      logger ||
      ((level, message, context) => {
        console.log(`[${level.toUpperCase()}] [${new Date().toISOString()}] ${message}`, context);
      });
  }

  /**
   * Start tracing an async operation
   */
  startTrace(name: string, metadata: Record<string, any> = {}): string {
    const traceId = this.generateTraceId();
    const context: TraceContext = {
      id: traceId,
      name,
      startTime: performance.now(),
      metadata: { ...metadata },
      children: [],
    };

    // Find parent context if exists
    const parentTrace = this.findActiveParent(name);
    if (parentTrace) {
      context.parentId = parentTrace.id;
      parentTrace.children.push(context);
    }

    this.activeTraces.set(traceId, context);
    this.logger('debug', `Started trace: ${name}`, { traceId });

    return traceId;
  }

  /**
   * End tracing for a specific trace ID
   */
  endTrace(traceId: string): PerformanceMetrics | null {
    const context = this.activeTraces.get(traceId);
    if (!context) {
      this.logger('warn', `Trace not found: ${traceId}`, {});
      return null;
    }

    context.endTime = performance.now();
    const metrics = this.calculateMetrics(context);

    this.logger('debug', `Ended trace: ${context.name}`, { traceId, metrics });

    // Remove from active traces
    this.activeTraces.delete(traceId);

    return metrics;
  }

  /**
   * Trace an async function automatically
   */
  async traceAsync<T>(
    name: string,
    fn: () => Promise<T>,
    metadata: Record<string, any> = {}
  ): Promise<T> {
    const traceId = this.startTrace(name, metadata);
    try {
      const result = await fn();
      return result;
    } catch (error) {
      this.logger('error', `Trace failed: ${name}`, { traceId, error });
      throw error;
    } finally {
      this.endTrace(traceId);
    }
  }

  /**
   * Add metadata to an active trace
   */
  addMetadata(traceId: string, metadata: Record<string, any>): void {
    const context = this.activeTraces.get(traceId);
    if (context) {
      context.metadata = { ...context.metadata, ...metadata };
    }
  }

  /**
   * Get active traces
   */
  getActiveTraces(): TraceContext[] {
    return Array.from(this.activeTraces.values());
  }

  /**
   * Get trace by ID (even if completed - would need persistence for that)
   */
  getTrace(traceId: string): TraceContext | null {
    return this.activeTraces.get(traceId) || null;
  }

  private generateTraceId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private findActiveParent(invokerName?: string): TraceContext | null {
    for (const trace of this.activeTraces.values()) {
      if (!trace.endTime && trace.name !== invokerName) {
        return trace;
      }
    }
    return null;
  }

  private calculateMetrics(context: TraceContext): PerformanceMetrics {
    const duration = context.endTime! - context.startTime;

    // Basic metrics - could be extended
    const metrics: PerformanceMetrics = {
      duration,
    };

    // Memory usage if available
    if ('memory' in performance) {
      metrics.memoryUsage = (performance as any).memory.usedJSHeapSize;
    }

    return metrics;
  }
}

export const asyncTracer = new AsyncTracer();

export function startTrace(name: string, metadata?: Record<string, any>): string {
  return asyncTracer.startTrace(name, metadata);
}

export function endTrace(traceId: string): PerformanceMetrics | null {
  return asyncTracer.endTrace(traceId);
}

export async function traceAsync<T>(
  name: string,
  fn: () => Promise<T>,
  metadata?: Record<string, any>
): Promise<T> {
  return asyncTracer.traceAsync(name, fn, metadata);
}

export function addTraceMetadata(traceId: string, metadata: Record<string, any>): void {
  asyncTracer.addMetadata(traceId, metadata);
}

export type { TraceContext, PerformanceMetrics };
