import {
  Assessment as AssessmentIcon,
  CheckCircle as CheckCircleIcon,
  Speed as SpeedIcon,
  Warning as WarningIcon,
} from '@mui/icons-material';
import {
  Box,
  Card,
  CardContent,
  CircularProgress,
  Paper,
  Tab,
  Tabs,
  Typography,
  useTheme,
} from '@mui/material';
import React, { useEffect } from 'react';
import { usePerformanceAnalysis } from '../hooks/usePerformanceAnalysis';
import { PerformanceRecommendations } from './PerformanceRecommendations';

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
      id={`performance-tabpanel-${index}`}
      aria-labelledby={`performance-tab-${index}`}
      {...other}
    >
      {value === index && (
        <Box sx={{ p: 3 }}>
          <Typography component="div">{children}</Typography>
        </Box>
      )}
    </div>
  );
}

function a11yProps(index: number) {
  return {
    id: `performance-tab-${index}`,
    'aria-controls': `performance-tabpanel-${index}`,
  };
}

const PerformanceDashboard: React.FC = () => {
  const theme = useTheme();
  const [tabValue, setTabValue] = React.useState(0);
  const { analysis, isLoading, error, analyzeProject, applyRecommendation, dismissRecommendation } =
    usePerformanceAnalysis();

  useEffect(() => {
    analyzeProject();
  }, []);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };

  const handleApplyRecommendation = (id: string) => {
    applyRecommendation(id);
  };

  const handleDismissRecommendation = (id: string) => {
    dismissRecommendation(id);
  };

  if (isLoading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="200px">
        <CircularProgress />
        <Box ml={2}>
          <Typography>Analyzing project performance...</Typography>
        </Box>
      </Box>
    );
  }

  if (error) {
    return (
      <Box p={3}>
        <Typography color="error">Error: {error.message}</Typography>
      </Box>
    );
  }

  if (!analysis) {
    return (
      <Box p={3}>
        <Typography>No performance data available. Try analyzing your project.</Typography>
      </Box>
    );
  }

  const { summary, recommendations } = analysis;
  const hasRecommendations = recommendations.length > 0;

  return (
    <Box sx={{ width: '100%' }}>
      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs
          value={tabValue}
          onChange={handleTabChange}
          aria-label="performance tabs"
          variant="scrollable"
          scrollButtons="auto"
        >
          <Tab
            icon={<AssessmentIcon />}
            iconPosition="start"
            label="Overview"
            {...a11yProps(0)}
          />
          <Tab
            icon={<WarningIcon />}
            iconPosition="start"
            label={`Recommendations ${hasRecommendations ? `(${recommendations.length})` : ''}`}
            {...a11yProps(1)}
          />
          <Tab
            icon={<SpeedIcon />}
            iconPosition="start"
            label="Metrics"
            {...a11yProps(2)}
          />
        </Tabs>
      </Box>

      <TabPanel value={tabValue} index={0}>
        <Box sx={{ display: 'flex', flexDirection: { xs: 'column', md: 'row' }, gap: 3 }}>
          <Box sx={{ width: { xs: '100%', md: '33%' } }}>
            <Card elevation={3}>
              <CardContent>
                <Box display="flex" alignItems="center" mb={2}>
                  <SpeedIcon color="primary" sx={{ mr: 1 }} />
                  <Typography variant="h6" component="div">
                    Performance Score
                  </Typography>
                </Box>
                <Box
                  sx={{
                    position: 'relative',
                    display: 'inline-flex',
                    width: '100%',
                    justifyContent: 'center',
                    my: 2,
                  }}
                >
                  <CircularProgress
                    variant="determinate"
                    value={summary.performanceScore}
                    size={120}
                    thickness={4}
                    sx={{
                      color: theme.palette.primary.main,
                    }}
                  />
                  <Box
                    sx={{
                      top: 0,
                      left: 0,
                      bottom: 0,
                      right: 0,
                      position: 'absolute',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                    }}
                  >
                    <Typography variant="h4" component="div" color="text.primary">
                      {Math.round(summary.performanceScore)}%
                    </Typography>
                  </Box>
                </Box>
                <Typography variant="body2" color="text.secondary" align="center">
                  Overall project performance
                </Typography>
              </CardContent>
            </Card>
          </Box>

          <Box sx={{ width: { xs: '100%', md: '67%' } }}>
            <Card elevation={3} sx={{ height: '100%' }}>
              <CardContent>
                <Box display="flex" alignItems="center" mb={2}>
                  <WarningIcon color="warning" sx={{ mr: 1 }} />
                  <Typography variant="h6" component="div">
                    Issues & Recommendations
                  </Typography>
                </Box>
                <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 2, justifyContent: 'space-between' }}>
                  <Box sx={{ width: { xs: '100%', sm: '48%', md: '30%' } }}>
                    <Paper
                      elevation={0}
                      sx={{
                        p: 2,
                        textAlign: 'center',
                        border: '1px solid',
                        borderColor: 'divider',
                        borderRadius: 1,
                      }}
                    >
                      <Typography variant="h4" color="error">
                        {summary.totalIssues}
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Total Issues
                      </Typography>
                    </Paper>
                  </Box>
                  <Box sx={{ width: { xs: '100%', sm: '48%', md: '30%' } }}>
                    <Paper
                      elevation={0}
                      sx={{
                        p: 2,
                        textAlign: 'center',
                        border: '1px solid',
                        borderColor: 'divider',
                        borderRadius: 1,
                      }}
                    >
                      <Typography variant="h4" color="warning.main">
                        {summary.totalRecommendations}
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Recommendations
                      </Typography>
                    </Paper>
                  </Box>
                  <Box sx={{ width: { xs: '100%', sm: '48%', md: '30%' } }}>
                    <Paper
                      elevation={0}
                      sx={{
                        p: 2,
                        textAlign: 'center',
                        border: '1px solid',
                        borderColor: 'divider',
                        borderRadius: 1,
                      }}
                    >
                      <Typography variant="h4" color="success.main">
                        {Math.round((1 - summary.totalRecommendations / (summary.totalRecommendations + 1)) * 100)}%
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Issues Resolved
                      </Typography>
                    </Paper>
                  </Box>
                </Box>
                <Box mt={3}>
                  <Typography variant="body2" color="text.secondary">
                    Last analyzed: {new Date(summary.timestamp).toLocaleString()}
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Box>
        </Box>
      </TabPanel>

      <TabPanel value={tabValue} index={1}>
        <PerformanceRecommendations
          recommendations={recommendations}
          onApplyRecommendation={handleApplyRecommendation}
          onDismissRecommendation={handleDismissRecommendation}
        />
      </TabPanel>

      <TabPanel value={tabValue} index={2}>
        <Box p={2}>
          <Typography variant="h6" gutterBottom>
            Performance Metrics
          </Typography>
          <Typography color="text.secondary">
            Performance metrics visualization will be available in the next update.
          </Typography>
        </Box>
      </TabPanel>
    </Box>
  );
};

export default PerformanceDashboard;
