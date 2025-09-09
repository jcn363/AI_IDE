import { invoke } from '@tauri-apps/api/core';
import { Diagnostic, DiagnosticSeverity } from 'vscode-languageserver';
import type { Range } from 'vscode-languageserver-types';
import type {
  AIAnalysisConfig,
  AnalysisCategory,
  AnalysisMetadata,
  ArchitectureSuggestion,
  CodeSmell,
  CompilerDiagnostic,
  CompilerIntegrationConfig,
  ErrorCodeExplanation,
  FixSuggestion,
  LearnedPattern,
  LearningPreferences,
  PerformanceHint,
  SecurityIssue,
  SeverityLevel,
  StyleViolation,
  AIProvider
} from '../types/index';

// Enhanced types for better analysis results
export interface AIAnalysisConfigExtended extends AIAnalysisConfig {
  learning_preferences: LearningPreferences;
  compiler_integration: CompilerIntegrationConfig;
}

type SecurityVulnerability = {
  severity: string;
  description?: string;
  confidence?: number;
  impact?: string;
};

type PerformanceIssue = {
  severity?: string;
  description?: string;
  confidence?: number;
  impact?: string;
};

export interface AnalysisPreferences {
  enable_code_smells: boolean;
  enable_performance: boolean;
  enable_security: boolean;
  enable_style: boolean;
  enable_architecture: boolean;
  enable_learning: boolean;
  timeout_seconds: number;
  max_suggestions_per_category: number;
  confidence_threshold: number;
  include_explanations: boolean;
  include_examples: boolean;
  privacy_mode: 'opt-in' | 'opt-out' | 'anonymous';
}

export interface FileAnalysisRequest {
  file_path: string;
  content: string;
  config?: AIAnalysisConfigExtended;
}

export interface AnalysisProgress {
  id: string;
  status: 'Queued' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';
  progress_percentage: number;
  current_file?: string;
  total_files: number;
  processed_files: number;
  start_time: string;
  estimated_completion?: string;
  error_message?: string;
}

// Enhanced interfaces for better analysis results
export interface CodeSuggestion {
  message: string;
  severity: DiagnosticSeverity;
  range: Range;
  suggestion: string;
  category: AnalysisCategory;
  confidence_score?: number;
  fix_action?: QuickFix;
  detailed_explanation?: string;
  learned_from?: string; // ID of learned pattern
  success_rate?: number; // Historical success rate
}

export interface QuickFix {
  title: string;
  changes: CodeChange[];
  is_preferred: boolean;
}

export interface CodeChange {
  file_path: string;
  range: [number, number, number, number]; // [start_line, start_col, end_line, end_col]
  new_text: string;
  old_text?: string;
  change_type?: 'insert' | 'delete' | 'replace' | 'move';
}

export interface CodeMetrics {
  complexity: number;
  maintainability: number;
  securityScore: number;
  performanceScore: number;
  lines_of_code?: number;
  cyclomatic_complexity?: number;
  technical_debt_minutes?: number;
  code_coverage_percentage?: number;
  qualityScore?: number;
}

export interface CodeAnalysisResult {
  diagnostics: Diagnostic[];
  suggestions: CodeSuggestion[];
  metrics: CodeMetrics;
  analysis_id?: string;
  timestamp?: string;
  code_smells?: CodeSmell[];
  performance_hints?: PerformanceHint[];
  security_issues?: SecurityIssue[];
  style_violations?: StyleViolation[];
  architecture_suggestions?: ArchitectureSuggestion[];
  learned_patterns?: LearnedPattern[];
  compiler_diagnostics?: CompilerDiagnostic[];
  metadata?: AnalysisMetadata;

  // Additional properties expected by StatusBar component
  summary?: {
    totalIssues: number;
    timestamp?: number;
  };
  codeSmells?: CodeSmell[]; // alias for code_smells
  style?: StyleViolation[]; // alias for style_violations
  security?: {
    vulnerabilities: SecurityIssue[];
  };
  architecture?: {
    patterns: ArchitectureSuggestion[];
  };
  progress?: number;
}

// Analysis type-specific request interfaces
export interface CodeStyleAnalysisRequest {
  file_path: string;
  content: string;
  style_rules?: string[];
  auto_fix?: boolean;
}

export interface ArchitectureAnalysisRequest {
  workspace_path: string;
  include_dependencies?: boolean;
  analysis_depth?: 'shallow' | 'deep';
}

export interface PerformanceAnalysisRequest {
  file_path?: string;
  workspace_path?: string;
  include_benchmarks?: boolean;
  profile_memory?: boolean;
}

export interface LearningSystemRequest {
  error_pattern: string;
  applied_fix: FixSuggestion;
  success: boolean;
  user_feedback?: 'positive' | 'negative' | 'neutral';
  context: string;
}

// Enhanced cache entry interface
interface CacheEntry {
  result: CodeAnalysisResult;
  timestamp: number;
  contentHash: string;
  analysisType: AnalysisCategory[];
  metadata?: AnalysisMetadata;
}

// Analysis request tracking
interface AnalysisRequest {
  id: string;
  promise: Promise<CodeAnalysisResult>;
  controller: AbortController;
  analysisType: AnalysisCategory[];
}

// Main AI Service that coordinates different AI features
export class AIService {
  private static instance: AIService;
  private codeAnalyzer: CodeAnalyzer;
  private isInitialized = false;
  
  private constructor() {
    this.codeAnalyzer = CodeAnalyzer.getInstance();
    // Initialize other services
  }

  public static getInstance(): AIService {
    if (!AIService.instance) {
      AIService.instance = new AIService();
    }
    return AIService.instance;
  }

  public async initialize(): Promise<void> {
    if (this.isInitialized) return;
    
    try {
      // Initialize all AI services
      await Promise.all([
        this.codeAnalyzer.initialize(),
        // Initialize other services when available
      ]);
      
      this.isInitialized = true;
      console.log('AIService initialized successfully');
    } catch (error) {
      console.error('Failed to initialize AIService:', error);
      throw error;
    }
  }

  public async analyzeAndFix(
    code: string, 
    filePath: string,
    options: { autoFix?: boolean; signal?: AbortSignal } = {}
  ): Promise<{ analysis: CodeAnalysisResult; appliedFixes?: string[] }> {
    const { autoFix = false, signal } = options;
    
    // Perform comprehensive analysis
    const analysis = await this.codeAnalyzer.analyzeCode(code, filePath, { signal });
    
    const result: { analysis: CodeAnalysisResult; appliedFixes?: string[] } = { analysis };
    
    // Apply automatic fixes if requested
    if (autoFix) {
      const appliedFixes: string[] = [];
      
      for (const suggestion of analysis.suggestions) {
        if (suggestion.fix_action && suggestion.fix_action.is_preferred) {
          try {
            await invoke('apply_ai_suggestion', {
              suggestion_id: `auto-fix-${Date.now()}`,
              changes: suggestion.fix_action.changes,
              create_backup: true,
            });
            appliedFixes.push(suggestion.message);
          } catch (error) {
            console.error('Failed to apply auto-fix:', error);
          }
        }
      }
      
      result.appliedFixes = appliedFixes;
    }
    
    return result;
  }

  public async generateWithContext(
    options: {
      prompt: string;
      language?: string;
      maxTokens?: number;
      temperature?: number;
    },
    context: {
      currentCode?: string;
      filePath?: string;
      analysisResult?: CodeAnalysisResult;
      projectContext?: Record<string, any>;
    }
  ): Promise<{ generatedCode: string; confidence: number }> {
    // Enhanced code generation using analysis context
    let enhancedPrompt = options.prompt;
    
    if (context.analysisResult) {
      const issues = context.analysisResult.suggestions
        .filter(s => s.severity === DiagnosticSeverity.Error || s.severity === DiagnosticSeverity.Warning)
        .map(s => s.message)
        .slice(0, 5); // Top 5 issues
      
      if (issues.length > 0) {
        enhancedPrompt += `\n\nPlease address these code quality issues:\n${issues.join('\n')}`;
      }
      
      if (context.analysisResult.metrics) {
        enhancedPrompt += `\n\nCurrent code metrics - Complexity: ${context.analysisResult.metrics.complexity}, Maintainability: ${context.analysisResult.metrics.maintainability}`;
      }
    }
    
    if (context.currentCode) {
      enhancedPrompt += `\n\nCurrent code context:\n\`\`\`${options.language || 'typescript'}\n${context.currentCode}\n\`\`\``;
    }
    
    // This would integrate with the actual code generation service
    // For now, return a placeholder
    return {
      generatedCode: `// Generated code based on: ${options.prompt}`,
      confidence: 0.8,
    };
  }

  public async getPerformanceSuggestions(
    filePath?: string,
    workspacePath?: string
  ): Promise<CodeSuggestion[]> {
    try {
      const suggestions = await invoke('get_performance_suggestions', {
        file_path: filePath,
        workspace_path: workspacePath,
        config: this.codeAnalyzer.getConfig(),
      });
      
      return suggestions.map((suggestion: any) => ({
        message: suggestion.description,
        severity: this.mapRustSeverityToDiagnostic(suggestion.impact),
        range: suggestion.location,
        suggestion: suggestion.suggestion,
        category: 'performance' as const,
        confidence_score: suggestion.confidence || 0.8,
        detailed_explanation: suggestion.detailed_explanation,
      }));
    } catch (error) {
      console.error('Failed to get performance suggestions:', error);
      return [];
    }
  }

  public async runCodeQualityCheck(
    targetPath: string,
    options: {
      runClippy?: boolean;
      runRustfmt?: boolean;
      runAIAnalysis?: boolean;
    } = {}
  ): Promise<any> {
    const {
      runClippy = true,
      runRustfmt = true,
      runAIAnalysis = true,
    } = options;
    
    try {
      return await invoke('run_code_quality_check', {
        target_path: targetPath,
        run_clippy: runClippy,
        run_rustfmt: runRustfmt,
        run_ai_analysis: runAIAnalysis,
        config: this.codeAnalyzer.getConfig(),
      });
    } catch (error) {
      console.error('Failed to run code quality check:', error);
      throw error;
    }
  }

  public async analyzeWorkspace(
    workspacePath: string,
    options: {
      includeDependencies?: boolean;
      includeSecurityScan?: boolean;
      signal?: AbortSignal;
    } = {}
  ): Promise<string> {
    const {
      includeDependencies = true,
      includeSecurityScan = true,
    } = options;
    
    try {
      return await invoke('analyze_workspace', {
        workspace_path: workspacePath,
        include_dependencies: includeDependencies,
        include_security_scan: includeSecurityScan,
        config: this.codeAnalyzer.getConfig(),
      });
    } catch (error) {
      console.error('Failed to analyze workspace:', error);
      throw error;
    }
  }

  public async applySuggestion(
    suggestionId: string,
    changes: CodeChange[],
    createBackup = true
  ): Promise<string> {
    try {
      return await invoke('apply_ai_suggestion', {
        suggestion_id: suggestionId,
        changes,
        create_backup: createBackup,
      });
    } catch (error) {
      console.error('Failed to apply suggestion:', error);
      throw error;
    }
  }

  public async updateConfiguration(config: Partial<AIAnalysisConfigExtended>): Promise<void> {
    await this.codeAnalyzer.updateConfig(config);
  }

  // Enhanced analysis methods using new CodeAnalyzer capabilities
  public async analyzeCodeStyle(
    code: string,
    filePath: string,
    options: {
      styleRules?: string[];
      autoFix?: boolean;
      signal?: AbortSignal;
    } = {}
  ): Promise<StyleViolation[]> {
    return this.codeAnalyzer.analyzeCodeStyle(code, filePath, options);
  }

  public async analyzeArchitecture(
    workspacePath: string,
    options: {
      includeDependencies?: boolean;
      analysisDepth?: 'shallow' | 'deep';
      signal?: AbortSignal;
    } = {}
  ): Promise<ArchitectureSuggestion[]> {
    return this.codeAnalyzer.analyzeArchitecture(workspacePath, options);
  }

  public async getEnhancedPerformanceHints(
    filePath?: string,
    workspacePath?: string,
    options: {
      includeBenchmarks?: boolean;
      profileMemory?: boolean;
      signal?: AbortSignal;
    } = {}
  ): Promise<PerformanceHint[]> {
    return this.codeAnalyzer.getPerformanceHints(filePath, workspacePath, options);
  }

  // Learning system integration
  public async recordFixSuccess(
    errorPattern: string,
    appliedFix: FixSuggestion,
    success: boolean,
    userFeedback?: 'positive' | 'negative' | 'neutral',
    context?: string
  ): Promise<boolean> {
    return this.codeAnalyzer.recordSuccessfulFix(errorPattern, appliedFix, success, userFeedback, context);
  }

  public async getLearnedPatterns(errorContext: string): Promise<LearnedPattern[]> {
    return this.codeAnalyzer.getLearnedPatterns(errorContext);
  }

  public async updateLearningPreferences(preferences: Partial<LearningPreferences>): Promise<void> {
    await this.codeAnalyzer.updateLearningPreferences(preferences);
  }

  // Compiler integration
  public async getCompilerDiagnostics(
    workspacePath: string,
    options: {
      includeExplanations?: boolean;
      includeSuggestedFixes?: boolean;
    } = {}
  ): Promise<{ diagnostics: CompilerDiagnostic[]; explanations: Record<string, ErrorCodeExplanation> }> {
    return this.codeAnalyzer.getCompilerDiagnostics(workspacePath, options);
  }

  public async explainErrorCode(errorCode: string): Promise<ErrorCodeExplanation | null> {
    return this.codeAnalyzer.explainErrorCode(errorCode);
  }

  // Enhanced analysis with specific types
  public async analyzeWithTypes(
    code: string,
    filePath: string,
    analysisTypes: AnalysisCategory[],
    options: { signal?: AbortSignal } = {}
  ): Promise<CodeAnalysisResult> {
    return this.codeAnalyzer.analyzeCode(code, filePath, {
      ...options,
      analysisTypes,
    });
  }

  public getCodeAnalyzer(): CodeAnalyzer {
    return this.codeAnalyzer;
  }

  private mapRustSeverityToDiagnostic(severity: string): DiagnosticSeverity {
    switch (severity.toLowerCase()) {
      case 'critical':
      case 'high':
        return DiagnosticSeverity.Error;
      case 'medium':
        return DiagnosticSeverity.Warning;
      case 'low':
        return DiagnosticSeverity.Information;
      default:
        return DiagnosticSeverity.Hint;
    }
  }
}

export class CodeAnalyzer {
  private static instance: CodeAnalyzer;
  private cache = new Map<string, CacheEntry>();
  private pendingRequests = new Map<string, AnalysisRequest>();
  private debounceTimers = new Map<string, NodeJS.Timeout>();
  private config: AIAnalysisConfigExtended;
  private isInitialized = false;
  
  private constructor() {
    this.config = this.getDefaultConfig();
  }

  // Public getter for config access by AIService
  public getConfig(): AIAnalysisConfigExtended {
    return this.config;
  }

  public static getInstance(): CodeAnalyzer {
    if (!CodeAnalyzer.instance) {
      CodeAnalyzer.instance = new CodeAnalyzer();
    }
    return CodeAnalyzer.instance;
  }

  public async initialize(): Promise<void> {
    if (this.isInitialized) return;
    
    try {
      // Initialize AI service with default config
      await invoke('initialize_ai_service', { config: this.config });
      this.isInitialized = true;
      console.log('CodeAnalyzer initialized successfully');
    } catch (error) {
      console.error('Failed to initialize CodeAnalyzer:', error);
      throw new Error(`CodeAnalyzer initialization failed: ${error}`);
    }
  }

  public async updateConfig(newConfig: Partial<AIAnalysisConfigExtended>): Promise<void> {
    this.config = { ...this.config, ...newConfig };
    
    try {
      await invoke('update_ai_config', { config: this.config });
      console.log('AI configuration updated successfully');
      
      // Clear cache when configuration changes to ensure fresh analysis
      this.clearCache();
    } catch (error) {
      console.error('Failed to update AI configuration:', error);
      throw new Error(`Failed to update AI configuration: ${error}`);
    }
  }

  private getDefaultConfig(): AIAnalysisConfigExtended {
    return {
      provider: { OpenAI: { api_key: '', model: 'gpt-4' } },
      analysis_preferences: {
        enable_code_smells: true,
        enable_performance: true,
        enable_security: true,
        enable_style: true,
        enable_architecture: true,
        enable_learning: true,
        timeout_seconds: 30,
        max_suggestions_per_category: 10,
        confidence_threshold: 0.7,
        include_explanations: true,
        include_examples: true,
        privacy_mode: 'opt-in',
      },
      enable_real_time: true,
      enable_workspace_analysis: true,
      max_file_size_kb: 1024,
      excluded_paths: ['target/', 'node_modules/', '.git/', 'dist/', 'build/'],
      learning_preferences: {
        enable_learning: true,
        privacy_mode: 'opt-in',
        share_anonymous_data: false,
        retain_personal_data: true,
        data_retention_days: 90,
        allow_model_training: false,
        confidence_threshold_for_learning: 0.8,
      },
      compiler_integration: {
        enable_compiler_integration: true,
        parse_cargo_check_output: true,
        enable_error_explanations: true,
        enable_suggested_fixes: true,
        cache_explanations: true,
        explanation_cache_ttl_hours: 24,
      },
    };
  }

  public async analyzeCode(
    code: string, 
    filePath: string, 
    options: { 
      useCache?: boolean; 
      debounceMs?: number; 
      signal?: AbortSignal;
      analysisTypes?: AnalysisCategory[];
    } = {}
  ): Promise<CodeAnalysisResult> {
    const { 
      useCache = true, 
      debounceMs = 300, 
      signal,
      analysisTypes = ['code-smell', 'performance', 'security', 'style', 'architecture']
    } = options;
    
    if (!this.isInitialized) {
      await this.initialize();
    }

    // Generate cache key including analysis types
    const contentHash = this.generateHash(code);
    const analysisTypesKey = analysisTypes.sort().join(',');
    const cacheKey = `${filePath}:${contentHash}:${analysisTypesKey}`;

    // Check cache first
    if (useCache && this.cache.has(cacheKey)) {
      const cached = this.cache.get(cacheKey)!;
      const cacheAge = Date.now() - cached.timestamp;
      const maxCacheAge = 5 * 60 * 1000; // 5 minutes
      
      if (cacheAge < maxCacheAge) {
        console.log('Returning cached analysis result for', filePath, 'with types:', analysisTypes);
        return cached.result;
      } else {
        this.cache.delete(cacheKey);
      }
    }

    // Handle debouncing
    if (debounceMs > 0) {
      return new Promise((resolve, reject) => {
        const existingEntry = this.debounceTimers.get(filePath) as any;

        // If there's an existing debounce entry, augment it
        if (existingEntry) {
          clearTimeout(existingEntry.timer);
          existingEntry.code = code;
          existingEntry.analysisTypes = analysisTypes;
          existingEntry.signal = signal;
          existingEntry.resolvers.push(resolve);
          existingEntry.rejecters.push(reject);
        }

        // Create a new debounce entry if none exists
        const entry = existingEntry || {
          code,
          analysisTypes,
          signal,
          resolvers: [resolve],
          rejecters: [reject],
          timer: null as any
        };

        // Set (or reset) the timer
        entry.timer = setTimeout(async () => {
          try {
            const result = await this.performAnalysis(entry.code, filePath, entry.signal, entry.analysisTypes);
            entry.resolvers.forEach((r: any) => r(result));
          } catch (error) {
            entry.rejecters.forEach((r: any) => r(error));
          } finally {
            this.debounceTimers.delete(filePath);
          }
        }, debounceMs);

        this.debounceTimers.set(filePath, entry);

        // Handle cancellation
        if (signal) {
          const abortHandler = () => {
            clearTimeout(entry.timer);
            entry.rejecters.forEach((r: any) => r(new Error('Analysis cancelled')));
            this.debounceTimers.delete(filePath);
            signal.removeEventListener('abort', abortHandler);
          };
          signal.addEventListener('abort', abortHandler);
        }
      });
    }

    return this.performAnalysis(code, filePath, signal, analysisTypes);
  }

  private async performAnalysis(
    code: string, 
    filePath: string, 
    signal?: AbortSignal,
    analysisTypes: AnalysisCategory[] = ['code-smell', 'performance', 'security', 'style', 'architecture']
  ): Promise<CodeAnalysisResult> {
    const requestId = `${filePath}:${Date.now()}`;
    
    // Check if there's already a pending request for this file
    const existingRequest = this.pendingRequests.get(filePath);
    if (existingRequest) {
      existingRequest.controller.abort();
      this.pendingRequests.delete(filePath);
    }

    const controller = new AbortController();
    
    try {
      // Create analysis request with enhanced configuration
      const enhancedConfig = {
        ...this.config,
        analysis_preferences: {
          ...this.config.analysis_preferences,
          enable_code_smells: analysisTypes.includes('code-smell'),
          enable_performance: analysisTypes.includes('performance'),
          enable_security: analysisTypes.includes('security'),
          enable_style: analysisTypes.includes('style'),
          enable_architecture: analysisTypes.includes('architecture'),
        }
      };

      const request: FileAnalysisRequest = {
        file_path: filePath,
        content: code,
        config: enhancedConfig,
      };

      // Track the request
      const analysisPromise = this.callRustAnalysis(request, controller.signal);
      this.pendingRequests.set(filePath, {
        id: requestId,
        promise: analysisPromise,
        controller,
        analysisType: analysisTypes,
      });

      // Handle external cancellation
      if (signal) {
        signal.addEventListener('abort', () => {
          controller.abort();
        });
      }

      const rustResult = await analysisPromise;
      
      // Process the result from Rust backend
      const processedResult = await this.processBackendResult(rustResult, code, filePath, analysisTypes);
      
      // Cache the result with analysis type information
      const contentHash = this.generateHash(code);
      const analysisTypesKey = analysisTypes.sort().join(',');
      const cacheKey = `${filePath}:${contentHash}:${analysisTypesKey}`;
      this.cache.set(cacheKey, {
        result: processedResult,
        timestamp: Date.now(),
        contentHash,
        analysisType: analysisTypes,
        metadata: processedResult.metadata,
      });

      // Clean up old cache entries (keep last 100)
      if (this.cache.size > 100) {
        const entries = Array.from(this.cache.entries());
        entries.sort((a, b) => a[1].timestamp - b[1].timestamp);
        const toDelete = entries.slice(0, entries.length - 100);
        toDelete.forEach(([key]) => this.cache.delete(key));
      }

      return processedResult;
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error('Analysis was cancelled');
      }
      
      console.error('Analysis failed for', filePath, ':', error);
      
      // Return fallback result with basic analysis
      return this.getFallbackAnalysisResult(code, filePath, error as Error, analysisTypes);
    } finally {
      this.pendingRequests.delete(filePath);
    }
  }

  private async callRustAnalysis(
    request: FileAnalysisRequest, 
    signal: AbortSignal
  ): Promise<any> {
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new Error('Analysis timeout'));
      }, this.config.analysis_preferences.timeout_seconds * 1000);

      signal.addEventListener('abort', () => {
        clearTimeout(timeoutId);
        reject(new Error('Analysis cancelled'));
      });

      invoke('analyze_file', request)
        .then((result) => {
          clearTimeout(timeoutId);
          resolve(result);
        })
        .catch((error) => {
          clearTimeout(timeoutId);
          reject(error);
        });
    });
  }

  private async processBackendResult(
    rustResult: any, 
    code: string, 
    filePath: string,
    analysisTypes: AnalysisCategory[]
  ): Promise<CodeAnalysisResult> {
    const diagnostics: Diagnostic[] = [];
    const suggestions: CodeSuggestion[] = [];

    // Process code smells
    if (rustResult.codeSmells && analysisTypes.includes('code-smell')) {
      this.processCodeSmells(rustResult.codeSmells, diagnostics, suggestions);
    }

    // Process performance hints (enhanced from performance_issues)
    if (rustResult.performanceHints && analysisTypes.includes('performance')) {
      this.processPerformanceHints(rustResult.performanceHints, diagnostics, suggestions);
    }

    // Process security issues (enhanced from security_vulnerabilities)
    if (rustResult.securityIssues && analysisTypes.includes('security')) {
      this.processSecurityIssues(rustResult.securityIssues, diagnostics, suggestions);
    }

    // Process style violations
    if (rustResult.styleViolations && analysisTypes.includes('style')) {
      this.processStyleViolations(rustResult.styleViolations, diagnostics, suggestions, filePath);
    }

    // Process architecture suggestions
    if (rustResult.architectureSuggestions && analysisTypes.includes('architecture')) {
      this.processArchitectureSuggestions(rustResult.architectureSuggestions, suggestions);
    }

    // Process learned patterns if available
    if (rustResult.learnedPatterns) {
      this.processLearnedPatterns(rustResult.learnedPatterns, suggestions);
    }

    // Process compiler diagnostics if available
    if (rustResult.compilerDiagnostics) {
      this.processCompilerDiagnostics(rustResult.compilerDiagnostics, diagnostics, suggestions);
    }

    // Map legacy field names for calculateEnhancedMetrics compatibility
    if (!rustResult.performance_issues && rustResult.performanceHints) {
      rustResult.performance_issues = rustResult.performanceHints;
    }
    if (!rustResult.security_vulnerabilities && rustResult.securityIssues) {
      rustResult.security_vulnerabilities = rustResult.securityIssues;
    }
    if (!rustResult.code_smells && rustResult.codeSmells) {
      rustResult.code_smells = rustResult.codeSmells;
    }

    // Calculate enhanced metrics
    const metrics = this.calculateEnhancedMetrics(rustResult, code);

    return {
      diagnostics,
      suggestions,
      metrics,
      analysis_id: rustResult.metadata?.analysisId,
      timestamp: rustResult.metadata?.timestamp,
      code_smells: rustResult.codeSmells,
      performance_hints: rustResult.performanceHints,
      security_issues: rustResult.securityIssues,
      style_violations: rustResult.styleViolations,
      architecture_suggestions: rustResult.architectureSuggestions,
      learned_patterns: rustResult.learnedPatterns,
      compiler_diagnostics: rustResult.compilerDiagnostics,
      metadata: rustResult.metadata,

      // New properties for StatusBar compatibility
      summary: {
        totalIssues: diagnostics.length + suggestions.length,
        timestamp: rustResult.metadata?.timestamp ? new Date(rustResult.metadata.timestamp).getTime() : undefined
      },
      codeSmells: rustResult.codeSmells,
      style: rustResult.styleViolations,
      security: rustResult.securityIssues ? {
        vulnerabilities: rustResult.securityIssues
      } : undefined,
      architecture: rustResult.architectureSuggestions ? {
        patterns: rustResult.architectureSuggestions
      } : undefined,
      progress: 100,
    };
  }
  private getFallbackAnalysisResult(
    code: string, 
    filePath: string, 
    error: Error,
    analysisTypes: AnalysisCategory[] = ['code-smell', 'performance', 'security', 'style', 'architecture']
  ): CodeAnalysisResult {
    console.warn('Using fallback analysis for', filePath, 'due to error:', error.message);
    
    const diagnostics: Diagnostic[] = [];
    const suggestions: CodeSuggestion[] = [];

    // Perform basic local analysis as fallback based on requested types
    if (analysisTypes.includes('code-smell')) {
      this.detectBasicCodeSmells(code, filePath, diagnostics, suggestions);
    }
    if (analysisTypes.includes('performance')) {
      this.checkBasicPerformance(code, filePath, diagnostics, suggestions);
    }
    if (analysisTypes.includes('security')) {
      this.checkBasicSecurity(code, filePath, diagnostics, suggestions);
    }
    if (analysisTypes.includes('style')) {
      this.checkBasicCodeStyle(code, filePath, suggestions);
    }
    if (analysisTypes.includes('architecture')) {
      this.checkBasicArchitecture(code, filePath, diagnostics, suggestions);
    }

    const metrics = this.calculateBasicMetrics(code);

    return { 
      diagnostics, 
      suggestions, 
      metrics,
      metadata: {
        analysisId: `fallback-${Date.now()}`,
        timestamp: new Date().toISOString(),
        durationMs: 0,
        analyzerVersion: 'fallback-1.0.0',
        fileCount: 1,
        linesAnalyzed: this.countLinesOfCode(code),
      }
    };
  }

  // Enhanced processing methods for new backend result types
  private processCodeSmells(
    codeSmells: CodeSmell[],
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    codeSmells.forEach(smell => {
      const severity = this.mapSeverityLevelToDiagnostic(smell.severity);
      
      diagnostics.push({
        range: this.convertRangeFromBackend(smell.lineRange, smell.columnRange),
        severity,
        message: smell.message,
        source: 'ai-code-smell',
        code: smell.smellType,
      });

      suggestions.push({
        message: smell.message,
        severity,
        range: this.convertRangeFromBackend(smell.lineRange, smell.columnRange),
        suggestion: smell.suggestion || 'Consider refactoring this code',
        category: 'code-smell',
        confidence_score: smell.confidence,
        detailed_explanation: smell.explanation || smell.message,
      });
    });
  }

  private processPerformanceHints(
    performanceHints: PerformanceHint[],
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    performanceHints.forEach(hint => {
      const severity = this.mapSeverityLevelToDiagnostic(hint.severity);
      
      diagnostics.push({
        range: this.convertRangeFromBackend(hint.lineRange, hint.columnRange),
        severity,
        message: hint.message,
        source: 'ai-performance',
        code: hint.hintType,
      });

      suggestions.push({
        message: hint.message,
        severity,
        range: this.convertRangeFromBackend(hint.lineRange, hint.columnRange),
        suggestion: hint.optimization || 'Consider optimizing this code',
        category: 'performance',
        confidence_score: hint.confidence,
        detailed_explanation: `${hint.message}\n\nOptimization: ${hint.optimization}\nEstimated impact: ${hint.estimatedImpact}`,
      });
    });
  }

  private processSecurityIssues(
    securityIssues: SecurityIssue[],
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    securityIssues.forEach(issue => {
      const severity = this.mapSeverityLevelToDiagnostic(issue.severity);
      
      diagnostics.push({
        range: this.convertRangeFromBackend(issue.lineRange, issue.columnRange),
        severity,
        message: issue.message,
        source: 'ai-security',
        code: issue.cweId || issue.issueType,
      });

      suggestions.push({
        message: issue.message,
        severity,
        range: this.convertRangeFromBackend(issue.lineRange, issue.columnRange),
        suggestion: issue.recommendation || 'Consider fixing this security issue',
        category: 'security',
        confidence_score: issue.confidence,
        detailed_explanation: issue.cweId 
          ? `${issue.message}\n\nCWE ID: ${issue.cweId}\nRecommendation: ${issue.recommendation}`
          : `${issue.message}\n\nRecommendation: ${issue.recommendation}`,
      });
    });
  }

  private processStyleViolations(
    styleViolations: StyleViolation[],
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[],
    filePath: string
  ): void {
    styleViolations.forEach(violation => {
      diagnostics.push({
        range: this.convertRangeFromBackend(violation.lineRange, violation.columnRange),
        severity: DiagnosticSeverity.Information,
        message: violation.message,
        source: 'ai-style',
        code: violation.violationType,
      });

      const quickFix: QuickFix | undefined = violation.autoFixable ? {
        title: `Fix ${violation.violationType}`,
        changes: [{
          file_path: filePath,
          range: [
            violation.lineRange[0],
            violation.columnRange[0],
            violation.lineRange[1],
            violation.columnRange[1],
          ],
          new_text: violation.suggestion || '',
          change_type: 'replace',
        }],
        is_preferred: true,
      } : undefined;

      suggestions.push({
        message: violation.message,
        severity: DiagnosticSeverity.Information,
        range: this.convertRangeFromBackend(violation.lineRange, violation.columnRange),
        suggestion: violation.suggestion || 'Consider fixing this style violation',
        category: 'style',
        confidence_score: violation.confidence,
        fix_action: quickFix,
      });
    });
  }

  private processArchitectureSuggestions(
    architectureSuggestions: ArchitectureSuggestion[],
    suggestions: CodeSuggestion[]
  ): void {
    architectureSuggestions.forEach(suggestion => {
      suggestions.push({
        message: suggestion.message,
        severity: this.mapSeverityLevelToDiagnostic(suggestion.severity),
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
        suggestion: suggestion.recommendation || 'Consider implementing this architectural improvement',
        category: 'architecture',
        confidence_score: suggestion.confidence,
        detailed_explanation: `${suggestion.message}\n\nRationale: ${suggestion.rationale}\nRecommendation: ${suggestion.recommendation}`,
      });
    });
  }

  private processLearnedPatterns(
    learnedPatterns: LearnedPattern[],
    suggestions: CodeSuggestion[]
  ): void {
    learnedPatterns.forEach(pattern => {
      suggestions.push({
        message: `Learned pattern: ${pattern.successfulFix.title}`,
        severity: DiagnosticSeverity.Information,
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
        suggestion: pattern.successfulFix.description,
        category: 'code-smell', // Default category for learned patterns
        confidence_score: pattern.confidence,
        learned_from: pattern.id,
        success_rate: pattern.successCount / (pattern.successCount + pattern.failureCount),
        detailed_explanation: `Based on ${pattern.successCount} successful applications. Last used: ${pattern.lastUsed}`,
      });
    });
  }

  private processCompilerDiagnostics(
    compilerDiagnostics: CompilerDiagnostic[],
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    compilerDiagnostics.forEach(diagnostic => {
      const severity = this.mapCompilerLevelToDiagnostic(diagnostic.level);
      
      diagnostic.spans.forEach(span => {
        if (span.isMainSpan) {
          diagnostics.push({
            range: {
              start: { line: span.lineStart - 1, character: span.columnStart - 1 },
              end: { line: span.lineEnd - 1, character: span.columnEnd - 1 }
            },
            severity,
            message: diagnostic.message,
            source: 'rustc',
            code: diagnostic.code?.code,
          });

          if (span.suggestedReplacement) {
            const quickFix: QuickFix = {
              title: 'Apply compiler suggestion',
              changes: [{
                file_path: span.fileName,
                range: [span.lineStart - 1, span.columnStart - 1, span.lineEnd - 1, span.columnEnd - 1],
                new_text: span.suggestedReplacement,
                old_text: span.text.join(''),
                change_type: 'replace',
              }],
              is_preferred: span.suggestionApplicability === 'machine-applicable',
            };

            suggestions.push({
              message: diagnostic.message,
              severity,
              range: {
                start: { line: span.lineStart - 1, character: span.columnStart - 1 },
                end: { line: span.lineEnd - 1, character: span.columnEnd - 1 }
              },
              suggestion: span.suggestedReplacement,
              category: 'code-smell',
              confidence_score: span.suggestionApplicability === 'machine-applicable' ? 0.9 : 0.7,
              fix_action: quickFix,
              detailed_explanation: diagnostic.code?.explanation || diagnostic.message,
            });
          }
        }
      });
    });
  }

  // Fallback implementations for when Rust backend is unavailable
  private detectBasicCodeSmells(
    code: string,
    filePath: string,
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    const lines = code.split('\n');
    
    // Detect long methods (basic heuristic)
    let currentFunctionStart = -1;
    let braceCount = 0;
    
    lines.forEach((line, index) => {
      const trimmed = line.trim();
      
      // Simple function detection
      if (trimmed.includes('function ') || trimmed.includes('fn ') || trimmed.includes('def ')) {
        currentFunctionStart = index;
        braceCount = 0;
      }
      
      // Count braces
      braceCount += (line.match(/{/g) || []).length;
      braceCount -= (line.match(/}/g) || []).length;
      
      // Function ended
      if (currentFunctionStart >= 0 && braceCount === 0 && trimmed.includes('}')) {
        const functionLength = index - currentFunctionStart + 1;
        if (functionLength > 50) {
          const range: Range = {
            start: { line: currentFunctionStart, character: 0 },
            end: { line: index, character: line.length }
          };
          
          diagnostics.push({
            range,
            severity: DiagnosticSeverity.Warning,
            message: `Long method detected (${functionLength} lines)`,
            source: 'basic-analysis',
            code: 'long-method',
          });
          
          suggestions.push({
            message: `This method is ${functionLength} lines long, consider breaking it into smaller functions`,
            severity: DiagnosticSeverity.Warning,
            range,
            suggestion: 'Break this method into smaller, more focused functions',
            category: 'code-smell',
            confidence_score: 0.7,
          });
        }
        currentFunctionStart = -1;
      }
      
      // Detect magic numbers
      const magicNumberRegex = /\b(?!0|1|2|10|100|1000)\d{2,}\b/g;
      const matches = line.match(magicNumberRegex);
      if (matches) {
        matches.forEach(match => {
          const column = line.indexOf(match);
          const range: Range = {
            start: { line: index, character: column },
            end: { line: index, character: column + match.length }
          };
          
          suggestions.push({
            message: `Magic number detected: ${match}`,
            severity: DiagnosticSeverity.Information,
            range,
            suggestion: `Consider extracting ${match} into a named constant`,
            category: 'code-smell',
            confidence_score: 0.6,
          });
        });
      }
    });
  }

  private checkBasicPerformance(
    code: string,
    filePath: string,
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    const lines = code.split('\n');
    
    lines.forEach((line, index) => {
      const trimmed = line.trim();
      
      // Detect nested loops (basic)
      if (trimmed.includes('for ') && code.substring(0, code.indexOf(line)).includes('for ')) {
        const range: Range = {
          start: { line: index, character: 0 },
          end: { line: index, character: line.length }
        };
        
        suggestions.push({
          message: 'Nested loop detected - potential performance concern',
          severity: DiagnosticSeverity.Information,
          range,
          suggestion: 'Consider optimizing nested loops or using more efficient algorithms',
          category: 'performance',
          confidence_score: 0.5,
        });
      }
      
      // Detect synchronous operations that might block
      if (trimmed.includes('readFileSync') || trimmed.includes('writeFileSync')) {
        const range: Range = {
          start: { line: index, character: 0 },
          end: { line: index, character: line.length }
        };
        
        suggestions.push({
          message: 'Synchronous file operation detected',
          severity: DiagnosticSeverity.Warning,
          range,
          suggestion: 'Consider using asynchronous file operations to avoid blocking',
          category: 'performance',
          confidence_score: 0.8,
        });
      }
    });
  }

  private checkBasicSecurity(
    code: string,
    filePath: string,
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    void filePath; // Ensure filePath parameter is recognized as used
    const lines = code.split('\n');
    
    lines.forEach((line, index) => {
      const trimmed = line.trim();
      
      // Detect potential hardcoded secrets
      const secretPatterns = [
        /password\s*[:=]\s*["'][^"']+["']/i,
        /api[_-]?key\s*[:=]\s*["'][^"']+["']/i,
        /secret\s*[:=]\s*["'][^"']+["']/i,
        /token\s*[:=]\s*["'][^"']+["']/i,
      ];
      
      secretPatterns.forEach(pattern => {
        if (pattern.test(line)) {
          const match = line.match(pattern);
          if (match) {
            const column = line.indexOf(match[0]);
            const range: Range = {
              start: { line: index, character: column },
              end: { line: index, character: column + match[0].length }
            };
            
            diagnostics.push({
              range,
              severity: DiagnosticSeverity.Error,
              message: 'Potential hardcoded secret detected',
              source: 'basic-security',
              code: 'hardcoded-secret',
            });
            
            suggestions.push({
              message: 'Hardcoded secret detected',
              severity: DiagnosticSeverity.Error,
              range,
              suggestion: 'Move secrets to environment variables or secure configuration',
              category: 'security',
              confidence_score: 0.9,
            });
          }
        }
      });
      
      // Detect eval usage
      if (trimmed.includes('eval(')) {
        const column = line.indexOf('eval(');
        const range: Range = {
          start: { line: index, character: column },
          end: { line: index, character: column + 5 }
        };
        
        suggestions.push({
          message: 'Use of eval() detected',
          severity: DiagnosticSeverity.Warning,
          range,
          suggestion: 'Avoid using eval() as it can lead to code injection vulnerabilities',
          category: 'security',
          confidence_score: 0.9,
        });
      }
    });
  }

  private checkBasicCodeStyle(
    code: string,
    filePath: string,
    suggestions: CodeSuggestion[]
  ): void {
    const lines = code.split('\n');
    
    lines.forEach((line, index) => {
      // Check for trailing whitespace
      if (line.endsWith(' ') || line.endsWith('\t')) {
        const range: Range = {
          start: { line: index, character: line.trimEnd().length },
          end: { line: index, character: line.length }
        };
        
        suggestions.push({
          message: 'Trailing whitespace detected',
          severity: DiagnosticSeverity.Information,
          range,
          suggestion: 'Remove trailing whitespace',
          category: 'style',
          confidence_score: 1.0,
          fix_action: {
            title: 'Remove trailing whitespace',
            changes: [{
              file_path: filePath,
              range: [index, line.trimEnd().length, index, line.length],
              new_text: '',
            }],
            is_preferred: true,
          },
        });
      }
      
      // Check for very long lines
      if (line.length > 120) {
        const range: Range = {
          start: { line: index, character: 120 },
          end: { line: index, character: line.length }
        };
        
        suggestions.push({
          message: `Line too long (${line.length} characters)`,
          severity: DiagnosticSeverity.Information,
          range,
          suggestion: 'Consider breaking long lines for better readability',
          category: 'style',
          confidence_score: 0.7,
        });
      }
    });
  }

  // Enhanced metrics calculation using Rust backend data
  private calculateEnhancedMetrics(rustResult: any, code: string): CodeMetrics {
    const baseMetrics = this.calculateBasicMetrics(code);
    
    // Use data from Rust analysis if available
    const complexity = rustResult.metrics?.cyclomatic_complexity || baseMetrics.complexity;
    const maintainability = rustResult.metrics?.maintainability_index || baseMetrics.maintainability;
    const securityScore = this.calculateSecurityScoreFromVulnerabilities(
      rustResult.security_vulnerabilities || []
    );
    const performanceScore = this.calculatePerformanceScoreFromIssues(
      rustResult.performance_issues || []
    );

    return {
      complexity,
      maintainability,
      securityScore,
      performanceScore,
      lines_of_code: rustResult.metrics?.lines_of_code || this.countLinesOfCode(code),
      cyclomatic_complexity: complexity,
      technical_debt_minutes: this.calculateTechnicalDebt(rustResult),
      code_coverage_percentage: rustResult.metrics?.code_coverage_percentage || 0,
    };
  }

  private calculateBasicMetrics(code: string): CodeMetrics {
    return {
      complexity: this.calculateCyclomaticComplexity(code),
      maintainability: this.calculateMaintainabilityIndex(code),
      securityScore: this.calculateSecurityScore(code),
      performanceScore: this.calculatePerformanceScore(code),
    };
  }

  private calculateCyclomaticComplexity(code: string): number {
    // Count decision points in the code
    const decisionKeywords = [
      'if', 'else', 'elif', 'while', 'for', 'switch', 'case', 
      'catch', 'try', '&&', '||', '?', 'match', 'when'
    ];
    
    let complexity = 1; // Base complexity
    
    decisionKeywords.forEach(keyword => {
      const regex = new RegExp(`\\b${keyword}\\b`, 'g');
      const matches = code.match(regex);
      if (matches) {
        complexity += matches.length;
      }
    });
    
    return Math.min(complexity, 50); // Cap at 50 for sanity
  }

  private calculateMaintainabilityIndex(code: string): number {
    const linesOfCode = this.countLinesOfCode(code);
    const cyclomaticComplexity = this.calculateCyclomaticComplexity(code);
    const halsteadVolume = this.estimateHalsteadVolume(code);
    
    // Simplified maintainability index calculation
    // Real formula: 171 - 5.2 * ln(Halstead Volume) - 0.23 * (Cyclomatic Complexity) - 16.2 * ln(Lines of Code)
    const maintainabilityIndex = Math.max(0, 
      171 - 5.2 * Math.log(halsteadVolume) - 0.23 * cyclomaticComplexity - 16.2 * Math.log(linesOfCode)
    );
    
    return Math.min(100, maintainabilityIndex);
  }

  private calculateSecurityScore(code: string): number {
    let score = 100;
    
    // Deduct points for potential security issues
    const securityPatterns = [
      { pattern: /eval\s*\(/g, penalty: 20 },
      { pattern: /innerHTML\s*=/g, penalty: 15 },
      { pattern: /document\.write\s*\(/g, penalty: 15 },
      { pattern: /password\s*[:=]\s*["'][^"']+["']/gi, penalty: 25 },
      { pattern: /api[_-]?key\s*[:=]\s*["'][^"']+["']/gi, penalty: 25 },
      { pattern: /\.exec\s*\(/g, penalty: 10 },
      { pattern: /setTimeout\s*\(\s*["'][^"']*["']/g, penalty: 10 },
    ];
    
    securityPatterns.forEach(({ pattern, penalty }) => {
      const matches = code.match(pattern);
      if (matches) {
        score -= penalty * matches.length;
      }
    });
    
    return Math.max(0, score);
  }

  private calculatePerformanceScore(code: string): number {
    let score = 100;
    
    // Deduct points for potential performance issues
    const performancePatterns = [
      { pattern: /for\s*\([^)]*\)\s*{[^}]*for\s*\(/g, penalty: 20 }, // Nested loops
      { pattern: /\.forEach\s*\([^)]*\)\s*{[^}]*\.forEach\s*\(/g, penalty: 15 },
      { pattern: /readFileSync|writeFileSync/g, penalty: 15 },
      { pattern: /JSON\.parse\s*\(\s*JSON\.stringify/g, penalty: 10 },
      { pattern: /new\s+RegExp\s*\(/g, penalty: 5 },
      { pattern: /\+\s*=.*\+/g, penalty: 5 }, // String concatenation in loops
    ];
    
    performancePatterns.forEach(({ pattern, penalty }) => {
      const matches = code.match(pattern);
      if (matches) {
        score -= penalty * matches.length;
      }
    });
    
    return Math.max(0, score);
  }

  private calculateSecurityScoreFromVulnerabilities(vulnerabilities: SecurityVulnerability[]): number {
    let score = 100;
    
    vulnerabilities.forEach(vuln => {
      switch (vuln.severity) {
        case 'Critical':
          score -= 25;
          break;
        case 'High':
          score -= 15;
          break;
        case 'Medium':
          score -= 10;
          break;
        case 'Low':
          score -= 5;
          break;
      }
    });
    
    return Math.max(0, score);
  }

  private calculatePerformanceScoreFromIssues(issues: PerformanceIssue[]): number {
    let score = 100;
    
    issues.forEach(issue => {
      switch (issue.impact) {
        case 'Critical':
          score -= 25;
          break;
        case 'High':
          score -= 15;
          break;
        case 'Medium':
          score -= 10;
          break;
        case 'Low':
          score -= 5;
          break;
      }
    });
    
    return Math.max(0, score);
  }

  private calculateTechnicalDebt(rustResult: any): number {
    let debtMinutes = 0;
    
    // Calculate debt from code smells
    if (rustResult.code_smells) {
      debtMinutes += rustResult.code_smells.reduce(
        (total: number, smell: CodeSmell) => total + (smell.estimated_fix_time_minutes || 10),
        0
      );
    }
    
    // Add debt from security vulnerabilities (estimated)
    if (rustResult.security_vulnerabilities) {
      debtMinutes += rustResult.security_vulnerabilities.length * 30; // 30 min per vulnerability
    }
    
    // Add debt from performance issues (estimated)
    if (rustResult.performance_issues) {
      debtMinutes += rustResult.performance_issues.length * 45; // 45 min per performance issue
    }
    
    return debtMinutes;
  }

  private countLinesOfCode(code: string): number {
    return code.split('\n').filter(line => line.trim().length > 0).length;
  }

  private estimateHalsteadVolume(code: string): number {
    // Simplified Halstead volume estimation
    const operators = code.match(/[+\-*/%=<>!&|^~?:;,(){}[\]]/g) || [];
    const operands = code.match(/\b[a-zA-Z_$][a-zA-Z0-9_$]*\b/g) || [];
    
    const uniqueOperators = new Set(operators).size;
    const uniqueOperands = new Set(operands).size;
    const totalOperators = operators.length;
    const totalOperands = operands.length;
    
    const vocabulary = uniqueOperators + uniqueOperands;
    const length = totalOperators + totalOperands;
    
    return length * Math.log2(vocabulary || 1);
  }



  private generateHash(content: string): string {
    // Simple hash function for caching
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString(36);
  }

  // Public utility methods
  public async cancelAnalysis(filePath: string): Promise<void> {
    const request = this.pendingRequests.get(filePath);
    if (request) {
      request.controller.abort();
      this.pendingRequests.delete(filePath);
    }
    
    const timer = this.debounceTimers.get(filePath);
    if (timer) {
      clearTimeout(timer);
      this.debounceTimers.delete(filePath);
    }
  }

  public clearCache(): void {
    this.cache.clear();
    console.log('Analysis cache cleared');
  }

  public getCacheStats(): { size: number; oldestEntry: number; newestEntry: number } {
    const entries = Array.from(this.cache.values());
    const timestamps = entries.map(entry => entry.timestamp);
    
    return {
      size: this.cache.size,
      oldestEntry: timestamps.length > 0 ? Math.min(...timestamps) : 0,
      newestEntry: timestamps.length > 0 ? Math.max(...timestamps) : 0,
    };
  }

  public async getAnalysisProgress(analysisId: string): Promise<AnalysisProgress | null> {
    try {
      return await invoke('get_analysis_progress', { analysis_id: analysisId });
    } catch (error) {
      console.error('Failed to get analysis progress:', error);
      return null;
    }
  }

  public async cancelWorkspaceAnalysis(analysisId: string): Promise<boolean> {
    try {
      await invoke('cancel_analysis', { analysis_id: analysisId });
      return true;
    } catch (error) {
      console.error('Failed to cancel analysis:', error);
      return false;
    }
  }

  // New analysis methods for specific types
  public async analyzeCodeStyle(
    code: string,
    filePath: string,
    options: {
      styleRules?: string[];
      autoFix?: boolean;
      signal?: AbortSignal;
    } = {}
  ): Promise<StyleViolation[]> {
    try {
      const request: CodeStyleAnalysisRequest = {
        file_path: filePath,
        content: code,
        style_rules: options.styleRules,
        auto_fix: options.autoFix,
      };

      const result = await invoke('analyze_code_style', request);
      return result.styleViolations || [];
    } catch (error) {
      console.error('Failed to analyze code style:', error);
      // Fallback to basic style analysis
      const diagnostics: Diagnostic[] = [];
      const suggestions: CodeSuggestion[] = [];
      this.checkBasicCodeStyle(code, filePath, suggestions);
      
      return suggestions
        .filter(s => s.category === 'style')
        .map(s => ({
          id: `style-${Date.now()}-${Math.random()}`,
          violationType: 'basic-style-check' as any,
          severity: 'info' as SeverityLevel,
          message: s.message,
          filePath,
          lineRange: [s.range.start.line, s.range.end.line],
          columnRange: [s.range.start.character, s.range.end.character],
          suggestion: s.suggestion,
          autoFixable: !!s.fix_action,
          confidence: s.confidence_score || 0.5,
        }));
    }
  }

  public async analyzeArchitecture(
    workspacePath: string,
    options: {
      includeDependencies?: boolean;
      analysisDepth?: 'shallow' | 'deep';
      signal?: AbortSignal;
    } = {}
  ): Promise<ArchitectureSuggestion[]> {
    try {
      const request: ArchitectureAnalysisRequest = {
        workspace_path: workspacePath,
        include_dependencies: options.includeDependencies,
        analysis_depth: options.analysisDepth,
      };

      const result = await invoke('analyze_architecture', request);
      return result.architectureSuggestions || [];
    } catch (error) {
      console.error('Failed to analyze architecture:', error);
      return [];
    }
  }

  public async getPerformanceHints(
    filePath?: string,
    workspacePath?: string,
    options: {
      includeBenchmarks?: boolean;
      profileMemory?: boolean;
      signal?: AbortSignal;
    } = {}
  ): Promise<PerformanceHint[]> {
    try {
      const request: PerformanceAnalysisRequest = {
        file_path: filePath,
        workspace_path: workspacePath,
        include_benchmarks: options.includeBenchmarks,
        profile_memory: options.profileMemory,
      };

      const result = await invoke('get_performance_hints', request);
      return result.performanceHints || [];
    } catch (error) {
      console.error('Failed to get performance hints:', error);
      return [];
    }
  }

  // Learning system integration methods
  public async recordSuccessfulFix(
    errorPattern: string,
    appliedFix: FixSuggestion,
    success: boolean,
    userFeedback?: 'positive' | 'negative' | 'neutral',
    context?: string
  ): Promise<boolean> {
    if (!this.config.learning_preferences.enable_learning) {
      return false;
    }

    try {
      const request: LearningSystemRequest = {
        error_pattern: errorPattern,
        applied_fix: appliedFix,
        success,
        user_feedback: userFeedback,
        context: context || '',
      };

      await invoke('record_successful_fix', request);
      return true;
    } catch (error) {
      console.error('Failed to record successful fix:', error);
      return false;
    }
  }

  public async getLearnedPatterns(errorContext: string): Promise<LearnedPattern[]> {
    if (!this.config.learning_preferences.enable_learning) {
      return [];
    }

    try {
      return await invoke('get_learned_patterns', { error_context: errorContext });
    } catch (error) {
      console.error('Failed to get learned patterns:', error);
      return [];
    }
  }

  public async updateLearningPreferences(preferences: Partial<LearningPreferences>): Promise<void> {
    this.config.learning_preferences = { ...this.config.learning_preferences, ...preferences };
    
    try {
      await invoke('update_learning_preferences', { preferences: this.config.learning_preferences });
    } catch (error) {
      console.error('Failed to update learning preferences:', error);
      throw error;
    }
  }

  // Compiler integration methods
  public async getCompilerDiagnostics(
    workspacePath: string,
    options: {
      includeExplanations?: boolean;
      includeSuggestedFixes?: boolean;
    } = {}
  ): Promise<{ diagnostics: CompilerDiagnostic[]; explanations: Record<string, ErrorCodeExplanation> }> {
    try {
      const result = await invoke('get_compiler_diagnostics', {
        workspace_path: workspacePath,
        include_explanations: options.includeExplanations ?? true,
        include_suggested_fixes: options.includeSuggestedFixes ?? true,
      });

      return {
        diagnostics: result.diagnostics || [],
        explanations: result.explanations || {},
      };
    } catch (error) {
      console.error('Failed to get compiler diagnostics:', error);
      return { diagnostics: [], explanations: {} };
    }
  }

  public async explainErrorCode(errorCode: string): Promise<ErrorCodeExplanation | null> {
    try {
      return await invoke('explain_error_code', { error_code: errorCode });
    } catch (error) {
      console.error('Failed to explain error code:', error);
      return null;
    }
  }

  // Enhanced utility methods
  private convertRangeFromBackend(lineRange: [number, number], columnRange: [number, number]): Range {
    return {
      start: { line: lineRange[0], character: columnRange[0] },
      end: { line: lineRange[1], character: columnRange[1] }
    };
  }

  private mapSeverityLevelToDiagnostic(severity: SeverityLevel): DiagnosticSeverity {
    switch (severity) {
      case 'critical':
        return DiagnosticSeverity.Error;
      case 'high':
        return DiagnosticSeverity.Warning;
      case 'medium':
        return DiagnosticSeverity.Warning;
      case 'low':
        return DiagnosticSeverity.Information;
      case 'info':
        return DiagnosticSeverity.Information;
      default:
        return DiagnosticSeverity.Hint;
    }
  }

  private mapCompilerLevelToDiagnostic(level: string): DiagnosticSeverity {
    switch (level.toLowerCase()) {
      case 'error':
        return DiagnosticSeverity.Error;
      case 'warning':
        return DiagnosticSeverity.Warning;
      case 'note':
      case 'help':
        return DiagnosticSeverity.Information;
      default:
        return DiagnosticSeverity.Hint;
    }
  }

  // Enhanced fallback analysis methods
  private checkBasicArchitecture(
    code: string,
    filePath: string,
    diagnostics: Diagnostic[],
    suggestions: CodeSuggestion[]
  ): void {
    const lines = code.split('\n');
    
    // Check for potential circular dependencies (basic heuristic)
    const imports = lines
      .filter(line => line.trim().startsWith('import ') || line.trim().startsWith('use '))
      .map(line => line.trim());
    
    if (imports.length > 20) {
      suggestions.push({
        message: `High number of imports detected (${imports.length})`,
        severity: DiagnosticSeverity.Information,
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
        suggestion: 'Consider organizing imports and reducing dependencies',
        category: 'architecture',
        confidence_score: 0.6,
        detailed_explanation: 'Too many imports can indicate tight coupling and make the code harder to maintain.',
      });
    }

    // Check for large files (potential god object)
    const linesOfCode = this.countLinesOfCode(code);
    if (linesOfCode > 500) {
      suggestions.push({
        message: `Large file detected (${linesOfCode} lines)`,
        severity: DiagnosticSeverity.Warning,
        range: { start: { line: 0, character: 0 }, end: { line: lines.length - 1, character: 0 } },
        suggestion: 'Consider breaking this file into smaller, more focused modules',
        category: 'architecture',
        confidence_score: 0.7,
        detailed_explanation: 'Large files can be difficult to maintain and may violate the Single Responsibility Principle.',
      });
    }
  }

  // Enhanced cache management with analysis type support
  public getCacheStatsByType(): Record<AnalysisCategory, { count: number; avgAge: number }> {
    const stats: Record<AnalysisCategory, { count: number; avgAge: number }> = {
      'code-smell': { count: 0, avgAge: 0 },
      'performance': { count: 0, avgAge: 0 },
      'security': { count: 0, avgAge: 0 },
      'style': { count: 0, avgAge: 0 },
      'architecture': { count: 0, avgAge: 0 },
    };

    const now = Date.now();
    const entries = Array.from(this.cache.values());

    entries.forEach(entry => {
      const age = now - entry.timestamp;
      entry.analysisType.forEach(type => {
        if (stats[type]) {
          stats[type].count++;
          stats[type].avgAge = (stats[type].avgAge + age) / 2;
        }
      });
    });

    return stats;
  }

  public clearCacheByType(analysisType: AnalysisCategory): void {
    const keysToDelete: string[] = [];
    
    this.cache.forEach((entry, key) => {
      if (entry.analysisType.includes(analysisType)) {
        keysToDelete.push(key);
      }
    });

    keysToDelete.forEach(key => this.cache.delete(key));
    console.log(`Cleared ${keysToDelete.length} cache entries for analysis type: ${analysisType}`);
  }
}
