import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import refactoringService from '../services/RefactoringService';
import type {
  AnalysisResult,
  RefactoringConfiguration,
  RefactoringContext,
  RefactoringOptions,
  RefactoringResult,
  RefactoringType,
} from '../../../types/refactoring';

interface UseRefactoringState {
  isAnalyzing: boolean;
  currentOperation: RefactoringType | null;
  analysisResult: AnalysisResult | null;
  error: string | null;
  progress: number;
  availableRefactorings: RefactoringType[];
  context: RefactoringContext | null;
  configuration: RefactoringConfiguration;
}

interface UseRefactoringActions {
  // Core operations
  executeRefactoring: (type: RefactoringType, context: RefactoringContext, options?: Partial<RefactoringOptions>) => Promise<RefactoringResult>;

  // Analysis operations
  analyzeContext: (context: RefactoringContext) => Promise<AnalysisResult>;
  getAvailableRefactorings: (context: RefactoringContext) => Promise<RefactoringType[]>;

  // Batch operations
  executeBatch: (operations: any[], config?: any) => Promise<any>;

  // Configuration
  updateConfiguration: (config: Partial<RefactoringConfiguration>) => void;
  reset: () => void;

  // State getters
  isReady: boolean;
  isProcessing: boolean;
}

export type UseRefactoringHook = UseRefactoringState & UseRefactoringActions;

const defaultConfiguration: RefactoringConfiguration = {
  enabledRefactorings: [
    'rename',
    'extract-function',
    'extract-variable',
    'extract-interface',
    'move-method',
    'inline-method',
    'introduce-parameter',
  ],
  defaultOptions: {
    createBackup: true,
    previewOnly: false,
    scope: 'file',
    includeTests: false,
    renameLinkedFiles: true,
    allowIncompleteRefs: false,
  },
  previewBeforeApply: true,
  confirmDestructiveChanges: true,
  maxPreviewLines: 50,
  excludePatterns: ['node_modules/**', 'target/**', 'build/**'],
};

export const useRefactoring = (initialConfig?: Partial<RefactoringConfiguration>): UseRefactoringHook => {
  const [state, setState] = useState<UseRefactoringState>({
    isAnalyzing: false,
    currentOperation: null,
    analysisResult: null,
    error: null,
    progress: 0,
    availableRefactorings: [],
    context: null,
    configuration: { ...defaultConfiguration, ...initialConfig },
  });

  // Core execution function - now uses RefactoringService
  const executeRefactoring = useCallback(async (
    type: RefactoringType,
    context: RefactoringContext,
    options?: Partial<RefactoringOptions>,
  ): Promise<RefactoringResult> => {
    setState(prev => ({
      ...prev,
      currentOperation: type,
      error: null,
    }));

    try {
      const mergedOptions = { ...state.configuration.defaultOptions, ...options };

      // Simulate progress during execution
      setState(prev => ({ ...prev, progress: 25 }));

      setState(prev => ({ ...prev, progress: 75 }));

      // Execute the refactoring using RefactoringService (which handles impact analysis and validation)
      const result = await refactoringService.executeRefactoring(type, context, mergedOptions);

      setState(prev => ({
        ...prev,
        progress: 100,
        currentOperation: null,
        error: result.success ? null : result.errorMessage || 'Unknown error occurred',
      }));

      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      setState(prev => ({
        ...prev,
        error: errorMessage,
        currentOperation: null,
        progress: 0,
      }));
      throw error;
    }
  }, [state.configuration]);

  // Analyze context for available refactorings using RefactoringService
  const analyzeContext = useCallback(async (context: RefactoringContext): Promise<AnalysisResult> => {
    setState(prev => ({ ...prev, isAnalyzing: true, context, error: null }));

    try {
      // Use RefactoringService instead of direct Tauri invoke - await the promise
      const analysis = await refactoringService.analyzeRefactoringContext(context);

      setState(prev => ({
        ...prev,
        isAnalyzing: false,
        analysisResult: analysis,
        availableRefactorings: (analysis.applicableRefactorings || []) as RefactoringType[],
      }));

      return analysis;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Analysis failed';
      setState(prev => ({
        ...prev,
        isAnalyzing: false,
        error: errorMessage,
        analysisResult: null,
      }));
      throw error;
    }
  }, []);

  // Get available refactorings for context using RefactoringService
  const getAvailableRefactorings = useCallback(async (context: RefactoringContext): Promise<RefactoringType[]> => {
    try {
      // Use RefactoringService instead of direct Tauri invoke
      const result = await refactoringService.getAvailableRefactorings(context);

      setState(prev => ({
        ...prev,
        availableRefactorings: result,
      }));

      return result;
    } catch (error) {
      console.error('Failed to get available refactorings:', error);
      // Return default enabled refactorings on error
      return state.configuration.enabledRefactorings;
    }
  }, [state.configuration]);

  // Execute batch of refactoring operations
  const executeBatch = useCallback(async (operations: any[], batchConfig?: any): Promise<any> => {
    try {
      const result = await refactoringService.executeBatchRefactoring(operations);

      // Generate tests if requested - this could be moved to service
      if (operations.some(op => op.options?.generateTests)) {
        // Note: test generation should be handled by the service, not the hook
        console.warn('Test generation should be implemented in RefactoringService');
      }

      return result;
    } catch (error) {
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Batch execution failed',
      }));
      throw error;
    }
  }, []);

  // Update configuration
  const updateConfiguration = useCallback((config: Partial<RefactoringConfiguration>) => {
    setState(prev => ({
      ...prev,
      configuration: { ...prev.configuration, ...config },
    }));
  }, []);

  // Reset state
  const reset = useCallback(() => {
    setState(prev => ({
      ...prev,
      currentOperation: null,
      analysisResult: null,
      error: null,
      progress: 0,
      availableRefactorings: [],
      context: null,
    }));
  }, []);

  // Derived state
  const isReady = !state.isAnalyzing && !state.currentOperation;
  const isProcessing = state.isAnalyzing || Boolean(state.currentOperation);

  // Auto-clear error after 5 seconds
  useEffect(() => {
    if (state.error) {
      const timer = setTimeout(() => {
        setState(prev => ({ ...prev, error: null }));
      }, 5000);

      return () => clearTimeout(timer);
    }
  }, [state.error]);

  return {
    // State
    ...state,
    isReady,
    isProcessing,

    // Actions
    executeRefactoring,
    analyzeContext,
    getAvailableRefactorings,
    executeBatch,
    updateConfiguration,
    reset,
  };
};

export default useRefactoring;