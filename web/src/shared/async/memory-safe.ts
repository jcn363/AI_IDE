import { performance } from 'node:perf_hooks';

// Declare WeakRef for environments where it's not available
declare global {
  interface WeakRef<T extends object = object> {
    readonly [Symbol.toStringTag]: "WeakRef";
    deref(): T | undefined;
  }

  var WeakRef: {
    new <T extends object>(target: T): WeakRef<T>;
    readonly prototype: WeakRef<object>;
  };
}

interface MemoryLeakTracker {
  id: string;
  name: string;
  size: number;
  allocatedAt: number;
  stack?: string;
}

interface MemoryStats {
  usedHeapSize: number;
  totalHeapSize: number;
  heapSizeLimit: number;
  externalMemory: number;
  trackingCount: number;
  leakCount: number;
}

interface MemoryThresholds {
  warningLimit: number;
  criticalLimit: number;
  leakDetectionInterval: number;
}

/**
 * Memory manager for tracking and preventing memory leaks
 */
export class MemoryManager {
  private memoryTracked = new Map<string, MemoryLeakTracker>();
  private leakThresholds: MemoryThresholds;
  private gcTimer?: NodeJS.Timeout;
  private leakDetectionEnabled = false;
  private lastMemoryUsage = 0;
  private leakDetectionInterval: number;
  private onMemoryWarning?: (stats: MemoryStats) => void;
  private onMemoryCritical?: (stats: MemoryStats) => void;

  constructor(thresholds?: Partial<MemoryThresholds>) {
    this.leakThresholds = {
      warningLimit: 512 * 1024 * 1024, // 512MB
      criticalLimit: 1024 * 1024 * 1024, // 1GB
      leakDetectionInterval: 30000, // 30 seconds
      ...thresholds
    };

    this.leakDetectionInterval = this.leakThresholds.leakDetectionInterval;
  }

  /**
   * Start automatic leak detection
   */
  startLeakDetection(): void {
    if (this.leakDetectionEnabled) return;

    this.leakDetectionEnabled = true;
    this.gcTimer = setInterval(() => {
      this.checkMemoryUsage();
      this.detectMemoryLeaks();
      this.performGarbageCollection();
    }, this.leakDetectionInterval);
  }

  /**
   * Stop leak detection
   */
  stopLeakDetection(): void {
    if (!this.leakDetectionEnabled) return;

    this.leakDetectionEnabled = false;
    if (this.gcTimer) {
      clearInterval(this.gcTimer);
      this.gcTimer = undefined;
    }
  }

  /**
   * Track memory allocation
   */
  trackAllocation(id: string, name: string, size: number, includeStack: boolean = false): void {
    const tracker: MemoryLeakTracker = {
      id,
      name,
      size,
      allocatedAt: performance.now(),
      stack: includeStack ? new Error().stack : undefined
    };

    this.memoryTracked.set(id, tracker);
  }

  /**
   * Release tracked memory
   */
  releaseAllocation(id: string): void {
    if (this.memoryTracked.delete(id)) {
      // Successfully released
    }
  }

  /**
   * Get tracked allocations
   */
  getTrackedAllocations(): MemoryLeakTracker[] {
    return Array.from(this.memoryTracked.values());
  }

  /**
   * Get memory statistics
   */
  getMemoryStats(): MemoryStats {
    let stats: MemoryStats;

    if (typeof global !== 'undefined' && (global as any).v8) {
      // Node.js environment
      const memoryUsage = require('v8').getHeapStatistics();
      stats = {
        usedHeapSize: memoryUsage.used_heap_size,
        totalHeapSize: memoryUsage.total_heap_size,
        heapSizeLimit: memoryUsage.heap_size_limit,
        externalMemory: memoryUsage.external_memory,
        trackingCount: this.memoryTracked.size,
        leakCount: this.calculateLeakCount()
      };
    } else {
      // Browser environment
      stats = {
        usedHeapSize: 0, // Placeholder
        totalHeapSize: 0,
        heapSizeLimit: 0,
        externalMemory: 0,
        trackingCount: this.memoryTracked.size,
        leakCount: this.calculateLeakCount()
      };

      if ('memory' in performance) {
        const mem = (performance as any).memory;
        stats.usedHeapSize = mem.usedJSHeapSize;
        stats.totalHeapSize = mem.totalJSHeapSize;
        stats.heapSizeLimit = mem.jsHeapSizeLimit;
      }
    }

    return stats;
  }

  /**
   * Force garbage collection (if available)
   */
  forceGarbageCollection(): void {
    if (typeof global !== 'undefined' && (global as any).gc) {
      (global as any).gc();
    }
  }

  /**
   * Set memory warning callback
   */
  onWarning(callback: (stats: MemoryStats) => void): void {
    this.onMemoryWarning = callback;
  }

  /**
   * Set memory critical callback
   */
  onCritical(callback: (stats: MemoryStats) => void): void {
    this.onMemoryCritical = callback;
  }

  private calculateLeakCount(): number {
    // Consider allocations older than threshold as potential leaks
    const now = performance.now();
    const leakThreshold = now - (this.leakDetectionInterval * 5); // 5x the detection interval

    return Array.from(this.memoryTracked.values())
      .filter(tracker => tracker.allocatedAt < leakThreshold)
      .length;
  }

  private checkMemoryUsage(): void {
    const stats = this.getMemoryStats();
    const { warningLimit, criticalLimit } = this.leakThresholds;

    if (stats.usedHeapSize > criticalLimit && this.onMemoryCritical) {
      this.onMemoryCritical(stats);
    } else if (stats.usedHeapSize > warningLimit && this.onMemoryWarning) {
      this.onMemoryWarning(stats);
    }
  }

  private detectMemoryLeaks(): void {
    const now = performance.now();
    const potentialLeaks = Array.from(this.memoryTracked.values())
      .filter(tracker => (now - tracker.allocatedAt) > (this.leakDetectionInterval * 10));

    if (potentialLeaks.length > 0) {
      console.warn(`Potential memory leak(s) detected: ${potentialLeaks.length} allocations`);
      potentialLeaks.forEach(leak => {
        console.warn(`  - ${leak.name} (${leak.id}): ${(now - leak.allocatedAt).toFixed(2)}ms since allocation`);
        if (leak.stack) {
          console.warn(`    Stack: ${leak.stack}`);
        }
      });
    }
  }

  private performGarbageCollection(): void {
    // Attempt GC if available
    this.forceGarbageCollection();
  }
}

/**
 * Memory-safe wrapper for functions
 */
export function withMemoryTracking<T>(
  name: string,
  fn: () => Promise<T>
): Promise<T> {
  const memoryManager = new MemoryManager();
  const id = `${name}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  return (async () => {
    // Track memory before execution
    memoryManager.trackAllocation(id, name, 0, true);

    try {
      const result = await fn();
      return result;
    } finally {
      // Release tracking
      memoryManager.releaseAllocation(id);
    }
  })();
}

/**
 * Automatic resource disposer
 */
export class ResourceDisposer {
  private resources = new Set<() => void | Promise<void>>();
  private disposed = false;

  /**
   * Add a resource cleanup function
   */
  addResource(cleanup: () => void | Promise<void>): void {
    if (this.disposed) {
      throw new Error('ResourceDisposer is already disposed');
    }
    this.resources.add(cleanup);
  }

  /**
   * Dispose of all resources
   */
  async dispose(): Promise<void> {
    if (this.disposed) return;

    this.disposed = true;

    const cleanupPromises = Array.from(this.resources.entries())
      .map(async ([resource]) => {
        try {
          await resource();
        } catch (error) {
          console.error('Error during resource cleanup:', error);
        }
      });

    await Promise.all(cleanupPromises);
    this.resources.clear();
  }

  /**
   * Check if disposed
   */
  isDisposed(): boolean {
    return this.disposed;
  }
}

/**
 * Memory-safe cache with automatic expiration
 */
export class SafeCache<T> {
  private cache = new Map<string, { value: T; expires: number; size: number }>();
  private maxSize: number;
  private defaultTTL: number;
  private memoryManager?: MemoryManager;

  constructor(options: { maxSize?: number; defaultTTL?: number; enableMemoryTracking?: boolean } = {}) {
    this.maxSize = options.maxSize || 100;
    this.defaultTTL = options.defaultTTL || 300000; // 5 minutes

    if (options.enableMemoryTracking) {
      this.memoryManager = new MemoryManager();

      // Start a cleanup timer
      setInterval(() => {
        this.evictExpired();
        this.evictOverSize();
      }, Math.min(this.defaultTTL, 60000)); // Check every minute or TTL, whichever is smaller
    }
  }

  /**
   * Set a value in the cache
   */
  set(key: string, value: T, ttl?: number): void {
    if (this.cache.size >= this.maxSize) {
      this.evictLRU();
    }

    const expires = Date.now() + (ttl || this.defaultTTL);
    const size = this.estimateSize(value);

    if (this.memoryManager) {
      this.memoryManager.trackAllocation(`cache-${key}`, `Cache item: ${key}`, size);
    }

    this.cache.set(key, { value, expires, size });
  }

  /**
   * Get a value from the cache
   */
  get(key: string): T | undefined {
    const entry = this.cache.get(key);
    if (!entry) return undefined;

    if (Date.now() > entry.expires) {
      this.cache.delete(key);
      return undefined;
    }

    return entry.value;
  }

  /**
   * Delete a value from the cache
   */
  delete(key: string): boolean {
    const existed = this.cache.delete(key);

    if (existed && this.memoryManager) {
      this.memoryManager.releaseAllocation(`cache-${key}`);
    }

    return existed;
  }

  /**
   * Clear all cache entries
   */
  clear(): void {
    if (this.memoryManager) {
      for (const key of this.cache.keys()) {
        this.memoryManager.releaseAllocation(`cache-${key}`);
      }
    }
    this.cache.clear();
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.cache.size;
  }

  private estimateSize(value: any): number {
    // Rough size estimation
    if (typeof value === 'string') return value.length * 2;
    if (typeof value === 'number') return 8;
    if (typeof value === 'boolean') return 1;
    if (Array.isArray(value)) return value.length * 100; // Rough estimate
    if (typeof value === 'object') return Object.keys(value).length * 200; // Rough estimate
    return 100; // Default
  }

  private evictLRU(): void {
    // Evict the oldest entry
    let oldestKey: string | null = null;
    let oldestExpire = Infinity;

    for (const [key, entry] of this.cache) {
      if (entry.expires < oldestExpire) {
        oldestExpire = entry.expires;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      this.delete(oldestKey);
    }
  }

  private evictExpired(): void {
    const now = Date.now();
    for (const [key, entry] of this.cache) {
      if (now > entry.expires) {
        this.delete(key);
      }
    }
  }

  private evictOverSize(): void {
    const entries = Array.from(this.cache.entries())
      .sort(([, a], [, b]) => a.expires - b.expires); // Sort by expiration

    while (this.cache.size > this.maxSize) {
      const [key] = entries.shift()!;
      this.delete(key);
    }
  }
}

/**
 * Utility for creating weak references (where available)
 */
export function createWeakRef<T extends object>(obj: T): WeakRef<T> | T {
  if (typeof WeakRef !== 'undefined') {
    return new WeakRef(obj);
  }
  // Fallback for environments without WeakRef
  return obj;
}

/**
 * Global memory manager instance
 */
export const globalMemoryManager = new MemoryManager();

export type { MemoryLeakTracker, MemoryStats, MemoryThresholds };