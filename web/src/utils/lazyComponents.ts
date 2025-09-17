// Lazy loading utilities for React components
// This file provides lazy-loaded versions of heavy components to improve initial load times

import React from 'react';

// Lazy load Monaco Editor components
export const LazyMonacoEditor = React.lazy(() =>
  import('../components/MonacoEditorWrapper')
);

// Lazy load heavy Material UI components
export const LazyMuiDialog = React.lazy(() =>
  import('@mui/material').then(module => ({ default: module.Dialog }))
);

export const LazyMuiMenu = React.lazy(() =>
  import('@mui/material').then(module => ({ default: module.Menu }))
);

export const LazyMuiSelect = React.lazy(() =>
  import('@mui/material').then(module => ({ default: module.Select }))
);

// Lazy load large component directories
export const LazyFileExplorer = React.lazy(() =>
  import('../components/FileExplorer')
);

export const LazyDebuggerPanel = React.lazy(() =>
  import('../components/DebuggerPanel')
);

export const LazyPerformancePanel = React.lazy(() =>
  import('../components/PerformancePanel')
);

// Lazy load AI/ML components
export const LazyMultiModalAI = React.lazy(() =>
  import('../components/MultiModalAI')
);

// Lazy load collaboration components
export const LazyCollaborationPanel = React.lazy(() =>
  import('../components/CollaborationPanel')
);

// Lazy load plugin-related components
export const LazyPluginMarketplace = React.lazy(() =>
  import('../components/PluginMarketplace')
);

// Performance monitoring utility
export const LAZY_LOADING_CONFIG = {
  // Components that should be prefetched after initial load
  prefetchComponents: [
    'LazyFileExplorer',
    'LazyDebuggerPanel',
    'LazyMonacoEditor',
  ],
  // Loading strategies
  loadingStrategies: {
    immediate: 'Load immediately when imported',
    onDemand: 'Load only when needed',
    prefetch: 'Load after initial page load',
  },
  // Bundle size thresholds
  thresholds: {
    small: 50, // KB - can load immediately
    medium: 200, // KB - use lazy loading
    large: 500, // KB - use lazy loading with prefetch
  },
};

// Prefetch function for critical components
export const prefetchComponent = (componentName: string) => {
  // This function can be called to prefetch components after initial load
  // Implementation would depend on specific requirements
  console.log(`Prefetching component: ${componentName}`);
};

// Utility to create lazy components with error boundaries
export const createLazyComponent = (
  importFn: () => Promise<any>,
  fallback?: React.ComponentType<any>
) => {
  const LazyComponent = React.lazy(importFn);

  return (props: any) => (
    <React.Suspense
      fallback={fallback ? React.createElement(fallback) : <div>Loading...</div>}
    >
      <LazyComponent {...props} />
    </React.Suspense>
  );
};