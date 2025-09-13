// Design system tokens and constants
// Centralizes all design-related constants to ensure consistency

/**
 * Spacing scale - follows 8px base grid
 */
export const SPACING = {
  xs: 0.5, // 4px
  sm: 1, // 8px
  md: 2, // 16px
  lg: 3, // 24px
  xl: 4, // 32px
  xxl: 6, // 48px
  xxxl: 8, // 64px
} as const;

/**
 * Layout dimensions and breakpoints
 */
export const LAYOUT = {
  // Panel dimensions
  PANEL_HEIGHT: '100%',
  PANEL_PADDING: SPACING.md,

  // Header dimensions (proportional to panel for responsive design)
  HEADER_HEIGHT_RATIO: 0.75,
  HEADER_HEIGHT_PX: 64,

  // Toolbar dimensions
  TOOLBAR_HEIGHT_RATIO: 0.75,
  TOOLBAR_HEIGHT_PX: 48,

  // Content area maximum heights for scrollable areas
  MAX_CONTENT_HEIGHT: 400,
  MAX_LIST_HEIGHT: 300,
  MAX_EDITOR_HEIGHT: 600,
} as const;

/**
 * Color constants - extending Material-UI theme colors
 */
export const COLORS = {
  // Status colors
  SUCCESS: 'success.main',
  ERROR: 'error.main',
  WARNING: 'warning.main',
  INFO: 'info.main',

  // Text colors
  TEXT_PRIMARY: 'text.primary',
  TEXT_SECONDARY: 'text.secondary',
  TEXT_DISABLED: 'text.disabled',

  // Background colors
  BG_DEFAULT: 'background.default',
  BG_PAPER: 'background.paper',
  BG_SURFACE: 'grey.50',

  // Border colors
  BORDER_LIGHT: 'divider',
  BORDER_MEDIUM: 'grey.300',
  BORDER_DARK: 'grey.700',
} as const;

/**
 * Animation/timing constants
 */
export const ANIMATIONS = {
  // Transition durations
  FAST: '0.1s',
  NORMAL: '0.2s',
  SLOW: '0.4s',

  // Easing functions
  EASE_IN: 'ease-in',
  EASE_OUT: 'ease-out',
  EASE_IN_OUT: 'ease-in-out',

  // Common transitions
  SMOOTH_TRANSITION: 'all 0.2s ease-in-out',
  FADE_TRANSITION: 'opacity 0.3s ease-in-out',
} as const;

/**
 * Component-specific constants
 */
export const COMPONENT = {
  // Button variations
  BUTTON_SIZES: {
    SMALL: 'small',
    MEDIUM: 'medium',
    LARGE: 'large',
  } as const,

  // Input variations
  INPUT_VARIANTS: {
    OUTLINED: 'outlined',
    FILLED: 'filled',
    STANDARD: 'standard',
  } as const,

  // Paper elevations
  PAPER_ELEVATIONS: {
    NONE: 0,
    LOW: 1,
    MEDIUM: 3,
    HIGH: 6,
  } as const,

  // Tab panel padding (Material-UI standard)
  TAB_PANEL_PADDING: SPACING.md,
} as const;

/**
 * Icon size constants
 */
export const ICON_SIZES = {
  SMALL: 16,
  MEDIUM: 20,
  LARGE: 24,
  XLARGE: 32,
} as const;

/**
 * Z-index layers for proper stacking
 */
export const Z_INDEX = {
  DROPDOWN: 1000,
  STICKY: 1020,
  MODAL: 1300,
  POPOVER: 1301,
  TOOLTIP: 1500,
} as const;

/**
 * Typography scale constants (Material-UI variants)
 */
export const TYPOGRAPHY = {
  VARIANTS: {
    H1: 'h1',
    H2: 'h2',
    H3: 'h3',
    H4: 'h4',
    H5: 'h5',
    H6: 'h6',
    SUBTITLE1: 'subtitle1',
    SUBTITLE2: 'subtitle2',
    BODY1: 'body1',
    BODY2: 'body2',
    CAPTION: 'caption',
    OVERLINE: 'overline',
    BUTTON: 'button',
  } as const,

  FONT_WEIGHTS: {
    LIGHT: 300,
    REGULAR: 400,
    MEDIUM: 500,
    BOLD: 600,
    BOLDER: 700,
  } as const,
} as const;

/**
 * Pattern constants for commonly used prop combinations
 */
export const PATTERNS = {
  // Common flex layouts
  CENTERED: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
  },

  // Space between layouts
  SPACE_BETWEEN: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },

  // Flex column
  COLUMN: {
    display: 'flex',
    flexDirection: 'column',
  },

  // Flex row with gap
  ROW_WITH_GAP: (gap = SPACING.sm) => ({
    display: 'flex',
    gap: gap,
    alignItems: 'center',
  }),

  // Scrollable container
  SCROLLABLE: (maxHeight = LAYOUT.MAX_CONTENT_HEIGHT) => ({
    maxHeight: maxHeight,
    overflow: 'auto',
  }),

  // Bordered container
  BORDERED: {
    border: `1px solid`,
    borderColor: COLORS.BORDER_LIGHT,
    borderRadius: 1,
  },
} as const;

/**
 * Screen size breakpoints (Material-UI standard)
 */
export const BREAKPOINTS = {
  xs: 0,
  sm: 600,
  md: 960,
  lg: 1280,
  xl: 1920,
} as const;

/**
 * Media query helpers
 */
export const MEDIA_QUERIES = {
  UP: (breakpoint: keyof typeof BREAKPOINTS) => `@media (min-width: ${BREAKPOINTS[breakpoint]}px)`,
  DOWN: (breakpoint: keyof typeof BREAKPOINTS) =>
    `@media (max-width: ${BREAKPOINTS[breakpoint] - 1}px)`,
} as const;
