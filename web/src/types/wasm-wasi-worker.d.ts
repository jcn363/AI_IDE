declare module '@vscode/wasm-wasi/worker' {
    import { WASI } from '@vscode/wasm-wasi';
    
    export function initializeWasm(): Promise<{
        wasi: WASI;
        // Add other exports as needed
    }>;
}
