import React from 'react';
import { Snackbar, Alert, Button, Dialog, DialogTitle, DialogContent, DialogActions, Typography, Box, Chip, Link } from '@mui/material';
import { useAppDispatch, useAppSelector } from '../../store';
import { selectCargoState } from '../../store/slices/cargoSlice';
import { editorActions } from '../../store/slices/editorSlice';

interface Notice {
  id: string;
  severity: 'success' | 'error';
  message: string;
}

export default function CargoNotifier() {
  const dispatch = useAppDispatch();
  const cargo = useAppSelector(selectCargoState);
  const announcedRef = React.useRef<Set<string>>(new Set());
  const [queue, setQueue] = React.useState<Notice[]>([]);
  const [open, setOpen] = React.useState(false);
  const [current, setCurrent] = React.useState<Notice | null>(null);
  const [detailsOpen, setDetailsOpen] = React.useState(false);
  const [copiedOpen, setCopiedOpen] = React.useState(false);

  // Settings state + listeners so updates apply immediately
  const readSettings = React.useCallback(() => {
    const enabled = localStorage.getItem('notifications.enabled');
    const duration = localStorage.getItem('notifications.duration');
    const showDiag = localStorage.getItem('notifications.showDiagCount');
    return {
      enabled: enabled === null ? true : enabled === 'true',
      duration: duration ? Math.max(1000, parseInt(duration, 10) || 4000) : 4000,
      showDiagCount: showDiag === null ? true : showDiag === 'true',
    };
  }, []);

  const [settings, setSettings] = React.useState(() => readSettings());

  React.useEffect(() => {
    const w = globalThis as any;
    const onStorage = (e: any) => {
      if (!e || !e.key || String(e.key).startsWith('notifications.')) {
        setSettings(readSettings());
      }
    };
    const onCustom = () => setSettings(readSettings());
    w.addEventListener?.('storage', onStorage as any);
    w.addEventListener?.('notifications:settings-changed', onCustom as any);
    return () => {
      w.removeEventListener?.('storage', onStorage as any);
      w.removeEventListener?.('notifications:settings-changed', onCustom as any);
    };
  }, [readSettings]);

  // Watch for newly finished cargo commands and enqueue notices
  React.useEffect(() => {
    if (!settings.enabled) return;
    const cmds = Object.values(cargo.commands || {});
    for (const c of cmds) {
      if ((c.status === 'success' || c.status === 'error') && !announcedRef.current.has(c.id)) {
        announcedRef.current.add(c.id);
        const diagCount = settings.showDiagCount && c.diagnostics?.length ? ` • diagnostics: ${c.diagnostics.length}` : '';
        const args = c.args?.length ? ' ' + c.args.join(' ') : '';
        const message = `${c.command}${args} ${c.status}${diagCount}`;
        setQueue((q) => [...q, { id: c.id, severity: c.status === 'success' ? 'success' : 'error', message }]);
      }
    }
  }, [cargo.commands, settings.enabled, settings.showDiagCount]);

  // Dequeue logic
  React.useEffect(() => {
    if (!open && queue.length > 0) {
      setCurrent(queue[0]);
      setQueue((q) => q.slice(1));
      setOpen(true);
    }
  }, [queue, open]);

  const handleClose = (_e?: unknown, reason?: string) => {
    if (reason === 'clickaway') return;
    setOpen(false);
  };

  const handleExited = () => {
    setCurrent(null);
  };

  const handleShowDetails = () => {
    setDetailsOpen(true);
  };

  const handleCloseDetails = () => {
    setDetailsOpen(false);
  };

  const currentCommand = React.useMemo(() => {
    if (!current) return null;
    const cmd = cargo.commands[current.id];
    return cmd || null;
  }, [current, cargo.commands]);

  return (
    <>
      <Snackbar
        open={open}
        autoHideDuration={settings.duration}
        onClose={handleClose}
        TransitionProps={{ onExited: handleExited }}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={handleClose}
          severity={current?.severity || 'success'}
          variant="filled"
          sx={{ width: '100%' }}
          action={
            <Button color="inherit" size="small" onClick={handleShowDetails}>
              Details
            </Button>
          }
        >
          {current?.message}
        </Alert>
      </Snackbar>

      <Dialog open={detailsOpen} onClose={handleCloseDetails} maxWidth="md" fullWidth>
        <DialogTitle>
          {currentCommand ? `${currentCommand.command} ${currentCommand.args?.join(' ') || ''}`.trim() : 'Command details'}
        </DialogTitle>
        <DialogContent dividers>
          {currentCommand && (
            <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap' }}>
              <Chip size="small" label={`Status: ${currentCommand.status}`} color={currentCommand.status === 'success' ? 'success' : currentCommand.status === 'error' ? 'error' : 'default'} />
              <Chip size="small" label={`CWD: ${currentCommand.cwd || '-'}`} variant="outlined" />
              <Chip size="small" label={`Diagnostics: ${currentCommand.diagnostics?.length || 0}`} variant="outlined" />
            </Box>
          )}
          <Typography variant="subtitle2" gutterBottom>
            Output
          </Typography>
          <Box component="pre" sx={{ bgcolor: 'background.default', color: 'text.primary', p: 1.5, borderRadius: 1, maxHeight: 360, overflow: 'auto', whiteSpace: 'pre-wrap' }}>
            {currentCommand?.output || 'No output'}
          </Box>
          {currentCommand?.diagnostics && currentCommand.diagnostics.length > 0 && (
            <>
              <Typography variant="subtitle2" sx={{ mt: 2 }} gutterBottom>
                Diagnostics
              </Typography>
              <Box component="ul" sx={{ m: 0, pl: 3 }}>
                {currentCommand.diagnostics.map((d, i) => {
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
                    setDetailsOpen(false);
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
        </DialogContent>
        <DialogActions>
          <Button
            onClick={async () => {
              try {
                const w = globalThis as any;
                await w.navigator?.clipboard?.writeText(currentCommand?.output || '');
                setCopiedOpen(true);
              } catch {
                // ignore clipboard errors
              }
            }}
          >
            Copy output
          </Button>
          <Button
            onClick={async () => {
              try {
                const w = globalThis as any;
                const json = currentCommand ? JSON.stringify({
                  id: currentCommand.id,
                  command: currentCommand.command,
                  args: currentCommand.args,
                  cwd: currentCommand.cwd,
                  status: currentCommand.status,
                  output: currentCommand.output,
                  diagnostics: currentCommand.diagnostics,
                  timestamp: currentCommand.timestamp,
                }, null, 2) : '{}';
                await w.navigator?.clipboard?.writeText(json);
                setCopiedOpen(true);
              } catch {
                // ignore
              }
            }}
          >
            Copy details (JSON)
          </Button>
          <Button onClick={handleCloseDetails}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Copy confirmation */}
      <Snackbar
        open={copiedOpen}
        autoHideDuration={2000}
        onClose={() => setCopiedOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
      >
        <Alert severity="success" variant="filled" onClose={() => setCopiedOpen(false)}>
          Copied output to clipboard
        </Alert>
      </Snackbar>
    </>
  );
}
