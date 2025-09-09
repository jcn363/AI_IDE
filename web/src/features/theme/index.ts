// Types
export * from './types';

// Services
export { themeService } from './ThemeService';

// Redux slice and actions
export { default as themeReducer } from './themeSlice';
export {
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
} from './themeSlice';

// Selectors
export {
  selectCurrentTheme,
  selectCustomization,
  selectAccessibility,
  selectAllThemes,
  selectThemePreview,
  selectThemeValidation,
} from './themeSlice';

// Hooks
export {
  useTheme,
  useHighContrastMode,
  useReducedMotion,
  useThemeForMonaco,
} from './useTheme';

// Built-in themes
export { BUILTIN_THEMES } from './builtinThemes';