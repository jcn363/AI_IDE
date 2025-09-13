import React from 'react';
import {
  Box,
  Button,
  IconButton,
  Tooltip,
  CircularProgress,
  Alert,
  Typography,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  SelectChangeEvent,
  FormControlLabel,
  Switch,
} from '@mui/material';
import {
  PlayArrow as RunIcon,
  Build as BuildIcon,
  CheckCircle as CheckIcon,
  BugReport as BugReportIcon,
  Close as CloseIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { useAppDispatch } from '../../store/store';
import {
  executeCargoStream,
  selectCargoState,
  selectCargoCommands,
} from '../../store/slices/cargoSlice';

interface CargoCommandsPanelProps {
  selectedCommand: string;
  commandArgs: string;
  showJsonDiagnostics: boolean;
  onCommandChange: (event: SelectChangeEvent<string>) => void;
  onArgsChange: React.ChangeEventHandler<HTMLInputElement>;
  onRunCommand: () => void;
  onJsonDiagnosticsChange: (checked: boolean) => void;
  isRunning: boolean;
  isCargoAvailable: boolean;
  error: string | null;
  onClearError: () => void;
}

const COMMAND_OPTIONS = [
  { value: 'build', label: 'Build', icon: <BuildIcon /> },
  { value: 'run', label: 'Run', icon: <RunIcon /> },
  { value: 'test', label: 'Test', icon: <BugReportIcon /> },
  { value: 'check', label: 'Check', icon: <CheckIcon /> },
  { value: 'clippy', label: 'Clippy', icon: <BugReportIcon /> },
  { value: 'fmt', label: 'Format', icon: <BugReportIcon /> },
];

export const CargoCommandsPanel: React.FC<CargoCommandsPanelProps> = ({
  selectedCommand,
  commandArgs,
  showJsonDiagnostics,
  onCommandChange,
  onArgsChange,
  onRunCommand,
  onJsonDiagnosticsChange,
  isRunning,
  isCargoAvailable,
  error,
  onClearError,
}) => {
  return (
    <Box>
      <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
        <FormControl sx={{ minWidth: 150 }} size="small">
          <InputLabel id="command-select-label">Command</InputLabel>
          <Select
            labelId="command-select-label"
            id="command-select"
            value={selectedCommand}
            label="Command"
            onChange={onCommandChange}
            disabled={isRunning || !isCargoAvailable}
          >
            {COMMAND_OPTIONS.map((option) => (
              <MenuItem key={option.value} value={option.value}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  {option.icon}
                  {option.label}
                </Box>
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <TextField
          label="Arguments"
          variant="outlined"
          size="small"
          fullWidth
          value={commandArgs}
          onChange={onArgsChange}
          disabled={isRunning || !isCargoAvailable}
          placeholder="--release --verbose"
        />

        <FormControlLabel
          control={
            <Switch
              size="small"
              checked={showJsonDiagnostics}
              onChange={(_, checked) => onJsonDiagnosticsChange(checked)}
            />
          }
          label={<Typography variant="caption">JSON diagnostics</Typography>}
          sx={{ ml: 1, mr: 1 }}
        />

        <Button
          variant="contained"
          onClick={onRunCommand}
          disabled={isRunning || !isCargoAvailable}
          startIcon={isRunning ? <CircularProgress size={20} /> : <RunIcon />}
        >
          {isRunning ? 'Running...' : 'Run'}
        </Button>
      </Box>

      {error && (
        <Alert
          severity="error"
          sx={{ mb: 2 }}
          onClose={onClearError}
          action={
            <IconButton aria-label="close" color="inherit" size="small" onClick={onClearError}>
              <CloseIcon fontSize="inherit" />
            </IconButton>
          }
        >
          {error}
        </Alert>
      )}
    </Box>
  );
};

export default CargoCommandsPanel;
