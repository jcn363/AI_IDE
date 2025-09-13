import type { OnMount } from '@monaco-editor/react';
import type * as monaco from 'monaco-editor';

export interface SnackbarState {
  open: boolean;
  message: string;
  severity: 'success' | 'error';
}

export interface EditorContentProps {
  activeFile: string | null;
  currentFileContent: string;
  onChange: (value?: string) => void;
  isEditorReady: boolean;
  language: string;
  onEditorDidMount: OnMount;
  editorRef: React.RefObject<monaco.editor.IStandaloneCodeEditor | null>;
  monacoRef: React.RefObject<typeof monaco | null>;
  children?: React.ReactNode;
}

export function extToLanguageId(ext?: string): string {
  if (!ext) return 'plaintext';
  const extMap: Record<string, string> = {
    js: 'javascript',
    jsx: 'javascript',
    ts: 'typescript',
    tsx: 'typescript',
    py: 'python',
    rs: 'rust',
    go: 'go',
    java: 'java',
    c: 'c',
    cpp: 'cpp',
    h: 'cpp',
    hpp: 'cpp',
    cs: 'csharp',
    php: 'php',
    rb: 'ruby',
    swift: 'swift',
    kt: 'kotlin',
    json: 'json',
    html: 'html',
    css: 'css',
    scss: 'scss',
    less: 'less',
    md: 'markdown',
    xml: 'xml',
    yaml: 'yaml',
    yml: 'yaml',
  };
  return extMap[ext.toLowerCase()] || 'plaintext';
}
