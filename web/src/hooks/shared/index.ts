// Shared Hooks Library
// Consolidated hooks for state management and cross-cutting concerns

export { useAsync } from './useAsync';
export { useDataLoader } from './useDataLoader';
export { useFormManager } from './useFormManager';

// Re-export types
export type { UseAsyncOptions, UseAsyncReturn, AsyncState } from './useAsync';

export type { UseDataLoaderOptions, UseDataLoaderReturn } from './useDataLoader';

export type { FormConfig, FormState, FormManagerReturn, ValidationRule } from './useFormManager';
