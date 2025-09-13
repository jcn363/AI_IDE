export type RefactoringType =
  | 'rename'
  | 'extract-function'
  | 'extract-variable'
  | 'extract-interface'
  | 'extract-class'
  | 'move-method'
  | 'move-class'
  | 'move-file'
  | 'inline-method'
  | 'inline-function'
  | 'inline-variable'
  | 'remove-parameter'
  | 'introduce-parameter'
  | 'replace-constructor'
  | 'replace-conditionals'
  | 'convert-method-to-function'
  | 'split-class'
  | 'merge-classes'
  | 'change-signature'
  | 'add-delegation'
  | 'remove-delegation'
  | 'encapsulate-field'
  | 'localize-variable'
  | 'add-missing-imports'
  | 'sort-imports'
  | 'convert-to-async'
  | 'generate-getters-setters'
  | 'interface-extraction'
  | 'pattern-conversion'
  | 'batch-interface-extraction'
  | 'async-await-conversion'
  | 'batch-pattern-conversion'
  | 'async-await-pattern-conversion';

export interface RefactoringContext {
  filePath: string;
  startLine: number;
  endLine: number;
  selection?: string;
  language?: string;
  symbols?: Array<{
    name: string;
    type: string;
    start: number;
    end: number;
  }>;
  variables?: Array<{
    name: string;
    type: string;
    scope: string;
  }>;
}

// =============================================================================
// ANALYSIS TYPES
// =============================================================================

// Analysis Categories
export type AnalysisCategory = 'code-smell' | 'performance' | 'security' | 'style' | 'architecture';

// Severity levels
export type SeverityLevel = 'critical' | 'high' | 'medium' | 'low' | 'info' | 'hint';

// Core AI Analysis Configuration
export interface AIAnalysisConfig {
  provider: AIProvider;
  analysis_preferences: AnalysisPreferences;
  enable_real_time: boolean;
  enable_workspace_analysis: boolean;
  max_file_size_kb: number;
  excluded_paths: string[];
}

// AI Provider Configuration
export interface AIProvider {
  OpenAI?: { api_key: string; model: string };
  Local?: { model_path: string };
  Ollama?: { model_name: string; endpoint: string };
}

// Analysis Preferences
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

// Learning Preferences (for the learning system)
export interface LearningPreferences {
  enable_learning: boolean;
  privacy_mode: 'opt-in' | 'opt-out' | 'anonymous';
  share_anonymous_data: boolean;
  retain_personal_data: boolean;
  data_retention_days: number;
  allow_model_training: boolean;
  confidence_threshold_for_learning: number;
}

// Compiler Integration Configuration
export interface CompilerIntegrationConfig {
  enable_compiler_integration: boolean;
  parse_cargo_check_output: boolean;
  enable_error_explanations: boolean;
  enable_suggested_fixes: boolean;
  cache_explanations: boolean;
  explanation_cache_ttl_hours: number;
}

// Analysis Metadata
export interface AnalysisMetadata {
  analysisId: string;
  timestamp: string;
  durationMs: number;
  analyzerVersion: string;
  fileCount: number;
  linesAnalyzed: number;
  errorMessage?: string;
  success?: boolean;
}

// Code Smell Types
export interface CodeSmell {
  id?: string;
  smellType: string;
  severity: SeverityLevel;
  message: string;
  explanation?: string;
  suggestion?: string;
  confidence?: number;
  lineRange: [number, number];
  columnRange: [number, number];
  estimated_fix_time_minutes?: number;
}

// Performance Hint Types
export interface PerformanceHint {
  id?: string;
  hintType: string;
  severity: SeverityLevel;
  message: string;
  optimization?: string;
  estimatedImpact?: string;
  confidence?: number;
  lineRange: [number, number];
  columnRange: [number, number];
}

// Security Issue Types
export interface SecurityIssue {
  id?: string;
  issueType: string;
  severity: SeverityLevel;
  message: string;
  recommendation?: string;
  cweId?: string;
  confidence?: number;
  lineRange: [number, number];
  columnRange: [number, number];
}

// Style Violation Types
export interface StyleViolation {
  id?: string;
  violationType: string;
  severity: SeverityLevel;
  message: string;
  suggestion?: string;
  autoFixable?: boolean;
  confidence?: number;
  lineRange: [number, number];
  columnRange: [number, number];
}

// Architecture Suggestion Types
export interface ArchitectureSuggestion {
  id?: string;
  severity: SeverityLevel;
  message: string;
  recommendation?: string;
  rationale?: string;
  confidence?: number;
}

// Compiler Diagnostic Types
export interface CompilerDiagnostic {
  level: string;
  message: string;
  spans: CompilerDiagnosticSpan[];
  code?: {
    code: string;
    explanation: string;
  };
  children: CompilerDiagnostic[];
}

export interface CompilerDiagnosticSpan {
  fileName: string;
  lineStart: number;
  lineEnd: number;
  columnStart: number;
  columnEnd: number;
  text: string[];
  suggestedReplacement?: string;
  suggestionApplicability?: string;
  isMainSpan?: boolean;
}

// Error Code Explanation Types
export interface ErrorCodeExplanation {
  code: string;
  title: string;
  explanation: string;
  examples?: string[];
  references?: string[];
}

// Fix Suggestion Types
export interface FixSuggestion {
  id: string;
  title: string;
  description: string;
  changes: Array<{
    file_path: string;
    range: [number, number, number, number]; // [start_line, start_col, end_line, end_col]
    new_text: string;
    old_text?: string;
  }>;
  category?: string;
  confidence?: number;
}

// Learned Pattern Types
export interface LearnedPattern {
  id: string;
  errorPattern: string;
  successfulFix: FixSuggestion;
  confidence: number;
  successCount: number;
  failureCount: number;
  lastUsed: string;
}

// Additional analysis result types from other files
export interface CombinedAnalysisResult {
  code: string;
  features: string[];
  suggestions: string[];
  confidence: number;
}

export interface GeneratedCode {
  code: string;
  explanation?: string;
  confidence?: number;
}

export interface ArchitecturalRecommendation {
  type: 'structure' | 'pattern' | 'principle';
  title: string;
  description: string;
  priority: 'high' | 'medium' | 'low';
  rationale: string;
  implementation: string[];
}

export interface ArchitecturalDecision {
  id: string;
  title: string;
  context: string;
  decision: string;
  consequences: string[];
  alternatives?: string[];
  status: 'proposed' | 'accepted' | 'rejected' | 'deprecated';
}

export interface FineTuneJob {
  id: string;
  status: TrainingStatus;
  progress: TrainingProgress;
  config: {
    model: string;
    trainingData: string;
    hyperparameters: Record<string, any>;
  };
}

export type TrainingStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface TrainingProgress {
  epoch: number;
  total_epochs: number;
  loss: number;
  accuracy?: number;
  eta?: string;
}

// Additional service types
export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp?: string;
}

export interface CodeAnalysisRequest {
  code: string;
  language?: string;
  filePath?: string;
  AnalysisConfig?: AIAnalysisConfig;
}

// Error Resolution Module Exports - Only truly new types
export {
  ChangeType,
  type CodeChange,
  type ErrorPattern,
  type AppliedFix,
  type LearningSystemRequest,
  type ErrorResolutionConfig,
  type PatternMatchResult,
  type PatternMatch,
  ErrorCategory,
  type RefactoringIntegration,
  type ValidationResult,
  type FixConflict,
  type ImpactAnalysis,
} from './error-resolution';
