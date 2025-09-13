import * as React from 'react';
import { Box, Paper, Typography, Chip, CircularProgress, IconButton, Tooltip } from '@mui/material';
import { Check, Close, KeyboardReturn, ThumbUp, ThumbDown } from '@mui/icons-material';
import type * as monaco from 'monaco-editor';
import { EmbedAIService } from '../../ai/services/EmbedAI';
import type { AIAnalysisConfig } from '../../ai/types';

type CompletionItem = {
  id: string;
  text: string;
  confidence: number;
  type: 'completion' | 'refinement' | 'suggestion';
  description?: string;
  metadata?: {
    source: 'ai' | 'lsp' | 'context';
    category?: string;
    language?: string;
  };
};

interface RealTimeCompletionPopupProps {
  editor: monaco.editor.IStandaloneCodeEditor | null;
  monaco: typeof monaco | null;
  cursorPosition: monaco.IPosition;
  content: string;
  language: string;
  aiEnabled?: boolean;
  aiConfig?: AIAnalysisConfig;
  onSuggestionApplied?: (suggestion: CompletionItem) => void;
  onFeedbackGiven?: (suggestionId: string, type: 'positive' | 'negative') => void;
}

const RealTimeCompletionPopup: React.FC<RealTimeCompletionPopupProps> = ({
  editor,
  monaco,
  cursorPosition,
  content,
  language,
  aiEnabled = true,
  aiConfig,
  onSuggestionApplied,
  onFeedbackGiven,
}) => {
  const [showPopup, setShowPopup] = React.useState(false);
  const [suggestions, setSuggestions] = React.useState<CompletionItem[]>([]);
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const [isLoading, setIsLoading] = React.useState(false);
  const [popupPosition, setPopupPosition] = React.useState({ x: 0, y: 0 });

  const aiService = React.useMemo(() => EmbedAIService.getInstance(), []);
  const popupRef = React.useRef<HTMLDivElement>(null);

  const hidePopup = React.useCallback(() => {
    setShowPopup(false);
    setSuggestions([]);
    setSelectedIndex(0);
  }, []);

  const calculatePopupPosition = React.useCallback(() => {
    if (!editor) return { x: 0, y: 0 };

    const editorContainer = editor.getDomNode();
    if (!editorContainer) return { x: 0, y: 0 };

    const cursorCoords = editor.getScrolledVisibleTop();
    const lineHeight = editor.getConfiguration().lineHeight;

    return {
      x: 16, // Small left margin
      y: cursorCoords + lineHeight,
    };
  }, [editor]);

  const generateSuggestions = React.useCallback(async () => {
    if (!aiEnabled || !aiConfig || isLoading) return;

    setIsLoading(true);
    try {
      // Get context around cursor
      const lines = content.split('\n');
      const currentLine = lines[cursorPosition.lineNumber - 1] || '';
      const lineStart = currentLine.substring(0, cursorPosition.column);
      const contextBefore = lines
        .slice(Math.max(0, cursorPosition.lineNumber - 6), cursorPosition.lineNumber)
        .join('\n');
      const contextAfter = lines
        .slice(cursorPosition.lineNumber, cursorPosition.lineNumber + 4)
        .join('\n');

      // Basic LSP context
      const lspSuggestions: CompletionItem[] = [
        {
          id: 'lsp-basic',
          text: lineStart,
          confidence: 0.3,
          type: 'completion',
          description: 'Basic completion',
          metadata: { source: 'lsp', language },
        },
      ];

      // AI-powered suggestions
      let aiSuggestions: CompletionItem[] = [];

      if (aiEnabled && aiConfig) {
        try {
          const completionResponse = await aiService.getCompletion(
            `Complete this ${language} code at line ${cursorPosition.lineNumber}, position ${cursorPosition.column}.
Current line context: "${lineStart}"
Context before: ${contextBefore}
Context after: ${contextAfter}\n\nProvide:`,
            [contextBefore, lineStart],
            aiConfig
          );

          // Parse AI response into multiple suggestions
          const suggestions = completionResponse.split('\n').filter((s) => s.trim());

          aiSuggestions = suggestions.slice(0, 3).map((suggestion, index) => ({
            id: `ai-${index}`,
            text: suggestion.trim(),
            confidence: Math.max(0.4, 1 / (index + 1)), // Decreasing confidence
            type: 'completion' as const,
            description: `AI suggestion ${index + 1}`,
            metadata: { source: 'ai', language, category: 'smart' },
          }));
        } catch (error) {
          console.warn('AI completion failed:', error);
        }
      }

      const allSuggestions = [...lspSuggestions, ...aiSuggestions];
      setSuggestions(allSuggestions.filter((s) => s.text && s.text !== lineStart));
      setSelectedIndex(0);
      setPopupPosition(calculatePopupPosition());

      if (allSuggestions.length > 0) {
        setShowPopup(true);
      }
    } catch (error) {
      console.error('Suggestion generation failed:', error);
    } finally {
      setIsLoading(false);
    }
  }, [
    aiEnabled,
    aiConfig,
    isLoading,
    content,
    cursorPosition,
    language,
    aiService,
    calculatePopupPosition,
  ]);

  const applySuggestion = React.useCallback(
    (index: number) => {
      if (!editor || !suggestions[index]) return;

      const suggestion = suggestions[index];
      const model = editor.getModel();
      if (!model) return;

      // Get current line content
      const lineContent = model.getLineContent(cursorPosition.lineNumber);
      const contentBeforeCursor = lineContent.substring(0, cursorPosition.column);

      // Insert suggestion at cursor position
      const range = new monaco.Range(
        cursorPosition.lineNumber,
        cursorPosition.column + 1,
        cursorPosition.lineNumber,
        lineContent.length + 1
      );

      const edit = {
        range,
        text: suggestion.text,
      };

      editor.executeEdits('completion-application', [edit]);
      onSuggestionApplied?.(suggestion);

      hidePopup();
    },
    [editor, suggestions, cursorPosition, onSuggestionApplied, hidePopup]
  );

  const handleKeyDown = React.useCallback(
    (event: KeyboardEvent) => {
      if (!showPopup || suggestions.length === 0) return;

      if (event.key === 'Escape') {
        hidePopup();
        event.preventDefault();
        return;
      }

      if (event.key === 'ArrowDown') {
        setSelectedIndex((prev) => (prev + 1) % suggestions.length);
        event.preventDefault();
        return;
      }

      if (event.key === 'ArrowUp') {
        setSelectedIndex((prev) => (prev - 1 + suggestions.length) % suggestions.length);
        event.preventDefault();
        return;
      }

      if (event.key === 'Enter') {
        applySuggestion(selectedIndex);
        event.preventDefault();
        return;
      }

      if (event.key === 'Tab') {
        applySuggestion(selectedIndex);
        event.preventDefault();
        return;
      }
    },
    [showPopup, suggestions.length, hidePopup, selectedIndex, applySuggestion]
  );

  const handleFeedback = React.useCallback(
    (suggestionId: string, type: 'positive' | 'negative') => {
      onFeedbackGiven?.(suggestionId, type);
      // Could also send feedback to AI service for learning
    },
    [onFeedbackGiven]
  );

  // Trigger suggestions when content changes or cursor moves
  React.useEffect(() => {
    if (aiEnabled && aiConfig && content) {
      const timeoutId = setTimeout(() => {
        generateSuggestions();
      }, 500); // Debounce

      return () => clearTimeout(timeoutId);
    }
  }, [content, cursorPosition, aiEnabled, aiConfig, generateSuggestions]);

  // Listen for keyboard events
  React.useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  // Hide popup when clicking outside
  React.useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (popupRef.current && !popupRef.current.contains(event.target as Node)) {
        hidePopup();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [hidePopup]);

  if (!showPopup || !editor) return null;

  return (
    <Paper
      ref={popupRef}
      elevation={8}
      sx={{
        position: 'absolute',
        left: popupPosition.x,
        top: popupPosition.y,
        minWidth: 300,
        maxWidth: 600,
        maxHeight: 300,
        overflow: 'auto',
        zIndex: 1000,
        backgroundColor: 'background.paper',
      }}
    >
      {isLoading && suggestions.length === 0 ? (
        <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
          <CircularProgress size={16} />
          <Typography variant="body2">Generating suggestions...</Typography>
        </Box>
      ) : (
        <Box sx={{ p: 1 }}>
          <Box
            sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}
          >
            <Typography variant="subtitle2">AI Suggestions</Typography>
            <IconButton size="small" onClick={hidePopup}>
              <Close fontSize="small" />
            </IconButton>
          </Box>

          {suggestions.map((suggestion, index) => (
            <Box
              key={suggestion.id}
              sx={{
                p: 1,
                mb: 0.5,
                backgroundColor: selectedIndex === index ? 'action.selected' : 'transparent',
                borderRadius: 1,
                cursor: 'pointer',
                '&:hover': {
                  backgroundColor: selectedIndex === index ? 'action.selected' : 'action.hover',
                },
              }}
              onClick={() => applySuggestion(index)}
            >
              <Box
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  mb: 0.5,
                }}
              >
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Typography variant="body2" sx={{ fontFamily: 'monospace', flex: 1 }}>
                    {suggestion.text.length > 60
                      ? `${suggestion.text.substring(0, 60)}...`
                      : suggestion.text}
                  </Typography>
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
                </Box>

                <Box sx={{ display: 'flex', gap: 0.5 }}>
                  <Tooltip title="Accept suggestion">
                    <IconButton size="small" onClick={() => applySuggestion(index)}>
                      <Check fontSize="small" />
                    </IconButton>
                  </Tooltip>
                  <Tooltip title="Good suggestion">
                    <IconButton
                      size="small"
                      onClick={() => handleFeedback(suggestion.id, 'positive')}
                    >
                      <ThumbUp fontSize="small" />
                    </IconButton>
                  </Tooltip>
                  <Tooltip title="Poor suggestion">
                    <IconButton
                      size="small"
                      onClick={() => handleFeedback(suggestion.id, 'negative')}
                    >
                      <ThumbDown fontSize="small" />
                    </IconButton>
                  </Tooltip>
                </Box>
              </Box>

              {suggestion.description && (
                <Typography variant="caption" color="text.secondary">
                  {suggestion.description}
                </Typography>
              )}
            </Box>
          ))}

          <Box sx={{ p: 1, textAlign: 'center', borderTop: 1, borderColor: 'divider' }}>
            <Typography variant="caption" color="text.secondary">
              Use ↑↓ to navigate, Enter/Tab to accept, Esc to cancel
            </Typography>
          </Box>
        </Box>
      )}
    </Paper>
  );
};

export default RealTimeCompletionPopup;
