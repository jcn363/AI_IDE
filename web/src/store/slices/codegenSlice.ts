import { PayloadAction, createAsyncThunk, createSlice } from '@reduxjs/toolkit';

// Types
export interface GenerationHistoryItem {
  id: string;
  request: {
    function_purpose: string;
    target_language: string;
    parameters: string[];
    return_type?: string;
    similar_functions: string[];
    error_handling: boolean;
    performance_requirements?: string;
    safety_requirements?: string;
  };
  result: {
    success: boolean;
    generated_function?: {
      name: string;
      code: string;
      confidence_score: number;
      complexity: number;
    };
    error?: string;
  };
  validation?: {
    overall_score: number;
    readability_score: number;
    maintainability_score: number;
    performance_score: number;
    security_score: number;
    compliance_score: number;
    issues: Array<{
      category: string;
      severity: string;
      message: string;
    }>;
  };
  timestamp: number;
  isFavorite: boolean;
  tags: string[];
}

export interface CodeGenState {
  history: GenerationHistoryItem[];
  favorites: GenerationHistoryItem[];
  isLoading: boolean;
  error: string | null;
  searchTerm: string;
  filterLanguage: string | null;
  sortBy: 'timestamp' | 'confidence' | 'complexity' | 'language';
  sortOrder: 'asc' | 'desc';
  maxHistorySize: number;
}

// Initial state
const initialState: CodeGenState = {
  history: [],
  favorites: [],
  isLoading: false,
  error: null,
  searchTerm: '',
  filterLanguage: null,
  sortBy: 'timestamp',
  sortOrder: 'desc',
  maxHistorySize: 100,
};

// Async thunks for persistence
export const saveGenerationHistory = createAsyncThunk(
  'codegen/saveHistory',
  async (history: GenerationHistoryItem[]) => {
    // In a real app, this would persist to the backend or secure storage
    // For now, we'll just simulate the operation
    await new Promise((resolve) => setTimeout(resolve, 100));
    return history;
  }
);

export const loadGenerationHistory = createAsyncThunk('codegen/loadHistory', async () => {
  // In a real app, this would load from the backend or secure storage
  // For now, we'll just simulate the operation
  await new Promise((resolve) => setTimeout(resolve, 100));
  return [] as GenerationHistoryItem[];
});

// Create the slice
const codegenSlice = createSlice({
  name: 'codegen',
  initialState,
  reducers: {
    // Add a new generation to history
    addToHistory: (state, action: PayloadAction<GenerationHistoryItem>) => {
      const newItem = action.payload;

      // Check if item already exists (update instead of add)
      const existingIndex = state.history.findIndex((item) => item.id === newItem.id);
      if (existingIndex >= 0) {
        state.history[existingIndex] = newItem;
      } else {
        // Add new item and maintain max history size
        state.history.unshift(newItem);
        if (state.history.length > state.maxHistorySize) {
          state.history = state.history.slice(0, state.maxHistorySize);
        }
      }

      // Update favorites if this item is favorited
      if (newItem.isFavorite) {
        const favoriteIndex = state.favorites.findIndex((item) => item.id === newItem.id);
        if (favoriteIndex >= 0) {
          state.favorites[favoriteIndex] = newItem;
        } else {
          state.favorites.unshift(newItem);
        }
      }
    },

    // Remove from history
    removeFromHistory: (state, action: PayloadAction<string>) => {
      const itemId = action.payload;
      state.history = state.history.filter((item) => item.id !== itemId);

      // Also remove from favorites if present
      state.favorites = state.favorites.filter((item) => item.id !== itemId);
    },

    // Clear all history
    clearHistory: (state) => {
      state.history = [];
      // Keep favorites, they are separate
    },

    // Toggle favorite status
    toggleFavorite: (state, action: PayloadAction<string>) => {
      const itemId = action.payload;

      // Find item in history
      const historyItem = state.history.find((item) => item.id === itemId);
      if (historyItem) {
        historyItem.isFavorite = !historyItem.isFavorite;

        if (historyItem.isFavorite) {
          // Add to favorites
          const favoriteIndex = state.favorites.findIndex((item) => item.id === itemId);
          if (favoriteIndex < 0) {
            state.favorites.unshift({ ...historyItem });
          }
        } else {
          // Remove from favorites
          state.favorites = state.favorites.filter((item) => item.id !== itemId);
        }
      }
    },

    // Add/remove tags
    updateTags: (state, action: PayloadAction<{ itemId: string; tags: string[] }>) => {
      const { itemId, tags } = action.payload;
      const historyItem = state.history.find((item) => item.id === itemId);
      if (historyItem) {
        historyItem.tags = tags;
        // Update favorites too
        const favoriteItem = state.favorites.find((item) => item.id === itemId);
        if (favoriteItem) {
          favoriteItem.tags = tags;
        }
      }
    },

    // Search and filter actions
    setSearchTerm: (state, action: PayloadAction<string>) => {
      state.searchTerm = action.payload;
    },

    setFilterLanguage: (state, action: PayloadAction<string | null>) => {
      state.filterLanguage = action.payload;
    },

    setSortBy: (
      state,
      action: PayloadAction<'timestamp' | 'confidence' | 'complexity' | 'language'>
    ) => {
      state.sortBy = action.payload;
    },

    setSortOrder: (state, action: PayloadAction<'asc' | 'desc'>) => {
      state.sortOrder = action.payload;
    },

    setMaxHistorySize: (state, action: PayloadAction<number>) => {
      state.maxHistorySize = action.payload;
    },

    // Error handling
    setError: (state, action: PayloadAction<string>) => {
      state.error = action.payload;
      state.isLoading = false;
    },

    clearError: (state) => {
      state.error = null;
    },

    // Loading state
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },
  },
  extraReducers: (builder) => {
    builder
      // Save history
      .addCase(saveGenerationHistory.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(saveGenerationHistory.fulfilled, (state) => {
        state.isLoading = false;
      })
      .addCase(saveGenerationHistory.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.error.message || 'Failed to save history';
      })

      // Load history
      .addCase(loadGenerationHistory.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(loadGenerationHistory.fulfilled, (state, action) => {
        state.isLoading = false;
        state.history = action.payload;
        // Extract favorites from history
        state.favorites = action.payload.filter((item) => item.isFavorite);
      })
      .addCase(loadGenerationHistory.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.error.message || 'Failed to load history';
      });
  },
});

// Export actions
export const {
  addToHistory,
  removeFromHistory,
  clearHistory,
  toggleFavorite,
  updateTags,
  setSearchTerm,
  setFilterLanguage,
  setSortBy,
  setSortOrder,
  setMaxHistorySize,
  setError,
  clearError,
  setLoading,
} = codegenSlice.actions;

// Export action creators for use in components
export const codegenActions = {
  addToHistory,
  removeFromHistory,
  clearHistory,
  toggleFavorite,
  updateTags,
  setSearchTerm,
  setFilterLanguage,
  setSortBy,
  setSortOrder,
  setMaxHistorySize,
  setError,
  clearError,
  setLoading,
  saveGenerationHistory,
  loadGenerationHistory,
};

// Selectors
export const selectCodegen = (state: { codegen: CodeGenState }) => state.codegen;

export const selectFilteredHistory = (state: { codegen: CodeGenState }) => {
  const { history, searchTerm, filterLanguage, sortBy, sortOrder } = state.codegen;

  let filtered = history.filter((item) => {
    // Search filter
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase();
      const matchesSearch =
        item.request.function_purpose.toLowerCase().includes(searchLower) ||
        item.result.generated_function?.name.toLowerCase().includes(searchLower) ||
        item.result.generated_function?.code.toLowerCase().includes(searchLower) ||
        item.request.target_language.toLowerCase().includes(searchLower);

      if (!matchesSearch) return false;
    }

    // Language filter
    if (filterLanguage && item.request.target_language !== filterLanguage) {
      return false;
    }

    return true;
  });

  // Sort
  filtered.sort((a, b) => {
    let comparison = 0;

    switch (sortBy) {
      case 'timestamp':
        comparison = a.timestamp - b.timestamp;
        break;
      case 'confidence':
        comparison =
          (a.result.generated_function?.confidence_score || 0) -
          (b.result.generated_function?.confidence_score || 0);
        break;
      case 'complexity':
        comparison =
          (a.result.generated_function?.complexity || 0) -
          (b.result.generated_function?.complexity || 0);
        break;
      case 'language':
        comparison = a.request.target_language.localeCompare(b.request.target_language);
        break;
      default:
        comparison = 0;
    }

    return sortOrder === 'asc' ? comparison : -comparison;
  });

  return filtered;
};

export const selectFilteredFavorites = (state: { codegen: CodeGenState }) => {
  const { favorites, searchTerm, filterLanguage, sortBy, sortOrder } = state.codegen;

  let filtered = favorites.filter((item) => {
    // Search filter
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase();
      const matchesSearch =
        item.request.function_purpose.toLowerCase().includes(searchLower) ||
        item.result.generated_function?.name.toLowerCase().includes(searchLower) ||
        item.result.generated_function?.code.toLowerCase().includes(searchLower) ||
        item.request.target_language.toLowerCase().includes(searchLower);

      if (!matchesSearch) return false;
    }

    // Language filter
    if (filterLanguage && item.request.target_language !== filterLanguage) {
      return false;
    }

    return true;
  });

  // Sort
  filtered.sort((a, b) => {
    let comparison = 0;

    switch (sortBy) {
      case 'timestamp':
        comparison = a.timestamp - b.timestamp;
        break;
      case 'confidence':
        comparison =
          (a.result.generated_function?.confidence_score || 0) -
          (b.result.generated_function?.confidence_score || 0);
        break;
      case 'complexity':
        comparison =
          (a.result.generated_function?.complexity || 0) -
          (b.result.generated_function?.complexity || 0);
        break;
      case 'language':
        comparison = a.request.target_language.localeCompare(b.request.target_language);
        break;
      default:
        comparison = 0;
    }

    return sortOrder === 'asc' ? comparison : -comparison;
  });

  return filtered;
};

// Export all selectors in one object for easier imports
export const codegenSelectors = {
  selectCodegen,
  selectFilteredHistory,
  selectFilteredFavorites,
  selectIsLoading: (state: { codegen: CodeGenState }) => state.codegen.isLoading,
  selectError: (state: { codegen: CodeGenState }) => state.codegen.error,
  selectHistoryCount: (state: { codegen: CodeGenState }) => state.codegen.history.length,
  selectFavoritesCount: (state: { codegen: CodeGenState }) => state.codegen.favorites.length,
  selectSearchTerm: (state: { codegen: CodeGenState }) => state.codegen.searchTerm,
  selectFilterLanguage: (state: { codegen: CodeGenState }) => state.codegen.filterLanguage,
  selectSortBy: (state: { codegen: CodeGenState }) => state.codegen.sortBy,
  selectSortOrder: (state: { codegen: CodeGenState }) => state.codegen.sortOrder,
} as const;

export default codegenSlice.reducer;
