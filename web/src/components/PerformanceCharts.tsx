import React, { useState, useEffect, useRef } from 'react';
import {
  Box,
  Paper,
  Typography,
  Grid,
  Card,
  CardContent,
  Tabs,
  Tab,
  Chip,
  Alert
} from '@mui/material';
import {
  Timeline as TimelineIcon,
  TrendingUp as TrendingUpIcon,
  Memory as MemoryIcon,
  Speed as SpeedIcon
} from '@mui/icons-material';
import * as d3 from 'd3';
import { invoke } from '@tauri-apps/api/tauri';

interface ChartData {
  timestamps: string[];
  values: number[];
  labels?: string[];
}

interface PerformanceData {
  latency_data: ChartData;
  throughput_data: ChartData;
  memory_data: ChartData;
  cpu_data: ChartData;
  error_rate_data: ChartData;
}

const PerformanceCharts: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [performanceData, setPerformanceData] = useState<PerformanceData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Refs for D3 charts
  const latencyChartRef = useRef<SVGSVGElement>(null);
  const throughputChartRef = useRef<SVGSVGElement>(null);
  const memoryChartRef = useRef<SVGSVGElement>(null);
  const cpuChartRef = useRef<SVGSVGElement>(null);

  const fetchPerformanceData = async () => {
    try {
      setLoading(true);
      // Generate mock data for demonstration
      const mockData: PerformanceData = {
        latency_data: {
          timestamps: Array.from({ length: 50 }, (_, i) => `2025-01-${String(i + 1).padStart(2, '0')}`),
          values: Array.from({ length: 50 }, () => Math.random() * 100 + 20)
        },
        throughput_data: {
          timestamps: Array.from({ length: 50 }, (_, i) => `2025-01-${String(i + 1).padStart(2, '0')}`),
          values: Array.from({ length: 50 }, () => Math.random() * 500 + 100)
        },
        memory_data: {
          timestamps: Array.from({ length: 50 }, (_, i) => `2025-01-${String(i + 1).padStart(2, '0')}`),
          values: Array.from({ length: 50 }, () => Math.random() * 200 + 300)
        },
        cpu_data: {
          timestamps: Array.from({ length: 50 }, (_, i) => `2025-01-${String(i + 1).padStart(2, '0')}`),
          values: Array.from({ length: 50 }, () => Math.random() * 30 + 10)
        },
        error_rate_data: {
          timestamps: Array.from({ length: 50 }, (_, i) => `2025-01-${String(i + 1).padStart(2, '0')}`),
          values: Array.from({ length: 50 }, () => Math.random() * 5)
        }
      };

      setPerformanceData(mockData);
      setError(null);
    } catch (err) {
      setError(`Failed to fetch performance data: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchPerformanceData();

    // Auto-refresh every 30 seconds
    const interval = setInterval(fetchPerformanceData, 30000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (performanceData) {
      renderCharts();
    }
  }, [performanceData, activeTab]);

  const renderCharts = () => {
    if (!performanceData) return;

    switch (activeTab) {
      case 0:
        renderLatencyChart();
        break;
      case 1:
        renderThroughputChart();
        break;
      case 2:
        renderMemoryChart();
        break;
      case 3:
        renderCpuChart();
        break;
    }
  };

  const renderLatencyChart = () => {
    if (!latencyChartRef.current || !performanceData) return;

    const svg = d3.select(latencyChartRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 20, right: 30, bottom: 40, left: 50 };
    const width = 600 - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleTime()
      .domain(d3.extent(performanceData.latency_data.timestamps, d => new Date(d)) as [Date, Date])
      .range([0, width]);

    const y = d3.scaleLinear()
      .domain([0, d3.max(performanceData.latency_data.values) as number])
      .range([height, 0]);

    // Add X axis
    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x).ticks(5));

    // Add Y axis
    g.append('g')
      .call(d3.axisLeft(y));

    // Add line
    const line = d3.line<number>()
      .x((_, i) => x(new Date(performanceData.latency_data.timestamps[i])))
      .y(d => y(d));

    g.append('path')
      .datum(performanceData.latency_data.values)
      .attr('fill', 'none')
      .attr('stroke', '#1976d2')
      .attr('stroke-width', 2)
      .attr('d', line);

    // Add area
    const area = d3.area<number>()
      .x((_, i) => x(new Date(performanceData.latency_data.timestamps[i])))
      .y0(height)
      .y1(d => y(d));

    g.append('path')
      .datum(performanceData.latency_data.values)
      .attr('fill', 'rgba(25, 118, 210, 0.1)')
      .attr('d', area);

    // Labels
    svg.append('text')
      .attr('x', width / 2 + margin.left)
      .attr('y', height + margin.top + 30)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Time');

    svg.append('text')
      .attr('transform', 'rotate(-90)')
      .attr('x', -(height / 2) - margin.top)
      .attr('y', 15)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Latency (ms)');
  };

  const renderThroughputChart = () => {
    if (!throughputChartRef.current || !performanceData) return;

    const svg = d3.select(throughputChartRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 20, right: 30, bottom: 40, left: 50 };
    const width = 600 - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleTime()
      .domain(d3.extent(performanceData.throughput_data.timestamps, d => new Date(d)) as [Date, Date])
      .range([0, width]);

    const y = d3.scaleLinear()
      .domain([0, d3.max(performanceData.throughput_data.values) as number])
      .range([height, 0]);

    // Add axes
    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x).ticks(5));

    g.append('g')
      .call(d3.axisLeft(y));

    // Add bars
    g.selectAll('rect')
      .data(performanceData.throughput_data.values)
      .enter()
      .append('rect')
      .attr('x', (_, i) => x(new Date(performanceData.throughput_data.timestamps[i])) - 2)
      .attr('y', d => y(d))
      .attr('width', 4)
      .attr('height', d => height - y(d))
      .attr('fill', '#2e7d32');

    // Labels
    svg.append('text')
      .attr('x', width / 2 + margin.left)
      .attr('y', height + margin.top + 30)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Time');

    svg.append('text')
      .attr('transform', 'rotate(-90)')
      .attr('x', -(height / 2) - margin.top)
      .attr('y', 15)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Requests/sec');
  };

  const renderMemoryChart = () => {
    if (!memoryChartRef.current || !performanceData) return;

    const svg = d3.select(memoryChartRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 20, right: 30, bottom: 40, left: 50 };
    const width = 600 - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleTime()
      .domain(d3.extent(performanceData.memory_data.timestamps, d => new Date(d)) as [Date, Date])
      .range([0, width]);

    const y = d3.scaleLinear()
      .domain([0, d3.max(performanceData.memory_data.values) as number])
      .range([height, 0]);

    // Add axes
    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x).ticks(5));

    g.append('g')
      .call(d3.axisLeft(y));

    // Add area chart
    const area = d3.area<number>()
      .x((_, i) => x(new Date(performanceData.memory_data.timestamps[i])))
      .y0(height)
      .y1(d => y(d));

    g.append('path')
      .datum(performanceData.memory_data.values)
      .attr('fill', 'rgba(245, 124, 0, 0.3)')
      .attr('stroke', '#f57c00')
      .attr('stroke-width', 2)
      .attr('d', area);

    // Labels
    svg.append('text')
      .attr('x', width / 2 + margin.left)
      .attr('y', height + margin.top + 30)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Time');

    svg.append('text')
      .attr('transform', 'rotate(-90)')
      .attr('x', -(height / 2) - margin.top)
      .attr('y', 15)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Memory (MB)');
  };

  const renderCpuChart = () => {
    if (!cpuChartRef.current || !performanceData) return;

    const svg = d3.select(cpuChartRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 20, right: 30, bottom: 40, left: 50 };
    const width = 600 - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleTime()
      .domain(d3.extent(performanceData.cpu_data.timestamps, d => new Date(d)) as [Date, Date])
      .range([0, width]);

    const y = d3.scaleLinear()
      .domain([0, d3.max(performanceData.cpu_data.values) as number])
      .range([height, 0]);

    // Add axes
    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x).ticks(5));

    g.append('g')
      .call(d3.axisLeft(y));

    // Add line
    const line = d3.line<number>()
      .x((_, i) => x(new Date(performanceData.cpu_data.timestamps[i])))
      .y(d => y(d));

    g.append('path')
      .datum(performanceData.cpu_data.values)
      .attr('fill', 'none')
      .attr('stroke', '#d32f2f')
      .attr('stroke-width', 2)
      .attr('d', line);

    // Add area
    const area = d3.area<number>()
      .x((_, i) => x(new Date(performanceData.cpu_data.timestamps[i])))
      .y0(height)
      .y1(d => y(d));

    g.append('path')
      .datum(performanceData.cpu_data.values)
      .attr('fill', 'rgba(211, 47, 47, 0.1)')
      .attr('d', area);

    // Labels
    svg.append('text')
      .attr('x', width / 2 + margin.left)
      .attr('y', height + margin.top + 30)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Time');

    svg.append('text')
      .attr('transform', 'rotate(-90)')
      .attr('x', -(height / 2) - margin.top)
      .attr('y', 15)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('CPU Usage (%)');
  };

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  return (
    <Box>
      <Typography variant="h6" gutterBottom sx={{ mb: 3 }}>
        <TimelineIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
        Performance Analytics
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <Paper sx={{ width: '100%' }}>
        <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tabs value={activeTab} onChange={handleTabChange} aria-label="performance chart tabs">
            <Tab
              label="Latency"
              icon={<SpeedIcon />}
              iconPosition="start"
            />
            <Tab
              label="Throughput"
              icon={<TrendingUpIcon />}
              iconPosition="start"
            />
            <Tab
              label="Memory"
              icon={<MemoryIcon />}
              iconPosition="start"
            />
            <Tab
              label="CPU"
              icon={<TimelineIcon />}
              iconPosition="start"
            />
          </Tabs>
        </Box>

        <Box sx={{ p: 3 }}>
          <Grid container spacing={3}>
            <Grid item xs={12}>
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    {activeTab === 0 && 'Response Time Trends'}
                    {activeTab === 1 && 'Request Throughput'}
                    {activeTab === 2 && 'Memory Usage Patterns'}
                    {activeTab === 3 && 'CPU Utilization'}
                  </Typography>

                  <Box display="flex" gap={2} sx={{ mb: 2 }}>
                    <Chip
                      label="Real-time"
                      color="success"
                      size="small"
                    />
                    <Chip
                      label="Auto-refresh: 30s"
                      variant="outlined"
                      size="small"
                    />
                  </Box>

                  <Box sx={{ display: 'flex', justifyContent: 'center' }}>
                    <svg
                      ref={
                        activeTab === 0 ? latencyChartRef :
                        activeTab === 1 ? throughputChartRef :
                        activeTab === 2 ? memoryChartRef :
                        cpuChartRef
                      }
                      width="650"
                      height="350"
                      style={{ border: '1px solid #e0e0e0', borderRadius: '4px' }}
                    />
                  </Box>
                </CardContent>
              </Card>
            </Grid>

            {/* Performance Summary */}
            <Grid item xs={12}>
              <Grid container spacing={2}>
                <Grid item xs={12} sm={6} md={3}>
                  <Card>
                    <CardContent sx={{ textAlign: 'center' }}>
                      <Typography variant="h4" color="primary">
                        45ms
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Avg Latency
                      </Typography>
                      <Chip
                        label="↓ 12%"
                        color="success"
                        size="small"
                        sx={{ mt: 1 }}
                      />
                    </CardContent>
                  </Card>
                </Grid>

                <Grid item xs={12} sm={6} md={3}>
                  <Card>
                    <CardContent sx={{ textAlign: 'center' }}>
                      <Typography variant="h4" color="success.main">
                        285
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Req/sec
                      </Typography>
                      <Chip
                        label="↑ 8%"
                        color="success"
                        size="small"
                        sx={{ mt: 1 }}
                      />
                    </CardContent>
                  </Card>
                </Grid>

                <Grid item xs={12} sm={6} md={3}>
                  <Card>
                    <CardContent sx={{ textAlign: 'center' }}>
                      <Typography variant="h4" color="warning.main">
                        512MB
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Peak Memory
                      </Typography>
                      <Chip
                        label="Stable"
                        color="default"
                        size="small"
                        sx={{ mt: 1 }}
                      />
                    </CardContent>
                  </Card>
                </Grid>

                <Grid item xs={12} sm={6} md={3}>
                  <Card>
                    <CardContent sx={{ textAlign: 'center' }}>
                      <Typography variant="h4" color="secondary">
                        23%
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        CPU Usage
                      </Typography>
                      <Chip
                        label="Optimal"
                        color="success"
                        size="small"
                        sx={{ mt: 1 }}
                      />
                    </CardContent>
                  </Card>
                </Grid>
              </Grid>
            </Grid>
          </Grid>
        </Box>
      </Paper>
    </Box>
  );
};

export default PerformanceCharts;