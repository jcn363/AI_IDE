import React from 'react';
import { Box, Paper, Typography, useTheme } from '@mui/material';
import { PerformanceDashboard } from '../../../../features/performance';

interface PerformanceTabProps {
  projectId: string;
}

const PerformanceTab: React.FC<PerformanceTabProps> = ({ projectId }) => {
  const theme = useTheme();

  return (
    <Box sx={{ p: 3 }}>
      <Paper
        elevation={0}
        sx={{
          p: 3,
          borderRadius: 2,
          backgroundColor: theme.palette.background.paper,
          border: `1px solid ${theme.palette.divider}`,
        }}
      >
        <Typography variant="h5" gutterBottom>
          Performance Analysis
        </Typography>
        <Typography variant="body1" color="text.secondary" paragraph>
          Review performance metrics and optimization recommendations for your project dependencies.
        </Typography>

        <Box mt={4}>
          <PerformanceDashboard />
        </Box>
      </Paper>
    </Box>
  );
};

export default PerformanceTab;
