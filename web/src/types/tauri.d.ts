// Tauri core types
declare module '@tauri-apps/api/core' {
  export function invoke<T = any>(cmd: string, args?: Record<string, any>): Promise<T>;
}

// Tauri event types
declare module '@tauri-apps/api/event' {
  export type UnlistenFn = () => void;
  export function listen<T = any>(
    event: string,
    handler: (event: { event: string; id: number; payload: T }) => void
  ): Promise<UnlistenFn>;
}

// Extend Window interface for Tauri
declare global {
  interface Window {
    __TAURI__?: {
      tauri: {
        invoke: <T = any>(cmd: string, args?: Record<string, any>) => Promise<T>;
      };
    };
  }
}
