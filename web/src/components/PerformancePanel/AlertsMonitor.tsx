import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  IconButton,
  Chip,
  Alert,
  Collapse,
  Button,
} from '@mui/material';
import {
  ErrorOutline as ErrorIcon,
  Warning as WarningIcon,
  Info as InfoIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
  Close as CloseIcon,
  Refresh as RefreshIcon,
} from '@mui/icons-material';

interface PerformanceAlert {
  timestamp: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  type: 'regression' | 'threshold_exceeded' | 'anomaly';
  title: string;
  message: string;
  details?: { [key: string]: string };
  resolved?: boolean;
  resolvedAt?: string;
}

interface AlertsMonitorProps {
  alerts: PerformanceAlert[];
  isLoading?: boolean;
  onRefresh?: () => void;
  onResolveAlert?: (alertId: string) => void;
  onDismissAlert?: (alertId: string) => void;
  autoRefreshInterval?: number; // in seconds
}

const AlertsMonitor: React.FC<AlertsMonitorProps> = ({
  alerts,
  isLoading = false,
  onRefresh,
  onResolveAlert,
  onDismissAlert,
  autoRefreshInterval = 30,
}) => {
  const [expandedAlerts, setExpandedAlerts] = useState<Set<string>>(new Set());
  const [filterSeverity, setFilterSeverity] = useState<string>('all');

  // Auto-refresh functionality
  useEffect(() => {
    if (autoRefreshInterval > 0 && onRefresh) {
      const interval = setInterval(() => {
        onRefresh();
      }, autoRefreshInterval * 1000);

      return () => clearInterval(interval);
    }
  }, [autoRefreshInterval, onRefresh]);

  const toggleAlertExpansion = (alertId: string) => {
    setExpandedAlerts(prev => {
      const newSet = new Set(prev);
      if (newSet.has(alertId)) {
        newSet.delete(alertId);
      } else {
        newSet.add(alertId);
      }
      return newSet;
    });
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical':
        return <ErrorIcon color="error" />;
      case 'high':
        return <WarningIcon color="error" />;
      case 'medium':
        return <WarningIcon color="warning" />;
      default:
        return <InfoIcon color="info" />;
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'error';
      case 'high':
        return 'error';
      case 'medium':
        return 'warning';
      default:
        return 'info';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    try {
      return new Date(timestamp).toLocaleString();
    } catch {
      return timestamp;
    }
  };

  const getAlertId = (alert: PerformanceAlert) => {
    return `${alert.timestamp}-${alert.type}-${alert.severity}`;
  };

  // Filter alerts based on selected severity
  const filteredAlerts = alerts.filter(alert => {
    if (filterSeverity === 'all') return true;
    return alert.severity === filterSeverity;
  });

  // Group alerts by status
  const activeAlerts = filteredAlerts.filter(alert => !alert.resolved);
  const resolvedAlerts = filteredAlerts.filter(alert => alert.resolved);

  // Calculate alert statistics
  const stats = {
    critical: alerts.filter(a => a.severity === 'critical' && !a.resolved).length,
    high: alerts.filter(a => a.severity === 'high' && !a.resolved).length,
    medium: alerts.filter(a => a.severity === 'medium' && !a.resolved).length,
    low: alerts.filter(a => a.severity === 'low' && !a.resolved).length,
  };

  return (
    <Box sx={{ p: 3 }}>
      {/* Header with controls */}
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h5">Alerts Monitor</Typography>

        <Box display="flex" alignItems="center" gap={2}>
          {/* Severity filter */}
          <Box display="flex" gap={1}>
            {[
              { key: 'all', label: 'All', count: activeAlerts.length },
              { key: 'critical', label: 'Critical', count: stats.critical },
              { key: 'high', label: 'High', count: stats.high },
              { key: 'medium', label: 'Medium', count: stats.medium },
            ].map(({ key, label, count }) => (
              <Chip
                key={key}
                label={`${label}${count > 0 ? ` (${count})` : ''}`}
                onClick={() => setFilterSeverity(key)}
                variant={filterSeverity === key ? 'filled' : 'outlined'}
                color={key === 'critical' ? 'error' : key === 'high' ? 'error' : key === 'medium' ? 'warning' : 'default'}
                size="small"
              />
            ))}
          </Box>

          {/* Refresh button */}
          <IconButton onClick={onRefresh} disabled={isLoading}>
            <RefreshIcon />
          </IconButton>
        </Box>
      </Box>

      {/* Alert Statistics */}
      <Box mb={3}>
        <Typography variant="h6" gutterBottom>Active Alerts Summary</Typography>
        <Box display="flex" gap={2} flexWrap="wrap">
          {stats.critical > 0 && (
            <Alert severity="error" variant="outlined">
              {stats.critical} Critical
            </Alert>
          )}
          {stats.high > 0 && (
            <Alert severity="warning" variant="outlined">
              {stats.high} High Priority
            </Alert>
          )}
          {stats.medium > 0 && (
            <Alert severity="warning" variant="outlined">
              {stats.medium} Medium Priority
            </Alert>
          )}
          {stats.low > 0 && (
            <Alert severity="info" variant="outlined">
              {stats.low} Low Priority
            </Alert>
          )}
          {activeAlerts.length === 0 && (
            <Alert severity="success" variant="outlined">
              No active alerts
            </Alert>
          )}
        </Box>
      </Box>

      {/* Active Alerts */}
      {activeAlerts.length > 0 && (
        <Card elevation={2} sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom color="error">
              Active Alerts ({activeAlerts.length})
            </Typography>

            <List sx={{ maxHeight: 400, overflow: 'auto' }}>
              {activeAlerts.map((alert) => {
                const alertId = getAlertId(alert);
                const isExpanded = expandedAlerts.has(alertId);

                return (
                  <React.Fragment key={alertId}>
                    <ListItem
                      sx={{
                        border: 1,
                        borderColor: 'divider',
                        borderRadius: 1,
                        mb: 1,
                        bgcolor: alert.severity === 'critical' ? 'error.50' : 'background.paper',
                        '&:hover': {
                          bgcolor: 'action.hover',
                        },
                        cursor: 'pointer'
                      }}
                      onClick={() => toggleAlertExpansion(alertId)}
                      secondaryAction={
                        <Box>
                          {onResolveAlert && !alert.resolved && (
                            <Button
                              size="small"
                              variant="outlined"
                              color="success"
                              onClick={(e) => {
                                e.stopPropagation();
                                onResolveAlert(getAlertId(alert));
                              }}
                              sx={{ mr: 1 }}
                            >
                              Resolve
                            </Button>
                          )}
                          {onDismissAlert && (
                            <IconButton
                              size="small"
                              onClick={(e) => {
                                e.stopPropagation();
                                onDismissAlert(getAlertId(alert));
                              }}
                            >
                              <CloseIcon />
                            </IconButton>
                          )}
                          {isExpanded ? <ExpandLessIcon /> : <ExpandMoreIcon />}
                        </Box>
                      }
                    >
                      <ListItemIcon>
                        {getSeverityIcon(alert.severity)}
                      </ListItemIcon>
                      <ListItemText
                        primary={
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography variant="subtitle1">
                              {alert.title}
                            </Typography>
                            <Chip
                              label={alert.severity.toUpperCase()}
                              size="small"
                              color={getSeverityColor(alert.severity) as any}
                              variant="filled"
                            />
                            <Typography variant="caption" color="text.secondary">
                              {formatTimestamp(alert.timestamp)}
                            </Typography>
                          </Box>
                        }
                        secondary={
                          <Box>
                            <Typography variant="body2" color="text.secondary">
                              {alert.message}
                            </Typography>
                            {alert.type && (
                              <Chip
                                label={alert.type.replace('_', ' ').toUpperCase()}
                                size="small"
                                variant="outlined"
                                sx={{ mt: 1 }}
                              />
                            )}
                          </Box>
                        }
                      />
                    </ListItem>

                    <Collapse in={isExpanded} timeout="auto" unmountOnExit>
                      <Box sx={{ pl: 4, pr: 2, pb: 2 }}>
                        {alert.details && Object.keys(alert.details).length > 0 && (
                          <Box mb={2}>
                            <Typography variant="subtitle2" gutterBottom>
                              Details
                            </Typography>
                            {Object.entries(alert.details).map(([key, value]) => (
                              <Box key={key} display="flex" gap={1} alignItems="center" mb={1}>
                                <Typography variant="body2" color="text.secondary" sx={{ minWidth: 120 }}>
                                  {key}:
                                </Typography>
                                <Typography variant="body2">{value}</Typography>
                              </Box>
                            ))}
                          </Box>
                        )}

                        {alert.resolvedAt && (
                          <Typography variant="body2" color="success.main">
                            Resolved at: {formatTimestamp(alert.resolvedAt)}
                          </Typography>
                        )}
                      </Box>
                    </Collapse>
                  </React.Fragment>
                );
              })}
            </List>
          </CardContent>
        </Card>
      )}

      {/* Resolved Alerts */}
      {resolvedAlerts.length > 0 && (
        <Card elevation={1}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Recently Resolved ({resolvedAlerts.length})
            </Typography>

            <List dense>
              {resolvedAlerts.map((alert) => (
                <ListItem key={getAlertId(alert)}>
                  <ListItemIcon sx={{ minWidth: 32 }}>
                    {getSeverityIcon(alert.severity)}
                  </ListItemIcon>
                  <ListItemText
                    primary={
                      <Box display="flex" alignItems="center" gap={1}>
                        <Typography variant="body2">
                          {alert.title}
                        </Typography>
                        <Chip
                          label="RESOLVED"
                          size="small"
                          color="success"
                          variant="outlined"
                        />
                      </Box>
                    }
                    secondary={`${alert.message} • Resolved ${formatTimestamp(alert.resolvedAt || '')}`}
                  />
                </ListItem>
              ))}
            </List>
          </CardContent>
        </Card>
      )}

      {/* Empty State */}
      {alerts.length === 0 && (
        <Box textAlign="center" py={6}>
          <InfoIcon color="disabled" sx={{ fontSize: 48, mb: 2 }} />
          <Typography variant="h6" color="text.secondary">
            No Alerts
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {isLoading ? 'Loading alerts...' : 'All systems performing normally'}
          </Typography>
        </Box>
      )}
    </Box>
  );
};

export default AlertsMonitor;