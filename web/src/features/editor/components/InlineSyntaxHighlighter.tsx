import * as React from 'react';
import { Box, Typography, Tooltip, Chip } from '@mui/material';
import type * as monaco from 'monaco-editor';
import { EmbedAIService } from '../../ai/services/EmbedAI';
import type { AIAnalysisConfig } from '../../ai/types';

type SyntaxError = {
  id: string;
  line: number;
  column: number;
  endColumn?: number;
  severity: 'error' | 'warning' | 'info';
  message: string;
  source: 'lsp' | 'ai' | 'parser';
  confidence?: number;
  suggestions?: string[];
};

type SyntaxHighlight = {
  id: string;
  type: 'highlight' | 'suggestion' | 'refactor';
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  color: string;
  tooltip: string;
  action?: () => void;
};

interface InlineSyntaxHighlighterProps {
  editor: monaco.editor.IStandaloneCodeEditor | null;
  monaco: typeof monaco | null;
  content: string;
  language: string;
  aiEnabled?: boolean;
  aiConfig?: AIAnalysisConfig;
}

const SEVERITY_COLORS = {
  error: '#f44336',
  warning: '#ff9800',
  info: '#2196f3',
};

const HIGHLIGHT_COLORS = {
  refactor: 'rgba(100, 149, 237, 0.3)', // Cornflower blue
  suggestion: 'rgba(50, 205, 50, 0.3)', // Lime green
  highlight: 'rgba(255, 215, 0, 0.3)', // Gold
};

const InlineSyntaxHighlighter: React.FC<InlineSyntaxHighlighterProps> = ({
  editor,
  monaco,
  content,
  language,
  aiEnabled = true,
  aiConfig,
}) => {
  const [syntaxErrors, setSyntaxErrors] = React.useState<SyntaxError[]>([]);
  const [highlights, setHighlights] = React.useState<SyntaxHighlight[]>([]);
  const [isAnalyzing, setIsAnalyzing] = React.useState(false);

  const aiService = React.useMemo(() => EmbedAIService.getInstance(), []);

  // Decorations for Monaco editor
  const [decorations, setDecorations] = React.useState<string[]>([]);

  React.useEffect(() => {
    if (!editor || !monaco || !content.trim()) return;

    const analyzeContent = async () => {
      setIsAnalyzing(true);
      const errors: SyntaxError[] = [];
      const newHighlights: SyntaxHighlight[] = [];

      try {
        // Basic LSP error analysis
        const model = editor.getModel();
        if (model && monaco) {
          // Get Monaco markers (built-in diagnostics)
          const markers = monaco.editor.getModelMarkers({ resource: model.uri });
          markers.forEach((marker) => {
            errors.push({
              id: `lsp-${marker.startLineNumber}-${marker.startColumn}`,
              line: marker.startLineNumber,
              column: marker.startColumn,
              endColumn: marker.endColumn,
              severity:
                marker.severity === monaco.MarkerSeverity.Error
                  ? 'error'
                  : marker.severity === monaco.MarkerSeverity.Warning
                    ? 'warning'
                    : 'info',
              message: marker.message,
              source: 'lsp',
            });
          });

          // AI-powered analysis if enabled
          if (aiEnabled && aiConfig) {
            try {
              // Analyze code for additional insights
              const analysis = await aiService.analyzeCode({
                code: content,
                analysisType: 'security',
                context: `Analysis of ${language} code`,
              });

              if (analysis.errors) {
                analysis.errors.forEach((error: any, index: number) => {
                  const line =
                    content.substring(0, error.loc?.start?.index || 0).split('\n').length || 1;
                  errors.push({
                    id: `ai-${index}`,
                    line,
                    column: 0,
                    severity: 'warning',
                    message: error.message || 'AI detected potential issue',
                    source: 'ai',
                    confidence: error.confidence || 0.7,
                    suggestions: error.suggestions || [],
                  });
                });
              }

              // Code quality highlights
              const suggestions = analysis.suggestions || [];
              suggestions.forEach((suggestion: any, index: number) => {
                if (suggestion.line && suggestion.content) {
                  newHighlights.push({
                    id: `suggestion-${index}`,
                    type: 'suggestion',
                    line: suggestion.line,
                    column: 0,
                    endLine: suggestion.line,
                    endColumn: content.split('\n')[suggestion.line - 1]?.length || 0,
                    color: HIGHLIGHT_COLORS.suggestion,
                    tooltip: suggestion.content,
                  });
                }
              });
            } catch (aiError) {
              console.warn('AI analysis failed:', aiError);
            }
          }
        }
      } catch (error) {
        console.error('Syntax analysis failed:', error);
      }

      setSyntaxErrors(errors);
      setHighlights(newHighlights);
      setIsAnalyzing(false);

      // Apply Monaco decorations
      applyDecorations(editor, monaco, errors, newHighlights);
    };

    const debounceTimer = setTimeout(analyzeContent, 1000); // Debounce analysis

    return () => clearTimeout(debounceTimer);
  }, [content, language, editor, monaco, aiEnabled, aiConfig, aiService]);

  const applyDecorations = (
    editor: monaco.editor.IStandaloneCodeEditor,
    monaco: typeof monaco,
    errors: SyntaxError[],
    highlights: SyntaxHighlight[]
  ) => {
    const decorations: monaco.editor.IModelDeltaDecoration[] = [];

    // Add decorations for highlights
    highlights.forEach((highlight) => {
      decorations.push({
        range: new monaco.Range(
          highlight.line,
          highlight.column + 1,
          highlight.endLine,
          (highlight.endColumn || highlight.column) + 1
        ),
        options: {
          className: 'custom-highlight',
          hoverMessage: { value: highlight.tooltip },
          inlineClassName: `inline-highlight-${highlight.type}`,
          stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
        },
      });
    });

    // Apply decorations
    const newDecorations = editor.deltaDecorations(decorationsId, decorations);
    setDecorations(newDecorations);
  };

  const decorationsId = React.useRef<string[]>([]);
  React.useEffect(() => {
    if (!editor || !monaco) return;
    decorationsId.current = [];
  }, [editor, monaco]);

  const getErrorsForLine = (lineNumber: number) =>
    syntaxErrors.filter((error) => error.line === lineNumber);

  const getHighlightsForLine = (lineNumber: number) =>
    highlights.filter((highlight) => highlight.line === lineNumber);

  if (!editor) return null;

  return (
    <>
      {isAnalyzing && (
        <Box
          sx={{
            position: 'fixed',
            top: 20,
            left: '50%',
            transform: 'translateX(-50%)',
            backgroundColor: 'background.paper',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
            padding: 1,
            zIndex: 1000,
            display: 'flex',
            alignItems: 'center',
            gap: 1,
          }}
        >
          <Typography variant="caption">Analyzing code...</Typography>
        </Box>
      )}

      {/* Error and warning overlays on the gutter */}
      {syntaxErrors.map((error) => (
        <Tooltip
          key={error.id}
          title={
            <div>
              <Typography variant="caption" sx={{ display: 'block', mb: 1 }}>
                {error.message}
              </Typography>
              {error.suggestions && error.suggestions.length > 0 && (
                <div>
                  <Typography variant="caption" sx={{ fontWeight: 'bold' }}>
                    Suggestions:
                  </Typography>
                  {error.suggestions.slice(0, 3).map((suggestion, idx) => (
                    <Typography key={idx} variant="caption" sx={{ display: 'block', ml: 1 }}>
                      • {suggestion}
                    </Typography>
                  ))}
                </div>
              )}
              {error.source === 'ai' && error.confidence && (
                <Chip
                  label={`${Math.round(error.confidence * 100)}% confidence`}
                  size="small"
                  variant="outlined"
                  sx={{ mt: 1 }}
                />
              )}
            </div>
          }
        >
          <Box
            sx={{
              position: 'absolute',
              left: 0,
              top: `${(error.line - 1) * 1.5}em`, // Approximate line height
              width: 8,
              height: 8,
              backgroundColor: SEVERITY_COLORS[error.severity],
              borderRadius: '50%',
              cursor: 'pointer',
              zIndex: 10,
              '&:hover': {
                width: 12,
                height: 12,
              },
            }}
          />
        </Tooltip>
      ))}

      {/* Highlight overlays on the editor */}
      {highlights.map((highlight) => {
        const lineHeight = 24; // Approximate Monaco line height
        const top = (highlight.line - 1) * lineHeight;

        return (
          <Box
            key={highlight.id}
            sx={{
              position: 'absolute',
              left: '50px', // Monaco gutter width
              top: `${top}px`,
              height: `${lineHeight}px`,
              width: 'calc(100% - 50px)',
              backgroundColor: highlight.color,
              pointerEvents: 'none',
              cursor: highlight.action ? 'pointer' : 'default',
              zIndex: 5,
            }}
            onClick={highlight.action}
          />
        );
      })}

      <style jsx global>{`
        .custom-highlight {
          border-radius: 2px;
          border: 1px solid rgba(0, 0, 0, 0.1);
        }

        .inline-highlight-refactor {
          background-color: ${HIGHLIGHT_COLORS.refactor};
        }

        .inline-highlight-suggestion {
          background-color: ${HIGHLIGHT_COLORS.suggestion};
        }

        .inline-highlight-highlight {
          background-color: ${HIGHLIGHT_COLORS.highlight};
        }
      `}</style>
    </>
  );
};

export default InlineSyntaxHighlighter;
