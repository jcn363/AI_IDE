// Preloading utilities for performance optimization
// Preload critical resources and components

// Preload critical fonts
export const preloadFonts = () => {
  const fontLinks = [
    'https://fonts.googleapis.com/css2?family=Roboto:wght@300;400;500;700&display=swap',
  ];

  fontLinks.forEach((href) => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.href = href;
    link.as = 'style';
    document.head.appendChild(link);
  });
};

// Preload critical images
export const preloadImages = (imageUrls: string[]) => {
  imageUrls.forEach((url) => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.href = url;
    link.as = 'image';
    document.head.appendChild(link);
  });
};

// Intelligent preloading based on route
export const preloadRoute = (route: string) => {
  const preloadMap: Record<string, string[]> = {
    '/editor': ['monaco-editor', '@monaco-editor/react'],
    '/debugger': ['react-window', 'react-virtuoso'],
    '/dependencies': ['d3'],
    '/testing': ['react-syntax-highlighter'],
  };

  const modules = preloadMap[route];
  if (modules) {
    modules.forEach((module) => {
      import(/* webpackPreload: true */ module).catch((err) =>
        console.warn(`Failed to preload ${module}:`, err)
      );
    });
  }
};

// Preload critical components on hover/intent
export const preloadOnIntent = (componentPath: string) => {
  return new Promise((resolve) => {
    const link = document.createElement('link');
    link.rel = 'prefetch';
    link.href = componentPath;
    link.onload = resolve;
    link.onerror = resolve; // Don't fail if prefetch fails
    document.head.appendChild(link);
  });
};

// Service worker for caching strategies (if supported)
export const setupServiceWorker = async () => {
  if ('serviceWorker' in navigator) {
    try {
      const registration = await navigator.serviceWorker.register('/sw.js');
      console.log('Service Worker registered:', registration);
    } catch (error) {
      console.warn('Service Worker registration failed:', error);
    }
  }
};

// Intersection Observer for lazy loading images
export const setupLazyLoading = () => {
  const imageObserver = new IntersectionObserver((entries, observer) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        const img = entry.target as HTMLImageElement;
        if (img.dataset.src) {
          img.src = img.dataset.src;
          img.classList.remove('lazy');
          observer.unobserve(img);
        }
      }
    });
  });

  // Observe all images with lazy class
  document.querySelectorAll('img.lazy').forEach((img) => {
    imageObserver.observe(img);
  });
};

// Resource hints for critical resources
export const addResourceHints = () => {
  const hints = [
    { rel: 'dns-prefetch', href: '//fonts.googleapis.com' },
    { rel: 'preconnect', href: '//fonts.gstatic.com', crossorigin: true },
  ];

  hints.forEach(({ rel, href, crossorigin }) => {
    const link = document.createElement('link');
    link.rel = rel;
    link.href = href;
    if (crossorigin) link.crossOrigin = crossorigin;
    document.head.appendChild(link);
  });
};

// Initialize all performance optimizations
export const initPerformanceOptimizations = () => {
  // Add resource hints
  addResourceHints();

  // Setup lazy loading
  setupLazyLoading();

  // Setup service worker
  setupServiceWorker();

  // Preload critical fonts
  preloadFonts();
};
