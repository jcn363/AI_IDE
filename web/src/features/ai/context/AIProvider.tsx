import React, { ReactNode, createContext, useCallback, useContext, useEffect, useState } from 'react';
import { AIService } from '..';
import type { CodeGenerationOptions, GeneratedCode, AISuggestion } from '../types';
import type { ErrorResolutionResult, DocumentationLink } from '../error-resolution/ErrorResolver';

interface AIState {
  isInitialized: boolean;
  isLoading: boolean;
  suggestions: AISuggestion[];
  activeSuggestionIndex: number | null;
  isPanelVisible: boolean;
  error: string | null;
  documentation?: DocumentationLink[];
}

interface AIActions {
  analyzeCode: (code: string, filePath: string) => Promise<void>;
  resolveError: (error: any, documentText: string, filePath: string) => Promise<ErrorResolutionResult | null>;
  generateCode: (options: CodeGenerationOptions) => Promise<GeneratedCode | null>;
  applyFix: (action: any) => Promise<boolean>;
  dismissSuggestion: (index: number) => void;
  showPanel: () => void;
  hidePanel: () => void;
  clearSuggestions: () => void;
}

const AIContext = createContext<{ state: AIState; actions: AIActions } | undefined>(undefined);

export const useAI = () => {
  const context = useContext(AIContext);
  if (!context) {
    throw new Error('useAI must be used within an AIProvider');
  }
  return context;
};

interface AIProviderProps {
  children: ReactNode;
  service?: AIService;
}

export const AIProvider: React.FC<AIProviderProps> = ({ children, service = AIService.getInstance() }) => {
  const [state, setState] = useState<AIState>({
    isInitialized: false,
    isLoading: false,
    suggestions: [],
    activeSuggestionIndex: null,
    isPanelVisible: false,
    error: null,
  });

  // Initialize the AI service
  useEffect(() => {
    const init = async () => {
      try {
        setState(prev => ({ ...prev, isLoading: true }));
        await service.initialize();
        setState(prev => ({
          ...prev,
          isInitialized: true,
          isLoading: false,
        }));
      } catch (error) {
        console.error('Failed to initialize AI service:', error);
        setState(prev => ({
          ...prev,
          isLoading: false,
          error: 'Failed to initialize AI service',
        }));
      }
    };

    init();
  }, [service]);

  const analyzeCode = useCallback(async (code: string, filePath: string) => {
    try {
      setState(prev => ({ ...prev, isLoading: true }));
      
      // In a real implementation, this would call the actual AI service
      // For now, we'll simulate analysis with a timeout
      await new Promise(resolve => {setTimeout(resolve, 1000);});
      
      // Simulate analysis results
      const newSuggestion: AISuggestion = {
        type: 'suggestion',
        message: 'Consider extracting this complex logic into a separate function',
        range: { start: { line: 10, character: 0 }, end: { line: 15, character: 1 } },
        codeActions: [{
          title: 'Extract method',
          action: async () => {
            // Implementation would go here
          },
        }],
        category: 'performance',
        severity: 2,
        source: 'code-analysis',
      };
      
      setState(prev => ({
        ...prev,
        suggestions: [...prev.suggestions, newSuggestion],
        isPanelVisible: true,
        isLoading: false,
      }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      const errorSuggestion: AISuggestion = {
        type: 'error',
        message: errorMessage,
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
        codeActions: [
          {
            title: 'Fix automatically',
            action: async () => {
              // Implementation would go here
            },
          },
        ],
        category: 'security', // Using 'security' as the default category for errors
        severity: 1, // Error
        source: 'error-resolution',
      };
      
      console.error('Error analyzing code:', error);
      setState(prev => ({
        ...prev,
        suggestions: [...prev.suggestions, errorSuggestion],
        isPanelVisible: true,
        isLoading: false,
        error: 'Failed to analyze code',
      }));
    }
  }, []);

  const resolveError = useCallback(async (error: Error & { range?: any; text?: string }, documentText: string, filePath: string) => {
    try {
      setState(prev => ({ ...prev, isLoading: true }));
      
      // In a real implementation, this would call the actual error resolver
      // For now, we'll simulate resolution with a timeout
      await new Promise(resolve => {setTimeout(resolve, 800);});
      
      // Create error suggestion
      const errorSuggestion: AISuggestion = {
        type: 'error',
        message: error.message || 'Unknown error occurred',
        range: error.range || { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
        codeActions: [
          {
            title: 'Fix automatically',
            action: async () => {
              // Implementation would go here
              await Promise.resolve();
            },
          },
        ],
        category: 'security',
        severity: 1, // Error
        source: 'error-resolution',
      };
      
      // Simulate resolution results
      const mockResolution: ErrorResolutionResult = {
        quickFixes: [{
          title: 'Add missing semicolon',
          kind: 'quickfix',
          edit: {
            changes: {
              [filePath]: [{
                range: error.range || { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
                newText: `${error.text || ''  };`,
              }],
            },
          },
        }],
        explanations: ['This error occurs because of a missing semicolon at the end of the line.'],
        relatedDocumentation: [
          {
            title: 'Rust Semicolon Documentation',
            url: 'https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html',
            description: 'Learn about Rust syntax and semicolon usage',
          },
        ],
      };

      setState(prev => ({
        ...prev,
        suggestions: [...prev.suggestions, {
          type: 'error',
          message: error.message,
          range: error.range,
          codeActions: mockResolution.quickFixes.map(fix => ({
            title: fix.title || 'Fix automatically',
            action: async () => {
              // Implementation would go here
              console.log('Applying fix:', fix.title);
              await Promise.resolve();
            },
          })),
          category: 'syntax',
          severity: 1, // Error
          source: 'error-resolution',
        }],
        isPanelVisible: true,
        isLoading: false,
      }));

      return mockResolution;
    } catch (error) {
      console.error('Error resolving error:', error);
      setState(prev => ({
        ...prev,
        isLoading: false,
        error: 'Failed to resolve error',
      }));
      return null;
    }
  }, []);

  const generateCode = useCallback(async (options: CodeGenerationOptions) => {
    try {
      setState(prev => ({ ...prev, isLoading: true }));
      
      // In a real implementation, this would call the actual code generator
      // For now, we'll simulate generation with a timeout
      await new Promise(resolve => {setTimeout(resolve, 1500);});
      
      // Simulate generated code
      const mockGeneratedCode: GeneratedCode = {
        content: `// Generated code based on: ${  options.context?.substring(0, 50)  }...`,
        range: {
          start: { line: options.cursorPosition.line, character: 0 },
          end: { line: options.cursorPosition.line, character: 0 },
        },
        edits: [],
        confidence: 0.9,
        type: 'example',
      };

      setState(prev => ({
        ...prev,
        isLoading: false,
      }));

      return mockGeneratedCode;
    } catch (error) {
      console.error('Error generating code:', error);
      setState(prev => ({
        ...prev,
        isLoading: false,
        error: 'Failed to generate code',
      }));
      return null;
    }
  }, []);

  const applyFix = useCallback(async (action: any) => {
    try {
      // In a real implementation, this would apply the actual code fix
      console.log('Applying fix:', action);
      return true;
    } catch (error) {
      console.error('Error applying fix:', error);
      return false;
    }
  }, []);

  const dismissSuggestion = useCallback((index: number) => {
    setState(prev => ({
      ...prev,
      suggestions: prev.suggestions.filter((_, i) => i !== index),
    }));
  }, []);

  const showPanel = useCallback(() => {
    setState(prev => ({
      ...prev,
      isPanelVisible: true,
    }));
  }, []);

  const hidePanel = useCallback(() => {
    setState(prev => ({
      ...prev,
      isPanelVisible: false,
    }));
  }, []);

  const clearSuggestions = useCallback(() => {
    setState(prev => ({
      ...prev,
      suggestions: [],
      activeSuggestionIndex: null,
    }));
  }, []);

  const value = {
    state,
    actions: {
      analyzeCode,
      resolveError,
      generateCode,
      applyFix,
      dismissSuggestion,
      showPanel,
      hidePanel,
      clearSuggestions,
    },
  };

  return <AIContext.Provider value={value}>{children}</AIContext.Provider>;
};

export default AIProvider;
