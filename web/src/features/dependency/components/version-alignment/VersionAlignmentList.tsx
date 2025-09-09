import React, { useMemo } from 'react';
import { VersionAlignment } from '../../types';
import { 
  Box, 
  Checkbox, 
  CircularProgress, 
  Divider, 
  FormControlLabel, 
  List, 
  Paper, 
  Typography,
  Skeleton,
  Stack,
  useTheme,
  alpha
} from '@mui/material';
import VersionAlignmentItem from './VersionAlignmentItem';
import { Check as CheckIcon } from '@mui/icons-material';

interface VersionAlignmentListProps {
  alignments: VersionAlignment[];
  selectedIds: string[];
  loading: boolean;
  onSelect: () => void;
  onSelectOne: (id: string) => void;
  onApplyAlignment: (alignment: VersionAlignment) => void | Promise<void>;
  onIgnoreAlignment: (alignment: VersionAlignment) => void;
  onResolveConflict?: (alignment: VersionAlignment, selectedVersion: string) => Promise<void> | void;
  isApplying?: boolean;
  isBulkApplying?: boolean;
}

const VersionAlignmentList: React.FC<VersionAlignmentListProps> = ({
  alignments,
  selectedIds,
  loading,
  onSelect,
  onSelectOne,
  onApplyAlignment,
  onIgnoreAlignment,
  onResolveConflict,
  isApplying = false,
  isBulkApplying = false,
}) => {
  const theme = useTheme();
  
  const loadingSkeletons = useMemo(() => (
    <Paper elevation={2}>
      <Box p={2}>
        <Stack spacing={2}>
          {[1, 2, 3].map((i) => (
            <Paper 
              key={i} 
              variant="outlined" 
              sx={{ 
                p: 2, 
                backgroundColor: alpha(theme.palette.primary.main, 0.02),
                borderRadius: 1,
              }}
            >
              <Stack direction="row" alignItems="center" spacing={2}>
                <Skeleton variant="circular" width={24} height={24} />
                <Skeleton width="30%" height={32} />
                <Skeleton width="15%" height={32} />
              </Stack>
              <Box mt={1}>
                <Skeleton width="40%" height={24} />
              </Box>
            </Paper>
          ))}
        </Stack>
      </Box>
    </Paper>
  ), [theme]);

  if (loading && alignments.length === 0) {
    return loadingSkeletons;
  }

  if (alignments.length === 0) {
    return (
      <Paper elevation={2}>
        <Box p={4} textAlign="center" color="text.secondary">
          <CheckIcon sx={{ fontSize: 48, mb: 2, opacity: 0.7 }} />
          <Typography variant="h6" gutterBottom>
            All dependencies are aligned
          </Typography>
          <Typography variant="body2">
            No version mismatches found in your workspace.
          </Typography>
        </Box>
      </Paper>
    );
  }

  const allSelected = useMemo(() => 
    alignments.length > 0 && alignments.every(a => selectedIds.includes(a.id)),
    [alignments, selectedIds]
  );
  
  const indeterminate = useMemo(() => 
    !allSelected && alignments.some(a => selectedIds.includes(a.id)),
    [allSelected, alignments, selectedIds]
  );

  const renderContent = () => {
    if (loading) {
      return loadingSkeletons;
    }

    return (
      <Paper elevation={2}>
        <Box p={2}>
          <Box 
            display="flex" 
            alignItems="center" 
            mb={2} 
            gap={2}
            sx={{
              position: 'sticky',
              top: 0,
              zIndex: 2,
              backgroundColor: 'background.paper',
              py: 1,
              borderBottom: `1px solid ${theme.palette.divider}`,
            }}
          >
            <FormControlLabel
              control={
                <Checkbox
                  checked={allSelected}
                  indeterminate={indeterminate}
                  onChange={onSelect}
                  disabled={loading || isBulkApplying}
                  icon={<span style={{ width: 20, height: 20 }} />}
                  checkedIcon={
                    <Box
                      sx={{
                        width: 20,
                        height: 20,
                        borderRadius: '4px',
                        backgroundColor: 'primary.main',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: 'primary.contrastText',
                      }}
                    >
                      <CheckIcon sx={{ fontSize: 16 }} />
                    </Box>
                  }
                />
              }
              label={
                <Typography variant="body2" color="text.secondary">
                  {selectedIds.length} selected
                </Typography>
              }
              sx={{ m: 0 }}
            />
            
            {isBulkApplying && (
              <Box 
                display="flex" 
                alignItems="center" 
                gap={1} 
                ml="auto"
                sx={{
                  px: 1.5,
                  py: 0.5,
                  borderRadius: 1,
                  backgroundColor: alpha(theme.palette.primary.main, 0.08),
                }}
              >
                <CircularProgress size={16} color="inherit" />
                <Typography variant="body2" color="primary">
                  Applying changes...
                </Typography>
              </Box>
            )}
          </Box>
          
          <List disablePadding>
            {alignments.map((alignment, index) => (
              <React.Fragment key={`${alignment.id}-${index}`}>
                {index > 0 && <Divider component="li" />}
                <VersionAlignmentItem
                  alignment={alignment}
                  selected={selectedIds.includes(alignment.id)}
                  onSelect={() => onSelectOne(alignment.id)}
                  onApply={onApplyAlignment}
                  onIgnore={onIgnoreAlignment}
                  onResolveConflict={onResolveConflict}
                  disabled={isApplying || isBulkApplying}
                />
              </React.Fragment>
            ))}
          </List>
          
          {alignments.length === 0 && (
            <Box 
              display="flex" 
              flexDirection="column" 
              alignItems="center" 
              justifyContent="center" 
              py={4}
              color="text.secondary"
            >
              <CheckIcon sx={{ fontSize: 48, mb: 1, opacity: 0.5 }} />
              <Typography variant="body1" color="inherit">
                No version mismatches found
              </Typography>
              <Typography variant="body2" color="inherit" mt={1}>
                All dependencies are properly aligned.
              </Typography>
            </Box>
          )}
        </Box>
      </Paper>
    );
  };

  return (
    <Box sx={{ position: 'relative' }}>
      {renderContent()}
    </Box>
  );
};

export default VersionAlignmentList;
