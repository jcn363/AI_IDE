import React, { useState, useEffect } from 'react';
import {
  Box, Typography, FormControl, FormLabel, RadioGroup,
  FormControlLabel, Radio, Checkbox, TextField, Alert,
  List, ListItem, ListItemText, Chip, Collapse, Button
} from '@mui/material';
import { WizardStepProps } from './RefactoringWizard';
import type { RefactoringContext } from '../../../../types/refactoring';

export const PatternConversionWizard: React.FC<WizardStepProps> = ({
  context,
  config,
  onConfigChange,
  onValidation,
}) => {
  const [detectedPatterns, setDetectedPatterns] = useState<Array<{id: string, name: string, description: string, confidence: number}>>([]);
  const [selectedPatterns, setSelectedPatterns] = useState<string[]>([]);
  const [conversionTarget, setConversionTarget] = useState<'async-await' | 'modern-syntax' | 'functional' | 'oop'>('modern-syntax');
  const [generateTests, setGenerateTests] = useState(false);
  const [analyzeImpact, setAnalyzeImpact] = useState(true);
  const [customPattern, setCustomPattern] = useState('');

  useEffect(() => {
    // Simulate analyzing code for pattern conversion opportunities
    const mockPatterns = [
      {
        id: 'callback-to-async',
        name: 'Callback to Promise/Async-Await',
        description: 'Convert callback-based functions to async/await',
        confidence: 0.85,
      },
      {
        id: 'closure-patterns',
        name: 'Closure Optimization',
        description: 'Optimize closure patterns and reduce memory leaks',
        confidence: 0.72,
      },
      {
        id: 'traditional-loop-to-functional',
        name: 'Traditional Loop to Functional',
        description: 'Convert for/while loops to functional programming patterns',
        confidence: 0.68,
      },
      {
        id: 'error-handling-modernization',
        name: 'Error Handling Modernization',
        description: 'Convert try-catch to Result types or modern error patterns',
        confidence: 0.79,
      },
      {
        id: 'data-structure-conversion',
        name: 'Data Structure Optimization',
        description: 'Replace arrays/vectors with more efficient data structures',
        confidence: 0.61,
      }
    ];
    setDetectedPatterns(mockPatterns);

    // Pre-select high-confidence patterns
    setSelectedPatterns(
      mockPatterns
        .filter(p => p.confidence >= 0.7)
        .map(p => p.id)
    );
  }, [context]);

  useEffect(() => {
    onValidation?.(selectedPatterns.length > 0);

    onConfigChange({
      selectedPatternIds: selectedPatterns,
      conversionTarget,
      customPatternText: customPattern.trim(),
      shouldGenerateTests: generateTests,
      shouldAnalyzeImpact: analyzeImpact,
      targetType: 'pattern',
    });
  }, [selectedPatterns, conversionTarget, customPattern, generateTests, analyzeImpact, onConfigChange, onValidation]);

  const handlePatternToggle = (patternId: string) => {
    setSelectedPatterns(prev =>
      prev.includes(patternId)
        ? prev.filter(id => id !== patternId)
        : [...prev, patternId]
    );
  };

  const getSelectedPatternDetails = () => {
    return detectedPatterns.filter(p => selectedPatterns.includes(p.id));
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'success';
    if (confidence >= 0.6) return 'warning';
    return 'error';
  };

  const handleSelectHighConfidence = () => {
    const highConfidence = detectedPatterns
      .filter(p => p.confidence >= 0.7)
      .map(p => p.id);
    setSelectedPatterns(highConfidence);
  };

  const handleClearSelection = () => {
    setSelectedPatterns([]);
  };

  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="subtitle1" gutterBottom>
        Pattern Conversion Configuration
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        Select which code patterns should be converted to modern or more efficient alternatives.
        The wizard analyzes your code and suggests optimizations.
      </Typography>

      <Box sx={{ mb: 4 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="subtitle2">Detected Patterns</Typography>
          <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
            <Chip label={`Selected: ${selectedPatterns.length}`} size="small" color="primary" />
            <Chip label={`Total: ${detectedPatterns.length}`} size="small" variant="outlined" />
            <Button size="small" onClick={handleSelectHighConfidence} variant="text">
              High Confidence Only
            </Button>
            <Button size="small" onClick={handleClearSelection} variant="text">
              Clear
            </Button>
          </Box>
        </Box>

        <List sx={{ maxHeight: 300, overflowY: 'auto', border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
          {detectedPatterns.map((pattern) => (
            <ListItem key={pattern.id} dense sx={{ flexDirection: 'column', alignItems: 'stretch' }}>
              <Box sx={{ display: 'flex', alignItems: 'flex-start', width: '100%', mb: 1 }}>
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={selectedPatterns.includes(pattern.id)}
                      onChange={() => handlePatternToggle(pattern.id)}
                      size="small"
                      disabled={pattern.confidence < 0.5}
                    />
                  }
                  label={
                    <Box>
                      <Typography variant="body2" fontWeight="medium">
                        {pattern.name}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {pattern.description}
                      </Typography>
                    </Box>
                  }
                  sx={{ flex: 1 }}
                />
                <Chip
                  label={`${Math.round(pattern.confidence * 100)}%`}
                  size="small"
                  color={getConfidenceColor(pattern.confidence)}
                  variant="outlined"
                />
              </Box>

              <Collapse in={selectedPatterns.includes(pattern.id)}>
                <Box sx={{ ml: 4, mb: 2 }}>
                  <Typography variant="caption" color="text.secondary">
                    <strong>Impact:</strong> High confidence conversion detected. This pattern
                    can be safely converted with minimal risk.
                  </Typography>
                </Box>
              </Collapse>
            </ListItem>
          ))}
        </List>
      </Box>

      {selectedPatterns.length > 0 && (
        <Box sx={{ mb: 4 }}>
          <Typography variant="subtitle2" gutterBottom>
            Conversion Target
          </Typography>

          <FormControl component="fieldset">
            <FormLabel component="legend">Choose the conversion style</FormLabel>
            <RadioGroup
              value={conversionTarget}
              onChange={(e) => setConversionTarget(e.target.value as any)}
              row
            >
              <FormControlLabel value="async-await" control={<Radio />} label="Async/Await" />
              <FormControlLabel value="modern-syntax" control={<Radio />} label="Modern Syntax" />
              <FormControlLabel value="functional" control={<Radio />} label="Functional" />
              <FormControlLabel value="oop" control={<Radio />} label="Object-Oriented" />
            </RadioGroup>
          </FormControl>
        </Box>
      )}

      <Box sx={{ mb: 4 }}>
        <Typography variant="subtitle2" gutterBottom>
          Conversion Options
        </Typography>

        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={analyzeImpact}
                onChange={(e) => setAnalyzeImpact(e.target.checked)}
              />
            }
            label="Analyze potential impact before conversion"
          />

          <FormControlLabel
            control={
              <Checkbox
                checked={generateTests}
                onChange={(e) => setGenerateTests(e.target.checked)}
              />
            }
            label="Generate tests for converted patterns"
          />
        </Box>
      </Box>

      <Box sx={{ mb: 4 }}>
        <Typography variant="subtitle2" gutterBottom>
          Custom Pattern (optional)
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
          Define a custom pattern conversion (JSON format)
        </Typography>
        <TextField
          multiline
          rows={4}
          fullWidth
          placeholder='{"find": "old_pattern", "replace": "new_pattern", "language": "rust"}'
          value={customPattern}
          onChange={(e) => setCustomPattern(e.target.value)}
          size="small"
        />
      </Box>

      <Collapse in={selectedPatterns.length === 0}>
        <Alert severity="warning" sx={{ mt: 2 }}>
          Please select at least one pattern to convert.
        </Alert>
      </Collapse>

      <Box sx={{ mt: 3, p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
        <Typography variant="body2" color="text.secondary">
          <strong>Expected Changes:</strong> Convert {selectedPatterns.length} pattern(s)
          to {conversionTarget} style. {getSelectedPatternDetails().length} high-confidence
          conversions detected with minimal risk assessment.
        </Typography>
      </Box>
    </Box>
  );
};