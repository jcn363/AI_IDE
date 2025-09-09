import React, { useState, useEffect } from 'react';
import {
  Box, Typography, FormControl, FormLabel, RadioGroup,
  FormControlLabel, Radio, Checkbox, TextField, Alert,
  List, ListItem, ListItemText, Chip, Button, IconButton,
  Divider, Paper
} from '@mui/material';
import { Add, Delete, DragHandle } from '@mui/icons-material';
import { WizardStepProps } from './RefactoringWizard';
import type { RefactoringContext, RefactoringType } from '../../../../types/refactoring';

interface BatchOperation {
  id: string;
  refactoringType: RefactoringType;
  context?: RefactoringContext;
  dependencies: string[];
  options: Record<string, any>;
}

export const BatchRefactoringWizard: React.FC<WizardStepProps> = ({
  context,
  config,
  onConfigChange,
  onValidation,
}) => {
  const [operations, setOperations] = useState<BatchOperation[]>([]);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);
  const [stopOnFirstError, setStopOnFirstError] = useState(false);
  const [createBackup, setCreateBackup] = useState(true);
  const [parallelExecution, setParallelExecution] = useState(false);

  useEffect(() => {
    const errors: string[] = [];

    if (operations.length === 0) {
      errors.push("At least one refactoring operation must be added");
    }

    // Check for cycles in dependencies
    const hasCycles = checkForDependencyCycles(operations);
    if (hasCycles) {
      errors.push("Circular dependencies detected in batch operations");
    }

    setValidationErrors(errors);
    onValidation?.(errors.length === 0);

    onConfigChange({
      operations,
      stopOnFirstError,
      createBackup,
      parallelExecution,
      validateIndependently: true,
    });
  }, [operations, stopOnFirstError, createBackup, parallelExecution, onConfigChange, onValidation]);

  const checkForDependencyCycles = (ops: BatchOperation[]): boolean => {
    // Simple cycle detection for now - could be enhanced
    const visited = new Set<string>();

    for (const op of ops) {
      if (visited.has(op.id)) continue;
      visited.add(op.id);

      for (const dep of op.dependencies) {
        if (visited.has(dep)) return true;
      }
    }

    return false;
  };

  const addOperation = () => {
    const newOp: BatchOperation = {
      id: `op_${Date.now()}`,
      refactoringType: 'rename' as RefactoringType,
      dependencies: [],
      options: {},
    };

    setOperations([...operations, newOp]);
  };

  const removeOperation = (id: string) => {
    setOperations(operations.filter(op => op.id !== id));
  };

  const updateOperation = (id: string, updates: Partial<BatchOperation>) => {
    setOperations(operations.map(op =>
      op.id === id ? { ...op, ...updates } : op
    ));
  };

  const moveOperation = (index: number, direction: 'up' | 'down') => {
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= operations.length) return;

    const newOps = [...operations];
    [newOps[index], newOps[newIndex]] = [newOps[newIndex], newOps[index]];
    setOperations(newOps);
  };

  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="subtitle1" gutterBottom>
        Batch Refactoring Configuration
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        Configure multiple refactoring operations to execute together.
        Operations can have dependencies and will be executed in order.
      </Typography>

      <Box sx={{ mb: 4 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="subtitle2">Operations</Typography>
          <Button variant="outlined" size="small" onClick={addOperation}>
            Add Operation
          </Button>
        </Box>

        {operations.length === 0 ? (
          <Paper sx={{ p: 3, textAlign: 'center', bgcolor: 'grey.50' }}>
            <Typography variant="body2" color="text.secondary">
              No operations added yet. Click "Add Operation" to get started.
            </Typography>
          </Paper>
        ) : (
          <List>
            {operations.map((operation, index) => (
              <Paper key={operation.id} sx={{ mb: 2, p: 2 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                  <IconButton
                    size="small"
                    sx={{ cursor: 'move' }}
                    onClick={() => moveOperation(index, 'up')}
                    disabled={index === 0}
                  >
                    <DragHandle />
                  </IconButton>

                  <Box sx={{ flex: 1 }}>
                    <TextField
                      label="Operation Type"
                      select
                      size="small"
                      value={operation.refactoringType}
                      onChange={(e) => updateOperation(operation.id, {
                        refactoringType: e.target.value as RefactoringType
                      })}
                      sx={{ mr: 2, minWidth: 150 }}
                    >
                      <option value="rename">Rename</option>
                      <option value="extract-method">Extract Method</option>
                      <option value="extract-variable">Extract Variable</option>
                      {/* Add more types as needed */}
                    </TextField>

                    <Chip
                      label={`ID: ${operation.id}`}
                      size="small"
                      variant="outlined"
                    />
                  </Box>

                  <IconButton
                    size="small"
                    onClick={() => moveOperation(index, 'down')}
                    disabled={index === operations.length - 1}
                  >
                    <DragHandle />
                  </IconButton>

                  <IconButton
                    size="small"
                    color="error"
                    onClick={() => removeOperation(operation.id)}
                  >
                    <Delete />
                  </IconButton>
                </Box>
              </Paper>
            ))}
          </List>
        )}
      </Box>

      <Divider sx={{ my: 3 }} />

      <Box sx={{ mb: 4 }}>
        <Typography variant="subtitle2" gutterBottom>
          Batch Execution Options
        </Typography>

        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={stopOnFirstError}
                onChange={(e) => setStopOnFirstError(e.target.checked)}
              />
            }
            label="Stop batch on first error"
          />

          <FormControlLabel
            control={
              <Checkbox
                checked={createBackup}
                onChange={(e) => setCreateBackup(e.target.checked)}
              />
            }
            label="Create backup before batch execution"
          />

          <FormControlLabel
            control={
              <Checkbox
                checked={parallelExecution}
                onChange={(e) => setParallelExecution(e.target.checked)}
              />
            }
            label="Execute operations in parallel (where possible)"
          />
        </Box>
      </Box>

      {validationErrors.length > 0 && (
        <Alert severity="error" sx={{ mb: 2 }}>
          <Typography variant="body2" sx={{ mb: 1 }}>
            <strong>Validation Errors:</strong>
          </Typography>
          <ul style={{ margin: 0, paddingLeft: 20 }}>
            {validationErrors.map((error, index) => (
              <li key={index}>{error}</li>
            ))}
          </ul>
        </Alert>
      )}

      <Box sx={{ mt: 3, p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
        <Typography variant="body2" color="text.secondary">
          <strong>Summary:</strong> {operations.length} operation(s) configured for batch execution.
          {operations.length > 0 && ' Operations will be executed sequentially.'}
        </Typography>
      </Box>
    </Box>
  );
};