import { invoke } from '@tauri-apps/api/core';
import type {
  AutomatedCodeReviewRequest,
  CodeReviewResult,
  ReviewComment,
  ReviewConfig,
  ReviewSuggestion,
} from '../types';

/**
 * Service for automated code review operations
 * Provides intelligent analysis and feedback for code changes
 */
export class CodeReviewService {
  private static instance: CodeReviewService;

  private constructor() {}

  static getInstance(): CodeReviewService {
    if (!CodeReviewService.instance) {
      CodeReviewService.instance = new CodeReviewService();
    }
    return CodeReviewService.instance;
  }

  /**
   * Perform automated code review on a file or directory
   */
  async reviewCode(
    targetPath: string,
    config?: ReviewConfig,
    createReport = true
  ): Promise<CodeReviewResult> {
    try {
      const request: AutomatedCodeReviewRequest = {
        targetPath,
        config: {
          provider: {
            type: 'codellama-rust',
            codellamaRust: {
              modelPath: '/models/codellama-7b',
              modelSize: 'Medium',
              quantization: 'Int4',
              loraAdapters: ['rust-review'],
            },
          },
          analysis_preferences: {
            enableCodeSmells: config?.includeStyle ?? true,
            enablePerformance: config?.includePerformance ?? true,
            enableSecurity: config?.includeSecurity ?? true,
            enableCodeStyle: config?.includeStyle ?? true,
            enableArchitecture: config?.includeArchitecture ?? false,
            enableLearning: true,
            confidenceThreshold: 0.7,
            timeoutSeconds: 300,
            includeExplanations: true,
            includeExamples: false,
            privacyMode: 'opt-in' as const,
          },
          enable_real_time: false,
          enable_workspace_analysis: false,
          max_file_size_kb: 1024,
          excluded_paths: [],
          learning_preferences: {
            enableLearning: false,
            privacyMode: 'opt-out' as const,
            shareAnonymousData: false,
            retainPersonalData: false,
            dataRetentionDays: 30,
            allowModelTraining: false,
          },
          compiler_integration: {
            enable_compiler_integration: false,
            parse_cargo_check_output: true,
            enable_error_explanations: true,
            enable_suggested_fixes: false,
            cache_explanations: false,
            explanation_cache_ttl_hours: 24,
          },
        },
        reviewConfig: config || {
          includeStyle: true,
          includePerformance: true,
          includeSecurity: true,
          includeArchitecture: true,
          maxCommentsPerFile: 20,
          severityThreshold: 'warning',
        },
      };

      const result = await invoke<CodeReviewResult>('run_automated_code_review', {
        request,
      });

      if (createReport) {
        console.log('Code review completed:', {
          overallScore: result.overallAssessment?.score,
          totalComments: result.summary?.totalComments,
          reviewId: result.metadata?.reviewId,
        });
      }

      return result;
    } catch (error) {
      console.error('Failed to perform code review:', error);
      throw new Error(`Code review failed: ${error}`);
    }
  }

  /**
   * Review pull request changes
   * @param prUrl Currently unused, reserved for future PR-specific functionality
   */
  async reviewPullRequest(
    targetPath: string,
    prUrl?: string,
    config?: ReviewConfig
  ): Promise<CodeReviewResult> {
    try {
      const request: AutomatedCodeReviewRequest = {
        targetPath,
        config: {
          provider: {
            type: 'codellama-rust',
            codellamaRust: {
              modelPath: '/models/codellama-7b',
              modelSize: 'Medium',
              quantization: 'Int4',
              loraAdapters: ['rust-review'],
            },
          },
          analysis_preferences: {
            enableCodeSmells: true,
            enablePerformance: true,
            enableSecurity: true,
            enableCodeStyle: true,
            enableArchitecture: true,
            enableLearning: true,
            confidenceThreshold: 0.7,
            timeoutSeconds: 600, // Longer timeout for PR reviews
            includeExplanations: true,
            includeExamples: true,
            privacyMode: 'opt-in' as const,
          },
          enable_real_time: false,
          enable_workspace_analysis: false,
          max_file_size_kb: 1024,
          excluded_paths: [],
          learning_preferences: {
            enableLearning: false,
            privacyMode: 'opt-out' as const,
            shareAnonymousData: false,
            retainPersonalData: false,
            dataRetentionDays: 30,
            allowModelTraining: false,
          },
          compiler_integration: {
            enable_compiler_integration: false,
            parse_cargo_check_output: true,
            enable_error_explanations: false,
            enable_suggested_fixes: false,
            cache_explanations: false,
            explanation_cache_ttl_hours: 24,
          },
        },
        reviewConfig: config || {
          includeStyle: true,
          includePerformance: true,
          includeSecurity: true,
          includeArchitecture: true,
          maxCommentsPerFile: 30,
          severityThreshold: 'info',
        },
      };

      const result = await invoke<CodeReviewResult>('run_automated_code_review', {
        request,
      });

      console.log('PR review completed:', {
        overallScore: result.overallAssessment?.score,
        totalComments: result.summary?.totalComments,
        criticalIssues: result.overallAssessment?.criticalIssues?.length,
        reviewId: result.metadata?.reviewId,
      });

      return result;
    } catch (error) {
      console.error('Failed to review pull request:', error);
      throw new Error(`PR review failed: ${error}`);
    }
  }

  /**
   * Get review comments for a specific file
   */
  async getReviewComments(
    filePath: string,
    content: string,
    severityFilter?: 'info' | 'warning' | 'error'
  ): Promise<ReviewComment[]> {
    try {
      const result = await this.reviewCode(
        filePath,
        {
          includeStyle: true,
          includePerformance: true,
          includeSecurity: true,
          includeArchitecture: false,
          maxCommentsPerFile: 50,
          severityThreshold: severityFilter || 'info',
        },
        false
      );

      let comments = result.reviewComments || [];

      // Apply severity filter if specified
      if (severityFilter) {
        comments = comments.filter((comment) => {
          const severityOrder = { info: 1, warning: 2, error: 3 };
          return (
            severityOrder[comment.severity as keyof typeof severityOrder] >=
            severityOrder[severityFilter]
          );
        });
      }

      return comments;
    } catch (error) {
      console.error('Failed to get review comments:', error);
      throw new Error(`Failed to get review comments: ${error}`);
    }
  }

  /**
   * Generate review report in markdown format
   */
  generateMarkdownReport(result: CodeReviewResult): string {
    const { overallAssessment, summary, suggestions, reviewComments } = result;

    let report = '# AI Code Review Report\n\n';

    // Overall assessment
    if (overallAssessment) {
      report += `## Overall Assessment\n\n`;
      report += `- **Score**: ${overallAssessment.score.toFixed(2)}/1.0\n`;
      report += `- **Grade**: ${overallAssessment.grade}\n`;
      report += `- **Summary**: ${overallAssessment.summary}\n\n`;

      if (overallAssessment.keyStrengths?.length) {
        report += `### Key Strengths\n\n`;
        overallAssessment.keyStrengths.forEach((strength) => {
          report += `- ${strength}\n`;
        });
        report += `\n`;
      }

      if (overallAssessment.criticalIssues?.length) {
        report += `### Critical Issues\n\n`;
        overallAssessment.criticalIssues.forEach((issue) => {
          report += `- ${issue}\n`;
        });
        report += `\n`;
      }
    }

    // Summary
    if (summary) {
      report += `## Summary\n\n`;
      report += `- **Total Comments**: ${summary.totalComments}\n`;

      if (Object.keys(summary.severityBreakdown || {}).length > 0) {
        report += `- **Severity Breakdown**:\n`;
        for (const [severity, count] of Object.entries(summary.severityBreakdown || {})) {
          report += `  - ${severity}: ${count}\n`;
        }
      }

      if (summary.categoryBreakdown && Object.keys(summary.categoryBreakdown).length > 0) {
        report += `- **Category Breakdown**:\n`;
        for (const [category, count] of Object.entries(summary.categoryBreakdown)) {
          report += `  - ${category}: ${count}\n`;
        }
      }

      report += `- **Estimated Effort**: ${summary.estimatedEffort}\n\n`;
    }

    // Review comments by severity
    if (reviewComments && reviewComments.length > 0) {
      report += `## Review Comments\n\n`;

      // Group by severity
      const groupedComments = this.groupCommentsBySeverity(reviewComments);

      for (const severity of ['error', 'warning', 'info']) {
        const comments = groupedComments[severity];
        if (comments && comments.length > 0) {
          report += `### ${severity.charAt(0).toUpperCase() + severity.slice(1)} Comments\n\n`;
          for (const comment of comments) {
            report += `#### ${comment.filePath}`;
            if (comment.line) {
              report += `:${comment.line}`;
              if (comment.column) {
                report += `:${comment.column}`;
              }
            }
            report += `\n\n${comment.message}\n\n`;

            if (comment.suggestion) {
              report += `**Suggestion**: ${comment.suggestion}\n\n`;
            }
          }
        }
      }
    }

    // Suggestions
    if (suggestions && suggestions.length > 0) {
      report += `## Suggestions\n\n`;
      for (const suggestion of suggestions) {
        report += `### ${suggestion.title}\n\n`;
        report += `${suggestion.description}\n\n`;
        if (suggestion.priority) {
          report += `**Priority**: ${suggestion.priority}\n\n`;
        }
      }
    }

    // Metadata
    if (result.metadata) {
      report += `## Metadata\n\n`;
      report += `- **Review ID**: ${result.metadata.reviewId}\n`;
      report += `- **Reviewer**: ${result.metadata.reviewer}\n`;
      report += `- **Timestamp**: ${result.metadata.timestamp}\n`;
      report += `- **Duration**: ${result.metadata.durationMs}ms\n`;
      report += `- **Files Reviewed**: ${result.metadata.fileCount}\n`;
    }

    return report;
  }

  /**
   * Analyze review results for actionable insights
   */
  analyzeReviewInsights(result: CodeReviewResult): ReviewInsights {
    const { summary, overallAssessment } = result;

    const insights: ReviewInsights = {
      criticalCount: 0,
      warningCount: 0,
      infoCount: 0,
      topIssues: [],
      improvementAreas: [],
      strengths: [],
    };

    // Count issues by severity
    if (summary?.severityBreakdown) {
      insights.criticalCount = summary.severityBreakdown.error || 0;
      insights.warningCount = summary.severityBreakdown.warning || 0;
      insights.infoCount = summary.severityBreakdown.info || 0;
    }

    // Identify top issues
    if (result.reviewComments) {
      const issueCounts = this.countIssuesByType(result.reviewComments);
      insights.topIssues = Object.entries(issueCounts)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 5)
        .map(([type, count]) => `${type}: ${count} occurrences`);
    }

    // Extract improvement areas
    if (summary?.categoryBreakdown) {
      const categories = Object.keys(summary.categoryBreakdown);
      insights.improvementAreas = categories.filter(
        (cat) => (summary.categoryBreakdown![cat] || 0) > 3
      );
    }

    // Extract strengths
    if (overallAssessment?.keyStrengths) {
      insights.strengths = overallAssessment.keyStrengths;
    }

    return insights;
  }

  /**
   * Format review results for PR comments
   */
  formatForPRComment(result: CodeReviewResult, maxComments = 10): string {
    const { overallAssessment, summary, reviewComments } = result;

    let comment = '## 🤖 AI Code Review\n\n';

    // Overall assessment
    if (overallAssessment) {
      const scoreEmoji = this.getScoreEmoji(overallAssessment.score);
      comment += `${scoreEmoji} **Overall Score**: ${overallAssessment.score.toFixed(2)}\n\n`;
      comment += `${overallAssessment.summary}\n\n`;
    }

    // Summary stats
    if (summary) {
      comment += '### 📊 Summary\n\n';
      comment += `🔴 **Errors**: ${summary.severityBreakdown?.error || 0}\n`;
      comment += `🟡 **Warnings**: ${summary.severityBreakdown?.warning || 0}\n`;
      comment += `🔵 **Info**: ${summary.severityBreakdown?.info || 0}\n`;
      comment += `💬 **Total Comments**: ${summary.totalComments}\n\n`;
    }

    // Key issues
    if (reviewComments && reviewComments.length > 0) {
      const sortedComments = [...reviewComments].sort((a, b) => {
        const severityOrder = { error: 3, warning: 2, info: 1 };
        return (
          (severityOrder[b.severity as keyof typeof severityOrder] || 0) -
          (severityOrder[a.severity as keyof typeof severityOrder] || 0)
        );
      });

      const topComments = sortedComments.slice(0, maxComments);

      if (topComments.length > 0) {
        comment += '### 🎯 Key Findings\n\n';
        for (const reviewComment of topComments) {
          const severityEmoji = this.getSeverityEmoji(reviewComment.severity);
          comment += `${severityEmoji} **${reviewComment.filePath}`;
          if (reviewComment.line) {
            comment += `:${reviewComment.line}`;
          }
          comment += `** - ${reviewComment.message}\n`;

          if (reviewComment.suggestion) {
            comment += `💡 ${reviewComment.suggestion}\n`;
          }
          comment += '\n';
        }
      }
    }

    if (result.reviewComments && result.reviewComments.length > maxComments) {
      comment += `*... and ${result.reviewComments.length - maxComments} more comments*\n\n`;
    }

    comment +=
      '---\n*This review was generated by AI. Please review the suggestions and apply appropriate changes.*';

    return comment;
  }

  /**
   * Get comments by file
   */
  groupCommentsByFile(comments: ReviewComment[]): Record<string, ReviewComment[]> {
    const grouped: Record<string, ReviewComment[]> = {};

    for (const comment of comments) {
      if (!grouped[comment.filePath]) {
        grouped[comment.filePath] = [];
      }
      grouped[comment.filePath].push(comment);
    }

    return grouped;
  }

  /**
   * Private helper methods
   */
  private groupCommentsBySeverity(comments: ReviewComment[]): Record<string, ReviewComment[]> {
    return comments.reduce(
      (groups, comment) => {
        const severity = comment.severity || 'info';
        if (!groups[severity]) {
          groups[severity] = [];
        }
        groups[severity].push(comment);
        return groups;
      },
      {} as Record<string, ReviewComment[]>
    );
  }

  private countIssuesByType(comments: ReviewComment[]): Record<string, number> {
    return comments.reduce(
      (counts, comment) => {
        const type = comment.category || 'general';
        counts[type] = (counts[type] || 0) + 1;
        return counts;
      },
      {} as Record<string, number>
    );
  }

  private getScoreEmoji(score: number): string {
    if (score >= 0.8) return '🟢';
    if (score >= 0.6) return '🟡';
    return '🔴';
  }

  private getSeverityEmoji(severity: string): string {
    switch (severity) {
      case 'error':
        return '🔴';
      case 'warning':
        return '🟡';
      case 'info':
        return '🔵';
      default:
        return '⚪';
    }
  }
}

/**
 * Review insights extracted from analysis
 */
export interface ReviewInsights {
  criticalCount: number;
  warningCount: number;
  infoCount: number;
  topIssues: string[];
  improvementAreas: string[];
  strengths: string[];
}

// Singleton instance
const codeReviewService = CodeReviewService.getInstance();

export default codeReviewService;
