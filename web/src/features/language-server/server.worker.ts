/// <reference no-default-lib="true"/>
/// <reference lib="webworker" />

import { createServer } from './server';

// Create a message port for communication with the main thread
const port = self as unknown as Worker;

// Initialize the WebAssembly module and start the language server
async function initialize() {
  try {
    // Dynamic import for Web Worker compatibility (avoid top-level await)
    const wasmWasiWorker = await import('@vscode/wasm-wasi/worker');
    const { initializeWasm } = wasmWasiWorker as unknown as {
      initializeWasm: () => Promise<{ wasi: unknown }>;
    };

    // Initialize WASM
    const wasm = await initializeWasm();

    // Create the language server and start listening
    const server = createServer(wasm as any, (message) => {
      port.postMessage(message);
    });
    server.listen();

    // Forward messages between the worker and the language server
    port.onmessage = (event) => {
      if (server.onMessage) {
        server.onMessage(event.data);
      }
    };

    console.log('Rust Analyzer Web Worker initialized');
  } catch (error) {
    console.error('Failed to initialize Rust Analyzer:', error);
    port.postMessage({
      jsonrpc: '2.0',
      method: 'window/showMessage',
      params: {
        type: 1, // Error
        message: `Failed to initialize Rust Analyzer: ${String(error)}`,
      },
    });
  }
}

// Start the initialization
initialize();
