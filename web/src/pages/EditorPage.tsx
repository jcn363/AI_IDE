import * as React from 'react';
import { ReactElement, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type * as monaco from 'monaco-editor';

// MUI Components
import { Alert, Box, CircularProgress, Snackbar, styled, useTheme } from '@mui/material';

// Monaco Editor
import type { OnMount } from '@monaco-editor/react';

// Components
import EditorTabs from '../components/EditorTabs';
import Toolbar from '../features/editor/components/Toolbar';
import EditorContent from '../features/editor/components/EditorContent';
import StatusBar from '../components/StatusBar';

// Store
import { useAppDispatch, useAppSelector } from '../store/store';
import type { RootState } from '../store/types';
import { editorActions } from '../store/slices/editorSlice';
import { tabManagementActions, tabManagementSelectors } from '../store/slices/tabManagementSlice';

// Hooks & Services
import { useLanguageServer } from '../features/language-server/useLanguageServer';
import { useEditorBreakpoints } from '../hooks/useEditorBreakpoints';
import { extToLanguageId } from '../features/editor/utils';

// Types
import { SnackbarState } from '../features/editor/types';

// Styled components
const EditorContainer = styled(Box)(({ theme }) => ({
  display: 'flex',
  flexDirection: 'column',
  height: '100vh',
  width: '100%',
  backgroundColor: theme.palette.background.default,
}));

const EditorWrapper = styled(Box)({
  flex: 1,
  display: 'flex',
  flexDirection: 'column',
  overflow: 'hidden',
  position: 'relative',
});

const EditorPage: React.FC = (): ReactElement => {
  const theme = useTheme();
  const dispatch = useAppDispatch();

  // Editor slice
  const editorState = useAppSelector((state: RootState) => state.editor);
  const isSaving = useAppSelector((state: RootState) => state.editor.isSaving);
  const isLoading = useAppSelector((state: RootState) => state.editor.isLoading);
  const error = useAppSelector((state: RootState) => state.editor.error);
  const navigationTarget = useAppSelector((state: RootState) => state.editor.navigationTarget);

  // Tabs/panes
  const activePane = useAppSelector(tabManagementSelectors.selectActivePane);
  const activePaneId = useAppSelector(tabManagementSelectors.selectActivePaneId);

  // Active file path
  const activeFilePath = activePane?.activeFile || editorState.activeFile || '';

  // Editor refs and state
  const [isEditorReady, setIsEditorReady] = useState<boolean>(false);
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<typeof monaco | null>(null);
  const [snackbar, setSnackbar] = useState<SnackbarState>({
    open: false,
    message: '',
    severity: 'success',
  });

  // Language server
  const languageServerState = useLanguageServer();
  const isConnected = languageServerState.isReady;

  // Current file content
  const currentFileContent = useMemo(() => {
    if (!activeFilePath) return '';
    return editorState.fileContents[activeFilePath] || '';
  }, [activeFilePath, editorState.fileContents]);

  // Language id
  const languageId = useMemo(() => {
    if (!activeFilePath) return 'plaintext';
    const parts = activeFilePath.split('.');
    const ext = parts.length > 1 ? parts.pop() : undefined;
    return extToLanguageId(ext);
  }, [activeFilePath]);

  // Tab actions
  const handleTabChange = useCallback(
    (filePath: string) => {
      if (!activePaneId) return;
      dispatch(
        tabManagementActions.openFileInPane({
          paneId: activePaneId,
          filePath,
        }),
      );
      dispatch(editorActions.setCurrentFile(filePath));
    },
    [activePaneId, dispatch],
  );

  const handleTabClose = useCallback(
    (filePath: string) => {
      if (!activePaneId) return;
      dispatch(
        tabManagementActions.closeFileInPane({
          paneId: activePaneId,
          filePath,
        }),
      );
    },
    [activePaneId, dispatch],
  );

  const renderEditorTabs = useCallback(() => {
    if (!activePaneId) return null;
    const files = (activePane?.files || []).map((f) =>
      (typeof f === 'string' ? { path: f, isPinned: false } : { path: f.path, isPinned: Boolean(f.isPinned) }),
    );

    return (
      <EditorTabs
        paneId={activePaneId}
        files={files}
        activeFile={activePane?.activeFile || null}
        onTabChange={handleTabChange}
        onTabClose={handleTabClose}
      />
    );
  }, [activePaneId, activePane, handleTabChange, handleTabClose]);

  // Editor content change
  const handleEditorChange = useCallback(
    (value?: string) => {
      if (!activePane?.activeFile) return;
      const content = value ?? '';
      dispatch(
        editorActions.updateFileContent({
          filePath: activePane.activeFile,
          content,
        }),
      );
    },
    [activePane?.activeFile, dispatch],
  );

  // Open a file from disk
  const handleOpenFile = useCallback(async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      const selected = await open({ multiple: false, title: 'Open File' });
      if (!selected || typeof selected !== 'string') return;
      const content = await readTextFile(selected);
      if (activePaneId) {
        dispatch(tabManagementActions.openFileInPane({ paneId: activePaneId, filePath: selected }));
      }
      dispatch(editorActions.setCurrentFile(selected));
      dispatch(editorActions.updateFileContent({ filePath: selected, content }));
    } catch (e) {
      console.error('Failed to open file:', e);
      setSnackbar({ open: true, message: 'Failed to open file', severity: 'error' });
    }
  }, [activePaneId, dispatch]);

  // Save file to disk
  const handleSaveFile = useCallback(async () => {
    if (!activeFilePath) return;
    try {
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      const content = editorRef.current ? editorRef.current.getValue() : currentFileContent;
      dispatch(editorActions.saveFileStart(activeFilePath));
      await writeTextFile(activeFilePath, content);
      dispatch(
        editorActions.updateFileContent({
          filePath: activeFilePath,
          content,
        }),
      );
      dispatch(editorActions.saveFileSuccess(activeFilePath));
      setSnackbar({
        open: true,
        message: 'File saved successfully',
        severity: 'success',
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to save file';
      dispatch(editorActions.saveFileFailure(msg));
      setSnackbar({
        open: true,
        message: `Error saving file: ${msg}`,
        severity: 'error',
      });
    }
  }, [activeFilePath, currentFileContent, dispatch]);

  // Run code
  const handleRunCode = useCallback(async () => {
    // TODO: Implement run code functionality
  }, []);

  // Breakpoints hook
  const { onEditorDidMount: onMountWithBreakpoints } = useEditorBreakpoints({
    activeFilePath,
    currentFileContent,
    editorRef,
    monacoRef,
  });

  // Editor mount handler to also handle isEditorReady state
  const handleEditorDidMount = useCallback<OnMount>(
    (editor, monacoInstance) => {
      editorRef.current = editor;
      monacoRef.current = monacoInstance;
      setIsEditorReady(true);
      onMountWithBreakpoints(editor, monacoInstance);
    },
    [onMountWithBreakpoints],
  );

  // Respond to navigation target
  useEffect(() => {
    if (!navigationTarget) return;
    const { filePath, line, column } = navigationTarget;

    if (filePath && activeFilePath !== filePath) {
      // Switch to/open file first; then this effect will run again
      dispatch(editorActions.setCurrentFile(filePath));
      if (activePaneId) {
        dispatch(tabManagementActions.openFileInPane({ paneId: activePaneId, filePath }));
      }
      return;
    }

    if (filePath && activeFilePath === filePath && editorRef.current) {
      const ed = editorRef.current;
      const pos = {
        lineNumber: Math.max(1, typeof line === 'number' ? line : 1),
        column: Math.max(1, typeof column === 'number' ? column : 1),
      };
      ed.setPosition(pos);
      ed.revealPositionInCenter(pos);
      ed.focus();
      dispatch(editorActions.setNavigationTarget(null));
    }
  }, [navigationTarget, activeFilePath, activePaneId, dispatch]);

  return (
    <EditorContainer>
      <Toolbar
        onSave={handleSaveFile}
        onOpenFile={handleOpenFile}
        onRunCode={handleRunCode}
        onOpenSettings={() => {
          /* TODO: Implement settings */
        }}
        isSaving={isSaving}
      />
      <EditorWrapper>
        {renderEditorTabs()}
        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          <EditorContent
            activeFile={activeFilePath}
            currentFileContent={currentFileContent}
            onChange={handleEditorChange}
            isEditorReady={isEditorReady && !isLoading}
            language={languageId}
            onEditorDidMount={handleEditorDidMount}
            editorRef={editorRef}
            monacoRef={monacoRef}
          >
            {isLoading && (
              <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
                <CircularProgress />
              </Box>
            )}
          </EditorContent>
        </Box>
      </EditorWrapper>

      <StatusBar activeFilePath={activeFilePath} isSaving={isSaving} isConnected={isConnected} />

      {/* Snackbar */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={() => setSnackbar((prev) => ({ ...prev, open: false }))}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        sx={{ bottom: { xs: 24, sm: 24 } }}
      >
        <Alert
          onClose={() => setSnackbar((prev) => ({ ...prev, open: false }))}
          severity={snackbar.severity}
          sx={{ width: '100%' }}
          variant="filled"
        >
          {snackbar.message}
        </Alert>
      </Snackbar>
    </EditorContainer>
  );
};

export default EditorPage;