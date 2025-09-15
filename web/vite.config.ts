import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import { nodePolyfills } from 'vite-plugin-node-polyfills';
import { visualizer } from 'rollup-plugin-visualizer';

export default defineConfig(({ mode }) => ({
  plugins: [
    react({
      jsxRuntime: 'automatic',
      babel: {
        babelrc: false,
        configFile: false,
        plugins: [
          ['@babel/plugin-transform-react-jsx', { runtime: 'automatic' }],
        ],
      },
    }),
    wasm({
      target: 'node',
    }),
    nodePolyfills({
      // To add polyfills only for specific modules, specify them here
      include: ['path', 'os', 'crypto'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
    }),
    // Bundle analyzer - only enabled in analyze mode
    ...(mode === 'analyze' ? [visualizer({
      filename: 'dist/bundle-analysis.html',
      open: true,
      template: 'treemap',
      gzipSize: true,
      brotliSize: true,
    })] : []),
  ],
  define: {
    'process.env': {},
    global: 'globalThis',
    // Enable React 19 Concurrent Mode features
    __REACT_CONCURRENT_MODE: true,
    __REACT_DEVTOOLS_GLOBAL_HOOK__: false,
  },
  worker: {
    format: 'es',
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: true,
    chunkSizeWarningLimit: 1500,
    cssCodeSplit: true,
    // Optimize for large codebases
    minify: 'esbuild',
    // Enable compression for better performance
    reportCompressedSize: true,
    // Asset optimization
    assetsInlineLimit: 4096, // Inline assets smaller than 4kb
    // Advanced tree shaking
    rollupOptions: {
      treeshake: {
        preset: 'recommended',
        moduleSideEffects: false,
      },
      output: {
        // ... existing output config
    // Preload modules for better performance
    modulePreload: {
      polyfill: false,
    },
    rollupOptions: {
      output: {
        // Asset optimization with better naming and compression
        assetFileNames: (assetInfo) => {
          const fileName = assetInfo.name;
          if (fileName?.endsWith('.png') || fileName?.endsWith('.jpg') || fileName?.endsWith('.jpeg') || fileName?.endsWith('.gif') || fileName?.endsWith('.webp')) {
            return 'assets/images/[name].[hash][extname]';
          }
          if (fileName?.endsWith('.svg')) {
            return 'assets/icons/[name].[hash][extname]';
          }
          if (fileName?.endsWith('.woff') || fileName?.endsWith('.woff2') || fileName?.endsWith('.ttf') || fileName?.endsWith('.eot')) {
            return 'assets/fonts/[name].[hash][extname]';
          }
          return 'assets/[name].[hash][extname]';
        },
        // Optimize chunk naming for better caching
        chunkFileNames: 'assets/js/[name]-[hash].js',
        entryFileNames: 'assets/js/[name]-[hash].js',
        manualChunks: (id) => {
          // Vendor chunk for node_modules
          if (id.includes('node_modules')) {
            // React ecosystem
            if (id.includes('react') || id.includes('react-dom') || id.includes('react-router')) {
              return 'react-vendor';
            }
            // Material UI ecosystem
            if (id.includes('@mui') || id.includes('@emotion')) {
              return 'mui-vendor';
            }
            // Monaco Editor
            if (id.includes('monaco') || id.includes('@monaco')) {
              return 'monaco-vendor';
            }
            // VSCode/LSP libraries
            if (id.includes('vscode-') || id.includes('languageclient')) {
              return 'lsp-vendor';
            }
            // Tauri libraries
            if (id.includes('@tauri-apps')) {
              return 'tauri-vendor';
            }
            // D3 and charting libraries
            if (id.includes('d3') || id.includes('three') || id.includes('@types/three')) {
              return 'visualization-vendor';
            }
            // UI libraries (Ant Design, etc.)
            if (id.includes('antd')) {
              return 'ui-vendor';
            }
            // Other vendor libraries
            return 'vendor';
          }

          // Application chunks
          if (id.includes('src/features/ai/') || id.includes('src/features/editor/')) {
            return 'ai-editor-chunk';
          }
          if (id.includes('src/features/cargoToml/') || id.includes('src/features/dependency/')) {
            return 'cargo-deps-chunk';
          }
          if (id.includes('src/features/terminal/') || id.includes('src/features/command-palette/')) {
            return 'terminal-cmd-chunk';
          }
          if (id.includes('src/features/performance/') || id.includes('src/features/search/')) {
            return 'performance-search-chunk';
          }

          // Pages chunks
          if (id.includes('src/pages/')) {
            const pageMatch = id.match(/src\/pages\/([^\/]+)/);
            if (pageMatch) {
              return `${pageMatch[1].toLowerCase()}-page`;
            }
            return 'pages-chunk';
          }
        },
      },
      // WASM import resolution
      external: (id) => {
        return id.endsWith('.wasm') ? false : undefined;
      },
  },
  optimizeDeps: {
    esbuildOptions: {
      // Node.js global to browser globalThis
      define: {
        global: 'globalThis',
      },
      loader: {
        '.wasm': 'binary',
        '.frag': 'text',
        '.vert': 'text',
      },
    },
    // Exclude problematic dependencies
    exclude: ['@wasm-tool/wasm-pack-plugin'],
  },
  // Server configurations for development
  server: {
    headers: {
      'Cross-Origin-Embedder-Policy': 'credentialless',
      'Cross-Origin-Opener-Policy': 'same-origin',
    },
  },
});
