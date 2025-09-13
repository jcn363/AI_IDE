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
    // Asset optimization
    assetsInlineLimit: 4096, // Inline assets smaller than 4kb
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
        manualChunks: {
          // Split heavy libraries into separate chunks
          react: ['react', 'react-dom'],
          mui: ['@mui/material', '@mui/icons-material', '@emotion/react', '@emotion/styled'],
          monaco: ['monaco-editor', '@monaco-editor/react', '@codingame/monaco-languageclient'],
          vscode: [
            'vscode-jsonrpc',
            'vscode-languageserver',
            'vscode-languageserver-protocol',
            'vscode-languageserver-types',
            'vscode-ws-jsonrpc',
          ],
          tauri: ['@tauri-apps/api', '@tauri-apps/plugin-fs', '@tauri-apps/plugin-dialog'],
          // New AI/ML and performance libraries
          virtualization: ['react-window', 'react-virtuoso'],
          webgl: ['three', '@types/three'],
          webvitals: ['web-vitals'],
          // WASM chunks
          wasm: ['./src/wasm/**/*'],
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
