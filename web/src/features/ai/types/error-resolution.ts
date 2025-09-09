// Error Resolution Module
// Comprehensive types and interfaces for error pattern recognition and automated fixing

/**
 * Types of changes that can be applied to resolve errors
 */
export enum ChangeType {
  Insert = 'insert',     // Insert new code/text
  Delete = 'delete',     // Remove code/text
  Replace = 'replace',   // Replace existing code/text
  Move = 'move'          // Move code/text to different location
}

/**
 * Represents a specific change to be applied
 */
export interface CodeChange {
  filePath: string;
  changeType: ChangeType;
  range: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  newText: string;
  oldText?: string;
  description: string;
}

/**
 * Core error pattern representation with matching capabilities
 */
export interface ErrorPattern {
  id: string;
  errorType: string;
  pattern: string | RegExp;
  context: string;
  frequency: number;
  lastSeen: string;
  confidence: number;
  language?: string;
  source?: string;
}

/**
 * Suggested fix with detailed change information
 */
export interface FixSuggestion {
  id: string;
  title: string;
  description: string;
  errorId: string;
  priority: 'low' | 'medium' | 'high' | 'critical';
  fixType: 'quick-fix' | 'refactor' | 'add-missing' | 'remove-unused' | 'type-conversion' | 'pattern-application';
  changes: CodeChange[];
  confidence: number;
  estimatedEffort: 'trivial' | 'low' | 'medium' | 'high';
  benefits: string[];
  risks: string[];
  dependencies?: string[]; // Other fixes that should be applied first
  testSuggestions?: string[];
  documentationLinks?: DocumentationLink[];
}

/**
 * Comprehensive error explanation with context
 */
export interface ErrorCodeExplanation {
  errorCode: string;
  title: string;
  explanation: string;
  severity: 'error' | 'warning' | 'hint' | 'info';
  examples: ErrorCodeExample[];
  documentationLinks: DocumentationLink[];
  relatedErrors: string[];
  quickFixes: string[];
  confidenceScore: number;
}

/**
 * Example showing error usage and solution
 */
export interface ErrorCodeExample {
  title: string;
  before: string;
  after: string;
  explanation: string;
  language: string;
}

/**
 * Documentation link with relevance information
 */
export interface DocumentationLink {
  title: string;
  url: string;
  description: string;
  relevance: 'low' | 'medium' | 'high';
  type: 'official-docs' | 'rust-book' | 'api-reference' | 'examples' | 'tutorial';
  language?: string;
}

/**
 * Enhanced compiler diagnostic information
 */
export interface CompilerDiagnostic {
  level: 'error' | 'warning' | 'note' | 'help';
  message: string;
  source: string;
  code: string;
  filePath: string;
  line: number;
  column: number;
  spans: CompilerDiagnosticSpan[];
  children: CompilerDiagnostic[];
  suggestions: CompilerDiagnosticSuggestion[];
}

/**
 * Diagnostic span with precise location information
 */
export interface CompilerDiagnosticSpan {
  filePath: string;
  lineStart: number;
  lineEnd: number;
  columnStart: number;
  columnEnd: number;
  text: string[];
  label?: string;
  isPrimary: boolean;
}

/**
 * Compiler-provided suggestion for fixing an issue
 */
export interface CompilerDiagnosticSuggestion {
  message: string;
  replacements: CompilerSuggestionReplacement[];
  applicability: 'MachineApplicable' | 'HasPlaceholders' | 'MaybeIncorrect' | 'Unspecified';
}

/**
 * Specific replacement suggested by the compiler
 */
export interface CompilerSuggestionReplacement {
  snippet: string;
  substitution: string;
  span: CompilerDiagnosticSpan;
}

/**
 * Result from error resolution attempt
 */
export interface ErrorResolutionResult {
  success: boolean;
  quickFixes: FixSuggestion[];
  explanations: ErrorCodeExplanation[];
  relatedDocumentation: DocumentationLink[];
  attemptedFixes: AppliedFix[];
  error: string | null;
  metadata: {
    resolutionTime: number;
    patternsMatched: number;
    aiSuggestionsUsed: number;
    confidenceThreshold: number;
  };
}

/**
 * Enhanced result with AI assistance and learning
 */
export interface EnhancedErrorResolutionResult extends ErrorResolutionResult {
  aiGeneratedFixes: FixSuggestion[];
  learnedPatterns: LearnedPattern[];
  compilerDiagnostics: CompilerDiagnostic[];
  confidence: number;
  estimatedSuccessRate: number;
}

/**
 * Tracking of applied fixes for learning purposes
 */
export interface AppliedFix {
  id: string;
  fixSuggestion: FixSuggestion;
  timestamp: string;
  success: boolean;
  feedback?: 'positive' | 'negative' | 'neutral';
  context: string;
  originalError: string;
  userNotes?: string;
}

/**
 * Learned pattern from previous successful fixes
 */
export interface LearnedPattern {
  id: string;
  errorPattern: ErrorPattern;
  successfulFix: FixSuggestion;
  successCount: number;
  failureCount: number;
  confidence: number;
  lastUsed: string;
  userFeedback?: 'positive' | 'negative' | 'neutral';
  context: string;
  performanceMetrics: {
    avgResolutionTime: number;
    firstAttemptSuccessRate: number;
    userSatisfaction: number;
  };
}

/**
 * Learning system request for pattern recording
 */
export interface LearningSystemRequest {
  errorPattern: ErrorPattern;
  appliedFix: FixSuggestion;
  success: boolean;
  userFeedback?: 'positive' | 'negative' | 'neutral';
  context: string;
  performanceData?: {
    resolutionTime: number;
    userAcceptanceSpeed: number;
  };
}

/**
 * Configuration for error resolution behavior
 */
export interface ErrorResolutionConfig {
  confidenceThreshold: number;
  enableAI: boolean;
  enableLearning: boolean;
  maxSuggestions: number;
  includeDocumentation: boolean;
  includeExamples: boolean;
  preferredLanguages: string[];
  excludedPatterns: string[];
  riskTolerance: 'low' | 'medium' | 'high';
}

/**
 * Error resolution request for backend processing
 */
export interface ErrorResolutionRequest {
  filePath: string;
  content: string;
  errors: string[];
  cursorPosition?: [number, number];
  projectContext?: Record<string, string>;
  config: ErrorResolutionConfig;
  useLearnedPatterns: boolean;
  includeCompilerDiagnostics: boolean;
}

/**
 * Pattern matching result with confidence scoring
 */
export interface PatternMatchResult {
  pattern: ErrorPattern;
  confidence: number;
  matches: PatternMatch[];
  suggestedFixes: FixSuggestion[];
}

export interface PatternMatch {
  line: number;
  column: number;
  length: number;
  context: string;
  capturedGroups?: Record<string, string>;
}

/**
 * Error categorization for better pattern organization
 */
export enum ErrorCategory {
  SyntaxError = 'syntax',
  TypeError = 'type',
  OwnershipBorrowing = 'ownership',
  LifetimeError = 'lifetime',
  TraitError = 'trait',
  MacroError = 'macro',
  LinkerError = 'linker',
  PerformanceWarning = 'performance',
  StyleWarning = 'style',
  DeprecatedUsage = 'deprecated',
  UnsafeCode = 'unsafe',
  MissingImport = 'missing_import',
  UnusedCode = 'unused',
  NamingConvention = 'naming',
  Documentation = 'documentation',
  TestingIssue = 'testing',
  ConcurrencyIssue = 'concurrency',
  Generic = 'generic'
}

/**
 * Integration interfaces with existing refactoring system
 */
export interface RefactoringIntegration {
  canApplyPattern(pattern: ErrorPattern, context: any): Promise<boolean>;
  applyChange(change: CodeChange): Promise<{ success: boolean; error?: string }>;
  validateFixConflicts(fixes: FixSuggestion[]): Promise<ValidationResult>;
  calculateImpact(fixes: FixSuggestion[]): Promise<ImpactAnalysis>;
}

export interface ValidationResult {
  valid: boolean;
  conflicts: FixConflict[];
  warnings: string[];
}

export interface FixConflict {
  type: 'overlap' | 'dependency' | 'semantic';
  fixes: string[];
  description: string;
  resolution?: string;
}

export interface ImpactAnalysis {
  filesAffected: string[];
  riskLevel: 'low' | 'medium' | 'high' | 'critical';
  breakingChanges: boolean;
  estimatedLinesChanged: number;
  dependencies: string[];
}