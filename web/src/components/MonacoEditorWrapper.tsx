import React, { Suspense } from 'react';
import { Box, CircularProgress } from './shared/MaterialUI';

// Lazy load Monaco Editor components for code splitting
const MonacoEditor = React.lazy(() =>
  import('@monaco-editor/react').then(module => ({ default: module.default }))
);

// Type definitions for Monaco Editor with improved type safety
type MonacoEditorInstance = {
  getValue: () => string;
  setValue: (value: string) => void;
  getModel: () => unknown;
  focus: () => void;
  layout: () => void;
  onDidChangeModelContent?: (callback: () => void) => void;
  onDidFocusEditorText?: (callback: () => void) => void;
  onDidBlurEditorText?: (callback: () => void) => void;
};

type MonacoInstance = {
  editor: {
    createModel: (value: string, language?: string) => unknown;
    setModelLanguage: (model: unknown, language: string) => void;
    getModel: (uri: unknown) => unknown;
  };
  languages: {
    register: (registration: unknown) => void;
    setMonarchTokensProvider: (languageId: string, provider: unknown) => void;
  };
};

type MonacoEditorChangeEvent = {
  changes: Array<{
    range: unknown;
    rangeLength: number;
    text: string;
    rangeOffset: number;
  }>;
  eol: string;
  versionId: number;
};

interface MonacoEditorWrapperProps {
  height?: string | number;
  language: string;
  theme: string;
  value: string;
  onChange: (value: string | undefined, event: MonacoEditorChangeEvent) => void;
  onMount: (editor: MonacoEditorInstance, monaco: MonacoInstance) => void;
  options: Record<string, unknown>;
  loading?: React.ReactNode;
  onEditorReady?: (editor: MonacoEditorInstance) => void;
}

const MonacoEditorWrapper = ({
  height = '100%',
  language,
  theme,
  value,
  onChange,
  onMount,
  options,
  loading,
  onEditorReady,
}: MonacoEditorWrapperProps) => {
  const defaultLoading = (
    <Box
      sx={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100%',
        width: '100%',
      }}
    >
      <CircularProgress />
    </Box>
  );

  const handleEditorMount = (editor: any, monaco: any) => {
    // Call original onMount callback
    if (onMount) {
      onMount(editor, monaco);
    }

    // Call multi-cursor ready callback
    if (onEditorReady) {
      onEditorReady(editor);
    }
  };

  return (
    <Suspense fallback={loading || defaultLoading}>
      <MonacoEditor
        height={height}
        language={language}
        theme={theme}
        value={value}
        onChange={onChange}
        onMount={handleEditorMount}
        options={options}
      />
    </Suspense>
  );
};

export default MonacoEditorWrapper;