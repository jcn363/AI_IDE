import React, { useState, useMemo, useCallback } from 'react';
import {
  Box,
  Typography,
  Paper,
  Button,
  Chip,
  IconButton,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Badge,
  Tooltip,
  Collapse,
  FormControlLabel,
  Switch,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  TextField,
  LinearProgress,
} from '@mui/material';
import {
  ExpandMore,
  Close,
  FilterList,
  Build,
  CheckCircle,
  Psychology,
  Transform,
  Code,
  Warning,
} from '@mui/icons-material';

import refactoringService from '../services/RefactoringService';

import type {
  RefactoringConfiguration,
  RefactoringContext,
  RefactoringType,
} from '../../../types/refactoring';

/**
 * Props for the RefactoringPanel component
 */
interface RefactoringPanelProps {
  visible: boolean;
  onClose: () => void;
  onApplyRefactoring: (
    type: RefactoringType,
    context: RefactoringContext,
    options: Record<string, any>
  ) => Promise<void>;
  availableRefactorings: RefactoringType[];
  currentContext: RefactoringContext | null;
  configuration: RefactoringConfiguration;
  onConfigurationUpdate: (config: Partial<RefactoringConfiguration>) => void;
  isAnalyzing?: boolean;
  analysisProgress?: number;
}

/**
 * Utility function to get the icon for a refactoring type
 */
const getRefactoringIcon = (type: RefactoringType): React.ReactElement => {
  const iconMap: Record<string, React.ReactElement> = {
    rename: <Transform sx={{ fontSize: 20 }} />,
    'extract-function': <Build sx={{ fontSize: 20 }} />,
    'extract-variable': <Build sx={{ fontSize: 20 }} />,
    'extract-interface': <Code sx={{ fontSize: 20 }} />,
    'move-method': <Code sx={{ fontSize: 20 }} />,
    'move-class': <Code sx={{ fontSize: 20 }} />,
    'inline-method': <CheckCircle sx={{ fontSize: 20 }} />,
    'convert-to-async': <Code sx={{ fontSize: 20 }} />,
    'pattern-conversion': <Psychology sx={{ fontSize: 20 }} />,
  };

  return iconMap[type] || <Psychology sx={{ fontSize: 20 }} />;
};

/**
 * Utility function to get display name for a refactoring type
 */
const getRefactoringDisplayName = (type: RefactoringType): string => {
  const nameMap: Record<string, string> = {
    rename: 'Rename',
    'extract-function': 'Extract Function',
    'extract-variable': 'Extract Variable',
    'extract-interface': 'Extract Interface',
    'move-method': 'Move Method',
    'move-class': 'Move Class',
    'inline-method': 'Inline Method',
    'convert-to-async': 'Convert to Async',
    'pattern-conversion': 'Pattern Conversion',
  };

  return nameMap[type] || 'Advanced Refactoring';
};

/**
 * Utility function to get description for a refactoring type
 */
const getRefactoringDescription = (type: RefactoringType): string => {
  const descMap: Record<string, string> = {
    rename: 'Safely rename symbols, variables, functions, or classes',
    'extract-function': 'Extract selected code into a new function',
    'extract-variable': 'Replace expression with a meaningful variable',
    'extract-interface': 'Extract common methods into an interface',
    'move-method': 'Move method to a different location',
    'move-class': 'Move class to a different module',
    'inline-method': 'Replace method call with method body',
    'convert-to-async': 'Convert to async/await pattern',
    'pattern-conversion': 'Apply design pattern transformations',
  };

  return descMap[type] || 'Advanced refactoring operation';
};

/**
 * Main refactoring panel component with clean architecture
 */
export const RefactoringPanel: React.FC<RefactoringPanelProps> = ({
  visible,
  onClose,
  onApplyRefactoring,
  availableRefactorings = [],
  currentContext,
  configuration,
  onConfigurationUpdate,
  isAnalyzing = false,
  analysisProgress = 0,
}) => {
  // Local state management
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  const [selectedType, setSelectedType] = useState<RefactoringType | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [refactoringInProgress, setRefactoringInProgress] = useState<RefactoringType | null>(null);

  // Categorize refactorings
  const refactoringsByCategory = useMemo(() => {
    const categories: Record<string, RefactoringType[]> = {
      'Basic Operations': ['rename', 'extract-function', 'extract-variable'],
      'Advanced Operations': ['extract-interface', 'move-method', 'move-class'],
      'Code Improvement': ['inline-method', 'convert-to-async', 'pattern-conversion'],
    };

    // Filter categories based on available refactorings
    const filtered: Record<string, RefactoringType[]> = {};
    Object.entries(categories).forEach(([category, types]) => {
      const availableInCategory = types.filter((type) => availableRefactorings.includes(type));
      if (availableInCategory.length > 0) {
        filtered[category] = availableInCategory;
      }
    });

    return filtered;
  }, [availableRefactorings]);

  // Event handlers
  const handleConfigurationUpdate = useCallback(
    (updates: Partial<RefactoringConfiguration>) => {
      onConfigurationUpdate(updates);
    },
    [onConfigurationUpdate]
  );

  const toggleCategory = useCallback((category: string) => {
    setExpandedCategories((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(category)) {
        newSet.delete(category);
      } else {
        newSet.add(category);
      }
      return newSet;
    });
  }, []);

  const handleRefactoringSelect = useCallback(
    async (type: RefactoringType) => {
      if (!currentContext) return;

      try {
        setSelectedType(type);
        setRefactoringInProgress(type);

        // Execute refactoring with default options
        await onApplyRefactoring(type, currentContext, {
          createBackup: true,
          scope: configuration.defaultOptions?.scope ?? 'file',
        });

        setSelectedType(null);
      } catch (error) {
        console.error('Refactoring operation failed:', error);
      } finally {
        setRefactoringInProgress(null);
      }
    },
    [currentContext, configuration, onApplyRefactoring]
  );

  // Don't render if not visible
  if (!visible) return null;

  return (
    <Paper
      elevation={3}
      sx={{
        position: 'absolute',
        top: 16,
        right: 16,
        width: 500,
        maxHeight: '80vh',
        overflow: 'hidden',
        zIndex: 1300,
        display: 'flex',
        flexDirection: 'column',
        bgcolor: 'background.paper',
      }}
    >
      {/* Header */}
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          p: 2,
          borderBottom: '1px solid',
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="h6" component="h2">
            Code Refactoring
          </Typography>
          <Chip label={`${availableRefactorings.length} Available`} color="primary" size="small" />
        </Box>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Tooltip title="Advanced Configuration">
            <IconButton
              onClick={() => setShowAdvanced(!showAdvanced)}
              color={showAdvanced ? 'primary' : 'default'}
              size="small"
              disabled={refactoringInProgress !== null}
            >
              <FilterList />
            </IconButton>
          </Tooltip>
          <Tooltip title="Close panel">
            <IconButton onClick={onClose} size="small">
              <Close />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Analysis Progress */}
      {isAnalyzing && (
        <Box sx={{ p: 1.5, borderBottom: '1px solid', borderColor: 'divider' }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
            <Typography variant="body2" color="text.secondary">
              Analyzing Refactoring Opportunities
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {Math.round(analysisProgress)}%
            </Typography>
          </Box>
          <LinearProgress variant="determinate" value={analysisProgress} />
        </Box>
      )}

      {/* Advanced Configuration */}
      <Collapse in={showAdvanced}>
        <Box
          sx={{
            p: 2,
            borderBottom: '1px solid',
            borderColor: 'divider',
            bgcolor: 'grey.50',
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
          }}
        >
          <Typography variant="h6" sx={{ fontSize: '1rem' }}>
            Advanced Configuration
          </Typography>

          <Box sx={{ display: 'flex', gap: 2, flexDirection: 'column' }}>
            <FormControlLabel
              control={
                <Switch
                  checked={configuration.previewBeforeApply ?? false}
                  onChange={(e) =>
                    handleConfigurationUpdate({
                      previewBeforeApply: e.target.checked,
                    })
                  }
                  size="small"
                />
              }
              label="Preview Changes Before Apply"
            />

            <FormControlLabel
              control={
                <Switch
                  checked={configuration.confirmDestructiveChanges ?? true}
                  onChange={(e) =>
                    handleConfigurationUpdate({
                      confirmDestructiveChanges: e.target.checked,
                    })
                  }
                  size="small"
                />
              }
              label="Confirm Destructive Changes"
            />

            <FormControlLabel
              control={
                <Switch
                  checked={configuration.defaultOptions?.createBackup ?? true}
                  onChange={(e) =>
                    handleConfigurationUpdate({
                      defaultOptions: {
                        ...configuration.defaultOptions,
                        createBackup: e.target.checked,
                      },
                    })
                  }
                  size="small"
                />
              }
              label="Create Backups"
            />

            <FormControl size="small" fullWidth>
              <InputLabel>Default Scope</InputLabel>
              <Select
                value={configuration.defaultOptions?.scope ?? 'file'}
                label="Default Scope"
                onChange={(e) =>
                  handleConfigurationUpdate({
                    defaultOptions: {
                      ...configuration.defaultOptions,
                      scope: e.target.value as string,
                    },
                  })
                }
              >
                <MenuItem value="file">Current File</MenuItem>
                <MenuItem value="module">Current Module</MenuItem>
                <MenuItem value="workspace">Entire Workspace</MenuItem>
              </Select>
            </FormControl>
          </Box>
        </Box>
      </Collapse>

      {/* Content */}
      <Box sx={{ flex: 1, overflowY: 'auto' }}>
        {!currentContext ? (
          <Box sx={{ p: 3, textAlign: 'center' }}>
            <Warning color="warning" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h6" color="text.secondary" gutterBottom>
              No Code Context Available
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Move the cursor to a valid location in your code to see available refactorings.
            </Typography>
          </Box>
        ) : availableRefactorings.length === 0 ? (
          <Box sx={{ p: 3, textAlign: 'center' }}>
            <CheckCircle color="success" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h6" color="text.secondary" gutterBottom>
              No Refactoring Opportunities
            </Typography>
            <Typography variant="body2" color="text.secondary">
              This code appears to be well-structured!
            </Typography>
          </Box>
        ) : (
          <Box>
            {/* Refactoring Categories */}
            {Object.entries(refactoringsByCategory).map(([category, types]) => {
              const availableCount = types.length;

              return (
                <Accordion
                  key={category}
                  expanded={expandedCategories.has(category)}
                  onChange={() => toggleCategory(category)}
                  sx={{ '&:before': { display: 'none' } }}
                >
                  <AccordionSummary expandIcon={<ExpandMore />}>
                    <Box
                      sx={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 1,
                        width: '100%',
                      }}
                    >
                      <Typography variant="subtitle2" sx={{ flex: 1 }}>
                        {category}
                      </Typography>
                      <Badge badgeContent={availableCount} color="primary" />
                    </Box>
                  </AccordionSummary>
                  <AccordionDetails sx={{ p: 1 }}>
                    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                      {types.map((type) => (
                        <Tooltip
                          key={type}
                          title={getRefactoringDescription(type)}
                          placement="left"
                        >
                          <Button
                            fullWidth
                            variant={selectedType === type ? 'contained' : 'outlined'}
                            color={selectedType === type ? 'primary' : 'inherit'}
                            disabled={refactoringInProgress !== null}
                            onClick={() => handleRefactoringSelect(type)}
                            sx={{
                              justifyContent: 'flex-start',
                              py: 1.5,
                              textAlign: 'left',
                              display: 'flex',
                              alignItems: 'center',
                              gap: 2,
                            }}
                            startIcon={getRefactoringIcon(type)}
                          >
                            <Box sx={{ flex: 1, textAlign: 'left' }}>
                              <Typography
                                variant="body2"
                                sx={{
                                  fontWeight: selectedType === type ? 'bold' : 'normal',
                                  whiteSpace: 'normal',
                                  lineHeight: 1.2,
                                }}
                              >
                                {getRefactoringDisplayName(type)}
                              </Typography>
                              <Typography
                                variant="caption"
                                color="text.secondary"
                                sx={{
                                  display: 'block',
                                  mt: 0.5,
                                  whiteSpace: 'normal',
                                  lineHeight: 1.2,
                                }}
                              >
                                {getRefactoringDescription(type)}
                              </Typography>
                            </Box>
                          </Button>
                        </Tooltip>
                      ))}
                    </Box>
                  </AccordionDetails>
                </Accordion>
              );
            })}
          </Box>
        )}
      </Box>

      {/* Progress overlay */}
      {refactoringInProgress && (
        <Box
          sx={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            bgcolor: 'rgba(255, 255, 255, 0.9)',
            zIndex: 1301,
          }}
        >
          <Paper sx={{ p: 3, textAlign: 'center', minWidth: 250 }}>
            <Build color="primary" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h6" gutterBottom>
              Applying {getRefactoringDisplayName(refactoringInProgress)}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Please wait...
            </Typography>
            <LinearProgress sx={{ mt: 2 }} />
          </Paper>
        </Box>
      )}
    </Paper>
  );
};

export default RefactoringPanel;
