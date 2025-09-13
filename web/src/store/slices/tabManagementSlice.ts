import { createSlice, PayloadAction, createSelector } from '@reduxjs/toolkit';
import type { RootState } from '../types';

export interface EditorPane {
  id: string;
  activeFile: string | null;
  files: Array<{
    path: string;
    isPinned: boolean;
  }>;
  splitDirection?: 'horizontal' | 'vertical';
  children?: [string, string];
}

interface DragState {
  isDragging: boolean;
  sourcePaneId: string | null;
  sourceIndex: number | null;
  targetPaneId: string | null;
  targetIndex: number | null;
}

export interface TabManagementState {
  editorPanes: Record<string, EditorPane>;
  rootPaneId: string;
  activePaneId: string;
  dragState: DragState;
}

const createPane = (
  id: string,
  activeFile: string | null = null,
  files: string[] = []
): EditorPane => ({
  id,
  activeFile,
  files: files.map((path) => ({ path, isPinned: false })),
});

const initialState: TabManagementState = {
  editorPanes: {
    'pane-1': createPane('pane-1'),
  },
  rootPaneId: 'pane-1',
  activePaneId: 'pane-1',
  dragState: {
    isDragging: false,
    sourcePaneId: null,
    sourceIndex: null,
    targetPaneId: null,
    targetIndex: null,
  },
};

const tabManagementSlice = createSlice({
  name: 'tabManagement',
  initialState,
  reducers: {
    // Set the active pane
    setActivePane: (state, action: PayloadAction<string>) => {
      if (state.editorPanes[action.payload]) {
        state.activePaneId = action.payload;
      }
    },

    // Split a pane in the specified direction
    splitPane: (
      state,
      action: PayloadAction<{ paneId: string; direction: 'horizontal' | 'vertical' }>
    ) => {
      const { paneId, direction } = action.payload;
      const pane = state.editorPanes[paneId];

      if (!pane || pane.children) return;

      // Generate deterministic pane IDs based on current state
      const existingIds = Object.keys(state.editorPanes)
        .map((id) => parseInt(id.replace('pane-', ''), 10))
        .filter((n) => !isNaN(n));
      const maxId = existingIds.length ? Math.max(...existingIds) : 0;
      const newPane1Id = `pane-${maxId + 1}`;
      const newPane2Id = `pane-${maxId + 2}`;

      // Create new panes
      state.editorPanes[newPane1Id] = createPane(
        newPane1Id,
        pane.activeFile,
        pane.files.map((f) => f.path)
      );
      state.editorPanes[newPane2Id] = createPane(newPane2Id);

      // Update parent pane to be a split container
      pane.splitDirection = direction;
      pane.children = [newPane1Id, newPane2Id];
      pane.activeFile = null;
      pane.files = [];

      // Set the first pane as active
      state.activePaneId = newPane1Id;
    },

    // Open a file in a specific pane
    openFileInPane: (state, action: PayloadAction<{ paneId: string; filePath: string }>) => {
      const { paneId, filePath } = action.payload;
      const pane = state.editorPanes[paneId];
      if (!pane) return;

      // Add file to pane if not already present
      // Check if file is already open in this pane
      const existingFileIndex = pane.files.findIndex((f: { path: string }) => f.path === filePath);

      if (existingFileIndex === -1) {
        // Add new file to the beginning of the files array if not pinned, otherwise after last pinned file
        const lastPinnedIndex = [...pane.files]
          .reverse()
          .findIndex((f: { isPinned: boolean }) => f.isPinned);
        const insertAt = lastPinnedIndex === -1 ? 0 : pane.files.length - lastPinnedIndex;
        pane.files.splice(insertAt, 0, { path: filePath, isPinned: false });
      }

      // Set as active file in this pane
      pane.activeFile = filePath;
      state.activePaneId = paneId;

      // Set as active
      pane.activeFile = filePath;
      state.activePaneId = paneId;
    },

    // Close a file in a specific pane
    closeFileInPane: (state, action: PayloadAction<{ paneId: string; filePath: string }>) => {
      const { paneId, filePath } = action.payload;
      const pane = state.editorPanes[paneId];
      if (!pane) return;

      const fileIndex = pane.files.findIndex((f) => f.path === filePath);
      if (fileIndex === -1) return;

      // Remove the file from the pane
      pane.files.splice(fileIndex, 1);

      // If this was the active file, update active file
      if (pane.activeFile === filePath) {
        pane.activeFile = pane.files[fileIndex]?.path || pane.files[fileIndex - 1]?.path || null;
      }

      // If this was the last file in the pane, close the pane if it's not the root
      if (pane.files.length === 0 && paneId !== state.rootPaneId) {
        // Find parent pane and remove the reference to this pane
        const parentPane = Object.values(state.editorPanes).find((p) =>
          p.children?.includes(paneId)
        );

        if (parentPane) {
          // If parent has only one child left, merge it up
          const otherChildId = parentPane.children?.find((id) => id !== paneId);
          if (otherChildId) {
            const otherChild = state.editorPanes[otherChildId];
            if (otherChild) {
              // Move all files to the other child
              otherChild.files = [...pane.files, ...otherChild.files];
              otherChild.activeFile = otherChild.files[0]?.path || null;

              // Update parent to be a regular pane
              parentPane.files = [...otherChild.files];
              parentPane.activeFile = otherChild.activeFile;
              parentPane.splitDirection = undefined;
              parentPane.children = undefined;

              // Clean up
              delete state.editorPanes[paneId];
              delete state.editorPanes[otherChildId];

              // Update active pane if needed
              if (state.activePaneId === paneId || state.activePaneId === otherChildId) {
                state.activePaneId = parentPane.id;
              }
            }
          }
        }
      }
    },

    // Move a tab within or between panes
    moveTab: (
      state,
      action: PayloadAction<{
        sourcePaneId: string;
        sourceIndex: number;
        targetPaneId: string;
        targetIndex: number;
      }>
    ) => {
      const { sourcePaneId, sourceIndex, targetPaneId, targetIndex } = action.payload;
      const sourcePane = state.editorPanes[sourcePaneId];
      const targetPane = state.editorPanes[targetPaneId];

      if (!sourcePane || !targetPane) return;

      const [movedTab] = sourcePane.files.splice(sourceIndex, 1);
      targetPane.files.splice(targetIndex, 0, movedTab);

      // Update active file if needed
      if (sourcePane.activeFile === movedTab.path) {
        if (sourcePane.files.length > 0) {
          sourcePane.activeFile =
            sourcePane.files[Math.min(sourceIndex, sourcePane.files.length - 1)].path;
        } else {
          sourcePane.activeFile = null as any;
        }
      }

      // If moving to a different pane, update active file
      if (sourcePaneId !== targetPaneId) {
        targetPane.activeFile = movedTab.path;
        state.activePaneId = targetPaneId;
      }
    },

    // Toggle pin state of a tab
    togglePinTab: (state, action: PayloadAction<{ paneId: string; filePath: string }>) => {
      const { paneId, filePath } = action.payload;
      const pane = state.editorPanes[paneId];
      if (!pane) return;

      const file = pane.files.find((f) => f.path === filePath);
      if (file) {
        const wasPinned = file.isPinned;
        file.isPinned = !wasPinned;

        // Move the file to the appropriate position
        const fileIndex = pane.files.findIndex((f) => f.path === filePath);
        if (fileIndex !== -1) {
          pane.files.splice(fileIndex, 1);

          if (file.isPinned) {
            // Move to the beginning of the pinned section
            const firstUnpinnedIndex = pane.files.findIndex(
              (f: { isPinned: boolean }) => !f.isPinned
            );
            const insertAt = firstUnpinnedIndex === -1 ? pane.files.length : firstUnpinnedIndex;
            pane.files.splice(insertAt, 0, file);
          } else {
            // Move to the end of the unpinned section
            const lastPinnedIndex = pane.files.reduce<number>(
              (acc, f, idx) => (f.isPinned ? idx : acc),
              -1
            );
            const insertAt = lastPinnedIndex === -1 ? 0 : lastPinnedIndex + 1;
            pane.files.splice(insertAt, 0, file);
          }
        }
      }
    },

    // Drag and drop
    startDragTab: (state, action: PayloadAction<{ paneId: string; index: number }>) => {
      const { paneId, index } = action.payload;
      const pane = state.editorPanes[paneId];

      if (!pane) return;

      state.dragState = {
        isDragging: true,
        sourcePaneId: paneId,
        sourceIndex: index,
        targetPaneId: paneId,
        targetIndex: index,
      };
    },

    updateDragTarget: (
      state,
      action: PayloadAction<{ paneId: string | null; index: number | null }>
    ) => {
      const { paneId, index } = action.payload;
      const { sourcePaneId, sourceIndex } = state.dragState;

      // Don't update if dragging over the same position
      if (state.dragState.targetPaneId === paneId && state.dragState.targetIndex === index) {
        return;
      }

      // Validate the target position
      if (paneId && index !== null) {
        const targetPane = state.editorPanes[paneId];
        if (!targetPane) return;

        // Don't allow dropping on itself or on a split container
        if (paneId === sourcePaneId && (index === sourceIndex || index === sourceIndex! + 1)) {
          return;
        }
      }

      state.dragState.targetPaneId = paneId;
      state.dragState.targetIndex = index;
    },

    endDragTab: (
      state,
      action: PayloadAction<{ createNewPane?: boolean; direction?: 'horizontal' | 'vertical' }>
    ) => {
      const { sourcePaneId, sourceIndex, targetPaneId, targetIndex } = state.dragState;
      const { createNewPane, direction } = action.payload || {};

      if (sourcePaneId === null || sourceIndex === null) {
        return;
      }

      const sourcePane = state.editorPanes[sourcePaneId];
      if (!sourcePane || sourcePane.files.length <= sourceIndex) {
        return;
      }

      // Handle dropping outside any pane (create new window/tab)
      if (targetPaneId === null || targetIndex === null) {
        // Reset drag state
        state.dragState = {
          isDragging: false,
          sourcePaneId: null,
          sourceIndex: null,
          targetPaneId: null,
          targetIndex: null,
        };
        return;
      }

      // Handle creating a new split pane
      if (createNewPane && direction) {
        // Create a new pane and move the tab there
        const newPaneId = `pane-${Date.now()}`;
        const [movedTab] = sourcePane.files.splice(sourceIndex, 1);

        // Create the new pane with the moved tab
        state.editorPanes[newPaneId] = {
          id: newPaneId,
          activeFile: movedTab.path,
          files: [movedTab],
        };

        // Update the source pane's active file if needed
        if (sourcePane.activeFile === movedTab.path) {
          sourcePane.activeFile = sourcePane.files[0]?.path || null;
        }

        // Create a new split container
        const parentId = `pane-${Date.now() + 1}`;
        state.editorPanes[parentId] = {
          id: parentId,
          activeFile: null,
          files: [],
          splitDirection: direction,
          children: [sourcePaneId, newPaneId],
        };

        // Update references to the source pane
        Object.values(state.editorPanes).forEach((pane) => {
          if (pane.children?.includes(sourcePaneId)) {
            const childIndex = pane.children.indexOf(sourcePaneId);
            pane.children[childIndex] = parentId;
          }
        });

        // Update root pane if needed
        if (state.rootPaneId === sourcePaneId) {
          state.rootPaneId = parentId;
        }

        // Set the new pane as active
        state.activePaneId = newPaneId;
      }
      // Handle moving between existing panes
      else if (targetPaneId && targetIndex !== null) {
        const targetPane = state.editorPanes[targetPaneId];
        if (!targetPane) return;

        const [movedTab] = sourcePane.files.splice(sourceIndex, 1);

        // Calculate the target index, considering pinned tabs
        let adjustedTargetIndex = targetIndex;
        if (sourcePaneId !== targetPaneId) {
          // When moving between panes, adjust the target index based on pinned state
          const targetPinnedCount = targetPane.files.filter((f) => f.isPinned).length;
          if (movedTab.isPinned) {
            adjustedTargetIndex = Math.min(adjustedTargetIndex, targetPinnedCount);
          } else {
            adjustedTargetIndex = Math.max(adjustedTargetIndex, targetPinnedCount);
          }
        }

        // Insert the tab at the target position
        targetPane.files.splice(adjustedTargetIndex, 0, movedTab);

        // Update active file in source pane if needed
        if (sourcePane.activeFile === movedTab.path) {
          sourcePane.activeFile = sourcePane.files[0]?.path || null;
        }

        // Update active file in target pane
        targetPane.activeFile = movedTab.path;
        state.activePaneId = targetPaneId;
      }

      // Reset drag state
      state.dragState = {
        isDragging: false,
        sourcePaneId: null,
        sourceIndex: null,
        targetPaneId: null,
        targetIndex: null,
      };
    },
  },
});

// Export actions
export const tabManagementActions = tabManagementSlice.actions;

// Selectors
export const tabManagementSelectors = {
  selectActivePaneId: (state: { tabManagement: TabManagementState }) =>
    state.tabManagement.activePaneId,
  selectEditorPanes: (state: { tabManagement: TabManagementState }) =>
    state.tabManagement.editorPanes,
  selectActivePane: (state: RootState) => {
    if (!state.tabManagement) return null;
    const { activePaneId, editorPanes } = state.tabManagement;
    return activePaneId ? editorPanes[activePaneId] : null;
  },
  selectActiveFiles: (state: RootState) => {
    if (!state.tabManagement) return [];
    const { activePaneId, editorPanes } = state.tabManagement;
    return activePaneId ? editorPanes[activePaneId]?.files || [] : [];
  },
  selectActiveFile: (state: RootState) => {
    if (!state.tabManagement) return null;
    const { activePaneId, editorPanes } = state.tabManagement;
    return activePaneId ? editorPanes[activePaneId]?.activeFile || null : null;
  },
  selectPinnedFiles: createSelector(
    [
      (state: RootState) => state.tabManagement?.editorPanes || {},
      (state: RootState) => state.tabManagement?.activePaneId,
    ],
    (editorPanes, activePaneId) => {
      if (!activePaneId || !editorPanes[activePaneId]) return [];
      return editorPanes[activePaneId].files.filter((file) => file.isPinned);
    }
  ),
  selectUnpinnedFiles: createSelector(
    [
      (state: RootState) => state.tabManagement?.editorPanes || {},
      (state: RootState) => state.tabManagement?.activePaneId,
    ],
    (editorPanes, activePaneId) => {
      if (!activePaneId || !editorPanes[activePaneId]) return [];
      return editorPanes[activePaneId].files.filter((file) => !file.isPinned);
    }
  ),
  selectCanSplitPane: (state: RootState, paneId: string) => {
    if (!state.tabManagement) return false;
    const pane = state.tabManagement.editorPanes[paneId];
    return !!pane && !pane.children;
  },
  selectDragState: (state: RootState) =>
    state.tabManagement?.dragState || {
      isDragging: false,
      sourcePaneId: null,
      sourceIndex: null,
      targetPaneId: null,
      targetIndex: null,
    },
};

export default tabManagementSlice.reducer;
