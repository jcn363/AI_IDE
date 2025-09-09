// Type definitions for Electron API

declare global {
  interface Window {
    electron: {
      // File system operations
      invoke<T = any>(channel: string, ...args: any[]): Promise<T>;
      
      // Cargo commands
      cargo: {
        execute: (command: string, args: string[], cwd: string) => Promise<{ output: string; error?: string }>;
        checkAvailable: () => Promise<{ available: boolean; version?: string }>;
        getMetadata: (projectPath: string) => Promise<{
          name: string;
          version: string;
          dependencies: Record<string, string>;
          targetDirectory: string;
        }>;
      };

      // File system operations
      fs: {
        readFile: (options: { path: string; encoding: BufferEncoding }) => Promise<string>;
        writeFile: (options: { path: string; content: string; encoding: BufferEncoding }) => Promise<void>;
        readDir: (path: string) => Promise<Array<{ name: string; isDirectory: boolean }>>;
        exists: (path: string) => Promise<boolean>;
      };

      // Process handling
      process: {
        cwd: () => string;
        platform: NodeJS.Platform;
        env: NodeJS.ProcessEnv;
      };

      // Window management
      window: {
        minimize: () => void;
        maximize: () => void;
        close: () => void;
        isMaximized: () => Promise<boolean>;
      };

      // IPC event listeners
      on: (channel: string, listener: (...args: any[]) => void) => void;
      removeListener: (channel: string, listener: (...args: any[]) => void) => void;
    };
  }
}

export {};
