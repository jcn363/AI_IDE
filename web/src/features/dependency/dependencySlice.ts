import { PayloadAction, createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import { RootState } from '../../store';
import { invoke } from '@tauri-apps/api/core';
import { 
  BulkActionResponse, 
  Severity, 
  VersionAlignment, 
  VersionAlignmentFilter,
  VersionAlignmentState,
  WorkspaceAlignmentResult,
} from './types';

// Helper function to filter and sort alignments
const filterAndSortAlignments = (
  alignments: VersionAlignment[], 
  filter: VersionAlignmentFilter,
): VersionAlignment[] => {
  let result = [...alignments];
  
  // Apply filters
  if (filter.severity && filter.severity !== 'all') {
    result = result.filter(a => a.severity === filter.severity);
  }
  
  if (filter.searchTerm) {
    const term = filter.searchTerm.toLowerCase();
    result = result.filter(a => 
      a.dependencyName.toLowerCase().includes(term) ||
      a.suggestedVersion.toLowerCase().includes(term) ||
      a.affectedPackages.some(pkg => pkg.toLowerCase().includes(term)),
    );
  }
  
  if (filter.showIgnored !== undefined) {
    result = result.filter((a: VersionAlignment) => a.isIgnored === filter.showIgnored);
  }
  
  // Apply sorting
  if (filter.sortBy) {
    const sortOrder = filter.sortOrder === 'desc' ? -1 : 1;
    result.sort((a, b) => {
      let compare = 0;
      
      switch (filter.sortBy) {
        case 'severity':
          const severityOrder: Record<Severity, number> = { high: 2, medium: 1, low: 0 };
          compare = (severityOrder[a.severity] - severityOrder[b.severity]) * sortOrder;
          break;
        case 'dependencyName':
          compare = a.dependencyName.localeCompare(b.dependencyName) * sortOrder;
          break;
        case 'suggestedVersion':
          compare = a.suggestedVersion.localeCompare(b.suggestedVersion) * sortOrder;
          break;
      }
      
      return compare || a.dependencyName.localeCompare(b.dependencyName);
    });
  }
  
  return result;
};

// Define the return type for the analyze_workspace_alignment Tauri command
type AnalyzeWorkspaceAlignmentResponse = {
  alignments: Array<{
    dependencyName: string;
    currentVersions: Record<string, string>;
    suggestedVersion: string;
    severity: Severity;
    description?: string;
    lastUpdated?: string;
    isIgnored?: boolean;
  }>;
  stats: {
    totalDependencies: number;
    alignedDependencies: number;
    conflicts: number;
    highSeverity: number;
    mediumSeverity: number;
    lowSeverity: number;
  };
};

// Define the return type for the apply_workspace_alignment Tauri command
type ApplyWorkspaceAlignmentResponse = {
  success: boolean;
  updatedCount: number;
  failedCount: number;
  message: string;
};

// Async thunk for applying workspace alignment

// Define the return type for the analyze_version_alignment Tauri command
type AnalyzeVersionAlignmentResponse = Array<{
  dependencyName: string;
  currentVersions: Record<string, string>;
  suggestedVersion: string;
  severity: Severity;
  description?: string;
  lastUpdated?: string;
  isIgnored?: boolean;
}>;

// Async thunks with proper typing
export const analyzeWorkspaceAlignment = createAsyncThunk<
  WorkspaceAlignmentResult,
  void,
  { state: RootState }
>(
  'versionAlignment/analyzeWorkspace',
  async (_, { rejectWithValue }) => {
    try {
      const response = await invoke<AnalyzeWorkspaceAlignmentResponse>('analyze_workspace_alignment');
      
      // Map the response to add id and affectedPackages
      const alignments = response.alignments.map(align => ({
        ...align,
        id: `${align.dependencyName}-${align.suggestedVersion}`,
        affectedPackages: Object.keys(align.currentVersions),
      }));
      
      return {
        alignments,
        stats: response.stats,
      };
    } catch (error) {
      return rejectWithValue(error instanceof Error ? error.message : 'Failed to analyze workspace alignment');
    }
  }
);

export const analyzeVersionAlignment = createAsyncThunk<
  VersionAlignment[],
  void,
  { state: RootState }
>(
  'versionAlignment/analyze',
  async (_, { getState }) => {
    const state = getState().dependency.versionAlignment;
    const response = await invoke<AnalyzeVersionAlignmentResponse>('analyze_version_alignment');
    
    // Map the response to add id and affectedPackages
    return response.map(align => ({
      ...align,
      id: `${align.dependencyName}-${align.suggestedVersion}`,
      affectedPackages: Object.keys(align.currentVersions),
    }));
  },
);

export const applyWorkspaceAlignment = createAsyncThunk<
  ApplyWorkspaceAlignmentResponse,
  string[] | undefined, // Optional array of alignment IDs to apply, or undefined to apply all
  { state: RootState }
>(
  'versionAlignment/applyWorkspace',
  async (alignmentIds, { getState, rejectWithValue }) => {
    try {
      const state = getState().dependency.versionAlignment;
      const alignmentsToApply = alignmentIds 
        ? state.alignments.filter(a => alignmentIds.includes(a.id))
        : state.alignments;
      
      if (alignmentsToApply.length === 0) {
        return {
          success: true,
          updatedCount: 0,
          failedCount: 0,
          message: 'No alignments to apply',
        };
      }
      
      const response = await invoke<ApplyWorkspaceAlignmentResponse>('apply_workspace_alignment', {
        alignments: alignmentsToApply.map(align => ({
          dependency_name: align.dependencyName,
          version: align.suggestedVersion,
        })),
      });
      
      return response;
    } catch (error) {
      return rejectWithValue(error instanceof Error ? error.message : 'Failed to apply workspace alignment');
    }
  }
);

// Define the return type for the apply_version_alignment Tauri command
interface ApplyVersionAlignmentResponse {
  success: boolean;
  message?: string;
  updatedCount?: number;
  failedCount?: number;
}

// Properly typed async thunk for applying version alignment
export const applyVersionAlignment = createAsyncThunk<
  BulkActionResponse, // Return type
  string[],           // First argument type (array of IDs)
  { state: RootState } // ThunkAPI type
>(
  'versionAlignment/apply',
  async (ids, { getState }) => {
    const state = getState().dependency.versionAlignment;
    const alignments = state.alignments.filter(a => ids.includes(a.id));
    const response = await Promise.all(
      alignments.map(align => 
        invoke<ApplyVersionAlignmentResponse>('apply_version_alignment', {
          dependencyName: align.dependencyName,
          version: align.suggestedVersion,
        }),
      ),
    );
    
    const success = response.every(r => r.success);
    const updatedCount = response.filter(r => r.success).length;
    const failedCount = response.length - updatedCount;
    
    return {
      success,
      message: success 
        ? `Successfully updated ${updatedCount} dependencies` 
        : `Failed to update ${failedCount} dependencies`,
      updatedCount,
      failedCount,
    } as BulkActionResponse;
  },
);

const initialState: VersionAlignmentState = {
  alignments: [],
  filteredAlignments: [],
  selectedIds: [],
  workspaceStats: {
    totalDependencies: 0,
    alignedDependencies: 0,
    conflicts: 0,
    highSeverity: 0,
    mediumSeverity: 0,
    lowSeverity: 0,
  },
  filter: {
    severity: 'all',
    searchTerm: '',
    showIgnored: false,
    sortBy: 'severity',
    sortOrder: 'desc',
  },
  status: 'idle',
  error: null,
  lastUpdated: new Date().toISOString(),
};

const versionAlignmentSlice = createSlice({
  name: 'versionAlignment',
  initialState,
  reducers: {
    clearVersionAlignment: (state: VersionAlignmentState) => {
      state.alignments = [];
      state.filteredAlignments = [];
      state.selectedIds = [];
      state.status = 'idle';
      state.error = null;
    },
    setFilter: (state: VersionAlignmentState, action: PayloadAction<Partial<VersionAlignmentFilter>>) => {
      state.filter = { ...state.filter, ...action.payload };
      state.filteredAlignments = filterAndSortAlignments(state.alignments, state.filter);
    },
    toggleSelectAlignment: (state: VersionAlignmentState, action: PayloadAction<string>) => {
      const id = action.payload;
      const index = state.selectedIds.indexOf(id);
      if (index === -1) {
        state.selectedIds.push(id);
      } else {
        state.selectedIds.splice(index, 1);
      }
    },
    selectAllAlignments: (state: VersionAlignmentState, action: PayloadAction<string[]>) => {
      state.selectedIds = action.payload;
    },
    clearSelectedAlignments: (state: VersionAlignmentState) => {
      state.selectedIds = [];
    },
    toggleIgnoreAlignment: (state: VersionAlignmentState, action: PayloadAction<string>) => {
      const id = action.payload;
      const alignment = state.alignments.find((a: VersionAlignment) => a.id === id);
      if (alignment) {
        alignment.isIgnored = !alignment.isIgnored;
        state.filteredAlignments = filterAndSortAlignments(state.alignments, state.filter);
      }
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(analyzeWorkspaceAlignment.pending, (state) => {
        state.status = 'loading';
        state.error = null;
      })
      .addCase(analyzeWorkspaceAlignment.fulfilled, (state, action) => {
        state.status = 'succeeded';
        state.alignments = action.payload.alignments;
        state.filteredAlignments = filterAndSortAlignments(action.payload.alignments, state.filter);
        state.workspaceStats = action.payload.stats;
        state.lastUpdated = new Date().toISOString();
      })
      .addCase(analyzeWorkspaceAlignment.rejected, (state, action) => {
        state.status = 'failed';
        state.error = action.payload as string || 'Failed to analyze workspace alignment';
      })
      .addCase(analyzeVersionAlignment.pending, (state) => {
        state.status = 'loading';
        state.error = null;
      })
      .addCase(analyzeVersionAlignment.fulfilled, (state, action) => {
        state.status = 'succeeded';
        state.alignments = action.payload;
        state.filteredAlignments = filterAndSortAlignments(action.payload, state.filter);
        state.lastUpdated = new Date().toISOString();
      })
      .addCase(analyzeVersionAlignment.rejected, (state, action) => {
        state.status = 'failed';
        state.error = action.error.message || 'Failed to analyze version alignment';
      })
      .addCase(applyVersionAlignment.pending, (state) => {
        state.status = 'loading';
      })
      .addCase(applyVersionAlignment.fulfilled, (state, action) => {
        state.status = 'succeeded';
        // Remove applied alignments from the list
        state.alignments = state.alignments.filter(
          (a) => !action.meta.arg.includes(a.id)
        );
        state.filteredAlignments = filterAndSortAlignments(state.alignments, state.filter);
        state.selectedIds = state.selectedIds.filter((id) => !action.meta.arg.includes(id));
      })
      .addCase(applyVersionAlignment.rejected, (state, action) => {
        state.status = 'failed';
        state.error = action.error.message || 'Failed to apply version alignment';
      })
      .addCase(applyWorkspaceAlignment.pending, (state) => {
        state.status = 'loading';
      })
      .addCase(applyWorkspaceAlignment.fulfilled, (state, action) => {
        state.status = 'succeeded';
        // Refresh the alignments after applying
        if (action.payload.updatedCount > 0) {
          state.alignments = state.alignments.filter(
            (a) => !action.meta.arg || action.meta.arg.length === 0 || !action.meta.arg.includes(a.id)
          );
          state.filteredAlignments = filterAndSortAlignments(state.alignments, state.filter);
          state.selectedIds = state.selectedIds.filter(
            (id) => !action.meta.arg || action.meta.arg.length === 0 || !action.meta.arg.includes(id)
          );
        }
      })
      .addCase(applyWorkspaceAlignment.rejected, (state, action) => {
        state.status = 'failed';
        state.error = action.payload as string || 'Failed to apply workspace alignment';
      });
  },
});

export const { 
  clearVersionAlignment, 
  setFilter, 
  toggleSelectAlignment, 
  selectAllAlignments, 
  clearSelectedAlignments,
  toggleIgnoreAlignment,
} = versionAlignmentSlice.actions;

// Extend the VersionAlignmentState with computed properties
interface VersionAlignmentSelectorResult extends Omit<VersionAlignmentState, 'selectedIds' | 'filteredAlignments'> {
  selectedCount: number;
  totalCount: number;
  hasSelection: boolean;
  hasAlignments: boolean;
  selectedIds: string[];
  filteredAlignments: VersionAlignment[];
}

export const selectVersionAlignment = (state: RootState): VersionAlignmentSelectorResult => {
  const {versionAlignment} = state.dependency;
  return {
    ...versionAlignment,
    selectedCount: versionAlignment.selectedIds.length,
    totalCount: versionAlignment.filteredAlignments.length,
    hasSelection: versionAlignment.selectedIds.length > 0,
    hasAlignments: versionAlignment.filteredAlignments.length > 0,
  };
};

export const selectAlignmentById = (state: RootState, id: string) => 
  state.dependency.versionAlignment.alignments.find((a: VersionAlignment) => a.id === id);

export const selectSelectedAlignments = (state: RootState) => {
  const { selectedIds, alignments } = state.dependency.versionAlignment;
  return alignments.filter((a: VersionAlignment) => selectedIds.includes(a.id));
};

export default versionAlignmentSlice.reducer;
