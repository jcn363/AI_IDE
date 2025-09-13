import { render, screen } from '@testing-library/react';
import React from 'react';
import { describe, expect, test, vi } from 'vitest';
import { CodeAction, CodeSuggestion } from '../../types';
import { AISuggestionPanel } from '../AISuggestionPanel';

describe('AISuggestionPanel', () => {
  const mockSuggestions: CodeSuggestion[] = [
    {
      id: '1',
      message: 'Test error',
      severity: 1, // error
      severityLevel: 'error',
      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
      category: 'code-smell',
      suggestion: 'Fix the code smell',
      confidence: 0.9,
      explanation: 'This is an error that needs to be fixed',
      quickFixes: [],
      source: 'test',
      timestamp: Date.now(),
    },
    {
      id: '2',
      message: 'Test warning',
      severity: 2, // warning
      severityLevel: 'warning',
      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
      category: 'performance',
      suggestion: 'Optimize performance issue',
      confidence: 0.8,
      explanation: 'This is a performance warning',
      quickFixes: [],
      source: 'test',
      timestamp: Date.now(),
    },
    {
      id: '3',
      message: 'Test info',
      severity: 3, // info
      severityLevel: 'info',
      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
      category: 'style',
      suggestion: 'Improve code style',
      confidence: 0.7,
      explanation: 'This is a style info',
      quickFixes: [],
      source: 'test',
      timestamp: Date.now(),
    },
  ];

  const defaultProps: React.ComponentProps<typeof AISuggestionPanel> = {
    suggestions: mockSuggestions,
    onApplyFix: vi.fn(),
    onDismiss: vi.fn(),
    onLearnMore: vi.fn(),
    visible: true,
    onClose: vi.fn(),
    onRefresh: vi.fn(),
    isAnalyzing: false,
    learnedPatterns: [],
    onRecordFix: vi.fn((suggestion: CodeSuggestion, fix: CodeAction) => {}),
    filters: {
      categories: [],
      severities: [],
      showOnlyFixable: false,
      minConfidence: 0,
      searchText: '',
    },
    sortOptions: {
      field: 'severity' as const,
      direction: 'asc' as const,
    },
    analysisResult: undefined,
    enhancedAnalysisResult: undefined,
    analysisProgress: undefined,
    searchText: '',
  };

  test('renders suggestions with correct severity', () => {
    const { container } = render(<AISuggestionPanel {...defaultProps} />);

    // Check that all suggestions are rendered
    const errorSuggestion = screen.getByText('Test error');
    const warningSuggestion = screen.getByText('Test warning');
    const infoSuggestion = screen.getByText('Test info');

    expect(errorSuggestion).toBeInTheDocument();
    expect(warningSuggestion).toBeInTheDocument();
    expect(infoSuggestion).toBeInTheDocument();

    // Check severity labels
    const errorLabel = screen.getByText('Error');
    const warningLabel = screen.getByText('Warning');
    const infoLabel = screen.getByText('Info');

    expect(errorLabel).toBeInTheDocument();
    expect(warningLabel).toBeInTheDocument();
    expect(infoLabel).toBeInTheDocument();
  });

  test('filters suggestions by severity', () => {
    const { container } = render(
      <AISuggestionPanel
        {...defaultProps}
        filters={{
          categories: defaultProps.filters?.categories ?? [],
          severities: ['error'],
          showOnlyFixable: defaultProps.filters?.showOnlyFixable ?? false,
          minConfidence: defaultProps.filters?.minConfidence ?? 0,
          searchText: defaultProps.filters?.searchText ?? '',
        }}
      />
    );

    // Only error should be visible
    const errorSuggestion = screen.queryByText('Test error');
    const warningSuggestion = screen.queryByText('Test warning');
    const infoSuggestion = screen.queryByText('Test info');

    expect(errorSuggestion).toBeInTheDocument();
    expect(warningSuggestion).not.toBeInTheDocument();
    expect(infoSuggestion).not.toBeInTheDocument();
  });

  test('sorts suggestions by severity', () => {
    const { container } = render(
      <AISuggestionPanel
        {...defaultProps}
        sortOptions={{
          field: 'severity' as const,
          direction: 'asc' as const,
        }}
      />
    );

    // Debug: Log the rendered component HTML after sorting
    console.log('Sorted HTML:', container.innerHTML);

    // Get all suggestion items
    const items = container.querySelectorAll('[data-testid="suggestion-item"]');

    // Debug: Log the text content of each item
    items.forEach((item, index) => {
      console.log(`Item ${index}:`, item.textContent);
    });

    // Check that we have the expected number of items
    expect(items.length).toBe(3);

    // Check order: error (1) should be first, then warning (2), then info (3)
    // Using toContain instead of toHaveTextContent to be more flexible with whitespace
    expect(items[0].textContent).toContain('Test error');
    expect(items[1].textContent).toContain('Test warning');
    expect(items[2].textContent).toContain('Test info');
  });
});
