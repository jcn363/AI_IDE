import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  FormControl,
  FormLabel,
  RadioGroup,
  FormControlLabel,
  Radio,
  Checkbox,
  TextField,
  Alert,
  List,
  ListItem,
  ListItemText,
  Chip,
  Collapse,
} from '@mui/material';
import { StepComponentProps } from './RefactoringWizard';
import type { RefactoringContext } from '../../../../types/refactoring';
import refactoringService from '../../../ai/services/RefactoringService';

export const AsyncAwaitConversionWizard: React.FC<StepComponentProps> = ({
  context,
  config,
  onConfigChange,
  onValidation,
}) => {
  const [conversionTargets, setConversionTargets] = useState<string[]>([]);
  const [selectedTargets, setSelectedTargets] = useState<string[]>([]);
  const [customFunctions, setCustomFunctions] = useState('');
  const [generateTests, setGenerateTests] = useState(false);
  const [analyzeDependencies, setAnalyzeDependencies] = useState(true);
  const [analysisResults, setAnalysisResults] = useState<Map<string, any>>(new Map());

  useEffect(() => {
    const analyzeCode = async () => {
      if (context?.filePath) {
        try {
          const result = await refactoringService.analyzeCodeForAsyncConversion(
            context.filePath,
            context.symbolName // Pass target function if available
          );

          // Filter to only convertible functions
          const convertibleFunctions = result.functions.filter((func) => func.isConvertibleToAsync);

          // Map to component expectations
          const functionNames = convertibleFunctions.map((func) => `${func.functionName}()`);
          const analysisMap = new Map();

          convertibleFunctions.forEach((func) => {
            analysisMap.set(`${func.functionName}()`, {
              isConvertible: func.isConvertibleToAsync,
              canBeAwaited: func.canBeAwaited,
              dependencies: func.dependencies,
              blockingOperations: func.blockingOperations,
              complexity: func.estimatedComplexity,
              signature: func.signature,
            });
          });

          setConversionTargets(functionNames);
          setAnalysisResults(analysisMap);

          // Pre-select all convertible functions by default if there are any
          if (convertibleFunctions.length > 0) {
            setSelectedTargets(functionNames);
          }
        } catch (error) {
          console.error('Failed to analyze code for async conversion:', error);
          // Fallback to empty state
          setConversionTargets([]);
          setAnalysisResults(new Map());
        }
      }
    };

    analyzeCode();
  }, [context, context?.filePath, context?.symbolName]);

  useEffect(() => {
    onValidation?.(selectedTargets.length > 0);

    onConfigChange({
      conversionTargets: selectedTargets,
      customFunctionNames: customFunctions.split('\n').filter((s) => s.trim()),
      shouldGenerateTests: generateTests,
      analyzeDependencies,
    });
  }, [
    selectedTargets,
    customFunctions,
    generateTests,
    analyzeDependencies,
    onConfigChange,
    onValidation,
  ]);

  const handleTargetToggle = (target: string) => {
    setSelectedTargets((prev) =>
      prev.includes(target) ? prev.filter((t) => t !== target) : [...prev, target]
    );
  };

  const handleSelectAll = () => {
    setSelectedTargets(conversionTargets);
  };

  const handleSelectNone = () => {
    setSelectedTargets([]);
  };

  const getFunctionInfo = (functionName: string) => {
    const analysis = analysisResults.get(functionName);
    return analysis || { blockingOperations: [], complexity: 'Unknown' };
  };

  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="subtitle1" gutterBottom>
        Async/Await Conversion Configuration
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        Select which functions should be converted to async/await pattern. The wizard will
        automatically identify promise-based functions.
      </Typography>

      <Box sx={{ mb: 4 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="subtitle2">Detected Functions</Typography>
          <Box sx={{ display: 'flex', gap: 1 }}>
            <Chip label={`Selected: ${selectedTargets.length}`} size="small" color="primary" />
            <Chip label={`Total: ${conversionTargets.length}`} size="small" variant="outlined" />
          </Box>
        </Box>

        <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
          <Chip
            label="Select All"
            onClick={handleSelectAll}
            size="small"
            variant="outlined"
            clickable
          />
          <Chip
            label="Select None"
            onClick={handleSelectNone}
            size="small"
            variant="outlined"
            clickable
          />
        </Box>

        <List
          sx={{
            maxHeight: 200,
            overflowY: 'auto',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
          }}
        >
          {conversionTargets.map((target) => {
            const info = getFunctionInfo(target);
            return (
              <ListItem key={target} dense sx={{ flexDirection: 'column', alignItems: 'stretch' }}>
                <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
                  <FormControlLabel
                    control={
                      <Checkbox
                        checked={selectedTargets.includes(target)}
                        onChange={() => handleTargetToggle(target)}
                        size="small"
                      />
                    }
                    label={
                      <Typography variant="body2" fontFamily="monospace">
                        {target}
                      </Typography>
                    }
                    sx={{ flex: 1 }}
                  />
                  <Box sx={{ display: 'flex', gap: 1, ml: 1 }}>
                    <Chip label={info.complexity} size="small" variant="outlined" color="warning" />
                  </Box>
                </Box>
                {info.blockingOperations.length > 0 && (
                  <Box sx={{ mt: 1, pl: 3 }}>
                    <Typography variant="caption" color="text.secondary">
                      Blocking operations: {info.blockingOperations.join(', ')}
                    </Typography>
                  </Box>
                )}
              </ListItem>
            );
          })}
        </List>
      </Box>

      <Box sx={{ mb: 4 }}>
        <Typography variant="subtitle2" gutterBottom>
          Additional Function Names (optional)
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
          Add custom function names to convert (one per line)
        </Typography>
        <TextField
          multiline
          rows={3}
          fullWidth
          placeholder="customFunction1()&#10;anotherFunction()&#10;processData()"
          value={customFunctions}
          onChange={(e) => setCustomFunctions(e.target.value)}
          size="small"
        />
      </Box>

      <Box sx={{ mb: 4 }}>
        <Typography variant="subtitle2" gutterBottom>
          Conversion Options
        </Typography>

        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={analyzeDependencies}
                onChange={(e) => setAnalyzeDependencies(e.target.checked)}
              />
            }
            label="Analyze function dependencies before conversion"
          />

          <FormControlLabel
            control={
              <Checkbox
                checked={generateTests}
                onChange={(e) => setGenerateTests(e.target.checked)}
              />
            }
            label="Generate unit tests for converted functions"
          />
        </Box>
      </Box>

      <Collapse in={selectedTargets.length === 0}>
        <Alert severity="warning" sx={{ mt: 2 }}>
          Please select at least one function to convert to async/await pattern.
        </Alert>
      </Collapse>

      <Box sx={{ mt: 3, p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
        <Typography variant="body2" color="text.secondary">
          <strong>Expected Changes:</strong> {selectedTargets.length} function(s) will be converted
          to use async/await syntax. Callers will need to be updated accordingly.
        </Typography>
      </Box>
    </Box>
  );
};
