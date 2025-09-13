import React, { useState, useMemo, KeyboardEvent } from 'react';
import {
  Box,
  Button,
  Divider,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  Tooltip,
  Typography,
  useTheme,
  styled,
  ListItemButtonProps,
} from '@mui/material';
import { VersionAlignment } from '../../types';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import RadioButtonUncheckedIcon from '@mui/icons-material/RadioButtonUnchecked';
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined';

interface VersionItemProps extends ListItemButtonProps {
  selected?: boolean;
  theme?: any;
}

const VersionItem = styled(ListItemButton, {
  shouldForwardProp: (prop) => prop !== 'selected',
})<VersionItemProps>(({ theme, selected = false }) => ({
  borderRadius: theme.shape.borderRadius,
  marginBottom: theme.spacing(1),
  border: `1px solid ${selected ? theme.palette.primary.main : theme.palette.divider}`,
  backgroundColor: selected ? theme.palette.action.selected : 'transparent',
  '&:hover': {
    backgroundColor: theme.palette.action.hover,
  },
  '&:focus-visible': {
    outline: `2px solid ${theme.palette.primary.main}`,
    outlineOffset: '2px',
  },
  '&.MuiListItemButton-root': {
    display: 'flex',
    alignItems: 'center',
    '&.Mui-focusVisible': {
      backgroundColor: theme.palette.action.selected,
    },
  },
}));

interface VisualConflictResolutionAssistantProps {
  alignment: VersionAlignment;
  onResolve: (resolution: VersionResolution) => void;
  onCancel: () => void;
}

interface VersionResolution {
  selectedVersion: string;
  resolutionNote?: string;
}

export const VisualConflictResolutionAssistant: React.FC<
  VisualConflictResolutionAssistantProps
> = ({ alignment, onResolve, onCancel }) => {
  const theme = useTheme();
  const [selectedVersion, setSelectedVersion] = useState(alignment.suggestedVersion);

  const versions = useMemo(() => {
    return Object.entries(alignment.currentVersions).map(([pkg, version]) => ({
      pkg,
      version,
      isSelected: version === alignment.suggestedVersion,
    }));
  }, [alignment]);

  const handleVersionSelect = (version: string) => {
    setSelectedVersion(version);
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLElement>, version: string) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleVersionSelect(version);
    }
  };

  return (
    <Paper elevation={3} sx={{ p: 3, mt: 2 }}>
      <Typography variant="h6" gutterBottom>
        Resolve Version Conflict: {alignment.dependencyName}
      </Typography>

      <Typography variant="body2" color="text.secondary" paragraph>
        The following packages are using different versions of this dependency:
      </Typography>

      <Box sx={{ mb: 3 }}>
        <List dense disablePadding>
          {Object.entries(alignment.currentVersions).map(([pkg, version]) => {
            const isSelected = selectedVersion === version;
            return (
              <VersionItem
                key={`${pkg}-${version}`}
                onClick={() => handleVersionSelect(version)}
                onKeyDown={(e) => handleKeyDown(e, version)}
                selected={isSelected}
                tabIndex={0}
                data-testid={`version-${version}`}
                component="li"
              >
                <ListItemIcon sx={{ minWidth: 40 }}>
                  {isSelected ? (
                    <CheckCircleIcon color="primary" />
                  ) : (
                    <RadioButtonUncheckedIcon color="action" />
                  )}
                </ListItemIcon>
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <Typography variant="body1" component="span" fontWeight={500}>
                        {pkg}
                      </Typography>
                      <Tooltip
                        title={`Version details for ${pkg}@${version}`}
                        arrow
                        placement="right"
                      >
                        <IconButton size="small" sx={{ ml: 1, opacity: 0.7 }}>
                          <InfoOutlinedIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                    </Box>
                  }
                  secondary={
                    <>
                      <Typography component="span" variant="body2" color="text.primary">
                        Version: {version}
                      </Typography>
                      {version === alignment.suggestedVersion && (
                        <Typography
                          component="span"
                          variant="caption"
                          color="primary"
                          sx={{
                            ml: 1,
                            px: 0.5,
                            py: 0.25,
                            bgcolor: 'primary.50',
                            borderRadius: 0.5,
                          }}
                        >
                          Suggested
                        </Typography>
                      )}
                    </>
                  }
                  secondaryTypographyProps={{ component: 'div' }}
                />
              </VersionItem>
            );
          })}
        </List>
      </Box>

      <Divider sx={{ my: 2 }} />

      <Box display="flex" justifyContent="flex-end" gap={2} mt={2}>
        <Button variant="outlined" onClick={onCancel}>
          Cancel
        </Button>
        <Button
          variant="contained"
          color="primary"
          onClick={() =>
            onResolve({
              selectedVersion: alignment.suggestedVersion,
            })
          }
        >
          Apply Resolution
        </Button>
      </Box>
    </Paper>
  );
};

export default VisualConflictResolutionAssistant;
