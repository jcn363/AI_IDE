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
      include: ['path', 'os', 'crypto'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
    }),
    ...(mode === 'analyze' ? [
      visualizer({
        filename: 'dist/bundle-analysis.html',
        open: true,
        template: 'treemap',
        gzipSize: true,
        brotliSize: true,
      })
    ] : []),
  ],
  define: {
    'process.env': {},
    global: 'globalThis',
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
    sourcemap: mode === 'development',
    chunkSizeWarningLimit: 1000, // Reduced to catch more issues
    cssCodeSplit: true,
    minify: 'esbuild',
    reportCompressedSize: true,
    assetsInlineLimit: 4096,
    // Enable compression and optimization
    compress: true,
    // Optimize CSS
    cssMinify: true,
    // Optimize dependencies
    modulePreload: {
      polyfill: false,
    },
    rollupOptions: {
      output: {
        // Enhanced asset naming for better caching
        assetFileNames: (assetInfo) => {
          const fileName = assetInfo.name;
          if (fileName?.endsWith('.png') || fileName?.endsWith('.jpg') || fileName?.endsWith('.jpeg') || fileName?.endsWith('.gif') || fileName?.endsWith('.webp')) {
            return 'assets/images/[name].[hash][extname]';
          }
          if (fileName?.endsWith('.svg')) {
            return 'assets/icons/[name].[hash][extname]';
          }
          if (fileName?.endsWith('.css')) {
            return 'assets/css/[name].[hash][extname]';
          }
          if (fileName?.endsWith('.woff') || fileName?.endsWith('.woff2') || fileName?.endsWith('.ttf')) {
            return 'assets/fonts/[name].[hash][extname]';
          }
          return 'assets/[name].[hash][extname]';
        },
        // Enhanced chunk naming for better code splitting
        chunkFileNames: (chunkInfo) => {
          const facadeModuleId = chunkInfo.facadeModuleId ? chunkInfo.facadeModuleId.split('/').pop() : 'chunk';
          return `assets/js/${facadeModuleId}-[hash].js`;
        },
        entryFileNames: 'assets/js/[name]-[hash].js',
        // Manual chunks for better code splitting
        manualChunks: {
          // Monaco Editor in its own chunk
          'monaco-editor': ['@monaco-editor/react', 'monaco-editor', '@monaco-editor/loader'],
          // Material UI in its own chunk
          'material-ui': ['@mui/material', '@mui/icons-material', '@mui/system'],
          // React and related in their own chunks
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          // Redux in its own chunk
          'redux-vendor': ['@reduxjs/toolkit', 'react-redux'],
        },
      },
      // External dependencies that shouldn't be bundled
      external: [],
    },
  },
  optimizeDeps: {
    esbuildOptions: {
      define: {
        global: 'globalThis',
      },
    },
    exclude: ['@wasm-tool/wasm-pack-plugin'],
  },
  server: {
    headers: {
      'Cross-Origin-Embedder-Policy': 'credentialless',
      'Cross-Origin-Opener-Policy': 'same-origin',
    },
  },
}));
