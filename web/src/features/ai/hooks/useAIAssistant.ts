import { createScopedLogger } from '@/utils/logging';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useRef, useState } from 'react';
import {
  AIAnalysisConfigExtended,
  AIService,
  AnalysisProgress,
  CodeAnalysisResult,
  CodeAnalyzer,
  CodeChange,
  CodeSuggestion,
} from '../code-analysis/CodeAnalyzer';
import { useAI } from '../context/AIProvider';
import type {
  AnalysisConfiguration,
  ArchitectureSuggestion,
  CodeGenerationOptions,
  CodeSmell,
  CompilerDiagnostic,
  CompilerIntegrationResult,
  EnhancedCodeAnalysisResult,
  EnhancedErrorResolutionResult,
  ErrorCodeExplanation,
  ErrorPattern,
  FixApplicationResult,
  FixSuggestion,
  LearnedPattern,
  LearningPreferences,
  LearningSystemData,
  PerformanceHint,
  SecurityIssue,
  StyleViolation,
} from '../types';

// Create a scoped logger for the AI Assistant
const log = createScopedLogger('ai:assistant');

interface AnalysisState {
  isAnalyzing: boolean;
  analysisResults: Map<string, CodeAnalysisResult>;
  enhancedAnalysisResults: Map<string, EnhancedCodeAnalysisResult>;
  workspaceAnalysisId: string | null;
  workspaceProgress: AnalysisProgress | null;
  lastAnalysisTime: Map<string, number>;
  analysisErrors: Map<string, string>;
  compilerDiagnostics: Map<string, CompilerDiagnostic[]>;
  learnedPatterns: LearnedPattern[];
  learningSystemData: LearningSystemData | null;
}

interface StyleAnalysisOptions {
  filePath?: string;
  workspacePath?: string;
  checkNamingConventions?: boolean;
  checkFormattingConsistency?: boolean;
  checkRustIdioms?: boolean;
  checkDocumentation?: boolean;
}

interface ArchitectureAnalysisOptions {
  workspacePath: string;
  checkCircularDependencies?: boolean;
  analyzeCoupling?: boolean;
  detectAntiPatterns?: boolean;
  includeMetrics?: boolean;
}

interface LearningSystemOptions {
  enableLearning?: boolean;
  recordSuccessfulFixes?: boolean;
  useLearnedPatterns?: boolean;
  confidenceThreshold?: number;
}

interface CompilerIntegrationOptions {
  parseCargoCheck?: boolean;
  enableErrorExplanations?: boolean;
  cacheExplanations?: boolean;
  includeDocumentationLinks?: boolean;
}

interface PerformanceSuggestionsOptions {
  filePath?: string;
  workspacePath?: string;
}

interface CodeQualityCheckOptions {
  runClippy?: boolean;
  runRustfmt?: boolean;
  runAIAnalysis?: boolean;
}

export const useAIAssistant = () => {
  const { state, actions } = useAI();
  const [isGenerating, setIsGenerating] = useState(false);
  const [generationError, setGenerationError] = useState<string | null>(null);
  const [generatedContent, setGeneratedContent] = useState<string | null>(null);

  // Enhanced analysis state
  const [analysisState, setAnalysisState] = useState<AnalysisState>({
    isAnalyzing: false,
    analysisResults: new Map(),
    enhancedAnalysisResults: new Map(),
    workspaceAnalysisId: null,
    workspaceProgress: null,
    lastAnalysisTime: new Map(),
    analysisErrors: new Map(),
    compilerDiagnostics: new Map(),
    learnedPatterns: [],
    learningSystemData: null,
  });

  // AI service and analyzer instances
  const aiServiceRef = useRef<AIService | null>(null);
  const codeAnalyzerRef = useRef<CodeAnalyzer | null>(null);
  const debounceTimersRef = useRef<Map<string, NodeJS.Timeout>>(new Map());
  const abortControllersRef = useRef<Map<string, AbortController>>(new Map());

  // Configuration state
  const [config, setConfig] = useState<any | null>(null);
  const [enhancedConfig, setEnhancedConfig] = useState<AnalysisConfiguration | null>(null);
  const [learningPreferences, setLearningPreferences] = useState<LearningPreferences | null>(null);

  // Initialize AI services
  useEffect(() => {
    let isMounted = true;

    const initializeServices = async () => {
      try {
        if (!aiServiceRef.current) {
          aiServiceRef.current = AIService.getInstance();
          await aiServiceRef.current.initialize();
        }

        if (!codeAnalyzerRef.current) {
          codeAnalyzerRef.current = CodeAnalyzer.getInstance();
          await codeAnalyzerRef.current.initialize();
        }

        // Load configuration
        const aiConfig = (await invoke('get_ai_config')) as AnalysisConfiguration;
        if (isMounted) setConfig(aiConfig);

        // Load enhanced configuration
        try {
          const enhancedConfig = (await invoke('get_enhanced_ai_config')) as AnalysisConfiguration;
          if (isMounted) setEnhancedConfig(enhancedConfig);
        } catch (error) {
          console.warn('Enhanced config not available, using defaults');
        }

        // Load learning preferences
        try {
          const learningPrefs = (await invoke('get_learning_preferences')) as LearningPreferences;
          if (isMounted) setLearningPreferences(learningPrefs);
        } catch (error) {
          console.warn('Learning preferences not available, using defaults');
        }

        // Load existing learned patterns
        try {
          const learningData = (await invoke('get_learning_system_data')) as LearningSystemData;
          if (isMounted) {
            setAnalysisState((prev) => ({
              ...prev,
              learnedPatterns: learningData.learnedPatterns,
              learningSystemData: learningData,
            }));
          }
        } catch (error) {
          console.warn('Learning system data not available');
        }
      } catch (error) {
        log.error('Failed to initialize AI services', error instanceof Error ? error : undefined, {
          operation: 'initializeServices',
        });
        if (isMounted) {
          const errorMessage =
            error instanceof Error
              ? `Failed to initialize AI services: ${error.message}`
              : 'Failed to initialize AI services';
          setGenerationError(errorMessage);
        }
      }
    };

    initializeServices();

    // Cleanup on unmount
    return () => {
      isMounted = false;
      // Cancel all pending operations
      abortControllersRef.current.forEach((controller) => controller.abort());
      debounceTimersRef.current.forEach((timer) => clearTimeout(timer));
    };
  }, []);

  // Workspace analysis progress polling
  useEffect(() => {
    let intervalId: NodeJS.Timeout | null = null;

    if (analysisState.workspaceAnalysisId) {
      intervalId = setInterval(async () => {
        try {
          const progress = (await invoke('get_analysis_progress', {
            analysis_id: analysisState.workspaceAnalysisId,
          })) as AnalysisProgress | null;

          if (progress) {
            setAnalysisState((prev) => ({
              ...prev,
              workspaceProgress: progress,
            }));

            // Stop polling if analysis is complete or failed
            if (
              progress.status === 'Completed' ||
              progress.status === 'Failed' ||
              progress.status === 'Cancelled'
            ) {
              setAnalysisState((prev) => ({
                ...prev,
                workspaceAnalysisId: null,
                isAnalyzing: false,
              }));
            }
          }
        } catch (error) {
          console.error('Failed to get analysis progress:', error);
        }
      }, 1000); // Poll every second
    }

    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [analysisState.workspaceAnalysisId]);

  const analyzeCurrentFile = useCallback(
    async (
      code: string,
      filePath: string,
      options: {
        useCache?: boolean;
        debounceMs?: number;
        realTime?: boolean;
      } = {}
    ) => {
      const { useCache = true, debounceMs = 300, realTime = false } = options;

      try {
        setGenerationError(null);

        // Clear any existing error for this file
        setAnalysisState((prev) => {
          const newErrors = new Map(prev.analysisErrors);
          newErrors.delete(filePath);
          return { ...prev, analysisErrors: newErrors };
        });

        // Check if we should debounce this request
        const now = Date.now();
        const lastAnalysis = analysisState.lastAnalysisTime.get(filePath) || 0;
        const timeSinceLastAnalysis = now - lastAnalysis;

        // Skip analysis if it's too recent (for real-time analysis)
        if (realTime && timeSinceLastAnalysis < 1000) {
          return;
        }

        // Cancel any existing analysis for this file
        const existingController = abortControllersRef.current.get(filePath);
        if (existingController) {
          existingController.abort();
        }

        // Create new abort controller
        const controller = new AbortController();
        abortControllersRef.current.set(filePath, controller);

        setAnalysisState((prev) => ({ ...prev, isAnalyzing: true }));

        // Use CodeAnalyzer for enhanced analysis
        if (!codeAnalyzerRef.current) {
          throw new Error('Code analyzer not initialized');
        }

        const result = await codeAnalyzerRef.current.analyzeCode(code, filePath, {
          useCache,
          debounceMs: realTime ? debounceMs : 0,
          signal: controller.signal,
        });

        // Update analysis results
        setAnalysisState((prev) => {
          const newResults = new Map(prev.analysisResults);
          newResults.set(filePath, result);
          const newLastAnalysisTime = new Map(prev.lastAnalysisTime);
          newLastAnalysisTime.set(filePath, now);

          return {
            ...prev,
            analysisResults: newResults,
            lastAnalysisTime: newLastAnalysisTime,
          };
        });

        // Update AI context with new suggestions
        if (result.suggestions.length > 0) {
          await actions.analyzeCode(code, filePath);
          actions.showPanel();
        }

        // Clean up controller
        abortControllersRef.current.delete(filePath);
      } catch (error) {
        log.error('Error analyzing file', error instanceof Error ? error : undefined, {
          filePath,
          operation: 'analyzeCurrentFile',
        });

        const errorMessage =
          error instanceof Error
            ? `Failed to analyze file ${filePath}: ${error.message}`
            : `Failed to analyze file ${filePath}`;
        setGenerationError(errorMessage);

        setAnalysisState((prev) => {
          const newErrors = new Map(prev.analysisErrors);
          newErrors.set(filePath, errorMessage);
          return {
            ...prev,
            analysisErrors: newErrors,
          };
        });

        // Clean up controller
        abortControllersRef.current.delete(filePath);
      } finally {
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: false }));
      }
    },
    [actions, analysisState.lastAnalysisTime, codeAnalyzerRef]
  );

  // New method: Analyze entire workspace
  const analyzeWorkspace = useCallback(
    async (
      workspacePath: string,
      options: {
        includeDependencies?: boolean;
        includeSecurityScan?: boolean;
      } = {}
    ) => {
      try {
        setGenerationError(null);
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: true }));

        if (!aiServiceRef.current) {
          throw new Error('AI service not initialized');
        }

        const analysisId = await aiServiceRef.current.analyzeWorkspace(workspacePath, options);

        setAnalysisState((prev) => ({
          ...prev,
          workspaceAnalysisId: analysisId,
        }));

        return analysisId;
      } catch (error) {
        log.error('Error analyzing workspace', error instanceof Error ? error : undefined, {
          workspacePath,
          operation: 'analyzeWorkspace',
        });
        const errorMessage =
          error instanceof Error
            ? `Failed to analyze workspace ${workspacePath}: ${error.message}`
            : `Failed to analyze workspace ${workspacePath}`;
        setGenerationError(errorMessage);
        throw error;
      } finally {
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: false }));
      }
    },
    [aiServiceRef]
  );

  // New method: Analyze code style
  const analyzeCodeStyle = useCallback(
    async (
      code: string,
      filePath: string,
      options: StyleAnalysisOptions = {}
    ): Promise<StyleViolation[]> => {
      try {
        setGenerationError(null);

        const result = (await invoke('analyze_code_style', {
          code,
          filePath,
          options: {
            checkNamingConventions: options.checkNamingConventions ?? true,
            checkFormattingConsistency: options.checkFormattingConsistency ?? true,
            checkRustIdioms: options.checkRustIdioms ?? true,
            checkDocumentation: options.checkDocumentation ?? true,
          },
        })) as StyleViolation[];

        return result;
      } catch (error) {
        console.error('Error analyzing code style:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to analyze code style';
        setGenerationError(errorMessage);
        return [];
      }
    },
    []
  );

  // New method: Analyze architecture
  const analyzeArchitecture = useCallback(
    async (options: ArchitectureAnalysisOptions): Promise<ArchitectureSuggestion[]> => {
      try {
        setGenerationError(null);
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: true }));

        const result = (await invoke('analyze_architecture', {
          workspacePath: options.workspacePath,
          options: {
            checkCircularDependencies: options.checkCircularDependencies ?? true,
            analyzeCoupling: options.analyzeCoupling ?? true,
            detectAntiPatterns: options.detectAntiPatterns ?? true,
            includeMetrics: options.includeMetrics ?? true,
          },
        })) as ArchitectureSuggestion[];

        return result;
      } catch (error) {
        log.error('Error analyzing architecture', error instanceof Error ? error : undefined, {
          operation: 'analyzeArchitecture',
          options: options,
        });
        const errorMessage =
          error instanceof Error
            ? `Failed to analyze architecture: ${error.message}`
            : 'Failed to analyze architecture';
        setGenerationError(errorMessage);
        return [];
      } finally {
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: false }));
      }
    },
    []
  );

  // New method: Get enhanced performance hints
  const getEnhancedPerformanceHints = useCallback(
    async (
      code: string,
      filePath: string,
      options: PerformanceSuggestionsOptions = {}
    ): Promise<PerformanceHint[]> => {
      try {
        setGenerationError(null);

        const result = (await invoke('get_enhanced_performance_hints', {
          code,
          filePath,
          workspacePath: options.workspacePath,
        })) as PerformanceHint[];

        return result;
      } catch (error) {
        console.error('Error getting enhanced performance hints:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to get performance hints';
        setGenerationError(errorMessage);
        return [];
      }
    },
    []
  );

  // New method: Record successful fix for learning
  const recordSuccessfulFix = useCallback(
    async (
      errorPattern: ErrorPattern,
      appliedFix: FixSuggestion,
      success: boolean,
      userFeedback?: 'positive' | 'negative' | 'neutral',
      context?: string
    ): Promise<void> => {
      try {
        if (!learningPreferences?.enableLearning) {
          return; // Learning disabled
        }

        await invoke('record_successful_fix', {
          errorPattern,
          appliedFix,
          success,
          userFeedback,
          context: context || '',
        });

        // Refresh learned patterns
        const learningData = (await invoke('get_learning_system_data')) as LearningSystemData;
        setAnalysisState((prev) => ({
          ...prev,
          learnedPatterns: learningData.learnedPatterns,
          learningSystemData: learningData,
        }));
      } catch (error) {
        console.error('Error recording successful fix:', error);
      }
    },
    [learningPreferences]
  );

  // New method: Get learned patterns for error
  const getLearnedPatterns = useCallback(
    async (errorType: string, context?: string): Promise<LearnedPattern[]> => {
      try {
        const patterns = (await invoke('get_learned_patterns', {
          errorType,
          context: context || '',
        })) as LearnedPattern[];

        return patterns;
      } catch (error) {
        console.error('Error getting learned patterns:', error);
        return [];
      }
    },
    []
  );

  // New method: Get compiler diagnostics
  const getCompilerDiagnostics = useCallback(
    async (
      filePath?: string,
      options: CompilerIntegrationOptions = {}
    ): Promise<CompilerDiagnostic[]> => {
      try {
        setGenerationError(null);

        const result = (await invoke('get_compiler_diagnostics', {
          filePath,
          options: {
            parseCargoCheck: options.parseCargoCheck ?? true,
            enableErrorExplanations: options.enableErrorExplanations ?? true,
            cacheExplanations: options.cacheExplanations ?? true,
            includeDocumentationLinks: options.includeDocumentationLinks ?? true,
          },
        })) as CompilerDiagnostic[];

        // Update state with diagnostics
        if (filePath) {
          setAnalysisState((prev) => {
            const newDiagnostics = new Map(prev.compilerDiagnostics);
            newDiagnostics.set(filePath, result);
            return { ...prev, compilerDiagnostics: newDiagnostics };
          });
        }

        return result;
      } catch (error) {
        console.error('Error getting compiler diagnostics:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to get compiler diagnostics';
        setGenerationError(errorMessage);
        return [];
      }
    },
    []
  );

  // New method: Explain error code
  const explainErrorCode = useCallback(
    async (errorCode: string): Promise<ErrorCodeExplanation | null> => {
      try {
        setGenerationError(null);

        const explanation = (await invoke('explain_error_code', {
          errorCode,
        })) as ErrorCodeExplanation;

        return explanation;
      } catch (error) {
        console.error('Error explaining error code:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to explain error code';
        setGenerationError(errorMessage);
        return null;
      }
    },
    []
  );

  // New method: Enhanced error resolution with learning
  const resolveErrorWithLearning = useCallback(
    async (
      errorMessage: string,
      filePath: string,
      context: string
    ): Promise<EnhancedErrorResolutionResult> => {
      try {
        setGenerationError(null);

        const result = (await invoke('resolve_error_with_learning', {
          errorMessage,
          filePath,
          context,
          useLearning: learningPreferences?.enableLearning ?? false,
        })) as EnhancedErrorResolutionResult;

        return result;
      } catch (error) {
        console.error('Error resolving error with learning:', error);
        const errorMessage = error instanceof Error ? error.message : 'Failed to resolve error';
        setGenerationError(errorMessage);
        return {
          quickFixes: [],
          explanations: [],
          relatedDocumentation: [],
          learnedPatterns: [],
          compilerDiagnostics: [],
          confidence: 0,
          estimatedSuccessRate: 0,
        };
      }
    },
    [learningPreferences]
  );

  // New method: Apply fix with learning feedback
  const applyFixWithLearning = useCallback(
    async (
      suggestionId: string,
      changes: CodeChange[],
      createBackup = true,
      recordForLearning = true
    ): Promise<FixApplicationResult> => {
      try {
        setGenerationError(null);

        const result = (await invoke('apply_fix_with_learning', {
          suggestionId,
          changes,
          createBackup,
          recordForLearning: recordForLearning && (learningPreferences?.enableLearning ?? false),
        })) as FixApplicationResult;

        // Refresh analysis for affected files
        const affectedFiles = [
          ...new Set(changes.map((change) => change.file_path ?? (change as any).filePath)),
        ].filter(Boolean);
        for (const filePath of affectedFiles) {
          // Clear cache for this file to force re-analysis
          if (codeAnalyzerRef.current) {
            await codeAnalyzerRef.current.cancelAnalysis(filePath);
          }

          // Remove from analysis results to trigger re-analysis
          setAnalysisState((prev) => {
            const newResults = new Map(prev.analysisResults);
            const newEnhancedResults = new Map(prev.enhancedAnalysisResults);
            newResults.delete(filePath);
            newEnhancedResults.delete(filePath);
            return {
              ...prev,
              analysisResults: newResults,
              enhancedAnalysisResults: newEnhancedResults,
            };
          });
        }

        // Update learned patterns if learning was enabled
        if (recordForLearning && learningPreferences?.enableLearning && result.learnedPatternId) {
          const learningData = (await invoke('get_learning_system_data')) as LearningSystemData;
          setAnalysisState((prev) => ({
            ...prev,
            learnedPatterns: learningData.learnedPatterns,
            learningSystemData: learningData,
          }));
        }

        return result;
      } catch (error) {
        console.error('Error applying fix with learning:', error);
        const errorMessage = error instanceof Error ? error.message : 'Failed to apply fix';
        setGenerationError(errorMessage);
        return {
          success: false,
          appliedChanges: [],
          errors: [errorMessage],
        };
      }
    },
    [learningPreferences, codeAnalyzerRef]
  );

  // New method: Update learning preferences
  const updateLearningPreferences = useCallback(
    async (newPreferences: Partial<LearningPreferences>): Promise<void> => {
      try {
        await invoke('update_learning_preferences', { preferences: newPreferences });

        setLearningPreferences((prev) =>
          prev
            ? { ...prev, ...newPreferences }
            : {
                enableLearning: false,
                privacyMode: 'opt-out',
                shareAnonymousData: false,
                retainPersonalData: false,
                dataRetentionDays: 30,
                allowModelTraining: false,
                ...newPreferences,
              }
        );
      } catch (error) {
        console.error('Error updating learning preferences:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to update learning preferences';
        setGenerationError(errorMessage);
        throw error;
      }
    },
    []
  );

  // Enhanced method: Run comprehensive enhanced analysis
  const runEnhancedAnalysis = useCallback(
    async (
      code: string,
      filePath: string,
      options: {
        includeStyle?: boolean;
        includeArchitecture?: boolean;
        includePerformance?: boolean;
        includeSecurity?: boolean;
        includeCodeSmells?: boolean;
        useCompilerDiagnostics?: boolean;
        useLearning?: boolean;
      } = {}
    ): Promise<EnhancedCodeAnalysisResult> => {
      try {
        setGenerationError(null);
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: true }));

        const result = (await invoke('run_enhanced_analysis', {
          code,
          filePath,
          options: {
            includeStyle: options.includeStyle ?? true,
            includeArchitecture: options.includeArchitecture ?? true,
            includePerformance: options.includePerformance ?? true,
            includeSecurity: options.includeSecurity ?? true,
            includeCodeSmells: options.includeCodeSmells ?? true,
            useCompilerDiagnostics: options.useCompilerDiagnostics ?? true,
            useLearning: options.useLearning ?? learningPreferences?.enableLearning ?? false,
          },
        })) as EnhancedCodeAnalysisResult;

        // Update enhanced analysis results
        setAnalysisState((prev) => {
          const newResults = new Map(prev.enhancedAnalysisResults);
          newResults.set(filePath, result);
          return {
            ...prev,
            enhancedAnalysisResults: newResults,
          };
        });

        return result;
      } catch (error) {
        log.error('Error running enhanced analysis', error instanceof Error ? error : undefined, {
          filePath,
          operation: 'runEnhancedAnalysis',
          options,
        });
        const errorMessage =
          error instanceof Error
            ? `Failed to run enhanced analysis for ${filePath}: ${error.message}`
            : `Failed to run enhanced analysis for ${filePath}`;
        setGenerationError(errorMessage);
        throw error;
      } finally {
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: false }));
      }
    },
    [learningPreferences]
  );

  // New method: Get performance suggestions
  const getPerformanceSuggestions = useCallback(
    async (options: PerformanceSuggestionsOptions = {}): Promise<CodeSuggestion[]> => {
      try {
        setGenerationError(null);

        if (!aiServiceRef.current) {
          throw new Error('AI service not initialized');
        }

        const suggestions = await aiServiceRef.current.getPerformanceSuggestions(
          options.filePath,
          options.workspacePath
        );

        return suggestions;
      } catch (error) {
        console.error('Error getting performance suggestions:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to get performance suggestions';
        setGenerationError(errorMessage);
        return [];
      }
    },
    [aiServiceRef]
  );

  // New method: Run comprehensive code quality check
  const runCodeQualityCheck = useCallback(
    async (targetPath: string, options: CodeQualityCheckOptions = {}) => {
      try {
        setGenerationError(null);
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: true }));

        if (!aiServiceRef.current) {
          throw new Error('AI service not initialized');
        }

        const result = await aiServiceRef.current.runCodeQualityCheck(targetPath, options);

        setAnalysisState((prev) => ({ ...prev, isAnalyzing: false }));

        return result;
      } catch (error) {
        console.error('Error running code quality check:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to run code quality check';
        setGenerationError(errorMessage);
        setAnalysisState((prev) => ({ ...prev, isAnalyzing: false }));
        throw error;
      }
    },
    [aiServiceRef]
  );

  // New method: Apply AI suggestion
  const applySuggestion = useCallback(
    async (suggestionId: string, changes: CodeChange[], createBackup = true): Promise<string> => {
      try {
        setGenerationError(null);

        if (!aiServiceRef.current) {
          throw new Error('AI service not initialized');
        }

        const result = await aiServiceRef.current.applySuggestion(
          suggestionId,
          changes,
          createBackup
        );

        // Refresh analysis for affected files
        const affectedFiles = [...new Set(changes.map((change) => change.file_path))];
        for (const filePath of affectedFiles) {
          // Clear cache for this file to force re-analysis
          if (codeAnalyzerRef.current) {
            await codeAnalyzerRef.current.cancelAnalysis(filePath);
          }

          // Remove from analysis results to trigger re-analysis
          setAnalysisState((prev) => {
            const newResults = new Map(prev.analysisResults);
            newResults.delete(filePath);
            return { ...prev, analysisResults: newResults };
          });
        }

        return result;
      } catch (error) {
        console.error('Error applying suggestion:', error);
        const errorMessage = error instanceof Error ? error.message : 'Failed to apply suggestion';
        setGenerationError(errorMessage);
        throw error;
      }
    },
    [aiServiceRef, codeAnalyzerRef]
  );

  const generateCode = useCallback(
    async (options: CodeGenerationOptions) => {
      try {
        setIsGenerating(true);
        setGenerationError(null);

        // Get analysis context for enhanced generation
        const analysisResult = options.filePath
          ? analysisState.analysisResults.get(options.filePath)
          : undefined;

        // Use AI service for context-aware generation
        if (aiServiceRef.current && analysisResult) {
          const enhancedResult = await aiServiceRef.current.generateWithContext(
            {
              prompt: `Generate ${options.type} code`,
              language: options.language,
            },
            {
              currentCode: options.fileContent,
              filePath: options.filePath,
              analysisResult,
            }
          );

          setGeneratedContent(enhancedResult.generatedCode);
          return {
            content: enhancedResult.generatedCode,
            confidence: enhancedResult.confidence,
          };
        }

        // Fallback to original generation
        const result = await actions.generateCode(options);
        if (result) {
          setGeneratedContent(result.content);
          return result;
        }
        return null;
      } catch (error) {
        console.error('Error generating code:', error);
        setGenerationError('Failed to generate code');
        return null;
      } finally {
        setIsGenerating(false);
      }
    },
    [actions, analysisState.analysisResults, aiServiceRef]
  );

  // Enhanced configuration management
  const updateConfiguration = useCallback(
    async (newConfig: Partial<AIAnalysisConfigExtended>) => {
      try {
        if (!aiServiceRef.current) {
          throw new Error('AI service not initialized');
        }

        await aiServiceRef.current.updateConfiguration(newConfig);

        // Update local config state
        setConfig((prev: AnalysisConfiguration | null) => {
          if (!prev) {
            // Provide minimal defaults when no previous config exists
            return {
              enabledCategories: ['code-smell', 'performance', 'security'],
              severityThreshold: 'warning',
              realTimeAnalysis: false,
              analysisOnSave: false,
              maxSuggestions: 10,
              aiProvider: { type: 'mock' },
              analysisPreferences: {
                enableCodeSmells: true,
                enableSecurity: true,
                enablePerformance: true,
                enableCodeStyle: true,
                enableArchitecture: true,
                enableLearning: false,
                confidenceThreshold: 0.5,
                timeoutSeconds: 30,
                includeExplanations: true,
                includeExamples: false,
                privacyMode: 'opt-out',
              },
              learningPreferences: {
                enableLearning: false,
                privacyMode: 'opt-out',
                shareAnonymousData: false,
                retainPersonalData: false,
                dataRetentionDays: 30,
                allowModelTraining: false,
              },
              confidenceThreshold: 0.5,
              excludePatterns: [],
              maxFileSizeKb: 1024,
              timeoutSeconds: 30,
              customRules: [],
              performance: {
                enableBenchmarking: false,
                profileMemoryUsage: false,
                detectHotPaths: false,
                enablePerformanceHints: true,
                enableOptimizationSuggestions: true,
              },
              security: {
                enableVulnerabilityScanning: true,
                checkDependencies: true,
                scanForSecrets: false,
                enableSecurityIssueDetection: true,
                includeCweMapping: false,
              },
              architecture: {
                enablePatternDetection: true,
                checkCircularDependencies: true,
                analyzeCoupling: true,
                enableArchitectureSuggestions: true,
                detectAntiPatterns: true,
              },
              codeStyle: {
                enableStyleViolationDetection: true,
                enforceNamingConventions: true,
                checkFormattingConsistency: true,
                enforceRustIdioms: true,
                requireDocumentation: false,
              },
              learning: {
                enableLearningSystem: false,
                recordSuccessfulFixes: false,
                useLearnedPatterns: false,
                shareAnonymousData: false,
                confidenceThresholdForLearning: 0.8,
              },
              compiler: {
                enableCompilerIntegration: false,
                parseCargoCheckOutput: false,
                enableErrorExplanations: false,
                enableSuggestedFixes: false,
                cacheExplanations: false,
              },
              ...newConfig,
            };
          }
          return { ...prev, ...newConfig };
        });

        // Clear cache to force re-analysis with new config
        if (codeAnalyzerRef.current) {
          codeAnalyzerRef.current.clearCache();
        }

        setAnalysisState((prev) => ({
          ...prev,
          analysisResults: new Map(),
          enhancedAnalysisResults: new Map(),
          lastAnalysisTime: new Map(),
        }));
      } catch (error) {
        console.error('Error updating configuration:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to update configuration';
        setGenerationError(errorMessage);
        throw error;
      }
    },
    [aiServiceRef, codeAnalyzerRef]
  );

  // New method: Update enhanced configuration
  const updateEnhancedConfiguration = useCallback(
    async (newConfig: Partial<AnalysisConfiguration>): Promise<void> => {
      try {
        await invoke('update_enhanced_ai_config', { config: newConfig });

        setEnhancedConfig((prev: AnalysisConfiguration | null) => {
          if (!prev) {
            // Provide minimal defaults when no previous config exists
            return {
              enabledCategories: ['code-smell', 'performance', 'security'],
              severityThreshold: 'warning',
              realTimeAnalysis: false,
              analysisOnSave: false,
              maxSuggestions: 10,
              aiProvider: { type: 'mock' },
              analysisPreferences: {
                enableCodeSmells: true,
                enableSecurity: true,
                enablePerformance: true,
                enableCodeStyle: true,
                enableArchitecture: true,
                enableLearning: false,
                confidenceThreshold: 0.5,
                timeoutSeconds: 30,
                includeExplanations: true,
                includeExamples: false,
                privacyMode: 'opt-out',
              },
              learningPreferences: {
                enableLearning: false,
                privacyMode: 'opt-out',
                shareAnonymousData: false,
                retainPersonalData: false,
                dataRetentionDays: 30,
                allowModelTraining: false,
              },
              confidenceThreshold: 0.5,
              excludePatterns: [],
              maxFileSizeKb: 1024,
              timeoutSeconds: 30,
              customRules: [],
              performance: {
                enableBenchmarking: false,
                profileMemoryUsage: false,
                detectHotPaths: false,
                enablePerformanceHints: true,
                enableOptimizationSuggestions: true,
              },
              security: {
                enableVulnerabilityScanning: true,
                checkDependencies: true,
                scanForSecrets: false,
                enableSecurityIssueDetection: true,
                includeCweMapping: false,
              },
              architecture: {
                enablePatternDetection: true,
                checkCircularDependencies: true,
                analyzeCoupling: true,
                enableArchitectureSuggestions: true,
                detectAntiPatterns: true,
              },
              codeStyle: {
                enableStyleViolationDetection: true,
                enforceNamingConventions: true,
                checkFormattingConsistency: true,
                enforceRustIdioms: true,
                requireDocumentation: false,
              },
              learning: {
                enableLearningSystem: false,
                recordSuccessfulFixes: false,
                useLearnedPatterns: false,
                shareAnonymousData: false,
                confidenceThresholdForLearning: 0.8,
              },
              compiler: {
                enableCompilerIntegration: false,
                parseCargoCheckOutput: false,
                enableErrorExplanations: false,
                enableSuggestedFixes: false,
                cacheExplanations: false,
              },
              ...newConfig,
            };
          }
          return { ...prev, ...newConfig };
        });

        // Clear cache to force re-analysis with new config
        if (codeAnalyzerRef.current) {
          codeAnalyzerRef.current.clearCache();
        }

        setAnalysisState((prev) => ({
          ...prev,
          analysisResults: new Map(),
          enhancedAnalysisResults: new Map(),
          lastAnalysisTime: new Map(),
        }));
      } catch (error) {
        console.error('Error updating enhanced configuration:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to update enhanced configuration';
        setGenerationError(errorMessage);
        throw error;
      }
    },
    [codeAnalyzerRef]
  );

  // Cancel analysis operations
  const cancelAnalysis = useCallback(
    async (filePath?: string) => {
      try {
        if (filePath) {
          // Cancel specific file analysis
          const controller = abortControllersRef.current.get(filePath);
          if (controller) {
            controller.abort();
            abortControllersRef.current.delete(filePath);
          }

          if (codeAnalyzerRef.current) {
            await codeAnalyzerRef.current.cancelAnalysis(filePath);
            log.debug('Frontend analysis cancelled for file', { filePath });
          }
        } else {
          // Cancel all analyses
          abortControllersRef.current.forEach((controller) => controller.abort());
          abortControllersRef.current.clear();

          // Cancel workspace analysis
          if (analysisState.workspaceAnalysisId) {
            log.debug('Cancelling backend analysis');
            await invoke('cancel_analysis', { analysis_id: analysisState.workspaceAnalysisId });

            setAnalysisState((prev) => ({
              ...prev,
              workspaceAnalysisId: null,
              workspaceProgress: null,
              isAnalyzing: false,
            }));

            log.info('Analysis cancelled successfully');
          }
        }
      } catch (error) {
        log.error('Failed to cancel analysis', error instanceof Error ? error : undefined, {
          filePath,
        });
      }
    },
    [analysisState.workspaceAnalysisId, codeAnalyzerRef]
  );

  // Enhanced cache management
  const clearAnalysisCache = useCallback(() => {
    log.info('Clearing analysis cache');
    if (codeAnalyzerRef.current) {
      codeAnalyzerRef.current.clearCache();
      log.info('Analysis cache cleared');
    }

    setAnalysisState((prev) => ({
      ...prev,
      analysisResults: new Map(),
      enhancedAnalysisResults: new Map(),
      lastAnalysisTime: new Map(),
      analysisErrors: new Map(),
      compilerDiagnostics: new Map(),
    }));
  }, [codeAnalyzerRef]);

  // New method: Clear learning data
  const clearLearningData = useCallback(async (): Promise<void> => {
    log.info('Clearing learning data');
    try {
      await invoke('clear_learning_data');

      setAnalysisState((prev) => ({
        ...prev,
        learnedPatterns: [],
        learningSystemData: null,
      }));

      log.info('Successfully cleared learning data');
    } catch (error) {
      log.error('Failed to clear learning data', error instanceof Error ? error : undefined);
      const errorMessage = error instanceof Error ? error.message : 'Failed to clear learning data';
      setGenerationError(errorMessage);
      throw error;
    }
  }, []);

  // Enhanced analysis statistics
  const getAnalysisStats = useCallback(() => {
    const cacheStats = codeAnalyzerRef.current?.getCacheStats() || {
      size: 0,
      oldestEntry: 0,
      newestEntry: 0,
    };

    return {
      cacheStats,
      analysisResultsCount: analysisState.analysisResults.size,
      enhancedAnalysisResultsCount: analysisState.enhancedAnalysisResults.size,
      errorsCount: analysisState.analysisErrors.size,
      compilerDiagnosticsCount: analysisState.compilerDiagnostics.size,
      learnedPatternsCount: analysisState.learnedPatterns.length,
      isAnalyzing: analysisState.isAnalyzing,
      workspaceProgress: analysisState.workspaceProgress,
      learningSystemData: analysisState.learningSystemData,
    };
  }, [analysisState, codeAnalyzerRef]);

  // New method: Get learning system statistics
  const getLearningStats = useCallback(() => {
    return {
      totalPatternsLearned: analysisState.learnedPatterns.length,
      averageConfidence:
        analysisState.learnedPatterns.length > 0
          ? analysisState.learnedPatterns.reduce((sum, pattern) => sum + pattern.confidence, 0) /
            analysisState.learnedPatterns.length
          : 0,
      successfulFixesCount: analysisState.learnedPatterns.reduce(
        (sum, pattern) => sum + pattern.successCount,
        0
      ),
      failedFixesCount: analysisState.learnedPatterns.reduce(
        (sum, pattern) => sum + pattern.failureCount,
        0
      ),
      learningEnabled: learningPreferences?.enableLearning ?? false,
      privacyMode: learningPreferences?.privacyMode ?? 'opt-out',
    };
  }, [analysisState.learnedPatterns, learningPreferences]);

  const generateTests = useCallback(
    async (code: string, filePath: string) => {
      const options: CodeGenerationOptions = {
        context: code,
        cursorPosition: { line: 0, character: 0 },
        fileContent: code,
        filePath,
        language: 'rust',
        type: 'test',
      };
      return generateCode(options);
    },
    [generateCode]
  );

  const generateDocumentation = useCallback(
    async (code: string, filePath: string) => {
      const options: CodeGenerationOptions = {
        context: code,
        cursorPosition: { line: 0, character: 0 },
        fileContent: code,
        filePath,
        language: 'rust',
        type: 'documentation',
      };
      return generateCode(options);
    },
    [generateCode]
  );

  const explainCode = useCallback(
    async (code: string, filePath?: string) => {
      try {
        setIsGenerating(true);
        setGenerationError(null);

        // Use AI service for enhanced code explanation with analysis context
        if (aiServiceRef.current && filePath) {
          const analysisResult = analysisState.analysisResults.get(filePath);

          const result = await aiServiceRef.current.generateWithContext(
            {
              prompt:
                'Explain this code in detail, including its purpose, functionality, and any potential issues',
              language: 'rust',
            },
            {
              currentCode: code,
              filePath,
              analysisResult,
            }
          );

          setGeneratedContent(result.generatedCode);
          return result.generatedCode;
        }

        // Fallback explanation
        await new Promise((resolve) => setTimeout(resolve, 1000));

        const explanation =
          `This code ${code.substring(0, 50)}... appears to be a function that performs some operation. ` +
          `It takes some input, processes it, and returns a result.`;

        setGeneratedContent(explanation);
        return explanation;
      } catch (error) {
        console.error('Error explaining code:', error);
        setGenerationError('Failed to explain code');
        return null;
      } finally {
        setIsGenerating(false);
      }
    },
    [aiServiceRef, analysisState.analysisResults]
  );

  const refactorCode = useCallback(
    async (code: string, filePath: string) => {
      const options: CodeGenerationOptions = {
        context: code,
        cursorPosition: { line: 0, character: 0 },
        fileContent: code,
        filePath,
        language: 'rust',
        type: 'refactor',
      };
      return generateCode(options);
    },
    [generateCode]
  );

  const clearGeneratedContent = useCallback(() => {
    setGeneratedContent(null);
    setGenerationError(null);
  }, []);

  const togglePanel = useCallback(() => {
    if (state.isPanelVisible) {
      actions.hidePanel();
    } else {
      actions.showPanel();
    }
  }, [state.isPanelVisible, actions]);

  return {
    // Enhanced State
    isGenerating,
    generationError,
    generatedContent,
    hasSuggestions: Array.isArray(state.suggestions) && state.suggestions.length > 0,
    isPanelVisible: state.isPanelVisible,
    suggestions: state.suggestions,

    // Analysis State
    isAnalyzing: analysisState.isAnalyzing,
    analysisResults: analysisState.analysisResults,
    enhancedAnalysisResults: analysisState.enhancedAnalysisResults,
    analysisErrors: analysisState.analysisErrors,
    workspaceProgress: analysisState.workspaceProgress,
    compilerDiagnostics: analysisState.compilerDiagnostics,
    learnedPatterns: analysisState.learnedPatterns,
    learningSystemData: analysisState.learningSystemData,
    config,
    enhancedConfig,
    learningPreferences,

    // Original Actions
    analyzeCurrentFile,
    generateCode,
    generateTests,
    generateDocumentation,
    explainCode,
    refactorCode,
    applyFix: actions.applyFix,
    dismissSuggestion: actions.dismissSuggestion,
    showPanel: actions.showPanel,
    hidePanel: actions.hidePanel,
    togglePanel,
    clearGeneratedContent,

    // Enhanced Analysis Actions
    analyzeWorkspace,
    getPerformanceSuggestions,
    runCodeQualityCheck,
    applySuggestion,
    updateConfiguration,
    cancelAnalysis,
    clearAnalysisCache,
    getAnalysisStats,

    // New Style Analysis Actions
    analyzeCodeStyle,
    analyzeArchitecture,
    getEnhancedPerformanceHints,
    runEnhancedAnalysis,

    // Learning System Actions
    recordSuccessfulFix,
    getLearnedPatterns,
    updateLearningPreferences,
    clearLearningData,
    getLearningStats,

    // Compiler Integration Actions
    getCompilerDiagnostics,
    explainErrorCode,
    resolveErrorWithLearning,
    applyFixWithLearning,

    // Enhanced Configuration Actions
    updateEnhancedConfiguration,
  };
};

export default useAIAssistant;

// Export types for external use
export type {
  AnalysisState,
  ArchitectureAnalysisOptions,
  CodeQualityCheckOptions,
  CompilerIntegrationOptions,
  LearningSystemOptions,
  PerformanceSuggestionsOptions,
  StyleAnalysisOptions,
};
