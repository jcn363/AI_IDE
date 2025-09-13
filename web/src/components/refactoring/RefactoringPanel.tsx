import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import RefactoringSuggestionsList from './RefactoringSuggestionsList';
import RefactoringExecutionDialog from './RefactoringExecutionDialog';
import BatchRefactoringPanel from './BatchRefactoringPanel';

// Main refactoring panel component
interface RefactoringPanelProps {
  currentFile: string;
  cursorPosition: { line: number; character: number };
  selection: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  } | null;
}

interface RefactoringPanelState {
  suggestions: RefactoringSuggestion[];
  availableOperations: RefactoringOperationInfo[];
  currentOperation: RefactoringOperationType | null;
  executionDialogOpen: boolean;
  batchModeActive: boolean;
  loading: boolean;
  error: string | null;
}

interface RefactoringSuggestion {
  operationType: RefactoringOperationType;
  name: string;
  description: string;
  confidenceScore: number;
  expectedImpact: RefactoringImpact;
  prerequisites: string[];
  quickFix: boolean;
}

interface RefactoringOperationInfo {
  operationType: RefactoringOperationType;
  name: string;
  description: string;
  requiresSelection: boolean;
  isExperimental: boolean;
  typicalConfidenceScore: number;
}

type RefactoringOperationType =
  | 'rename'
  | 'extractFunction'
  | 'extractVariable'
  | 'extractInterface'
  | 'convertToAsync'
  | 'splitClass'
  | 'patternConversion'
  | 'inlineVariable'
  | 'inlineFunction'
  | 'moveMethod'
  | 'changeSignature';

type RefactoringImpact = 'low' | 'medium' | 'high';

class RefactoringPanel extends React.Component<RefactoringPanelProps, RefactoringPanelState> {
  constructor(props: RefactoringPanelProps) {
    super(props);

    this.state = {
      suggestions: [],
      availableOperations: [],
      currentOperation: null,
      executionDialogOpen: false,
      batchModeActive: false,
      loading: false,
      error: null,
    };
  }

  componentDidMount() {
    this.loadAvailableOperations();
    this.updateSuggestions();
  }

  componentDidUpdate(prevProps: RefactoringPanelProps) {
    // Update suggestions when file, position, or selection changes
    if (
      prevProps.currentFile !== this.props.currentFile ||
      prevProps.cursorPosition.line !== this.props.cursorPosition.line ||
      prevProps.cursorPosition.character !== this.props.cursorPosition.character ||
      JSON.stringify(prevProps.selection) !== JSON.stringify(this.props.selection)
    ) {
      this.updateSuggestions();
    }
  }

  async loadAvailableOperations() {
    try {
      this.setState({ loading: true });
      const operations = await invoke<RefactoringOperationInfo[]>(
        'get_available_refactoring_operations'
      );
      this.setState({ availableOperations: operations, loading: false });
    } catch (error) {
      this.handleError('Failed to load refactoring operations', error);
    }
  }

  async updateSuggestions() {
    if (!this.props.currentFile) return;

    try {
      this.setState({ loading: true });

      const request = {
        filePath: this.props.currentFile,
        position: this.props.cursorPosition,
        range: this.props.selection
          ? {
              start: this.props.selection.start,
              end: this.props.selection.end,
            }
          : null,
        contextBudget: 5000, // Limit context for performance
      };

      const response = await invoke<{
        filePath: string;
        position: typeof request.position;
        suggestions: RefactoringSuggestion[];
        totalSuggestions: number;
        analysisContext: string;
      }>('get_refactoring_suggestions', { request });

      this.setState({
        suggestions: response.suggestions,
        loading: false,
        error: null,
      });
    } catch (error) {
      this.handleError('Failed to load refactoring suggestions', error);
    }
  }

  async executeRefactoring(operationType: RefactoringOperationType) {
    this.setState({
      currentOperation: operationType,
      executionDialogOpen: true,
    });
  }

  handleRefactoringExecute = (operationType: RefactoringOperationType, options: any) => {
    // Trigger execution and close dialog
    this.executeRefactoringFull(operationType, options);
    this.setState({ executionDialogOpen: false });
  };

  handleRefactoringCancel = () => {
    this.setState({
      executionDialogOpen: false,
      currentOperation: null,
    });
  };

  async executeRefactoringFull(operationType: RefactoringOperationType, options: any) {
    try {
      this.setState({ loading: true });

      // Create execution context
      const context = {
        filePath: this.props.currentFile,
        symbolName: null, // Would be determined from LSP context
        symbolKind: null, // Would be determined from LSP context
        cursorLine: this.props.cursorPosition.line,
        cursorCharacter: this.props.cursorPosition.character,
        selection: this.props.selection,
        projectRoot: '/workspace', // Would come from workspace context
      };

      const request = {
        filePath: this.props.currentFile,
        operationType,
        context,
        options,
      };

      const result = await invoke<{
        success: boolean;
        id: string;
        changes: any[];
        errorMessage: string | null;
        warnings: string[];
        newContent: string | null;
      }>('execute_refactoring_operation', { request });

      if (result.success) {
        // Refresh suggestions after successful execution
        await this.updateSuggestions();
        this.handleSuccess('Refactoring completed successfully');
      } else {
        this.handleError('Refactoring operation failed', result.errorMessage);
      }
    } catch (error) {
      this.handleError('Refactoring execution failed', error);
    } finally {
      this.setState({ loading: false });
    }
  }

  toggleBatchMode = () => {
    this.setState((prevState) => ({
      batchModeActive: !prevState.batchModeActive,
    }));
  };

  handleError(message: string, error: any) {
    console.error(message, error);
    this.setState({
      error: `${message}: ${error}`,
      loading: false,
    });
  }

  handleSuccess(message: string) {
    console.log(message);
    this.setState({
      error: null,
      loading: false,
    });
    // Could emit success event to parent component
  }

  render() {
    const {
      suggestions,
      availableOperations,
      executionDialogOpen,
      currentOperation,
      batchModeActive,
      loading,
      error,
    } = this.state;

    if (batchModeActive) {
      return (
        <div className="refactoring-panel">
          <BatchRefactoringPanel
            currentFile={this.props.currentFile}
            availableOperations={availableOperations}
            onBack={() => this.setState({ batchModeActive: false })}
          />
        </div>
      );
    }

    return (
      <div className="refactoring-panel">
        <div className="refactoring-header">
          <h3>AI-Assisted Refactoring</h3>
          <div className="refactoring-controls">
            <button className="btn btn-secondary" onClick={this.toggleBatchMode} disabled={loading}>
              Batch Mode
            </button>
            <button
              className="btn btn-outline"
              onClick={() => this.updateSuggestions()}
              disabled={loading}
            >
              {loading ? 'Analyzing...' : 'Refresh'}
            </button>
          </div>
        </div>

        {error && (
          <div className="alert alert-error">
            <span>{error}</span>
            <button onClick={() => this.setState({ error: null })} className="btn-close">
              ×
            </button>
          </div>
        )}

        <div className="refactoring-content">
          <RefactoringSuggestionsList
            suggestions={suggestions}
            onSuggestionClick={(operationType) => this.executeRefactoring(operationType)}
            loading={loading}
          />

          <div className="refactoring-info">
            <div className="stats">
              <div className="stat">
                <div className="stat-label">Available</div>
                <div className="stat-value">{availableOperations.length}</div>
              </div>
              <div className="stat">
                <div className="stat-label">Suggested</div>
                <div className="stat-value">{suggestions.length}</div>
              </div>
              <div className="stat">
                <div className="stat-label">High Confidence</div>
                <div className="stat-value">
                  {suggestions.filter((s) => s.confidenceScore > 0.8).length}
                </div>
              </div>
            </div>
          </div>
        </div>

        {executionDialogOpen && currentOperation && (
          <RefactoringExecutionDialog
            operationType={currentOperation}
            operationInfo={availableOperations.find((op) => op.operationType === currentOperation)}
            onExecute={this.handleRefactoringExecute}
            onCancel={this.handleRefactoringCancel}
          />
        )}
      </div>
    );
  }
}

export default RefactoringPanel;
export type { RefactoringOperationType, RefactoringImpact, RefactoringSuggestion };
