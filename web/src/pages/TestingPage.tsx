import { Box, Button, Paper, List, ListItem, ListItemButton, ListItemText, Stack, Typography, TextField } from '@mui/material';
import * as React from 'react';
import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppSelector } from '../store/store';
import { selectCurrentProjectPath, selectCargoCommands } from '../store/slices/cargoSlice';

export default function TestingPage() {
  const projectPath = useAppSelector(selectCurrentProjectPath);
  const cargoCommands = useAppSelector(selectCargoCommands);
  const [tests, setTests] = useState<string[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>('');
  const [coverageAvailable, setCoverageAvailable] = useState<boolean>(false);
  const [coverageOutput, setCoverageOutput] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);

  const filteredTests = useMemo(() => {
    if (!filter.trim()) return tests;
    const q = filter.toLowerCase();
    return tests.filter(t => t.toLowerCase().includes(q));
  }, [tests, filter]);

  useEffect(() => {
    (async () => {
      try {
        const avail = await invoke<boolean>('coverage_is_available');
        setCoverageAvailable(!!avail);
      } catch {}
    })();
  }, []);

  useEffect(() => {
    if (!projectPath) return;
    (async () => {
      setLoading(true);
      try {
        const list = await invoke<string[]>('test_list', { projectPath: projectPath });
        setTests(list);
      } catch (e) {
        // ignore
      } finally {
        setLoading(false);
      }
    })();
  }, [projectPath]);

  const lastTestCommand = useMemo(() => {
    const entries = Object.values(cargoCommands);
    const sorted = entries
      .filter(c => c.args && c.args[0] === 'test')
      .sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0));
    return sorted[0];
  }, [cargoCommands]);

  const runTests = async (name?: string) => {
    if (!projectPath) return;
    try {
      await invoke('test_run_stream', {
        projectPath: projectPath,
        testFilter: name ?? null,
      });
    } catch (e) {
      // ignore
    }
  };

  const runCoverage = async () => {
    if (!projectPath) return;
    try {
      const out = await invoke<string>('coverage_run', { projectPath });
      setCoverageOutput(out);
    } catch (e: any) {
      setCoverageOutput(String(e));
    }
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
      <Typography variant="h5">Testing</Typography>
      <Stack direction="row" spacing={1} alignItems="center">
        <TextField 
          size="small" 
          label="Filter" 
          value={filter} 
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
            setFilter((e.target as any).value);
          }}
          
          sx={{ width: 240 }} 
        />
        <Button variant="contained" size="small" disabled={!projectPath} onClick={() => runTests()}>Run All</Button>
        <Button size="small" disabled={!selected || !projectPath} onClick={() => selected && runTests(selected)}>Run Selected</Button>
        <Button size="small" disabled={!projectPath} onClick={async () => {
          if (!projectPath) return;
          try {
            const list = await invoke<string[]>('test_list', { projectPath });
            setTests(list);
          } catch {}
        }}>Refresh</Button>
        <Box sx={{ flexGrow: 1 }} />
        {coverageAvailable && (
          <Button variant="outlined" size="small" onClick={runCoverage}>Run Coverage</Button>
        )}
      </Stack>

      <Box sx={{ display: 'flex', gap: 2 }}>
        <Paper sx={{ flex: 1, p: 1 }} variant="outlined">
          <Typography variant="subtitle1" sx={{ mb: 1 }}>Discovered Tests</Typography>
          <List dense sx={{ maxHeight: 400, overflow: 'auto' }}>
            {filteredTests.map((t) => (
              <ListItem key={t} disablePadding>
                <ListItemButton selected={selected === t} onClick={() => setSelected(t)}>
                  <ListItemText primary={t} />
                </ListItemButton>
              </ListItem>
            ))}
          </List>
        </Paper>

        <Paper sx={{ flex: 2, p: 1 }} variant="outlined">
          <Typography variant="subtitle1" sx={{ mb: 1 }}>Last Test Output</Typography>
          <Box sx={{ height: 400, overflow: 'auto', bgcolor: '#0b0b0b', color: '#e0e0e0', p: 1, fontFamily: 'monospace', fontSize: '0.8rem' }}>
            <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>{lastTestCommand?.output || 'No test output yet'}</pre>
          </Box>
        </Paper>
      </Box>

      {coverageAvailable && (
        <Paper sx={{ p: 1 }} variant="outlined">
          <Typography variant="subtitle1" sx={{ mb: 1 }}>Coverage Output</Typography>
          <Box sx={{ maxHeight: 240, overflow: 'auto', bgcolor: '#111', color: '#ddd', p: 1, fontFamily: 'monospace', fontSize: '0.8rem' }}>
            <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>{coverageOutput || 'No coverage run yet'}</pre>
          </Box>
        </Paper>
      )}
    </Box>
  );
}