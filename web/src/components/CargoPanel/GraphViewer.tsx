import React from 'react';
import { Box, Button, List, ListItem, ListItemText, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';

interface GraphViewerProps {
  fullMetadata: any;
  isLoading: boolean;
  currentProjectPath: string | null;
  onLoadGraph: () => void;
}

interface DependencyNode {
  name: string;
  version: string;
  dependencies: string[];
}

export const GraphViewer: React.FC<GraphViewerProps> = ({
  fullMetadata,
  isLoading,
  currentProjectPath,
  onLoadGraph,
}) => {
  const renderDependencyTree = (packages: any[], visited = new Set<string>()) => {
    if (!packages || packages.length === 0) return null;

    return packages.map((pkg: any) => {
      const key = `${pkg.name}-${pkg.version}`;
      if (visited.has(key)) return null;
      visited.add(key);

      return (
        <ListItem key={key} alignItems="flex-start">
          <ListItemText
            primary={`${pkg.name} @ ${pkg.version}`}
            secondary={(pkg.dependencies || []).map((d: any) => d.name).join(', ') || 'No deps'}
            sx={{ wordBreak: 'break-word' }}
          />
        </ListItem>
      );
    });
  };

  return (
    <Box>
      <Box sx={{ mb: 2 }}>
        <Button
          variant="contained"
          onClick={onLoadGraph}
          disabled={!currentProjectPath || isLoading}
        >
          {isLoading ? 'Loading...' : 'Reload Graph'}
        </Button>
      </Box>

      {fullMetadata ? (
        <Box sx={{ maxHeight: 400, overflow: 'auto' }}>
          <List dense>
            {renderDependencyTree(fullMetadata.packages)}
          </List>
        </Box>
      ) : (
        <Typography variant="body2" sx={{ mt: 2 }}>
          No dependency graph loaded. Click "Reload Graph" to view dependencies.
        </Typography>
      )}
    </Box>
  );
};

export default GraphViewer;