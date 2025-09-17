import React, { useState, useEffect, useRef } from 'react';
import {
  Box,
  Paper,
  Typography,
  Grid,
  Card,
  CardContent,
  Chip,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Divider,
  Alert,
  CircularProgress,
  Button,
  IconButton,
  Tooltip
} from '@mui/material';
import {
  Timeline as TimelineIcon,
  Psychology as PsychologyIcon,
  TrendingUp as TrendingUpIcon,
  Schedule as ScheduleIcon,
  Assessment as AssessmentIcon,
  Refresh as RefreshIcon
} from '@mui/icons-material';
import * as d3 from 'd3';
import { invoke } from '@tauri-apps/api/tauri';

interface PatternResult {
  pattern_id: string;
  pattern_type: string;
  confidence: number;
  strength: number;
  next_occurrence: string;
  associated_tasks: string[];
}

const PatternAnalysisView: React.FC = () => {
  const [patterns, setPatterns] = useState<PatternResult[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedPattern, setSelectedPattern] = useState<PatternResult | null>(null);

  // D3 chart refs
  const patternStrengthChartRef = useRef<SVGSVGElement>(null);
  const patternTimelineRef = useRef<SVGSVGElement>(null);

  const fetchPatterns = async () => {
    try {
      setLoading(true);
      setError(null);

      // Mock pattern data for demonstration
      const mockPatterns: PatternResult[] = [
        {
          pattern_id: 'daily_coding_morning',
          pattern_type: 'Daily',
          confidence: 0.89,
          strength: 0.76,
          next_occurrence: '2h 15m',
          associated_tasks: ['Completion', 'Analysis']
        },
        {
          pattern_id: 'weekend_refactoring',
          pattern_type: 'Weekly',
          confidence: 0.82,
          strength: 0.68,
          next_occurrence: '1d 4h',
          associated_tasks: ['Refactoring', 'Analysis']
        },
        {
          pattern_id: 'project_specific_patterns',
          pattern_type: 'ProjectSpecific',
          confidence: 0.95,
          strength: 0.91,
          next_occurrence: '30m',
          associated_tasks: ['Completion', 'Refactoring']
        },
        {
          pattern_id: 'task_sequence_debug_test',
          pattern_type: 'TaskSequence',
          confidence: 0.78,
          strength: 0.72,
          next_occurrence: '1h 45m',
          associated_tasks: ['Analysis', 'Refactoring']
        }
      ];

      setPatterns(mockPatterns);

      // Try to fetch real patterns
      const request = {
        user_id: 'user123',
        activities: [
          { activity_type: 'coding', timestamp: '2025-01-01T09:00:00Z', duration: 1800, model_task: 'completion' },
          { activity_type: 'debugging', timestamp: '2025-01-01T09:35:00Z', duration: 1200, model_task: 'analysis' },
          { activity_type: 'refactoring', timestamp: '2025-01-01T10:00:00Z', duration: 2400, model_task: 'refactoring' }
        ]
      };

      try {
        const realPatterns = await invoke('analyze_behavior_patterns', request);
        setPatterns(realPatterns as PatternResult[]);
      } catch (err) {
        // Fall back to mock data if real data unavailable
        console.log('Using mock pattern data:', err);
      }

    } catch (err) {
      setError(`Failed to fetch patterns: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchPatterns();
  }, []);

  useEffect(() => {
    if (patterns.length > 0) {
      renderCharts();
    }
  }, [patterns]);

  const renderCharts = () => {
    renderPatternStrengthChart();
    renderPatternTimelineChart();
  };

  const renderPatternStrengthChart = () => {
    if (!patternStrengthChartRef.current || patterns.length === 0) return;

    const svg = d3.select(patternStrengthChartRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 20, right: 30, bottom: 60, left: 60 };
    const width = 400 - margin.left - margin.right;
    const height = 250 - margin.top - margin.bottom;

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleBand()
      .domain(patterns.map(p => p.pattern_type))
      .range([0, width])
      .padding(0.1);

    const y = d3.scaleLinear()
      .domain([0, 1])
      .range([height, 0]);

    // Add X axis
    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x))
      .selectAll('text')
      .attr('transform', 'rotate(-45)')
      .style('text-anchor', 'end');

    // Add Y axis
    g.append('g')
      .call(d3.axisLeft(y));

    // Add bars for confidence
    g.selectAll('.confidence-bar')
      .data(patterns)
      .enter()
      .append('rect')
      .attr('class', 'confidence-bar')
      .attr('x', d => x(d.pattern_type) || 0)
      .attr('y', d => y(d.confidence))
      .attr('width', x.bandwidth() / 2)
      .attr('height', d => height - y(d.confidence))
      .attr('fill', '#1976d2');

    // Add bars for strength
    g.selectAll('.strength-bar')
      .data(patterns)
      .enter()
      .append('rect')
      .attr('class', 'strength-bar')
      .attr('x', d => (x(d.pattern_type) || 0) + x.bandwidth() / 2)
      .attr('y', d => y(d.strength))
      .attr('width', x.bandwidth() / 2)
      .attr('height', d => height - y(d.strength))
      .attr('fill', '#2e7d32');

    // Labels
    svg.append('text')
      .attr('x', width / 2 + margin.left)
      .attr('y', height + margin.top + 50)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Pattern Type');

    svg.append('text')
      .attr('transform', 'rotate(-90)')
      .attr('x', -(height / 2) - margin.top)
      .attr('y', 15)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Score');
  };

  const renderPatternTimelineChart = () => {
    if (!patternTimelineRef.current || patterns.length === 0) return;

    const svg = d3.select(patternTimelineRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 20, right: 30, bottom: 40, left: 60 };
    const width = 400 - margin.left - margin.right;
    const height = 250 - margin.top - margin.bottom;

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleBand()
      .domain(patterns.map(p => p.pattern_id))
      .range([0, width])
      .padding(0.2);

    const y = d3.scaleLinear()
      .domain([0, d3.max(patterns, d => parseFloat(d.next_occurrence.split(' ')[0])) as number])
      .range([height, 0]);

    // Add X axis
    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x))
      .selectAll('text')
      .attr('transform', 'rotate(-45)')
      .style('text-anchor', 'end');

    // Add Y axis
    g.append('g')
      .call(d3.axisLeft(y));

    // Add timeline bars
    g.selectAll('.timeline-bar')
      .data(patterns)
      .enter()
      .append('rect')
      .attr('class', 'timeline-bar')
      .attr('x', d => x(d.pattern_id) || 0)
      .attr('y', d => y(parseFloat(d.next_occurrence.split(' ')[0])))
      .attr('width', x.bandwidth())
      .attr('height', d => height - y(parseFloat(d.next_occurrence.split(' ')[0])))
      .attr('fill', '#f57c00');

    // Labels
    svg.append('text')
      .attr('x', width / 2 + margin.left)
      .attr('y', height + margin.top + 35)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Pattern ID');

    svg.append('text')
      .attr('transform', 'rotate(-90)')
      .attr('x', -(height / 2) - margin.top)
      .attr('y', 15)
      .attr('text-anchor', 'middle')
      .style('font-size', '12px')
      .text('Time Until Next (hours)');
  };

  const getPatternIcon = (patternType: string) => {
    switch (patternType) {
      case 'Daily':
        return <ScheduleIcon />;
      case 'Weekly':
        return <TimelineIcon />;
      case 'ProjectSpecific':
        return <PsychologyIcon />;
      case 'TaskSequence':
        return <AssessmentIcon />;
      default:
        return <TrendingUpIcon />;
    }
  };

  const getPatternColor = (confidence: number) => {
    if (confidence >= 0.8) return 'success';
    if (confidence >= 0.6) return 'warning';
    return 'error';
  };

  const getPatternTypeDescription = (patternType: string) => {
    switch (patternType) {
      case 'Daily':
        return 'Recurring daily patterns in user behavior';
      case 'Weekly':
        return 'Weekly cycles in activity patterns';
      case 'ProjectSpecific':
        return 'Patterns specific to current project context';
      case 'TaskSequence':
        return 'Sequential patterns in task execution';
      default:
        return 'General pattern recognition';
    }
  };

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="400px">
        <CircularProgress />
        <Typography variant="body1" sx={{ ml: 2 }}>
          Analyzing user patterns...
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
      <Box display="flex" justifyContent="space-between" alignItems="center" sx={{ mb: 3 }}>
        <Typography variant="h6">
          <PsychologyIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
          Pattern Recognition Analysis
        </Typography>

        <Button
          startIcon={<RefreshIcon />}
          onClick={fetchPatterns}
          disabled={loading}
        >
          Refresh Patterns
        </Button>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <Grid container spacing={3}>
        {/* Pattern List */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Detected Patterns
            </Typography>
            <Divider sx={{ mb: 2 }} />

            <List>
              {patterns.map((pattern) => (
                <ListItem
                  key={pattern.pattern_id}
                  button
                  selected={selectedPattern?.pattern_id === pattern.pattern_id}
                  onClick={() => setSelectedPattern(pattern)}
                  sx={{ mb: 1 }}
                >
                  <ListItemIcon>
                    {getPatternIcon(pattern.pattern_type)}
                  </ListItemIcon>
                  <ListItemText
                    primary={
                      <Box display="flex" alignItems="center" gap={1}>
                        <Typography variant="body1">
                          {pattern.pattern_type}
                        </Typography>
                        <Chip
                          label={`${(pattern.confidence * 100).toFixed(0)}%`}
                          size="small"
                          color={getPatternColor(pattern.confidence)}
                        />
                      </Box>
                    }
                    secondary={
                      <Box>
                        <Typography variant="body2" color="text.secondary">
                          Next occurrence: {pattern.next_occurrence}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          Strength: {(pattern.strength * 100).toFixed(1)}%
                        </Typography>
                      </Box>
                    }
                  />
                </ListItem>
              ))}
            </List>
          </Paper>
        </Grid>

        {/* Pattern Details */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Pattern Details
            </Typography>
            <Divider sx={{ mb: 2 }} />

            {selectedPattern ? (
              <Box>
                <Typography variant="h6" color="primary" gutterBottom>
                  {selectedPattern.pattern_type}
                </Typography>

                <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                  {getPatternTypeDescription(selectedPattern.pattern_type)}
                </Typography>

                <Grid container spacing={2} sx={{ mb: 3 }}>
                  <Grid item xs={6}>
                    <Typography variant="body2" color="text.secondary">
                      Confidence Score
                    </Typography>
                    <Typography variant="h5" color="primary">
                      {(selectedPattern.confidence * 100).toFixed(1)}%
                    </Typography>
                  </Grid>

                  <Grid item xs={6}>
                    <Typography variant="body2" color="text.secondary">
                      Pattern Strength
                    </Typography>
                    <Typography variant="h5" color="secondary">
                      {(selectedPattern.strength * 100).toFixed(1)}%
                    </Typography>
                  </Grid>
                </Grid>

                <Typography variant="body2" color="text.secondary" gutterBottom>
                  Associated Tasks:
                </Typography>
                <Box display="flex" gap={1} flexWrap="wrap" sx={{ mb: 2 }}>
                  {selectedPattern.associated_tasks.map((task, index) => (
                    <Chip
                      key={index}
                      label={task}
                      size="small"
                      variant="outlined"
                      color="primary"
                    />
                  ))}
                </Box>

                <Typography variant="body2" color="text.secondary">
                  Next predicted occurrence: <strong>{selectedPattern.next_occurrence}</strong>
                </Typography>
              </Box>
            ) : (
              <Typography variant="body2" color="text.secondary">
                Select a pattern from the list to view detailed information.
              </Typography>
            )}
          </Paper>
        </Grid>

        {/* Pattern Strength Chart */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Pattern Confidence vs Strength
              </Typography>

              <Box display="flex" gap={1} sx={{ mb: 2 }}>
                <Box display="flex" alignItems="center">
                  <Box sx={{ width: 12, height: 12, backgroundColor: '#1976d2', mr: 1 }} />
                  <Typography variant="body2">Confidence</Typography>
                </Box>
                <Box display="flex" alignItems="center">
                  <Box sx={{ width: 12, height: 12, backgroundColor: '#2e7d32', mr: 1 }} />
                  <Typography variant="body2">Strength</Typography>
                </Box>
              </Box>

              <svg
                ref={patternStrengthChartRef}
                width="420"
                height="290"
                style={{ border: '1px solid #e0e0e0', borderRadius: '4px' }}
              />
            </CardContent>
          </Card>
        </Grid>

        {/* Pattern Timeline Chart */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Pattern Timeline
              </Typography>

              <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                Time until next occurrence for each pattern
              </Typography>

              <svg
                ref={patternTimelineRef}
                width="420"
                height="290"
                style={{ border: '1px solid #e0e0e0', borderRadius: '4px' }}
              />
            </CardContent>
          </Card>
        </Grid>

        {/* Pattern Insights */}
        <Grid item xs={12}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              <AssessmentIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
              Pattern Analysis Insights
            </Typography>
            <Divider sx={{ mb: 2 }} />

            <Grid container spacing={2}>
              <Grid item xs={12} sm={6} md={3}>
                <Card variant="outlined">
                  <CardContent sx={{ textAlign: 'center' }}>
                    <Typography variant="h4" color="success.main">
                      {patterns.filter(p => p.confidence >= 0.8).length}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      High Confidence Patterns
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>

              <Grid item xs={12} sm={6} md={3}>
                <Card variant="outlined">
                  <CardContent sx={{ textAlign: 'center' }}>
                    <Typography variant="h4" color="primary">
                      {patterns.length}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Total Patterns Detected
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>

              <Grid item xs={12} sm={6} md={3}>
                <Card variant="outlined">
                  <CardContent sx={{ textAlign: 'center' }}>
                    <Typography variant="h4" color="secondary">
                      {(patterns.reduce((sum, p) => sum + p.strength, 0) / patterns.length * 100).toFixed(1)}%
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Average Pattern Strength
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>

              <Grid item xs={12} sm={6} md={3}>
                <Card variant="outlined">
                  <CardContent sx={{ textAlign: 'center' }}>
                    <Typography variant="h4" color="warning.main">
                      {patterns.filter(p => parseFloat(p.next_occurrence.split(' ')[0]) < 2).length}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Patterns Due Soon
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
};

export default PatternAnalysisView;