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

export const ExtractInterfaceWizard: React.FC<StepComponentProps> = ({
  context,
  config,
  onConfigChange,
  onValidation,
}) => {
  const [detectedClasses, setDetectedClasses] = useState<string[]>([]);
  const [selectedClasses, setSelectedClasses] = useState<string[]>([]);
  const [interfaceName, setInterfaceName] = useState('');
  const [includePublicMethods, setIncludePublicMethods] = useState(true);
  const [includeProperties, setIncludeProperties] = useState(true);
  const [generateTests, setGenerateTests] = useState(false);
  const [analysisResults, setAnalysisResults] = useState<Map<string, any>>(new Map());

  useEffect(() => {
    const analyzeCode = async () => {
      if (context?.filePath) {
        try {
          const result = await refactoringService.analyzeCodeForInterfaceExtraction(
            context.filePath,
            context.symbolName // Pass target class if available
          );

          // Filter to only suitable classes
          const suitableClasses = result.classes.filter((cls) => cls.isSuitableForInterface);

          // Map to component expectations
          const classNames = suitableClasses.map((cls) => cls.className);
          const analysisMap = new Map();

          suitableClasses.forEach((cls) => {
            analysisMap.set(cls.className, {
              publicMethods: cls.publicMethods.length,
              properties: cls.publicProperties.length,
              complexity: cls.complexityScore,
              isSuitable: cls.isSuitableForInterface,
              reasonNotSuitable: cls.reasonNotSuitable,
            });
          });

          setDetectedClasses(classNames);
          setAnalysisResults(analysisMap);

          // Pre-select the most suitable class by default
          if (suitableClasses.length > 0) {
            const bestClass = suitableClasses.sort(
              (a, b) => b.complexityScore - a.complexityScore
            )[0];
            setSelectedClasses([bestClass.className]);
            setInterfaceName(`I${bestClass.className}`);
          }
        } catch (error) {
          console.error('Failed to analyze code for interface extraction:', error);
          // Fallback to empty state
          setDetectedClasses([]);
          setAnalysisResults(new Map());
        }
      }
    };

    analyzeCode();
  }, [context, context?.filePath, context?.symbolName]);

  useEffect(() => {
    const isValid = selectedClasses.length > 0 && interfaceName.trim().length > 0;
    onValidation?.(isValid);

    onConfigChange({
      selectedClasses,
      interfaceName: interfaceName.trim(),
      includePublicMethods,
      includeProperties,
      shouldGenerateTests: generateTests,
      targetType: 'interface',
    });
  }, [
    selectedClasses,
    interfaceName,
    includePublicMethods,
    includeProperties,
    generateTests,
    onConfigChange,
    onValidation,
  ]);

  const handleClassToggle = (className: string) => {
    const isSelected = selectedClasses.includes(className);

    if (isSelected) {
      setSelectedClasses((prev) => prev.filter((c) => c !== className));
      if (selectedClasses.length === 1) {
        setInterfaceName('');
      }
    } else {
      setSelectedClasses((prev) => [...prev, className]);
      if (selectedClasses.length === 0) {
        setInterfaceName(`I${className}`);
      }
    }
  };

  const getClassInfo = (className: string) => {
    const analysis = analysisResults.get(className);
    return analysis || { publicMethods: 0, properties: 0, complexity: 0 };
  };

  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="subtitle1" gutterBottom>
        Extract Interface Configuration
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        Select which classes should have interfaces extracted from them. The wizard will analyze
        public members and suggest interface contracts.
      </Typography>

      <Box sx={{ mb: 4 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="subtitle2">Detected Classes</Typography>
          <Box sx={{ display: 'flex', gap: 1 }}>
            <Chip label={`Selected: ${selectedClasses.length}`} size="small" color="primary" />
            <Chip label={`Total: ${detectedClasses.length}`} size="small" variant="outlined" />
          </Box>
        </Box>

        <List
          sx={{
            maxHeight: 250,
            overflowY: 'auto',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
          }}
        >
          {detectedClasses.map((className) => {
            const info = getClassInfo(className);
            return (
              <ListItem
                key={className}
                dense
                sx={{ flexDirection: 'column', alignItems: 'stretch' }}
              >
                <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
                  <FormControlLabel
                    control={
                      <Checkbox
                        checked={selectedClasses.includes(className)}
                        onChange={() => handleClassToggle(className)}
                        size="small"
                      />
                    }
                    label={
                      <Typography variant="body2" fontFamily="monospace" fontWeight="medium">
                        {className}
                      </Typography>
                    }
                    sx={{ flex: 1 }}
                  />
                  <Box sx={{ display: 'flex', gap: 1, ml: 1 }}>
                    <Chip
                      label={`${info.publicMethods} methods`}
                      size="small"
                      variant="outlined"
                      color="info"
                    />
                    <Chip
                      label={`${info.properties} props`}
                      size="small"
                      variant="outlined"
                      color="success"
                    />
                  </Box>
                </Box>
              </ListItem>
            );
          })}
        </List>
      </Box>

      <Collapse in={selectedClasses.length > 0}>
        <Box sx={{ mb: 4 }}>
          <Typography variant="subtitle2" gutterBottom>
            Interface Configuration
          </Typography>

          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <TextField
              label="Interface Name"
              value={interfaceName}
              onChange={(e) => setInterfaceName(e.target.value)}
              placeholder="IMyInterface"
              helperText="Name for the extracted interface"
              size="small"
              required
            />

            <FormControlLabel
              control={
                <Checkbox
                  checked={includePublicMethods}
                  onChange={(e) => setIncludePublicMethods(e.target.checked)}
                />
              }
              label="Include public methods in interface"
            />

            <FormControlLabel
              control={
                <Checkbox
                  checked={includeProperties}
                  onChange={(e) => setIncludeProperties(e.target.checked)}
                />
              }
              label="Include public properties in interface"
            />

            <FormControlLabel
              control={
                <Checkbox
                  checked={generateTests}
                  onChange={(e) => setGenerateTests(e.target.checked)}
                />
              }
              label="Generate interface contract tests"
            />
          </Box>
        </Box>
      </Collapse>

      <Collapse in={selectedClasses.length === 0}>
        <Alert severity="warning" sx={{ mt: 2 }}>
          Please select at least one class to extract an interface from.
        </Alert>
      </Collapse>

      <Collapse in={selectedClasses.length > 0 && interfaceName.trim() === ''}>
        <Alert severity="warning" sx={{ mt: 2 }}>
          Please provide a name for the interface.
        </Alert>
      </Collapse>

      <Box sx={{ mt: 3, p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
        <Typography variant="body2" color="text.secondary">
          <strong>Expected Changes:</strong> Extract interface "{interfaceName}" from{' '}
          {selectedClasses.length} class(es). The interface will include
          {includePublicMethods ? ' public methods' : ''}
          {includePublicMethods && includeProperties ? ' and' : ''}
          {includeProperties ? ' public properties' : ''}.
        </Typography>
      </Box>
    </Box>
  );
};
