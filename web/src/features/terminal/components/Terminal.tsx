import * as React from 'react';
import { Box, TextField, Button, IconButton, Paper, Typography } from '@mui/material';
import { Close } from '@mui/icons-material';

interface TerminalProps {
  terminalOpen: boolean;
  terminalProgram: string;
  terminalArgs: string;
  terminalDir: string;
  terminalId: string;
  terminalLines: string[];
  onTerminalProgramChange: (value: string) => void;
  onTerminalArgsChange: (value: string) => void;
  onTerminalDirChange: (value: string) => void;
  onStartTerminal: () => void;
  onCloseTerminal: () => void;
}

const Terminal: React.FC<TerminalProps> = ({
  terminalOpen,
  terminalProgram,
  terminalArgs,
  terminalDir,
  terminalLines,
  onTerminalProgramChange,
  onTerminalArgsChange,
  onTerminalDirChange,
  onStartTerminal,
  onCloseTerminal,
}) => {
  if (!terminalOpen) return null;

  return (
    <Paper
      elevation={3}
      sx={{
        width: '100%',
        height: '300px',
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: '#1E1E1E',
        color: '#FFFFFF',
      }}
    >
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          p: 1,
          borderBottom: '1px solid #333',
        }}
      >
        <Typography variant="subtitle2">Terminal</Typography>
        <IconButton size="small" onClick={onCloseTerminal} sx={{ color: '#FFFFFF' }}>
          <Close fontSize="small" />
        </IconButton>
      </Box>

      <Box sx={{ p: 1, display: 'flex', gap: 1, borderBottom: '1px solid #333' }}>
        <TextField
          size="small"
          label="Program"
          value={terminalProgram}
          onChange={(e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
            onTerminalProgramChange((e.target as any).value)
          }
          variant="outlined"
          sx={{ flex: 1 }}
          InputLabelProps={{ style: { color: '#999' } }}
          inputProps={{ style: { color: '#FFF' } }}
        />
        <TextField
          size="small"
          label="Arguments"
          value={terminalArgs}
          onChange={(e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
            onTerminalArgsChange((e.target as any).value)
          }
          variant="outlined"
          sx={{ flex: 2 }}
          InputLabelProps={{ style: { color: '#999' } }}
          inputProps={{ style: { color: '#FFF' } }}
        />
        <TextField
          size="small"
          label="Directory"
          value={terminalDir}
          onChange={(e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
            onTerminalDirChange((e.target as any).value)
          }
          variant="outlined"
          sx={{ flex: 2 }}
          InputLabelProps={{ style: { color: '#999' } }}
          inputProps={{ style: { color: '#FFF' } }}
        />
        <Button variant="contained" onClick={onStartTerminal} sx={{ minWidth: '100px' }}>
          Start
        </Button>
      </Box>

      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
          p: 1,
          fontFamily: 'monospace',
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
        }}
      >
        {terminalLines.map((line: string, index: number) => (
          <div key={index} style={{ lineHeight: '1.5' }}>
            {line}
          </div>
        ))}
      </Box>
    </Paper>
  );
};

export default Terminal;
