import { useCallback, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { themeService } from './ThemeService';
import {
  setCurrentTheme,
  loadThemes,
  createTheme,
  updateTheme,
  deleteTheme,
  applyCustomization,
  resetCustomization,
  loadMarketplace,
  downloadTheme,
  updateAccessibility,
  resetAccessibility,
} from './themeSlice';
import {
  selectCurrentTheme,
  selectCustomization,
  selectAccessibility,
  selectAllThemes,
  selectThemePreview,
  selectThemeValidation,
} from './themeSlice';

export function useTheme() {
  const dispatch = useDispatch();

  const currentTheme = useSelector(selectCurrentTheme);
  const customization = useSelector(selectCustomization);
  const accessibility = useSelector(selectAccessibility);
  const allThemes = useSelector(selectAllThemes);
  const themePreview = useSelector(selectThemePreview);

  // Initialize theme service
  useEffect(() => {
    dispatch(loadThemes());

    // Load initial accessibility options
    const currentAccessibility = themeService.getAccessibilityOptions();
    dispatch(updateAccessibility(currentAccessibility));
  }, [dispatch]);

  const setTheme = useCallback((themeId: string) => {
    dispatch(setCurrentTheme(themeId));
  }, [dispatch]);

  const createNewTheme = useCallback((themeDefinition: any) => {
    dispatch(createTheme(themeDefinition));
  }, [dispatch]);

  const updateExistingTheme = useCallback((themeDefinition: any) => {
    dispatch(updateTheme(themeDefinition));
  }, [dispatch]);

  const deleteExistingTheme = useCallback((themeId: string) => {
    dispatch(deleteTheme(themeId));
  }, [dispatch]);

  const applyThemeCustomization = useCallback((customization: any) => {
    dispatch(applyCustomization(customization));
  }, [dispatch]);

  const resetThemeCustomization = useCallback(() => {
    dispatch(resetCustomization());
  }, [dispatch]);

  const loadThemesFromMarketplace = useCallback(() => {
    dispatch(loadMarketplace());
  }, [dispatch]);

  const downloadThemeFromMarketplace = useCallback((item: any) => {
    dispatch(downloadTheme(item));
  }, [dispatch]);

  const updateAccessibilityOptions = useCallback((options: any) => {
    dispatch(updateAccessibility(options));
  }, [dispatch]);

  const resetAccessibilityOptions = useCallback(() => {
    dispatch(resetAccessibility());
  }, [dispatch]);

  const validateThemeDefinition = useCallback((theme: any) => {
    return selectThemeValidation({} as any, theme);
  }, []);

  const generateThemeFromPalette = useCallback((baseColor: string) => {
    return themeService.generateAccessiblePalette(baseColor);
  }, []);

  return {
    // State
    currentTheme,
    customization,
    accessibility,
    allThemes,
    themePreview,

    // Actions
    setTheme,
    createNewTheme,
    updateExistingTheme,
    deleteExistingTheme,
    applyThemeCustomization,
    resetThemeCustomization,
    loadThemesFromMarketplace,
    downloadThemeFromMarketplace,
    updateAccessibilityOptions,
    resetAccessibilityOptions,

    // Utilities
    validateTheme: validateThemeDefinition,
    generatePalette: generateThemeFromPalette,

    // Service methods
    getThemeById: (themeId: string) => themeService.loadTheme(themeId),
    getAllAvailableThemes: () => themeService.getMarketplaceThemes(),
    exportTheme: (themeId: string) => {
      const theme = themeService.loadTheme(themeId);
      return theme ? JSON.stringify(theme, null, 2) : null;
    },
    importTheme: (themeData: string) => {
      try {
        const theme = JSON.parse(themeData);
        const validation = themeService.validateTheme(theme);
        if (validation.isValid) {
          themeService.saveTheme(theme);
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },
  };
}

export function useHighContrastMode() {
  const { accessibility, updateAccessibilityOptions } = useTheme();

  const toggleHighContrast = useCallback(() => {
    updateAccessibilityOptions({
      highContrastMode: !accessibility.highContrastMode
    });
  }, [accessibility.highContrastMode, updateAccessibilityOptions]);

  return {
    isHighContrast: accessibility.highContrastMode,
    toggleHighContrast,
  };
}

export function useReducedMotion() {
  const { accessibility, updateAccessibilityOptions } = useTheme();

  const toggleReducedMotion = useCallback(() => {
    updateAccessibilityOptions({
      reducedMotion: !accessibility.reducedMotion
    });
  }, [accessibility.reducedMotion, updateAccessibilityOptions]);

  return {
    isReducedMotion: accessibility.reducedMotion,
    toggleReducedMotion,
  };
}

export function useThemeForMonaco(editorRef: React.RefObject<any>) {
  const { currentTheme } = useTheme();

  useEffect(() => {
    if (editorRef.current && currentTheme) {
      themeService.applyToMonaco(editorRef.current);
    }
  }, [currentTheme, editorRef]);

  return {
    applyThemeToMonaco: (editor: any) => {
      themeService.applyToMonaco(editor);
    },
  };
}