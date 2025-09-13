import * as wasmWasi from '@vscode/wasm-wasi/v1';

// Define minimal types for language server protocol
interface InitializeParams {
  processId: number | null;
  rootUri: string | null;
  capabilities: any;
  workspaceFolders?: { uri: string; name: string }[] | null;
}

interface InitializeResult {
  capabilities: {
    textDocumentSync: number;
    completionProvider?: {
      resolveProvider?: boolean;
      triggerCharacters?: string[];
    };
    hoverProvider?: boolean;
    definitionProvider?: boolean;
    referencesProvider?: boolean;
    documentSymbolProvider?: boolean;
    workspaceSymbolProvider?: boolean;
    codeActionProvider?: boolean;
    documentFormattingProvider?: boolean;
    documentRangeFormattingProvider?: boolean;
    renameProvider?: boolean;
    executeCommandProvider?: {
      commands: string[];
    };
  };
  serverInfo?: {
    name: string;
    version: string;
  };
}

// Create a simple language server implementation
export function createServer(_wasm: wasmWasi.Wasm, onSendMessage?: (message: any) => void) {
  // Simple message handler for the web worker
  const messageHandler = (event: MessageEvent) => {
    const message = event.data;

    // Handle initialization request
    if (message.method === 'initialize') {
      const result: InitializeResult = {
        capabilities: {
          textDocumentSync: 1, // Full text document sync
          completionProvider: {
            resolveProvider: true,
            triggerCharacters: ['.', ':', '<', '"', '/', '*', "'"],
          },
          hoverProvider: true,
          definitionProvider: true,
          referencesProvider: true,
          documentSymbolProvider: true,
          workspaceSymbolProvider: true,
          codeActionProvider: true,
          documentFormattingProvider: true,
          documentRangeFormattingProvider: true,
          renameProvider: true,
          executeCommandProvider: {
            commands: [],
          },
        },
        serverInfo: {
          name: 'rust-analyzer',
          version: '0.1.0',
        },
      };

      // Send the initialization response
      self.postMessage({
        jsonrpc: '2.0',
        id: message.id,
        result: result,
      });

      if (onSendMessage) {
        onSendMessage({ type: 'initialized' });
      }
    }

    // Forward other messages
    if (onSendMessage) {
      onSendMessage(message);
    }
  };

  // Set up the message handler
  self.onmessage = messageHandler;

  // Return a simple server interface
  return {
    onMessage: (handler: (message: any) => void) => {
      self.onmessage = (event: MessageEvent) => handler(event.data);
    },
    postMessage: (message: any) => {
      self.postMessage(message);
    },
    // Add any additional methods needed by the client
    listen: () => {
      // Start listening for messages
      self.onmessage = messageHandler;
    },
    // Clean up method
    dispose: () => {
      self.onmessage = null as any;
    },
  };
}
