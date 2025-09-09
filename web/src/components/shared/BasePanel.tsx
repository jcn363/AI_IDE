import {
  Error as ErrorIcon,
  ExpandLess as ExpandLessIcon,
  ExpandMore as ExpandMoreIcon,
  Info as InfoIcon,
  CheckCircle as SuccessIcon,
  Warning as WarningIcon,
} from '@mui/icons-material';
import {
  Alert,
  Badge,
  Box,
  Chip,
  CircularProgress,
  Collapse,
  IconButton,
  Paper,
  SxProps,
  Theme,
  Typography,
} from '@mui/material';
import React, { useState } from 'react';

import { BasePanelProps, PANEL_CONSTANTS } from '../../utils/consolidated';

/**
 * Enhanced panel props extending base props with additional layout options
 */
export interface EnhancedPanelProps extends BasePanelProps {
  /**
   * Whether the panel can be collapsed
   */
  collapsible?: boolean;
  /**
   * Initial collapsed state
   */
  initiallyCollapsed?: boolean;
  /**
   * Status to display in header
   */
  status?: 'success' | 'warning' | 'error' | 'info';
  /**
   * Status message
   */
  statusMessage?: string;
  /**
   * Badge to show in header (count, etc.)
   */
  badge?: number | string;
  /**
   * Show toolbar area below header
   */
  toolbar?: React.ReactNode;
  /**
   * Custom header content
   */
  headerContent?: React.ReactNode;
  /**
   * Elevation level
   */
  elevation?: number;
  /**
   * Border variant
   */
  variant?: 'elevation' | 'outlined';
  /**
   * Panel style variant
   */
  panelStyle?: 'default' | 'card' | 'minimal';
  /**
   * Footer content
   */
  footer?: React.ReactNode;
  /**
   * Show loading state
   */
  loading?: boolean;
  /**
   * Show dividers between sections
   */
  showDividers?: boolean;
  /**
   * Custom status icon
   */
  statusIcon?: React.ReactNode;
  /**
   * MUI sx styles prop for inline styling
   */
  sx?: SxProps<Theme>;
}

/**
 * BasePanel - Enhanced foundational component for all panels
 *
 * Features:
 * - Collapsible sections
 * - Toolbar areas
 * - Status indicators
 * - Flexible layout options
 * - Consistent spacing and theming
 */
export const BasePanel: React.FC<EnhancedPanelProps> = ({
  title,
  headerActions,
  toolbar,
  headerContent,
  footer,
  children,
  fullHeight = true,
  padded = true,
  collapsible = false,
  initiallyCollapsed = false,
  status,
  statusMessage,
  statusIcon,
  badge,
  elevation = 1,
  variant = 'elevation',
  panelStyle = 'default',
  loading = false,
  showDividers = false,
  className,
  style,
  'data-testid': testId,
  sx,
  ...paperProps
}) => {
  const [collapsed, setCollapsed] = useState(initiallyCollapsed);

  // Base container styles based on panel style
  const baseStyles: SxProps<Theme> = {
    display: 'flex',
    flexDirection: 'column',
    transition: 'all 0.2s ease-in-out',
    ...panelStyle === 'card' && {
      borderRadius: 3,
      overflow: 'hidden',
    },
    ...panelStyle === 'minimal' && {
      boxShadow: 'none',
      border: 1,
      borderColor: 'divider',
    },
    ...sx,
  };

  // Header styles
  const headerStyles: SxProps<Theme> = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    minHeight: PANEL_CONSTANTS.HEADER_HEIGHT * 0.75,
    px: padded ? 2 : 1,
    py: 1.5,
    backgroundColor: panelStyle === 'card' ? 'background.paper' : 'transparent',
    borderBottom: showDividers ? 1 : 0,
    borderBottomColor: 'divider',
  };

  // Toolbar styles
  const toolbarStyles: SxProps<Theme> = {
    display: 'flex',
    alignItems: 'center',
    px: padded ? 2 : 1,
    py: 1,
    backgroundColor: 'grey.50',
    borderTop: 1,
    borderTopColor: 'divider',
    gap: 1,
  };

  // Content area styles
  const contentStyles: SxProps<Theme> = {
    flex: 1,
    overflow: 'auto',
    px: padded ? 2 : 0,
    py: padded ? 2 : 0,
    opacity: collapsed ? 0.5 : 1,
    transition: 'opacity 0.2s ease-in-out',
  };

  // Footer styles
  const footerStyles: SxProps<Theme> = {
    borderTop: 1,
    borderTopColor: 'divider',
    px: padded ? 2 : 1,
    py: 1.5,
    backgroundColor: 'grey.100',
  };

  // Status icon helper
  const getStatusIcon = () => {
    if (statusIcon) return statusIcon;

    switch (status) {
      case 'success': return <SuccessIcon color="success" />;
      case 'warning': return <WarningIcon color="warning" />;
      case 'error': return <ErrorIcon color="error" />;
      case 'info': return <InfoIcon color="info" />;
      default: return null;
    }
  };

  // Status indicator styles
  const statusIndicatorStyles: SxProps<Theme> = {
    display: 'flex',
    alignItems: 'center',
    gap: 1,
  };

  return (
    <Paper
      className={className}
      style={style}
      data-testid={testId}
      sx={baseStyles}
      elevation={elevation}
      variant={variant}
      {...paperProps}
    >
      {/* Header Section */}
      {(title || headerActions || status || headerContent) && (
        <Box sx={headerStyles}>
          {/* Title with badge and collapsible */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {collapsible && title && (
              <IconButton
                size="small"
                onClick={() => setCollapsed(!collapsed)}
                sx={{ p: 0.5 }}
              >
                {collapsed ? (
                  <ExpandMoreIcon sx={{ fontSize: 18 }} />
                ) : (
                  <ExpandLessIcon sx={{ fontSize: 18 }} />
                )}
              </IconButton>
            )}

            {title && (
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Typography variant="h6" component="h2">
                  {title}
                </Typography>

                {badge && (
                  <Badge
                    badgeContent={badge}
                    color="primary"
                    sx={{ ml: 1 }}
                  />
                )}
              </Box>
            )}

            {/* Status indicator */}
            {status && (
              <Box sx={statusIndicatorStyles}>
                {getStatusIcon()}
                {statusMessage && (
                  <Chip
                    label={statusMessage}
                    size="small"
                    color={status}
                    variant="outlined"
                  />
                )}
              </Box>
            )}
          </Box>

          {/* Custom header content and actions */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {headerContent}
            {headerActions}
          </Box>
        </Box>
      )}

      {/* Status message below header */}
      {status && statusMessage && !collapsed && (
        <Box sx={{ px: 2, pb: 1 }}>
          <Alert severity={status} variant="outlined" sx={{ mb: 0 }}>
            {statusMessage}
          </Alert>
        </Box>
      )}

      {/* Toolbar Area */}
      {toolbar && (
        <Box sx={toolbarStyles}>
          {toolbar}
        </Box>
      )}

      {/* Content Area */}
      {children && (
        <Collapse in={!collapsed} timeout="auto">
          <Box sx={contentStyles}>
            {children}
          </Box>
        </Collapse>
      )}

      {/* Footer */}
      {footer && (
        <Box sx={footerStyles}>
          {footer}
        </Box>
      )}

      {/* Loading overlay */}
      {loading && (
        <Box
          sx={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            bgcolor: 'rgba(255, 255, 255, 0.8)',
            zIndex: 1,
            borderRadius: 1,
          }}
        >
          <CircularProgress size={32} />
        </Box>
      )}
    </Paper>
  );
};

// Re-export for backward compatibility
export default BasePanel;