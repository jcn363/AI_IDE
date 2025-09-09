import { performance } from 'perf_hooks';
import { performanceAnalyzer } from './PerformanceAnalyzer';
import { PerformanceMetric } from '../types';

type MetricCollectorOptions = {
  collectInterval?: number; // in milliseconds
  maxSamples?: number;
};

export class MetricsCollector {
  private collectInterval: number;
  private maxSamples: number;
  private intervalId: NodeJS.Timeout | null = null;
  private metrics: Map<string, PerformanceMetric[]> = new Map();
  private customMetrics: Map<string, () => number> = new Map();

  constructor(options: MetricCollectorOptions = {}) {
    this.collectInterval = options.collectInterval || 5000; // Default 5 seconds
    this.maxSamples = options.maxSamples || 100; // Keep last 100 samples per metric
  }

  public start() {
    if (this.intervalId) {
      this.stop();
    }

    this.intervalId = setInterval(() => this.collectMetrics(), this.collectInterval);
    return this;
  }

  public stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
    return this;
  }

  public registerCustomMetric(id: string, collector: () => number) {
    this.customMetrics.set(id, collector);
    return this;
  }

  public unregisterCustomMetric(id: string) {
    this.customMetrics.delete(id);
    return this;
  }

  public getMetricHistory(metricId: string): PerformanceMetric[] {
    return this.metrics.get(metricId) || [];
  }

  public clearMetrics() {
    this.metrics.clear();
    return this;
  }

  private collectMetrics() {
    const timestamp = Date.now();

    // Collect system metrics
    this.collectSystemMetrics(timestamp);

    // Collect custom metrics
    this.collectCustomMetrics(timestamp);
  }

  private collectSystemMetrics(timestamp: number) {
    try {
      // Collect memory usage
      if (global.gc) {
        global.gc();
      }
      
      const memoryUsage = process.memoryUsage();
      
      // Convert bytes to MB for better readability
      const toMB = (bytes: number) => Math.round((bytes / 1024 / 1024) * 100) / 100;
      
      this.addMetric({
        id: 'memory.heapUsed',
        name: 'Heap Used',
        value: toMB(memoryUsage.heapUsed),
        unit: 'MB',
        timestamp,
      });
      
      this.addMetric({
        id: 'memory.heapTotal',
        name: 'Heap Total',
        value: toMB(memoryUsage.heapTotal),
        unit: 'MB',
        timestamp,
      });
      
      this.addMetric({
        id: 'memory.rss',
        name: 'Resident Set Size',
        value: toMB(memoryUsage.rss),
        unit: 'MB',
        timestamp,
      });
      
      // Collect CPU usage (this is a simple implementation)
      const startUsage = process.cpuUsage();
      const startTime = performance.now();
      
      // Small delay to measure CPU usage
      setTimeout(() => {
        const endTime = performance.now();
        const endUsage = process.cpuUsage(startUsage);
        
        const elapsedTime = (endTime - startTime) * 1000; // convert to microseconds
        const cpuPercent = ((endUsage.user + endUsage.system) / elapsedTime) * 100;
        
        this.addMetric({
          id: 'cpu.usage',
          name: 'CPU Usage',
          value: Math.min(100, Math.max(0, cpuPercent)),
          unit: '%',
          timestamp,
        });
      }, 100);
      
    } catch (error) {
      console.error('Error collecting system metrics:', error);
    }
  }

  private collectCustomMetrics(timestamp: number) {
    this.customMetrics.forEach((collector, id) => {
      try {
        const value = collector();
        this.addMetric({
          id: `custom.${id}`,
          name: id,
          value,
          unit: '',
          timestamp,
        });
      } catch (error) {
        console.error(`Error collecting custom metric ${id}:`, error);
      }
    });
  }

  private addMetric(metric: PerformanceMetric) {
    const { id } = metric;
    
    if (!this.metrics.has(id)) {
      this.metrics.set(id, []);
    }
    
    const metrics = this.metrics.get(id)!;
    metrics.push(metric);
    
    // Trim old metrics if we exceed max samples
    if (metrics.length > this.maxSamples) {
      metrics.shift();
    }
    
    // Also send to the performance analyzer
    performanceAnalyzer.addMetric(metric);
  }
}

// Export a singleton instance
export const metricsCollector = new MetricsCollector();

// Start collecting metrics by default if in a Node.js environment
if (typeof window === 'undefined') {
  metricsCollector.start();
}
