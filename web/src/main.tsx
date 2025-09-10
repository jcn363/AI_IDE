import React, { Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { CssBaseline } from '@mui/material';
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
import './styles/debugger.css';
import { webVitals } from 'web-vitals';

// Error boundary for WebAssembly and other errors
class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean }> {
  constructor(props: { children: React.ReactNode }) {
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
  webVitals(() => {
    // Core Web Vitals reporting can be integrated with backend logging here
    console.log('Web Vitals measured');
  });
};

const rootEl = document.getElementById('root')!;
const root = createRoot(rootEl);

// Initialize monitoring
if (typeof window !== 'undefined') {
  initWebVitals();

  // Set CSS property for WebGL support
  document.documentElement.style.setProperty('--webgl-supported', checkWebGLSupport() ? '1' : '0');
}

root.render(
  <React.StrictMode>
    <ErrorBoundary>
      <Suspense fallback={<div>Loading...</div>}>
        <CssBaseline />
        <App />
      </Suspense>
    </ErrorBoundary>
  </React.StrictMode>
);
