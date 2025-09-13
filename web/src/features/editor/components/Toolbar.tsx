import * as React from 'react';
import { Box, IconButton, Tooltip } from '@mui/material';
import { Save, Code, FolderOpen, Settings } from '@mui/icons-material';

interface ToolbarProps {
  onSave: () => void;
  onOpenFile: () => void;
  onOpenSettings: () => void;
  onRunCode: () => void;
  isSaving: boolean;
}

const Toolbar: React.FC<ToolbarProps> = ({
  onSave,
  onOpenFile,
  onOpenSettings,
  onRunCode,
  isSaving,
}) => {
  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        padding: (theme) => theme.spacing(0.5, 2),
        borderBottom: (theme) => `1px solid ${theme.palette.divider}`,
        backgroundColor: (theme) => theme.palette.background.paper,
        minHeight: '48px',
      }}
    >
      <Tooltip title="Save (Ctrl+S)">
        <span>
          <IconButton
            onClick={onSave}
            disabled={isSaving}
            size="small"
            sx={{
              margin: (theme) => theme.spacing(0, 0.5),
              padding: (theme) => theme.spacing(1),
              borderRadius: (theme) => theme.shape.borderRadius,
              '&:hover': {
                backgroundColor: (theme) => theme.palette.action.hover,
              },
            }}
          >
            <Save fontSize="small" />
          </IconButton>
        </span>
      </Tooltip>

      <Tooltip title="Open File">
        <IconButton
          onClick={onOpenFile}
          size="small"
          sx={{
            margin: (theme) => theme.spacing(0, 0.5),
            padding: (theme) => theme.spacing(1),
            borderRadius: (theme) => theme.shape.borderRadius,
            '&:hover': {
              backgroundColor: (theme) => theme.palette.action.hover,
            },
          }}
        >
          <FolderOpen fontSize="small" />
        </IconButton>
      </Tooltip>

      <Tooltip title="Run Code">
        <IconButton
          onClick={onRunCode}
          size="small"
          sx={{
            margin: (theme) => theme.spacing(0, 0.5),
            padding: (theme) => theme.spacing(1),
            borderRadius: (theme) => theme.shape.borderRadius,
            '&:hover': {
              backgroundColor: (theme) => theme.palette.action.hover,
            },
          }}
        >
          <Code fontSize="small" />
        </IconButton>
      </Tooltip>

      <Box sx={{ flexGrow: 1 }} />

      <Tooltip title="Settings">
        <IconButton
          onClick={onOpenSettings}
          size="small"
          sx={{
            margin: (theme) => theme.spacing(0, 0.5),
            padding: (theme) => theme.spacing(1),
            borderRadius: (theme) => theme.shape.borderRadius,
            '&:hover': {
              backgroundColor: (theme) => theme.palette.action.hover,
            },
          }}
        >
          <Settings fontSize="small" />
        </IconButton>
      </Tooltip>
    </Box>
  );
};

export default Toolbar;
