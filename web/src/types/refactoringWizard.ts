// Wizard-specific interfaces for code analysis
export interface ParameterInfo {
  name: string;
  paramType: string;
  isOptional: boolean;
}

export interface MethodInfo {
  name: string;
  signature: string;
  lineNumber: number;
  isAsync: boolean;
  returnType?: string;
  parameters: ParameterInfo[];
}

export interface PropertyInfo {
  name: string;
  propertyType: string;
  lineNumber: number;
  isReadonly: boolean;
}

export interface ClassAnalysisResult {
  className: string;
  publicMethods: MethodInfo[];
  publicProperties: PropertyInfo[];
  complexityScore: number;
  isSuitableForInterface: boolean;
  reasonNotSuitable?: string;
}

export interface InterfaceExtractionAnalysisResponse {
  classes: ClassAnalysisResult[];
  filePath: string;
  totalClasses: number;
  suitableClasses: number;
}

export interface FunctionAnalysisResult {
  functionName: string;
  signature: string;
  lineNumber: number;
  isConvertibleToAsync: boolean;
  canBeAwaited: boolean;
  dependencies: string[];
  blockingOperations: string[];
  estimatedComplexity: string;
}

export interface AsyncConversionAnalysisResponse {
  functions: FunctionAnalysisResult[];
  filePath: string;
  totalFunctions: number;
  convertibleFunctions: number;
}

// Request interfaces for backend calls
export interface InterfaceExtractionRequest {
  filePath: string;
  targetClass?: string;
}

export interface AsyncConversionRequest {
  filePath: string;
  targetFunction?: string;
}

// Wizard state management
export interface ExtractInterfaceWizardState {
  selectedClasses: string[];
  interfaceName: string;
  includePublicMethods: boolean;
  includeProperties: boolean;
  shouldGenerateTests: boolean;
  isAnalysisComplete: boolean;
  analysisResults: ClassAnalysisResult[];
}

export interface AsyncAwaitWizardState {
  conversionTargets: string[];
  selectedTargets: string[];
  customFunctions: string[];
  shouldGenerateTests: boolean;
  analyzeDependencies: boolean;
  isAnalysisComplete: boolean;
  analysisResults: FunctionAnalysisResult[];
}

// Wizard configuration
export interface WizardConfiguration {
  previewBeforeApply: boolean;
  confirmDestructiveChanges: boolean;
  createBackups: boolean;
  defaultScope: 'file' | 'module' | 'workspace';
  includeTests: boolean;
}
