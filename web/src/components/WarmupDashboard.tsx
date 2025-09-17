import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  Paper,
  Typography,
  Card,
  CardContent,
  Chip,
  LinearProgress,
  Alert,
  Tabs,
  Tab,
  Button,
  IconButton
} from '@mui/material';
import RefreshIcon from '@mui/icons-material/Refresh';
import { invoke } from '@tauri-apps/api/tauri';
import PredictionMetrics from './PredictionMetrics';
import PerformanceCharts from './PerformanceCharts';
import ModelWarmupControls from './ModelWarmupControls';
import PatternAnalysisView from './PatternAnalysisView';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`warmup-tabpanel-${index}`}
      aria-labelledby={`warmup-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

const WarmupDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [systemStatus, setSystemStatus] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date>(new Date());

  const fetchSystemStatus = async () => {
    try {
      setLoading(true);
      const status = await invoke('get_system_status');
      setSystemStatus(status);
      setLastUpdated(new Date());
      setError(null);
    } catch (err) {
      setError(`Failed to fetch system status: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSystemStatus();

    // Auto-refresh every 30 seconds
    const interval = setInterval(fetchSystemStatus, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const handleRefresh = () => {
    fetchSystemStatus();
  };

  const getStatusColor = (status: string) => {
    switch (status?.toLowerCase()) {
      case 'healthy':
        return 'success';
      case 'degraded':
        return 'warning';
      case 'unhealthy':
        return 'error';
      default:
        return 'default';
    }
  };

  return (
    <Box sx={{ width: '100%', height: '100%' }}>
      {/* Header */}
      <Paper sx={{ p: 2, mb: 2 }}>
        <Box display="flex" justifyContent="space-between" alignItems="center">
          <Box>
            <Typography variant="h5" component="h1" gutterBottom>
              Model Warmup Prediction System
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Real-time monitoring and control of AI model warmup operations
            </Typography>
          </Box>

          <Box display="flex" alignItems="center" gap={2}>
            {systemStatus && (
              <Chip
                label={`System: ${systemStatus.system_health}`}
                color={getStatusColor(systemStatus.system_health)}
                size="small"
              />
            )}

            <Typography variant="caption" color="text.secondary">
              Last updated: {lastUpdated.toLocaleTimeString()}
            </Typography>

            <IconButton onClick={handleRefresh} disabled={loading}>
              <RefreshIcon />
            </IconButton>
          </Box>
        </Box>

        {loading && <LinearProgress sx={{ mt: 1 }} />}

        {error && (
          <Alert severity="error" sx={{ mt: 2 }}>
            {error}
          </Alert>
        )}
      </Paper>

      {/* Service Status Cards */}
      {systemStatus && (
        <Grid container spacing={2} sx={{ mb: 2 }}>
          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Typography variant="h6" color="primary">
                  Warmup Predictor
                </Typography>
                <Chip
                  label={systemStatus.warmup_predictor_active ? 'Active' : 'Inactive'}
                  color={systemStatus.warmup_predictor_active ? 'success' : 'error'}
                  size="small"
                  sx={{ mt: 1 }}
                />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Typography variant="h6" color="primary">
                  Pattern Analyzer
                </Typography>
                <Chip
                  label={systemStatus.pattern_analyzer_active ? 'Active' : 'Inactive'}
                  color={systemStatus.pattern_analyzer_active ? 'success' : 'error'}
                  size="small"
                  sx={{ mt: 1 }}
                />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Typography variant="h6" color="primary">
                  ML Trainer
                </Typography>
                <Chip
                  label={systemStatus.ml_trainer_active ? 'Active' : 'Inactive'}
                  color={systemStatus.ml_trainer_active ? 'success' : 'error'}
                  size="small"
                  sx={{ mt: 1 }}
                />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Typography variant="h6" color="primary">
                  Benchmark Tool
                </Typography>
                <Chip
                  label={systemStatus.benchmarker_active ? 'Active' : 'Inactive'}
                  color={systemStatus.benchmarker_active ? 'success' : 'error'}
                  size="small"
                  sx={{ mt: 1 }}
                />
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}

      {/* Main Content Tabs */}
      <Paper sx={{ width: '100%' }}>
        <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tabs value={activeTab} onChange={handleTabChange} aria-label="warmup dashboard tabs">
            <Tab label="Prediction Metrics" />
            <Tab label="Performance Charts" />
            <Tab label="ML Model Control" />
            <Tab label="Pattern Analysis" id="warmup-tab-3" aria-controls="warmup-tabpanel-3" />
          </Tabs>
        </Box>

        <TabPanel value={activeTab} index={0}>
          <PredictionMetrics />
        </TabPanel>

        <TabPanel value={activeTab} index={1}>
          <PerformanceCharts />
        </TabPanel>

        <TabPanel value={activeTab} index={2}>
          <ModelWarmupControls onRefresh={handleRefresh} />
        </TabPanel>

        <TabPanel value={activeTab} index={3}>
          <PatternAnalysisView />
        </TabPanel>
      </Paper>
    </Box>
  );
};

export default WarmupDashboard;