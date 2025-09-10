import * as React from 'react';
import { Box, CircularProgress, Chip, Typography } from '@mui/material';
import type * as monaco from 'monaco-editor';
import MonacoEditorWrapper from '../../../components/MonacoEditorWrapper';
import { EmbedAIService } from '../../ai/services/EmbedAI';
import type { AIAnalysisConfig } from '../../ai/types';
import type { OnMount } from '@monaco-editor/react';

type AIEditorContentProps = {
  activeFile: string | null;
  currentFileContent: string;
  onChange: (value?: string) => void;
  isEditorReady: boolean;
  language: string;
  onEditorDidMount: OnMount;
  editorRef: React.RefObject<monaco.editor.IStandaloneCodeEditor | null>;
  monacoRef: React.RefObject<typeof monaco | null>;
  children?: React.ReactNode;
  aiEnabled?: boolean;
  aiConfig?: AIAnalysisConfig;
};

type AISuggestionItem = {
  text: string;
  confidence: number;
  type: 'completion' | 'refinement' | 'suggestion';
  metadata?: any;
};

const CONFIDENCE_THRESHOLDS = {
  HIGH: 0.8,
  MEDIUM: 0.6,
  LOW: 0.4
};

const getConfidenceColor = (confidence: number): 'success' | 'warning' | 'error' => {
  if (confidence >= CONFIDENCE_THRESHOLDS.HIGH) return 'success';
  if (confidence >= CONFIDENCE_THRESHOLDS.MEDIUM) return 'warning';
  return 'error';
};

const getConfidenceLabel = (confidence: number): string => {
  if (confidence >= CONFIDENCE_THRESHOLDS.HIGH) return 'High';
  if (confidence >= CONFIDENCE_THRESHOLDS.MEDIUM) return 'Medium';
  return 'Low';
};

const AIEditorContent: React.FC<AIEditorContentProps> = ({
  activeFile,
  currentFileContent,
  onChange,
  isEditorReady,
  language,
  onEditorDidMount,
  editorRef,
  monacoRef,
  children,
  aiEnabled = true,
  aiConfig
}) => {
  const [isAILoading, setIsAILoading] = React.useState(false);
  const [currentSuggestion, setCurrentSuggestion] = React.useState<AISuggestionItem | null>(null);
  const [cursorPosition, setCursorPosition] = React.useState<monaco.IPosition>({ lineNumber: 0, column: 0 });
  const completionTimeoutRef = React.useRef<NodeJS.Timeout | null>(null);

  const aiService = React.useMemo(() => EmbedAIService.getInstance(), []);

  const handleEditorChange = React.useCallback((value: string | undefined) => {
    onChange(value);

    // Clear previous suggestion and timeout
    setCurrentSuggestion(null);
    if (completionTimeoutRef.current) {
      clearTimeout(completionTimeoutRef.current);
    }

    // Trigger AI completion with debounce
    if (aiEnabled && value && aiConfig) {
      completionTimeoutRef.current = setTimeout(() => {
        requestAISuggestion(value);
      }, 1000); // 1 second debounce
    }
  }, [onChange, aiEnabled, aiConfig]);

  const requestAISuggestion = React.useCallback(async (code: string) => {
    if (!aiEnabled || !aiConfig) return;

    try {
      setIsAILoading(true);

      // Get completion from AI service
      const completion = await aiService.getCompletion(
        `Complete this ${language} code snippet intelligently:`,
        [code],
        aiConfig
      );

      // Calculate confidence based on completion length and context match
      const confidence = Math.min(
        (completion.length / 200) * 0.5 + // Length-based confidence
        0.5, // Base confidence
        1.0 // Cap at 100%
      );

      setCurrentSuggestion({
        text: completion,
        confidence,
        type: 'completion'
      });
    } catch (error) {
      console.error('AI completion error:', error);
    } finally {
      setIsAILoading(false);
    }
  }, [aiEnabled, aiConfig, language, aiService]);

  const handleEditorMount = React.useCallback((
    editor: monaco.editor.IStandaloneCodeEditor,
    monacoInstance: typeof monaco,
  ) => {
    if (editorRef) {
      editorRef.current = editor;
    }
    if (monacoRef) {
      monacoRef.current = monacoInstance;
    }

    // Track cursor position for AI suggestions
    editor.onDidChangeCursorPosition((e) => {
      setCursorPosition(e.position);
    });

    // Listen for content changes to trigger AI suggestions
    editor.onDidChangeModelContent(() => {
      const content = editor.getValue();
      // Debounce content change handling
      if (completionTimeoutRef.current) {
        clearTimeout(completionTimeoutRef.current);
      }
      if (aiEnabled && aiConfig) {
        completionTimeoutRef.current = setTimeout(() => {
          requestAISuggestion(content);
        }, 1500); // Longer debounce for content changes
      }
    });

    onEditorDidMount(editor, monacoInstance);
  }, [editorRef, monacoRef, onEditorDidMount, aiEnabled, aiConfig, requestAISuggestion]);

  const insertSuggestion = React.useCallback(() => {
    if (!editorRef.current || !currentSuggestion) return;

    const editor = editorRef.current;
    const model = editor.getModel();
    if (!model) return;

    // Get current position
    const position = editor.getPosition();
    if (!position) return;

    // Insert suggestion at cursor
    const range = new monaco.Range(
      position.lineNumber,
      position.column,
      position.lineNumber,
      position.column
    );

    const edit = {
      range,
      text: currentSuggestion.text
    };

    editor.executeEdits('ai-completion', [edit]);

    // Clear suggestion after insertion
    setCurrentSuggestion(null);
  }, [currentSuggestion, editorRef]);

  return (
    <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', position: 'relative' }}>
      {!isEditorReady ? (
        <Box sx={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          backgroundColor: 'rgba(0, 0, 0, 0.5)',
          zIndex: 1,
        }}>
          <CircularProgress />
        </Box>
      ) : null}

      {/* AI Suggestion Panel */}
      {currentSuggestion && (
        <Box sx={{
          position: 'absolute',
          bottom: 0,
          left: 0,
          right: 0,
          backgroundColor: 'background.paper',
          borderTop: '1px solid',
          borderColor: 'divider',
          padding: 2,
          zIndex: 2,
          display: 'flex',
          alignItems: 'center',
          gap: 2
        }}>
          <Box sx={{ flex: 1 }}>
            <Typography variant="body2" sx={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>
              <Box component="span" sx={{ color: 'text.secondary' }}>AI Suggestion:</Box>
              {' '}
              {currentSuggestion.text.length > 100
                ? `${currentSuggestion.text.substring(0, 100)}...`
                : currentSuggestion.text
              }
            </Typography>
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Chip
              label={`${getConfidenceLabel(currentSuggestion.confidence)} (${Math.round(currentSuggestion.confidence * 100)}%)`}
              color={getConfidenceColor(currentSuggestion.confidence)}
              size="small"
              variant="outlined"
            />
            <Chip
              label="Apply"
              color="primary"
              size="small"
              onClick={insertSuggestion}
              sx={{ cursor: 'pointer' }}
            />
            <Chip
              label="Dismiss"
              variant="outlined"
              size="small"
              onClick={() => setCurrentSuggestion(null)}
              sx={{ cursor: 'pointer' }}
            />
          </Box>
        </Box>
      )}

      {/* AI Loading Indicator */}
      {isAILoading && (
        <Box sx={{
          position: 'absolute',
          top: 10,
          right: 10,
          zIndex: 3,
          display: 'flex',
          alignItems: 'center',
          gap: 1,
          backgroundColor: 'background.paper',
          padding: 1,
          borderRadius: 1,
          border: '1px solid',
          borderColor: 'divider'
        }}>
          <CircularProgress size={16} />
          <Typography variant="caption">AI thinking...</Typography>
        </Box>
      )}

      <MonacoEditorWrapper
        key={activeFile || 'empty'}
        language={language}
        theme="vs-dark"
        value={currentFileContent}
        onChange={handleEditorChange}
        onMount={handleEditorMount}
        options={{
          automaticLayout: true,
          fontSize: 14,
          formatOnPaste: true,
          formatOnType: true,
          minimap: { enabled: true },
          scrollBeyondLastLine: false,
          wordWrap: 'on',
          readOnly: !isEditorReady,
          // Add some AI-related editor features
          acceptSuggestionOnCommitCharacter: true,
          acceptSuggestionOnEnter: 'smart',
          tabCompletion: 'on',
          snippetSuggestions: 'inline',
          suggest: {
            showKeywords: true,
            showSnippets: true,
            showVariables: true,
            showFunctions: true,
            showProperties: true,
            showMethods: true,
            showModules: true,
            showClasses: true,
            showInterfaces: true
          }
        }}
      />
      {children}
    </Box>
  );
};

export default AIEditorContent;