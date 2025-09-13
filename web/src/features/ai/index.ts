// =============================================================================
// AI Feature Module Exports
// =============================================================================

// Core Provider and Context
export { AIProvider, useAIService } from './AIProvider';

// =============================================================================
// COMPONENTS - NEWLY IMPLEMENTED AI FEATURES
// =============================================================================

// Core AI Feature Panels
export { FineTuningPanel } from './components/FineTuningPanel';
export { ArchitecturalAdvisorPanel } from './components/ArchitecturalAdvisorPanel';
export { SpecificationGeneratorPanel } from './components/SpecificationGeneratorPanel';

// Existing Configuration Panels
export { ModelConfigurationPanel } from './components/ModelConfigurationPanel';

// =============================================================================
// SERVICES
// =============================================================================

// New AI Services
export { EmbedAIService } from './services/EmbedAI';

// Core AI Services (placeholders - extend when needed)
export { AIService } from './code-analysis/CodeAnalyzer';

// =============================================================================
// HOOKS
// =============================================================================

// Newly implemented hooks
export { useArchitecturalAdvice } from './hooks/useArchitecturalAdvice';

// Hook for accessing hook helpers (architectural advice hook exports its own types)
export type {
  ArchitecturalAdviceState,
  ArchitecturalAdviceConfig,
} from './hooks/useArchitecturalAdvice';

// =============================================================================
// TYPE EXPORTS
// =============================================================================

// Core AI Types from types.ts
export type {
  AIAnalysisConfig,
  CodeAnalysisResult,
  ArchitecturalRecommendation,
  ArchitecturalDecision,
  GeneratedCode,
  FineTuneJob,
  TrainingStatus,
  TrainingProgress,
} from './types';

// Service-specific types
export type { ChatMessage, CodeAnalysisRequest } from './services/EmbedAI';

// =============================================================================
// LICENSE AND ATTRIBUTIONS
// =============================================================================

// Service versions and capabilities exposed for debugging
export const AI_ServiceVersion = {
  core: '1.0.0',
  analysis: '2.1.0',
  generation: '1.8.0',
  review: '1.5.0',
  architectural: '1.3.0',
  finetuning: '2.0.0',
} as const;
