import { useCallback, useState } from 'react';
import CodeReviewService from '../services/CodeReviewService';
import type { CodeReviewResult, ReviewComment, ReviewConfig } from '../types';

interface UseCodeReviewReturn {
  // State
  reviewResult: CodeReviewResult | null;
  loading: boolean;
  error: string | null;

  // Actions
  reviewFile: (filePath: string, config?: ReviewConfig) => Promise<void>;
  reviewWorkspace: (workspacePath: string, config?: ReviewConfig) => Promise<void>;
  reviewPullRequest: (targetPath: string, prUrl?: string, config?: ReviewConfig) => Promise<void>;

  // Utilities
  generateReport: (format?: 'markdown' | 'json') => string | null;
  exportReport: (format?: 'markdown' | 'json') => void;
  getCommentsByFile: (filePath: string) => ReviewComment[];
  clearResults: () => void;
}

/**
 * Custom hook for automated code review
 */
export const useCodeReview = (): UseCodeReviewReturn => {
  const [reviewResult, setReviewResult] = useState<CodeReviewResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Review a single file
  const reviewFile = useCallback(async (filePath: string, config?: ReviewConfig) => {
    try {
      setLoading(true);
      setError(null);

      console.log(`Starting code review for file: ${filePath}`);
      const result = await CodeReviewService.reviewCode(filePath, config);
      setReviewResult(result);

      console.log(`Code review completed for ${filePath}:`, {
        score: result.overallAssessment?.score,
        totalComments: result.summary?.totalComments,
      });

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Code review failed';
      setError(errorMessage);
      console.error('Code review failed:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  // Review entire workspace
  const reviewWorkspace = useCallback(async (workspacePath: string, config?: ReviewConfig) => {
    try {
      setLoading(true);
      setError(null);

      console.log(`Starting workspace code review: ${workspacePath}`);
      const result = await CodeReviewService.reviewCode(workspacePath, {
        includeStyle: config?.includeStyle ?? true,
        includePerformance: config?.includePerformance ?? true,
        includeSecurity: config?.includeSecurity ?? true,
        includeArchitecture: config?.includeArchitecture ?? true,
        maxCommentsPerFile: config?.maxCommentsPerFile ?? 50,
        severityThreshold: config?.severityThreshold ?? 'info',
      });
      setReviewResult(result);

      console.log(`Workspace review completed:`, {
        score: result.overallAssessment?.score,
        totalComments: result.summary?.totalComments,
        fileCount: result.metadata?.fileCount,
      });

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Workspace review failed';
      setError(errorMessage);
      console.error('Workspace review failed:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  // Review pull request
  const reviewPullRequest = useCallback(async (
    targetPath: string,
    prUrl?: string,
    config?: ReviewConfig,
  ) => {
    try {
      setLoading(true);
      setError(null);

      console.log(`Starting PR review for: ${targetPath}`);
      const result = await CodeReviewService.reviewPullRequest(targetPath, prUrl, config);
      setReviewResult(result);

      console.log(`PR review completed:`, {
        score: result.overallAssessment?.score,
        totalComments: result.summary?.totalComments,
        recommendation: result.overallAssessment?.score && result.overallAssessment.score >= 0.7 ? 'Approved' : 'Needs Work',
      });

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'PR review failed';
      setError(errorMessage);
      console.error('PR review failed:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  // Generate formatted report
  const generateReport = useCallback((format: 'markdown' | 'json' = 'markdown'): string | null => {
    if (!reviewResult) return null;

    if (format === 'json') {
      return JSON.stringify(reviewResult, null, 2);
    }

    return CodeReviewService.generateMarkdownReport(reviewResult);
  }, [reviewResult]);

  // Export report to file
  const exportReport = useCallback((format: 'markdown' | 'json' = 'markdown') => {
    if (!reviewResult) return;

    const report = generateReport(format);
    if (!report) return;

    const blob = new Blob([report], {
      type: format === 'json' ? 'application/json' : 'text/markdown'
    });

    const url = URL.createObjectURL(blob);
    const a = document.createElement('a') as HTMLAnchorElement;
    a.href = url;
    a.download = `ai-code-review-report.${format === 'json' ? 'json' : 'md'}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, [generateReport, reviewResult]);

  // Get comments filtered by file
  const getCommentsByFile = useCallback((filePath: string): ReviewComment[] => {
    if (!reviewResult?.reviewComments) return [];

    return reviewResult.reviewComments.filter(comment =>
      comment.filePath === filePath
    );
  }, [reviewResult]);

  // Clear all results
  const clearResults = useCallback(() => {
    setReviewResult(null);
    setError(null);
  }, []);

  return {
    reviewResult,
    loading,
    error,
    reviewFile,
    reviewWorkspace,
    reviewPullRequest,
    generateReport,
    exportReport,
    getCommentsByFile,
    clearResults,
  };
};