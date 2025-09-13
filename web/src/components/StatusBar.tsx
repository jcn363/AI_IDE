import * as React from 'react';
import { Box } from '@mui/material';

type StatusBarProps = {
  activeFilePath: string | null | string;
  isSaving: boolean;
  isConnected: boolean;
};

const StatusBar: React.FC<StatusBarProps> = ({ activeFilePath, isSaving, isConnected }) => {
  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        px: 2,
        py: 0.5,
        bgcolor: 'background.paper',
        borderTop: 1,
        borderColor: 'divider',
        minHeight: '24px',
        fontSize: '0.75rem',
        color: 'text.secondary',
      }}
    >
      {activeFilePath ? (
        <>
          <span>{String(activeFilePath).split('/').pop()}</span>
          <Box sx={{ ml: 2, color: 'text.secondary' }}>{isSaving ? 'Saving...' : 'Saved'}</Box>
        </>
      ) : (
        <span>No file open</span>
      )}

      <Box sx={{ flexGrow: 1 }} />

      {activeFilePath && (
        <Box sx={{ display: 'flex', alignItems: 'center' }}>
          <Box
            sx={{
              width: 6,
              height: 6,
              bgcolor: isConnected ? 'success.main' : 'error.main',
              borderRadius: '50%',
              mr: 1,
            }}
          />
          <span>{isConnected ? 'LSP: Connected' : 'LSP: Disconnected'}</span>
        </Box>
      )}
    </Box>
  );
};

export default StatusBar;
