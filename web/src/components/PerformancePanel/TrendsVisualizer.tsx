import React, { useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Chip,
  LinearProgress,
} from '@mui/material';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area,
} from 'recharts';
import { TrendingUp, TrendingDown, TrendingFlat } from '@mui/icons-material';

// Trend data interface (matches Rust TrendAnalysis)
interface TrendAnalysis {
  metric_name: string;
  trend_coefficient: number;
  confidence: number;
  description: string;
  next_prediction: number;
}

// Historical data point
interface DataPoint {
  timestamp: string;
  value: number;
  isPredicted?: boolean;
}

// Component props
interface TrendsVisualizerProps {
  trends: TrendAnalysis[];
  historicalData: { [metricName: string]: DataPoint[] };
  isLoading?: boolean;
  onMetricSelect?: (metricName: string) => void;
}

const TrendsVisualizer: React.FC<TrendsVisualizerProps> = ({
  trends,
  historicalData,
  isLoading = false,
  onMetricSelect,
}) => {
  const [selectedMetric, setSelectedMetric] = useState<string>('');

  // Find selected trend data
  const selectedTrend = trends.find((t) => t.metric_name === selectedMetric);
  const selectedData = historicalData[selectedMetric];

  // Get trend direction icon and color
  const getTrendIndicator = (coefficient: number) => {
    if (coefficient > 0.1) {
      return { icon: TrendingUp, color: '#4caf50', direction: 'up' };
    } else if (coefficient < -0.1) {
      return { icon: TrendingDown, color: '#f44336', direction: 'down' };
    } else {
      return { icon: TrendingFlat, color: '#9e9e9e', direction: 'flat' };
    }
  };

  // Format confidence as percentage
  const formatConfidence = (confidence: number): string => {
    return `${Math.round(confidence * 100)}%`;
  };

  // Get confidence description
  const getConfidenceDescription = (confidence: number): string => {
    if (confidence >= 0.8) return 'High Confidence';
    if (confidence >= 0.6) return 'Medium Confidence';
    if (confidence >= 0.4) return 'Low Confidence';
    return 'Very Low Confidence';
  };

  const TrendIndicator: React.FC<{ trend: TrendAnalysis }> = ({ trend }) => {
    const { icon: Icon, color, direction } = getTrendIndicator(trend.trend_coefficient);

    return (
      <Box display="flex" alignItems="center" gap={1}>
        <Icon sx={{ color, fontSize: 24 }} />
        <Box>
          <Typography variant="body2" fontWeight="medium">
            {direction.toUpperCase()}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {formatConfidence(trend.confidence)}
          </Typography>
        </Box>
      </Box>
    );
  };

  if (isLoading) {
    return (
      <Box p={3}>
        <LinearProgress />
        <Typography align="center" sx={{ mt: 2 }}>
          Loading trend analysis...
        </Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h5" gutterBottom>
        Performance Trends Analysis
      </Typography>

      {/* Trends Overview */}
      <Grid container spacing={2} mb={3}>
        <Grid item xs={12}>
          <Typography variant="h6" gutterBottom>
            Metric Trends Summary
          </Typography>
          <Grid container spacing={2}>
            {trends.map((trend) => (
              <Grid item xs={12} sm={6} md={4} key={trend.metric_name}>
                <Card
                  elevation={2}
                  sx={{
                    cursor: 'pointer',
                    '&:hover': { elevation: 4 },
                  }}
                  onClick={() => {
                    setSelectedMetric(trend.metric_name);
                    onMetricSelect?.(trend.metric_name);
                  }}
                >
                  <CardContent sx={{ pb: '16px !important' }}>
                    <Box display="flex" justifyContent="space-between" mb={1}>
                      <Typography variant="subtitle1">
                        {trend.metric_name.replace('_', ' ').toUpperCase()}
                      </Typography>
                      <TrendIndicator trend={trend} />
                    </Box>

                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      {getConfidenceDescription(trend.confidence)}
                    </Typography>

                    <Box mb={1}>
                      <Typography variant="body2">
                        Trend: {trend.trend_coefficient.toFixed(3)}
                      </Typography>
                      <Typography variant="body2">
                        Next: {trend.next_prediction.toFixed(2)}
                      </Typography>
                    </Box>

                    <Typography variant="caption" sx={{ mt: 1, display: 'block' }}>
                      {trend.description}
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        </Grid>
      </Grid>

      {/* Detailed Trend Visualization */}
      {selectedTrend && selectedData && (
        <Card elevation={2}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              {selectedTrend.metric_name.replace('_', ' ').toUpperCase()} - Trend Analysis
            </Typography>

            {/* Trend Alert */}
            <Alert severity="info" sx={{ mb: 3 }}>
              <Typography variant="body2">
                {selectedTrend.description} with {formatConfidence(selectedTrend.confidence)}{' '}
                confidence. Next predicted value: {selectedTrend.next_prediction.toFixed(2)}
              </Typography>
            </Alert>

            <Grid container spacing={3}>
              <Grid item xs={12} md={8}>
                <Box mb={2}>
                  <Typography variant="subtitle1" gutterBottom>
                    Historical Trend
                  </Typography>
                </Box>

                <ResponsiveContainer width="100%" height={400}>
                  <AreaChart data={selectedData}>
                    <defs>
                      <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="#8884d8" stopOpacity={0.8} />
                        <stop offset="95%" stopColor="#8884d8" stopOpacity={0.1} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                      dataKey="timestamp"
                      tickFormatter={(value) => new Date(value).toLocaleTimeString()}
                    />
                    <YAxis />
                    <Tooltip
                      labelFormatter={(value) => new Date(value).toLocaleString()}
                      formatter={(value: number, name: string) => [
                        value.toFixed(2),
                        name.replace('_', ' ').toUpperCase(),
                      ]}
                    />
                    <Area
                      type="monotone"
                      dataKey="value"
                      stroke="#8884d8"
                      fillOpacity={1}
                      fill="url(#colorValue)"
                      strokeWidth={2}
                    />
                    {/* Prediction line for future values */}
                    {selectedData
                      .filter((d) => d.isPredicted)
                      .map((point, index) => (
                        <Line
                          key={`pred-${index}`}
                          type="monotone"
                          dataKey="value"
                          stroke="#ff7300"
                          strokeDasharray="5 5"
                          dot={{ fill: '#ff7300', strokeWidth: 2, r: 4 }}
                          connectNulls={false}
                        />
                      ))}
                  </AreaChart>
                </ResponsiveContainer>
              </Grid>

              <Grid item xs={12} md={4}>
                <Typography variant="subtitle1" gutterBottom>
                  Trend Statistics
                </Typography>

                <Box sx={{ mb: 2 }}>
                  <Typography variant="body2" color="text.secondary">
                    Direction
                  </Typography>
                  <Typography
                    variant="h6"
                    color={
                      selectedTrend.trend_coefficient > 0
                        ? 'success.main'
                        : selectedTrend.trend_coefficient < 0
                          ? 'error.main'
                          : 'text.primary'
                    }
                  >
                    {selectedTrend.trend_coefficient > 0
                      ? 'Improving'
                      : selectedTrend.trend_coefficient < 0
                        ? 'Degrading'
                        : 'Stable'}
                  </Typography>
                </Box>

                <Box sx={{ mb: 2 }}>
                  <Typography variant="body2" color="text.secondary">
                    Confidence Level
                  </Typography>
                  <LinearProgress
                    variant="determinate"
                    value={selectedTrend.confidence * 100}
                    sx={{ mt: 1, height: 8, borderRadius: 4 }}
                  />
                  <Typography variant="body2" sx={{ mt: 0.5 }}>
                    {formatConfidence(selectedTrend.confidence)}
                  </Typography>
                </Box>

                <Box sx={{ mb: 2 }}>
                  <Typography variant="body2" color="text.secondary">
                    Trend Coefficient
                  </Typography>
                  <Typography variant="h6">{selectedTrend.trend_coefficient.toFixed(3)}</Typography>
                </Box>

                <Box sx={{ mb: 2 }}>
                  <Typography variant="body2" color="text.secondary">
                    Next Prediction
                  </Typography>
                  <Typography variant="h6">{selectedTrend.next_prediction.toFixed(2)}</Typography>
                </Box>

                <Box sx={{ mb: 2 }}>
                  <Typography variant="body2" color="text.secondary">
                    Data Points
                  </Typography>
                  <Typography variant="h6">{selectedData.length}</Typography>
                </Box>

                {/* Predicted vs Actual comparison if available */}
                {selectedData.some((d) => d.isPredicted) && (
                  <Alert severity="info">
                    <Typography variant="body2">
                      Forecasting: {selectedData.filter((d) => d.isPredicted).length} predicted
                      value(s) shown in orange
                    </Typography>
                  </Alert>
                )}
              </Grid>
            </Grid>
          </CardContent>
        </Card>
      )}

      {/* No metric selected state */}
      {!selectedTrend && (
        <Box textAlign="center" py={8}>
          <Typography variant="h6" color="text.secondary">
            Select a Metric to View Detailed Trends
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            Click on any trend card above to see detailed analysis and predictions
          </Typography>
        </Box>
      )}

      {/* Metric selector if no trends available */}
      {trends.length === 0 && !isLoading && (
        <Box textAlign="center" py={8}>
          <Typography variant="h6" color="text.secondary">
            No Trend Data Available
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            Trend data will be available once sufficient historical data is collected
          </Typography>
        </Box>
      )}
    </Box>
  );
};

export default TrendsVisualizer;
