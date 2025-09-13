import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type {
  ThemeDefinition,
  ThemeCustomization,
  ThemeMarketplaceItem,
  AccessibilityOptions,
} from './types';
import { themeService } from './ThemeService';

interface ThemeState {
  current: string;
  available: Record<string, ThemeDefinition>;
  custom: Record<string, ThemeDefinition>;
  marketplace: ThemeMarketplaceItem[];
  customization: ThemeCustomization;
  accessibility: AccessibilityOptions;
  loading: boolean;
  error: string | null;
}

const initialState: ThemeState = {
  current: 'default-dark',
  available: {},
  custom: {},
  marketplace: [],
  customization: {},
  accessibility: {
    highContrastMode: false,
    reducedMotion: false,
    colorBlindness: 'none',
    fontSize: 'medium',
    fontWeight: 'normal',
    lineHeight: 'normal',
    saturation: 1.0,
    brightness: 1.0,
    focusIndicatorStyle: 'outline',
    focusIndicatorColor: '#007acc',
    focusIndicatorWidth: '2px',
  },
  loading: false,
  error: null,
};

const themeSlice = createSlice({
  name: 'theme',
  initialState,
  reducers: {
    setCurrentTheme: (state, action: PayloadAction<string>) => {
      themeService.setCurrent(action.payload);
      state.current = action.payload;
    },

    loadThemes: (state) => {
      // Load themes from service
      themeService.getCurrent();

      // In a real implementation, this would load all themes
      // For now, we'll just set loading state
      state.loading = true;
    },

    loadThemesSuccess: (
      state,
      action: PayloadAction<{
        available: Record<string, ThemeDefinition>;
        custom: Record<string, ThemeDefinition>;
      }>
    ) => {
      state.available = action.payload.available;
      state.custom = action.payload.custom;
      state.loading = false;
      state.error = null;
    },

    loadThemesFailure: (state, action: PayloadAction<string>) => {
      state.loading = false;
      state.error = action.payload;
    },

    createTheme: (state, action: PayloadAction<ThemeDefinition>) => {
      themeService.saveTheme(action.payload);
      state.custom[action.payload.id] = action.payload;
    },

    updateTheme: (state, action: PayloadAction<ThemeDefinition>) => {
      themeService.saveTheme(action.payload);
      if (action.payload.id in state.custom) {
        state.custom[action.payload.id] = action.payload;
      } else {
        state.available[action.payload.id] = action.payload;
      }
    },

    deleteTheme: (state, action: PayloadAction<string>) => {
      themeService.deleteTheme(action.payload);
      delete state.custom[action.payload];
      delete state.available[action.payload];
    },

    applyCustomization: (state, action: PayloadAction<ThemeCustomization>) => {
      themeService.applyCustomization(action.payload);
      state.customization = action.payload;
    },

    resetCustomization: (state) => {
      themeService.resetCustomization();
      state.customization = {};
    },

    loadMarketplace: (state) => {
      state.marketplace = themeService.getMarketplaceThemes();
    },

    downloadTheme: (state, action: PayloadAction<ThemeMarketplaceItem>) => {
      themeService.downloadTheme(action.payload);
      state.custom[action.payload.theme.id] = action.payload.theme;
    },

    updateAccessibility: (state, action: PayloadAction<Partial<AccessibilityOptions>>) => {
      const updatedAccessibility = { ...state.accessibility, ...action.payload };
      themeService.updateAccessibilityOptions(updatedAccessibility);
      state.accessibility = updatedAccessibility;
    },

    resetAccessibility: (state) => {
      const defaultAccessibility = {
        highContrastMode: false,
        reducedMotion: false,
        colorBlindness: 'none',
        fontSize: 'medium',
        fontWeight: 'normal',
        lineHeight: 'normal',
        saturation: 1.0,
        brightness: 1.0,
        focusIndicatorStyle: 'outline',
        focusIndicatorColor: '#007acc',
        focusIndicatorWidth: '2px',
      };
      themeService.updateAccessibilityOptions(defaultAccessibility);
      state.accessibility = defaultAccessibility;
    },

    setError: (state, action: PayloadAction<string>) => {
      state.error = action.payload;
    },

    clearError: (state) => {
      state.error = null;
    },
  },
});

export const {
  setCurrentTheme,
  loadThemes,
  loadThemesSuccess,
  loadThemesFailure,
  createTheme,
  updateTheme,
  deleteTheme,
  applyCustomization,
  resetCustomization,
  loadMarketplace,
  downloadTheme,
  updateAccessibility,
  resetAccessibility,
  setError,
  clearError,
} = themeSlice.actions;

export default themeSlice.reducer;

// Selectors
export const selectCurrentTheme = (state: any) => themeService.getCurrent();
export const selectCustomization = (state: any) => themeService.getCustomization();
export const selectAccessibility = (state: any) => themeService.getAccessibilityOptions();
export const selectAllThemes = (state: any) => {
  const available = themeService.getCurrent()
    ? {
        [themeService.getCurrent()!.id]: themeService.getCurrent(),
      }
    : {};
  return {
    available,
    custom: {},
  };
};
export const selectThemePreview = (state: any) => themeService.generateThemePreview();
export const selectThemeValidation = (state: any, theme: ThemeDefinition) =>
  themeService.validateTheme(theme);
