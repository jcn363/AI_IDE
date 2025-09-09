import React, { useEffect, useRef, useState, useCallback } from 'react';
import { Box, LinearProgress } from '@mui/material';
import * as monaco from 'monaco-editor';
import { CollaborationManager } from '../CollaborationManager';
import type { UserPresence, ChangeOperation, Conflict } from '../types';
import { PresenceIndicator } from './PresenceIndicator';
import { ConflictResolver } from './ConflictResolver';

interface CollaborativeEditorProps {
  filePath: string;
  language: string;
  content: string;
  onContentChange: (content: string) => void;
  collaborationManager: CollaborationManager;
  onCollaborationError?: (error: Error) => void;
}

export const CollaborativeEditor: React.FC<CollaborativeEditorProps> = ({
  filePath,
  language,
  content,
  onContentChange,
  collaborationManager,
  onCollaborationError,
}) => {
  const editorRef = useRef<HTMLElement>(null);
  const monacoRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [users, setUsers] = useState<UserPresence[]>([]);
  const [conflicts, setConflicts] = useState<Conflict[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [showConflictResolver, setShowConflictResolver] = useState(false);
  const [activeConflict, setActiveConflict] = useState<Conflict | null>(null);

  // Initialize collaboration session
  useEffect(() => {
    const initializeCollaboration = async () => {
      try {
        setIsLoading(true);
        await collaborationManager.joinSession(`file_${filePath.replace(/[\/\\]/g, '_')}`);

        // Set up event listeners
        const unsubscribePresence = collaborationManager.onPresenceUpdate((presence) => {
          if (presence.currentFile === filePath) {
            updateUserPresence(presence);
          }
        });

        const unsubscribeChange = collaborationManager.onChange((operation) => {
          if (operation.filePath === filePath) {
            handleRemoteChange(operation);
          }
        });

        collaborationManager.onConflict((conflict) => {
          if (conflict.filePath === filePath) {
            handleConflict(conflict);
          }
        });

        // Update user's presence to indicate they're editing this file
        collaborationManager.updatePresence({
          currentFile: filePath,
        });

        return () => {
          unsubscribePresence();
          unsubscribeChange();
        };
      } catch (error) {
        console.error('Failed to initialize collaboration:', error);
        if (onCollaborationError) {
          onCollaborationError(error as Error);
        }
      } finally {
        setIsLoading(false);
      }
    };

    initializeCollaboration();
  }, [filePath, collaborationManager, onCollaborationError]);

  // Initialize Monaco editor
  useEffect(() => {
    if (!editorRef.current || monacoRef.current) return;

    const editor = monaco.editor.create(editorRef.current, {
      value: content,
      language,
      theme: 'vs-dark',
      automaticLayout: true,
      fontSize: 14,
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      renderWhitespace: 'boundary',
      renderLineHighlight: 'all',
      folding: true,
      lineNumbers: 'on',
      glyphMargin: true,
    });

    monacoRef.current = editor;

    // Set up editor event listeners for collaboration
    const cursorChangeHandler = editor.onDidChangeCursorPosition(() => {
      const position = editor.getPosition();
      if (position) {
        collaborationManager.broadcastCursor({
          lineNumber: position.lineNumber,
          column: position.column,
        });
      }
    });

    const selectionChangeHandler = editor.onDidChangeCursorSelection(() => {
      const selection = editor.getSelection();
      if (selection) {
        collaborationManager.broadcastSelection(selection);
      }
    });

    const contentChangeHandler = editor.onDidChangeModelContent(({ changes }) => {
      changes.forEach((change) => {
        const operation: ChangeOperation = {
          id: `change_${Date.now()}`,
          userId: 'current_user', // In real implementation, get from auth
          filePath,
          timestamp: Date.now(),
          type: 'insert',
          position: {
            startLine: change.range.startLineNumber,
            startColumn: change.range.startColumn,
            endLine: change.range.endLineNumber,
            endColumn: change.range.endColumn,
          },
          content: change.text,
          previousContent: change.rangeLength ? content.slice(change.range.offset, change.range.offset + change.rangeLength) : undefined,
        };

        collaborationManager.sendChange(operation);
        onContentChange(editor.getValue());
      });
    });

    editor.onDidChangeCursorPosition(() => {
      // Update presence with cursor position
      const position = editor.getPosition();
      if (position) {
        collaborationManager.updatePresence({
          cursorPosition: {
            startLine: position.lineNumber,
            startColumn: position.column,
            endLine: position.lineNumber,
            endColumn: position.column,
          },
        });
      }
    });

    return () => {
      cursorChangeHandler.dispose();
      selectionChangeHandler.dispose();
      contentChangeHandler.dispose();
      editor.dispose();
      monacoRef.current = null;
    };
  }, [content, language, filePath, collaborationManager, onContentChange]);

  // Update content when prop changes
  useEffect(() => {
    if (monacoRef.current && content !== monacoRef.current.getValue()) {
      monacoRef.current.setValue(content);
    }
  }, [content]);

  const updateUserPresence = useCallback((presence: UserPresence) => {
    setUsers(prevUsers => {
      const existingIndex = prevUsers.findIndex(u => u.userId === presence.userId);
      if (existingIndex >= 0) {
        const updated = [...prevUsers];
        updated[existingIndex] = presence;
        return updated;
      } else {
        return [...prevUsers, presence].filter(u =>
          u.currentFile === filePath && u.status === 'online'
        );
      }
    });
  }, [filePath]);

  const handleRemoteChange = useCallback((operation: ChangeOperation) => {
    if (!monacoRef.current) return;

    // Apply remote change to editor
    const monacoOp = monacoRef.current.getModel();
    if (monacoOp) {
      if (operation.type === 'insert' && operation.content) {
        const startPosition = monacoOp.getPositionAt(operation.position.startColumn);
        const endPosition = monacoOp.getPositionAt(operation.position.endColumn);

        startPosition.lineNumber = operation.position.startLine;
        startPosition.column = operation.position.startColumn;
        endPosition.lineNumber = operation.position.endLine;
        endPosition.column = operation.position.endColumn;

        monacoOp.applyEdits([{
          range: new monaco.Range(
            operation.position.startLine,
            operation.position.startColumn,
            operation.position.endLine,
            operation.position.endColumn
          ),
          text: operation.content,
        }], false);
      }
    }
  }, []);

  const handleConflict = useCallback((conflict: Conflict) => {
    setConflicts(prev => [...prev, conflict]);
    setActiveConflict(conflict);
    setShowConflictResolver(true);
  }, []);

  const handleResolveConflict = useCallback((resolution: 'local' | 'remote' | 'merge') => {
    if (activeConflict) {
      collaborationManager.resolveConflict(activeConflict.id, resolution);
      setConflicts(prev => prev.filter(c => c.id !== activeConflict.id));
      setShowConflictResolver(false);
      setActiveConflict(null);
    }
  }, [activeConflict, collaborationManager]);

  const handleMergeConflict = useCallback((mergedContent: string) => {
    if (activeConflict && mergedContent) {
      collaborationManager.resolveConflict(activeConflict.id, 'merge', mergedContent);
      setConflicts(prev => prev.filter(c => c.id !== activeConflict.id));
      setShowConflictResolver(false);
      setActiveConflict(null);
    }
  }, [activeConflict, collaborationManager]);

  // Render user decorations in editor
  useEffect(() => {
    if (!monacoRef.current) return;

    const decorations: string[] = [];

    users.forEach(user => {
      if (user.cursorPosition && user.userId !== 'current_user') {
        const decoration: monaco.editor.IModelDecoration = {
          range: new monaco.Range(
            user.cursorPosition.startLine,
            user.cursorPosition.startColumn,
            user.cursorPosition.endLine,
            user.cursorPosition.endColumn
          ),
          options: {
            className: `collaboration-cursor-${user.userId}`,
            stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
            afterContentClassName: `collaboration-cursor-tooltip collaboration-cursor-${user.userId}`,
            after: {
              content: ` ${user.name}`,
              inlineClassName: `collaboration-cursor-name collaboration-cursor-${user.userId}`,
            },
          },
        };
        decorations.push(monacoRef.current!.deltaDecorations([], [decoration])[0]);
      }
    });

    return () => {
      if (monacoRef.current) {
        monacoRef.current.deltaDecorations(decorations, []);
      }
    };
  }, [users]);

  // Add CSS styles for collaboration cursors
  useEffect(() => {
    const styles = users.map(user => `
      .collaboration-cursor-${user.userId} {
        background-color: ${user.color}40;
        border-left: 2px solid ${user.color};
      }
      .collaboration-cursor-name,
      .collaboration-cursor-tooltip {
        color: ${user.color};
        font-weight: bold;
      }
    `).join('\n');

    const styleEl = document.createElement('style');
    styleEl.textContent = styles;
    document.head.appendChild(styleEl);

    return () => {
      document.head.removeChild(styleEl);
    };
  }, [users]);

  return (
    <Box sx={{ position: 'relative', height: '100%' }}>
      {isLoading && <LinearProgress sx={{ position: 'absolute', top: 0, left: 0, right: 0, zIndex: 10 }} />}

      <Box sx={{ position: 'absolute', top: 8, right: 8, zIndex: 5, display: 'flex', gap: 1 }}>
        {users.map(user => (
          <PresenceIndicator key={user.userId} user={user} size="small" />
        ))}
      </Box>

      {conflicts.length > 0 && (
        <Box sx={{ position: 'absolute', top: 48, right: 8, zIndex: 5 }}>
          <Box
            sx={{
              backgroundColor: 'error.main',
              color: 'error.contrastText',
              px: 2,
              py: 1,
              borderRadius: 1,
              fontSize: '0.875rem',
              cursor: 'pointer',
            }}
            onClick={() => setShowConflictResolver(true)}
          >
            {conflicts.length} conflict{conflicts.length > 1 ? 's' : ''} detected
          </Box>
        </Box>
      )}

      <div ref={editorRef} style={{ height: '100%', width: '100%' }} />

      {showConflictResolver && activeConflict && (
        <ConflictResolver
          conflict={activeConflict}
          onResolve={handleResolveConflict}
          onMerge={handleMergeConflict}
        />
      )}
    </Box>
  );
};