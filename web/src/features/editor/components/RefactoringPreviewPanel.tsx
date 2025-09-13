import * as React from 'react';
import {
  Box,
  Paper,
  Typography,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Chip,
  IconButton,
  Button,
  CircularProgress,
  Alert,
  Divider,
} from '@mui/material';
import {
  ExpandMore,
  Check,
  Close,
  Restore,
  Compare,
  Code,
  Warning,
  CheckCircle,
  Error as ErrorIcon,
} from '@mui/icons-material';
import type * as monaco from 'monaco-editor';
import { RefactoringService } from '../../ai/services/RefactoringService';
import { EmbedAIService } from '../../ai/services/EmbedAI';
import type { AIAnalysisConfig } from '../../ai/types';

type DiffLine = {
  lineNumber: number;
  type: 'add' | 'remove' | 'context';
  content: string;
  oldContent?: string;
  newContent?: string;
};

type RefactoringSuggestion = {
  id: string;
  title: string;
  description: string;
  confidence: number;
  impact: 'low' | 'medium' | 'high';
  type:
    | 'extract-method'
    | 'rename-variable'
    | 'inline-function'
    | 'move-method'
    | 'change-signature'
    | 'custom';
  ranges: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  }[];
  before: string;
  after: string;
  diffs: DiffLine[];
  metadata?: {
    explanation: string;
    risks: string[];
    dependencies: string[];
    complexity: number;
  };
};

type RefactoringPreviewPanelProps = {
  editor: monaco.editor.IStandaloneCodeEditor | null;
  monaco: typeof monaco | null;
  selectedCode: string;
  selection: monaco.IRange | null;
  language: string;
  aiEnabled?: boolean;
  aiConfig?: AIAnalysisConfig;
  onRefactoringApplied?: (suggestion: RefactoringSuggestion) => void;
  onRefactoringRejected?: (suggestionId: string) => void;
};

const RefactoringPreviewPanel: React.FC<RefactoringPreviewPanelProps> = ({
  editor,
  monaco,
  selectedCode,
  selection,
  language,
  aiEnabled = true,
  aiConfig,
  onRefactoringApplied,
  onRefactoringRejected,
}) => {
  const [suggestions, setSuggestions] = React.useState<RefactoringSuggestion[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [expandedItems, setExpandedItems] = React.useState<Set<string>>(new Set());

  const refactoringService = React.useMemo(() => RefactoringService.getInstance(), []);
  const aiService = React.useMemo(() => EmbedAIService.getInstance(), []);

  const generateSuggestions = React.useCallback(async () => {
    if (!selectedCode.trim() || !aiConfig) return;

    setLoading(true);
    try {
      const generatedSuggestions: RefactoringSuggestion[] = [];

      // Get AI-powered refactoring suggestions
      if (aiEnabled && aiConfig) {
        try {
          const refactorResponse = await aiService.suggestRefactoring(
            selectedCode,
            `Refactor this ${language} code for better readability, maintainability, and performance.`,
            aiConfig
          );

          if (refactorResponse && typeof refactorResponse === 'object') {
            // Process AI response into suggestions
            const aiSuggestions = Array.isArray(refactorResponse)
              ? refactorResponse
              : [refactorResponse];

            for (const suggestion of aiSuggestions) {
              if (suggestion && suggestion.title && suggestion.before && suggestion.after) {
                const diffs = generateDiffs(suggestion.before, suggestion.after);

                generatedSuggestions.push({
                  id: `ai-${Date.now()}-${Math.random()}`,
                  title: suggestion.title,
                  description: suggestion.description || 'AI-generated refactoring suggestion',
                  confidence: suggestion.confidence || 0.7,
                  impact: suggestion.impact || 'medium',
                  type: suggestion.type || 'custom',
                  ranges: [
                    selection
                      ? {
                          startLine: selection.startLineNumber,
                          startColumn: selection.startColumn,
                          endLine: selection.endLineNumber,
                          endColumn: selection.endColumn,
                        }
                      : {
                          startLine: 1,
                          startColumn: 1,
                          endLine: selectedCode.split('\n').length,
                          endColumn: selectedCode.length,
                        },
                  ],
                  before: suggestion.before,
                  after: suggestion.after,
                  diffs,
                  metadata: {
                    explanation:
                      suggestion.explanation || 'AI-suggested refactoring to improve code quality.',
                    risks: suggestion.risks || [],
                    dependencies: suggestion.dependencies || [],
                    complexity: suggestion.complexity || 1,
                  },
                });
              }
            }
          }
        } catch (aiError) {
          console.warn('AI refactoring failed:', aiError);
        }
      }

      // Add standard refactoring suggestions
      const standardSuggestions = generateStandardRefactoringSuggestions(
        selectedCode,
        selection,
        language
      );
      generatedSuggestions.push(...standardSuggestions);

      setSuggestions(generatedSuggestions);

      // Auto-expand high-confidence suggestions
      const highConfidence = generatedSuggestions.filter((s) => s.confidence > 0.8);
      if (highConfidence.length > 0) {
        const ids = highConfidence.map((s) => s.id);
        setExpandedItems(new Set(ids));
      }
    } catch (error) {
      console.error('Refactoring generation failed:', error);
    } finally {
      setLoading(false);
    }
  }, [selectedCode, selection, language, aiEnabled, aiConfig, aiService]);

  const generateDiffs = (before: string, after: string): DiffLine[] => {
    const beforeLines = before.split('\n');
    const afterLines = after.split('\n');
    const diffs: DiffLine[] = [];

    // Simple line-by-line diff (in a real implementation, use a proper diff algorithm)
    const maxLines = Math.max(beforeLines.length, afterLines.length);

    for (let i = 0; i < maxLines; i++) {
      const beforeLine = beforeLines[i];
      const afterLine = afterLines[i];

      if (!beforeLine && afterLine) {
        diffs.push({
          lineNumber: i + 1,
          type: 'add',
          content: afterLine,
        });
      } else if (beforeLine && !afterLine) {
        diffs.push({
          lineNumber: i + 1,
          type: 'remove',
          content: beforeLine,
        });
      } else if (beforeLine !== afterLine) {
        diffs.push({
          lineNumber: i + 1,
          type: 'context',
          content: afterLine || beforeLine,
          oldContent: beforeLine,
          newContent: afterLine,
        });
      } else {
        diffs.push({
          lineNumber: i + 1,
          type: 'context',
          content: beforeLine,
        });
      }
    }

    return diffs;
  };

  const generateStandardRefactoringSuggestions = (
    code: string,
    selection: monaco.IRange | null,
    language: string
  ): RefactoringSuggestion[] => {
    const standardSuggestions: RefactoringSuggestion[] = [];

    // Extract method/function suggestion
    if (
      (code.length > 100 && code.includes('function')) ||
      code.includes('def') ||
      code.includes('fn')
    ) {
      const before = code;
      const after = `// Extracted method\nextractedMethod();\n\n// Remaining code goes here`;

      standardSuggestions.push({
        id: `standard-extract-${Date.now()}`,
        title: 'Extract Method',
        description: 'Extract selected code into a separate method for better readability',
        confidence: 0.85,
        impact: 'high',
        type: 'extract-method',
        ranges: selection
          ? [
              {
                startLine: selection.startLineNumber,
                startColumn: selection.startColumn,
                endLine: selection.endLineNumber,
                endColumn: selection.endColumn,
              },
            ]
          : [],
        before,
        after,
        diffs: generateDiffs(before, after),
        metadata: {
          explanation: 'Method extraction improves code organization and reusability.',
          risks: ['Potential naming conflicts', 'Scope changes'],
          dependencies: [],
          complexity: 3,
        },
      });
    }

    // Variable renaming suggestion
    if (
      code.includes('var ') ||
      code.includes('let ') ||
      code.includes('const ') ||
      code.includes('int ') ||
      code.includes('String ') ||
      code.includes('let ')
    ) {
      standardSuggestions.push({
        id: `standard-rename-${Date.now()}`,
        title: 'Rename Variable',
        description: 'Improve variable naming for clarity',
        confidence: 0.9,
        impact: 'low',
        type: 'rename-variable',
        ranges: selection
          ? [
              {
                startLine: selection.startLineNumber,
                startColumn: selection.startColumn,
                endLine: selection.endLineNumber,
                endColumn: selection.endColumn,
              },
            ]
          : [],
        before: code,
        after: code, // Would be modified in actual implementation
        diffs: [],
        metadata: {
          explanation: 'Clear variable names improve code maintainability.',
          risks: ['Breaking changes if variables are exported'],
          dependencies: [],
          complexity: 1,
        },
      });
    }

    return standardSuggestions;
  };

  const applyRefactoring = React.useCallback(
    async (suggestion: RefactoringSuggestion) => {
      if (!editor) return;

      try {
        const model = editor.getModel();
        if (!model || suggestion.ranges.length === 0) return;

        const range = suggestion.ranges[0];
        const monacoRange = new monaco.Range(
          range.startLine,
          range.startColumn,
          range.endLine,
          range.endColumn
        );

        editor.executeEdits('refactoring-application', [
          {
            range: monacoRange,
            text: suggestion.after,
          },
        ]);

        onRefactoringApplied?.(suggestion);
        setSuggestions((prev) => prev.filter((s) => s.id !== suggestion.id));
      } catch (error) {
        console.error('Refactoring application failed:', error);
      }
    },
    [editor, onRefactoringApplied]
  );

  const rejectSuggestion = React.useCallback(
    (suggestionId: string) => {
      onRefactoringRejected?.(suggestionId);
      setSuggestions((prev) => prev.filter((s) => s.id !== suggestionId));
    },
    [onRefactoringRejected]
  );

  const toggleExpanded = React.useCallback((id: string) => {
    setExpandedItems((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  }, []);

  const renderDiff = (diffs: DiffLine[]) => {
    return (
      <Box sx={{ fontFamily: 'monospace', fontSize: '0.875rem' }}>
        {diffs.map((diff, index) => (
          <Box
            key={index}
            sx={{
              display: 'flex',
              backgroundColor:
                diff.type === 'add'
                  ? 'success.main'
                  : diff.type === 'remove'
                    ? 'error.main'
                    : 'transparent',
              opacity: 0.8,
              p: 0.25,
              borderRadius: 0.5,
            }}
          >
            <Box
              sx={{
                minWidth: 40,
                color: 'text.secondary',
                textAlign: 'right',
                pr: 1,
                borderRight: 1,
                borderColor: 'divider',
              }}
            >
              {diff.lineNumber}
            </Box>
            <Box
              sx={{
                ml: 1,
                color:
                  diff.type === 'add'
                    ? 'success.contrastText'
                    : diff.type === 'remove'
                      ? 'error.contrastText'
                      : 'text.primary',
              }}
            >
              {diff.type === 'add' && '+'}
              {diff.type === 'remove' && '-'} {diff.content}
            </Box>
          </Box>
        ))}
      </Box>
    );
  };

  React.useEffect(() => {
    if (selectedCode && aiConfig) {
      generateSuggestions();
    }
  }, [selectedCode, aiConfig, generateSuggestions]);

  if (!selectedCode) {
    return (
      <Paper sx={{ p: 2 }}>
        <Typography variant="body2" color="text.secondary">
          Select code to see refactoring suggestions
        </Typography>
      </Paper>
    );
  }

  return (
    <Paper sx={{ height: '100%', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
      <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Typography variant="h6">Refactoring Suggestions</Typography>
          <Button
            variant="outlined"
            startIcon={<Compare />}
            onClick={generateSuggestions}
            disabled={loading}
            size="small"
          >
            {loading ? <CircularProgress size={16} /> : 'Refresh'}
          </Button>
        </Box>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          Selected{' '}
          {selection ? `${selection.endLineNumber - selection.startLineNumber + 1} lines` : 'code'}{' '}
          for refactoring
        </Typography>
      </Box>

      <Box sx={{ flex: 1, overflow: 'auto' }}>
        {suggestions.length === 0 && !loading && (
          <Alert severity="info" sx={{ m: 2 }}>
            No refactoring suggestions available for the selected code.
          </Alert>
        )}

        {suggestions.map((suggestion) => (
          <Accordion
            key={suggestion.id}
            expanded={expandedItems.has(suggestion.id)}
            onChange={() => toggleExpanded(suggestion.id)}
            sx={{ '&:before': { display: 'none' } }}
          >
            <AccordionSummary expandIcon={<ExpandMore />}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flex: 1 }}>
                <Typography variant="subtitle2">{suggestion.title}</Typography>

                <Box sx={{ display: 'flex', gap: 1 }}>
                  <Chip
                    label={`${Math.round(suggestion.confidence * 100)}%`}
                    size="small"
                    color={
                      suggestion.confidence > 0.8
                        ? 'success'
                        : suggestion.confidence > 0.6
                          ? 'warning'
                          : 'default'
                    }
                    variant="outlined"
                  />

                  <Chip
                    label={suggestion.impact.toUpperCase()}
                    size="small"
                    color={
                      suggestion.impact === 'high'
                        ? 'error'
                        : suggestion.impact === 'medium'
                          ? 'warning'
                          : 'info'
                    }
                  />

                  <Chip
                    label={suggestion.type.replace('-', ' ').toUpperCase()}
                    size="small"
                    variant="outlined"
                  />
                </Box>
              </Box>
            </AccordionSummary>

            <AccordionDetails>
              <Typography variant="body2" sx={{ mb: 2 }}>
                {suggestion.description}
              </Typography>

              {suggestion.metadata && (
                <>
                  <Alert
                    severity={suggestion.metadata.risks.length > 0 ? 'warning' : 'info'}
                    sx={{ mb: 2 }}
                    icon={suggestion.metadata.risks.length > 0 ? <Warning /> : <CheckCircle />}
                  >
                    <Typography variant="body2">{suggestion.metadata.explanation}</Typography>
                  </Alert>

                  {suggestion.metadata.risks.length > 0 && (
                    <Box sx={{ mb: 2 }}>
                      <Typography variant="body2" sx={{ fontWeight: 'bold', mb: 1 }}>
                        Potential risks:
                      </Typography>
                      {suggestion.metadata.risks.map((risk, index) => (
                        <Typography
                          key={index}
                          variant="body2"
                          sx={{ ml: 2, color: 'warning.main' }}
                        >
                          • {risk}
                        </Typography>
                      ))}
                    </Box>
                  )}
                </>
              )}

              <Divider sx={{ my: 2 }} />

              <Typography variant="subtitle2" sx={{ mb: 1 }}>
                Changes Preview:
              </Typography>

              {suggestion.diffs.length > 0 ? (
                renderDiff(suggestion.diffs)
              ) : (
                <Typography variant="body2" color="text.secondary">
                  No diff preview available
                </Typography>
              )}

              <Box sx={{ display: 'flex', gap: 1, mt: 2 }}>
                <Button
                  variant="contained"
                  startIcon={<Check />}
                  onClick={() => applyRefactoring(suggestion)}
                  color="primary"
                  size="small"
                >
                  Apply Refactoring
                </Button>

                <Button
                  variant="outlined"
                  startIcon={<Close />}
                  onClick={() => rejectSuggestion(suggestion.id)}
                  color="secondary"
                  size="small"
                >
                  Dismiss
                </Button>

                {suggestion.metadata?.complexity && suggestion.metadata.complexity > 2 && (
                  <Alert severity="warning" sx={{ flex: 1 }}>
                    High complexity refactoring - review carefully
                  </Alert>
                )}
              </Box>
            </AccordionDetails>
          </Accordion>
        ))}
      </Box>
    </Paper>
  );
};

export default RefactoringPreviewPanel;
