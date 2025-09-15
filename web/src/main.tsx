import React, { Suspense } from 'react';
import type { ReactNode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { CssBaseline } from './components/shared/MaterialUI';
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
import './styles/debugger.css';
import { getCLS, getFID, getLCP, getTTFB } from 'web-vitals';
import { initPerformanceOptimizations } from './utils/preload';
import { initPerformanceMonitoring } from './utils/performance';

// Error boundary for WebAssembly and other errors
class ErrorBoundary extends React.Component<{ children: ReactNode }, { hasError: boolean }> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return <div>Error loading application. Please refresh the page.</div>;
    }
    return this.props.children;
  }
}

// WebGL support detection
const checkWebGLSupport = (): boolean => {
  try {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
    return !!(gl && gl instanceof WebGLRenderingContext);
  } catch {
    return false;
  }
};

// Initialize Web Vitals monitoring
const initWebVitals = () => {
  getCLS((metric) => {
    // Core Web Vitals reporting can be integrated with backend logging here
    console.log('CLS measured:', metric);
  });
  getFID((metric) => {
    // Core Web Vitals reporting can be integrated with backend logging here
    console.log('FID measured:', metric);
  });
  getLCP((metric) => {
    // Core Web Vitals reporting can be integrated with backend logging here
    console.log('LCP measured:', metric);
  });
  getTTFB((metric) => {
    // Core Web Vitals reporting can be integrated with backend logging here
    console.log('TTFB measured:', metric);
  });
};

// Service Worker registration for caching
const registerServiceWorker = async () => {
  if ('serviceWorker' in navigator) {
    try {
      const registration = await navigator.serviceWorker.register('/sw.js');
      console.log('Service Worker registered:', registration);

      // Handle updates
      registration.addEventListener('updatefound', () => {
        const newWorker = registration.installing;
        if (newWorker) {
          newWorker.addEventListener('statechange', () => {
            if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
              // New content is available, notify user
              console.log(
                'New content is available and will be used when all tabs for this page are closed.'
              );
            }
          });
        }
      });
    } catch (error) {
      console.log('Service Worker registration failed:', error);
    }
  }
};

const rootEl = document.getElementById('root');
if (!rootEl) {
  console.error('Root element not found. Ensure there is a div with id="root" in the HTML.');
  throw new Error('Root element not found');
}
const root = createRoot(rootEl);

// Initialize monitoring
if (typeof window !== 'undefined') {
  initWebVitals();

  // Initialize performance optimizations
  initPerformanceOptimizations();

  // Initialize performance monitoring
  initPerformanceMonitoring();

  // Register service worker for caching
  registerServiceWorker();

  // Set CSS property for WebGL support
  document.documentElement.style.setProperty('--webgl-supported', checkWebGLSupport() ? '1' : '0');
}

const Loading = () => (
  <div
    role="status"
    aria-live="polite"
    style={{ minHeight: '200px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
  >
    Loading...
  </div>
);

root.render(
  <React.StrictMode>
    <CssBaseline />
    <ErrorBoundary>
      <Suspense fallback={<Loading />}>
        <App />
      </Suspense>
    </ErrorBoundary>
  </React.StrictMode>
);
