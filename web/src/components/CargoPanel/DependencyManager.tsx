import React, { useState } from 'react';
import {
  Box,
  Button,
  TextField,
  FormControlLabel,
  Switch,
  Typography,
  Alert,
  Tabs,
  Tab,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  Paper,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { Add as AddIcon } from '@mui/icons-material';

interface DependencyManagerProps {
  features: Record<string, string[]> | null;
  conflicts: Array<{ name: string; versions: string[] }>;
  loading: boolean;
  error: string | null;
  currentProjectPath: string | null;
  featureEdit: { depName: string; features: string; defaultFeatures: boolean };
  newDependency: { name: string; version: string; features: string };
  addDialogOpen: boolean;
  onAddDialogClose: () => void;
  onLoadFeatures: () => void;
  onLoadConflicts: () => void;
  onApplyFeatures: () => void;
  onUpdateDependencies: (packageName?: string) => void;
  onAddDependency: () => void;
  onFeatureEditChange: (edit: {
    depName: string;
    features: string;
    defaultFeatures: boolean;
  }) => void;
  onNewDependencyChange: (dep: { name: string; version: string; features: string }) => void;
}

interface PackageConflict {
  name: string;
  versions: string[];
}

export const DependencyManager: React.FC<DependencyManagerProps> = ({
  features,
  conflicts,
  loading,
  error,
  currentProjectPath,
  featureEdit,
  newDependency,
  addDialogOpen,
  onAddDialogClose,
  onLoadFeatures,
  onLoadConflicts,
  onApplyFeatures,
  onUpdateDependencies,
  onAddDependency,
  onFeatureEditChange,
  onNewDependencyChange,
}) => {
  const [depTab, setDepTab] = useState<number>(0);

  const handleDepTabChange = (_: React.SyntheticEvent, newValue: number) => {
    setDepTab(newValue);
  };

  return (
    <Box>
      <Box sx={{ mb: 2, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Typography variant="h6">Package Management</Typography>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Button
            variant="outlined"
            startIcon={<AddIcon />}
            onClick={() => onAddDialogClose()} // Reusing as toggle
            disabled={!currentProjectPath}
          >
            Add Dependency
          </Button>
          <Tabs value={depTab} onChange={handleDepTabChange} aria-label="dependency-tabs">
            <Tab label="Features" />
            <Tab label="Conflicts" />
            <Tab label="Update" />
          </Tabs>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => {}}>
          {error}
        </Alert>
      )}

      {/* Features Management */}
      {depTab === 0 && (
        <Box>
          <Box sx={{ mb: 2, display: 'flex', gap: 1, alignItems: 'center', flexWrap: 'wrap' }}>
            <Button
              variant="contained"
              onClick={onLoadFeatures}
              disabled={!currentProjectPath || loading}
            >
              {loading ? 'Loading...' : 'Load Features'}
            </Button>

            <TextField
              label="Dependency name"
              size="small"
              value={featureEdit.depName}
              onChange={(e) =>
                onFeatureEditChange({
                  ...featureEdit,
                  depName: (e.target as any).value,
                })
              }
            />

            <TextField
              label="Features (comma-separated)"
              size="small"
              value={featureEdit.features}
              onChange={(e) =>
                onFeatureEditChange({
                  ...featureEdit,
                  features: (e.target as any).value,
                })
              }
            />

            <FormControlLabel
              control={
                <Switch
                  size="small"
                  checked={featureEdit.defaultFeatures}
                  onChange={(_, checked) =>
                    onFeatureEditChange({
                      ...featureEdit,
                      defaultFeatures: checked,
                    })
                  }
                />
              }
              label={<Typography variant="caption">default-features</Typography>}
            />

            <Button
              variant="outlined"
              onClick={onApplyFeatures}
              disabled={!featureEdit.depName || loading}
            >
              Apply
            </Button>
          </Box>

          {features ? (
            <Paper sx={{ p: 2, maxHeight: 300, overflow: 'auto' }}>
              <List dense>
                {Object.entries(features).map(([name, feat]) => (
                  <ListItem key={name}>
                    <ListItemText primary={name} secondary={(feat as string[]).join(', ')} />
                  </ListItem>
                ))}
              </List>
            </Paper>
          ) : (
            <Typography variant="body2">No features loaded.</Typography>
          )}
        </Box>
      )}

      {/* Version Conflicts */}
      {depTab === 1 && (
        <Box>
          <Box sx={{ mb: 2, display: 'flex', gap: 1 }}>
            <Button
              variant="contained"
              onClick={onLoadConflicts}
              disabled={!currentProjectPath || loading}
            >
              {loading ? 'Scanning...' : 'Scan Conflicts'}
            </Button>
          </Box>

          {conflicts.length > 0 ? (
            <List dense>
              {(conflicts as PackageConflict[]).map((conflict) => (
                <ListItem
                  key={conflict.name}
                  secondaryAction={
                    <Button
                      size="small"
                      variant="outlined"
                      onClick={() => onUpdateDependencies(conflict.name)}
                    >
                      Update -p {conflict.name}
                    </Button>
                  }
                >
                  <ListItemText
                    primary={conflict.name}
                    secondary={`versions: ${conflict.versions.join(', ')}`}
                  />
                </ListItem>
              ))}
            </List>
          ) : (
            <Typography variant="body2">No conflicts found or not scanned.</Typography>
          )}
        </Box>
      )}

      {/* Update Dependencies */}
      {depTab === 2 && (
        <Box>
          <Box sx={{ display: 'flex', gap: 1, mb: 2, alignItems: 'center' }}>
            <Button
              variant="contained"
              onClick={() => onUpdateDependencies(undefined)}
              disabled={!currentProjectPath || loading}
            >
              {loading ? 'Updating...' : 'Update All'}
            </Button>

            <TextField
              size="small"
              label="Package (optional)"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const val = ((e.target as any).value as string)?.trim();
                  onUpdateDependencies(val || undefined);
                }
              }}
            />
          </Box>

          <Typography variant="body2">
            Run cargo update for all dependencies or a specific package using -p.
          </Typography>
        </Box>
      )}

      {/* Add Dependency Dialog */}
      <Dialog open={addDialogOpen} onClose={onAddDialogClose}>
        <DialogTitle>Add New Dependency</DialogTitle>
        <DialogContent>
          <DialogContentText>Add a new crate dependency to your project.</DialogContentText>

          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 2 }}>
            <TextField
              label="Package name"
              value={newDependency.name}
              onChange={(e) =>
                onNewDependencyChange({
                  ...newDependency,
                  name: (e.target as any).value,
                })
              }
              fullWidth
              required
            />

            <TextField
              label="Version (optional)"
              value={newDependency.version}
              onChange={(e) =>
                onNewDependencyChange({
                  ...newDependency,
                  version: (e.target as any).value,
                })
              }
              fullWidth
              placeholder="latest or 1.0.0"
            />

            <TextField
              label="Features (comma-separated, optional)"
              value={newDependency.features}
              onChange={(e) =>
                onNewDependencyChange({
                  ...newDependency,
                  features: (e.target as any).value,
                })
              }
              fullWidth
              placeholder="derive,rc"
            />
          </Box>
        </DialogContent>

        <DialogActions>
          <Button onClick={onAddDialogClose}>Cancel</Button>
          <Button onClick={onAddDependency} variant="contained" disabled={!newDependency.name}>
            Add
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default DependencyManager;
