import { PayloadAction, createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import type { TabManagementState } from './tabManagementSlice';

// Types
export type EditorTheme = 'vs' | 'vs-dark' | 'hc-black';

// Define the shape of our root state
export interface RootState {
  editor: EditorState;
  tabManagement: TabManagementState;
}

export interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  content?: string;
  lastModified?: number;
}

interface EditorPane {
  id: string;
  activeFile: string | null;
  files: Array<{
    path: string;
    isPinned: boolean;
  }>;
  splitDirection?: 'horizontal' | 'vertical';
  children?: [string, string]; // IDs of child panes
}

export interface EditorState {
  selection: any;
  theme: EditorTheme;
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
  lastSaved: Record<string, number>; // Map of file paths to last saved timestamp
  fileTree: FileNode | null;
  isLoading: boolean;
  navigationTarget?: { filePath: string; line?: number; column?: number } | null;
}

interface UpdateFileContentPayload {
  filePath: string;
  content: string;
}

// Initial state
const createPane = (
  id: string,
  activeFile: string | null = null,
  files: string[] = []
): EditorPane => ({
  id,
  activeFile,
  files: files.map((path) => ({ path, isPinned: false })),
});

const initialState: EditorState = {
  theme: 'vs-dark',
  fontSize: 14,
  fontFamily: '"Fira Code", "Courier New", monospace',
  wordWrap: true,
  minimap: true,
  lineNumbers: true,
  tabSize: 2,
  currentFile: null,
  fileContent: '',
  isSaving: false,
  error: null,
  openFiles: [],
  activeFile: null,
  fileContents: {},
  lastSaved: {},
  fileTree: null,
  isLoading: false,
  navigationTarget: null,
  selection: undefined,
};

// Async thunks
export const loadFileTree = createAsyncThunk<FileNode, string>(
  'editor/loadFileTree',
  async (rootPath: string) => {
    // In a real app, this would be an API call to the backend
    // For now, we'll use a mock implementation
    return new Promise((resolve) => {
      setTimeout(() => {
        const mockFileTree: FileNode = {
          name: 'project',
          path: '/project',
          type: 'directory',
          children: [
            {
              name: 'src',
              path: '/project/src',
              type: 'directory',
              children: [
                {
                  name: 'main.rs',
                  path: '/project/src/main.rs',
                  type: 'file',
                  content: 'fn main() {\n    println!("Hello, world!");\n}\n',
                  lastModified: Date.now(),
                },
                {
                  name: 'lib.rs',
                  path: '/project/src/lib.rs',
                  type: 'file',
                  content: '// Your library code here\n',
                  lastModified: Date.now() - 1000 * 60 * 60, // 1 hour ago
                },
              ],
            },
            {
              name: 'Cargo.toml',
              path: '/project/Cargo.toml',
              type: 'file',
              content:
                '[package]\nname = "my-project"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n',
              lastModified: Date.now() - 1000 * 60 * 60 * 24, // 1 day ago
            },
          ],
        };
        resolve(mockFileTree);
      }, 300);
    });
  }
);

// Helper function to find a file in the file tree
const findFileInTree = (node: FileNode, path: string): FileNode | null => {
  if (node.path === path) return node;
  if (node.children) {
    for (const child of node.children) {
      const found = findFileInTree(child, path);
      if (found) return found;
    }
  }
  return null;
};

// Create the slice
const editorSlice = createSlice({
  name: 'editor',
  initialState,
  reducers: {
    setTheme: (state, action: PayloadAction<EditorTheme>) => {
      state.theme = action.payload;
    },
    setFontSize: (state, action: PayloadAction<number>) => {
      state.fontSize = action.payload;
    },
    setFontFamily: (state, action: PayloadAction<string>) => {
      state.fontFamily = action.payload;
    },
    setWordWrap: (state, action: PayloadAction<boolean>) => {
      state.wordWrap = action.payload;
    },
    setMinimap: (state, action: PayloadAction<boolean>) => {
      state.minimap = action.payload;
    },
    setLineNumbers: (state, action: PayloadAction<boolean>) => {
      state.lineNumbers = action.payload;
    },
    setTabSize: (state, action: PayloadAction<number>) => {
      state.tabSize = action.payload;
    },
    setCurrentFile: (state, action: PayloadAction<string>) => {
      state.currentFile = action.payload;
      if (!state.openFiles.includes(action.payload)) {
        state.openFiles.push(action.payload);
      }
      state.activeFile = action.payload;
    },
    closeFile: (state, action: PayloadAction<string>) => {
      const filePath = action.payload;
      state.openFiles = state.openFiles.filter((file) => file !== filePath);
      if (state.activeFile === filePath) {
        state.activeFile = state.openFiles[0] || null;
      }
    },
    switchToFile: (state, action: PayloadAction<string>) => {
      const filePath = action.payload;
      if (state.openFiles.includes(filePath)) {
        state.activeFile = filePath;
      }
    },
    updateFileContent: (state, action: PayloadAction<UpdateFileContentPayload>) => {
      const { filePath, content } = action.payload;
      state.fileContents[filePath] = content;
      if (state.activeFile === filePath) {
        state.fileContent = content;
      }
    },
    saveFileStart: (state, action: PayloadAction<string>) => {
      state.isSaving = true;
      state.error = null;
      state.activeFile = action.payload;
    },
    saveFileSuccess: (state, action: PayloadAction<string>) => {
      state.isSaving = false;
      state.lastSaved[action.payload] = Date.now();
    },
    saveFileFailure: (state, action: PayloadAction<string>) => {
      state.isSaving = false;
      state.error = action.payload;
    },
    clearError: (state) => {
      state.error = null;
    },
    setNavigationTarget: (
      state,
      action: PayloadAction<{ filePath: string; line?: number; column?: number } | null>
    ) => {
      state.navigationTarget = action.payload;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(loadFileTree.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(loadFileTree.fulfilled, (state, action) => {
        state.isLoading = false;
        state.fileTree = action.payload;

        // If we have a current file, update its content from the tree
        if (state.currentFile) {
          const file = findFileInTree(action.payload, state.currentFile);
          if (file && file.content !== undefined) {
            state.fileContent = file.content;
          }
        }
      })
      .addCase(loadFileTree.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.error.message || 'Failed to load file tree';
      });
  },
});

// Export actions
export const {
  setTheme,
  setFontSize,
  setFontFamily,
  setWordWrap,
  setMinimap,
  setLineNumbers,
  setTabSize,
  setCurrentFile,
  updateFileContent,
  closeFile,
  switchToFile,
  saveFileStart,
  saveFileSuccess,
  saveFileFailure,
  clearError,
  setNavigationTarget,
} = editorSlice.actions;

// Export action creators for use in components
export const editorActions = {
  setTheme,
  setFontSize,
  setFontFamily,
  setWordWrap,
  setMinimap,
  setLineNumbers,
  setTabSize,
  setCurrentFile,
  closeFile,
  switchToFile,
  updateFileContent,
  saveFileStart,
  saveFileSuccess,
  saveFileFailure,
  clearError,
  setNavigationTarget,
};

// Export selectors
export const selectEditor = (state: RootState) => state.editor;

export const selectCurrentFile = (state: RootState) => {
  if (!state.editor.currentFile) return null;
  return {
    path: state.editor.currentFile,
    content: state.editor.fileContents[state.editor.currentFile] || '',
  };
};
// Export all selectors in one object for easier imports
export const editorSelectors = {
  selectEditor: (state: RootState) => state.editor,
  selectCurrentFile: (state: RootState) => {
    if (!state.editor.currentFile) return null;
    return {
      path: state.editor.currentFile,
      content: state.editor.fileContents[state.editor.currentFile] || '',
    };
  },
  selectIsSaving: (state: RootState) => state.editor.isSaving,
  selectIsLoading: (state: RootState) => state.editor.isLoading,
  selectError: (state: RootState) => state.editor.error,
  selectActiveFile: (state: RootState) => state.editor.activeFile,
  selectFileContent: (state: RootState, filePath: string) =>
    state.editor.fileContents[filePath] || '',
} as const;

export default editorSlice.reducer;
