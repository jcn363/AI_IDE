import {
  AnalysisConfiguration,
  ArchitectureSuggestion,
  CodeSmell,
  EnhancedCodeAnalysisResult,
  LearnedPattern,
  PerformanceHint,
  SecurityIssue,
  SeverityLevel,
  StyleViolation,
} from "../types";
import type { CodeAction } from "vscode-languageserver";
import { CodeActionKind } from "vscode-languageserver";

export const sampleAnalysisConfig: AnalysisConfiguration = {
  enabledCategories: [
    "code-smell",
    "performance",
    "security",
    "style",
    "architecture",
  ],
  severityThreshold: "warning",
  realTimeAnalysis: true,
  analysisOnSave: true,
  maxSuggestions: 50,
  aiProvider: {
    type: "mock",
  },
  analysisPreferences: {
    enableCodeSmells: true,
    enableSecurity: true,
    enablePerformance: true,
    enableCodeStyle: true,
    enableArchitecture: true,
    enableLearning: true,
    confidenceThreshold: 0.7,
    timeoutSeconds: 30,
    includeExplanations: true,
    includeExamples: false,
    privacyMode: "opt-in",
  },
  learningPreferences: {
    enableLearning: true,
    privacyMode: "opt-out",
    shareAnonymousData: true,
    retainPersonalData: false,
    dataRetentionDays: 365,
    allowModelTraining: true,
  },
  confidenceThreshold: 0.8,
  excludePatterns: ["**/node_modules/**", "**/dist/**", "**/.git/**"],
  maxFileSizeKb: 1024,
  timeoutSeconds: 60,
  customRules: [],
  performance: {
    enableBenchmarking: true,
    profileMemoryUsage: false,
    detectHotPaths: true,
    enablePerformanceHints: true,
    enableOptimizationSuggestions: true,
  },
  security: {
    enableVulnerabilityScanning: true,
    checkDependencies: true,
    scanForSecrets: false,
    enableSecurityIssueDetection: true,
    includeCweMapping: true,
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
    checkFormattingConsistency: false,
    enforceRustIdioms: true,
    requireDocumentation: true,
  },
  learning: {
    enableLearningSystem: true,
    recordSuccessfulFixes: true,
    useLearnedPatterns: true,
    shareAnonymousData: true,
    confidenceThresholdForLearning: 0.85,
  },
  compiler: {
    enableCompilerIntegration: true,
    parseCargoCheckOutput: true,
    enableErrorExplanations: true,
    enableSuggestedFixes: true,
    cacheExplanations: true,
  },
};

export const sampleFixSuggestion: CodeAction = {
  title: "Fix issue",
  kind: CodeActionKind.QuickFix,
  edit: {
    changes: {},
  },
};

export const sampleCodeSmell: CodeSmell = {
  id: "cs-001",
  smellType: "long-method",
  severity: "warning" as SeverityLevel,
  message: "Method is too long and should be refactored",
  filePath: "src/main.rs",
  lineRange: [10, 50] as [number, number],
  columnRange: [0, 80] as [number, number],
  suggestion: "Consider breaking this method into smaller functions",
  confidence: 0.85,
  explanation: "Long methods are harder to understand and maintain",
  examples: ["fn process_data() { /* 40+ lines */ }"],
};

export const sampleStyleViolation: StyleViolation = {
  id: "sv-001",
  violationType: "naming-convention",
  severity: "warning" as SeverityLevel,
  message: "Variable name should use snake_case",
  filePath: "src/lib.rs",
  lineRange: [5, 5] as [number, number],
  columnRange: [8, 15] as [number, number],
  suggestion: 'Rename "myVar" to "my_var"',
  autoFixable: true,
  confidence: 0.95,
};

export const sampleLearnedPattern: LearnedPattern = {
  id: "learned-pattern-1",
  errorPattern: {
    id: "console-log-pattern",
    errorType: "console-statement",
    pattern: "console\\.(log|warn|error|info)\\s*\\(",
    context: "JavaScript/TypeScript production code",
    frequency: 5,
    lastSeen: new Date().toISOString(),
    confidence: 0.9,
  },
  successfulFix: {
    id: "fix-1",
    title: "Replace console.log with logger",
    description: "Replace console.log with a proper logger instance",
    fixType: "quick-fix",
    changes: [],
    confidence: 0.9,
    estimatedEffort: "low",
    benefits: ["Better logging infrastructure", "Consistent logging format"],
    risks: ["Potential performance impact"],
  },
  successCount: 5,
  failureCount: 0,
  confidence: 0.9,
  lastUsed: new Date().toISOString(),
  userFeedback: null,
  context: "Replace console statements with proper logging",
};

export const sampleEnhancedAnalysisResult: EnhancedCodeAnalysisResult = {
  codeSmells: [sampleCodeSmell],
  securityIssues: [],
  performanceHints: [],
  styleViolations: [sampleStyleViolation],
  architectureSuggestions: [],
  qualityScore: 85,
  metadata: {
    analysisId: "analysis-123",
    timestamp: new Date().toISOString(),
    durationMs: 1250,
    analyzerVersion: "1.0.0",
    fileCount: 24,
    linesAnalyzed: 1000,
  },
};
