// client.ts
import { MonacoLanguageClient } from '@codingame/monaco-languageclient';
import { BrowserMessageReader, BrowserMessageWriter } from 'vscode-languageserver-protocol/browser';
import { CloseAction, ErrorAction, LanguageClientOptions, Message } from 'vscode-languageclient';
import { createMessageConnection } from 'vscode-languageserver/browser';
import { WorkspaceFolder as LSPWorkspaceFolder } from 'vscode-languageserver-protocol';
import * as monaco from 'monaco-editor';

// Declare the Monaco environment
interface Window {
  MonacoEnvironment?: {
    getWorker(moduleId: string, label: string): Worker;
  };
}

declare const window: Window;

interface LanguageServerOptions {
  rootUri?: string;
  workspaceFolders?: LSPWorkspaceFolder[];
  initializationOptions?: any;
}

export class LanguageServerClient {
  private client: MonacoLanguageClient | null = null;
  private worker: Worker | null = null;
  private isClientRunning = false;
  private disposables: monaco.IDisposable[] = [];
  private options: LanguageServerOptions;

  constructor(options: LanguageServerOptions = {}) {
    this.options = {
      ...options,
      workspaceFolders: options.workspaceFolders || (options.rootUri ? [{
        uri: options.rootUri,
        name: 'workspace'
      }] : [])
    };
  }

  async start(): Promise<void> {
    if (this.isClientRunning) {
      console.log('Language server client already started');
      return;
    }

    // Ensure Monaco environment is properly set up
    if (typeof window === 'undefined') {
      throw new Error('LanguageServerClient must run in a browser environment');
    }

    try {
      // Register Monaco language features
      monaco.languages.register({ id: 'rust', extensions: ['.rs'] });

      // Create a web worker that will run the language server
      // Avoid import.meta for compatibility with CommonJS builds
      const workerUrl = new URL('./server.worker.ts', (globalThis as any).location?.href || 'http://localhost/');
      this.worker = new Worker(workerUrl, {
        type: 'module',
        name: 'rust-analyzer-worker'
      });
      const clientOptions: LanguageClientOptions = {
        documentSelector: ['rust'],
        errorHandler: {
          error: () => ErrorAction.Continue,
          closed: () => CloseAction.DoNotRestart
        },
        outputChannel: {
          name: 'Rust Language Server',
          append: (value: string) => console.log(value),
          appendLine: (value: string) => console.log(value),
          replace: (value: string) => console.log(value),
          clear: () => {},
          show: () => {},
          hide: () => {},
          dispose: () => {}
        } as any,
        workspaceFolder: this.options.workspaceFolders?.[0] ? {
          uri: monaco.Uri.parse(this.options.workspaceFolders[0].uri),
          name: this.options.workspaceFolders[0].name,
          index: 0
        } : undefined,
        initializationOptions: this.options.initializationOptions || {
          "rust-analyzer": {
            checkOnSave: true,
            cargo: { allFeatures: true },
            diagnostics: { enable: true }
          }
        }
      };

      // Create message reader and writer
      const reader = new BrowserMessageReader(this.worker);
      const writer = new BrowserMessageWriter(this.worker);

      // Create the language client
      this.client = new MonacoLanguageClient({
        name: 'Rust Language Client',
        clientOptions,
        connectionProvider: {
          get: (errorHandler, closeHandler) => {
            const connection = createMessageConnection(reader, writer, {
              error: (message: string) => console.error(message),
              warn: (message: string) => console.warn(message),
              info: (message: string) => console.log(message),
              log: (message: string) => console.log(message)
            }, undefined);
            
            // Set up error handling
            connection.onError(([error, message, code]) => {
              console.error('Connection error:', error);
              // Create a proper error object for the error handler
              const errorMessage = typeof error === 'string' ? error : error.message;
              const errorObj = new Error(errorMessage);
              // Add code property if it exists
              if (code) {
                (errorObj as any).code = code;
              }
              // Create a proper Message object for the second parameter
              const messageObj: Message = {
                jsonrpc: '2.0',
                error: {
                  code: code || 0,
                  message: errorMessage,
                  data: error
                }
              } as unknown as Message;
              // Call the error handler with the correct types
              const action = clientOptions.errorHandler?.error(errorObj, messageObj, code || 0);
              if (action === ErrorAction.Shutdown) {
                this.stop();
              }
            });
            
            // Set up close handling
            connection.onClose(() => {
              console.log('Connection closed');
              closeHandler();
            });
            
            // Start listening
            connection.listen();
            
            // Return the connection wrapped in a Promise that resolves to IConnection
            return Promise.resolve({
              ...connection,
              dispose: () => {
                connection.dispose();
                reader.dispose();
                writer.dispose();
              }
            } as any); // Cast to any to satisfy TypeScript
          }
        }
      });

      // Start the client
      await this.client.start();
      this.isClientRunning = true;
      console.log('Language server client is ready');
    } catch (error) {
      console.error('Failed to start language server client:', error);
      await this.stop().catch(() => {});
      throw error;
    }
  }

  async stop(): Promise<void> {
    if (!this.isClientRunning) {
      return;
    }

    this.isClientRunning = false;

    // Stop the client if it exists
    if (this.client) {
      try {
        await this.client.stop();
      } catch (error) {
        console.error('Error stopping language client:', error);
      } finally {
        this.client = null;
      }
    }

    // Terminate the worker if it exists
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }

    // Dispose all registered disposables
    const disposables = [...this.disposables];
    this.disposables = [];

    for (const disposable of disposables) {
      try {
        if (disposable && typeof disposable.dispose === 'function') {
          await Promise.resolve(disposable.dispose());
        }
      } catch (error) {
        console.error('Error disposing resource:', error);
      }
    }
  }

  isRunning(): boolean {
    return this.client !== null && this.isClientRunning;
  }

  getClient(): MonacoLanguageClient | null {
    return this.client;
  }

  async sendNotification(method: string, params?: unknown): Promise<void> {
    if (!this.client || !this.isClientRunning) {
      console.warn('Cannot send notification: Language server client not started');
      return;
    }
    try {
      await this.client.sendNotification(method, params);
    } catch (error) {
      console.error(`Failed to send notification ${method}:`, error);
      throw error;
    }
  }

  async sendRequest<T>(method: string, params?: unknown): Promise<T> {
    if (!this.client || !this.isClientRunning) {
      throw new Error('Language server client not started');
    }
    try {
      return await this.client.sendRequest<T>(method, params);
    } catch (error) {
      console.error(`Failed to send request ${method}:`, error);
      throw error;
    }
  }
}

export const languageServerClient = new LanguageServerClient();