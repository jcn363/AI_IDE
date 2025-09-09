export interface PerformanceMetric {
  id: string;
  name: string;
  value: number;
  unit: string;
  timestamp: number;
  context?: Record<string, any>;
}

export interface PerformanceIssue {
  id: string;
  type: 'warning' | 'error' | 'suggestion';
  severity: 'low' | 'medium' | 'high';
  message: string;
  details: string;
  recommendation: string;
  affectedFiles?: string[];
  metrics?: string[];
  timestamp: number;
}

export interface PerformanceRecommendation {
  id: string;
  title: string;
  description: string;
  impact: 'low' | 'medium' | 'high';
  category: 'dependency' | 'code' | 'build' | 'runtime' | 'memory';
  affectedFiles?: string[];
  affectedDependencies?: string[];
  estimatedImprovement?: string;
  action: {
    type: 'command' | 'refactor' | 'configuration' | 'dependency-update';
    command?: string;
    description: string;
  };
  relatedIssues?: string[];
  timestamp: number;
}

export interface PerformanceAnalysisResult {
  metrics: PerformanceMetric[];
  issues: PerformanceIssue[];
  recommendations: PerformanceRecommendation[];
  summary: {
    totalIssues: number;
    totalRecommendations: number;
    performanceScore: number;
    timestamp: number;
  };
}
