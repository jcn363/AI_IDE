import React from 'react';
import { invoke } from '@tauri-apps/api/core';

interface RefactoringExecutionDialogProps {
  operationType: string;
  operationInfo?: {
    operationType: string;
    name: string;
    description: string;
    requiresSelection: boolean;
    isExperimental: boolean;
    typicalConfidenceScore: number;
  };
  onExecute: (operationType: string, options: any) => void;
  onCancel: () => void;
}

interface RefactoringExecutionDialogState {
  loading: boolean;
  options: { [key: string]: any };
  validationErrors: { [key: string]: string };
  validationWarnings: { [key: string]: string };
}

class RefactoringExecutionDialog extends React.Component<
  RefactoringExecutionDialogProps,
  RefactoringExecutionDialogState
> {
  constructor(props: RefactoringExecutionDialogProps) {
    super(props);

    this.state = {
      loading: false,
      options: {},
      validationErrors: {},
      validationWarnings: {},
    };
  }

  handleOptionChange = (optionKey: string, value: any) => {
    this.setState(
      (prevState) => ({
        options: {
          ...prevState.options,
          [optionKey]: value,
        },
      }),
      () => this.validateOption(optionKey, value)
    );
  };

  validateOption = async (optionKey: string, value: any) => {
    if (!this.props.operationInfo?.requiresSelection) return;

    try {
      const validationRequest = {
        operationType: this.props.operationType,
        options: { ...this.state.options, [optionKey]: value },
      };

      const result = await invoke<{
        valid: boolean;
        errors: { [key: string]: string };
        warnings: { [key: string]: string };
      }>('validate_refactoring_options', { request: validationRequest });

      this.setState((prevState) => ({
        validationErrors: {
          ...prevState.validationErrors,
          ...result.errors,
        },
        validationWarnings: {
          ...prevState.validationWarnings,
          ...result.warnings,
        },
      }));
    } catch (error) {
      console.error('Validation error:', error);
    }
  };

  handleExecute = () => {
    // Check for validation errors
    const hasErrors = Object.keys(this.state.validationErrors).length > 0;
    if (hasErrors) {
      return; // Don't proceed if there are validation errors
    }

    this.props.onExecute(this.props.operationType, this.state.options);
  };

  renderOptionInput(optionKey: string, optionConfig: any) {
    const value = this.state.options[optionKey] || optionConfig.defaultValue || '';
    const error = this.state.validationErrors[optionKey];
    const warning = this.state.validationWarnings[optionKey];

    switch (optionConfig.type) {
      case 'string':
        return (
          <div key={optionKey} className="form-group">
            <label htmlFor={`option-${optionKey}`}>{optionConfig.label}</label>
            <input
              id={`option-${optionKey}`}
              type="text"
              value={value}
              onChange={(e) => this.handleOptionChange(optionKey, e.target.value)}
              placeholder={optionConfig.placeholder}
              className={error ? 'input-error' : ''}
            />
            {error && <div className="validation-error">{error}</div>}
            {warning && <div className="validation-warning">{warning}</div>}
          </div>
        );

      case 'boolean':
        return (
          <div key={optionKey} className="form-group">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={value}
                onChange={(e) => this.handleOptionChange(optionKey, e.target.checked)}
              />
              {optionConfig.label}
            </label>
            {error && <div className="validation-error">{error}</div>}
            {warning && <div className="validation-warning">{warning}</div>}
          </div>
        );

      default:
        return null;
    }
  }

  render() {
    const { operationInfo } = this.props;
    const { loading } = this.state;
    const hasErrors = Object.keys(this.state.validationErrors).length > 0;

    return (
      <div className="modal-overlay">
        <div className="modal-content refactoring-dialog">
          <div className="modal-header">
            <h4>Configure Refactoring</h4>
            <button className="modal-close" onClick={this.props.onCancel}>
              ×
            </button>
          </div>

          <div className="modal-body">
            {operationInfo && (
              <div className="refactoring-info">
                <h5>{operationInfo.name}</h5>
                <p>{operationInfo.description}</p>
                <div className="refactoring-meta">
                  <span className="confidence-score">
                    Confidence: {operationInfo.typicalConfidenceScore}%
                  </span>
                  {operationInfo.requiresSelection && (
                    <span className="selection-required">Selection Required</span>
                  )}
                  {operationInfo.isExperimental && (
                    <span className="experimental">Experimental</span>
                  )}
                </div>
              </div>
            )}

            <div className="refactoring-options">
              {/* Render operation-specific options here */}
              {/* This would come from the backend or be hardcoded for now */}
              {this.renderOptionInput('generateTests', {
                type: 'boolean',
                label: 'Generate Tests',
                defaultValue: true,
              })}
              {this.renderOptionInput('preserveComments', {
                type: 'boolean',
                label: 'Preserve Comments',
                defaultValue: true,
              })}
            </div>

            {hasErrors && (
              <div className="validation-summary error-summary">
                <h6>Please fix the following errors:</h6>
                <ul>
                  {Object.entries(this.state.validationErrors).map(([key, error]) => (
                    <li key={key}>{error}</li>
                  ))}
                </ul>
              </div>
            )}

            {Object.keys(this.state.validationWarnings).length > 0 && (
              <div className="validation-summary warning-summary">
                <h6>Warnings:</h6>
                <ul>
                  {Object.entries(this.state.validationWarnings).map(([key, warning]) => (
                    <li key={key}>{warning}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>

          <div className="modal-footer">
            <button className="btn btn-secondary" onClick={this.props.onCancel} disabled={loading}>
              Cancel
            </button>
            <button
              className="btn btn-primary"
              onClick={this.handleExecute}
              disabled={loading || hasErrors}
            >
              {loading ? 'Executing...' : 'Execute'}
            </button>
          </div>
        </div>
      </div>
    );
  }
}

export default RefactoringExecutionDialog;
