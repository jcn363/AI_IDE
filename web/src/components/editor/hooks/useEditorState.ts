import { useState, useCallback } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import type { RootState } from '../../../store/types';
import { editorActions, EditorTheme } from '../../../store/slices/editorSlice';
import { SnackbarState } from '../types';

export const useEditorState = () => {
  const dispatch = useDispatch();
  const [snackbar, setSnackbar] = useState<SnackbarState>({
    open: false,
    message: '',
    severity: 'info',
  });

  // Selectors
  const {
    theme,
    fontSize,
    fontFamily,
    wordWrap,
    minimap,
    lineNumbers,
    tabSize,
    currentFile,
    fileContent,
    isSaving,
    error,
    openFiles,
    activeFile,
    fileContents,
    fileTree,
    isLoading,
  } = useSelector((state: RootState) => state.editor);

  // Actions
  const setTheme = useCallback(
    (theme: EditorTheme) => {
      dispatch(editorActions.setTheme(theme));
    },
    [dispatch]
  );

  const setFontSize = useCallback(
    (size: number) => {
      dispatch(editorActions.setFontSize(size));
    },
    [dispatch]
  );

  const setWordWrap = useCallback(
    (enabled: boolean) => {
      dispatch(editorActions.setWordWrap(enabled));
    },
    [dispatch]
  );

  const setMinimap = useCallback(
    (enabled: boolean) => {
      dispatch(editorActions.setMinimap(enabled));
    },
    [dispatch]
  );

  const setLineNumbers = useCallback(
    (enabled: boolean) => {
      dispatch(editorActions.setLineNumbers(enabled));
    },
    [dispatch]
  );

  const setTabSize = useCallback(
    (size: number) => {
      dispatch(editorActions.setTabSize(size));
    },
    [dispatch]
  );

  const setCurrentFile = useCallback(
    (filePath: string) => {
      dispatch(editorActions.setCurrentFile(filePath));
    },
    [dispatch]
  );

  const closeFile = useCallback(
    (filePath: string) => {
      dispatch(editorActions.closeFile(filePath));
    },
    [dispatch]
  );

  const switchToFile = useCallback(
    (filePath: string) => {
      dispatch(editorActions.switchToFile(filePath));
    },
    [dispatch]
  );

  const updateFileContent = useCallback(
    (filePath: string, content: string) => {
      dispatch(editorActions.updateFileContent({ filePath, content }));
    },
    [dispatch]
  );

  const saveFile = useCallback(
    async (filePath: string, content: string) => {
      try {
        dispatch(editorActions.saveFileStart(filePath));
        // Here you would typically make an API call to save the file
        // await api.saveFile(filePath, content);
        dispatch(editorActions.saveFileSuccess(filePath));
        setSnackbar({
          open: true,
          message: 'File saved successfully',
          severity: 'success',
        });
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to save file';
        dispatch(editorActions.saveFileFailure(errorMessage));
        setSnackbar({
          open: true,
          message: `Error: ${errorMessage}`,
          severity: 'error',
        });
      }
    },
    [dispatch]
  );

  const handleCloseSnackbar = useCallback(() => {
    setSnackbar((prev) => ({ ...prev, open: false }));
  }, []);

  return {
    // State
    theme,
    fontSize,
    fontFamily,
    wordWrap,
    minimap,
    lineNumbers,
    tabSize,
    currentFile,
    fileContent,
    isSaving,
    error,
    openFiles,
    activeFile,
    fileContents,
    fileTree,
    isLoading,
    snackbar,

    // Actions
    setTheme,
    setFontSize,
    setWordWrap,
    setMinimap,
    setLineNumbers,
    setTabSize,
    setCurrentFile,
    closeFile,
    switchToFile,
    updateFileContent,
    saveFile,
    handleCloseSnackbar,
  };
};
