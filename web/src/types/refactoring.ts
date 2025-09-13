// Shared refactoring types for frontend integration
// These types align with the backend refactoring API

export type RefactoringType =
  | 'rename'
  | 'extract-function'
  | 'extract-variable'
  | 'extract-class'
  | 'extract-interface'
  | 'inline-function'
  | 'inline-variable'
  | 'inline-method'
  | 'move-method'
  | 'move-class'
  | 'move-file'
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
  | 'pattern-conversion'
  | 'batch-interface-extraction'
  | 'batch-pattern-conversion';

export interface RefactoringContext {
  filePath: string;
  startLine: number;
  startCharacter: number;
  endLine?: number;
  endCharacter?: number;
  symbolName?: string;
  symbolKind?: 'function' | 'variable' | 'class' | 'interface' | 'method';
  selection?: any;
  cursorPosition?: any;
}

export interface RefactoringOptions {
  createBackup?: boolean;
  generateTests?: boolean;
  applyToAllOccurrences?: boolean;
  preserveReferences?: boolean;
  ignoreSafeOperations?: boolean;
  extraOptions?: Record<string, any>;
}

export interface RefactoringResult {
  id: string;
  type: string;
  success: boolean;
  changes: CodeChange[];
  errorMessage?: string;
  error?: RefactoringError;
  timestamp: string;
  duration: number;
  affectedFiles: number;
  metrics?: RefactoringMetrics;
}

export interface RefactoringError {
  code: string;
  message: string;
  details?: string;
  recoverable: boolean;
}

export interface RefactoringMetrics {
  operationsAttempted: number;
  operationsSucceeded: number;
  operationsFailed: number;
  totalBytesChanged: number;
  averageComplexity: number;
}

export interface CodeChange {
  filePath: string;
  range: CodeRange;
  oldText: string;
  newText: string;
  changeType: 'Insertion' | 'Replacement' | 'Deletion';
}

export interface CodeRange {
  startLine: number;
  startCharacter: number;
  endLine: number;
  endCharacter: number;
}

export interface RefactoringAnalysis {
  isSafe: boolean;
  confidenceScore: number;
  potentialImpact: 'Low' | 'Medium' | 'High' | 'Critical';
  affectedFiles: string[];
  affectedSymbols: string[];
  breakingChanges: string[];
  suggestions: string[];
  warnings: string[];
  possibleRefactorings?: string[];
}

export interface AnalysisResult {
  analysisId: string;
  timestamp: string;
  filePath: string;
  symbolInfo?: SymbolAnalysis;
  structuralAnalysis: StructuralAnalysis;
  applicableRefactorings: string[];
  possibleRefactorings?: string[];
  confidenceLevels: Record<string, number>;
  warnings: string[];
  recommendations: string[];
}

export interface SymbolAnalysis {
  name?: string;
  kind: string;
  range: CodeRange;
  references: number;
  canMove: boolean;
  canRename: boolean;
}

export interface StructuralAnalysis {
  complexityScore: number;
  hasComplexFunctions: boolean;
  hasLargeFunctionsCount: number;
  canExtractMethods: boolean;
  canExtractVariables: boolean;
  hasClasses: boolean;
  hasInterfaces: boolean;
}

export interface BackendCapabilities {
  supportedRefactorings: string[];
  supportedFileTypes: string[];
  features: BackendFeatures;
  performanceMetrics: Record<string, number>;
  configurationOptions: string[];
}

export interface BackendFeatures {
  batchOperations: boolean;
  analysis: boolean;
  backupRecovery: boolean;
  testGeneration: boolean;
  aiAnalysis: boolean;
  lspIntegration: boolean;
  gitIntegration: boolean;
  crossLanguageSupport: boolean;
  parallelProcessing: boolean;
}

export interface WizardStep {
  id: string;
  title: string;
  description: string;
  component: React.ComponentType<any>;
  validateStep?: (data: any) => boolean;
}

export interface RefactoringConfiguration {
  enabledRefactorings: RefactoringType[];
  defaultOptions?: {
    createBackup?: boolean;
    previewOnly?: boolean;
    scope?: 'file' | 'module' | 'workspace';
    includeTests?: boolean;
    renameLinkedFiles?: boolean;
    allowIncompleteRefs?: boolean;
  };
  previewBeforeApply?: boolean;
  confirmDestructiveChanges?: boolean;
  maxPreviewLines?: number;
  excludePatterns?: string[];
}

export interface RefactoringWizardData {
  refactoringType: RefactoringType;
  context: RefactoringContext;
  options: RefactoringOptions;
  extraData?: Record<string, any>;
}
