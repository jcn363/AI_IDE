import CloseIcon from '@mui/icons-material/Close';
import RefreshIcon from '@mui/icons-material/Refresh';
import {
  Alert,
  AlertTitle,
  Box,
  Button,
  Chip,
  CircularProgress,
  Collapse,
  IconButton,
  Paper,
  Snackbar,
  Stack,
  Typography,
  useTheme,
} from '@mui/material';
import React, { useCallback, useEffect, useState } from 'react';
import { useAppDispatch, useAppSelector } from '../../../../store/hooks';
import { VersionAlignment } from '../../types';
import {
  analyzeWorkspaceAlignment,
  applyWorkspaceAlignment,
  clearSelectedAlignments,
  selectAllAlignments,
  selectVersionAlignment,
  toggleIgnoreAlignment,
  toggleSelectAlignment,
} from '../../dependencySlice';
import VersionAlignmentList from './VersionAlignmentList';
import WorkspaceStats from './WorkspaceStats';

const WorkspaceAlignmentView: React.FC = () => {
  const theme = useTheme();
  const dispatch = useAppDispatch();
  const {
    filteredAlignments: alignments,
    status,
    error,
    selectedIds,
  } = useAppSelector(selectVersionAlignment);
  const [isBulkApplying, setIsBulkApplying] = useState(false);
  const [snackbar, setSnackbar] = useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info' | 'warning';
  }>({
    open: false,
    message: '',
    severity: 'info',
  });
  const [stats, setStats] = useState<{
    totalDependencies: number;
    alignedDependencies: number;
    conflicts: number;
    highSeverity: number;
    mediumSeverity: number;
    lowSeverity: number;
  }>({
    totalDependencies: 0,
    alignedDependencies: 0,
    conflicts: 0,
    highSeverity: 0,
    mediumSeverity: 0,
    lowSeverity: 0,
  });
  const [isApplying, setIsApplying] = useState(false);

  useEffect(() => {
    if (alignments.length > 0) {
      const highSeverity = alignments.filter((a) => a.severity === 'high').length;
      const mediumSeverity = alignments.filter((a) => a.severity === 'medium').length;
      const lowSeverity = alignments.filter((a) => a.severity === 'low').length;

      setStats({
        totalDependencies: alignments.reduce(
          (acc, curr) => acc + Object.keys(curr.currentVersions).length,
          0
        ),
        alignedDependencies: 0,
        conflicts: alignments.length,
        highSeverity,
        mediumSeverity,
        lowSeverity,
      });
    }
  }, [alignments]);

  const refreshAlignments = useCallback(async () => {
    try {
      await dispatch(analyzeWorkspaceAlignment()).unwrap();
    } catch (err) {
      setSnackbar({
        open: true,
        message: 'Failed to refresh version alignments',
        severity: 'error',
      });
    }
  }, [dispatch]);

  useEffect(() => {
    refreshAlignments();
  }, [refreshAlignments]);

  const handleApplyAll = useCallback(async () => {
    try {
      setIsBulkApplying(true);
      await dispatch(applyWorkspaceAlignment(selectedIds)).unwrap();
      setSnackbar({
        open: true,
        message: `Successfully applied ${selectedIds.length} version alignments`,
        severity: 'success',
      });
    } catch (err) {
      setSnackbar({
        open: true,
        message: 'Failed to apply version alignments',
        severity: 'error',
      });
    } finally {
      setIsBulkApplying(false);
    }
  }, [dispatch, selectedIds]);

  const handleSelectAll = useCallback(() => {
    if (selectedIds.length === alignments.length) {
      dispatch(clearSelectedAlignments());
    } else {
      dispatch(selectAllAlignments(alignments.map((a: VersionAlignment) => a.id)));
    }
  }, [dispatch, selectedIds.length, alignments]);

  const handleSelectOne = useCallback(
    (id: string) => {
      dispatch(toggleSelectAlignment(id));
    },
    [dispatch]
  );

  const handleApplyAlignment = useCallback(
    async (alignment: VersionAlignment) => {
      try {
        setIsApplying(true);
        await dispatch(applyWorkspaceAlignment([alignment.id])).unwrap();
        setSnackbar({
          open: true,
          message: `Successfully applied version alignment for ${alignment.dependencyName}`,
          severity: 'success',
        });
      } catch (err) {
        setSnackbar({
          open: true,
          message: 'Failed to apply version alignment',
          severity: 'error',
        });
      } finally {
        setIsApplying(false);
      }
    },
    [dispatch]
  );

  const handleResolveConflict = useCallback(
    async (alignment: VersionAlignment, selectedVersion: string) => {
      try {
        setIsApplying(true);
        // Update the alignment with the selected version
        const updatedAlignment = {
          ...alignment,
          suggestedVersion: selectedVersion,
          currentVersions: {
            [Object.keys(alignment.currentVersions)[0]]: selectedVersion,
          },
        };

        // Apply the resolution
        await dispatch(applyWorkspaceAlignment([updatedAlignment.id])).unwrap();

        setSnackbar({
          open: true,
          message: `Resolved version conflict for ${alignment.dependencyName} (${selectedVersion})`,
          severity: 'success',
        });
      } catch (err) {
        setSnackbar({
          open: true,
          message: 'Failed to resolve version conflict',
          severity: 'error',
        });
        throw err;
      } finally {
        setIsApplying(false);
      }
    },
    [dispatch]
  );

  const handleIgnoreAlignment = useCallback((alignment: VersionAlignment) => {
    // Implementation for ignoring an alignment
    setSnackbar({
      open: true,
      message: `Ignored version alignment for ${alignment.dependencyName}`,
      severity: 'info',
    });
  }, []);

  const handleCloseSnackbar = useCallback(() => {
    setSnackbar((prev) => ({ ...prev, open: false }));
  }, []);

  return (
    <Box sx={{ p: 3, maxWidth: 1200, mx: 'auto' }}>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h5" component="h1">
          Workspace Version Alignment
        </Typography>
        <Button
          variant="outlined"
          startIcon={<RefreshIcon />}
          onClick={refreshAlignments}
          disabled={status === 'loading'}
          sx={{ minWidth: 120 }}
        >
          {status === 'loading' ? 'Refreshing...' : 'Refresh'}
        </Button>
      </Box>

      <Collapse in={Boolean(error)} sx={{ mb: 2 }}>
        <Alert
          severity="error"
          action={
            <IconButton aria-label="close" color="inherit" size="small" onClick={() => {}}>
              <CloseIcon fontSize="inherit" />
            </IconButton>
          }
        >
          {error}
        </Alert>
      </Collapse>

      <WorkspaceStats
        total={stats.totalDependencies}
        aligned={stats.alignedDependencies}
        conflicts={stats.conflicts}
        highSeverity={stats.highSeverity}
        mediumSeverity={stats.mediumSeverity}
        lowSeverity={stats.lowSeverity}
      />

      <Paper
        elevation={0}
        sx={{
          mt: 3,
          border: `1px solid ${theme.palette.divider}`,
          borderRadius: 1,
          overflow: 'hidden',
        }}
      >
        <Box
          p={2}
          display="flex"
          justifyContent="space-between"
          alignItems="center"
          sx={{
            backgroundColor: theme.palette.background.paper,
            borderBottom: `1px solid ${theme.palette.divider}`,
          }}
        >
          <Typography variant="subtitle1" color="text.primary">
            Version Mismatches
            {alignments.length > 0 && (
              <Chip
                label={`${alignments.length} found`}
                size="small"
                sx={{ ml: 1, fontWeight: 500 }}
                color={alignments.length > 0 ? 'primary' : 'default'}
                variant={alignments.length > 0 ? 'filled' : 'outlined'}
              />
            )}
          </Typography>

          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              onClick={handleSelectAll}
              disabled={status === 'loading' || alignments.length === 0}
              sx={{ minWidth: 120 }}
            >
              {selectedIds.length === alignments.length ? 'Deselect All' : 'Select All'}
            </Button>

            <Button
              variant="contained"
              color="primary"
              onClick={handleApplyAll}
              disabled={isBulkApplying || selectedIds.length === 0 || status === 'loading'}
              startIcon={isBulkApplying ? <CircularProgress size={20} color="inherit" /> : null}
              sx={{ minWidth: 180 }}
            >
              {isBulkApplying ? 'Applying...' : `Apply (${selectedIds.length})`}
            </Button>
          </Stack>
        </Box>

        <VersionAlignmentList
          alignments={alignments}
          selectedIds={selectedIds}
          loading={status === 'loading'}
          onSelect={handleSelectAll}
          onSelectOne={handleSelectOne}
          onApplyAlignment={handleApplyAlignment}
          onIgnoreAlignment={handleIgnoreAlignment}
          onResolveConflict={handleResolveConflict}
          isBulkApplying={isBulkApplying}
        />
      </Paper>

      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={handleCloseSnackbar}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        sx={{ '& .MuiPaper-root': { minWidth: 300 } }}
      >
        <Alert
          onClose={handleCloseSnackbar}
          severity={snackbar.severity}
          sx={{ width: '100%', boxShadow: 3 }}
          variant="filled"
        >
          <AlertTitle sx={{ fontWeight: 'bold' }}>
            {snackbar.severity === 'success'
              ? 'Success'
              : snackbar.severity === 'error'
                ? 'Error'
                : snackbar.severity === 'warning'
                  ? 'Warning'
                  : 'Info'}
          </AlertTitle>
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Box>
  );
};

export default WorkspaceAlignmentView;
