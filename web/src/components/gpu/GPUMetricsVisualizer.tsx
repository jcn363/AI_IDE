import React, { useState, useEffect, useMemo, useCallback } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Button,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  IconButton,
  Tooltip,
  CircularProgress,
  Alert,
  useTheme,
  Chip
} from '@mui/material';
import { SelectChangeEvent } from '@mui/material/Select';
import {
  Download as DownloadIcon,
  Refresh as RefreshIcon,
  ZoomIn as ZoomInIcon,
  ZoomOut as ZoomOutIcon,
  PlayArrow as PlayIcon,
  Pause as PauseIcon
} from '@mui/icons-material';
import WebGLCanvas from './WebGLCanvas';

// Types for GPU metrics
export interface GPUMetric {
  timestamp: number;
  gpuUtilization: number;
  memoryUsage: number;
  memoryTotal: number;
  temperature?: number;
  powerUsage?: number;
  inferenceTime?: number;
  batchSize?: number;
  modelName?: string;
}

export interface GPUMetricsData {
  device: string;
  metrics: GPUMetric[];
  startTime: number;
  endTime: number;
  modelComparisons?: Record<string, GPUMetric[]>;
}

// Props for the GPU Metrics Visualizer component
interface GPUMetricsVisualizerProps {
  data?: GPUMetricsData;
  width: number;
  height: number;
  onMetricsRequest?: (timeRange: { start: number; end: number }) => void;
  onExportRequest?: (format: 'csv' | 'json' | 'png') => void;
  realTime?: boolean;
  showControls?: boolean;
  comparisonMode?: boolean;
  theme?: 'light' | 'dark';
}

// Time range options for the visualizer
const TIME_RANGES = [
  { label: '1 minute', value: 60 * 1000 },
  { label: '5 minutes', value: 5 * 60 * 1000 },
  { label: '15 minutes', value: 15 * 60 * 1000 },
  { label: '1 hour', value: 60 * 60 * 1000 },
  { label: '4 hours', value: 4 * 60 * 60 * 1000 },
  { label: '24 hours', value: 24 * 60 * 60 * 1000 }
];

// WebGL shader programs for GPU metrics visualization
const metricVertexShader = `
  attribute vec2 a_position;
  attribute vec2 a_texCoord;
  attribute vec4 a_color;
  uniform mat4 u_projection;
  uniform float u_time;
  varying vec2 v_texCoord;
  varying vec4 v_color;

  void main() {
    gl_Position = u_projection * vec4(a_position, 0.0, 1.0);
    v_texCoord = a_texCoord;
    v_color = a_color;
  }
`;

const metricFragmentShader = `
  precision mediump float;
  uniform vec4 u_baseColor;
  uniform float u_alpha;
  varying vec2 v_texCoord;
  varying vec4 v_color;

  void main() {
    float alpha = u_alpha;
    if (v_texCoord.y > 0.95) {
      alpha *= 0.3; // Fade top for chart border
    }
    gl_FragColor = v_color * u_baseColor * vec4(1.0, 1.0, 1.0, alpha);
  }
`;

// GPU Metrics Visualizer Component
export const GPUMetricsVisualizer: React.FC<GPUMetricsVisualizerProps> = ({
  data,
  width,
  height,
  onMetricsRequest,
  onExportRequest,
  realTime = true,
  showControls = true,
  comparisonMode = false,
  theme = 'light',
}) => {
  const [selectedTimeRange, setSelectedTimeRange] = useState(TIME_RANGES[2].value);
  const [isPlaying, setIsPlaying] = useState(realTime);
  const [zoom, setZoom] = useState(1.0);
  const [selectedMetric, setSelectedMetric] = useState<'gpu' | 'memory' | 'temperature'>('gpu');
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const themeMui = useTheme();

  // Calculate metric statistics
  const metricStats = useMemo(() => {
    if (!data?.metrics.length) return null;

    const metrics = data.metrics;
    const recentMetrics = metrics.slice(-50); // Last 50 data points

    return {
      gpu: {
        current: recentMetrics[recentMetrics.length - 1]?.gpuUtilization || 0,
        average: recentMetrics.reduce((sum, m) => sum + m.gpuUtilization, 0) / recentMetrics.length,
        peak: Math.max(...recentMetrics.map(m => m.gpuUtilization)),
        trend: recentMetrics.length > 1 ?
          recentMetrics[recentMetrics.length - 1].gpuUtilization - recentMetrics[0].gpuUtilization : 0
      },
      memory: {
        used: recentMetrics[recentMetrics.length - 1]?.memoryUsage || 0,
        total: data.metrics[0]?.memoryTotal || 1,
        percentage: ((recentMetrics[recentMetrics.length - 1]?.memoryUsage || 0) /
                    (data.metrics[0]?.memoryTotal || 1)) * 100
      }
    };
  }, [data]);

  // WebGL render function
  const handleWebGLRender = useCallback((gl: WebGLRenderingContext, time: number) => {
    if (!data?.metrics.length) return;

    // Set viewport
    gl.viewport(0, 0, width, height);

    // Clear canvas
    const baseColor = theme === 'dark' ? [0.1, 0.1, 0.15, 1.0] : [0.95, 0.95, 0.98, 1.0];
    gl.clearColor(...baseColor);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    // Render metric chart (simplified vertex-based rendering)
    const metrics = data.metrics.slice(-100); // Render last 100 points
    if (metrics.length > 0) {
      const vertices: number[] = [];
      const colors: number[] = [];

      metrics.forEach((metric, i) => {
        const x = (i / metrics.length) * width * zoom;
        let value = metric.gpuUtilization;

        if (selectedMetric === 'memory') {
          value = (metric.memoryUsage / metric.memoryTotal) * 100;
        } else if (selectedMetric === 'temperature' && metric.temperature) {
          value = (metric.temperature / 100) * 100; // Assume max temp 100°C
        }

        const y = (value / 100) * height;

        // Create line segments
        if (i > 0) {
          const prevMetric = metrics[i - 1];
          let prevValue = prevMetric.gpuUtilization;

          if (selectedMetric === 'memory') {
            prevValue = (prevMetric.memoryUsage / prevMetric.memoryTotal) * 100;
          } else if (selectedMetric === 'temperature' && prevMetric.temperature) {
            prevValue = (prevMetric.temperature / 100) * 100;
          }

          const prevX = ((i - 1) / metrics.length) * width * zoom;
          const prevY = (prevValue / 100) * height;

          // Previous point
          vertices.push(prevX, prevY);
          colors.push(0.2, 0.6, 1.0, 1.0);

          // Current point
          vertices.push(x, y);
          colors.push(0.2, 0.6, 1.0, 1.0);
        }
      });

      // Render using WebGL (simplified)
      // In a real implementation, this would set up buffers and shaders for actual rendering
    }
  }, [data, width, height, zoom, selectedMetric, theme]);

  // Handle time range selection
  const handleTimeRangeChange = (event: SelectChangeEvent<number>) => {
    const value = event.target.value as number;
    setSelectedTimeRange(value);
    onMetricsRequest?.({
      start: Date.now() - value,
      end: Date.now()
    });
  };

  // Handle export
  const handleExport = (format: 'csv' | 'json' | 'png') => {
    onExportRequest?.(format);
  };

  // Toggle play/pause
  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  // Zoom controls
  const handleZoomIn = () => {
    setZoom(prev => Math.min(prev * 1.2, 5.0));
  };

  const handleZoomOut = () => {
    setZoom(prev => Math.max(prev / 1.2, 0.5));
  };

  // Periodic data refresh
  useEffect(() => {
    if (!realTime || !isPlaying) return;

    const interval = setInterval(() => {
      onMetricsRequest?.({
        start: Date.now() - selectedTimeRange,
        end: Date.now()
      });
    }, 5000); // Update every 5 seconds

    return () => clearInterval(interval);
  }, [realTime, isPlaying, selectedTimeRange, onMetricsRequest]);

  // Initial data request
  useEffect(() => {
    if (onMetricsRequest) {
      setIsLoading(true);
      onMetricsRequest({
        start: Date.now() - selectedTimeRange,
        end: Date.now()
      }).finally(() => setIsLoading(false));
    }
  }, [onMetricsRequest]);

  if (!data) {
    return (
      <Card sx={{ width, height, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <CircularProgress />
        <Typography variant="body2" sx={{ ml: 2 }}>
          Loading GPU metrics...
        </Typography>
      </Card>
    );
  }

  return (
    <Card sx={{ width, height }}>
      <CardContent sx={{ height: '100%', p: 0 }}>
        {/* Header with controls */}
        {showControls && (
          <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
            <Grid container spacing={2} alignItems="center">
              <Grid item xs={12} sm={6}>
                <Typography variant="h6">GPU Performance Monitor</Typography>
                {data.device && (
                  <Typography variant="body2" color="text.secondary">
                    Device: {data.device}
                  </Typography>
                )}
              </Grid>
              <Grid item xs={12} sm={6}>
                <Box sx={{ display: 'flex', gap: 1, alignItems: 'center', justifyContent: 'flex-end' }}>
                  <FormControl size="small" sx={{ minWidth: 120 }}>
                    <InputLabel>Time Range</InputLabel>
                    <Select
                      value={selectedTimeRange}
                      onChange={handleTimeRangeChange}
                      label="Time Range"
                    >
                      {TIME_RANGES.map(range => (
                        <MenuItem key={range.value} value={range.value}>
                          {range.label}
                        </MenuItem>
                      ))}
                    </Select>
                  </FormControl>
                  <IconButton size="small" onClick={handleZoomIn}>
                    <ZoomInIcon />
                  </IconButton>
                  <IconButton size="small" onClick={handleZoomOut}>
                    <ZoomOutIcon />
                  </IconButton>
                  <IconButton size="small" onClick={handlePlayPause}>
                    {isPlaying ? <PauseIcon /> : <PlayIcon />}
                  </IconButton>
                  <Tooltip title="Export as CSV">
                    <IconButton size="small" onClick={() => handleExport('csv')}>
                      <DownloadIcon />
                    </IconButton>
                  </Tooltip>
                </Box>
              </Grid>
            </Grid>
          </Box>
        )}

        {/* Stats Summary */}
        {metricStats && (
          <Box sx={{ p: 2, backgroundColor: themeMui.palette.grey[50] }}>
            <Grid container spacing={2}>
              <Grid item xs={4}>
                <Box textAlign="center">
                  <Typography variant="h4" color="primary">
                    {metricStats.gpu.current.toFixed(0)}%
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    GPU Usage
                  </Typography>
                </Box>
              </Grid>
              <Grid item xs={4}>
                <Box textAlign="center">
                  <Typography variant="h4" color="secondary">
                    {(metricStats.memory.used / (1024 * 1024)).toFixed(1)}GB
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Memory Used
                  </Typography>
                </Box>
              </Grid>
              <Grid item xs={4}>
                <Box textAlign="center">
                  <Typography variant="h4" color="success.main">
                    {metricStats.memory.percentage.toFixed(0)}%
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Memory Usage
                  </Typography>
                </Box>
              </Grid>
            </Grid>
          </Box>
        )}

        {/* Main visualization */}
        <Box sx={{ flex: 1, position: 'relative' }}>
          <WebGLCanvas
            width={width}
            height={height - (showControls ? 160 : 0) - (metricStats ? 80 : 0)}
            onRender={handleWebGLRender}
            backgroundColor={theme === 'dark' ? [0.1, 0.1, 0.15, 1.0] : [0.95, 0.95, 0.98, 1.0]}
          />
          {isLoading && (
            <Box
              sx={{
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                backgroundColor: 'rgba(255, 255, 255, 0.7)',
              }}
            >
              <CircularProgress />
            </Box>
          )}
        </Box>

        {/* Error display */}
        {error && (
          <Box sx={{ p: 2 }}>
            <Alert severity="error">{error}</Alert>
          </Box>
        )}
      </CardContent>
    </Card>
  );
};

export default GPUMetricsVisualizer;