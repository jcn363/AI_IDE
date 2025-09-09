import {
  PerformanceAnalysisResult,
  PerformanceIssue,
  PerformanceMetric,
  PerformanceRecommendation,
} from '../types';

export class PerformanceAnalyzer {
  private metrics: PerformanceMetric[] = [];
  private issues: PerformanceIssue[] = [];
  private recommendations: PerformanceRecommendation[] = [];

  constructor() {
    this.initializeDefaultMetrics();
  }

  private initializeDefaultMetrics() {
    // Initialize with default system metrics
    this.metrics = [
      {
        id: 'system.memory.used',
        name: 'Memory Used',
        value: 0,
        unit: 'MB',
        timestamp: Date.now(),
      },
      {
        id: 'system.cpu.usage',
        name: 'CPU Usage',
        value: 0,
        unit: '%',
        timestamp: Date.now(),
      },
    ];
  }

  public addMetric(metric: Omit<PerformanceMetric, 'timestamp'>) {
    this.metrics.push({
      ...metric,
      timestamp: Date.now(),
    });
  }

  public analyzeDependencies(dependencies: any[]): PerformanceRecommendation[] {
    const depRecommendations: PerformanceRecommendation[] = [];
    const now = Date.now();

    // Check for outdated dependencies
    const outdatedDeps = dependencies.filter(
      (dep) => dep.latestVersion && dep.version !== dep.latestVersion
    );

    if (outdatedDeps.length > 0) {
      const depNames = outdatedDeps.map((d) => d.name).join(', ');
      depRecommendations.push({
        id: `dep-update-${now}`,
        title: 'Update Outdated Dependencies',
        description: `Found ${outdatedDeps.length} outdated dependencies that could be updated.`,
        impact: 'medium',
        category: 'dependency',
        affectedDependencies: outdatedDeps.map((d) => d.name),
        estimatedImprovement: 'Varies by dependency',
        action: {
          type: 'dependency-update',
          description: 'Update dependencies to their latest versions',
        },
        timestamp: now,
      });
    }

    // Check for large dependencies
    const largeDeps = dependencies.filter((dep) => dep.size && dep.size > 1024 * 1024 * 5); // >5MB
    if (largeDeps.length > 0) {
      const largeDepNames = largeDeps.map((d) => d.name).join(', ');
      depRecommendations.push({
        id: `dep-size-${now}`,
        title: 'Large Dependencies Detected',
        description: `Found ${largeDeps.length} large dependencies that may impact performance.`,
        impact: 'high',
        category: 'dependency',
        affectedDependencies: largeDeps.map((d) => d.name),
        estimatedImprovement: 'Significant reduction in bundle size',
        action: {
          type: 'command',
          command: 'analyze-bundle',
          description: 'Run bundle analysis to identify optimization opportunities',
        },
        timestamp: now,
      });
    }

    this.recommendations = [...this.recommendations, ...depRecommendations];
    return depRecommendations;
  }

  public analyzeBuild(buildMetrics: any): PerformanceRecommendation[] {
    const buildRecommendations: PerformanceRecommendation[] = [];
    const now = Date.now();

    if (buildMetrics?.duration > 30000) {
      // 30 seconds
      buildRecommendations.push({
        id: `build-slow-${now}`,
        title: 'Slow Build Detected',
        description: `Build took ${(buildMetrics.duration / 1000).toFixed(2)} seconds to complete.`,
        impact: 'medium',
        category: 'build',
        estimatedImprovement: '20-50% faster builds',
        action: {
          type: 'configuration',
          description: 'Enable incremental compilation and parallel builds',
        },
        timestamp: now,
      });
    }

    if (buildMetrics?.warnings?.length > 10) {
      buildRecommendations.push({
        id: `build-warnings-${now}`,
        title: 'Multiple Build Warnings',
        description: `Found ${buildMetrics.warnings.length} build warnings that may indicate potential issues.`,
        impact: 'low',
        category: 'build',
        estimatedImprovement: 'Cleaner build output',
        action: {
          type: 'command',
          command: 'show-warnings',
          description: 'Review and address build warnings',
        },
        timestamp: now,
      });
    }

    this.recommendations = [...this.recommendations, ...buildRecommendations];
    return buildRecommendations;
  }

  public getAnalysis(): PerformanceAnalysisResult {
    const now = Date.now();
    const criticalIssues = this.issues.filter((i) => i.severity === 'high').length;
    const warnings = this.issues.filter((i) => i.severity !== 'high').length;
    const highImpactRecs = this.recommendations.filter((r) => r.impact === 'high').length;

    // Calculate a simple performance score (0-100)
    const issueScore = Math.max(0, 100 - (criticalIssues * 10 + warnings * 2));
    const recScore = Math.min(20, highImpactRecs * 5);
    const performanceScore = Math.max(0, Math.min(100, issueScore - recScore));

    return {
      metrics: this.metrics,
      issues: this.issues,
      recommendations: this.recommendations,
      summary: {
        totalIssues: this.issues.length,
        totalRecommendations: this.recommendations.length,
        performanceScore,
        timestamp: now,
      },
    };
  }

  public clear() {
    this.metrics = [];
    this.issues = [];
    this.recommendations = [];
    this.initializeDefaultMetrics();
  }
}

export const performanceAnalyzer = new PerformanceAnalyzer();
