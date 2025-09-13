import {
  Check as CheckIcon,
  Error as ErrorIcon,
  Info as InfoIcon,
  Warning as WarningIcon,
} from '@mui/icons-material';
import {
  Alert,
  AlertTitle,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  LinearProgress,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Paper,
  Typography,
  useTheme,
} from '@mui/material';
import React, { useState } from 'react';
import { VersionAlignment } from '../../types';

interface BatchResolutionDialogProps {
  open: boolean;
  onClose: () => void;
  alignments: VersionAlignment[];
  selectedIds: string[];
  onConfirm: (resolution: { selectedVersion: string; alignmentId: string }[]) => void;
  isApplying?: boolean;
}

const BatchResolutionDialog: React.FC<BatchResolutionDialogProps> = ({
  open,
  onClose,
  alignments,
  selectedIds,
  onConfirm,
  isApplying = false,
}) => {
  const theme = useTheme();
  const [selectedResolutions, setSelectedResolutions] = useState<Record<string, string>>({});

  const selectedAlignments = alignments.filter((align) => selectedIds.includes(align.id));

  // Initialize selected resolutions with suggested versions
  React.useEffect(() => {
    const initialResolutions = selectedAlignments.reduce(
      (acc, align) => ({
        ...acc,
        [align.id]: align.suggestedVersion,
      }),
      {} as Record<string, string>
    );
    setSelectedResolutions(initialResolutions);
  }, [selectedIds, alignments]);

  const handleVersionSelect = (alignmentId: string, version: string) => {
    setSelectedResolutions((prev) => ({
      ...prev,
      [alignmentId]: version,
    }));
  };

  const handleConfirm = () => {
    const resolutions = Object.entries(selectedResolutions).map(([alignmentId, version]) => ({
      alignmentId,
      selectedVersion: version,
    }));
    onConfirm(resolutions);
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'high':
        return <ErrorIcon color="error" fontSize="small" />;
      case 'medium':
        return <WarningIcon color="warning" fontSize="small" />;
      case 'low':
        return <InfoIcon color="info" fontSize="small" />;
      default:
        return <InfoIcon color="info" fontSize="small" />;
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="md"
      fullWidth
      aria-labelledby="batch-resolution-dialog-title"
    >
      <DialogTitle id="batch-resolution-dialog-title">Batch Resolve Conflicts</DialogTitle>

      <DialogContent dividers>
        <Alert severity="info" sx={{ mb: 2 }}>
          <AlertTitle>Resolving {selectedAlignments.length} Dependencies</AlertTitle>
          Review and confirm the version changes below. You can modify the selected version for each
          dependency.
        </Alert>

        <Paper variant="outlined" sx={{ maxHeight: 400, overflow: 'auto', mb: 2 }}>
          <List dense>
            {selectedAlignments.map((alignment, index) => (
              <React.Fragment key={alignment.id}>
                <ListItem
                  alignItems="flex-start"
                  secondaryAction={
                    <Box display="flex" alignItems="center">
                      <Typography variant="body2" color="text.secondary" sx={{ mr: 1 }}>
                        Current: {Object.values(alignment.currentVersions)[0]}
                      </Typography>
                      <Typography variant="body2" color="primary" sx={{ fontWeight: 'medium' }}>
                        → {selectedResolutions[alignment.id]}
                      </Typography>
                    </Box>
                  }
                >
                  <ListItemIcon sx={{ minWidth: 36 }}>
                    {getSeverityIcon(alignment.severity)}
                  </ListItemIcon>
                  <ListItemText
                    primary={
                      <Typography variant="subtitle2">{alignment.dependencyName}</Typography>
                    }
                    secondary={
                      <Typography variant="caption" color="text.secondary">
                        {alignment.affectedPackages.length} package
                        {alignment.affectedPackages.length !== 1 ? 's' : ''}
                      </Typography>
                    }
                  />
                </ListItem>
                {index < selectedAlignments.length - 1 && <Divider component="li" />}
              </React.Fragment>
            ))}
          </List>
        </Paper>

        {isApplying && (
          <Box sx={{ mt: 2 }}>
            <Typography variant="body2" color="text.secondary" gutterBottom>
              Applying changes...
            </Typography>
            <LinearProgress />
          </Box>
        )}
      </DialogContent>

      <DialogActions sx={{ p: 2 }}>
        <Button onClick={onClose} color="inherit" disabled={isApplying}>
          Cancel
        </Button>
        <Button
          onClick={handleConfirm}
          variant="contained"
          color="primary"
          disabled={isApplying || selectedAlignments.length === 0}
          startIcon={<CheckIcon />}
        >
          Apply {selectedAlignments.length} Change{selectedAlignments.length !== 1 ? 's' : ''}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default BatchResolutionDialog;
