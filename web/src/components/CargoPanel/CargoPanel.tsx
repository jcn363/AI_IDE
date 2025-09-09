import React, { useEffect, useMemo, useState } from 'react';
import { Box, IconButton, Alert, Tabs, Tab, Dialog, DialogTitle, DialogContent, DialogActions, Button, Typography, TextField, List, ListItem, ListItemText, Paper, FormControlLabel, Switch, CircularProgress, SelectChangeEvent, } from '@mui/material';
import { Speed as SpeedIcon, Close as CloseIcon, Add as AddIcon } from '@mui/icons-material';

import { useSelector } from 'react-redux';
import { invoke } from '@tauri-apps/api/core';
import { useAppDispatch } from '../../store/store';
import type { RootState } from '../../store/types';
import PerformancePanel from '../PerformancePanel/PerformancePanel';
import {
  executeCargoCommand,
  executeCargoStream,
  clearCommandOutput,
  selectCargoState,
  selectCurrentProjectPath,
  selectIsCargoAvailable,
  selectCargoCommands,
  clearError,
  type CargoCommandName,
  CargoCommand
} from '../../store/slices/cargoSlice';

// Import new modular components
import CargoCommandsPanel from './CargoCommandsPanel';
import DependencyManager from './DependencyManager';
import GraphViewer from './GraphViewer';
import LockfileViewer from './LockfileViewer';
import CommandOutput from './CommandOutput';

// Import shared enhanced BasePanel
import { BasePanel } from '../shared/BasePanel';
import { SharedTabPanel, SharedTabsHelper } from '../common/SharedTabPanel';
import { createErrorHandler } from '../../utils/consolidated';

// Consolidated error handler
const errorHandler = createErrorHandler('CargoPanel');

const CargoPanel: React.FC = () => {
  const [selectedCommand, setSelectedCommand] = useState<CargoCommandName>('build');
  const [commandArgs, setCommandArgs] = useState<string>('');
  const [output, setOutput] = useState<string>('');
  const [localError, setLocalError] = useState<string>('');
  const [activeTab, setActiveTab] = useState<number>(0);
  const [showPerformancePanel, setShowPerformancePanel] = useState<boolean>(false);
  const [useJsonDiagnostics, setUseJsonDiagnostics] = useState<boolean>(false);
  const [isAddDependencyDialogOpen, setIsAddDependencyDialogOpen] = useState<boolean>(false);
  const [newDependency, setNewDependency] = useState<{
    name: string;
    version: string;
    features: string;
  }>({
    name: '',
    version: '',
    features: '',
  });

  // Dependency management state
  const dispatch = useAppDispatch();
  const commands = useSelector((state: RootState) => selectCargoCommands(state));
  const currentProjectPath = useSelector((state: RootState) => selectCurrentProjectPath(state));
  const manifestPath = useMemo(() => (currentProjectPath ? `${currentProjectPath}/Cargo.toml` : null), [currentProjectPath]);

  const [depTab, setDepTab] = useState<number>(0); // 0=Graph,1=Lock,2=Features,3=Conflicts,4=Update
  const [fullMetadata, setFullMetadata] = useState<any | null>(null);
  const [lockfile, setLockfile] = useState<any | null>(null);
  const [featuresMap, setFeaturesMap] = useState<Record<string, string[]> | null>(null);
  const [conflicts, setConflicts] = useState<Array<{ name: string; versions: string[] }>>([]);

  const loadConflicts = async () => {
    if (!currentProjectPath) return;
    
    setDepLoading(true);
    try {
      const result = await invoke<string>('execute_command', {
        command: 'tree',
        args: '--duplicates',
        cwd: currentProjectPath,
      });

      if (result) {
        const conflictLines = result.split('\n').filter(line => line.includes('(*)'));
        const conflictMap = new Map<string, Set<string>>();

        conflictLines.forEach(line => {
          const match = line.match(/^([^\s]+) v([\d.]+)/);
          if (match) {
            const [, name, version] = match;
            if (!conflictMap.has(name)) {
              conflictMap.set(name, new Set());
            }
            conflictMap.get(name)?.add(version);
          }
        });

        const conflicts = Array.from(conflictMap.entries())
          .filter(([_, versions]) => versions.size > 1)
          .map(([name, versions]) => ({
            name,
            versions: Array.from(versions)
          }));

        setConflicts(conflicts);
      }
    } catch (error) {
      console.error('Error loading conflicts:', error);
      setDepError(`Failed to load conflicts: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setDepLoading(false);
    }
  };
  const updateDependencies = async (packageName?: string) => {
    if (!currentProjectPath) return;
    
    setDepLoading(true);
    try {
      const args = packageName ? ['-p', packageName] : [];
      await invoke('execute_command', {
        command: 'update',
        args: args.join(' '),
        cwd: currentProjectPath,
      });
      
      // Refresh the UI after update
      if (depTab === 3) {
        loadConflicts();
      }
    } catch (error) {
      console.error('Error updating dependencies:', error);
      setDepError(`Failed to update ${packageName || 'dependencies'}: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setDepLoading(false);
    }
  };

  const [depLoading, setDepLoading] = useState<boolean>(false);
  const [depError, setDepError] = useState<string>('');
  const [featureEdit, setFeatureEdit] = useState<{ depName: string; features: string; defaultFeatures: boolean }>({ depName: '', features: '', defaultFeatures: true });
  const isCargoAvailable = useSelector((state: RootState) => selectIsCargoAvailable(state));
  const { isLoading, error: cargoError } = useSelector((state: RootState) => selectCargoState(state));
  const isRunning = isLoading;
  const error = cargoError || localError;

  const handleCommandChange = (event: SelectChangeEvent<string>) => {
    const value = event.target.value as CargoCommandName;
    setSelectedCommand(value);
  };

  const handleArgsChange: React.ChangeEventHandler<HTMLInputElement> = (event) => {
    const target = event.target as unknown as { value: string };
    setCommandArgs(target.value);
  };

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const handleDepTabChange = (_: React.SyntheticEvent, v: number) => setDepTab(v);

  const loadLockfile = async () => {
    if (!currentProjectPath) {
      setDepError('No project selected');
      return;
    }

    setDepLoading(true);
    setDepError('');

    try {
      const lockfilePath = `${currentProjectPath}/Cargo.lock`;
      const result = await invoke('read_file', { path: lockfilePath });
      
      if (result && typeof result === 'string') {
        // Simple parsing of Cargo.lock file
        const packages: any[] = [];
        let currentPkg: any = {};
        
        result.split('\n').forEach(line => {
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
        setLockfile({ packages });
      } else {
        setDepError('Failed to read lockfile');
      }
    } catch (error) {
      console.error('Error loading lockfile:', error);
      setDepError(`Failed to load lockfile: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setDepLoading(false);
    }
  };


  const loadFullMetadata = async () => {
    if (!currentProjectPath) {
      setDepError('No project selected');
      return;
    }

    setDepLoading(true);
    setDepError('');

    try {
      const result = await invoke('run_cargo_command', {
        command: 'metadata',
        args: '--format-version=1 --no-deps',
        cwd: currentProjectPath,
      });
      
      if (result && typeof result === 'string') {
        const metadata = JSON.parse(result);
        setFullMetadata(metadata);
      } else {
        setDepError('Failed to parse metadata');
      }
    } catch (error) {
      console.error('Error loading metadata:', error);
      setDepError(`Failed to load metadata: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setDepLoading(false);
    }
  };

  const applyDependencyFeatures = async () => {
    if (!currentProjectPath || !featureEdit.depName) {
      setDepError('No project or dependency selected');
      return;
    }

    setDepLoading(true);
    setDepError('');

    try {
      // Build the features string
      const features = featureEdit.features
        .split(',')
        .map(f => f.trim())
        .filter(f => f.length > 0);

      // Update Cargo.toml using the Tauri backend
      await invoke('update_dependency_features', {
        manifestPath: `${currentProjectPath}/Cargo.toml`,
        dependencyName: featureEdit.depName,
        features,
        defaultFeatures: featureEdit.defaultFeatures
      });

      // Reload features after update
      await loadFeatures();
      
      // Reset the form
      setFeatureEdit({ depName: '', features: '', defaultFeatures: true });
    } catch (error) {
      console.error('Error updating features:', error);
      setDepError(`Failed to update features: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setDepLoading(false);
    }
  };

  const loadFeatures = async () => {
    if (!currentProjectPath) {
      setDepError('No project selected');
      return;
    }

    setDepLoading(true);
    setDepError('');

    try {
      // First try to get features using cargo metadata
      const result = await invoke('run_cargo_command', {
        command: 'metadata',
        args: '--format-version=1 --no-deps',
        cwd: currentProjectPath,
      });
      
      if (result && typeof result === 'string') {
        const metadata = JSON.parse(result);
        const featuresMap: Record<string, string[]> = {};
        
        // Extract features from workspace members
        if (metadata.workspace_members) {
          for (const pkgId of metadata.workspace_members) {
            const pkg = metadata.packages.find((p: any) => p.id === pkgId);
            if (pkg && pkg.features) {
              featuresMap[pkg.name] = Object.keys(pkg.features);
            }
          }
        }
        
        // If no workspace members, check root package
        if (Object.keys(featuresMap).length === 0 && metadata.packages) {
          const rootPkg = metadata.packages.find((p: any) => !p.id.includes(' '));
          if (rootPkg?.features) {
            featuresMap[rootPkg.name] = Object.keys(rootPkg.features);
          }
        }
        
        setFeaturesMap(featuresMap);
      } else {
        setDepError('Failed to parse metadata');
      }
    } catch (error) {
      console.error('Error loading features:', error);
      setDepError(`Failed to load features: ${error instanceof Error ? error.message : String(error)}`);
      setFeaturesMap(null);
    } finally {
      setDepLoading(false);
    }
  };

  const handleRunCommand = async () => {
    if (!currentProjectPath) {
      setLocalError('No project selected');
      return;
    }

    const args = commandArgs ? commandArgs.split(' ').filter(arg => arg.trim() !== '') : [];
    const jsonCapable = ['build','check','test','clippy','run'];
    if (useJsonDiagnostics && jsonCapable.includes(selectedCommand)) {
      if (!args.includes('--message-format=json')) args.unshift('--message-format=json');
    }

    try {
      setLocalError('');
      dispatch(clearError());

      // Execute streaming command (returns immediately; output flows via events)
      await dispatch(
        executeCargoStream({
          command: selectedCommand,
          args,
          cwd: currentProjectPath,
        })
      );

      // Switch to the output tab to show the command execution
      setActiveTab(2);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to execute command';
      setLocalError(errorMessage);
    }
  };

  const handleClearOutput = () => {
    // Clear all command history
    dispatch(clearCommandOutput({ commandId: 'all' }));
  };

  const handleAddDependency = async () => {
    if (!newDependency.name) {
      setLocalError('Dependency name is required');
      return;
    }

    if (!currentProjectPath) {
      setLocalError('No project selected');
      return;
    }

    try {
      const command = 'add';
      const args = [newDependency.name];

      if (newDependency.version) {
        args.push('--version', newDependency.version);
      }

      if (newDependency.features) {
        args.push('--features', newDependency.features);
      }

      const result = await dispatch(
        executeCargoCommand({
          command: command as CargoCommandName,
          args,
          cwd: currentProjectPath,
        })
      ) as unknown as { type: string; error?: { message?: string } };

      // Check if the action was rejected by checking the action type
      if (result && typeof result === 'object' && 'type' in result &&
        typeof result.type === 'string' && result.type.endsWith('/rejected')) {
        throw new Error(result.error?.message || 'Failed to add dependency');
      }

      // Close the dialog and reset the form
      setIsAddDependencyDialogOpen(false);
      setNewDependency({
        name: '',
        version: '',
        features: ''
      });

      // Show success message or update UI as needed
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to add dependency';
      setLocalError(errorMessage);
    }
  };

  const handleShowPerformancePanel = () => {
    setShowPerformancePanel(true);
  };

  const handleClosePerformancePanel = () => {
    setShowPerformancePanel(false);
  };

  const [projectPath, setProjectPath] = useState('');

  useEffect(() => {
    const getProjectPath = async () => {
      try {
        const path = await invoke('get_project_path');
        if (path && typeof path === 'string') {
          setProjectPath(path);
        }
      } catch (error) {
        console.error('Failed to get project path:', error);
      }
    };
    
    getProjectPath();
  }, []);

  return (
    <BasePanel title="Cargo">
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
        <Tabs value={activeTab} onChange={handleTabChange} aria-label="cargo tabs">
          <Tab label="Commands" {...SharedTabsHelper.getA11yProps(0)} />
          <Tab label="Dependencies" {...SharedTabsHelper.getA11yProps(1)} />
          <Tab label="Output" {...SharedTabsHelper.getA11yProps(2)} />
        </Tabs>
      </Box>

      <SharedTabPanel value={activeTab} index={0}>
        <CargoCommandsPanel
          selectedCommand={selectedCommand}
          commandArgs={commandArgs}
          showJsonDiagnostics={useJsonDiagnostics}
          onCommandChange={handleCommandChange}
          onArgsChange={handleArgsChange}
          onRunCommand={handleRunCommand}
          onJsonDiagnosticsChange={setUseJsonDiagnostics}
          isRunning={isRunning}
          isCargoAvailable={isCargoAvailable}
          error={error}
          onClearError={() => {
            setLocalError('');
            dispatch(clearError());
          }}
        />
      </SharedTabPanel>

      <SharedTabPanel value={activeTab} index={1}>
        <Box sx={{ mb: 2, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Button
            variant="outlined"
            startIcon={<SpeedIcon />}
            onClick={handleShowPerformancePanel}
            sx={{ mr: 1 }}
          >
            Performance Analysis
          </Button>
          <Tabs value={depTab} onChange={handleDepTabChange} aria-label="dependency tabs">
            <Tab label="Graph" />
            <Tab label="Cargo.lock" />
            <Tab label="Features" />
            <Tab label="Conflicts" />
            <Tab label="Update" />
          </Tabs>
          <Box>
            <Button
              variant="outlined"
              startIcon={<AddIcon />}
              onClick={() => setIsAddDependencyDialogOpen(true)}
              disabled={!currentProjectPath}
              sx={{ ml: 2 }}
            >
              Add Dependency
            </Button>
          </Box>
        </Box>

        {depError && (
          <Alert severity="error" sx={{ mb: 2 }} onClose={() => setDepError('')}>
            {depError}
          </Alert>
        )}

        {/* Graph */}
        {depTab === 0 && (
          <Box>
            <Box sx={{ mb: 2 }}>
              <Button variant="contained" onClick={loadFullMetadata} disabled={!currentProjectPath || depLoading}>
                {depLoading ? 'Loading...' : 'Reload Graph'}
              </Button>
            </Box>
            {fullMetadata ? (
              <List dense>
                {(fullMetadata.packages || []).map((p: any) => (
                  <ListItem key={`${p.name}@${p.version}`} alignItems="flex-start">
                    <ListItemText
                      primary={`${p.name} @ ${p.version}`}
                      secondary={(p.dependencies || []).map((d: any) => d.name).join(', ') || 'No deps'}
                    />
                  </ListItem>
                ))}
              </List>
            ) : (
              <Typography variant="body2">No metadata loaded.</Typography>
            )}
          </Box>
        )}

        {/* Cargo.lock */}
        {depTab === 1 && (
          <Box>
            <Box sx={{ mb: 2, display: 'flex', gap: 1 }}>
              <Button variant="contained" onClick={loadLockfile} disabled={!currentProjectPath || depLoading}>
                {depLoading ? 'Loading...' : 'Load Cargo.lock'}
              </Button>
              <Button variant="outlined" onClick={() => lockfile && setLockfile(null)} disabled={!lockfile}>Clear</Button>
            </Box>
            {lockfile ? (
              <Paper sx={{ p: 2, maxHeight: 360, overflow: 'auto' }}>
                <pre style={{ margin: 0 }}>{JSON.stringify(lockfile, null, 2)}</pre>
              </Paper>
            ) : (
              <Typography variant="body2">Lockfile not loaded.</Typography>
            )}
          </Box>
        )}

        {/* Features */}
        {depTab === 2 && (
          <Box>
            <Box sx={{ mb: 2, display: 'flex', gap: 1, alignItems: 'center', flexWrap: 'wrap' }}>
              <Button variant="contained" onClick={loadFeatures} disabled={!manifestPath || depLoading}>
                {depLoading ? 'Loading...' : 'Load Features'}
              </Button>
              <TextField
                label="Dependency name"
                size="small"
                value={featureEdit.depName}
                onChange={(e) => setFeatureEdit({ ...featureEdit, depName: (e.target as any).value })}
              />
              <TextField
                label="Features (comma-separated)"
                size="small"
                value={featureEdit.features}
                onChange={(e) => setFeatureEdit({ ...featureEdit, features: (e.target as any).value })}
              />
              <FormControlLabel
                control={<Switch size="small" checked={featureEdit.defaultFeatures} onChange={(_, v) => setFeatureEdit({ ...featureEdit, defaultFeatures: v })} />}
                label={<Typography variant="caption">default-features</Typography>}
              />
              <Button variant="outlined" onClick={applyDependencyFeatures} disabled={!featureEdit.depName || depLoading}>Apply</Button>
            </Box>
            {featuresMap ? (
              <List dense>
                {Object.entries(featuresMap).map(([k, v]) => (
                  <ListItem key={k}>
                    <ListItemText primary={k} secondary={(v || []).join(', ')} />
                  </ListItem>
                ))}
              </List>
            ) : (
              <Typography variant="body2">No features loaded.</Typography>
            )}
          </Box>
        )}

        {/* Conflicts */}
        {depTab === 3 && (
          <Box>
            <Box sx={{ mb: 2, display: 'flex', gap: 1 }}>
              <Button variant="contained" onClick={loadConflicts} disabled={!currentProjectPath || depLoading}>
                {depLoading ? 'Scanning...' : 'Scan Conflicts'}
              </Button>
            </Box>
            {conflicts.length > 0 ? (
              <List dense>
                {conflicts.map((c) => (
                  <ListItem key={c.name} secondaryAction={
                    <Button size="small" variant="outlined" onClick={() => updateDependencies(c.name)}>Update -p {c.name}</Button>
                  }>
                    <ListItemText primary={c.name} secondary={`versions: ${c.versions.join(', ')}`} />
                  </ListItem>
                ))}
              </List>
            ) : (
              <Typography variant="body2">No conflicts found or not scanned.</Typography>
            )}
          </Box>
        )}

        {/* Update */}
        {depTab === 4 && (
          <Box>
            <Box sx={{ display: 'flex', gap: 1, mb: 2, alignItems: 'center' }}>
              <Button variant="contained" onClick={() => updateDependencies(undefined)} disabled={!currentProjectPath || depLoading}>
                {depLoading ? 'Updating...' : 'Update All'}
              </Button>
              <TextField size="small" label="Package (optional)" onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const val = (e.target as any).value?.trim();
                  updateDependencies(val || undefined);
                }
              }} />
            </Box>
            <Typography variant="body2">Run cargo update for all deps or a specific package using -p.</Typography>
          </Box>
        )}
      </SharedTabPanel>

      <SharedTabPanel value={activeTab} index={2}>
        <CommandOutput
          activeTab={activeTab}
          commands={commands}
          handleClearOutput={handleClearOutput}
          parseJson={useJsonDiagnostics}
        />
      </SharedTabPanel>

      {/* Add Dependency Dialog */}
      <Dialog open={isAddDependencyDialogOpen} onClose={() => setIsAddDependencyDialogOpen(false)}>
        <DialogTitle>Add Dependency</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, minWidth: '400px', pt: 2 }}>
            <TextField
              label="Crate name"
              value={newDependency.name}
              onChange={(e) => {
                const target = e.target as unknown as { value: string };
                setNewDependency({ ...newDependency, name: target.value });
              }}
              fullWidth
              size="small"
            />
            <TextField
              label="Version (optional)"
              value={newDependency.version}
              onChange={(e) => {
                const target = e.target as unknown as { value: string };
                setNewDependency({ ...newDependency, version: target.value });
              }}
              placeholder="1.0.0"
              fullWidth
              size="small"
            />
            <TextField
              label="Features (comma-separated, optional)"
              value={newDependency.features}
              onChange={(e) => {
                const target = e.target as unknown as { value: string };
                setNewDependency({ ...newDependency, features: target.value });
              }}
              placeholder="derive,rc"
              fullWidth
              size="small"
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsAddDependencyDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleAddDependency}
            variant="contained"
            disabled={!newDependency.name}
          >
            Add
          </Button>
        </DialogActions>
      </Dialog>

      {/* Performance Panel Dialog */}
      <Dialog
        open={showPerformancePanel}
        onClose={handleClosePerformancePanel}
        maxWidth="lg"
        fullWidth
        fullScreen
        sx={{ '& .MuiDialog-paper': { height: '90vh' } }}
      >
        <DialogTitle>
          <Box display="flex" justifyContent="space-between" alignItems="center">
            <Typography variant="h6">Performance Analysis</Typography>
            <IconButton onClick={handleClosePerformancePanel} size="small">
              <CloseIcon />
            </IconButton>
          </Box>
        </DialogTitle>
        <DialogContent dividers>
          {currentProjectPath ? (
            <PerformancePanel projectPath={currentProjectPath} />
          ) : (
            <Box display="flex" justifyContent="center" alignItems="center" height="300px">
              <CircularProgress />
            </Box>
          )}
        </DialogContent>
      </Dialog>
    </BasePanel>
  );
};

export default CargoPanel;
