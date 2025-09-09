import { Box, Card, CardContent, Typography } from '@mui/material';
import React from 'react';

interface WorkspaceStatsProps {
  total: number;
  aligned: number;
  conflicts: number;
  highSeverity: number;
  mediumSeverity: number;
  lowSeverity: number;
  loading?: boolean;
}

export const WorkspaceStats: React.FC<WorkspaceStatsProps> = ({
  aligned,
  conflicts,
  highSeverity,
  loading = false,
  lowSeverity,
  mediumSeverity,
  total,
}) => {
  if (loading) {
    return (
      <Box mb={3}>
        <Typography>Loading statistics...</Typography>
      </Box>
    );
  }

  const statItems = [
    { label: 'Total Dependencies', value: total },
    { label: 'Aligned', value: aligned, color: 'success.main' },
    { label: 'Conflicts', value: conflicts, color: 'error.main' },
    { label: 'High Severity', value: highSeverity, color: 'error.main' },
    { label: 'Medium Severity', value: mediumSeverity, color: 'warning.main' },
    { label: 'Low Severity', value: lowSeverity, color: 'info.main' },
  ];

  return (
    <Box mb={3}>
      <Box sx={{
        display: 'grid',
        gridTemplateColumns: {
          xs: '1fr',
          sm: 'repeat(2, 1fr)',
          md: 'repeat(3, 1fr)',
          lg: 'repeat(6, 1fr)',
        },
        gap: 2,
      }}>
        {statItems.map((item) => (
          <Box key={item.label}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="h6" component="div" color={item.color}>
                  {item.value}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {item.label}
                </Typography>
              </CardContent>
            </Card>
          </Box>
        ))}
      </Box>
    </Box>
  );
};

export default WorkspaceStats;
