import React from 'react';
import { Box, Paper, Typography, SxProps, Theme, PaperProps } from '@mui/material';
import { BasePanelProps, PANEL_CONSTANTS } from '../../utils/consolidated';

/**
 * Extended panel props combining base props with Paper props
 */
export interface ExtendedPanelProps
  extends BasePanelProps,
    Omit<PaperProps, keyof BasePanelProps> {}

/**
 * BasePanel - A foundational component for all panels
 *
 * Provides consistent structure: header with title/actions, content area with proper spacing.
 * Replaces repetitive Paper + Box + Typography patterns scattered throughout the codebase.
 */
export const BasePanel: React.FC<ExtendedPanelProps> = ({
  title,
  headerActions,
  children,
  fullHeight = true,
  padded = true,
  className,
  style,
  'data-testid': testId,
  sx,
  ...paperProps
}) => {
  // Base container styles - consistent foundation for all panels
  const baseStyles: SxProps<Theme> = {
    p: padded ? PANEL_CONSTANTS.PADDING : 0,
    height: fullHeight ? PANEL_CONSTANTS.MAX_HEIGHT : 'auto',
    display: 'flex',
    flexDirection: 'column',
    ...sx,
  };

  // Header section styles
  const headerStyles: SxProps<Theme> = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    mb: title || headerActions ? 2 : 0,
    minHeight: PANEL_CONSTANTS.HEADER_HEIGHT * 0.75,
  };

  // Content area styles - flex-grow ensures it takes remaining space
  const contentStyles: SxProps<Theme> = {
    flex: 1,
    overflow: 'auto',
  };

  return (
    <Paper className={className} style={style} data-testid={testId} {...paperProps} sx={baseStyles}>
      {/* Optional Header Section */}
      {(title || headerActions) && (
        <Box sx={headerStyles}>
          {title && (
            <Typography variant="h6" component="h2">
              {title}
            </Typography>
          )}
          {headerActions && <Box sx={{ display: 'flex', gap: 1 }}>{headerActions}</Box>}
        </Box>
      )}

      {/* Content Area */}
      <Box sx={contentStyles}>{children}</Box>
    </Paper>
  );
};

/**
 * PanelSection - For sectioning content within larger panels
 */
export const PanelSection: React.FC<
  React.PropsWithChildren<{
    title?: string;
    actions?: React.ReactNode;
    padded?: boolean;
    sx?: SxProps<Theme>;
  }>
> = ({ title, actions, children, padded = true, sx = {} }) => {
  const sectionStyles: SxProps<Theme> = {
    mb: 3,
    '&:last-child': { mb: 0 },
    ...sx,
  };

  return (
    <Box sx={sectionStyles}>
      {(title || actions) && (
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            mb: 2,
            minHeight: PANEL_CONSTANTS.TOOLBAR_HEIGHT * 0.75,
          }}
        >
          {title && (
            <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>
              {title}
            </Typography>
          )}
          {actions && <Box sx={{ display: 'flex', gap: 1 }}>{actions}</Box>}
        </Box>
      )}
      <Box sx={{ p: padded ? 1 : 0 }}>{children}</Box>
    </Box>
  );
};

export default BasePanel;
