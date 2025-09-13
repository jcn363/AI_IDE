import { Alert, Box, Button, Chip, IconButton, Paper, Tooltip, Typography } from '@mui/material';
import React, { useCallback, useEffect, useRef, useState } from 'react';

import { ArrowForward, Build, Code, Refresh } from '@mui/icons-material';

// Import services and types
import { useAIService } from '../../ai/AIProvider';
import { RefactoringPanel } from '../../ai/components/RefactoringPanel';
import type { RefactoringContext, RefactoringTarget, RefactoringType } from '../../ai/types';

// Import editor hooks
import { editorActions } from '../../../store/slices/editorSlice';
import { useAppDispatch, useAppSelector } from '../../../store/store';

/**
 * Example demonstrating how refactoring system integrates with editor
 * This shows the complete integration pattern between:
 * - Editor state management (Redux)
 * - AI services (Context pattern)
 * - LSP integration
 * - UI components
 */
export const RefactoringIntegrationExample: React.FC = () => {
  // ============================================================================
  // REDUX INTEGRATION (Existing IDE Pattern)
  // ============================================================================
  const dispatch = useAppDispatch();
  const editorState = useAppSelector((state) => state.editor);

  // Get current file and selection from editor state
  const { activeFile } = editorState;
  const { selection } = editorState;

  // ============================================================================
  // AI SERVICES INTEGRATION (New Refactoring Services)
  // ============================================================================
  const { refactoringService } = useAIService();

  // ============================================================================
  // LOCAL STATE MANAGEMENT
  // ============================================================================
  const [context, setContext] = useState<RefactoringContext | null>(null);
  const [availableRefactorings, setAvailableRefactorings] = useState<RefactoringType[]>([]);
  const [showRefactoringPanel, setShowRefactoringPanel] = useState(false);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysisProgress, setAnalysisProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  // ============================================================================
  // INTEGRATION METHODS
  // ============================================================================

  /**
   * Analyzes available refactorings for current context
   * Uses the RefactoringService to determine what operations are available
   */
  const analyzeRefactorings = useCallback(
    async (ctx: RefactoringContext) => {
      try {
        setIsAnalyzing(true);
        setAnalysisProgress(0);

        // Simulate analysis progress
        const progressInterval = setInterval(() => {
          setAnalysisProgress((prev) => {
            if (prev < 90) return prev + 10;
            clearInterval(progressInterval);
            return 90;
          });
        }, 100);

        // Call the refactoring service to analyze the context
        const available = await refactoringService.getAvailableRefactorings(ctx);

        // Integration with LSP - get semantic tokens and language server state
        const enhancedAnalysis = await getEnhancedAnalysis(ctx);

        setAvailableRefactorings(available);
        setAnalysisProgress(100);
        clearInterval(progressInterval);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Analysis failed');
      } finally {
        setIsAnalyzing(false);
      }
    },
    [refactoringService]
  );

  /**
   * Integrates with Monaco editor to get current context
   * This would be called from the Monaco editor onMount/onChange handlers
   */
  const updateEditorContext = useCallback(
    (
      filePath: string,
      selection: {
        startLine: number;
        startColumn: number;
        endLine: number;
        endColumn: number;
      } | null,
      cursorPosition: { line: number; character: number } | null
    ) => {
      const newContext: RefactoringContext = {
        filePath,
        selection: selection
          ? {
              start: { line: selection.startLine, character: selection.startColumn },
              end: { line: selection.endLine, character: selection.endColumn },
            }
          : undefined,
        cursorPosition: cursorPosition
          ? {
              line: cursorPosition.line,
              character: cursorPosition.character,
            }
          : undefined,
      };

      setContext(newContext);
      analyzeRefactorings(newContext);
    },
    []
  );

  /**
   * Gets enhanced analysis from LSP and other sources
   */
  const getEnhancedAnalysis = async (ctx: RefactoringContext) => {
    // This would integrate with:
    // 1. Language server for semantic analysis
    // 2. Code analysis for complexity metrics
    // 3. Existing AI services for suggestions
    return {
      symbols: [], // From LSP semantic tokens
      metrics: {
        complexity: 0,
        dependencies: [],
        suggestions: [],
      },
    };
  };

  /**
   * Handles refactoring execution
   * This integrates with:
   * - RefactoringService for the actual transformation
   * - Editor state for real-time updates
   * - Language server for symbol resolution
   */
  const handleRefactoringExecution = useCallback(
    async (type: RefactoringType, ctx: RefactoringContext, options: any) => {
      try {
        // Execute the refactoring
        const result = await refactoringService.executeRefactoring(type, ctx, options);

        if (result.success) {
          // Update editor with the changes
          dispatch(
            editorActions.updateFileContent({
              filePath: ctx.filePath,
              content: applyChangesToContent(ctx.filePath, result.changes),
            })
          );

          // If changes affect other files, update those too
          for (const change of result.changes) {
            if (change.filePath !== ctx.filePath) {
              dispatch(
                editorActions.updateFileContent({
                  filePath: change.filePath,
                  content: applyChangesToContent(change.filePath, [change]),
                })
              );
            }
          }

          // Success feedback
          console.log(`Refactoring '${type}' completed successfully`, result);

          // Refresh available refactorings after the change
          await analyzeRefactorings(ctx);
        } else {
          throw new Error(result.error || 'Refactoring failed');
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Refactoring execution failed');
      }
    },
    [refactoringService, dispatch, analyzeRefactorings]
  );

  /**
   * Applies changes to file content
   * This would integrate with the Monaco editor's editing API
   */
  const applyChangesToContent = (filePath: string, changes: any[]): string => {
    // This would:
    // 1. Get current content from editor state
    // 2. Apply the changes (insert/delete/replace)
    // 3. Return the modified content
    const currentContent = editorState.fileContents[filePath] || '';

    // Simplified implementation - in practice, this would be more sophisticated
    console.log('Applying changes:', changes);
    return currentContent; // Placeholder - actual implementation would apply changes
  };

  // ============================================================================
  // EFFECTS
  // ============================================================================

  // Update context when editor selection changes
  useEffect(() => {
    if (activeFile && selection) {
      updateEditorContext(
        activeFile,
        {
          startLine: selection.start.lineNumber,
          startColumn: selection.startColumn,
          endLine: selection.endLineNumber,
          endColumn: selection.endColumn,
        },
        {
          line: selection.start.lineNumber,
          character: selection.startColumn,
        }
      );
    }
  }, [activeFile, selection, updateEditorContext]);

  // ============================================================================
  // RENDER
  // ============================================================================

  return (
    <Box sx={{ position: 'relative', width: '100%', height: '100%' }}>
      {/* Refactoring Toolbar */}
      <Paper sx={{ p: 1, mb: 2, display: 'flex', gap: 1, alignItems: 'center' }}>
        <Typography variant="h6">AI Refactoring</Typography>

        {/* Current file status */}
        {activeFile && <Chip label={activeFile.split('/').pop()} icon={<Code />} size="small" />}

        {/* Available refactorings indicator */}
        <Box sx={{ flex: 1 }} />
        <Chip
          label={`${availableRefactorings.length} Available`}
          color={availableRefactorings.length > 0 ? 'primary' : 'default'}
          size="small"
        />

        {/* Analysis status */}
        {isAnalyzing && (
          <Chip label={`Analyzing... ${analysisProgress}%`} color="secondary" size="small" />
        )}

        {/* Action buttons */}
        <Button
          size="small"
          variant="contained"
          onClick={() => setShowRefactoringPanel(!showRefactoringPanel)}
          disabled={!context}
          startIcon={<Build />}
        >
          {showRefactoringPanel ? 'Hide' : 'Refactor'}
        </Button>

        {context && (
          <Tooltip title="Refresh analysis">
            <IconButton
              size="small"
              onClick={() => context && analyzeRefactorings(context)}
              disabled={isAnalyzing}
            >
              <Refresh />
            </IconButton>
          </Tooltip>
        )}
      </Paper>

      {/* Error display */}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Quick refactoring suggestions */}
      {availableRefactorings.length > 0 && !showRefactoringPanel && (
        <Paper sx={{ p: 2, mb: 2 }}>
          <Typography variant="subtitle2" sx={{ mb: 1 }}>
            Quick Refactorings
          </Typography>
          <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
            {availableRefactorings.slice(0, 5).map((type) => (
              <Button
                key={type}
                size="small"
                variant="outlined"
                onClick={() => context && handleRefactoringExecution(type, context, {})}
                startIcon={<ArrowForward fontSize="small" />}
              >
                {type.replace('-', ' ').replace(/\b\w/g, (l) => l.toUpperCase())}
              </Button>
            ))}
            <Button size="small" variant="text" onClick={() => setShowRefactoringPanel(true)}>
              View All
            </Button>
          </Box>
        </Paper>
      )}

      {/* Full Refactoring Panel */}
      {showRefactoringPanel && context && (
        <RefactoringPanel
          visible={showRefactoringPanel}
          onClose={() => setShowRefactoringPanel(false)}
          onApplyRefactoring={handleRefactoringExecution}
          availableRefactorings={availableRefactorings}
          currentContext={context}
          configuration={refactoringService.getConfiguration()}
          onConfigurationUpdate={(config) => refactoringService.updateConfiguration(config)}
          isAnalyzing={isAnalyzing}
          analysisProgress={analysisProgress}
        />
      )}

      {/* Integration notes */}
      <Box sx={{ mt: 4, p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
        <Typography variant="h6" color="primary" sx={{ mb: 1 }}>
          🔗 Integration Overview
        </Typography>
        <Typography variant="body2" sx={{ mb: 1 }}>
          <strong>Redux Integration:</strong> Editor state, file contents, and user actions
        </Typography>
        <Typography variant="body2" sx={{ mb: 1 }}>
          <strong>AI Context Integration:</strong> RefactoringService through useAIService hook
        </Typography>
        <Typography variant="body2" sx={{ mb: 1 }}>
          <strong>LSP Integration:</strong> Language server for semantic analysis and symbol
          resolution
        </Typography>
        <Typography variant="body2" sx={{ mb: 1 }}>
          <strong>Monaco Editor Integration:</strong> Selection tracking and content updates
        </Typography>
      </Box>
    </Box>
  );
};

export default RefactoringIntegrationExample;
