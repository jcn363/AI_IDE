/**
 * Component for parsing and displaying Rust Cargo.lock files
 * Provides structured view of pinned dependency versions
 */

import React, { useState, useEffect } from 'react';
import { Box, Button, Paper, Typography, CircularProgress } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';

interface LockfileViewerProps {
  projectPath: string | null;
  onError: (error: string) => void;
}

/**
 * Displays the contents of the Cargo.lock file in a structured format
 */
export const LockfileViewer: React.FC<LockfileViewerProps> = ({
  projectPath,
  onError
}) => {
  const [lockfileData, setLockfileData] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const loadLockfile = async () => {
    if (!projectPath) {
      onError('No project selected');
      return;
    }

    setLoading(true);
    try {
      const lockfilePath = `${projectPath}/Cargo.lock`;
      const result = await invoke<string>('read_file', { path: lockfilePath });

      if (result) {
        // Simple parsing of Cargo.lock file
        const packages: any[] = [];
        let currentPkg: any = {};

        result.split('\n').forEach((line: string) => {
          line = line.trim();
          if (line.startsWith('[[')) {
            if (currentPkg.name) packages.push(currentPkg);
            currentPkg = {};
          } else if (line.startsWith('name = ')) {
            currentPkg.name = line.split('=')[1].trim().replace(/["']/g, '');
          } else if (line.startsWith('version = ')) {
            currentPkg.version = line.split('=')[1].trim().replace(/["']/g, '');
          } else if (line.startsWith('dependencies = [')) {
            currentPkg.dependencies = [];
          } else if (line.match(/^\s*"[^"]+"/)) {
            const dep = line.trim().replace(/[,"]/g, '');
            if (currentPkg.dependencies) {
              currentPkg.dependencies.push(dep);
            }
          }
        });

        if (currentPkg.name) packages.push(currentPkg);
        setLockfileData({ packages });
      } else {
        onError('Failed to read lockfile');
      }
    } catch (error) {
      console.error('Error loading lockfile:', error);
      onError(`Failed to load lockfile: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const clearLockfile = () => {
    setLockfileData(null);
  };

  useEffect(() => {
    if (projectPath && lockfileData === null && !loading) {
      loadLockfile();
    }
  }, [projectPath]);

  if (loading) {
    return <CircularProgress />;
  }

  return (
    <Box className="lockfile-viewer">
      <Box sx={{ mb: 2, display: 'flex', gap: 1 }}>
        <Button variant="contained" onClick={loadLockfile} disabled={!projectPath || loading}>
          {loading ? 'Loading...' : 'Load Cargo.lock'}
        </Button>
        <Button variant="outlined" onClick={clearLockfile} disabled={!lockfileData}>
          Clear
        </Button>
      </Box>

      {lockfileData ? (
        <Paper sx={{ p: 2, maxHeight: 360, overflow: 'auto' }}>
          <pre style={{ margin: 0 }}>{JSON.stringify(lockfileData, null, 2)}</pre>
        </Paper>
      ) : (
        <Typography variant="body2">Lockfile not loaded.</Typography>
      )}
    </Box>
  );
};

export default LockfileViewer;