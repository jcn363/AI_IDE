import type * as monaco from 'monaco-editor';

export type ThemeType = 'light' | 'dark' | 'high-contrast';

export interface ColorPalette {
  // Base colors
  primary: string;
  secondary: string;
  accent: string;
  background: string;
  surface: string;
  error: string;
  warning: string;
  success: string;
  info: string;

  // Text colors
  text: {
    primary: string;
    secondary: string;
    disabled: string;
    hint: string;
  };

  // Border colors
  border: {
    light: string;
    medium: string;
    heavy: string;
  };

  // Interactive states
  hover: string;
  active: string;
  selected: string;
  focus: string;

  // Semantic colors for specific UI elements
  editor: {
    background: string;
    foreground: string;
    selection: string;
    lineHighlight: string;
    cursor: string;
    gutter: string;
  };

  terminal: {
    background: string;
    foreground: string;
    cursor: string;
  };

  sidebar: {
    background: string;
    border: string;
    item: {
      background: string;
      hover: string;
      selected: string;
    };
  };

  // Accessibility colors (high contrast)
  highContrast?: ColorPalette;
}

export interface TypographyScale {
  fontFamily: string;
  fontSize: {
    xs: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
    '2xl': string;
    '3xl': string;
  };
  fontWeight: {
    light: number;
    normal: number;
    medium: number;
    semibold: number;
    bold: number;
  };
  lineHeight: {
    tight: number;
    normal: number;
    relaxed: number;
  };
}

export interface SpacingScale {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  '2xl': string;
  '3xl': string;
}

export interface BorderRadiusScale {
  none: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  full: string;
}

export interface ShadowScale {
  none: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  '2xl': string;
}

export interface TransitionScale {
  fast: string;
  normal: string;
  slow: string;
}

export interface ThemeDefinition {
  id: string;
  name: string;
  description: string;
  type: ThemeType;
  colors: ColorPalette;
  typography: TypographyScale;
  spacing: SpacingScale;
  borderRadius: BorderRadiusScale;
  shadows: ShadowScale;
  transitions: TransitionScale;

  // Monaco editor theme can be embedded
  monacoTheme?: monaco.editor.IStandaloneThemeData;

  // Metadata
  author?: string;
  version?: string;
  tags?: string[];
  license?: string;
  isBuiltIn?: boolean;
  isAccessible?: boolean;
  contrastRatio?: {
    normal: number;
    large: number;
  };

  // Custom settings
  customSettings?: Record<string, any>;
}

export interface ThemeStore {
  current: string;
  available: Record<string, ThemeDefinition>;
  custom: Record<string, ThemeDefinition>;
  presets: ThemeDefinition[];
}

export interface ThemeCustomization {
  colorOverrides?: Partial<ColorPalette>;
  typographyOverrides?: Partial<TypographyScale>;
  spacingOverrides?: Partial<SpacingScale>;
  borderRadiusOverrides?: Partial<BorderRadiusScale>;
  shadowOverrides?: Partial<ShadowScale>;
  transitionOverrides?: Partial<TransitionScale>;
  customCss?: string;
  customSettings?: Record<string, any>;
}

export interface ThemeMarketplaceItem {
  id: string;
  theme: ThemeDefinition;
  downloads: number;
  rating: number;
  reviews: number;
  author: string;
  repository?: string;
  previewImages?: string[];
  compatibleVersions?: string[];
  updatedAt: number;
  tags: string[];
}

export interface AccessibilityOptions {
  // Contrast settings
  highContrastMode: boolean;
  reducedMotion: boolean;
  colorBlindness: 'none' | 'protanopia' | 'deuteranopia' | 'tritanopia';

  // Font settings
  fontSize: 'small' | 'medium' | 'large' | 'extra-large';
  fontWeight: 'normal' | 'bold';
  lineHeight: 'tight' | 'normal' | 'relaxed';

  // Color adjustments
  saturation: number; // 0-1
  brightness: number; // 0-1

  // Focus indicators
  focusIndicatorStyle: 'outline' | 'background' | 'border';
  focusIndicatorColor: string;
  focusIndicatorWidth: string;
}

export interface ThemeService {
  // Theme management
  getCurrent(): ThemeDefinition | null;
  setCurrent(themeId: string): void;
  loadTheme(themeId: string): ThemeDefinition | null;
  saveTheme(theme: ThemeDefinition): void;
  deleteTheme(themeId: string): void;

  // Customization
  getCustomization(): ThemeCustomization;
  applyCustomization(customization: ThemeCustomization): void;
  resetCustomization(): void;

  // Marketplace
  getMarketplaceThemes(): ThemeMarketplaceItem[];
  downloadTheme(item: ThemeMarketplaceItem): Promise<void>;
  uploadTheme(theme: ThemeDefinition): Promise<void>;

  // Accessibility
  getAccessibilityOptions(): AccessibilityOptions;
  updateAccessibilityOptions(options: Partial<AccessibilityOptions>): void;
  generateAccessiblePalette(baseColor: string): ColorPalette;

  // Utilities
  applyToMonaco(editor: monaco.editor.IStandaloneCodeEditor): void;
  generateThemePreview(): string;
  validateTheme(theme: ThemeDefinition): { isValid: boolean; errors: string[] };
}