import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

export default defineConfig({
  plugins: [
    react(),
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
        },
      },
    },
  },
  optimizeDeps: {
    esbuildOptions: {
      // Node.js global to browser globalThis
      define: {
        global: 'globalThis',
      },
    },
  },
});
