import {
  Build as BuildIcon,
  CheckCircle as CheckCircleIcon,
  Code as CodeIcon,
  Info as InfoIcon,
  Memory as MemoryIcon,
  Speed as SpeedIcon,
} from '@mui/icons-material';
import { Box, Card, CardContent, Chip, List, Typography, useTheme } from '@mui/material';
import Button from '@mui/material/Button';
import React from 'react';
import { PerformanceRecommendation } from '../types';

interface PerformanceRecommendationsProps {
  recommendations: PerformanceRecommendation[];
  onApplyRecommendation?: (id: string) => void;
  onDismissRecommendation?: (id: string) => void;
}

export const PerformanceRecommendations: React.FC<PerformanceRecommendationsProps> = ({
  recommendations,
  onApplyRecommendation,
  onDismissRecommendation,
}) => {
  const theme = useTheme();

  const getImpactColor = (impact: PerformanceRecommendation['impact']) => {
    switch (impact) {
      case 'high':
        return theme.palette.error.main;
      case 'medium':
        return theme.palette.warning.main;
      case 'low':
        return theme.palette.info.main;
      default:
        return theme.palette.text.secondary;
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'dependency':
        return <InfoIcon />;
      case 'code':
        return <CodeIcon />;
      case 'build':
        return <BuildIcon />;
      case 'runtime':
        return <SpeedIcon />;
      case 'memory':
        return <MemoryIcon />;
      default:
        return <InfoIcon />;
    }
  };

  if (recommendations.length === 0) {
    return (
      <Box display="flex" flexDirection="column" alignItems="center" justifyContent="center" p={4}>
        <CheckCircleIcon color="success" fontSize="large" />
        <Typography variant="h6" color="textSecondary" gutterBottom>
          No performance recommendations
        </Typography>
        <Typography variant="body2" color="textSecondary" align="center">
          Your code is performing well. We'll let you know if we find any optimization
          opportunities.
        </Typography>
      </Box>
    );
  }

  return (
    <List disablePadding>
      {recommendations.map((rec) => (
        <Card key={rec.id} variant="outlined" sx={{ mb: 2 }}>
          <CardContent>
            <Box display="flex" alignItems="center" mb={1}>
              <Box mr={1}>{getCategoryIcon(rec.category)}</Box>
              <Typography variant="h6" component="div">
                {rec.title}
              </Typography>
              <Box flexGrow={1} />
              <Chip
                label={rec.impact}
                size="small"
                sx={{
                  backgroundColor: getImpactColor(rec.impact),
                  color: theme.palette.getContrastText(getImpactColor(rec.impact)),
                  fontWeight: 'bold',
                }}
              />
            </Box>

            <Typography variant="body2" color="text.secondary" paragraph>
              {rec.description}
            </Typography>

            {rec.estimatedImprovement && (
              <Box display="flex" alignItems="center" mb={1}>
                <SpeedIcon color="action" fontSize="small" sx={{ mr: 1 }} />
                <Typography variant="caption" color="text.secondary">
                  Estimated improvement: {rec.estimatedImprovement}
                </Typography>
              </Box>
            )}

            <Box display="flex" mt={2} justifyContent="flex-end">
              <Button
                size="small"
                variant="outlined"
                color="primary"
                onClick={() => onApplyRecommendation?.(rec.id)}
                sx={{ mr: 1 }}
              >
                Apply
              </Button>
              <Button size="small" variant="text" onClick={() => onDismissRecommendation?.(rec.id)}>
                Dismiss
              </Button>
            </Box>
          </CardContent>
        </Card>
      ))}
    </List>
  );
};

export default PerformanceRecommendations;
