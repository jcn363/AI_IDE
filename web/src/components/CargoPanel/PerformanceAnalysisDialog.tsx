/**
 * Dialog component for displaying performance analysis
 * Shows cargo performance metrics and analysis
 */

import React from 'react';
import { Dialog, DialogTitle, DialogContent, Typography, CircularProgress } from '@mui/material';

interface PerformanceAnalysisDialogProps {
  open: boolean;
  onClose: () => void;
  projectPath: string | null;
}

/**
 * Modal dialog for performance analysis
 */
export const PerformanceAnalysisDialog: React.FC<PerformanceAnalysisDialogProps> = ({
  open,
  onClose,
  projectPath,
}) => {
  return (
    <Dialog open={open} onClose={onClose} maxWidth="lg" fullWidth fullScreen>
      <DialogTitle>Performance Analysis</DialogTitle>
      <DialogContent dividers>
        {projectPath ? (
          <Typography variant="body1">
            Performance analysis for {projectPath}
            {/* TODO: Integrate with actual PerformancePanel component */}
          </Typography>
        ) : (
          <CircularProgress />
        )}
      </DialogContent>
    </Dialog>
  );
};

export default PerformanceAnalysisDialog;