import type {
  ThemeDefinition,
  ThemeCustomization,
  ThemeMarketplaceItem,
  AccessibilityOptions,
  ThemeService,
  ColorPalette,
} from './types';
import { BUILTIN_THEMES } from './builtinThemes';
import * as monaco from 'monaco-editor';

export class ThemeServiceImpl implements ThemeService {
  private themes: Map<string, ThemeDefinition> = new Map();
  private currentThemeId: string;
  private customization: ThemeCustomization = {};
  private accessibilityOptions: AccessibilityOptions = {
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

  constructor() {
    // Load built-in themes
    BUILTIN_THEMES.forEach((theme) => {
      this.themes.set(theme.id, theme);
    });

    // Initialize with default theme
    this.currentThemeId = 'default-dark';

    // Load user customizations and accessibility settings
    this.loadFromStorage();

    // Initialize Monaco themes
    this.initializeMonacoThemes();
  }

  getCurrent(): ThemeDefinition | null {
    return this.themes.get(this.currentThemeId) || null;
  }

  setCurrent(themeId: string): void {
    const theme = this.themes.get(themeId);
    if (theme) {
      this.currentThemeId = themeId;
      this.applyTheme(theme);
      this.saveToStorage();
    }
  }

  loadTheme(themeId: string): ThemeDefinition | null {
    return this.themes.get(themeId) || null;
  }

  saveTheme(theme: ThemeDefinition): void {
    this.themes.set(theme.id, theme);
    if (theme.monacoTheme) {
      this.registerMonacoTheme(theme);
    }
    this.saveToStorage();
  }

  deleteTheme(themeId: string): void {
    const theme = this.themes.get(themeId);
    if (theme && !theme.isBuiltIn) {
      this.themes.delete(themeId);
      if (this.currentThemeId === themeId) {
        this.setCurrent('default-dark');
      }
      this.saveToStorage();
    }
  }

  getCustomization(): ThemeCustomization {
    return { ...this.customization };
  }

  applyCustomization(customization: ThemeCustomization): void {
    this.customization = { ...customization };
    const currentTheme = this.getCurrent();
    if (currentTheme) {
      this.applyTheme(currentTheme);
    }
    this.saveToStorage();
  }

  resetCustomization(): void {
    this.customization = {};
    const currentTheme = this.getCurrent();
    if (currentTheme) {
      this.applyTheme(currentTheme);
    }
    this.saveToStorage();
  }

  getMarketplaceThemes(): ThemeMarketplaceItem[] {
    // Mock marketplace data - in a real implementation, this would fetch from an API
    return [];
  }

  async downloadTheme(item: ThemeMarketplaceItem): Promise<void> {
    // Download and install theme
    this.saveTheme(item.theme);
  }

  async uploadTheme(theme: ThemeDefinition): Promise<void> {
    // Upload theme to marketplace
    console.log('Uploading theme:', theme);
  }

  getAccessibilityOptions(): AccessibilityOptions {
    return { ...this.accessibilityOptions };
  }

  updateAccessibilityOptions(options: Partial<AccessibilityOptions>): void {
    this.accessibilityOptions = { ...this.accessibilityOptions, ...options };
    const currentTheme = this.getCurrent();
    if (currentTheme) {
      this.applyTheme(currentTheme);
    }
    this.saveToStorage();
  }

  generateAccessiblePalette(baseColor: string): ColorPalette {
    // Generate an accessible color palette based on WCAG guidelines
    // This is a simplified implementation
    return {
      primary: baseColor,
      secondary: this.adjustColor(baseColor, -20),
      accent: this.adjustColor(baseColor, 20),
      background: this.accessibilityOptions.highContrastMode ? '#000000' : '#ffffff',
      surface: this.accessibilityOptions.highContrastMode ? '#0f0f0f' : '#f8f9fa',
      error: '#dc3545',
      warning: '#ffc107',
      success: '#28a745',
      info: '#17a2b8',
      text: {
        primary: this.accessibilityOptions.highContrastMode ? '#ffffff' : '#212529',
        secondary: this.accessibilityOptions.highContrastMode ? '#cccccc' : '#6c757d',
        disabled: this.accessibilityOptions.highContrastMode ? '#888888' : '#adb5bd',
        hint: this.accessibilityOptions.highContrastMode ? '#cccccc' : '#6c757d',
      },
      border: {
        light: this.accessibilityOptions.highContrastMode ? '#333333' : '#dee2e6',
        medium: this.accessibilityOptions.highContrastMode ? '#666666' : '#adb5bd',
        heavy: this.accessibilityOptions.highContrastMode ? '#999999' : '#6c757d',
      },
      hover: this.accessibilityOptions.highContrastMode ? '#1a1a1a' : '#e9ecef',
      active: this.accessibilityOptions.highContrastMode ? '#2a2a2a' : '#dee2e6',
      selected: baseColor,
      focus: baseColor,
      editor: {
        background: this.accessibilityOptions.highContrastMode ? '#000000' : '#ffffff',
        foreground: this.accessibilityOptions.highContrastMode ? '#ffffff' : '#212529',
        selection: baseColor,
        lineHighlight: this.accessibilityOptions.highContrastMode ? '#1a1a1a' : '#f8f9fa',
        cursor: this.accessibilityOptions.highContrastMode ? '#ffffff' : baseColor,
        gutter: this.accessibilityOptions.highContrastMode ? '#0f0f0f' : '#f8f9fa',
      },
      terminal: {
        background: '#000000',
        foreground: '#ffffff',
        cursor: '#ffffff',
      },
      sidebar: {
        background: this.accessibilityOptions.highContrastMode ? '#0f0f0f' : '#f8f9fa',
        border: this.accessibilityOptions.highContrastMode ? '#333333' : '#dee2e6',
        item: {
          background: 'transparent',
          hover: this.accessibilityOptions.highContrastMode ? '#1a1a1a' : '#e9ecef',
          selected: baseColor,
        },
      },
    };
  }

  applyToMonaco(editor: monaco.editor.IStandaloneCodeEditor): void {
    const currentTheme = this.getCurrent();
    if (currentTheme?.monacoTheme) {
      monaco.editor.defineTheme('custom-theme', currentTheme.monacoTheme);
      monaco.editor.setTheme('custom-theme');
    }
  }

  generateThemePreview(): string {
    // Generate a CSS string for theme preview
    const theme = this.getCurrent();
    if (!theme) return '';

    return `
      .theme-preview {
        background-color: ${theme.colors.background};
        color: ${theme.colors.text.primary};
        border: 1px solid ${theme.colors.border.medium};
        border-radius: ${theme.borderRadius.md};
      }
      .theme-preview-surface {
        background-color: ${theme.colors.surface};
      }
      .theme-preview-primary {
        background-color: ${theme.colors.primary};
        color: ${theme.colors.text.primary};
      }
    `;
  }

  validateTheme(theme: ThemeDefinition): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    // Basic validation
    if (!theme.id) errors.push('Theme ID is required');
    if (!theme.name) errors.push('Theme name is required');
    if (!theme.colors) errors.push('Theme colors are required');
    if (!['light', 'dark', 'high-contrast'].includes(theme.type)) {
      errors.push('Theme type must be light, dark, or high-contrast');
    }

    // Check contrast ratios
    if (theme.isAccessible && theme.contrastRatio) {
      if (theme.contrastRatio.normal < 4.5) {
        errors.push('Normal text contrast ratio must be at least 4.5:1 for accessibility');
      }
      if (theme.contrastRatio.large < 3.0) {
        errors.push('Large text contrast ratio must be at least 3:1 for accessibility');
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }

  // Private methods
  private applyTheme(theme: ThemeDefinition): void {
    const customizedTheme = this.applyCustomizationToTheme(theme);

    // Apply to CSS variables
    this.applyToCSSVariables(customizedTheme);

    // Update Monaco editor if it exists
    if (customizedTheme.monacoTheme) {
      this.applyToMonacoTheme(customizedTheme);
    }

    // Update document classes
    this.updateDocumentClasses(customizedTheme);
  }

  private applyCustomizationToTheme(theme: ThemeDefinition): ThemeDefinition {
    const customized = { ...theme };

    if (this.customization.colorOverrides) {
      customized.colors = { ...customized.colors, ...this.customization.colorOverrides };
    }

    // Apply accessibility adjustments
    if (this.accessibilityOptions.highContrastMode && theme.colors.highContrast) {
      customized.colors = theme.colors.highContrast;
    }

    return customized;
  }

  private applyToCSSVariables(theme: ThemeDefinition): void {
    const root = document.documentElement;
    const variables: Record<string, string> = {};

    // Colors
    Object.entries(theme.colors).forEach(([key, value]) => {
      if (typeof value === 'string') {
        variables[`--color-${key}`] = value;
      } else if (typeof value === 'object') {
        Object.entries(value).forEach(([subKey, subValue]) => {
          if (typeof subValue === 'string') {
            variables[`--color-${key}-${subKey}`] = subValue;
          } else if (typeof subValue === 'object') {
            Object.entries(subValue).forEach(([subSubKey, subSubValue]) => {
              if (typeof subSubValue === 'string') {
                variables[`--color-${key}-${subKey}-${subSubKey}`] = subSubValue;
              }
            });
          }
        });
      }
    });

    // Typography, spacing, etc.
    Object.entries(theme.typography.fontSize).forEach(([key, value]) => {
      variables[`--font-size-${key}`] = value;
    });

    Object.entries(theme.spacing).forEach(([key, value]) => {
      variables[`--spacing-${key}`] = value;
    });

    // Apply variables
    Object.entries(variables).forEach(([key, value]) => {
      root.style.setProperty(key, value);
    });
  }

  private applyToMonacoTheme(theme: ThemeDefinition): void {
    if (theme.monacoTheme) {
      monaco.editor.defineTheme('custom-theme', theme.monacoTheme);
    }
  }

  private updateDocumentClasses(theme: ThemeDefinition): void {
    const body = document.body;
    const classesToRemove = ['theme-light', 'theme-dark', 'theme-high-contrast'];
    const classToAdd = `theme-${theme.type}`;

    // Remove existing theme classes
    body.classList.remove(...classesToRemove);

    // Add new theme class
    body.classList.add(classToAdd);

    // Add accessibility classes
    if (this.accessibilityOptions.highContrastMode) {
      body.classList.add('high-contrast');
    } else {
      body.classList.remove('high-contrast');
    }

    if (this.accessibilityOptions.reducedMotion) {
      body.classList.add('reduced-motion');
    } else {
      body.classList.remove('reduced-motion');
    }
  }

  private initializeMonacoThemes(): void {
    this.themes.forEach((theme) => {
      if (theme.monacoTheme) {
        this.registerMonacoTheme(theme);
      }
    });
  }

  private registerMonacoTheme(theme: ThemeDefinition): void {
    if (theme.monacoTheme) {
      monaco.editor.defineTheme(`theme-${theme.id}`, theme.monacoTheme);
    }
  }

  private loadFromStorage(): void {
    try {
      const storedCustomization = localStorage.getItem('theme-customization');
      if (storedCustomization) {
        this.customization = JSON.parse(storedCustomization);
      }

      const storedCurrent = localStorage.getItem('current-theme');
      if (storedCurrent) {
        this.currentThemeId = storedCurrent;
      }

      const storedAccessibility = localStorage.getItem('accessibility-options');
      if (storedAccessibility) {
        this.accessibilityOptions = {
          ...this.accessibilityOptions,
          ...JSON.parse(storedAccessibility),
        };
      }

      const storedCustomThemes = localStorage.getItem('custom-themes');
      if (storedCustomThemes) {
        const parsedCustom = JSON.parse(storedCustomThemes);
        Object.entries(parsedCustom).forEach(([id, theme]: [string, any]) => {
          this.themes.set(id, theme);
        });
      }
    } catch (error) {
      console.error('Failed to load theme settings:', error);
    }
  }

  private saveToStorage(): void {
    try {
      localStorage.setItem('theme-customization', JSON.stringify(this.customization));
      localStorage.setItem('current-theme', this.currentThemeId);
      localStorage.setItem('accessibility-options', JSON.stringify(this.accessibilityOptions));

      const customThemes: Record<string, ThemeDefinition> = {};
      this.themes.forEach((theme, id) => {
        if (!theme.isBuiltIn) {
          customThemes[id] = theme;
        }
      });
      localStorage.setItem('custom-themes', JSON.stringify(customThemes));
    } catch (error) {
      console.error('Failed to save theme settings:', error);
    }
  }

  private adjustColor(color: string, amount: number): string {
    // Simple color adjustment - in a real implementation, you'd use a color manipulation library
    return color; // Placeholder
  }
}

// Create singleton instance
export const themeService = new ThemeServiceImpl();
