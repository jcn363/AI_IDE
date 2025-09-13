import { useState, useCallback } from 'react';
import type {
  ArchitecturalRecommendation,
  ArchitecturalDecision,
  RiskAssessment,
  PriorityAction,
  ArchitecturalRoadmap,
} from '../types';

export interface ArchitecturalAdviceState {
  isLoading: boolean;
  recommendations: ArchitecturalRecommendation[];
  decisions: ArchitecturalDecision[];
  error: string | null;
}

export interface ArchitecturalAdviceConfig {
  projectPath?: string;
  analysisScope?: 'file' | 'module' | 'workspace' | 'project';
  focusAreas?: string[];
  priorityThreshold?: 'low' | 'medium' | 'high';
}

export const useArchitecturalAdvice = (config: ArchitecturalAdviceConfig = {}) => {
  const [state, setState] = useState<ArchitecturalAdviceState>({
    isLoading: false,
    recommendations: [],
    decisions: [],
    error: null,
  });

  const analyzeArchitecture = useCallback(
    async (codeOrDescription: string, context?: string[]) => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        // This would call the actual Tauri backend command
        const request = {
          targetPath: config.projectPath || '.',
          config: {
            provider: 'code-llama-rust', // Could be configurable
            analysisPreferences: {
              enable_code_smells: true,
              enable_performance: true,
              enable_security: true,
              enable_style: true,
              enable_architecture: true,
              enable_learning: false,
              timeout_seconds: 30,
              max_suggestions_per_category: 5,
              confidence_threshold: 0.7,
              include_explanations: true,
              include_examples: false,
            },
          },
          analysisScope: config.analysisScope || 'project',
        };

        // Mock response for demonstration
        const mockResponse: ArchitecturalRecommendation = {
          primary_recommendations: [
            {
              title: 'Implement Dependency Injection',
              description:
                'Use dependency injection to decouple business logic from infrastructure concerns.',
              impact: 'High',
              effort: 'Medium',
              rationale:
                'This will improve testability and maintainability by making dependencies explicit and easy to mock.',
            },
            {
              title: 'Add Request Validation',
              description: 'Implement comprehensive input validation at API boundaries.',
              impact: 'High',
              effort: 'Low',
              rationale:
                'Prevents invalid data from propagating through the system and improves security.',
            },
          ],
          secondary_suggestions: [
            {
              title: 'Consider implementing CQRS pattern',
              description:
                'Separate command and query operations for better performance and scalability.',
              priority: 'High',
              category: 'Architecture',
            },
            {
              title: 'Add comprehensive logging and monitoring',
              description:
                'Implement structured logging and performance monitoring throughout the application.',
              priority: 'Medium',
              category: 'Infrastructure',
            },
          ],
          risk_assessment: {
            overall_risk: 0.25,
            risk_factors: [
              'Breaking changes may affect existing integrations',
              'Learning curve for new architectural patterns',
            ],
            mitigation_strategies: [
              'Implement changes incrementally in feature branches',
              'Provide migration guides and training for new patterns',
            ],
          },
          priority_actions: [
            {
              action: 'Create architecture documentation',
              timeline: 'Week 1',
              rationale: 'Foundation for understanding current state and planning improvements',
            },
            {
              action: 'Review and categorize technical debt',
              timeline: 'Week 1-2',
              rationale: 'Prioritize improvements based on business impact',
            },
          ],
          roadmap: {
            short_term: [
              'Implement core architectural patterns (dependency injection, validation)',
              'Create automated testing infrastructure',
              'Set up monitoring and logging framework',
            ],
            medium_term: [
              'Refactor monolithic components into smaller, focused modules',
              'Implement caching layer for performance optimization',
              'Add comprehensive error handling and resilience patterns',
            ],
            long_term: [
              'Consider splitting monolithic application into microservices',
              'Implement distributed caching and high availability',
              'Add advanced security features and compliance automation',
            ],
          },
        };

        // Simulate API call delay
        await new Promise((resolve) => setTimeout(resolve, 1000));

        setState((prev) => ({
          ...prev,
          recommendations: [mockResponse],
          isLoading: false,
        }));

        return mockResponse;
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : 'Architecture analysis failed';
        setState((prev) => ({
          ...prev,
          error: errorMessage,
          isLoading: false,
        }));
        throw error;
      }
    },
    [config]
  );

  const getDecisions = useCallback(async () => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      // This would fetch architectural decisions from the backend
      const mockDecisions: ArchitecturalDecision[] = [
        {
          id: 'decision-001',
          title: 'Database Selection',
          context: 'Choosing the right database for user data storage',
          problem:
            'Need to handle user data with complex relationships and high read/write operations',
          solution_choice: 'PostgreSQL with connection pooling',
          consequences: [
            'Better support for complex queries and relationships',
            'Increased deployment complexity but manageable',
            'Better performance for analytical queries',
          ],
          alternatives: [
            {
              name: 'SQLite + File-based',
              description: 'Simple file-based database with no server management',
              pros: ['No server management', 'Easy deployment'],
              cons: ['Limited concurrency', 'No complex queries', 'Difficult scaling'],
              implementation_effort: 'Minimal',
              risk_level: 'Low',
            },
          ],
          criteria: [
            {
              name: 'Scalability',
              description: 'Ability to handle growth in user base',
              weight: 0.8,
              category: 'Technical',
            },
          ],
          status: 'Implemented',
          made_at: '2024-01-15T10:30:00Z',
          implemented_at: '2024-02-01T14:20:00Z',
          success_score: 0.85,
        },
      ];

      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 500));

      setState((prev) => ({
        ...prev,
        decisions: mockDecisions,
        isLoading: false,
      }));

      return mockDecisions;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to fetch decisions';
      setState((prev) => ({
        ...prev,
        error: errorMessage,
        isLoading: false,
      }));
      throw error;
    }
  }, []);

  const createDecision = useCallback(
    async (decision: Omit<ArchitecturalDecision, 'id' | 'made_at' | 'status'>) => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        // This would create a new architectural decision
        const newDecision: ArchitecturalDecision = {
          ...decision,
          id: `decision-${Date.now()}`,
          status: 'Proposed',
          made_at: new Date().toISOString(),
        };

        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 300));

        setState((prev) => ({
          ...prev,
          decisions: [...prev.decisions, newDecision],
          isLoading: false,
        }));

        return newDecision;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to create decision';
        setState((prev) => ({
          ...prev,
          error: errorMessage,
          isLoading: false,
        }));
        throw error;
      }
    },
    []
  );

  const updateDecision = useCallback(
    async (
      decisionId: string,
      updates: Partial<Pick<ArchitecturalDecision, 'status' | 'solution_choice' | 'success_score'>>
    ) => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        const updatedDecisions = state.decisions.map((decision) =>
          decision.id === decisionId ? { ...decision, ...updates } : decision
        );

        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 200));

        setState((prev) => ({
          ...prev,
          decisions: updatedDecisions,
          isLoading: false,
        }));

        return updatedDecisions.find((d) => d.id === decisionId);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to update decision';
        setState((prev) => ({
          ...prev,
          error: errorMessage,
          isLoading: false,
        }));
        throw error;
      }
    },
    [state.decisions]
  );

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  const clearRecommendations = useCallback(() => {
    setState((prev) => ({ ...prev, recommendations: [], error: null }));
  }, []);

  const clearDecisions = useCallback(() => {
    setState((prev) => ({ ...prev, decisions: [], error: null }));
  }, []);

  return {
    // State
    ...state,

    // Actions
    analyzeArchitecture,
    getDecisions,
    createDecision,
    updateDecision,
    clearError,
    clearRecommendations,
    clearDecisions,

    // Computed properties
    hasRecommendations: state.recommendations.length > 0,
    hasDecisions: state.decisions.length > 0,
    hasError: state.error !== null,

    // Helper methods
    getLatestRecommendation: () => state.recommendations[state.recommendations.length - 1],
    getDecisionById: (id: string) => state.decisions.find((d) => d.id === id),
    getDecisionsByStatus: (status: ArchitecturalDecision['status']) =>
      state.decisions.filter((d) => d.status === status),
  };
};
