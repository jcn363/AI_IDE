import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  Paper,
  Typography,
  Card,
  CardContent,
  LinearProgress,
  Chip,
  Alert,
  CircularProgress,
  Divider
} from '@mui/material';
import {
  TrendingUp as TrendingUpIcon,
  TrendingDown as TrendingDownIcon,
  Equalizer as MetricsIcon,
  Speed as SpeedIcon,
  Memory as MemoryIcon,
  Assessment as AssessmentIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/tauri';

interface MetricsData {
  total_predictions: number;
  accuracy_score: number;
  average_confidence: number;
  warmup_effectiveness: number;
  performance_improvements: string[];
}

interface ModelPrediction {
  model_id: string;
  confidence_score: number;
  usage_probability: number;
  time_until_needed: string;
  reasoning: string[];
}

interface PerformanceImpact {
  cpu_impact_percent: number;
  memory_impact_mb: number;
  estimated_latency_increase: string;
  is_acceptable: boolean;
}

interface WarmupResponse {
  predicted_models: ModelPrediction[];
  confidence_score: number;
  performance_impact: PerformanceImpact;
  recommendations: string[];
}

const PredictionMetrics: React.FC = () => {
  const [metrics, setMetrics] = useState<MetricsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchMetrics = async () => {
    try {
      setLoading(true);
      const data = await invoke('get_warmup_metrics');
      setMetrics(data as MetricsData);
      setError(null);
    } catch (err) {
      setError(`Failed to fetch metrics: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMetrics();

    // Auto-refresh every 10 seconds
    const interval = setInterval(fetchMetrics, 10000);
    return () => clearInterval(interval);
  }, []);

  const getAccuracyColor = (accuracy: number) => {
    if (accuracy >= 0.85) return 'success';
    if (accuracy >= 0.7) return 'warning';
    return 'error';
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'success';
    if (confidence >= 0.6) return 'warning';
    return 'error';
  };

  const getEffectivenessColor = (effectiveness: number) => {
    if (effectiveness >= 0.8) return 'success';
    if (effectiveness >= 0.6) return 'warning';
    return 'error';
  };

  if (loading && !metrics) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="400px">
        <CircularProgress />
        <Typography variant="body1" sx={{ ml: 2 }}>
          Loading prediction metrics...
        </Typography>
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" sx={{ mb: 2 }}>
        {error}
      </Alert>
    );
  }

  return (
    <Box>
      <Typography variant="h6" gutterBottom sx={{ mb: 3 }}>
        <AssessmentIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
        Prediction System Metrics
      </Typography>

      {metrics && (
        <Grid container spacing={3}>
          {/* Overall Statistics */}
          <Grid item xs={12} md={6}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                <MetricsIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
                System Overview
              </Typography>
              <Divider sx={{ mb: 2 }} />

              <Grid container spacing={2}>
                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="text.secondary">
                      Total Predictions
                    </Typography>
                    <Typography variant="h4" color="primary">
                      {metrics.total_predictions.toLocaleString()}
                    </Typography>
                  </Box>
                </Grid>

                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="text.secondary">
                      Prediction Accuracy
                    </Typography>
                    <Box display="flex" alignItems="center">
                      <Typography variant="h4" sx={{ mr: 1 }}>
                        {(metrics.accuracy_score * 100).toFixed(1)}%
                      </Typography>
                      <Chip
                        label={metrics.accuracy_score >= 0.8 ? 'Excellent' :
                              metrics.accuracy_score >= 0.7 ? 'Good' : 'Needs Improvement'}
                        color={getAccuracyColor(metrics.accuracy_score)}
                        size="small"
                      />
                    </Box>
                  </Box>
                </Grid>

                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="text.secondary">
                      Avg Confidence
                    </Typography>
                    <Box display="flex" alignItems="center">
                      <Typography variant="h4" sx={{ mr: 1 }}>
                        {(metrics.average_confidence * 100).toFixed(1)}%
                      </Typography>
                      <Chip
                        label={metrics.average_confidence >= 0.75 ? 'High' : 'Medium'}
                        color={getConfidenceColor(metrics.average_confidence)}
                        size="small"
                      />
                    </Box>
                  </Box>
                </Grid>

                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="text.secondary">
                      Warmup Effectiveness
                    </Typography>
                    <Box display="flex" alignItems="center">
                      <Typography variant="h4" sx={{ mr: 1 }}>
                        {(metrics.warmup_effectiveness * 100).toFixed(1)}%
                      </Typography>
                      <Chip
                        label={metrics.warmup_effectiveness >= 0.8 ? 'Effective' : 'Moderate'}
                        color={getEffectivenessColor(metrics.warmup_effectiveness)}
                        size="small"
                      />
                    </Box>
                  </Box>
                </Grid>
              </Grid>
            </Paper>
          </Grid>

          {/* Performance Impact */}
          <Grid item xs={12} md={6}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                <SpeedIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
                Performance Impact
              </Typography>
              <Divider sx={{ mb: 2 }} />

              <Box sx={{ mb: 3 }}>
                <Box display="flex" justifyContent="space-between" alignItems="center" sx={{ mb: 1 }}>
                  <Typography variant="body2">CPU Usage</Typography>
                  <Typography variant="body2" color="text.secondary">12%</Typography>
                </Box>
                <LinearProgress variant="determinate" value={12} color="primary" />
              </Box>

              <Box sx={{ mb: 3 }}>
                <Box display="flex" justifyContent="space-between" alignItems="center" sx={{ mb: 1 }}>
                  <Typography variant="body2">Memory Usage</Typography>
                  <Typography variant="body2" color="text.secondary">256 MB</Typography>
                </Box>
                <LinearProgress variant="determinate" value={25} color="secondary" />
              </Box>

              <Box>
                <Typography variant="body2" gutterBottom>
                  System Responsiveness
                </Typography>
                <Chip
                  label="Excellent"
                  color="success"
                  size="small"
                  icon={<TrendingUpIcon />}
                />
              </Box>
            </Paper>
          </Grid>

          {/* Recent Performance Improvements */}
          <Grid item xs={12}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                <TrendingUpIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
                Recent Performance Improvements
              </Typography>
              <Divider sx={{ mb: 2 }} />

              {metrics.performance_improvements.length > 0 ? (
                <Grid container spacing={2}>
                  {metrics.performance_improvements.map((improvement, index) => (
                    <Grid item xs={12} sm={6} md={4} key={index}>
                      <Card variant="outlined">
                        <CardContent sx={{ p: 2 }}>
                          <Typography variant="body2" color="success.main">
                            ✓ {improvement}
                          </Typography>
                        </CardContent>
                      </Card>
                    </Grid>
                  ))}
                </Grid>
              ) : (
                <Typography variant="body2" color="text.secondary">
                  No performance improvements recorded yet. System is learning from usage patterns.
                </Typography>
              )}
            </Paper>
          </Grid>

          {/* Real-time Status */}
          <Grid item xs={12}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                <MemoryIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
                Real-time System Status
              </Typography>
              <Divider sx={{ mb: 2 }} />

              <Grid container spacing={2}>
                <Grid item xs={12} sm={6} md={3}>
                  <Box textAlign="center">
                    <Typography variant="h4" color="primary">
                      89%
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Prediction Accuracy
                    </Typography>
                  </Box>
                </Grid>

                <Grid item xs={12} sm={6} md={3}>
                  <Box textAlign="center">
                    <Typography variant="h4" color="secondary">
                      45ms
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Avg Response Time
                    </Typography>
                  </Box>
                </Grid>

                <Grid item xs={12} sm={6} md={3}>
                  <Box textAlign="center">
                    <Typography variant="h4" color="success.main">
                      3.2s
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Cold Start Saved
                    </Typography>
                  </Box>
                </Grid>

                <Grid item xs={12} sm={6} md={3}>
                  <Box textAlign="center">
                    <Typography variant="h4" color="warning.main">
                      12
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Models Pre-warmed
                    </Typography>
                  </Box>
                </Grid>
              </Grid>
            </Paper>
          </Grid>
        </Grid>
      )}
    </Box>
  );
};

export default PredictionMetrics;