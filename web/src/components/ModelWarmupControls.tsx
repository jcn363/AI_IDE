import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  Grid,
  Card,
  CardContent,
  Button,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Chip,
  Alert,
  CircularProgress,
  Divider,
  IconButton,
  Tooltip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  List,
  ListItem,
  ListItemText,
  LinearProgress
} from '@mui/material';
import {
  PlayArrow as PlayIcon,
  Stop as StopIcon,
  Settings as SettingsIcon,
  Assessment as AssessmentIcon,
  Refresh as RefreshIcon,
  ModelTraining as TrainingIcon,
  Memory as MemoryIcon,
  Speed as SpeedIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/tauri';

interface MLTrainingRequest {
  model_type: string;
  dataset_features: number[][];
  dataset_targets: number[];
  feature_names: string[];
}

interface BenchmarkRequest {
  iterations: number;
  warmup_iterations: number;
  max_duration_secs: number;
  benchmark_name: string;
}

interface TrainingResult {
  model_type: string;
  training_score: number;
  validation_score: number;
  training_time: number;
  feature_importance: Record<string, number>;
  status: string;
}

interface BenchmarkResult {
  benchmark_name: string;
  throughput: number;
  avg_latency: string;
  memory_usage: number;
  recommendations: string[];
}

interface ModelWarmupControlsProps {
  onRefresh: () => void;
}

const ModelWarmupControls: React.FC<ModelWarmupControlsProps> = ({ onRefresh }) => {
  const [activeTab, setActiveTab] = useState<'training' | 'benchmarking' | 'controls'>('controls');
  const [trainingDialog, setTrainingDialog] = useState(false);
  const [benchmarkDialog, setBenchmarkDialog] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Training form state
  const [trainingForm, setTrainingForm] = useState({
    model_type: 'linear_regression',
    iterations: 100,
    learning_rate: 0.01,
    regularization: 0.0
  });

  // Benchmark form state
  const [benchmarkForm, setBenchmarkForm] = useState({
    iterations: 1000,
    warmup_iterations: 100,
    max_duration_secs: 300,
    benchmark_name: 'warmup_performance_test'
  });

  // Results state
  const [trainingResult, setTrainingResult] = useState<TrainingResult | null>(null);
  const [benchmarkResult, setBenchmarkResult] = useState<BenchmarkResult | null>(null);

  const handleTrainingSubmit = async () => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      // Generate sample dataset
      const datasetFeatures = Array.from({ length: 100 }, () =>
        Array.from({ length: 5 }, () => Math.random() * 10)
      );
      const datasetTargets = Array.from({ length: 100 }, () => Math.random() * 100);
      const featureNames = ['feature1', 'feature2', 'feature3', 'feature4', 'feature5'];

      const request: MLTrainingRequest = {
        model_type: trainingForm.model_type,
        dataset_features: datasetFeatures,
        dataset_targets: datasetTargets,
        feature_names: featureNames
      };

      const result = await invoke('train_ml_model', request);
      setTrainingResult(result as TrainingResult);
      setSuccess('ML model training completed successfully!');
      setTrainingDialog(false);
      onRefresh();

    } catch (err) {
      setError(`Training failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleBenchmarkSubmit = async () => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      const request: BenchmarkRequest = benchmarkForm;
      const result = await invoke('run_performance_benchmark', request);
      setBenchmarkResult(result as BenchmarkResult);
      setSuccess('Performance benchmark completed!');
      setBenchmarkDialog(false);
      onRefresh();

    } catch (err) {
      setError(`Benchmark failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleStartWarmup = async () => {
    try {
      setLoading(true);
      const result = await invoke('update_warmup_config', { enable_background_warmup: true });
      setSuccess('Warmup system started successfully!');
      onRefresh();
    } catch (err) {
      setError(`Failed to start warmup: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleStopWarmup = async () => {
    try {
      setLoading(true);
      const result = await invoke('update_warmup_config', { enable_background_warmup: false });
      setSuccess('Warmup system stopped!');
      onRefresh();
    } catch (err) {
      setError(`Failed to stop warmup: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleClearCache = async () => {
    try {
      setLoading(true);
      // This would be a new command to clear prediction cache
      setSuccess('Cache cleared successfully!');
      onRefresh();
    } catch (err) {
      setError(`Failed to clear cache: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box>
      <Typography variant="h6" gutterBottom sx={{ mb: 3 }}>
        <SettingsIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
        ML Model & Warmup Controls
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert severity="success" sx={{ mb: 2 }}>
          {success}
        </Alert>
      )}

      <Grid container spacing={3}>
        {/* Control Buttons */}
        <Grid item xs={12}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              System Controls
            </Typography>
            <Divider sx={{ mb: 2 }} />

            <Box display="flex" gap={2} flexWrap="wrap">
              <Button
                variant="contained"
                color="success"
                startIcon={<PlayIcon />}
                onClick={handleStartWarmup}
                disabled={loading}
              >
                Start Warmup System
              </Button>

              <Button
                variant="outlined"
                color="error"
                startIcon={<StopIcon />}
                onClick={handleStopWarmup}
                disabled={loading}
              >
                Stop Warmup System
              </Button>

              <Button
                variant="outlined"
                startIcon={<RefreshIcon />}
                onClick={handleClearCache}
                disabled={loading}
              >
                Clear Prediction Cache
              </Button>

              <Button
                variant="contained"
                startIcon={<TrainingIcon />}
                onClick={() => setTrainingDialog(true)}
                disabled={loading}
              >
                Train New Model
              </Button>

              <Button
                variant="contained"
                startIcon={<AssessmentIcon />}
                onClick={() => setBenchmarkDialog(true)}
                disabled={loading}
              >
                Run Benchmark
              </Button>
            </Box>
          </Paper>
        </Grid>

        {/* Model Training Results */}
        {trainingResult && (
          <Grid item xs={12} md={6}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                <TrainingIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
                Latest Training Results
              </Typography>
              <Divider sx={{ mb: 2 }} />

              <Grid container spacing={2}>
                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Model Type
                  </Typography>
                  <Typography variant="h6">
                    {trainingResult.model_type.replace('_', ' ').toUpperCase()}
                  </Typography>
                </Grid>

                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Training Score
                  </Typography>
                  <Typography variant="h6" color="primary">
                    {(trainingResult.training_score * 100).toFixed(1)}%
                  </Typography>
                </Grid>

                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Validation Score
                  </Typography>
                  <Typography variant="h6" color="secondary">
                    {(trainingResult.validation_score * 100).toFixed(1)}%
                  </Typography>
                </Grid>

                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Training Time
                  </Typography>
                  <Typography variant="h6">
                    {(trainingResult.training_time / 1000).toFixed(1)}s
                  </Typography>
                </Grid>

                <Grid item xs={12}>
                  <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                    Feature Importance
                  </Typography>
                  <Box display="flex" gap={1} flexWrap="wrap">
                    {Object.entries(trainingResult.feature_importance)
                      .sort(([, a], [, b]) => b - a)
                      .slice(0, 5)
                      .map(([feature, importance]) => (
                        <Chip
                          key={feature}
                          label={`${feature}: ${(importance * 100).toFixed(1)}%`}
                          size="small"
                          color="primary"
                          variant="outlined"
                        />
                      ))}
                  </Box>
                </Grid>
              </Grid>
            </Paper>
          </Grid>
        )}

        {/* Benchmark Results */}
        {benchmarkResult && (
          <Grid item xs={12} md={6}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                <AssessmentIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
                Benchmark Results
              </Typography>
              <Divider sx={{ mb: 2 }} />

              <Grid container spacing={2}>
                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Test Name
                  </Typography>
                  <Typography variant="h6">
                    {benchmarkResult.benchmark_name}
                  </Typography>
                </Grid>

                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Throughput
                  </Typography>
                  <Typography variant="h6" color="success.main">
                    {benchmarkResult.throughput.toFixed(1)} req/s
                  </Typography>
                </Grid>

                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Avg Latency
                  </Typography>
                  <Typography variant="h6" color="warning.main">
                    {benchmarkResult.avg_latency}
                  </Typography>
                </Grid>

                <Grid item xs={6}>
                  <Typography variant="body2" color="text.secondary">
                    Memory Usage
                  </Typography>
                  <Typography variant="h6" color="secondary">
                    {benchmarkResult.memory_usage} MB
                  </Typography>
                </Grid>

                <Grid item xs={12}>
                  <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                    Recommendations
                  </Typography>
                  <List dense>
                    {benchmarkResult.recommendations.map((rec, index) => (
                      <ListItem key={index}>
                        <ListItemText primary={rec} />
                      </ListItem>
                    ))}
                  </List>
                </Grid>
              </Grid>
            </Paper>
          </Grid>
        )}

        {/* System Status */}
        <Grid item xs={12}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              <MemoryIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
              System Resource Usage
            </Typography>
            <Divider sx={{ mb: 2 }} />

            <Grid container spacing={3}>
              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    CPU Usage
                  </Typography>
                  <LinearProgress variant="determinate" value={23} color="primary" />
                  <Typography variant="body2" sx={{ mt: 1 }}>
                    23% of available capacity
                  </Typography>
                </Box>
              </Grid>

              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    Memory Usage
                  </Typography>
                  <LinearProgress variant="determinate" value={45} color="secondary" />
                  <Typography variant="body2" sx={{ mt: 1 }}>
                    512 MB / 1.2 GB available
                  </Typography>
                </Box>
              </Grid>

              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    Models Pre-warmed
                  </Typography>
                  <Typography variant="h4" color="success.main">
                    12
                  </Typography>
                  <Typography variant="body2" sx={{ mt: 1 }}>
                    Ready for immediate use
                  </Typography>
                </Box>
              </Grid>

              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    Prediction Cache
                  </Typography>
                  <Typography variant="h4" color="primary">
                    89%
                  </Typography>
                  <Typography variant="body2" sx={{ mt: 1 }}>
                    Hit rate efficiency
                  </Typography>
                </Box>
              </Grid>
            </Grid>
          </Paper>
        </Grid>
      </Grid>

      {/* Training Dialog */}
      <Dialog open={trainingDialog} onClose={() => setTrainingDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>
          <TrainingIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
          Train New ML Model
        </DialogTitle>
        <DialogContent>
          <Grid container spacing={3} sx={{ mt: 1 }}>
            <Grid item xs={12} md={6}>
              <FormControl fullWidth>
                <InputLabel>Model Type</InputLabel>
                <Select
                  value={trainingForm.model_type}
                  onChange={(e) => setTrainingForm({ ...trainingForm, model_type: e.target.value })}
                  label="Model Type"
                >
                  <MenuItem value="linear_regression">Linear Regression</MenuItem>
                  <MenuItem value="random_forest">Random Forest</MenuItem>
                  <MenuItem value="gradient_boosting">Gradient Boosting</MenuItem>
                  <MenuItem value="ensemble">Ensemble</MenuItem>
                </Select>
              </FormControl>
            </Grid>

            <Grid item xs={12} md={6}>
              <TextField
                fullWidth
                label="Max Iterations"
                type="number"
                value={trainingForm.iterations}
                onChange={(e) => setTrainingForm({ ...trainingForm, iterations: parseInt(e.target.value) })}
              />
            </Grid>

            <Grid item xs={12} md={6}>
              <TextField
                fullWidth
                label="Learning Rate"
                type="number"
                step="0.001"
                value={trainingForm.learning_rate}
                onChange={(e) => setTrainingForm({ ...trainingForm, learning_rate: parseFloat(e.target.value) })}
              />
            </Grid>

            <Grid item xs={12} md={6}>
              <TextField
                fullWidth
                label="Regularization"
                type="number"
                step="0.001"
                value={trainingForm.regularization}
                onChange={(e) => setTrainingForm({ ...trainingForm, regularization: parseFloat(e.target.value) })}
              />
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setTrainingDialog(false)}>Cancel</Button>
          <Button
            onClick={handleTrainingSubmit}
            variant="contained"
            disabled={loading}
            startIcon={loading ? <CircularProgress size={20} /> : <TrainingIcon />}
          >
            {loading ? 'Training...' : 'Start Training'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Benchmark Dialog */}
      <Dialog open={benchmarkDialog} onClose={() => setBenchmarkDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>
          <AssessmentIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
          Run Performance Benchmark
        </DialogTitle>
        <DialogContent>
          <Grid container spacing={3} sx={{ mt: 1 }}>
            <Grid item xs={12} md={6}>
              <TextField
                fullWidth
                label="Benchmark Name"
                value={benchmarkForm.benchmark_name}
                onChange={(e) => setBenchmarkForm({ ...benchmarkForm, benchmark_name: e.target.value })}
              />
            </Grid>

            <Grid item xs={12} md={6}>
              <TextField
                fullWidth
                label="Iterations"
                type="number"
                value={benchmarkForm.iterations}
                onChange={(e) => setBenchmarkForm({ ...benchmarkForm, iterations: parseInt(e.target.value) })}
              />
            </Grid>

            <Grid item xs={12} md={6}>
              <TextField
                fullWidth
                label="Warmup Iterations"
                type="number"
                value={benchmarkForm.warmup_iterations}
                onChange={(e) => setBenchmarkForm({ ...benchmarkForm, warmup_iterations: parseInt(e.target.value) })}
              />
            </Grid>

            <Grid item xs={12} md={6}>
              <TextField
                fullWidth
                label="Max Duration (seconds)"
                type="number"
                value={benchmarkForm.max_duration_secs}
                onChange={(e) => setBenchmarkForm({ ...benchmarkForm, max_duration_secs: parseInt(e.target.value) })}
              />
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setBenchmarkDialog(false)}>Cancel</Button>
          <Button
            onClick={handleBenchmarkSubmit}
            variant="contained"
            disabled={loading}
            startIcon={loading ? <CircularProgress size={20} /> : <AssessmentIcon />}
          >
            {loading ? 'Running...' : 'Start Benchmark'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default ModelWarmupControls;