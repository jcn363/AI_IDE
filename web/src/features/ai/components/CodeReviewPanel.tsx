import React, { useEffect, useState, useRef, useCallback } from 'react';
import CodeReviewService from '../services/CodeReviewService';
import type { CodeReviewResult, ReviewComment } from '../types';

interface CodeReviewPanelProps {
  filePath?: string;
  workspacePath?: string;
  onReviewComplete?: (result: CodeReviewResult) => void;
}

export const CodeReviewPanel: React.FC<CodeReviewPanelProps> = ({
  filePath,
  workspacePath,
  onReviewComplete,
}) => {
  const [reviewResult, setReviewResult] = useState<CodeReviewResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'comments' | 'suggestions' | 'report'>(
    'overview'
  );
  const latestRequestIdRef = useRef<number>(0);

  const runReview = useCallback(async () => {
    if (!filePath && !workspacePath) return;

    const currentRequestId = ++latestRequestIdRef.current;

    setLoading(true);
    setError(null);

    try {
      const targetPath = filePath || workspacePath || '.';
      const result = await CodeReviewService.reviewCode(targetPath, {
        includeStyle: true,
        includePerformance: true,
        includeSecurity: true,
        includeArchitecture: true,
        maxCommentsPerFile: 20,
        severityThreshold: 'warning',
      });

      // Only update state if this is the latest request
      if (currentRequestId === latestRequestIdRef.current) {
        setReviewResult(result);
        onReviewComplete?.(result);
      }
    } catch (err) {
      // Only update error state if this is the latest request
      if (currentRequestId === latestRequestIdRef.current) {
        setError(err instanceof Error ? err.message : 'Review failed');
      }
    } finally {
      // Only update loading state if this is the latest request
      if (currentRequestId === latestRequestIdRef.current) {
        setLoading(false);
      }
    }
  }, [filePath, workspacePath, onReviewComplete]);

  useEffect(() => {
    if (filePath || workspacePath) {
      runReview();
    }
  }, [filePath, workspacePath]);

  if (loading) {
    return (
      <div className="code-review-panel bg-gray-900 text-white p-6 rounded-lg">
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
          <span className="ml-3">Running AI code review...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="code-review-panel bg-gray-900 text-white p-6 rounded-lg">
        <div className="text-center py-8">
          <div className="text-red-400 mb-4">
            <svg
              className="mx-auto h-12 w-12"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
          <h3 className="text-lg font-semibold mb-2">Review Failed</h3>
          <p className="text-gray-400">{error}</p>
          <button
            onClick={runReview}
            className="mt-4 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  if (!reviewResult) {
    return (
      <div className="code-review-panel bg-gray-900 text-white p-6 rounded-lg">
        <div className="text-center py-8 text-gray-400">
          <p>Select a file or workspace to begin AI code review</p>
        </div>
      </div>
    );
  }

  return (
    <div className="code-review-panel bg-gray-900 text-white rounded-lg overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-gray-700">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-bold">AI Code Review</h2>
          <div className="flex items-center space-x-2">
            {reviewResult.overallAssessment && (
              <div
                className={`px-2 py-1 rounded text-sm font-medium ${
                  (reviewResult.overallAssessment.score ?? 0) >= 0.8
                    ? 'bg-green-800 text-green-100'
                    : (reviewResult.overallAssessment.score ?? 0) >= 0.6
                      ? 'bg-yellow-800 text-yellow-100'
                      : 'bg-red-800 text-red-100'
                }`}
              >
                Score: {((reviewResult.overallAssessment.score ?? 0) * 100).toFixed(0)}%
              </div>
            )}
            <button
              onClick={runReview}
              disabled={loading}
              className="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Re-run Review
            </button>
          </div>
        </div>

        {/* Navigation Tabs */}
        <div className="flex space-x-1 mt-4">
          {[
            { key: 'overview', label: 'Overview', icon: '📊' },
            { key: 'comments', label: 'Comments', icon: '💬' },
            { key: 'suggestions', label: 'Suggestions', icon: '💡' },
            { key: 'report', label: 'Report', icon: '📝' },
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key as any)}
              className={`px-3 py-2 rounded text-sm flex items-center space-x-2 ${
                activeTab === tab.key ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Content */}
      <div className="p-4 max-h-96 overflow-y-auto">
        {activeTab === 'overview' && <OverviewTab reviewResult={reviewResult} />}
        {activeTab === 'comments' && <CommentsTab reviewResult={reviewResult} />}
        {activeTab === 'suggestions' && <SuggestionsTab reviewResult={reviewResult} />}
        {activeTab === 'report' && <ReportTab reviewResult={reviewResult} />}
      </div>
    </div>
  );
};

// Overview Tab Component
const OverviewTab: React.FC<{ reviewResult: CodeReviewResult }> = ({ reviewResult }) => {
  const { overallAssessment, summary } = reviewResult;

  return (
    <div className="space-y-4">
      {overallAssessment && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 bg-gray-800 rounded">
            <h3 className="font-semibold mb-2">Overall Assessment</h3>
            <div className="text-2xl font-bold mb-1">{overallAssessment.grade}</div>
            <p className="text-gray-400 text-sm">{overallAssessment.summary}</p>
          </div>

          <div className="p-4 bg-gray-800 rounded">
            <h3 className="font-semibold mb-2">Key Strengths</h3>
            <ul className="text-sm text-gray-300 space-y-1">
              {overallAssessment.keyStrengths?.slice(0, 3).map((strength, i) => (
                <li key={i}>• {strength}</li>
              ))}
            </ul>
          </div>
        </div>
      )}

      {summary && (
        <div className="grid grid-cols-3 gap-4">
          <div className="p-4 bg-gray-800 rounded text-center">
            <div className="text-2xl font-bold text-red-400">
              {summary.severityBreakdown?.error || 0}
            </div>
            <div className="text-sm text-gray-400">Errors</div>
          </div>
          <div className="p-4 bg-gray-800 rounded text-center">
            <div className="text-2xl font-bold text-yellow-400">
              {summary.severityBreakdown?.warning || 0}
            </div>
            <div className="text-sm text-gray-400">Warnings</div>
          </div>
          <div className="p-4 bg-gray-800 rounded text-center">
            <div className="text-2xl font-bold text-blue-400">
              {summary.severityBreakdown?.info || 0}
            </div>
            <div className="text-sm text-gray-400">Info</div>
          </div>
        </div>
      )}
    </div>
  );
};

// Comments Tab Component
const CommentsTab: React.FC<{ reviewResult: CodeReviewResult }> = ({ reviewResult }) => {
  const { reviewComments } = reviewResult;

  if (!reviewComments || reviewComments.length === 0) {
    return <div className="text-center py-8 text-gray-400">No review comments found</div>;
  }

  const groupedComments = reviewComments.reduce(
    (groups, comment) => {
      const severity = comment.severity?.toLowerCase().trim();
      const normalizedSeverity = severity === 'error' || severity === 'warning' ? severity : 'info';
      if (!groups[normalizedSeverity]) groups[normalizedSeverity] = [];
      groups[normalizedSeverity].push(comment);
      return groups;
    },
    {} as Record<string, ReviewComment[]>
  );

  return (
    <div className="space-y-4">
      {Object.entries(groupedComments).map(([severity, comments]) => (
        <div key={severity}>
          <h3
            className={`text-lg font-semibold mb-2 capitalize ${
              severity === 'error'
                ? 'text-red-400'
                : severity === 'warning'
                  ? 'text-yellow-400'
                  : 'text-blue-400'
            }`}
          >
            {severity} Comments ({comments.length})
          </h3>
          <div className="space-y-2">
            {comments.map((comment, i) => (
              <div key={i} className="p-3 bg-gray-800 rounded">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="font-medium text-sm">{comment.filePath}</div>
                    {comment.line && (
                      <div className="text-xs text-gray-400">Line {comment.line}</div>
                    )}
                    <p className="mt-1">{comment.message}</p>
                    {comment.suggestion && (
                      <div className="mt-2 p-2 bg-blue-900 rounded text-sm">
                        <div className="text-blue-300 font-medium">Suggestion:</div>
                        <div>{comment.suggestion}</div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
};

// Suggestions Tab Component
const SuggestionsTab: React.FC<{ reviewResult: CodeReviewResult }> = ({ reviewResult }) => {
  const { suggestions } = reviewResult;

  if (!suggestions || suggestions.length === 0) {
    return <div className="text-center py-8 text-gray-400">No suggestions available</div>;
  }

  return (
    <div className="space-y-2">
      {suggestions.map((suggestion, i) => (
        <div key={i} className="p-3 bg-gray-800 rounded">
          <div className="flex items-start space-x-3">
            <div className="text-blue-400">💡</div>
            <div className="flex-1">
              <h4 className="font-medium">{suggestion.title}</h4>
              <p className="text-gray-400 text-sm mt-1">{suggestion.description}</p>
              {suggestion.priority && (
                <div className="mt-2">
                  <span className="text-xs bg-gray-700 px-2 py-1 rounded">
                    {suggestion.priority} priority
                  </span>
                </div>
              )}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};

// Report Tab Component
const ReportTab: React.FC<{ reviewResult: CodeReviewResult }> = ({ reviewResult }) => {
  const [reportFormat, setReportFormat] = useState<'markdown' | 'json'>('markdown');

  const exportReport = () => {
    let report: string;
    let mimeType: string;
    let filename: string;

    if (reportFormat === 'json') {
      report = JSON.stringify(reviewResult, null, 2);
      mimeType = 'application/json';
      filename = 'ai-code-review-report.json';
    } else {
      report = CodeReviewService.generateMarkdownReport(reviewResult);
      mimeType = 'text/plain';
      filename = 'ai-code-review-report.md';
    }

    const blob = new Blob([report], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a') as HTMLAnchorElement;
    a.href = url;
    a.download = filename;
    a.click();

    // Delay URL revocation to allow download to start
    setTimeout(() => {
      URL.revokeObjectURL(url);
    }, 100);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <label className="text-sm text-gray-400">Export format:</label>
          <select
            value={reportFormat}
            onChange={(e) => setReportFormat(e.target.value as 'markdown' | 'json')}
            className="ml-2 px-2 py-1 bg-gray-800 border border-gray-700 rounded text-sm"
          >
            <option value="markdown">Markdown</option>
            <option value="json">JSON</option>
          </select>
        </div>
        <button
          onClick={exportReport}
          className="px-3 py-1 bg-green-600 hover:bg-green-700 rounded text-sm"
        >
          Export Report
        </button>
      </div>

      <div className="p-4 bg-gray-800 rounded max-h-64 overflow-y-auto">
        <pre className="text-sm whitespace-pre-wrap">
          {CodeReviewService.generateMarkdownReport(reviewResult)}
        </pre>
      </div>
    </div>
  );
};
