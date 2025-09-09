// Common types used across the editor components

export interface FileWithContent {
  path: string;
  content: string;
}

export interface FileNode {
  path: string;
  name: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  content?: string;
  lastModified?: number;
}

export interface EditorTheme {
  theme: 'vs-dark' | 'light' | 'hc-black';
  fontSize: number;
  fontFamily: string;
  wordWrap: boolean;
  minimap: boolean;
  lineNumbers: boolean;
  tabSize: number;
}

export interface EditorState {
  theme: 'vs-dark' | 'light' | 'hc-black';
  fontSize: number;
  fontFamily: string;
  wordWrap: boolean;
  minimap: boolean;
  lineNumbers: boolean;
  tabSize: number;
  currentFile: string | null;
  fileContent: string;
  isSaving: boolean;
  error: string | null;
  openFiles: string[];
  activeFile: string | null;
  fileContents: { [key: string]: string };
  lastSaved: Record<string, number>;
  fileTree: FileNode | null;
  isLoading: boolean;
}

export interface UpdateFileContentPayload {
  filePath: string;
  content: string;
}

export interface EditorPane {
  id: string;
  activeFile: string | null;
  files: Array<{
    path: string;
    isPinned: boolean;
  }>;
  path: string;
  isPinned: boolean;
  splitDirection?: 'horizontal' | 'vertical';
  children?: [string, string];
}

export interface SnackbarState {
  open: boolean;
  message: string;
  severity: 'success' | 'error' | 'info' | 'warning';
}

export interface EditorPageProps {
  className?: string;
  style?: React.CSSProperties;
}
