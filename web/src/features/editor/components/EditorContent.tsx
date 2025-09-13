import * as React from 'react';
import { Box, CircularProgress } from '@mui/material';
import type * as monaco from 'monaco-editor';
import MonacoEditorWrapper from '../../../components/MonacoEditorWrapper';
import type { OnMount } from '@monaco-editor/react';

type EditorContentProps = {
  activeFile: string | null;
  currentFileContent: string;
  onChange: (value?: string) => void;
  isEditorReady: boolean;
  language: string;
  onEditorDidMount: OnMount;
  editorRef: React.RefObject<monaco.editor.IStandaloneCodeEditor | null>;
  monacoRef: React.RefObject<typeof monaco | null>;
  children?: React.ReactNode;
};

const EditorContent: React.FC<EditorContentProps> = ({
  activeFile,
  currentFileContent,
  onChange,
  isEditorReady,
  language,
  onEditorDidMount,
  editorRef,
  monacoRef,
  children,
}) => {
  const handleEditorMount = (
    editor: monaco.editor.IStandaloneCodeEditor,
    monacoInstance: typeof monaco
  ) => {
    if (editorRef) {
      editorRef.current = editor;
    }
    if (monacoRef) {
      monacoRef.current = monacoInstance;
    }
    onEditorDidMount(editor, monacoInstance);
  };

  return (
    <Box
      sx={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        position: 'relative',
      }}
    >
      {!isEditorReady ? (
        <Box
          sx={{
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
          }}
        >
          <CircularProgress />
        </Box>
      ) : null}

      <MonacoEditorWrapper
        key={activeFile || 'empty'}
        language={language}
        theme="vs-dark"
        value={currentFileContent}
        onChange={onChange}
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
        }}
      />
      {children}
    </Box>
  );
};

export default EditorContent;
