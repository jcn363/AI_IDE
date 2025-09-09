export * from './types';
export * from './services/PerformanceAnalyzer';
export * from './services/MetricsCollector';
export * from './components/PerformanceRecommendations';
export * from './hooks/usePerformanceAnalysis';

export { default as PerformanceDashboard } from './components/PerformanceDashboard';

export const initializePerformanceModule = () => {
  // Initialize any performance monitoring or metrics collection here
  if (process.env.NODE_ENV === 'development') {
    // Enable additional debug metrics in development
    const { metricsCollector } = require('./services/MetricsCollector');
    metricsCollector.start();
  }
  
  console.log('Performance module initialized');
};
