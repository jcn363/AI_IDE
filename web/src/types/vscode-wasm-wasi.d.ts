declare module '@vscode/wasm-wasi/worker' {
    import { WASI } from '@vscode/wasm-wasi';
    
    export interface WasmWasiWorker {
        initializeWasm(): Promise<{
            wasi: WASI;
            // Add other exports as needed
        }>;
    }
    
    const worker: WasmWasiWorker;
    export default worker;
}
