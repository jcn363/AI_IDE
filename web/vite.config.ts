import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

export default defineConfig({
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
    rollupOptions: {
      output: {
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
        // Asset handling for WebGL shaders and WASM files
        assetFileNames: (assetInfo) => {
          const fileName = assetInfo.name;
          if (fileName?.endsWith('.wasm')) return 'assets/wasm/[name].[hash][extname]';
          if (fileName?.endsWith('.frag') || fileName?.endsWith('.vert')) return 'assets/shaders/[name].[hash][extname]';
          return 'assets/[name].[hash][extname]';
        },
      },
      // WASM import resolution
      external: (id) => {
        return id.endsWith('.wasm') ? false : undefined;
      },
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
