// Model management types
export type ModelType = 'CodeLlama' | 'StarCoder' | 'Custom';
export type ModelSize = 'Small' | 'Medium' | 'Large';
export type Quantization = 'None' | 'Int8' | 'Int4' | 'GPTQ';
export type DeviceType = 'Cpu' | 'Cuda' | 'Auto';

// Quantization types
export type QuantizationPrecision = 'Int4' | 'Int8' | 'GPTQ' | 'NF4' | 'FP16';

export interface QuantizationConfig {
  targetPrecision: QuantizationPrecision;
  calibrationDataset?: string;
  calibrationSamples?: number;
  symmetric: boolean;
  blockSize?: number;
  optimizeForMemory: boolean;
  optimizeForSpeed: boolean;
  preserveQuality: boolean;
  awqConfig?: {
    groupSize: number;
    bits: number;
    zeroPoint: boolean;
  };
  gptqConfig?: {
    bits: number;
    groupSize: number;
    dampPercent: number;
    descAct: boolean;
    sym: boolean;
  };
}

export interface QuantizationOption {
  precision: QuantizationPrecision;
  supported: boolean;
  memoryReduction: number;
  speedImprovement: number;
  qualityImpact: 'low' | 'medium' | 'high';
  recommended: boolean;
  requirements?: string[];
  description?: string;
}

export interface ValidationResult {
  isValid: boolean;
  warnings: string[];
  errors: string[];
  memoryEstimates?: {
    original: number;
    quantized: number;
    reductionPercent: number;
  };
  performanceEstimates?: {
    originalLatency: number;
    quantizedLatency: number;
    improvementPercent: number;
  };
}

export type ModelStatus =
  | 'Available'
  | 'Downloading'
  | 'Downloaded'
  | 'Loading'
  | 'Loaded'
  | 'Unloading'
  | 'Error';

export type TrainingStatus =
  | 'Created'
  | 'Initializing'
  | 'PreparingData'
  | 'Training'
  | 'Evaluating'
  | 'Saving'
  | 'Completed'
  | 'Failed'
  | 'Cancelled'
  | 'Paused';

export interface ModelInfo {
  id: string;
  modelType: ModelType;
  modelSize: ModelSize;
  modelPath?: string;
  quantization?: Quantization;
  loraAdapters?: string[];
  status: ModelStatus;
  memoryUsageMB?: number;
  isLoaded: boolean;
  supportsFineTuning: boolean;
  lastUsedAt?: string;
  createdAt?: string;
}

// Dataset and training types
export interface DatasetFilters {
  minFileSize: number;
  maxFileSize: number;
  allowedExtensions: string[];
  qualityThreshold: number;
  includeTests: boolean;
  maxSamples?: number;
}

export interface DatasetPreparationRequest {
  sourcePaths: string[];
  outputPath: string;
  taskType: 'CodeCompletion' | 'ErrorCorrection' | 'Documentation';
  filters: DatasetFilters;
}

export interface TrainingConfigInfo {
  learningRate: number;
  batchSize: number;
  maxEpochs: number;
  loraRank?: number;
  mixedPrecision: boolean;
  maxSeqLength: number;
  datasetSize?: number;
}

export interface FineTuningRequest {
  jobName: string;
  description?: string;
  baseModel: string;
  datasetPath: string;
  config: TrainingConfigInfo;
  outputPath?: string;
  enableMonitoring: boolean;
}

export interface TrainingProgress {
  epoch: number;
  totalEpochs: number;
  step: number;
  totalSteps: number;
  loss?: number;
  learningRate?: number;
  estimatedTimeRemaining?: number;
  memoryUsageMb?: number;
  gpuUtilization?: number;
}

export interface TrainingMetrics {
  finalLoss: number;
  trainingTimeSeconds: number;
  peakMemoryUsageMb: number;
  samplesPerSecond: number;
  validationLoss?: number;
  perplexity?: number;
  bleuScore?: number;
  codeBleuScore?: number;
}

export interface FineTuneJobInfo {
  jobId: string;
  name: string;
  description?: string;
  baseModel: string;
  modelType: ModelType;
  status: TrainingStatus;
  progress: TrainingProgress;
  createdAt: string;
  updatedAt: string;
  outputPath?: string;
  metrics?: TrainingMetrics;
  errorMessage?: string;
  config: TrainingConfigInfo;
}

// Model management request types
export interface ModelLoadingRequest {
  model_path: string;
  model_type: ModelType;
  model_size: ModelSize;
  quantization?: Quantization;
  lora_adapters: string[];
  device: DeviceType;
  endpoint?: string;
}

// Refactoring types
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
  startCharacter: number;
  endLine: number;
  endCharacter: number;
  selectedText?: string;
  symbolName?: string;
  selection?: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  cursorPosition?: { line: number; character: number };
  usages?: any[]; // For tracking multiple usages of a symbol
}

export interface RefactoringOptions {
  enableBackup: boolean;
  generateTests: boolean;
  applyToAllOccurrences: boolean;
  preserveReferences: boolean;
  refactoringType?: 'single' | 'batch' | 'interface' | 'pattern';
}

export interface RefactoringAnalysis {
  possibleRefactorings: RefactoringType[];
  confidence: number;
  impact: 'low' | 'medium' | 'high';
  affectedFiles: string[];
  risks: string[];
  suggestions: string[];
  isSafe?: boolean;
  warnings?: string[];
  conflicts?: any[];
  dependencies?: any[];
  preview?: {
    before: string;
    after: string;
    changes: any[];
  };
}

export interface RefactoringResult {
  id: string;
  type: RefactoringType;
  target: RefactoringTarget;
  original: RefactoringContext;
  changes: CodeChange[];
  analysis: RefactoringAnalysis;
  success: boolean;
  timestamp: string;
  errorMessage?: string;
  duration?: number;
  affectedFiles?: number;
}

export interface RefactoringTarget {
  type: string;
  name: string;
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  analysis: {
    isSafe: boolean;
    warnings: string[];
    conflicts: any[];
    dependencies: any[];
    preview: {
      before: string;
      after: string;
      changes: any[];
    };
  };
}

export interface CodeChange {
  filePath: string;
  range: {
    startLine: number;
    startCharacter: number;
    endLine: number;
    endCharacter: number;
  };
  oldText: string;
  newText: string;
  changeType?: 'Insertion' | 'Replacement' | 'Deletion';
  newContent?: string;
}

export interface LSPRefactoringRequest {
  textDocument: { uri: string };
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  refactoringKind: RefactoringType;
  params: any;
}

export interface LSPRefactoringResponse {
  success: boolean;
  edits?: TextEdit[];
  workspaceEdit?: WorkspaceEdit;
  changes?: Record<string, TextEdit[]>;
  documentChanges?: DocumentChange[];
  error?: string;
}

export interface TextEdit {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  newText: string;
}

export interface WorkspaceEdit {
  changes?: { [uri: string]: TextEdit[] };
  documentChanges?: DocumentChange[];
}

export interface DocumentChange {
  textDocument: { uri: string; version?: number };
  edits: TextEdit[];
}

export interface RefactoringConfiguration {
  enabledRefactorings: RefactoringType[];
  defaultOptions: {
    createBackup: boolean;
    previewOnly: boolean;
    scope: 'file' | 'module' | 'workspace';
    includeTests: boolean;
    renameLinkedFiles: boolean;
    allowIncompleteRefs: boolean;
  };
  previewBeforeApply?: boolean; // Made optional since UI might add it
  confirmDestructiveChanges?: boolean; // Made optional since UI might add it
  maxPreviewLines?: number; // Made optional since UI might add it
  excludePatterns?: string[]; // Made optional since UI might add it
}

// Model management request types
export interface ModelLoadingRequest {
  modelPath: string;
  modelType: ModelType;
  modelSize: ModelSize;
  quantization?: Quantization;
  loraAdapters: string[];
  device: DeviceType;
  endpoint?: string;
}

export interface PerformanceMetrics {
  cacheHitRatio: number;
  backendResponseTime: number;
  operationCount: number;
  lastUpdated: string;
}

export interface ResourceStatus {
  memoryUsageGb: number;
  memoryLimitGb: number;
  gpuUsagePercent: number;
  gpuMemoryUsageGb: number;
  gpuMemoryLimitGb: number;
  activeJobs: number;
  availableModels: number;
  systemLoad: number;
}

// AI Analysis types
export type AnalysisCategory =
  | 'syntax-error'
  | 'type-error'
  | 'logic-error'
  | 'performance-issue'
  | 'security-issue'
  | 'code-smell'
  | 'best-practice-violation'
  | 'documentation-issue'
  | 'test-coverage'
  | 'dependency-issue'
  | 'refactoring-opportunity'
  | 'compatibility-issue';

export type SeverityLevel = 'info' | 'warning' | 'error' | 'critical';

export interface AnalysisResult {
  filePath: string;
  line: number;
  column: number;
  length: number;
  category: AnalysisCategory;
  severity: SeverityLevel;
  message: string;
  suggestion?: string;
  code?: string;
  ruleId?: string;
  fixable?: boolean;
  confidence?: number;
  context?: {
    surroundingLines?: string[];
    variables?: string[];
    functions?: string[];
    imports?: string[];
  };
}

export interface AnalysisRequest {
  files: string[];
  categories?: AnalysisCategory[];
  includeTests?: boolean;
  maxIssues?: number;
  minConfidence?: number;
  targetDirectory?: string;
}

export interface AnalysisProgress {
  totalFiles: number;
  processedFiles: number;
  currentFile?: string;
  issuesFound: number;
  estimatedTimeRemaining?: number;
}

// Code generation types
export type CodeActionType =
  | 'complete-code'
  | 'fix-error'
  | 'add-tests'
  | 'optimize-performance'
  | 'improve-readability'
  | 'add-documentation'
  | 'refactor-code'
  | 'generate-function'
  | 'generate-class'
  | 'generate-interface'
  | 'generate-tests';

export interface CodeGenerationRequest {
  prompt: string;
  context: {
    filePath: string;
    line: number;
    column: number;
    selectedText?: string;
    language: string;
    framework?: string;
    dependencies?: string[];
  };
  actionType?: CodeActionType;
  options: {
    maxLength?: number;
    temperature?: number;
    includeComments?: boolean;
    includeTests?: boolean;
  };
}

export interface CodeGenerationResponse {
  code: string;
  explanation?: string;
  language: string;
  confidence: number;
  metadata?: {
    complexity?: number;
    testability?: number;
    maintainability?: number;
    dependencies?: string[];
    imports?: string[];
  };
}

// Pattern analysis types
export type PatternType =
  | 'algorithmic'
  | 'architectural'
  | 'design'
  | 'coding-style'
  | 'naming-convention'
  | 'error-handling'
  | 'concurrency'
  | 'memory-management'
  | 'performance'
  | 'security'
  | 'testing'
  | 'documentation';

export interface PatternMatch {
  pattern: string;
  type: PatternType;
  confidence: number;
  matches: Array<{
    file: string;
    line: number;
    context: string;
  }>;
  suggestion?: string;
  impact: 'low' | 'medium' | 'high';
}

// Model inference types
export interface InferenceRequest {
  modelId: string;
  input: string;
  maxTokens?: number;
  temperature?: number;
  stopSequences?: string[];
  context?: Record<string, any>;
}

export interface InferenceResponse {
  output: string;
  confidence?: number;
  tokensUsed?: number;
  metadata?: Record<string, any>;
  error?: string;
}

// Error correction types
export type ErrorCorrectionType =
  | 'syntax-error'
  | 'type-error'
  | 'logic-error'
  | 'runtime-error'
  | 'compilation-error'
  | 'import-error'
  | 'lint-error';

export interface ErrorCorrectionRequest {
  error: {
    type: ErrorCorrectionType;
    message: string;
    stackTrace?: string;
    file?: string;
    line?: number;
    column?: number;
  };
  context: {
    code: string;
    language: string;
    surroundingContext?: string;
    recentChanges?: string[];
  };
}

export interface ErrorCorrectionSuggestion {
  type: 'fix' | 'explanation' | 'refactor';
  description: string;
  codeChanges?: Array<{
    file: string;
    line: number;
    oldCode: string;
    newCode: string;
  }>;
  confidence: number;
  additionalContext?: string;
}

// Documentation generation types
export type DocumentationType =
  | 'function-doc'
  | 'class-doc'
  | 'module-doc'
  | 'api-doc'
  | 'readme'
  | 'changelog'
  | 'tutorial'
  | 'architecture-doc';

export interface DocumentationRequest {
  code: string;
  language: string;
  type: DocumentationType;
  context?: {
    projectName?: string;
    existingDocs?: string;
    targetAudience?: string;
    technicalLevel?: 'beginner' | 'intermediate' | 'advanced';
  };
}

// Refactoring suggestion types
export interface RefactoringSuggestion {
  id: string;
  type: RefactoringType;
  description: string;
  rationale: string;
  impact: {
    complexity: 'increase' | 'decrease' | 'maintain';
    maintainability: 'improve' | 'worsen' | 'neutral';
    performance?: 'improve' | 'worsen' | 'neutral';
  };
  changes: Array<{
    file: string;
    line?: number;
    oldCode?: string;
    newCode: string;
    explanation: string;
  }>;
  confidence: number;
  risks?: string[];
  alternatives?: string[];
}

// AI service status types
export type AIServiceStatus =
  | 'starting'
  | 'ready'
  | 'busy'
  | 'error'
  | 'updating'
  | 'maintenance'
  | 'offline';

export interface AIServiceInfo {
  name: string;
  status: AIServiceStatus;
  version: string;
  capabilities: string[];
  queueLength?: number;
  lastHeartbeat?: string;
  performanceMetrics?: {
    requestsPerMinute: number;
    averageLatencyMs: number;
    successRate: number;
    errorRate: number;
  };
}

// Learning and adaptation types
export interface LearningData {
  userId?: string;
  action: string;
  context: Record<string, any>;
  outcome: 'success' | 'failure' | 'partial';
  duration?: number;
  metadata?: Record<string, any>;
}

export interface ModelImprovementFeedback {
  suggestionId: string;
  action: 'accept' | 'reject' | 'modify';
  originalSuggestion: string;
  modifiedSuggestion?: string;
  reason?: string;
  qualityRating?: number;
}

// Collaboration types
export interface AICollaborator {
  id: string;
  name: string;
  role: string;
  capabilities: string[];
  status: AIServiceStatus;
  preferences?: {
    codingStyle?: string;
    language?: string;
    framework?: string;
  };
}

export interface CollaborationSession {
  id: string;
  collaborators: AICollaborator[];
  context: {
    project: string;
    files: string[];
    objectives: string[];
  };
  startTime: string;
  activeTask?: string;
  messages?: Array<{
    timestamp: string;
    sender: string;
    type: 'suggestion' | 'question' | 'decision' | 'result';
    content: string;
  }>;
}

// Code review types
export type ReviewCategory =
  | 'security'
  | 'performance'
  | 'maintainability'
  | 'readability'
  | 'architecture'
  | 'best-practices'
  | 'testing'
  | 'documentation';

export interface CodeReview {
  file: string;
  reviewer: string;
  category: ReviewCategory;
  severity: SeverityLevel;
  line?: number;
  comment: string;
  suggestion?: string;
  referencedCode?: string;
  confidence: number;
  timestamp: string;
}

// Performance analysis types
export interface PerformanceAnalysis {
  file: string;
  function?: string;
  complexity: {
    cognitive: number;
    cyclomatic: number;
    maintainability_index: number;
  };
  issues: Array<{
    type: 'complexity' | 'loops' | 'recursion' | 'memory' | 'algorithm';
    severity: SeverityLevel;
    location: { line: number; column: number };
    suggestion: string;
  }>;
  recommendations: string[];
}

// Security analysis types
export type SecurityIssueType =
  | 'injection'
  | 'authentication'
  | 'authorization'
  | 'cryptography'
  | 'data-validation'
  | 'input-validation'
  | 'session-management'
  | 'access-control'
  | 'configuration'
  | 'error-handling';

export interface SecurityIssue {
  type: SecurityIssueType;
  severity: SeverityLevel;
  file: string;
  line?: number;
  description: string;
  vulnerableCode: string;
  remediation: string;
  cwe?: string;
  confidence: number;
  exploitability?: 'low' | 'medium' | 'high';
}

// Testing assistance types
export type TestType =
  | 'unit'
  | 'integration'
  | 'e2e'
  | 'performance'
  | 'load'
  | 'stress'
  | 'smoke'
  | 'regression'
  | 'security';

export interface TestGenerationRequest {
  code: string;
  language: string;
  framework?: string;
  testType: TestType;
  coverage?: number;
  includeEdgeCases?: boolean;
  includeMocking?: boolean;
}

// AI configuration types
export interface AIConfiguration {
  provider: 'openai' | 'anthropic' | 'local' | 'custom';
  model: string;
  temperature: number;
  maxTokens: number;
  apiKey?: string;
  endpoint?: string;
  timeout?: number;
  retryAttempts?: number;
}

export interface AICapabilities {
  codeGeneration: boolean;
  codeAnalysis: boolean;
  refactoring: boolean;
  documentation: boolean;
  testing: boolean;
  errorCorrection: boolean;
  performance: boolean;
  security: boolean;
  collaboration: boolean;
}

// Model management enhanced types
export interface ModelCapability {
  name: string;
  description: string;
  required: boolean;
  confidence?: number;
}

export interface ModelEnvironment {
  os: string;
  architecture: string;
  pythonVersion: string;
  gpuAvailable: boolean;
  gpuMemory: number;
  ram: number;
  diskSpace: number;
}

export interface ModelDeployment {
  modelId: string;
  instances: number;
  strategy: 'single' | 'load-balanced' | 'auto-scaling';
  environment: ModelEnvironment;
  monitoring: {
    metrics: string[];
    alerts: string[];
    logLevel: 'debug' | 'info' | 'warning' | 'error';
  };
}

// Workflow management types
export interface AIWorkflow {
  id: string;
  name: string;
  description?: string;
  steps: Array<{
    id: string;
    type: 'analysis' | 'generation' | 'refactoring' | 'testing' | 'review';
    config: any;
    dependencies?: string[];
    timeout?: number;
  }>;
  triggers: Array<{
    type: 'manual' | 'on-commit' | 'on-pull-request' | 'scheduled' | 'on-file-change';
    config: any;
  }>;
  createdAt: string;
  updatedAt: string;
  enabled: boolean;
}

// Continuous learning types
export interface LearningPattern {
  pattern: string;
  frequency: number;
  success: number;
  failures: number;
  avgConfidence: number;
  lastUpdated: string;
  contexts: string[];
}

export interface KnowledgeBase {
  concepts: Record<string, any>;
  patterns: LearningPattern[];
  preferences: Record<string, any>;
  metrics: {
    totalInferences: number;
    successfulSuggestions: number;
    rejectedSuggestions: number;
    learningRate: number;
  };
}

// UI/UX assistance types
export interface UXSuggestion {
  type: 'layout' | 'navigation' | 'accessibility' | 'usability' | 'visual';
  component?: string;
  location?: string;
  description: string;
  implementation: string;
  impact: 'low' | 'medium' | 'high';
  confidence: number;
}

// Metrics and analytics types
export interface AIMetrics {
  requests: number;
  responses: number;
  avgResponseTime: number;
  errorRate: number;
  userSatisfaction?: number;
  codeQualityImprovement?: number;
  productivityGain?: number;
  costEfficiency?: number;
  timeSaved?: number;
}

// Validation and quality assurance types
export interface QualityGate {
  name: string;
  metric: string;
  operator: 'lt' | 'lte' | 'eq' | 'gte' | 'gt';
  value: number;
  required: boolean;
}

export interface ValidationResult {
  passed: boolean;
  score: number;
  gates: Array<{
    gate: string;
    passed: boolean;
    actualValue?: number;
    requiredValue?: number;
  }>;
  recommendations?: string[];
}

// Advanced refactoring types
export interface BatchRefactoring {
  operations: RefactoringSuggestion[];
  order: string[];
  conflicts: Array<{
    operation1: string;
    operation2: string;
    type: 'structural' | 'semantic' | 'dependency';
    resolution: string;
  }>;
  preview: string;
}

export interface InteractiveRefactoring {
  sessionId: string;
  state: 'planning' | 'previewing' | 'accepting' | 'refining';
  operations: RefactoringSuggestion[];
  userFeedback: Array<{
    operationId: string;
    feedback: 'approve' | 'reject' | 'modify';
    comments?: string;
  }>;
}

// Context-aware assistance types
export interface DevelopmentContext {
  user: {
    experience: 'beginner' | 'intermediate' | 'expert';
    domain: string[];
    preferences: Record<string, any>;
  };
  project: {
    type: string;
    size: 'small' | 'medium' | 'large';
    complexity: number;
    technologies: string[];
    standards: string[];
  };
  session: {
    duration: number;
    filesEdited: string[];
    patterns: string[];
    currentTask?: string;
  };
}

// Intelligent task planning types
export interface Task {
  id: string;
  description: string;
  type: string;
  complexity: number;
  dependencies: string[];
  estimatedTime: number;
  requiredSkills: string[];
}

export interface TaskPlan {
  goal: string;
  tasks: Task[];
  timeline: Array<{
    phase: string;
    tasks: string[];
    duration: number;
  }>;
  risks: string[];
  contingencies: string[];
}

// Adaptive recommendations types
export interface Recommendation {
  id: string;
  type: string;
  content: string;
  context: string[];
  effectiveness: number;
  confidence: number;
  reasoning: string;
  alternatives?: Recommendation[];
}

export interface UserProfile {
  userId: string;
  preferences: Record<string, any>;
  history: Array<{
    action: string;
    feedback: 'positive' | 'negative' | 'neutral';
    timestamp: string;
  }>;
  patterns: LearningPattern[];
}

// Export all types to make them available across the codebase
export type AIAnalysisResult = AnalysisResult;
export type AICodeGenerationRequest = CodeGenerationRequest;
export type AICodeGenerationResponse = CodeGenerationResponse;
export type AIPatternMatch = PatternMatch;
export type AIInferenceRequest = InferenceRequest;
export type AIInferenceResponse = InferenceResponse;
export type AIErrorCorrectionRequest = ErrorCorrectionRequest;
export type AIErrorCorrectionSuggestion = ErrorCorrectionSuggestion;
export type AIDocumentationRequest = DocumentationRequest;
export type AIRefactoringSuggestion = RefactoringSuggestion;
export type AIAIServiceInfo = AIServiceInfo;
export type AILearningData = LearningData;
export type AIModelImprovementFeedback = ModelImprovementFeedback;
export type AIAICollaborator = AICollaborator;
export type AICollaborationSession = CollaborationSession;
export type AICodeReview = CodeReview;
export type AIPerformanceAnalysis = PerformanceAnalysis;
export type AISecurityIssue = SecurityIssue;
export type AITestGenerationRequest = TestGenerationRequest;
export type AIAIConfiguration = AIConfiguration;
export type AIAICapabilities = AICapabilities;
export type AIModelCapability = ModelCapability;
export type AIModelEnvironment = ModelEnvironment;
export type AIModelDeployment = ModelDeployment;
export type AIAIWorkflow = AIWorkflow;
export type AILearningPattern = LearningPattern;
export type AIKnowledgeBase = KnowledgeBase;
export type AIUXSuggestion = UXSuggestion;
export type AIAIMetrics = AIMetrics;
export type AIQualityGate = QualityGate;
export type AIValidationResult = ValidationResult;
export type AIBatchRefactoring = BatchRefactoring;
export type AIInteractiveRefactoring = InteractiveRefactoring;
export type AIDevelopmentContext = DevelopmentContext;
export type AITask = Task;
export type AITaskPlan = TaskPlan;
export type AIRecommendation = Recommendation;
export type AIUserProfile = UserProfile;

// Missing exports for backward compatibility
export type AnalysisConfiguration = AIConfiguration;
export type CodeGenerationOptions = CodeGenerationRequest['options'];
export type GeneratedCode = CodeGenerationResponse;
export type EnhancedCodeAnalysisResult = AnalysisResult;
