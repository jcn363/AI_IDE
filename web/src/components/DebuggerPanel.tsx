import React, { useCallback, useState } from 'react';
import {
  Box,
  Button,
  CircularProgress,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  Paper,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import {
  Pause as PauseIcon,
  PlayArrow as StartIcon,
  Input as StepIntoIcon,
  Output as StepOutIcon,
  SkipNext as StepOverIcon,
  Stop as StopIcon,
} from '@mui/icons-material';
import { useAppDispatch, useAppSelector } from '../store/store';
import { debuggerActions, selectDebugger } from '../store/slices/debuggerSlice';
import { DebuggerService } from '../services/debuggerService';
import { VariablesList } from './VariablesList';

interface DebuggerControlsProps {
  onError: (error: string | null) => void;
  isRunning: boolean;
  isPaused: boolean;
  onStart: (execPath: string, workDir: string) => Promise<void>;
  onPause: () => Promise<void>;
  onContinue: () => Promise<void>;
  onStepOver: () => Promise<void>;
  onStepInto: () => Promise<void>;
  onStepOut: () => Promise<void>;
  onStop: () => Promise<void>;
}

const DebuggerControls = React.memo<DebuggerControlsProps>(({
  onError,
  isRunning,
  isPaused,
  onStart,
  onPause,
  onContinue,
  onStepOver,
  onStepInto,
  onStepOut,
  onStop,
}) => {
  const [execPath, setExecPath] = useState('');
  const [workDir, setWorkDir] = useState('');
  const [busy, setBusy] = useState(false);

  const handleStart = useCallback(async () => {
    if (!execPath || !workDir) {
      onError('Executable path and working directory are required');
      return;
    }
    setBusy(true);
    onError(null);
    try {
      await onStart(execPath, workDir);
    } catch (err) {
      onError(err instanceof Error ? err.message : 'Failed to start debugger');
      console.error('Debugger start error:', err);
    } finally {
      setBusy(false);
    }
  }, [execPath, workDir, onError, onStart]);

  const withBusy = useCallback(
    (fn: () => Promise<void>) => async () => {
      setBusy(true);
      try {
        await fn();
      } finally {
        setBusy(false);
      }
    },
    [],
  );

  const handlePause = withBusy(onPause);
  const handleContinue = withBusy(onContinue);
  const handleStepOver = withBusy(onStepOver);
  const handleStepInto = withBusy(onStepInto);
  const handleStepOut = withBusy(onStepOut);
  const handleStop = withBusy(onStop);

  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <TextField
        size="small"
        label="Executable"
        value={execPath}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setExecPath((e.target as any).value)}
        sx={{ minWidth: 240 }}
        aria-label="Executable path"
      />
      <TextField
        size="small"
        label="Working Dir"
        value={workDir}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setWorkDir((e.target as any).value)}
        sx={{ minWidth: 240 }}
        aria-label="Working directory"
      />
      <Button
        variant="contained"
        onClick={handleStart}
        disabled={busy || isRunning || !execPath || !workDir}
        startIcon={busy ? <CircularProgress size={16} /> : <StartIcon />}
        aria-label="Start debugging session"
      >
        Start
      </Button>
      <Button
        onClick={handlePause}
        disabled={busy || !isRunning || isPaused}
        startIcon={<PauseIcon />}
        aria-label="Pause debugging session"
      >
        Pause
      </Button>
      <Button
        onClick={handleContinue}
        disabled={busy || !isPaused}
        startIcon={<StartIcon />}
        aria-label="Continue execution"
      >
        Continue
      </Button>
      <Button
        onClick={handleStepOver}
        disabled={busy || !isPaused}
        startIcon={<StepOverIcon />}
        aria-label="Step over"
      >
        Step Over
      </Button>
      <Button
        onClick={handleStepInto}
        disabled={busy || !isPaused}
        startIcon={<StepIntoIcon />}
        aria-label="Step into"
      >
        Step Into
      </Button>
      <Button
        onClick={handleStepOut}
        disabled={busy || !isPaused}
        startIcon={<StepOutIcon />}
        aria-label="Step out"
      >
        Step Out
      </Button>
      <Button
        onClick={handleStop}
        color="error"
        disabled={busy || !isRunning}
        startIcon={<StopIcon />}
        aria-label="Stop debugging session"
      >
        Stop
      </Button>
    </Stack>
  );
});

export default function DebuggerPanel() {
  const {
    variables = [],
    callStack = [],
    breakpoints = [],
    lastEvalResult = '',
    state: debuggerState = 'stopped',
  } = useAppSelector(selectDebugger) || {};

  const dispatch = useAppDispatch();
  const [varsError, setVarsError] = useState<string | null>(null);
  const [expr, setExpr] = useState('');

  const isRunning = debuggerState === 'running';
  const isPaused = debuggerState === 'paused';

  // Memoize service callbacks so DebuggerControls (memoized) doesn't re-render unnecessarily
  const startCb = useCallback(async (execPath: string, workDir: string) => {
    await DebuggerService.start(execPath, workDir, []);
    dispatch(debuggerActions.openDebugger());
  }, [dispatch]);

  const pauseCb = useCallback(async () => {
    await DebuggerService.pause();
  }, []);

  const contCb = useCallback(async () => {
    await DebuggerService.cont();
  }, []);

  const stepOverCb = useCallback(async () => {
    await DebuggerService.stepOver();
  }, []);

  const stepIntoCb = useCallback(async () => {
    await DebuggerService.stepInto();
  }, []);

  const stepOutCb = useCallback(async () => {
    await DebuggerService.stepOut();
  }, []);

  const stopCb = useCallback(async () => {
    await DebuggerService.stop();
  }, []);

  const selectFrame = useCallback(async (frameId: number) => {
    try {
      await DebuggerService.selectFrame(frameId);
    } catch (e) {
      console.error('Select frame failed:', e);
    }
  }, []);

  const toggleBp = useCallback(async (id: number) => {
    try {
      await DebuggerService.toggleBreakpoint(id);
    } catch (e) {
      console.error('Toggle breakpoint failed:', e);
    }
  }, []);

  const removeBp = useCallback(async (id: number) => {
    try {
      await DebuggerService.removeBreakpoint(id);
    } catch (e) {
      console.error('Remove breakpoint failed:', e);
    }
  }, []);

  const evaluateExpr = useCallback(async () => {
    if (!expr) return;
    try {
      const result = await DebuggerService.evaluate(expr);
      dispatch(debuggerActions.setEvalResult(result));
    } catch (err) {
      console.error('Evaluation error:', err);
    }
  }, [dispatch, expr]);

  return (
    <Box component="div" sx={{ display: 'flex', flexDirection: 'column', height: '100%', gap: 2, p: 2 }}>
      <Typography variant="h6">Debugger</Typography>

      <DebuggerControls
        onError={setVarsError}
        isRunning={isRunning}
        isPaused={isPaused}
        onStart={startCb}
        onPause={pauseCb}
        onContinue={contCb}
        onStepOver={stepOverCb}
        onStepInto={stepIntoCb}
        onStepOut={stepOutCb}
        onStop={stopCb}
      />

      <Typography variant="subtitle1">Variables</Typography>
      <VariablesList
        variables={variables}
        isLoading={false}
        error={varsError}
        onError={setVarsError}
      />

      <Typography variant="subtitle1">Call Stack</Typography>
      <List dense sx={{ maxHeight: 120, overflow: 'auto', border: '1px solid', borderColor: 'divider' }}>
        {callStack.map((f) => (
          <ListItem key={f.id} disablePadding>
            <ListItemButton onClick={() => selectFrame(f.id)}>
              <ListItemText primary={f.function} secondary={`${f.file}:${f.line}`} />
            </ListItemButton>
          </ListItem>
        ))}
      </List>

      <Typography variant="subtitle1">Breakpoints</Typography>
      <List dense sx={{ maxHeight: 120, overflow: 'auto', border: '1px solid', borderColor: 'divider' }}>
        {breakpoints.map((b) => (
          <ListItem
            key={String(b.id)}
            secondaryAction={
              <Stack direction="row" spacing={1}>
                <Button size="small" onClick={() => toggleBp(b.id)}>
                  {b.enabled ? 'Disable' : 'Enable'}
                </Button>
                <Button size="small" color="error" onClick={() => removeBp(b.id)}>
                  Remove
                </Button>
              </Stack>
            }
          >
            <ListItemText primary={`${b.file}:${b.line}`} secondary={b.condition || ''} />
          </ListItem>
        ))}
      </List>

      <Paper variant="outlined" sx={{ p: 2 }}>
        <Typography variant="subtitle1" gutterBottom>
          Watch / Evaluate
        </Typography>
        <Stack direction="row" spacing={1} alignItems="center">
          <TextField
            fullWidth
            size="small"
            value={expr}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => setExpr((e.target as any).value)}
            placeholder="Enter expression to evaluate"
            disabled={!isRunning && !isPaused}
            onKeyDown={async (e) => {
              if (e.key === 'Enter' && expr) {
                e.preventDefault();
                await evaluateExpr();
              }
            }}
            aria-label="Expression to evaluate"
          />
          <Button
            variant="outlined"
            onClick={evaluateExpr}
            disabled={!expr || !isPaused}
            sx={{ whiteSpace: 'nowrap' }}
            aria-label="Evaluate expression"
          >
            Evaluate
          </Button>
        </Stack>
        {lastEvalResult && (
          <Box sx={{ mt: 2, p: 1, bgcolor: 'background.default', borderRadius: 1, fontFamily: 'monospace' }}>
            {lastEvalResult}
          </Box>
        )}
      </Paper>
    </Box>
  );
}
