import type { ThemeDefinition, ColorPalette, TypographyScale, SpacingScale, BorderRadiusScale, ShadowScale, TransitionScale } from './types';

// Common base values
const typographyBase: TypographyScale = {
  fontFamily: '"Inter", "Segoe UI", "Roboto", system-ui, sans-serif',
  fontSize: {
    xs: '0.75rem',
    sm: '0.875rem',
    md: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
    '2xl': '1.5rem',
    '3xl': '1.875rem',
  },
  fontWeight: {
    light: 300,
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },
  lineHeight: {
    tight: 1.25,
    normal: 1.5,
    relaxed: 1.75,
  },
};

const spacingBase: SpacingScale = {
  xs: '0.25rem',
  sm: '0.5rem',
  md: '1rem',
  lg: '1.5rem',
  xl: '2rem',
  '2xl': '3rem',
  '3xl': '4rem',
};

const borderRadiusBase: BorderRadiusScale = {
  none: '0',
  sm: '0.125rem',
  md: '0.375rem',
  lg: '0.5rem',
  xl: '0.75rem',
  full: '9999px',
};

const shadowBase: ShadowScale = {
  none: 'none',
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
  '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
};

const transitionBase: TransitionScale = {
  fast: '150ms ease-in-out',
  normal: '200ms ease-in-out',
  slow: '300ms ease-in-out',
};

// Light theme colors
const lightPalette: ColorPalette = {
  primary: '#007acc',
  secondary: '#6c757d',
  accent: '#17a2b8',
  background: '#ffffff',
  surface: '#f8f9fa',
  error: '#dc3545',
  warning: '#ffc107',
  success: '#28a745',
  info: '#17a2b8',
  text: {
    primary: '#212529',
    secondary: '#6c757d',
    disabled: '#adb5bd',
    hint: '#6c757d',
  },
  border: {
    light: '#dee2e6',
    medium: '#adb5bd',
    heavy: '#6c757d',
  },
  hover: '#e9ecef',
  active: '#dee2e6',
  selected: '#007acc',
  focus: '#007acc',
  editor: {
    background: '#ffffff',
    foreground: '#212529',
    selection: '#cce5ff',
    lineHighlight: '#f8f9fa',
    cursor: '#007acc',
    gutter: '#f8f9fa',
  },
  terminal: {
    background: '#000000',
    foreground: '#ffffff',
    cursor: '#ffffff',
  },
  sidebar: {
    background: '#f8f9fa',
    border: '#dee2e6',
    item: {
      background: 'transparent',
      hover: '#e9ecef',
      selected: '#cce5ff',
    },
  },
};

// Dark theme colors
const darkPalette: ColorPalette = {
  primary: '#4aa3fc',
  secondary: '#adb5bd',
  accent: '#5bc0de',
  background: '#1e1e1e',
  surface: '#252526',
  error: '#f48771',
  warning: '#ffcc02',
  success: '#4ec9b0',
  info: '#4fc4ff',
  text: {
    primary: '#cccccc',
    secondary: '#898989',
    disabled: '#6c757d',
    hint: '#898989',
  },
  border: {
    light: '#3e3e3e',
    medium: '#585858',
    heavy: '#898989',
  },
  hover: '#2d2d30',
  active: '#37373d',
  selected: '#4aa3fc',
  focus: '#4aa3fc',
  editor: {
    background: '#1e1e1e',
    foreground: '#cccccc',
    selection: '#264f78',
    lineHighlight: '#2d2d30',
    cursor: '#aeafad',
    gutter: '#252526',
  },
  terminal: {
    background: '#000000',
    foreground: '#ffffff',
    cursor: '#ffffff',
  },
  sidebar: {
    background: '#252526',
    border: '#3e3e3e',
    item: {
      background: 'transparent',
      hover: '#2d2d30',
      selected: '#264f78',
    },
  },
};

// High contrast theme colors
const highContrastPalette: ColorPalette = {
  primary: '#ffff00',
  secondary: '#ffffff',
  accent: '#00ffff',
  background: '#000000',
  surface: '#0f0f0f',
  error: '#ff0000',
  warning: '#ffff00',
  success: '#00ff00',
  info: '#00ffff',
  text: {
    primary: '#ffffff',
    secondary: '#cccccc',
    disabled: '#888888',
    hint: '#cccccc',
  },
  border: {
    light: '#333333',
    medium: '#666666',
    heavy: '#999999',
  },
  hover: '#1a1a1a',
  active: '#2a2a2a',
  selected: '#ffff00',
  focus: '#ffff00',
  editor: {
    background: '#000000',
    foreground: '#ffffff',
    selection: '#ffff00',
    lineHighlight: '#1a1a1a',
    cursor: '#ffff00',
    gutter: '#0f0f0f',
  },
  terminal: {
    background: '#000000',
    foreground: '#ffffff',
    cursor: '#ffffff',
  },
  sidebar: {
    background: '#0f0f0f',
    border: '#333333',
    item: {
      background: 'transparent',
      hover: '#1a1a1a',
      selected: '#ffff00',
    },
  },
};

// Built-in themes
export const BUILTIN_THEMES: ThemeDefinition[] = [
  {
    id: 'default-light',
    name: 'Light Theme',
    description: 'Clean and bright theme for daytime use',
    type: 'light',
    colors: lightPalette,
    typography: typographyBase,
    spacing: spacingBase,
    borderRadius: borderRadiusBase,
    shadows: shadowBase,
    transitions: transitionBase,
    isBuiltIn: true,
    isAccessible: true,
    contrastRatio: {
      normal: 4.5,
      large: 3.0,
    },
  },
  {
    id: 'default-dark',
    name: 'Dark Theme',
    description: 'Easy on the eyes theme for nighttime use',
    type: 'dark',
    colors: darkPalette,
    typography: typographyBase,
    spacing: spacingBase,
    borderRadius: borderRadiusBase,
    shadows: shadowBase,
    transitions: transitionBase,
    isBuiltIn: true,
    isAccessible: true,
    contrastRatio: {
      normal: 4.5,
      large: 3.0,
    },
  },
  {
    id: 'default-high-contrast',
    name: 'High Contrast',
    description: 'Maximum accessibility with high contrast colors',
    type: 'high-contrast',
    colors: highContrastPalette,
    typography: {
      ...typographyBase,
      fontSize: {
        ...typographyBase.fontSize,
        md: '1.125rem', // Slightly larger base font
      },
    },
    spacing: spacingBase,
    borderRadius: borderRadiusBase,
    shadows: {
      ...shadowBase,
      sm: '0 1px 3px 0 rgb(255 255 255 / 0.3)',
      md: '0 4px 6px -1px rgb(255 255 255 / 0.3), 0 2px 4px -2px rgb(255 255 255 / 0.3)',
    },
    transitions: transitionBase,
    isBuiltIn: true,
    isAccessible: true,
    contrastRatio: {
      normal: 7.0,
      large: 4.5,
    },
  },
];