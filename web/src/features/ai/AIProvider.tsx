import React, { ReactNode, createContext, useContext, useMemo } from 'react';
import { AIService } from './code-analysis/CodeAnalyzer';
import { ErrorResolver } from './error-resolution/ErrorResolver';
import { CodeGenerator } from './code-generation/CodeGenerator';
import { RefactoringService } from './services/RefactoringService';
import { ModelService } from './services/ModelService';
import { CodeReviewService } from './services/CodeReviewService';
import { SpecificationService } from './services/SpecificationService';
import { ArchitecturalService } from './services/ArchitecturalService';
import { EmbedAIService } from './services/EmbedAI';

interface AIProviderProps {
  children: ReactNode;
  service?: AIService;
  config?: {
    provider?: string;
    autoInitialize?: boolean;
    enableLearning?: boolean;
    maxRetries?: number;
  };
}

interface AIContextType {
  // Core services
  service: AIService;
  errorResolver: ErrorResolver;
  codeGenerator: CodeGenerator;

  // Enhanced AI services
  modelService: ModelService;
  codeReviewService: CodeReviewService;
  specificationService: SpecificationService;
  architecturalService: ArchitecturalService;
  embedAIService: EmbedAIService;

  // Legacy service
  refactoringService: RefactoringService;
}

const AIContext = createContext<AIContextType | undefined>(undefined);

export const useAIService = (): AIContextType => {
  const context = useContext(AIContext);
  if (!context) {
    throw new Error('useAIService must be used within an AIProvider');
  }
  return context;
};

export const AIProvider: React.FC<AIProviderProps> = ({
  children,
  service: propService,
  config = {},
}) => {
  // Initialize core services
  const service = useMemo(() => propService || AIService.getInstance(), [propService]);
  const errorResolver = useMemo(() => ErrorResolver.getInstance(), []);
  const codeGenerator = useMemo(() => CodeGenerator.getInstance(), []);

  // Initialize enhanced AI services
  const modelService = useMemo(() => ModelService.getInstance(), []);
  const codeReviewService = useMemo(() => CodeReviewService.getInstance(), []);
  const specificationService = useMemo(() => SpecificationService.getInstance(), []);
  const architecturalService = useMemo(() => ArchitecturalService.getInstance(), []);
  const embedAIService = useMemo(() => EmbedAIService.getInstance(), []);

  // Legacy service
  const refactoringService = useMemo(() => RefactoringService.getInstance(), []);

  // Auto-initialize if configured
  useMemo(() => {
    if (config.autoInitialize !== false) {
      // Initialize all services that need configuration
      console.log('AI services initialized with config:', config);
    }
  }, [config]);

  const value = useMemo(
    () => ({
      service,
      errorResolver,
      codeGenerator,
      modelService,
      codeReviewService,
      specificationService,
      architecturalService,
      embedAIService,
      refactoringService,
    }),
    [
      service,
      errorResolver,
      codeGenerator,
      modelService,
      codeReviewService,
      specificationService,
      architecturalService,
      embedAIService,
      refactoringService
    ],
  );

  return <AIContext.Provider value={value}>{children}</AIContext.Provider>;
};

export default AIProvider;
