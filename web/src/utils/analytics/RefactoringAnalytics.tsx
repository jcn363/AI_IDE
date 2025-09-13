import React from 'react';
import { invoke } from '@tauri-apps/api/core';

interface UsagePattern {
  operationType: string;
  frequency: number;
  averageDuration: number;
  successRate: number;
  peakUsageHour: number;
  fileSizeTendency: 'small' | 'medium' | 'large';
  selectionComplexity: 'simple' | 'complex' | 'very_complex';
}

interface UserBehavior {
  productivityScore: number;
  refactoringTimeSpent: number;
  operationsPerSession: number;
  mostUsedOperations: string[];
  averageSessionLength: number;
  workflowPatterns: string[];
  learningProgress: number; // How user skill improves over time
}

interface AnalyticsData {
  totalRefactorings: number;
  usagePatterns: UsagePattern[];
  userBehavior: UserBehavior;
  trendingOperations: string[];
  performanceInsights: string[];
  recommendations: string[];
  lastUpdated: number;
}

interface RefactoringEvent {
  operationType: string;
  timestamp: number;
  fileSize: number;
  duration: number;
  success: boolean;
  confidence?: number;
  selectionLines: number;
  impactedSymbols: number;
  filePath: string;
  workspaceSize?: number;
  contextComplexity: number; // 0-10 scale
}

class RefactoringAnalytics extends React.Component {
  private events: RefactoringEvent[] = [];
  private sessionStart: number = Date.now();

  constructor(props: any) {
    super(props);
    this.initializeAnalytics();
  }

  private async initializeAnalytics() {
    // Load existing analytics from storage
    try {
      const stored = await invoke<string>('get_analytics_data', {});
      const parsed = JSON.parse(stored);

      if (parsed.events) {
        this.events = parsed.events;
        this.sessionStart = Date.now();
      }
    } catch (error) {
      console.log('No existing analytics data found, starting fresh');
    }

    // Set up periodic analytics update
    setInterval(() => this.saveAnalytics(), 60000); // Save every minute

    // Set up usage tracking
    this.trackInitialUsage();
  }

  trackRefactoringEvent(event: RefactoringEvent): void {
    this.events.push(event);

    // Limit stored events to prevent unbounded growth
    if (this.events.length > 10000) {
      this.events = this.events.slice(-5000); // Keep last 5000 events
    }

    this.analyzeUsagePatterns();
  }

  private trackInitialUsage(): void {
    // Track when user starts using the system
    const startEvent: RefactoringEvent = {
      operationType: 'system_initialization',
      timestamp: Date.now(),
      fileSize: 0,
      duration: 0,
      success: true,
      selectionLines: 0,
      impactedSymbols: 0,
      filePath: 'system',
      contextComplexity: 0,
    };
    this.events.push(startEvent);
  }

  async analyzeUsagePatterns(): Promise<AnalyticsData> {
    if (this.events.length === 0) {
      return this.getEmptyAnalytics();
    }

    const now = Date.now();
    const last24Hours = now - 24 * 60 * 60 * 1000;
    const recentEvents = this.events.filter((e) => e.timestamp > last24Hours);

    const operationsByType = this.groupBy(this.events, 'operationType');
    const usagePatterns = Object.entries(operationsByType).map(([type, events]) => {
      const avgDuration = events.reduce((sum, e) => sum + e.duration, 0) / events.length;
      const successRate = events.filter((e) => e.success).length / events.length;
      const usageByHour = this.groupBy(events, (e) => new Date(e.timestamp).getHours());
      const peakHour = Object.keys(usageByHour).reduce(
        (peak, hour) => (usageByHour[hour].length > usageByHour[peak]?.length ? hour : peak),
        '0'
      );

      const fileSizes = events.map((e) => e.fileSize);
      const avgFileSize = this.median(fileSizes);
      const fileSizeTendency =
        avgFileSize < 500 ? 'small' : avgFileSize < 5000 ? 'medium' : 'large';

      const selectionComplexity = this.calculateSelectionComplexity(events);

      return {
        operationType: type,
        frequency: events.length,
        averageDuration: avgDuration,
        successRate: successRate,
        peakUsageHour: parseInt(peakHour),
        fileSizeTendency,
        selectionComplexity,
      };
    });

    // Identify trending operations
    const recentOperations = this.groupBy(recentEvents, 'operationType');
    const allOperations = this.groupBy(this.events, 'operationType');

    const trending = Object.keys(recentOperations)
      .sort((a, b) => {
        const recentA = recentOperations[a]?.length || 0;
        const recentB = recentOperations[b]?.length || 0;
        const totalA = allOperations[a]?.length || 0;
        const totalB = allOperations[b]?.length || 0;

        // Calculate trending score (recent activity vs total activity)
        const scoreA = totalA > 0 ? recentA / totalA : recentA;
        const scoreB = totalB > 0 ? recentB / totalB : recentB;

        return scoreB - scoreA;
      })
      .slice(0, 5);

    return {
      totalRefactorings: this.events.length,
      usagePatterns: usagePatterns,
      userBehavior: this.calculateUserBehavior(),
      trendingOperations: trending,
      performanceInsights: this.generatePerformanceInsights(),
      recommendations: this.generateRecommendations(),
      lastUpdated: Date.now(),
    };
  }

  private groupBy<T, K extends string | number>(
    array: T[],
    keyFn: (item: T) => K
  ): Record<string, T[]> {
    return array.reduce(
      (groups, item) => {
        const key = keyFn(item).toString();
        if (!groups[key]) groups[key] = [];
        groups[key].push(item);
        return groups;
      },
      {} as Record<string, T[]>
    );
  }

  private median(numbers: number[]): number {
    if (numbers.length === 0) return 0;
    const sorted = [...numbers].sort((a, b) => a - b);
    const middle = Math.floor(sorted.length / 2);
    return sorted.length % 2 === 0 ? (sorted[middle - 1] + sorted[middle]) / 2 : sorted[middle];
  }

  private calculateSelectionComplexity(
    events: RefactoringEvent[]
  ): 'simple' | 'complex' | 'very_complex' {
    const avgSelectionLines = events.reduce((sum, e) => sum + e.selectionLines, 0) / events.length;
    const avgImpactedSymbols =
      events.reduce((sum, e) => sum + e.impactedSymbols, 0) / events.length;

    if (avgSelectionLines <= 10 && avgImpactedSymbols <= 5) return 'simple';
    if (avgSelectionLines <= 50 && avgImpactedSymbols <= 20) return 'complex';
    return 'very_complex';
  }

  private calculateUserBehavior(): UserBehavior {
    if (this.events.length === 0) return this.getEmptyUserBehavior();

    const sessionDuration = Date.now() - this.sessionStart;
    const totalRefactoringTime = this.events.reduce((sum, e) => sum + e.duration, 0);
    const successfulOps = this.events.filter((e) => e.success).length;
    const productivityScore = (successfulOps / this.events.length) * 100;

    const operationsByType = this.groupBy(this.events, 'operationType');
    const mostUsed = Object.entries(operationsByType)
      .sort(([, a], [, b]) => b.length - a.length)
      .slice(0, 3)
      .map(([type]) => type);

    // Detect workflow patterns
    const workflowPatterns = this.detectWorkflowPatterns();

    // Calculate learning progress (improving confidence over time)
    const sortedEvents = [...this.events]
      .filter((e) => e.confidence !== undefined)
      .sort((a, b) => a.timestamp - b.timestamp);

    let learningProgress = 0;
    if (sortedEvents.length > 1) {
      const firstQuarter = sortedEvents.slice(0, sortedEvents.length / 4);
      const lastQuarter = sortedEvents.slice(-Math.max(1, sortedEvents.length / 4));

      const earlyAvg =
        firstQuarter.reduce((sum, e) => sum + (e.confidence || 0), 0) / firstQuarter.length;
      const lateAvg =
        lastQuarter.reduce((sum, e) => sum + (e.confidence || 0), 0) / lastQuarter.length;

      learningProgress = lateAvg - earlyAvg;
    }

    return {
      productivityScore,
      refactoringTimeSpent: totalRefactoringTime,
      operationsPerSession: this.events.length,
      mostUsedOperations: mostUsed,
      averageSessionLength: sessionDuration,
      workflowPatterns,
      learningProgress,
    };
  }

  private detectWorkflowPatterns(): string[] {
    const patterns = [];
    const events = [...this.events].sort((a, b) => a.timestamp - b.timestamp);

    // Look for common sequences
    const sequences: { [key: string]: number } = {};

    for (let i = 0; i < events.length - 1; i++) {
      const sequence = `${events[i].operationType} → ${events[i + 1].operationType}`;

      if (events[i + 1].timestamp - events[i].timestamp < 30000) {
        // Within 30 seconds
        sequences[sequence] = (sequences[sequence] || 0) + 1;
      }
    }

    // Extract most common patterns
    const commonPatterns = Object.entries(sequences)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 3)
      .map(([pattern]) => pattern);

    if (commonPatterns.length > 0) {
      patterns.push(...commonPatterns);
    }

    // Detect refactoring styles
    if (
      events.filter((e) => ['extractFunction', 'extractVariable'].includes(e.operationType))
        .length >
      0.3 * events.length
    ) {
      patterns.push('extraction-focused');
    }

    if (events.filter((e) => ['rename'].includes(e.operationType)).length > 0.5 * events.length) {
      patterns.push('rename-heavy');
    }

    return patterns;
  }

  private generatePerformanceInsights(): string[] {
    const insights = [];
    const userBehavior = this.calculateUserBehavior();

    if (userBehavior.productivityScore < 70) {
      insights.push('Productivity could be improved - consider using batch operations or presets');
    }

    if (userBehavior.refactoringTimeSpent > 3600000) {
      // 1 hour
      insights.push('High refactoring time detected - consider AI model optimization');
    }

    const errorEvents = this.events.filter((e) => !e.success);
    if (errorEvents.length > 0.1 * this.events.length) {
      insights.push('High error rate detected - review operation parameters');
    }

    const largeFileEvents = this.events.filter((e) => e.fileSize > 10000);
    if (largeFileEvents.length > 0.3 * this.events.length) {
      insights.push('Frequently working with large files - consider memory optimizations');
    }

    return insights;
  }

  private generateRecommendations(): string[] {
    const recommendations = [];
    const patterns = this.calculateUserBehavior().mostUsedOperations;

    if (patterns.length > 0) {
      recommendations.push(`Profile optimized for ${patterns[0]} operations`);
    }

    const avgDuration =
      this.events.reduce((sum, e) => sum + e.duration, 0) / Math.max(this.events.length, 1);
    if (avgDuration > 5000) {
      recommendations.push('Consider background processing for long operations');
    }

    const lowConfidenceEvents = this.events.filter((e) => (e.confidence || 0) < 0.5);
    if (lowConfidenceEvents.length > 0.3 * this.events.length) {
      recommendations.push('Consider increasing confidence threshold or model fine-tuning');
    }

    return recommendations;
  }

  private getEmptyAnalytics(): AnalyticsData {
    return {
      totalRefactorings: 0,
      usagePatterns: [],
      userBehavior: this.getEmptyUserBehavior(),
      trendingOperations: [],
      performanceInsights: [],
      recommendations: [],
      lastUpdated: Date.now(),
    };
  }

  private getEmptyUserBehavior(): UserBehavior {
    return {
      productivityScore: 0,
      refactoringTimeSpent: 0,
      operationsPerSession: 0,
      mostUsedOperations: [],
      averageSessionLength: 0,
      workflowPatterns: [],
      learningProgress: 0,
    };
  }

  private async saveAnalytics(): Promise<void> {
    try {
      const data = {
        events: this.events,
        analytics: await this.analyzeUsagePatterns(),
        sessionStart: this.sessionStart,
      };

      await invoke('save_analytics_data', { data: JSON.stringify(data) });
    } catch (error) {
      console.error('Failed to save analytics:', error);
    }
  }

  render() {
    return null; // This is a utility class
  }
}

export default RefactoringAnalytics;
export type { RefactoringEvent, UsagePattern, UserBehavior, AnalyticsData };
