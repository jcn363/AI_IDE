import React from 'react';
import {
  Box,
  Button,
  Chip,
  Divider,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  SelectChangeEvent,
  Stack,
  TextField,
  Typography,
  Paper,
  IconButton,
  Collapse,
  Link,
} from '@mui/material';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import { useAppDispatch, useAppSelector } from '../store';
import { editorActions } from '../store/slices/editorSlice';
import { executeCargoStream, selectCargoCommands, selectCurrentProjectPath, clearCommandOutput, cancelCargoCommand } from '../store/slices/cargoSlice';

function genId() {
  return 'cmd-' + Date.now().toString(36) + '-' + Math.random().toString(36).slice(2, 8);
}

// stats are now parsed in cargoSlice and attached to each command

export default function BuildPage() {
  const dispatch = useAppDispatch();
  const commands = useAppSelector(selectCargoCommands);
  const projectPath = useAppSelector(selectCurrentProjectPath);
  const [profile, setProfile] = React.useState<'debug' | 'release' | 'custom'>('debug');
  const [customProfile, setCustomProfile] = React.useState('');
  const [extraArgs, setExtraArgs] = React.useState('');
  const [expanded, setExpanded] = React.useState<Record<string, boolean>>({});

  const list = React.useMemo(() => {
    return Object.values(commands).sort((a, b) => b.timestamp - a.timestamp);
  }, [commands]);

  const startBuild = React.useCallback(() => {
    const id = genId();
    const args: string[] = [];
    if (profile === 'release') args.push('--release');
    if (profile === 'custom' && customProfile.trim()) {
      args.push('--profile', customProfile.trim());
    }
    if (extraArgs.trim()) {
      const parts = extraArgs.split(' ').filter(Boolean);
      args.push(...parts);
    }
    dispatch(
      executeCargoStream({ command: 'build', args, cwd: projectPath || '', commandId: id }) as any
    );
    setExpanded((e) => ({ ...e, [id]: true }));
  }, [dispatch, profile, customProfile, extraArgs, projectPath]);

  // Persist and restore build controls
  React.useEffect(() => {
    try {
      const p = (globalThis.localStorage?.getItem('build.profile') as any) || 'debug';
      if (p === 'debug' || p === 'release' || p === 'custom') setProfile(p);
      const cp = globalThis.localStorage?.getItem('build.customProfile') || '';
      setCustomProfile(cp || '');
      const ea = globalThis.localStorage?.getItem('build.extraArgs') || '';
      setExtraArgs(ea || '');
    } catch {}
  }, []);

  React.useEffect(() => {
    try {
      globalThis.localStorage?.setItem('build.profile', profile);
    } catch {}
  }, [profile]);

  React.useEffect(() => {
    try {
      globalThis.localStorage?.setItem('build.customProfile', customProfile);
    } catch {}
  }, [customProfile]);

  React.useEffect(() => {
    try {
      globalThis.localStorage?.setItem('build.extraArgs', extraArgs);
    } catch {}
  }, [extraArgs]);

  return (
    <Box sx={{ maxWidth: 1200 }}>
      <Typography variant="h5" gutterBottom>
        Cargo Build System
      </Typography>

      <Paper sx={{ p: 2, mb: 3 }}>
        <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} alignItems="center">
          <FormControl size="small" sx={{ minWidth: 160 }}>
            <InputLabel id="profile-label">Profile</InputLabel>
            <Select
              labelId="profile-label"
              value={profile}
              label="Profile"
              onChange={(e: SelectChangeEvent<'debug' | 'release' | 'custom'>) =>
                setProfile(e.target.value as any)
              }
            >
              <MenuItem value="debug">debug</MenuItem>
              <MenuItem value="release">release</MenuItem>
              <MenuItem value="custom">custom</MenuItem>
            </Select>
          </FormControl>
          {profile === 'custom' && (
            <TextField
              size="small"
              label="--profile"
              placeholder="custom-profile"
              value={customProfile}
              onChange={(e: any) => setCustomProfile(e.target?.value ?? '')}
            />
          )}
          <TextField
            size="small"
            fullWidth
            label="Extra args"
            placeholder="e.g. --features foo,bar -Z timings"
            value={extraArgs}
            onChange={(e: any) => setExtraArgs(e.target?.value ?? '')}
          />
          <Button variant="contained" startIcon={<PlayArrowIcon />} onClick={startBuild} disabled={!projectPath}>
            Build
          </Button>
          <Button variant="outlined" color="warning" onClick={() => dispatch(clearCommandOutput({ commandId: 'all' }))}>
            Clear All
          </Button>
        </Stack>
        {!projectPath && (
          <Typography variant="caption" color="warning.main" sx={{ mt: 1, display: 'block' }}>
            No project folder selected. Open a workspace to enable builds.
          </Typography>
        )}
      </Paper>

      <Typography variant="h6" gutterBottom>
        Recent Builds
      </Typography>

      <Stack spacing={2}>
        {list.map((c) => {
          const isExpanded = expanded[c.id] || false;
          const diag = c.diagnostics || [];
          const errorCount = diag.filter((d) => (d.level || '').toLowerCase() === 'error').length;
          const warnCount = diag.filter((d) => (d.level || '').toLowerCase() === 'warning' || (d.level || '').toLowerCase() === 'warn').length;
          return (
            <Paper key={c.id} sx={{ p: 2 }}>
              <Stack direction="row" spacing={2} alignItems="center" justifyContent="space-between">
                <Stack direction="row" spacing={2} alignItems="center">
                  <IconButton onClick={() => setExpanded((e) => ({ ...e, [c.id]: !isExpanded }))} size="small">
                    {isExpanded ? <ExpandLessIcon /> : <ExpandMoreIcon />}
                  </IconButton>
                  <Typography variant="subtitle1">{c.command} {c.args.join(' ')}</Typography>
                  <Chip size="small" label={c.status} color={c.status === 'running' ? 'info' : c.status === 'success' ? 'success' : c.status === 'error' ? 'error' : 'default'} />
                  <Chip size="small" label={`compiled: ${c.stats?.compiling ?? 0}`} />
                  <Chip size="small" label={`fresh: ${c.stats?.fresh ?? 0}`} />
                  <Chip size="small" label={`finished: ${c.stats?.finished ?? 0}`} />
                  {warnCount > 0 && <Chip size="small" color="warning" label={`warnings: ${warnCount}`} />}
                  {errorCount > 0 && <Chip size="small" color="error" label={`errors: ${errorCount}`} />}
                </Stack>
                <Stack direction="row" spacing={1}>
                  <IconButton
                    size="small"
                    onClick={() => dispatch(cancelCargoCommand({ id: c.id }) as any)}
                    disabled={c.status !== 'running'}
                    color="warning"
                  >
                    <StopIcon />
                  </IconButton>
                </Stack>
              </Stack>
              <Collapse in={isExpanded}>
                <Divider sx={{ my: 1 }} />
                <Typography variant="subtitle2" gutterBottom>Output</Typography>
                <Box component="pre" sx={{ bgcolor: 'background.default', p: 1.5, borderRadius: 1, maxHeight: 220, overflow: 'auto', whiteSpace: 'pre-wrap' }}>
                  {c.output || 'No output'}
                </Box>
                {c.diagnostics && c.diagnostics.length > 0 && (
                  <>
                    <Typography variant="subtitle2" sx={{ mt: 1 }} gutterBottom>Diagnostics</Typography>
                    <Box component="ul" sx={{ m: 0, pl: 3 }}>
                      {c.diagnostics.map((d, i) => {
                        const first = d.spans && d.spans[0];
                        const hasSpan = Boolean(first && first.file_name && first.line_start);
                        const handleOpen = () => {
                          if (!hasSpan) return;
                          dispatch(
                            editorActions.setNavigationTarget({
                              filePath: first!.file_name,
                              line: first!.line_start,
                              column: first!.column_start || 1,
                            })
                          );
                        };
                        return (
                          <li key={i}>
                            <Typography variant="body2">
                              [{d.level}] {d.message}{' '}
                              {hasSpan && (
                                <Link component="button" type="button" onClick={handleOpen} sx={{ ml: 1 }}>
                                  Open
                                </Link>
                              )}
                            </Typography>
                          </li>
                        );
                      })}
                    </Box>
                  </>
                )}
              </Collapse>
            </Paper>
          );
        })}
      </Stack>
    </Box>
  );
}
