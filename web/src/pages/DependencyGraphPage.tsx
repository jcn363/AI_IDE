import React from 'react';
import { Box, CircularProgress, Paper, Typography, useTheme } from '@mui/material';
import { useCurrentProject } from '../hooks/useCurrentProject';
import DependencyGraphVisualization from '../features/cargoToml/DependencyGraphVisualization';

const DependencyGraphPage: React.FC = () => {
  const theme = useTheme();
  const { currentProject, projects, loading, error } = useCurrentProject();
  
  // Show loading state
  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <CircularProgress />
      </Box>
    );
  }
  
  // Show error if any
  if (error) {
    return (
      <Box sx={{ p: 3 }}>
        <Paper sx={{ p: 3, textAlign: 'center', bgcolor: 'error.light' }}>
          <Typography variant="h6" gutterBottom color="error">
            Error Loading Project
          </Typography>
          <Typography variant="body1" color="error">
            {error}
          </Typography>
        </Paper>
      </Box>
    );
  }
  
  // If no project is selected, show a message
  if (!currentProject) {
    return (
      <Box sx={{ p: 3 }}>
        <Paper sx={{ p: 3, textAlign: 'center' }}>
          <Typography variant="h6" gutterBottom>
            No Project Selected
          </Typography>
          <Typography variant="body1">
            Please open a project to view its dependency graph.
          </Typography>
        </Paper>
      </Box>
    );
  }

  return (
    <Box
      sx={{
        height: 'calc(100vh - 64px)', // Adjust based on your header height
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: theme.palette.background.default,
      }}
    >
      <Box sx={{ p: 2, borderBottom: `1px solid ${theme.palette.divider}` }}>
        <Typography variant="h5" component="h1">
          Dependency Graph: {currentProject.name}
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ 
          fontFamily: 'monospace',
          wordBreak: 'break-all',
          mt: 1,
        }}>
          {currentProject.path}
        </Typography>
      </Box>
      
      <Box sx={{ flex: 1, overflow: 'hidden' }}>
        <DependencyGraphVisualization 
          projectPath={currentProject.path}
          width="100%"
          height="100%"
          showControls={true}
        />
      </Box>
    </Box>
  );
};

export default DependencyGraphPage;
