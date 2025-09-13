import React, { useState } from 'react';
import {
  Alert,
  AlertTitle,
  Box,
  Button,
  ButtonGroup,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  FormControl,
  InputLabel,
  LinearProgress,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  MenuItem,
  Paper,
  Select,
  SelectChangeEvent,
  TextField,
  Toolbar,
  Tooltip,
  Typography,
} from '@mui/material';
import {
  Check as ApplyIcon,
  CheckBox as CheckBoxIcon,
  CheckBoxOutlineBlank as CheckBoxOutlineBlankIcon,
  Check as CheckIcon,
  Error as ErrorIcon,
  FilterAlt as FilterIcon,
  VisibilityOff as HideIcon,
  DeleteOutline as IgnoreIcon,
  Info as InfoIcon,
  Refresh as RefreshIcon,
  Visibility as ShowIcon,
  Sort as SortIcon,
  Warning as WarningIcon,
} from '@mui/icons-material';
import { Severity } from '../../types';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import {
  analyzeVersionAlignment,
  applyVersionAlignment,
  clearSelectedAlignments,
  selectAllAlignments,
  selectVersionAlignment,
  setFilter,
  toggleIgnoreAlignment,
} from '../../dependencySlice';

const severityOptions: { value: Severity | 'all'; label: string }[] = [
  { value: 'all', label: 'All Severities' },
  { value: 'high', label: 'High' },
  { value: 'medium', label: 'Medium' },
  { value: 'low', label: 'Low' },
];

const sortOptions = [
  { value: 'severity', label: 'Severity' },
  { value: 'dependencyName', label: 'Dependency Name' },
  { value: 'suggestedVersion', label: 'Suggested Version' },
];

const VersionAlignmentToolbar: React.FC = () => {
  const dispatch = useAppDispatch();
  const {
    selectedCount,
    totalCount,
    hasSelection,
    hasAlignments,
    filter,
    status,
    alignments,
    selectedIds,
  } = useAppSelector(selectVersionAlignment);
  const [batchDialogOpen, setBatchDialogOpen] = useState(false);
  const [isBatchApplying, setIsBatchApplying] = useState(false);

  const handleSeverityChange = (event: SelectChangeEvent) => {
    dispatch(setFilter({ severity: event.target.value as Severity | 'all' }));
  };

  const handleSearchChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    dispatch(setFilter({ searchTerm: (event.target as any).value }));
  };

  const handleSortChange = (event: SelectChangeEvent) => {
    const [sortBy, sortOrder] = event.target.value.split('_');
    dispatch(
      setFilter({
        sortBy: sortBy as 'severity' | 'dependencyName' | 'suggestedVersion',
        sortOrder: sortOrder as 'asc' | 'desc',
      })
    );
  };

  const toggleShowIgnored = () => {
    dispatch(setFilter({ showIgnored: !filter.showIgnored }));
  };

  const handleSelectAll = () => {
    if (selectedCount === totalCount) {
      return dispatch(clearSelectedAlignments());
    }

    if (typeof window === 'undefined' || typeof document === 'undefined') {
      return;
    }

    const elements = document.querySelectorAll('[data-alignment-id]');
    const allIds: string[] = [];

    elements.forEach((el: Element) => {
      const id = el.getAttribute('data-alignment-id');
      if (id) {
        allIds.push(id);
      }
    });

    dispatch(selectAllAlignments(allIds));
  };

  const handleBulkAction = (action: 'apply' | 'ignore') => {
    if (typeof window === 'undefined' || typeof document === 'undefined') return;

    const elements = document.querySelectorAll('[data-alignment-id]');
    const selectedIds: string[] = [];

    elements.forEach((el: Element) => {
      const id = el.getAttribute('data-alignment-id');
      if (id !== null) {
        selectedIds.push(id);
      }
    });

    if (action === 'apply') {
      dispatch(applyVersionAlignment(selectedIds));
    } else {
      selectedIds.forEach((id) => dispatch(toggleIgnoreAlignment(id)));
    }
  };

  const handleRefresh = () => {
    dispatch(analyzeVersionAlignment());
  };

  const handleBatchResolve = () => {
    setBatchDialogOpen(true);
  };

  const handleBatchDialogClose = () => {
    setBatchDialogOpen(false);
  };

  const handleBatchApply = async () => {
    if (selectedIds.length === 0) return;

    try {
      setIsBatchApplying(true);
      const result = await dispatch(applyVersionAlignment(selectedIds) as any).unwrap();

      // Show success/error message
      // The actual UI feedback is handled by the extraReducers in the slice

      // Close the dialog
      setBatchDialogOpen(false);
    } catch (error) {
      console.error('Failed to apply batch resolution:', error);
    } finally {
      setIsBatchApplying(false);
    }
  };

  return (
    <>
      <Toolbar
        sx={{
          borderBottom: '1px solid',
          borderColor: 'divider',
          gap: 2,
          flexWrap: 'wrap',
          p: 2,
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flexWrap: 'wrap', flexGrow: 1 }}>
          <FormControl size="small" sx={{ minWidth: 120 }}>
            <InputLabel id="severity-filter-label">Severity</InputLabel>
            <Select
              labelId="severity-filter-label"
              id="severity-filter"
              value={filter.severity}
              label="Severity"
              onChange={handleSeverityChange}
              size="small"
              startAdornment={
                <FilterIcon fontSize="small" sx={{ mr: 1, color: 'text.secondary' }} />
              }
            >
              {severityOptions.map((option) => (
                <MenuItem key={option.value} value={option.value}>
                  {option.label}
                </MenuItem>
              ))}
            </Select>
          </FormControl>

          <FormControl size="small" sx={{ minWidth: 180 }}>
            <InputLabel id="sort-by-label">Sort by</InputLabel>
            <Select
              labelId="sort-by-label"
              id="sort-by"
              value={`${filter.sortBy}_${filter.sortOrder}`}
              label="Sort by"
              onChange={handleSortChange}
              size="small"
              startAdornment={<SortIcon fontSize="small" sx={{ mr: 1, color: 'text.secondary' }} />}
            >
              {sortOptions.map((option) => (
                <React.Fragment key={option.value}>
                  <MenuItem value={`${option.value}_asc`}>{option.label} (A-Z)</MenuItem>
                  <MenuItem value={`${option.value}_desc`}>{option.label} (Z-A)</MenuItem>
                </React.Fragment>
              ))}
            </Select>
          </FormControl>

          <TextField
            size="small"
            placeholder="Search dependencies..."
            variant="outlined"
            onChange={handleSearchChange}
            value={filter.searchTerm || ''}
            sx={{ minWidth: 200 }}
          />

          <Box sx={{ display: 'flex', gap: 1, ml: 'auto' }}>
            <Tooltip title={filter.showIgnored ? 'Hide ignored' : 'Show ignored'}>
              <span>
                <Button
                  size="small"
                  onClick={toggleShowIgnored}
                  startIcon={filter.showIgnored ? <ShowIcon /> : <HideIcon />}
                  disabled={!hasAlignments}
                  color={filter.showIgnored ? 'primary' : 'inherit'}
                >
                  {filter.showIgnored ? 'Showing Ignored' : 'Hidden'}
                </Button>
              </span>
            </Tooltip>

            <Tooltip title="Refresh">
              <span>
                <Button
                  size="small"
                  onClick={handleRefresh}
                  disabled={status === 'loading'}
                  startIcon={<RefreshIcon />}
                >
                  Refresh
                </Button>
              </span>
            </Tooltip>
          </Box>
        </Box>

        {hasAlignments && (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, width: '100%', mt: 1 }}>
            <Button
              size="small"
              onClick={handleSelectAll}
              startIcon={
                selectedCount === totalCount ? <CheckBoxIcon /> : <CheckBoxOutlineBlankIcon />
              }
              disabled={!hasAlignments}
            >
              {selectedCount === totalCount ? 'Deselect All' : 'Select All'}
            </Button>

            <Button
              size="small"
              variant="contained"
              color="primary"
              startIcon={<CheckIcon />}
              onClick={handleBatchResolve}
              disabled={!hasSelection || status === 'loading'}
              sx={{ ml: 1 }}
            >
              Resolve {selectedCount} Selected
            </Button>

            <Chip
              label={`${selectedCount} selected`}
              size="small"
              variant="outlined"
              sx={{ ml: 'auto' }}
            />

            <ButtonGroup size="small" disabled={!hasSelection}>
              <Button
                onClick={() => handleBulkAction('ignore')}
                startIcon={<IgnoreIcon />}
                color="inherit"
              >
                Ignore
              </Button>
              <Button
                onClick={() => handleBulkAction('apply')}
                startIcon={<ApplyIcon />}
                color="primary"
                variant="contained"
              >
                Apply
              </Button>
            </ButtonGroup>
          </Box>
        )}
      </Toolbar>

      {/* Batch Resolution Dialog */}
      <Dialog
        open={batchDialogOpen}
        onClose={handleBatchDialogClose}
        maxWidth="md"
        fullWidth
        aria-labelledby="batch-resolution-dialog-title"
      >
        <DialogTitle id="batch-resolution-dialog-title">Batch Resolve Conflicts</DialogTitle>

        <DialogContent dividers>
          <Alert severity="info" sx={{ mb: 2 }}>
            <AlertTitle>Resolving {selectedCount} Dependencies</AlertTitle>
            The following dependencies will be updated to their suggested versions.
          </Alert>

          <Paper variant="outlined" sx={{ maxHeight: 300, overflow: 'auto' }}>
            <List dense>
              {selectedIds.slice(0, 10).map((id, index) => {
                const alignment = alignments.find((a) => a.id === id);
                if (!alignment) return null;

                return (
                  <React.Fragment key={id}>
                    <ListItem>
                      <ListItemIcon sx={{ minWidth: 36 }}>
                        {alignment.severity === 'high' ? (
                          <ErrorIcon color="error" fontSize="small" />
                        ) : alignment.severity === 'medium' ? (
                          <WarningIcon color="warning" fontSize="small" />
                        ) : (
                          <InfoIcon color="info" fontSize="small" />
                        )}
                      </ListItemIcon>
                      <ListItemText
                        primary={
                          <Typography variant="subtitle2">{alignment.dependencyName}</Typography>
                        }
                        secondary={
                          <Box component="span" display="flex" alignItems="center">
                            <Typography variant="caption" color="text.secondary" sx={{ mr: 1 }}>
                              {Object.values(alignment.currentVersions)[0]}
                            </Typography>
                            <Typography
                              variant="caption"
                              color="primary"
                              sx={{ fontWeight: 'medium' }}
                            >
                              → {alignment.suggestedVersion}
                            </Typography>
                          </Box>
                        }
                      />
                    </ListItem>
                    {index < selectedCount - 1 && <Divider component="li" />}
                  </React.Fragment>
                );
              })}

              {selectedCount > 10 && (
                <ListItem>
                  <Typography variant="body2" color="text.secondary">
                    ...and {selectedCount - 10} more
                  </Typography>
                </ListItem>
              )}
            </List>
          </Paper>

          {isBatchApplying && (
            <Box sx={{ mt: 2 }}>
              <Typography variant="body2" color="text.secondary" gutterBottom>
                Applying changes...
              </Typography>
              <LinearProgress />
            </Box>
          )}
        </DialogContent>

        <DialogActions sx={{ p: 2 }}>
          <Button onClick={handleBatchDialogClose} color="inherit" disabled={isBatchApplying}>
            Cancel
          </Button>
          <Button
            onClick={handleBatchApply}
            variant="contained"
            color="primary"
            disabled={isBatchApplying || selectedCount === 0}
            startIcon={<CheckIcon />}
          >
            Apply {selectedCount} Change{selectedCount !== 1 ? 's' : ''}
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
};

export default VersionAlignmentToolbar;
