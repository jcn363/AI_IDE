import {
  Check as CheckIcon,
  Close as CloseIcon,
  Refresh as RefreshIcon,
  Speed as SpeedIcon,
  Warning as WarningIcon,
} from '@mui/icons-material';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Container,
  Snackbar,
  Stack,
  Tab,
  Tabs,
  Typography,
  useTheme,
} from '@mui/material';
import React, { useEffect, useMemo, useState } from 'react';
import { useAppDispatch, useAppSelector } from '../../../../store/hooks';
import {
  analyzeVersionAlignment,
  applyVersionAlignment,
  clearSelectedAlignments,
  selectAllAlignments,
  selectSelectedAlignments,
  selectVersionAlignment,
  toggleIgnoreAlignment,
  toggleSelectAlignment,
} from '../../dependencySlice';
import { VersionAlignment } from '../../types';
import PerformanceTab from './PerformanceTab';
import VersionAlignmentList from './VersionAlignmentList';
import VersionAlignmentToolbar from './VersionAlignmentToolbar';

const SeverityChip: React.FC<{ severity: 'low' | 'medium' | 'high' }> = ({ severity }) => {
  const theme = useTheme();
  const styles = {
    low: {
      bgcolor: theme.palette.info.light,
      color: theme.palette.info.contrastText,
    },
    medium: {
      bgcolor: theme.palette.warning.light,
      color: theme.palette.warning.contrastText,
    },
    high: {
      bgcolor: theme.palette.error.light,
      color: theme.palette.error.contrastText,
    },
  };

  return (
    <Chip
      label={severity.charAt(0).toUpperCase() + severity.slice(1)}
      size="small"
      sx={styles[severity]}
    />
  );
};

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
      id={`version-alignment-tabpanel-${index}`}
      aria-labelledby={`version-alignment-tab-${index}`}
      style={{ height: '100%' }}
      {...other}
    >
      {value === index && <Box sx={{ p: 0, height: '100%' }}>{children}</Box>}
    </div>
  );
}

function a11yProps(index: number) {
  return {
    id: `version-alignment-tab-${index}`,
    'aria-controls': `version-alignment-tabpanel-${index}`,
  };
}

export const VersionAlignmentView: React.FC = () => {
  const theme = useTheme();
  const dispatch = useAppDispatch();
  const [tabValue, setTabValue] = useState(0);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };
  const { filteredAlignments, selectedCount, status, error } =
    useAppSelector(selectVersionAlignment);

  const selectedAlignments = useAppSelector(selectSelectedAlignments);
  const [snackbar, setSnackbar] = useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info' | 'warning';
    action?: React.ReactNode;
  }>({
    open: false,
    message: '',
    severity: 'success',
  });

  // Auto-refresh alignments when component mounts
  useEffect(() => {
    if (status === 'idle') {
      dispatch(analyzeVersionAlignment());
    }
  }, [status, dispatch]);

  const handleApplyAlignment = async (alignment: VersionAlignment) => {
    try {
      setIsApplying(true);
      const result = await (
        dispatch(applyVersionAlignment([alignment.id])) as unknown as Promise<{
          updatedCount: number;
        }>
      ).then(
        (res) => res,
        (err) => {
          throw err;
        }
      );

      setSnackbar({
        open: true,
        message:
          result.updatedCount > 0
            ? 'Version alignment applied successfully'
            : 'No changes were made',
        severity: result.updatedCount > 0 ? 'success' : 'info',
      });

      // Refresh the alignments to show updated state
      await dispatch(analyzeVersionAlignment() as any);
      dispatch(clearSelectedAlignments());
    } catch (err) {
      setSnackbar({
        open: true,
        message: err instanceof Error ? err.message : 'Failed to apply version alignment',
        severity: 'error',
      });
    } finally {
      setIsApplying(false);
    }
  };

  const [isApplying, setIsApplying] = useState(false);
  const [isBulkApplying, setIsBulkApplying] = useState(false);

  const handleBulkApply = async () => {
    if (selectedCount === 0) return;

    try {
      setIsBulkApplying(true);
      const result = await (
        dispatch(applyVersionAlignment(selectedAlignments.map((a) => a.id))) as unknown as Promise<{
          updatedCount: number;
          failedCount: number;
        }>
      ).then(
        (res) => res,
        (err) => {
          throw err;
        }
      );

      setSnackbar({
        open: true,
        message:
          result.failedCount > 0
            ? `Applied ${result.updatedCount} of ${selectedCount} alignments. ${result.failedCount} failed.`
            : `Successfully applied ${result.updatedCount} version alignments`,
        severity: result.failedCount > 0 ? 'warning' : 'success',
      });

      if (result.updatedCount > 0) {
        // Refresh the alignments to show updated state
        await dispatch(analyzeVersionAlignment() as any);
      }

      dispatch(clearSelectedAlignments());
    } catch (err) {
      setSnackbar({
        open: true,
        message: err instanceof Error ? err.message : 'Failed to apply version alignments',
        severity: 'error',
      });
    } finally {
      setIsBulkApplying(false);
    }
  };

  const handleIgnoreAlignment = async (alignment: VersionAlignment) => {
    try {
      const result = await (
        dispatch(toggleIgnoreAlignment(alignment.id)) as unknown as Promise<{ ignored: boolean }>
      ).then(
        (res) => res,
        (err) => {
          throw err;
        }
      );

      setSnackbar({
        open: true,
        message: result.ignored
          ? `Ignored version alignment for ${alignment.dependencyName}`
          : `Stopped ignoring version alignment for ${alignment.dependencyName}`,
        severity: 'info',
      });

      // Refresh the alignments to show updated state
      await dispatch(analyzeVersionAlignment() as any);
    } catch (err) {
      setSnackbar({
        open: true,
        message: err instanceof Error ? err.message : 'Failed to update ignore status',
        severity: 'error',
      });
    }
  };

  const handleSelectAll = () => {
    if (selectedCount === filteredAlignments.length) {
      dispatch(clearSelectedAlignments());
    } else {
      dispatch(selectAllAlignments(filteredAlignments.map((a) => a.id)));
    }
  };

  const handleCloseSnackbar = () => {
    setSnackbar((prev) => ({ ...prev, open: false }));
  };

  const handleSelectAlignment = (id: string) => {
    dispatch(toggleSelectAlignment(id));
  };

  const handleApplyAlignmentWrapper = async (alignment: VersionAlignment) => {
    await handleApplyAlignment(alignment);
  };

  const stats = useMemo(() => {
    const stats = { high: 0, medium: 0, low: 0 };
    filteredAlignments.forEach((a) => {
      if (!a.isIgnored) stats[a.severity as keyof typeof stats] += 1;
    });
    return stats;
  }, [filteredAlignments]);

  return (
    <Box>
      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={handleCloseSnackbar}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert
          onClose={handleCloseSnackbar}
          severity={snackbar.severity}
          sx={{ width: '100%' }}
          variant="filled"
        >
          {snackbar.message}
        </Alert>
      </Snackbar>

      <Container maxWidth="xl" sx={{ py: 4, height: 'calc(100vh - 100px)' }}>
        <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%', gap: 3 }}>
          <Card sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
              <Tabs
                value={tabValue}
                onChange={handleTabChange}
                aria-label="version alignment tabs"
                variant="fullWidth"
              >
                <Tab
                  icon={<CheckIcon />}
                  iconPosition="start"
                  label="Version Alignment"
                  {...a11yProps(0)}
                />
                <Tab
                  icon={<SpeedIcon />}
                  iconPosition="start"
                  label="Performance"
                  {...a11yProps(1)}
                />
              </Tabs>
            </Box>

            <Box sx={{ flex: 1, overflow: 'auto' }}>
              <TabPanel value={tabValue} index={0}>
                <CardContent>
                  <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                    <Box>
                      <Typography variant="h5" component="h1" gutterBottom>
                        Version Alignment
                      </Typography>
                      <Typography variant="body1" color="text.secondary">
                        Review and resolve version conflicts across your workspace.
                      </Typography>
                    </Box>
                    <Box display="flex" gap={1}>
                      {status === 'loading' && <CircularProgress size={24} />}
                      <Button
                        variant="outlined"
                        size="small"
                        onClick={() => dispatch(analyzeVersionAlignment())}
                        startIcon={<RefreshIcon />}
                      >
                        Re-analyze
                      </Button>
                    </Box>
                  </Box>

                  <VersionAlignmentToolbar />

                  {status === 'succeeded' ? (
                    <Box display="flex" justifyContent="center" my={4}>
                      <CircularProgress />
                    </Box>
                  ) : error ? (
                    <Alert severity="error" sx={{ mt: 2 }}>
                      {error}
                    </Alert>
                  ) : (
                    <VersionAlignmentList
                      alignments={filteredAlignments}
                      selectedIds={selectedAlignments.map((a) => a.id)}
                      loading={status === 'loading'}
                      onSelect={handleSelectAll}
                      onSelectOne={handleSelectAlignment}
                      onApplyAlignment={handleApplyAlignmentWrapper}
                      onIgnoreAlignment={handleIgnoreAlignment}
                      isApplying={isApplying}
                      isBulkApplying={isBulkApplying}
                    />
                  )}
                </CardContent>
              </TabPanel>

              <TabPanel value={tabValue} index={1}>
                <CardContent>
                  <PerformanceTab projectId="current" />
                </CardContent>
              </TabPanel>
            </Box>
          </Card>
        </Box>
      </Container>
    </Box>
  );
};

export default VersionAlignmentView;
