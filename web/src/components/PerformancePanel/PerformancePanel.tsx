import React, { useState } from 'react';
import {
  Box,
  Button,
  CircularProgress,
  Typography,
  Paper,
  Tabs,
  Tab,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Collapse,
  Alert,
  AlertTitle,
  LinearProgress,
  FormControlLabel,
  Switch,
  Chip,
} from '@mui/material';
import Grid from '@mui/material/Grid';
import {
  Speed as PerformanceIcon,
  Warning as WarningIcon,
  Info as InfoIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
} from '@mui/icons-material';
import { OptimizationSuggestion } from '../../types/performance';
import usePerformanceAnalysis from '../../hooks/usePerformanceAnalysis';

interface PerformancePanelProps {
  projectPath: string;
}

const PerformancePanel: React.FC<PerformancePanelProps> = ({ projectPath }) => {
  const [suggestions, setSuggestions] = useState<OptimizationSuggestion[]>([]);
  const [activeTab, setActiveTab] = useState(0);
  const [expandedCrates, setExpandedCrates] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<string | null>(null);
  const [releaseMode, setReleaseMode] = useState(false);
  const [incremental, setIncremental] = useState(true);
  
  const { 
    isLoading, 
    error: analysisError, 
    metrics, 
    analyzePerformance 
  } = usePerformanceAnalysis(projectPath);

  const handleAnalyze = async () => {
    if (!projectPath) {
      setError('No project path provided');
      return;
    }
    try {
      const result = await analyzePerformance(projectPath, releaseMode, incremental);
      setError(null);
      
      // Generate optimization suggestions
      const suggestionsList: OptimizationSuggestion[] = [];
      
      // Check for long build times (threshold: 10 seconds)
      if (result?.total_time && result.total_time > 10_000) {
        suggestionsList.push({
          type: 'build_time',
          message: 'Long build time detected',
          details: `Total build time is ${(result.total_time / 1000).toFixed(2)}s. Consider optimizing your build.`,
          severity: 'warning'
        });
      }
      
      // Check for crates with long build times
      if (result?.crates) {
        const totalTime = result.total_time || 1;
        for (const [name, crate] of Object.entries(result.crates)) {
          if (crate.build_time && (crate.build_time / totalTime) > 0.3) {
            suggestionsList.push({
              type: 'crate_build_time',
              message: `Crate '${name}' is taking ${((crate.build_time / totalTime) * 100).toFixed(1)}% of build time`,
              details: 'Consider optimizing this crate or its dependencies.',
              severity: 'warning'
            });
          }
        }
      }
      
      setSuggestions(suggestionsList);
    } catch (err) {
      console.error('Error analyzing performance:', err);
      setError('Failed to analyze performance');
    }
  };

  const handleTabChange = (_: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const toggleCrateExpansion = (crateName: string) => {
    setExpandedCrates(prev => ({
      ...prev,
      [crateName]: !prev[crateName]
    }));
  };

  const formatTime = (ms?: number): string => {
    if (typeof ms !== 'number') return '0ms';
    if (ms < 1000) return `${Math.round(ms)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  const getTotalBuildTime = (): number => {
    if (!metrics?.crates) return 0;
    return Object.values(metrics.crates).reduce(
      (total, crate) => total + (crate?.build_time || 0), 
      0
    );
  };

  const getCrateBuildTime = (crate: { build_time?: number } | undefined): number => {
    return crate?.build_time || 0;
  };

  const getCratePercentage = (crate: { build_time?: number } | undefined): number => {
    if (!crate) return 0;
    const total = getTotalBuildTime();
    if (total === 0) return 0;
    return (getCrateBuildTime(crate) / total) * 100;
  };

  return (
    <Box sx={{ p: 2, height: '100%', display: 'flex', flexDirection: 'column', gap: 2 }}>
      <Paper sx={{ p: 2, mb: 2 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="h6" gutterBottom>
            Performance Analysis
          </Typography>
          <Box display="flex" gap={1} alignItems="center">
            <FormControlLabel
              control={
                <Switch
                  checked={releaseMode}
                  onChange={(_, checked) => setReleaseMode(checked)}
                  color="primary"
                />
              }
              label="Release Mode"
              labelPlacement="start"
            />
            <FormControlLabel
              control={
                <Switch
                  checked={incremental}
                  onChange={(_, checked) => setIncremental(checked)}
                  color="primary"
                />
              }
              label="Incremental"
              labelPlacement="start"
            />
            <Button
              variant="contained"
              color="primary"
              onClick={handleAnalyze}
              disabled={isLoading}
              startIcon={isLoading ? <CircularProgress size={20} /> : <PerformanceIcon />}
            >
              {isLoading ? 'Analyzing...' : 'Analyze Performance'}
            </Button>
          </Box>
        </Box>

        {analysisError && (
          <Alert severity="error" sx={{ mb: 2 }}>
            <AlertTitle>Analysis Error</AlertTitle>
            {analysisError}
          </Alert>
        )}

        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        {isLoading && <LinearProgress />}
      </Paper>

      <Tabs value={activeTab} onChange={handleTabChange} sx={{ mb: 2 }}>
        <Tab label="Summary" />
        <Tab label="Crates" />
        <Tab label="Dependencies" />
      </Tabs>

      {activeTab === 0 && (
        <Box>
          <Typography variant="h6" gutterBottom>
            Build Summary
          </Typography>
          {metrics ? (
            <Grid container spacing={2}>
              <Grid size={{ xs: 12, sm: 6, md: 4 }}>
                <Paper sx={{ p: 2, height: '100%' }}>
                  <Typography variant="subtitle2" color="text.secondary">Total Build Time</Typography>
                  <Typography variant="h4">{formatTime(metrics?.total_time ?? getTotalBuildTime())}</Typography>
                </Paper>
              </Grid>
              <Grid size={{ xs: 12, sm: 6, md: 4 }}>
                <Paper sx={{ p: 2, height: '100%' }}>
                  <Typography variant="subtitle2" color="text.secondary">Crates Built</Typography>
                  <Typography variant="h4">{Object.keys(metrics.crates || {}).length}</Typography>
                </Paper>
              </Grid>
              <Grid size={{ xs: 12, sm: 6, md: 4 }}>
                <Paper sx={{ p: 2, height: '100%' }}>
                  <Typography variant="subtitle2" color="text.secondary">Dependencies</Typography>
                  <Typography variant="h4">{Object.keys(metrics.dependencies || {}).length}</Typography>
                </Paper>
              </Grid>
            </Grid>
          ) : (
            <Box sx={{ textAlign: 'center', p: 4 }}>
              <InfoIcon color="action" sx={{ fontSize: 48, mb: 2 }} />
              <Typography variant="body1" color="text.secondary">
                {isLoading ? 'Analyzing build performance...' : 'Run performance analysis to view build summary'}
              </Typography>
            </Box>
          )}

          {suggestions.length > 0 && (
            <Box sx={{ mt: 4 }}>
              <Typography variant="h6" gutterBottom>
                Optimization Suggestions
              </Typography>
              <List dense>
                {suggestions.map((suggestion, index) => (
                  <ListItem key={index}>
                    <ListItemIcon>
                      {suggestion.severity === 'error' ? (
                        <WarningIcon color="error" />
                      ) : suggestion.severity === 'warning' ? (
                        <WarningIcon color="warning" />
                      ) : (
                        <InfoIcon color="info" />
                      )}
                    </ListItemIcon>
                    <ListItemText
                      primary={suggestion.message}
                      secondary={suggestion.details}
                    />
                  </ListItem>
                ))}
              </List>
            </Box>
          )}
        </Box>
      )}

      {activeTab === 1 && (
        <Box>
          <Typography variant="h6" gutterBottom>
            Crate Metrics
          </Typography>
          {metrics?.crates ? (
            <List>
              {Object.entries(metrics.crates)
                .sort(([, a], [, b]) => (b?.build_time || 0) - (a?.build_time || 0))
                .map(([name, crateMetrics]) => (
                  <React.Fragment key={name}>
                    <ListItem
                      onClick={() => toggleCrateExpansion(name)}
                      sx={{
                        bgcolor: expandedCrates[name] ? 'action.hover' : 'transparent',
                        cursor: 'pointer',
                        '&:hover': {
                          bgcolor: 'action.hover'
                        },
                        borderLeftWidth: 3,
                        borderLeftStyle: 'solid',
                        borderLeftColor: getCratePercentage(crateMetrics) > 20 ? 'error.main' : 'primary.main',
                        mb: 1,
                        borderRadius: 1
                      }}
                    >
                      <ListItemText
                        primary={
                          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <Typography variant="subtitle1">{name}</Typography>
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                              <Chip 
                                label={`${getCratePercentage(crateMetrics).toFixed(1)}%`} 
                                size="small" 
                                color={getCratePercentage(crateMetrics) > 20 ? 'error' : 'default'}
                                variant="outlined"
                              />
                              <Typography variant="body2" color="text.secondary">
                                {formatTime(crateMetrics?.build_time)}
                              </Typography>
                            </Box>
                          </Box>
                        }
                        secondary={
                          <Box sx={{ mt: 1 }}>
                            <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
                              <Chip 
                                label={`${crateMetrics?.codegen_units || 0} codegen units`} 
                                size="small" 
                                variant="outlined"
                              />
                              <Chip 
                                label={`${crateMetrics?.dependencies?.length || 0} deps`} 
                                size="small" 
                                variant="outlined"
                              />
                              {crateMetrics?.incremental && (
                                <Chip 
                                  label="Incremental" 
                                  size="small" 
                                  color="success"
                                  variant="outlined"
                                />
                              )}
                            </Box>
                            {expandedCrates[name] && crateMetrics?.features?.length > 0 && (
                              <Box sx={{ mt: 1 }}>
                                <Typography variant="caption" color="text.secondary">
                                  Features: {crateMetrics.features.join(', ')}
                                </Typography>
                              </Box>
                            )}
                          </Box>
                        }
                      />
                      {expandedCrates[name] ? <ExpandLessIcon /> : <ExpandMoreIcon />}
                    </ListItem>
                    <Collapse in={expandedCrates[name]} timeout="auto" unmountOnExit>
                      <Box sx={{ pl: 4, pr: 2, pb: 1, mt: 1 }}>
                        <Grid container spacing={2}>
                          <Grid size={{ xs: 12, sm: 6 }}>
                          <Paper sx={{ p: 2, height: '100%' }}>
                          <Typography variant="subtitle2" gutterBottom>Build Metrics</Typography>
                              <List dense disablePadding>
                                <ListItem>
                                  <ListItemText
                                    primary="Build Time"
                                    secondary={formatTime(crateMetrics?.build_time)}
                                  />
                                </ListItem>
                                <ListItem>
                                  <ListItemText
                                    primary="Codegen Time"
                                    secondary={formatTime(crateMetrics?.codegen_time)}
                                  />
                                </ListItem>
                                <ListItem>
                                  <ListItemText
                                    primary="Codegen Units"
                                    secondary={crateMetrics?.codegen_units || 'N/A'}
                                  />
                                </ListItem>
                              </List>
                            </Paper>
                          </Grid>
                          {crateMetrics?.dependencies && crateMetrics.dependencies.length > 0 && (
                            <Grid size={{ xs: 12, sm: 6 }}>
                            <Paper sx={{ p: 2, height: '100%' }}>
                            <Typography variant="subtitle2" gutterBottom>
                            Dependencies ({crateMetrics.dependencies.length})
                                </Typography>
                                <Box sx={{ maxHeight: 150, overflow: 'auto' }}>
                                  <List dense disablePadding>
                                    {crateMetrics.dependencies.map((dep: string, i: number) => (
                                      <ListItem key={i} sx={{ py: 0.5 }}>
                                        <ListItemText
                                          primary={dep}
                                          primaryTypographyProps={{ variant: 'body2' }}
                                        />
                                      </ListItem>
                                    ))}
                                  </List>
                                </Box>
                              </Paper>
                            </Grid>
                          )}
                        </Grid>
                      </Box>
                    </Collapse>
                  </React.Fragment>
                ))}
            </List>
          ) : (
            <Box sx={{ textAlign: 'center', p: 4 }}>
              <InfoIcon color="action" sx={{ fontSize: 48, mb: 2 }} />
              <Typography variant="body1" color="text.secondary">
                {isLoading ? 'Analyzing build performance...' : 'Run performance analysis to view crate metrics'}
              </Typography>
            </Box>
          )}
        </Box>
      )}

      {activeTab === 2 && (
        <Box>
          <Typography variant="h6" gutterBottom>
            Dependencies
          </Typography>
          {metrics?.dependencies ? (
            <Box>
              <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                {Object.keys(metrics.dependencies).length} dependencies found
              </Typography>
              <List dense>
                {Object.entries(metrics.dependencies)
                  .sort(([, a], [, b]) => (b as number) - (a as number))
                  .map(([name, time]) => (
                    <ListItem key={name} divider>
                      <ListItemText
                        primary={
                          <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                            <Typography variant="body2">{name}</Typography>
                            <Typography variant="body2" color="text.secondary">
                              {formatTime(time as number)}
                            </Typography>
                          </Box>
                        }
                      />
                    </ListItem>
                  ))}
              </List>
            </Box>
          ) : (
            <Box sx={{ textAlign: 'center', p: 4 }}>
              <InfoIcon color="action" sx={{ fontSize: 48, mb: 2 }} />
              <Typography variant="body1" color="text.secondary">
                {isLoading ? 'Analyzing build performance...' : 'Run performance analysis to view dependency metrics'}
              </Typography>
            </Box>
          )}
        </Box>
      )}
    </Box>
  );
};

export default PerformancePanel;
