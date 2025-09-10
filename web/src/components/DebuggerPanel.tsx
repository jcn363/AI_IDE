import React, { useCallback, useState, useEffect } from 'react';
import {
  Box,
  Button,
  CircularProgress,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  Stack,
  TextField,
  Typography,
  Tab,
  Tabs,
  Badge,
  Chip,
} from '@mui/material';
import {
  Pause as PauseIcon,
  PlayArrow as StartIcon,
  Input as StepIntoIcon,
  Output as StepOutIcon,
  SkipNext as StepOverIcon,
  Stop as StopIcon,
  Timeline,
  TaskAlt,
  Future,
  Memory,
  Warning,
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

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

interface TaskInfo {
  id: number;
  name: string;
  status: string;
  spawnLocation: string;
}

interface FutureInfo {
  id: number;
  expression: string;
  state: string;
}

interface DeadlockInfo {
  tasks: number[];
  description: string;
  severity: string;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`debug-tabpanel-${index}`}
      aria-labelledby={`debug-tab-${index}`}
      {...other}
    >
      {value === index && (
        <Box sx={{ p: 3 }}>
          {children}
        </Box>
      )}
    </div>
  );
}

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
  const [tabValue, setTabValue] = useState(0);

  // Async debugging state
  const [activeTasks, setActiveTasks] = useState<TaskInfo[]>([]);
  const [futures, setFutures] = useState<FutureInfo[]>([]);
  const [deadlocks, setDeadlocks] = useState<DeadlockInfo[]>([]);
  const [asyncError, setAsyncError] = useState<string | null>(null);

  const isRunning = debuggerState === 'running';
  const isPaused = debuggerState === 'paused';

  useEffect(() => {
    if (isRunning) {
      // Fetch async debugging data when debugger is running
      fetchAsyncDebuggingData();
    }
  }, [isRunning]);

  const fetchAsyncDebuggingData = useCallback(async () => {
    try {
      // Fetch tasks, futures, and deadlock info
      const [tasksResult, futuresResult, deadlocksResult] = await Promise.allSettled([
        DebuggerService.getActiveTasks?.() || Promise.resolve({ data: [] }),
        DebuggerService.getFutures?.() || Promise.resolve({ data: [] }),
        DebuggerService.detectDeadlocks?.() || Promise.resolve({ data: [] }),
      ]);

      if (tasksResult.status === 'fulfilled') {
        setActiveTasks(tasksResult.value.data);
      }
      if (futuresResult.status === 'fulfilled') {
        setFutures(futuresResult.value.data);
      }
      if (deadlocksResult.status === 'fulfilled') {
        setDeadlocks(deadlocksResult.value.data);
      }
      setAsyncError(null);
    } catch (err) {
      setAsyncError(err instanceof Error ? err.message : 'Failed to fetch async debugging data');
    }
  }, []);

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

  const inspectTask = useCallback(async (taskId: number) => {
    try {
      await DebuggerService.inspectTask?.(taskId);
    } catch (e) {
      console.error('Task inspection failed:', e);
    }
  }, []);

  const inspectFuture = useCallback(async (futureId: number) => {
    try {
      await DebuggerService.inspectFuture?.(futureId);
    } catch (e) {
      console.error('Future inspection failed:', e);
    }
  }, []);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };

  return (
    <Box component="div" sx={{ display: 'flex', flexDirection: 'column', height: '100%', p: 2 }}>
      <Stack direction="row" alignItems="center" spacing={2} mb={2}>
        <Typography variant="h6">Debugger</Typography>
        <Chip
          label={debuggerState.toUpperCase()}
          color={isRunning ? (isPaused ? 'warning' : 'success') : 'default'}
          size="small"
        />
      </Stack>

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

      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs
          value={tabValue}
          onChange={handleTabChange}
          aria-label="debugger tabs"
          variant="scrollable"
          scrollButtons="auto"
        >
          <Tab
            icon={<Timeline />}
            label="Synchronous"
            iconPosition="start"
            sx={{ minWidth: 'auto' }}
          />
          <Tab
            icon={<TaskAlt />}
            iconPosition="start"
            label={
              <Badge color="error" variant={deadlocks.length > 0 ? 'dot' : 'standard'}>
                Async Tasks
              </Badge>
            }
            sx={{ minWidth: 'auto' }}
          />
          <Tab
            icon={<Future />}
            iconPosition="start"
            label="Futures/Streams"
            sx={{ minWidth: 'auto' }}
          />
          <Tab
            icon={<Warning />}
            iconPosition="start"
            label={
              <Badge color="error" variant={deadlocks.length > 0 ? 'dot' : 'standard'}>
                Deadlocks
              </Badge>
            }
            sx={{ minWidth: 'auto' }}
          />
        </Tabs>
      </Box>

      {/* Synchronous Debugging Tab */}
      <TabPanel value={tabValue} index={0}>
        <Stack spacing={2}>
          <Paper variant="outlined" sx={{ p: 2 }}>
            <Typography variant="subtitle1">Variables</Typography>
            <VariablesList
              variables={variables}
              isLoading={false}
              error={varsError}
              onError={setVarsError}
            />
          </Paper>

          <Paper variant="outlined" sx={{ p: 2 }}>
            <Typography variant="subtitle1">Call Stack</Typography>
            <List dense sx={{ maxHeight: 200, overflow: 'auto', border: '1px solid', borderColor: 'divider' }}>
              {callStack.map((f) => (
                <ListItem key={f.id} disablePadding>
                  <ListItemButton onClick={() => selectFrame(f.id)}>
                    <ListItemText
                      primary={
                        <Stack direction="row" alignItems="center" spacing={1}>
                          <Typography variant="body2">{f.function}</Typography>
                          <Chip label={`${f.file}:${f.line}`} size="small" variant="outlined" />
                        </Stack>
                      }
                    />
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </Paper>

          <Paper variant="outlined" sx={{ p: 2 }}>
            <Typography variant="subtitle1">Breakpoints</Typography>
            <List dense sx={{ maxHeight: 200, overflow: 'auto', border: '1px solid', borderColor: 'divider' }}>
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
          </Paper>

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
        </Stack>
      </TabPanel>

      {/* Async Tasks Tab */}
      <TabPanel value={tabValue} index={1}>
        <Stack spacing={2}>
          <Paper variant="outlined" sx={{ p: 2 }}>
            <Stack direction="row" alignItems="center" spacing={2} mb={2}>
              <Typography variant="subtitle1">Active Tasks</Typography>
              <Chip label={`${activeTasks.length} active`} size="small" />
              <Button
                size="small"
                onClick={fetchAsyncDebuggingData}
                startIcon={<Memory />}
                disabled={!isRunning}
              >
                Refresh
              </Button>
            </Stack>
            <List dense sx={{ maxHeight: 300, overflow: 'auto', border: '1px solid', borderColor: 'divider' }}>
              {activeTasks.map((task) => (
                <ListItem key={task.id} disablePadding>
                  <ListItemButton onClick={() => inspectTask(task.id)}>
                    <ListItemIcon>
                      <TaskAlt sx={{ color: task.status === 'running' ? 'success.main' : 'warning.main' }} />
                    </ListItemIcon>
                    <ListItemText
                      primary={
                        <Stack direction="row" alignItems="center" spacing={1}>
                          <Typography variant="body2">{task.name}</Typography>
                          <Chip label={task.status} size="small" color={task.status === 'running' ? 'success' : 'warning'} />
                        </Stack>
                      }
                      secondary={task.spawnLocation}
                    />
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
            {asyncError && (
              <Typography color="error" variant="caption" sx={{ mt: 1 }}>
                {asyncError}
              </Typography>
            )}
          </Paper>
        </Stack>
      </TabPanel>

      {/* Futures Tab */}
      <TabPanel value={tabValue} index={2}>
        <Stack spacing={2}>
          <Paper variant="outlined" sx={{ p: 2 }}>
            <Stack direction="row" alignItems="center" spacing={2} mb={2}>
              <Typography variant="subtitle1">Futures & Streams</Typography>
              <Chip label={`${futures.length} active`} size="small" />
            </Stack>
            <List dense sx={{ maxHeight: 300, overflow: 'auto', border: '1px solid', borderColor: 'divider' }}>
              {futures.map((future) => (
                <ListItem key={future.id} disablePadding>
                  <ListItemButton onClick={() => inspectFuture(future.id)}>
                    <ListItemIcon>
                      <Future sx={{ color: future.state === 'ready' ? 'success.main' : 'info.main' }} />
                    </ListItemIcon>
                    <ListItemText
                      primary={
                        <Stack direction="row" alignItems="center" spacing={1}>
                          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                            {future.expression.slice(0, 50)}...
                          </Typography>
                          <Chip label={future.state} size="small" color={future.state === 'ready' ? 'success' : 'info'} />
                        </Stack>
                      }
                      secondary={`Future ${future.id}`}
                    />
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </Paper>
        </Stack>
      </TabPanel>

      {/* Deadlocks Tab */}
      <TabPanel value={tabValue} index={3}>
        <Stack spacing={2}>
          <Paper variant="outlined" sx={{ p: 2 }}>
            <Stack direction="row" alignItems="center" spacing={2} mb={2}>
              <Typography variant="subtitle1">Deadlock Detection</Typography>
              <Chip
                label={`${deadlocks.length} detected`}
                size="small"
                color={deadlocks.length > 0 ? 'error' : 'success'}
              />
            </Stack>
            {deadlocks.length === 0 ? (
              <Typography color="success.main" variant="body2">
                ✅ No deadlocks detected
              </Typography>
            ) : (
              <List dense sx={{ maxHeight: 300, overflow: 'auto', border: '1px solid', borderColor: 'divider' }}>
                {deadlocks.map((deadlock, index) => (
                  <ListItem key={index} disablePadding>
                    <ListItemIcon>
                      <Warning color="error" />
                    </ListItemIcon>
                    <ListItemText
                      primary={
                        <Stack direction="row" alignItems="center" spacing={1}>
                          <Typography variant="body2">{deadlock.description}</Typography>
                          <Chip
                            label={deadlock.severity}
                            size="small"
                            color={deadlock.severity === 'high' ? 'error' : deadlock.severity === 'medium' ? 'warning' : 'info'}
                          />
                        </Stack>
                      }
                      secondary={`Tasks: [${deadlock.tasks.join(', ')}]`}
                    />
                  </ListItem>
                ))}
              </List>
            )}
          </Paper>
        </Stack>
      </TabPanel>
    </Box>
  );
}
