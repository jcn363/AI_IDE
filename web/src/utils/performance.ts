// Performance monitoring utilities
// Track loading times, bundle sizes, and user experience metrics
import React from 'react';

export interface PerformanceMetrics {
  loadTime: number;
  bundleSize: number;
  firstPaint: number;
  firstContentfulPaint: number;
  largestContentfulPaint: number;
  firstInputDelay: number;
  cumulativeLayoutShift: number;
}

// Track component load times
export class PerformanceTracker {
  private static instance: PerformanceTracker;
  private metrics: Map<string, number> = new Map();

  static getInstance(): PerformanceTracker {
    if (!PerformanceTracker.instance) {
      PerformanceTracker.instance = new PerformanceTracker();
    }
    return PerformanceTracker.instance;
  }

  startTracking(componentName: string) {
    this.metrics.set(`${componentName}_start`, performance.now());
  }

  endTracking(componentName: string): number {
    const startTime = this.metrics.get(`${componentName}_start`);
    if (!startTime) return 0;

    const loadTime = performance.now() - startTime;
    this.metrics.set(`${componentName}_loadTime`, loadTime);

    console.log(`Component ${componentName} loaded in ${loadTime.toFixed(2)}ms`);
    return loadTime;
  }

  getMetrics(): Record<string, number> {
    const result: Record<string, number> = {};
    this.metrics.forEach((value, key) => {
      result[key] = value;
    });
    return result;
  }

  // Track Web Vitals
  trackWebVitals() {
    if (typeof window !== 'undefined' && 'web-vitals' in window) {
      import('web-vitals').then(({ getCLS, getFID, getFCP, getLCP, getTTFB }) => {
        getCLS(console.log);
        getFID(console.log);
        getFCP(console.log);
        getLCP(console.log);
        getTTFB(console.log);
      });
    }
  }

  // Track bundle size impact
  trackBundleSize() {
    // This would be populated by the build process
    const bundleSizes = {
      total: 0,
      vendor: 0,
      main: 0,
    };

    console.log('Bundle Sizes:', bundleSizes);
    return bundleSizes;
  }

  // Memory usage tracking
  trackMemoryUsage() {
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      console.log('Memory Usage:', {
        used: Math.round((memory.usedJSHeapSize / 1048576) * 100) / 100 + ' MB',
        total: Math.round((memory.totalJSHeapSize / 1048576) * 100) / 100 + ' MB',
        limit: Math.round((memory.jsHeapSizeLimit / 1048576) * 100) / 100 + ' MB',
      });
    }
  }

  // Network timing
  trackNetworkTiming() {
    if ('getEntriesByType' in performance) {
      const navigation = performance.getEntriesByType(
        'navigation'
      )[0] as PerformanceNavigationTiming;
      if (navigation) {
        console.log('Navigation Timing:', {
          loadTime: navigation.loadEventEnd - navigation.loadEventStart,
          domContentLoaded:
            navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
          firstByte: navigation.responseStart - navigation.requestStart,
        });
      }
    }
  }
}

// Performance monitoring hook for React components
export const usePerformanceMonitoring = (componentName: string) => {
  const tracker = PerformanceTracker.getInstance();

  React.useEffect(() => {
    tracker.startTracking(componentName);

    return () => {
      tracker.endTracking(componentName);
    };
  }, [componentName]);

  return {
    trackAction: (actionName: string) => tracker.startTracking(`${componentName}_${actionName}`),
    endAction: (actionName: string) => tracker.endTracking(`${componentName}_${actionName}`),
  };
};

// Bundle size monitoring
export const monitorBundleSize = () => {
  // This would be integrated with build tools to track bundle size changes
  const thresholds = {
    warning: 2 * 1024 * 1024, // 2MB
    error: 5 * 1024 * 1024, // 5MB
  };

  // Mock implementation - would be replaced with actual build data
  const currentSize = 1.5 * 1024 * 1024; // 1.5MB

  if (currentSize > thresholds.error) {
    console.error(
      `Bundle size (${(currentSize / 1024 / 1024).toFixed(2)}MB) exceeds error threshold`
    );
  } else if (currentSize > thresholds.warning) {
    console.warn(
      `Bundle size (${(currentSize / 1024 / 1024).toFixed(2)}MB) exceeds warning threshold`
    );
  } else {
    console.log(`Bundle size: ${(currentSize / 1024 / 1024).toFixed(2)}MB ✓`);
  }
};

// Initialize performance monitoring
export const initPerformanceMonitoring = () => {
  const tracker = PerformanceTracker.getInstance();

  // Track Web Vitals
  tracker.trackWebVitals();

  // Monitor bundle size
  monitorBundleSize();

  // Track memory usage periodically
  setInterval(() => {
    tracker.trackMemoryUsage();
  }, 30000); // Every 30 seconds

  // Track network timing
  tracker.trackNetworkTiming();

  console.log('Performance monitoring initialized');
};

// React lazy component wrapper with performance tracking
export const lazyWithPerformance = (importFunc: () => Promise<any>, componentName: string) => {
  return React.lazy(() => {
    const tracker = PerformanceTracker.getInstance();
    tracker.startTracking(componentName);

    return importFunc().then((module) => {
      tracker.endTracking(componentName);
      return module;
    });
  });
};
