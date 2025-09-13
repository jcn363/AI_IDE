import React, { useCallback, useEffect, useState } from 'react';
import type { WizardStepProps } from './RefactoringWizard';
import refactoringService from '../../services/RefactoringService';

interface WizardState {
  functionNames: string[];
  selectedFunctions: string[];
  generateTests: boolean;
  applyToAll: boolean;
  customOptions: {
    timeout: number;
    errorHandling: boolean;
  };
  isSupported: boolean;
  loading: boolean;
}

export const AsyncAwaitWizard: React.FC<WizardStepProps> = ({
  context,
  onConfigChange,
  onValidation,
}) => {
  const [wizardState, setWizardState] = useState<WizardState>({
    functionNames: [],
    selectedFunctions: [],
    generateTests: true,
    applyToAll: false,
    customOptions: {
      timeout: 30000,
      errorHandling: true,
    },
    isSupported: true,
    loading: true,
  });

  // Check file type support
  useEffect(() => {
    const checkSupport = async () => {
      try {
        const supported = await refactoringService.isFileTypeSupported(context.filePath);
        setWizardState((prev) => ({
          ...prev,
          isSupported: supported,
          loading: false,
        }));
      } catch (error) {
        console.warn('Failed to check file type support:', error);
        setWizardState((prev) => ({ ...prev, isSupported: false, loading: false }));
      }
    };
    checkSupport();
  }, [context.filePath]);

  // Populate function names on mount
  useEffect(() => {
    const loadFunctions = async () => {
      try {
        // First try enhanced analysis for function names
        const analysis = await refactoringService.analyzeRefactoringContextEnhanced(context);
        // TODO: Parse actual function names from enhanced analysis
        // For now, provide sample function names that could be converted to async
        const sampleFunctions = [
          'process_data',
          'fetch_user',
          'calculate_result',
          'validate_input',
          'save_file',
        ];
        setWizardState((prev) => ({
          ...prev,
          functionNames: sampleFunctions,
        }));
      } catch (error) {
        console.warn('Failed to load function names:', error);
        setWizardState((prev) => ({ ...prev, functionNames: [] }));
      }
    };

    if (wizardState.isSupported) {
      loadFunctions();
    } else {
      setWizardState((prev) => ({ ...prev, loading: false }));
    }
  }, [context, wizardState.isSupported]);

  // Update validation: require at least one selection (allow multiple for future batch support)
  useEffect(() => {
    const isValid = wizardState.isSupported && wizardState.selectedFunctions.length > 0;
    onValidation?.(isValid);
  }, [wizardState.selectedFunctions, wizardState.isSupported, onValidation]);

  // Update config when state changes - emit proper config object
  useEffect(() => {
    if (onConfigChange && !wizardState.loading) {
      if (wizardState.selectedFunctions.length === 1) {
        // Single function conversion - emit only options
        onConfigChange({
          generateTests: wizardState.generateTests,
          applyToAllOccurrences: wizardState.applyToAll,
          extraOptions: {
            functionName: wizardState.selectedFunctions[0],
            timeout: wizardState.customOptions.timeout,
            errorHandling: wizardState.customOptions.errorHandling,
          },
        });
      } else if (wizardState.selectedFunctions.length > 1) {
        // Multi-function selection (would use batch, but not currently supported by backend)
        // For now, disable the config by not emitting
        console.warn('Multi-function selection not yet supported by backend');
        // TODO: Implement batch conversion when backend supports it
      }
    }
  }, [wizardState, context, onConfigChange]);

  const handleFunctionSelection = useCallback((functionName: string) => {
    setWizardState((prev) => ({
      ...prev,
      selectedFunctions: prev.selectedFunctions.includes(functionName)
        ? prev.selectedFunctions.filter((fn) => fn !== functionName)
        : [...prev.selectedFunctions, functionName],
    }));
  }, []);

  const handleOptionChange = useCallback((key: string, value: boolean | number) => {
    setWizardState((prev) => ({
      ...prev,
      customOptions: {
        ...prev.customOptions,
        [key]: value,
      },
    }));
  }, []);

  if (wizardState.loading) {
    return (
      <div style={{ padding: '20px', maxWidth: '600px', margin: '0 auto' }}>
        <h2>Async/Await Conversion Wizard</h2>
        <p>Loading function analysis...</p>
      </div>
    );
  }

  if (!wizardState.isSupported) {
    return (
      <div style={{ padding: '20px', maxWidth: '600px', margin: '0 auto' }}>
        <h2>Async/Await Conversion Wizard</h2>
        <div
          style={{
            color: '#d32f2f',
            padding: '12px',
            background: '#fdeaea',
            borderRadius: '4px',
            border: '1px solid #f8c6c6',
          }}
        >
          <p>⚠️ Async/await conversion is only supported for Rust (.rs) files.</p>
          <p>The current file ({context.filePath}) is not supported.</p>
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: '20px', maxWidth: '600px', margin: '0 auto' }}>
      <h2>Async/Await Conversion Wizard</h2>
      <p>Convert Promise-based functions to async/await syntax</p>

      <div style={{ marginBottom: '30px' }}>
        <h3>Functions to Convert</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {wizardState.functionNames.length > 0 ? (
            wizardState.functionNames.map((funcName) => (
              <label
                key={funcName}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  padding: '8px',
                  background: wizardState.selectedFunctions.includes(funcName)
                    ? '#e3f2fd'
                    : 'white',
                  borderRadius: '4px',
                  border: '1px solid #ddd',
                  cursor: 'pointer',
                  opacity:
                    wizardState.selectedFunctions.length > 1 &&
                    wizardState.selectedFunctions.includes(funcName)
                      ? 0.6
                      : 1,
                }}
              >
                <input
                  type="checkbox"
                  checked={wizardState.selectedFunctions.includes(funcName)}
                  onChange={() => handleFunctionSelection(funcName)}
                  // Disable additional selections if one is already selected (single-function mode)
                  disabled={
                    wizardState.selectedFunctions.length >= 1 &&
                    !wizardState.selectedFunctions.includes(funcName)
                  }
                />
                <span>{funcName}</span>
                {wizardState.selectedFunctions.length > 1 &&
                  wizardState.selectedFunctions.includes(funcName) && (
                    <span style={{ fontSize: '12px', color: '#ff6b35' }}>
                      (Multi-select disabled - backend supports single function only)
                    </span>
                  )}
              </label>
            ))
          ) : (
            <p>No functions detected in the current context.</p>
          )}
        </div>
        {wizardState.selectedFunctions.length > 1 && (
          <p style={{ color: '#ff6b35', fontSize: '14px', marginTop: '8px' }}>
            ⚠️ Backend supports conversion of exactly one function at a time. Only the first
            selection will be converted.
          </p>
        )}
      </div>

      <div style={{ marginBottom: '30px' }}>
        <h3>Conversion Options</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={wizardState.generateTests}
              onChange={(e) =>
                setWizardState((prev) => ({
                  ...prev,
                  generateTests: e.target.checked,
                }))
              }
            />
            <span>Generate async test cases</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={wizardState.applyToAll}
              onChange={(e) =>
                setWizardState((prev) => ({
                  ...prev,
                  applyToAll: e.target.checked,
                }))
              }
            />
            <span>Apply to all occurrences</span>
          </label>
        </div>
      </div>

      <div style={{ marginBottom: '30px' }}>
        <h3>Advanced Settings</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '15px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <span>Timeout (ms):</span>
            <input
              type="number"
              value={wizardState.customOptions.timeout}
              onChange={(e) => handleOptionChange('timeout', parseInt(e.target.value, 10))}
              min="5000"
              max="120000"
              style={{
                width: '100px',
                padding: '4px 8px',
                border: '1px solid #ccc',
                borderRadius: '4px',
              }}
            />
          </label>

          <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={wizardState.customOptions.errorHandling}
              onChange={(e) => handleOptionChange('errorHandling', e.target.checked)}
            />
            <span>Add comprehensive error handling</span>
          </label>
        </div>
      </div>
    </div>
  );
};
