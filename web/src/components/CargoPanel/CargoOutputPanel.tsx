import React, { useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';
import { Box, Paper, Typography, IconButton, Tooltip } from '@mui/material';
import { Close as CloseIcon } from '@mui/icons-material';
import { selectCargoCommands } from '../../store/slices/cargoSlice';

interface CargoOutputPanelProps {
  onClose?: () => void;
}

const CargoOutputPanel: React.FC<CargoOutputPanelProps> = ({ onClose }) => {
  const commands = useSelector(selectCargoCommands);
  const endOfOutputRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new output arrives
  useEffect(() => {
    if (endOfOutputRef.current) {
      // @ts-ignore - scrollIntoView exists on the element
      endOfOutputRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [commands]);

  const commandList = Object.values(commands).sort((a, b) => b.timestamp - a.timestamp);

  if (commandList.length === 0) {
    return (
      <Paper sx={{ p: 2, height: '100%' }}>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Typography variant="h6">Cargo Output</Typography>
          {onClose && (
            <Tooltip title="Close panel">
              <IconButton onClick={onClose} size="small">
                <CloseIcon />
              </IconButton>
            </Tooltip>
          )}
        </Box>
        <Box
          display="flex"
          justifyContent="center"
          alignItems="center"
          height="calc(100% - 48px)"
        >
          <Typography color="textSecondary">
            No Cargo commands have been executed yet.
          </Typography>
        </Box>
      </Paper>
    );
  }

  return (
    <Paper sx={{ p: 2, height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
        <Typography variant="h6">Cargo Output</Typography>
        {onClose && (
          <Tooltip title="Close panel">
            <IconButton onClick={onClose} size="small">
              <CloseIcon />
            </IconButton>
          </Tooltip>
        )}
      </Box>
      <Box
        sx={{
          flexGrow: 1,
          overflow: 'auto',
          bgcolor: 'background.default',
          p: 2,
          borderRadius: 1,
          fontFamily: 'monospace',
          whiteSpace: 'pre-wrap',
          '& pre': { margin: 0, fontFamily: 'inherit' },
        }}
      >
        {commandList.map((cmd) => (
          <Box key={cmd.id} mb={2}>
            <Box
              sx={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                mb: 0.5,
                p: 0.5,
                bgcolor: cmd.status === 'error' ? 'error.dark' : 'primary.dark',
                color: 'common.white',
                borderRadius: 0.5,
              }}
            >
              <Typography variant="caption" fontFamily="monospace">
                $ cargo {cmd.command} {cmd.args.join(' ')}
              </Typography>
              <Typography variant="caption">
                {new Date(cmd.timestamp).toLocaleTimeString()}
              </Typography>
            </Box>
            <Box
              component="pre"
              sx={{
                m: 0,
                p: 1,
                bgcolor: 'background.paper',
                border: '1px solid',
                borderColor: 'divider',
                borderRadius: 0.5,
                overflowX: 'auto',
              }}
            >
              {cmd.output}
              {cmd.error && (
                <Box color="error.main" component="span">
                  {cmd.error}
                </Box>
              )}
            </Box>
          </Box>
        ))}
        <div ref={endOfOutputRef} />
      </Box>
    </Paper>
  );
};

export default CargoOutputPanel;
