import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Alert,
  Chip,
  LinearProgress,
  IconButton,
} from '@mui/material';
import {
  TrendingUp as TrendingUpIcon,
  TrendingDown as TrendingDownIcon,
  Warning as WarningIcon,
  Refresh as RefreshIcon,
} from '@mui/icons-material';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  BarChart,
  Bar,
} from 'recharts';

// Unified Performance Metrics interface (matches Rust struct)
interface UnifiedPerformanceMetrics {
  timestamp: string;
  timing: {
    total_time_ns?: number;
    response_time_ns?: number;
    compile_time_ns?: number;
    analysis_time_ns?: number;
    cache_time_ns?: number;
    crypto_time_ns?: number;
    latency_ns: number[];
  };
  counters: {
    total_operations?: number;
    successful_operations?: number;
    failed_operations?: number;
    cache_hits?: number;
    cache_misses?: number;
    allocations_analyzed?: number;
    error_count?: number;
  };
  rates: {
    cpu_usage_percent?: number;
    memory_usage_percent?: number;
    cache_hit_rate?: number;
    success_rate?: number;
    throughput_ops_per_sec?: number;
  };
  resources: {
    memory_bytes?: number;
    peak_memory_bytes?: number;
    cpu_time_ns?: number;
    network_bytes?: number;
  };
  analysis: {
    files_analyzed?: number;
    lines_analyzed?: number;
    refactoring_suggestions?: number;
    quality_score?: number;
  };
  security: {
    encryption_ops?: number;
    decryption_ops?: number;
    avg_encryption_time_ns?: number;
    security_scans?: number;
    vulnerabilities_found?: number;
  };
  build: {
    build_time_ns?: number;
    build_successful?: boolean;
    warnings_count?: number;
    errors_count?: number;
    incremental_build?: boolean;
  };
  learning: {
    training_ops?: number;
    predictions?: number;
    learning_iterations?: number;
    model_accuracy?: number;
    training_loss?: number;
  };
  extensions: { [key: string]: any };
}

interface MetricsDashboardProps {
  metrics: UnifiedPerformanceMetrics[];
  isLoading?: boolean;
  autoRefreshInterval?: number; // in seconds
  onRefresh?: () => void;
}

const MetricsDashboard: React.FC<MetricsDashboardProps> = ({
  metrics,
  isLoading = false,
  autoRefreshInterval = 30,
  onRefresh,
}) => {
  const [selectedTimeframe, setSelectedTimeframe] = useState<'1h' | '6h' | '24h' | '7d'>('1h');
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Auto-refresh functionality
  useEffect(() => {
    if (autoRefreshInterval > 0 && onRefresh) {
      const interval = setInterval(() => {
        onRefresh();
        setLastUpdate(new Date());
      }, autoRefreshInterval * 1000);

      return () => clearInterval(interval);
    }
  }, [autoRefreshInterval, onRefresh]);

  // Filter metrics based on selected timeframe
  const filteredMetrics = React.useMemo(() => {
    const now = new Date();
    const timeframeMs = {
      '1h': 60 * 60 * 1000,
      '6h': 6 * 60 * 60 * 1000,
      '24h': 24 * 60 * 60 * 1000,
      '7d': 7 * 24 * 60 * 60 * 1000,
    }[selectedTimeframe];

    const cutoff = new Date(now.getTime() - timeframeMs);

    return metrics.filter((m) => {
      const metricTime = new Date(m.timestamp);
      return metricTime >= cutoff;
    });
  }, [metrics, selectedTimeframe]);

  // Calculate key statistics
  const stats = React.useMemo(() => {
    if (filteredMetrics.length === 0) return null;

    const latest = filteredMetrics[filteredMetrics.length - 1];
    const previous =
      filteredMetrics.length > 1 ? filteredMetrics[filteredMetrics.length - 2] : null;

    return {
      cpuUsage: {
        current: latest.rates.cpu_usage_percent || 0,
        trend: previous
          ? (latest.rates.cpu_usage_percent || 0) - (previous.rates.cpu_usage_percent || 0)
          : 0,
      },
      memoryUsage: {
        current: latest.resources.memory_bytes ? latest.resources.memory_bytes / (1024 * 1024) : 0, // MB
        peak: latest.resources.peak_memory_bytes
          ? latest.resources.peak_memory_bytes / (1024 * 1024)
          : 0,
      },
      responseTime: {
        current: latest.timing.response_time_ns ? latest.timing.response_time_ns / 1_000_000 : 0, // ms
        trend:
          previous && previous.timing.response_time_ns && latest.timing.response_time_ns
            ? (latest.timing.response_time_ns - previous.timing.response_time_ns) / 1_000_000
            : 0,
      },
      throughput: {
        current: latest.rates.throughput_ops_per_sec || 0,
        trend: previous
          ? (latest.rates.throughput_ops_per_sec || 0) -
            (previous.rates.throughput_ops_per_sec || 0)
          : 0,
      },
      successRate: {
        current: latest.rates.success_rate ? latest.rates.success_rate * 100 : 0,
        trend:
          previous && previous.rates.success_rate && latest.rates.success_rate
            ? (latest.rates.success_rate - previous.rates.success_rate) * 100
            : 0,
      },
    };
  }, [filteredMetrics]);

  const formatTime = (ns: number): string => {
    if (ns < 1_000_000) return `${(ns / 1_000).toFixed(2)}μs`;
    if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(2)}ms`;
    return `${(ns / 1_000_000_000).toFixed(2)}s`;
  };

  const formatBytes = (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    return `${value.toFixed(1)} ${units[unitIndex]}`;
  };

  const TrendIndicator: React.FC<{ value: number; inverse?: boolean }> = ({
    value,
    inverse = false,
  }) => {
    if (value === 0) return null;

    const isPositive = inverse ? value < 0 : value > 0;
    const Icon = isPositive ? TrendingUpIcon : TrendingDownIcon;
    const color = isPositive ? 'success' : 'error';

    return (
      <Box display="flex" alignItems="center" gap={0.5}>
        <Icon sx={{ fontSize: 16, color: `${color}.main` }} />
        <Typography variant="body2" color={`${color}.main`}>
          {Math.abs(value).toFixed(1)}
        </Typography>
      </Box>
    );
  };

  if (isLoading) {
    return (
      <Box p={3}>
        <LinearProgress />
        <Typography variant="body2" align="center" sx={{ mt: 2 }}>
          Loading metrics...
        </Typography>
      </Box>
    );
  }

  if (!stats) {
    return (
      <Box p={3} textAlign="center">
        <Typography variant="body1" color="text.secondary">
          No metrics data available
        </Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ p: 3 }}>
      {/* Header with controls */}
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h5">Performance Dashboard</Typography>

        <Box display="flex" alignItems="center" gap={2}>
          {/* Timeframe selector */}
          <Box display="flex" gap={1}>
            {(['1h', '6h', '24h', '7d'] as const).map((tf) => (
              <Chip
                key={tf}
                label={tf}
                onClick={() => setSelectedTimeframe(tf)}
                variant={selectedTimeframe === tf ? 'filled' : 'outlined'}
                size="small"
              />
            ))}
          </Box>

          {/* Refresh button */}
          <IconButton
            onClick={() => {
              onRefresh?.();
              setLastUpdate(new Date());
            }}
            disabled={isLoading}
          >
            <RefreshIcon />
          </IconButton>

          {/* Last update */}
          <Typography variant="body2" color="text.secondary">
            Last update: {lastUpdate.toLocaleTimeString()}
          </Typography>
        </Box>
      </Box>

      {/* Key Metrics Cards */}
      <Grid container spacing={3} mb={4}>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={2}>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                CPU Usage
              </Typography>
              <Typography variant="h4" component="div" gutterBottom>
                {stats.cpuUsage.current.toFixed(1)}%
              </Typography>
              <TrendIndicator value={stats.cpuUsage.trend} />
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={2}>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                Memory Usage
              </Typography>
              <Typography variant="h4" component="div" gutterBottom>
                {stats.memoryUsage.current.toFixed(1)} MB
              </Typography>
              {stats.memoryUsage.peak > 0 && (
                <Typography variant="body2" color="text.secondary">
                  Peak: {stats.memoryUsage.peak.toFixed(1)} MB
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={2}>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                Response Time
              </Typography>
              <Typography variant="h4" component="div" gutterBottom>
                {formatTime(stats.responseTime.current * 1_000_000)}
              </Typography>
              <TrendIndicator value={stats.responseTime.trend} inverse />
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={2}>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                Throughput
              </Typography>
              <Typography variant="h4" component="div" gutterBottom>
                {stats.throughput.current.toFixed(0)} ops/s
              </Typography>
              <TrendIndicator value={stats.throughput.trend} />
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Charts */}
      <Grid container spacing={3}>
        <Grid item xs={12} lg={8}>
          <Card elevation={2}>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                CPU Usage Trend
              </Typography>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart
                  data={filteredMetrics.map((m) => ({
                    time: new Date(m.timestamp).toLocaleTimeString(),
                    cpu: m.rates.cpu_usage_percent || 0,
                  }))}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="time" />
                  <YAxis />
                  <Tooltip />
                  <Line type="monotone" dataKey="cpu" stroke="#8884d8" strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} lg={4}>
          <Card elevation={2}>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                System Health
              </Typography>

              {/* Success Rate */}
              <Box mb={2}>
                <Box display="flex" justifyContent="space-between" mb={1}>
                  <Typography variant="body2">Success Rate</Typography>
                  <Typography variant="body2">{stats.successRate.current.toFixed(1)}%</Typography>
                </Box>
                <LinearProgress
                  variant="determinate"
                  value={stats.successRate.current}
                  sx={{
                    height: 8,
                    borderRadius: 4,
                    '& .MuiLinearProgress-bar': {
                      borderRadius: 4,
                    },
                  }}
                />
              </Box>

              {/* Error Count */}
              {filteredMetrics[filteredMetrics.length - 1]?.counters.error_count > 0 && (
                <Alert severity="warning" sx={{ mb: 2 }}>
                  <Typography variant="body2">
                    Errors detected:{' '}
                    {filteredMetrics[filteredMetrics.length - 1]?.counters.error_count}
                  </Typography>
                </Alert>
              )}

              {/* Build Status */}
              {filteredMetrics[filteredMetrics.length - 1]?.build.build_successful !==
                undefined && (
                <Chip
                  label={
                    filteredMetrics[filteredMetrics.length - 1]?.build.build_successful
                      ? 'Build OK'
                      : 'Build Failed'
                  }
                  color={
                    filteredMetrics[filteredMetrics.length - 1]?.build.build_successful
                      ? 'success'
                      : 'error'
                  }
                  size="small"
                />
              )}
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12}>
          <Card elevation={2}>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Response Time Distribution
              </Typography>
              <ResponsiveContainer width="100%" height={250}>
                <BarChart
                  data={filteredMetrics.slice(-20).map((m) => ({
                    time: new Date(m.timestamp).toLocaleTimeString(),
                    response: m.timing.response_time_ns ? m.timing.response_time_ns / 1_000_000 : 0, // convert to ms
                  }))}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="time" />
                  <YAxis />
                  <Tooltip />
                  <Bar dataKey="response" fill="#82ca9d" />
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
};

export default MetricsDashboard;
