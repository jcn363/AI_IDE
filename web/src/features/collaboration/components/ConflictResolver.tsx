import React, { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  Paper,
  Divider,
  TextField,
} from '@mui/material';
import { Warning, Merge, ArrowBack, ArrowForward } from '@mui/icons-material';
import type { ConflictResolverProps } from '../types';

export const ConflictResolver: React.FC<ConflictResolverProps> = ({
  conflict,
  onResolve,
  onMerge,
}) => {
  const [mergedContent, setMergedContent] = useState('');
  const [showManualMerge, setShowManualMerge] = useState(false);

  const handleResolve = (resolution: 'local' | 'remote' | 'merge') => {
    if (resolution === 'merge' && !showManualMerge && !mergedContent) {
      setShowManualMerge(true);
      return;
    }

    if (resolution === 'merge' && showManualMerge && mergedContent) {
      onMerge(mergedContent);
    } else {
      onResolve(resolution);
    }
  };

  const generateAutoMergeSuggestion = () => {
    // Simple auto-merge for demo - in real implementation would be more sophisticated
    if (conflict.localChange.content && conflict.remoteChange.content) {
      return `${conflict.localChange.content}\n${conflict.remoteChange.content}`;
    }
    return '';
  };

  const autoMerge = generateAutoMergeSuggestion();
  if (autoMerge && !mergedContent) {
    setMergedContent(autoMerge);
  }

  return (
    <Dialog open={true} maxWidth="lg" fullWidth>
      <DialogTitle>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Warning color="warning" />
          <Typography variant="h6">Conflict Resolution</Typography>
        </Box>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          {conflict.filePath} - Lines {conflict.localChange.position.startLine} to{' '}
          {conflict.localChange.position.endLine}
        </Typography>
      </DialogTitle>

      <DialogContent>
        <Box sx={{ mb: 3 }}>
          <Typography variant="subtitle1" gutterBottom>
            Conflict Type: {conflict.type.charAt(0).toUpperCase() + conflict.type.slice(1)}
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Your changes conflict with changes made by {conflict.otherUserId}
          </Typography>
        </Box>

        {!showManualMerge ? (
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <Paper sx={{ p: 2, bgcolor: 'success.light' }}>
              <Typography variant="h6" sx={{ mb: 1, color: 'success.contrastText' }}>
                Your Changes (Local)
              </Typography>
              <Typography variant="body2" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                {conflict.localChange.content || '(Empty)'}
              </Typography>
            </Paper>

            <Divider sx={{ my: 2 }}>
              <Typography variant="caption" color="text.secondary">
                VS
              </Typography>
            </Divider>

            <Paper sx={{ p: 2, bgcolor: 'info.light' }}>
              <Typography variant="h6" sx={{ mb: 1, color: 'info.contrastText' }}>
                Remote Changes ({conflict.otherUserId})
              </Typography>
              <Typography variant="body2" sx={{ fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                {conflict.remoteChange.content || '(Empty)'}
              </Typography>
            </Paper>
          </Box>
        ) : (
          <Box sx={{ mt: 2 }}>
            <Typography variant="h6" gutterBottom>
              Merge Changes
            </Typography>
            <TextField
              fullWidth
              multiline
              rows={6}
              value={mergedContent}
              onChange={(e) => setMergedContent(e.target.value)}
              placeholder="Enter merged content..."
              sx={{ fontFamily: 'monospace' }}
              helperText="Combine both changes manually or modify as needed"
            />
            {mergedContent && (
              <Button onClick={() => setMergedContent('')} size="small" sx={{ mt: 1 }}>
                Clear
              </Button>
            )}
          </Box>
        )}
      </DialogContent>

      <DialogActions>
        <Button
          onClick={() => onResolve('local')}
          startIcon={<ArrowBack />}
          variant="outlined"
          color="primary"
        >
          Use Your Changes
        </Button>

        <Button
          onClick={() => onResolve('remote')}
          endIcon={<ArrowForward />}
          variant="outlined"
          color="secondary"
        >
          Use Remote Changes
        </Button>

        <Button
          onClick={() => handleResolve('merge')}
          startIcon={<Merge />}
          variant="contained"
          color="primary"
          disabled={showManualMerge && !mergedContent.trim()}
        >
          {showManualMerge ? 'Apply Merge' : 'Merge Changes'}
        </Button>

        {!showManualMerge && (
          <Button onClick={() => setShowManualMerge(true)} variant="text" size="small">
            Manual Merge
          </Button>
        )}

        {showManualMerge && (
          <Button
            onClick={() => {
              setShowManualMerge(false);
              setMergedContent(autoMerge);
            }}
            variant="text"
            size="small"
          >
            Auto Merge
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
};
