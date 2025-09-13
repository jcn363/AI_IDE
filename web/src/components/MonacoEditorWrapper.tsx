// @ts-nocheck
import React from 'react';
import { Box, CircularProgress } from './shared/MaterialUI';

// Direct import with type checking disabled
const MonacoEditor = React.lazy(() => import('@monaco-editor/react'));

interface MonacoEditorWrapperProps {
  height?: string | number;
  language: string;
  theme: string;
  value: string;
  onChange: (value: string | undefined, event: any) => void;
  onMount: (editor: any, monaco: any) => void;
  options: any;
  loading?: React.ReactNode;
  onEditorReady?: (editor: any) => void; // New callback for multi-cursor integration
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
    <React.Suspense fallback={loading || defaultLoading}>
      <MonacoEditor
        height={height}
        language={language}
        theme={theme}
        value={value}
        onChange={onChange}
        onMount={handleEditorMount}
        options={options}
      />
    </React.Suspense>
  );
};

export default MonacoEditorWrapper;
