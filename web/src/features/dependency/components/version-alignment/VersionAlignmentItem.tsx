import React, { useState } from 'react';
import {
  Box,
  Button,
  ButtonGroup,
  Chip,
  CircularProgress,
  Collapse,
  IconButton,
  ListItem,
  ListItemButton,
  ListItemText,
  Stack,
  Tooltip,
  Typography,
  alpha,
  useTheme,
} from '@mui/material';
import {
  ArrowRightAlt as ArrowIcon,
  Check as CheckIcon,
  Close as CloseIcon,
  Error as ErrorIcon,
  Info as InfoIcon,
  Warning as WarningIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
} from '@mui/icons-material';
import { VersionAlignment } from '../../types';
import { VisualConflictResolutionAssistant } from './VisualConflictResolutionAssistant';

interface VersionAlignmentItemProps {
  alignment: VersionAlignment;
  selected?: boolean;
  disabled?: boolean;
  onSelect?: () => void;
  onApply?: (alignment: VersionAlignment) => Promise<void> | void;
  onIgnore?: (alignment: VersionAlignment) => void;
  onResolveConflict?: (alignment: VersionAlignment, selectedVersion: string) => Promise<void> | void;
}

const VersionAlignmentItem: React.FC<VersionAlignmentItemProps> = ({
  alignment,
  selected = false,
  disabled = false,
  onSelect,
  onApply,
  onIgnore,
  onResolveConflict,
}) => {
  const theme = useTheme();
  const [isApplying, setIsApplying] = useState(false);
  const [showResolutionAssistant, setShowResolutionAssistant] = useState(false);
  
  const handleApply = async () => {
    if (disabled || !onApply) return;
    try {
      setIsApplying(true);
      await onApply(alignment);
    } finally {
      setIsApplying(false);
    }
  };

  const handleResolveConflict = async (resolution: { selectedVersion: string }) => {
    if (disabled || !onResolveConflict) return;
    try {
      setIsApplying(true);
      await onResolveConflict(alignment, resolution.selectedVersion);
      setShowResolutionAssistant(false);
    } finally {
      setIsApplying(false);
    }
  };
  
  const handleIgnore = () => {
    if (disabled || !onIgnore) return;
    onIgnore(alignment);
  };

  const hasConflict = Object.keys(alignment.currentVersions).length > 1;
  
  const { 
    dependencyName, 
    currentVersions, 
    suggestedVersion, 
    severity = 'medium', 
  } = alignment;
  
  const getSeverityColor = (): 'error' | 'warning' | 'info' | 'primary' => {
    switch (severity) {
      case 'high': return 'error';
      case 'medium': return 'warning';
      case 'low': return 'info';
      default: return 'primary';
    }
  };

  const severityColor = getSeverityColor();
  const hoverColor = severityColor === 'primary' 
    ? theme.palette.primary.main 
    : theme.palette[severityColor].main;
  
  const getSeverityIcon = () => {
    switch (severity) {
      case 'high': return <ErrorIcon fontSize="small" color="error" />;
      case 'medium': return <WarningIcon fontSize="small" color="warning" />;
      case 'low': return <InfoIcon fontSize="small" color="info" />;
      default: return null;
    }
  };
  
  const versionsCount = Object.keys(currentVersions).length;
  const hasMultipleVersions = versionsCount > 1;
  const versionsText = hasMultipleVersions 
    ? `${versionsCount} different versions`
    : Object.values(currentVersions)[0];

  return (
    <ListItem 
        component={onSelect ? 'div' : 'li'}
        disablePadding
        sx={{
          backgroundColor: selected ? alpha(theme.palette.primary.main, 0.04) : 'inherit',
          borderRadius: 1,
          mb: 1,
          transition: 'background-color 0.2s ease',
          '&:hover': {
            backgroundColor: theme.palette.action.hover,
          },
        }}
        secondaryAction={
          <Stack direction="row" spacing={1} alignItems="center">
            {hasConflict && onResolveConflict && (
              <Button
                size="small"
                variant="contained"
                color="warning"
                onClick={() => setShowResolutionAssistant(!showResolutionAssistant)}
                disabled={disabled}
                endIcon={showResolutionAssistant ? <ExpandLessIcon /> : <ExpandMoreIcon />}
              >
                {showResolutionAssistant ? 'Hide Resolver' : 'Resolve Conflict'}
              </Button>
            )}
            {onApply && (
              <Button
                size="small"
                variant="contained"
                color="primary"
                onClick={handleApply}
                disabled={disabled || isApplying || hasConflict}
                startIcon={
                  isApplying ? (
                    <CircularProgress size={16} color="inherit" />
                  ) : (
                    <CheckIcon />
                  )
                }
              >
                {isApplying ? 'Applying...' : 'Apply'}
              </Button>
            )}
            {onIgnore && (
              <Button
                size="small"
                variant="outlined"
                color="inherit"
                onClick={handleIgnore}
                disabled={disabled}
                startIcon={<CloseIcon />}
              >
                Ignore
              </Button>
            )}
          </Stack>
        }
      >
        <ListItemButton 
          onClick={onSelect}
          selected={selected}
          disabled={disabled}
          sx={{
            borderRadius: 1,
            py: 1.5,
            px: 2,
            '&.Mui-selected': {
              backgroundColor: 'transparent',
              '&:hover': {
                backgroundColor: 'transparent',
              },
            },
          }}
        >
          <ListItemText
            primary={
              <Box display="flex" alignItems="center" gap={1} mb={0.5}>
                {getSeverityIcon()}
                <Typography 
                  variant="subtitle1" 
                  component="span"
                  fontWeight={500}
                  sx={{ 
                    color: theme.palette.text.primary,
                    textTransform: 'capitalize',
                  }}
                >
                  {dependencyName}
                </Typography>
                <Chip
                  size="small"
                  label={
                    <Box display="flex" alignItems="center" gap={0.5}>
                      <Typography variant="caption" fontWeight={500}>
                        {versionsText}
                      </Typography>
                      <ArrowIcon fontSize="inherit" />
                      <Typography variant="caption" fontWeight={600}>
                        {suggestedVersion}
                      </Typography>
                    </Box>
                  }
color={severityColor}
                  variant="outlined"
                  sx={{
                    height: 24,
                    borderRadius: 1,
                    borderWidth: 1.5,
                    '& .MuiChip-label': {
                      px: 1,
                      py: 0.5,
                    },
                  }}
                />
              </Box>
            }
            secondary={
              <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                {Object.entries(currentVersions).map(([packageName, version]) => (
                  <Tooltip 
                    key={packageName}
                    title={`${packageName} is using version ${version}`}
                    arrow
                  >
                    <Chip
                      label={`${packageName}: ${version}`}
                      size="small"
                      variant="outlined"
                      sx={{
                        borderRadius: 1,
                        borderWidth: 1,
                        '& .MuiChip-label': {
                          px: 1,
                          py: 0.25,
                        },
                      }}
                    />
                  </Tooltip>
                ))}
              </Stack>
            }
        />
      </ListItemButton>
    </ListItem>
  );
};

export default VersionAlignmentItem;
