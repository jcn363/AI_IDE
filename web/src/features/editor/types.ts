import type { OnMount } from '@monaco-editor/react';
import type * as monaco from 'monaco-editor';

export type FileWithContent = {
  path: string;
  content: string;
};

export type FileNode = {
  path: string;
  name: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  content?: string;
};

export interface SnackbarState {
  open: boolean;
  message: string;
  severity: 'success' | 'error';
}

export interface EditorContentProps {
  activeFile: string | null;
  currentFileContent: string;
  handleSaveFile: () => void;
  isSaving: boolean;
  isEditorReady: boolean;
  onEditorDidMount: OnMount;
  editorRef: React.RefObject<monaco.editor.IStandaloneCodeEditor | null>;
  monacoRef: React.RefObject<typeof monaco | null>;
  children?: React.ReactNode;
}

export interface EditorTabsProps {
  activeFile: string | null;
  files: string[];
  onTabClick: (file: string) => void;
  onTabClose: (file: string) => void;
  onTabMove: (dragIndex: number, hoverIndex: number) => void;
}

export interface TerminalProps {
  terminalOpen: boolean;
  terminalProgram: string;
  terminalArgs: string;
  terminalDir: string;
  terminalId: string;
  terminalLines: string[];
  onTerminalProgramChange: (value: string) => void;
  onTerminalArgsChange: (value: string) => void;
  onTerminalDirChange: (value: string) => void;
  onStartTerminal: () => void;
  onCloseTerminal: () => void;
}

export interface EditorPageProps {
  className?: string;
  style?: React.CSSProperties;
}
