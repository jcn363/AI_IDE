import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import AdvancedBatchOrchestrator from '../../utils/batch/AdvancedBatchOrchestrator';
import {
  BatchWorkflow,
  BatchOperation,
  ExecutionContext,
  ConflictAnalysis,
  OptimisationSuggestion,
} from '../../utils/batch/AdvancedBatchOrchestrator';

interface AdvancedBatchManagerProps {
  currentFile: string;
  onWorkflowCompleted?: (results: ExecutionContext) => void;
  onProgressUpdate?: (progress: number, currentOperation: string) => void;
}

interface AdvancedBatchManagerState {
  workflows: BatchWorkflow[];
  currentWorkflow: BatchWorkflow | null;
  templateWorkflows: Array<{
    id: string;
    name: string;
    description: string;
    operations: string[];
  }>;
  executionState: ExecutionContext | null;
  conflictAnalysis: ConflictAnalysis | null;
  optimisationSuggestions: OptimisationSuggestion[];
  isCreatingWorkflow: boolean;
  isExecuting: boolean;
  showDetails: boolean;
  selectedTemplate: string;
  progress: number;
  statusMessage: string;
  dryRunResults: ExecutionContext | null;
}

class AdvancedBatchManager extends React.Component<
  AdvancedBatchManagerProps,
  AdvancedBatchManagerState
> {
  private orchestrator: AdvancedBatchOrchestrator;

  constructor(props: AdvancedBatchManagerProps) {
    super(props);

    this.orchestrator = new AdvancedBatchOrchestrator();

    this.state = {
      workflows: [],
      currentWorkflow: null,
      templateWorkflows: [
        {
          id: 'codebase_cleanup',
          name: 'Codebase Cleanup',
          description:
            'Comprehensive cleanup including dead code removal, imports optimization, and formatting',
          operations: [
            'extractFunction',
            'inlineVariable',
            'removeDeadCode',
            'organizeImports',
            'formatCode',
          ],
        },
        {
          id: 'refactor_legacy_code',
          name: 'Legacy Code Modernization',
          description: 'Convert legacy patterns to modern Rust idiomatic code',
          operations: [
            'convertToAsync',
            'extractInterface',
            'replaceDeprecatedApis',
            'updatePatterns',
          ],
        },
        {
          id: 'performance_optimization',
          name: 'Performance Optimization',
          description: 'Performance-focused refactoring including memory optimization',
          operations: ['optimizeMemory', 'algorithmRefinement', 'inlineCriticalPaths'],
        },
      ],
      executionState: null,
      conflictAnalysis: null,
      optimisationSuggestions: [],
      isCreatingWorkflow: false,
      isExecuting: false,
      showDetails: false,
      selectedTemplate: '',
      progress: 0,
      statusMessage: '',
      dryRunResults: null,
    };

    this.loadSavedWorkflows();
  }

  componentDidUpdate(prevProps: AdvancedBatchManagerProps) {
    if (prevProps.currentFile !== this.props.currentFile) {
      this.analyzeCurrentContext();
    }
  }

  private async loadSavedWorkflows() {
    try {
      const saved = await invoke<string>('get_saved_workflows', {});
      if (saved) {
        const workflows = JSON.parse(saved);
        this.setState({ workflows });
      }
    } catch (error) {
      console.log('No saved workflows found');
    }
  }

  private async analyzeCurrentContext() {
    if (!this.state.currentWorkflow || this.state.templateWorkflows.length === 0) return;

    try {
      const suggestions = await this.orchestrator.generateOptimisationSuggestions(
        this.state.currentWorkflow
      );
      this.setState({ optimisationSuggestions: suggestions });
    } catch (error) {
      console.error('Failed to generate optimisation suggestions:', error);
    }
  }

  createWorkflowFromTemplate = async (templateId: string) => {
    this.setState({ isCreatingWorkflow: true, statusMessage: 'Creating workflow...' });

    try {
      const template = this.state.templateWorkflows.find((t) => t.id === templateId);
      if (!template) return;

      const options = {
        name: template.name,
        description: template.description,
        files: this.props.currentFile ? [this.props.currentFile] : undefined,
        filters: { priority: 'medium' }, // Default filter
      };

      const newWorkflow = await this.orchestrator.createWorkflow(templateId, options);

      this.setState((prevState) => ({
        workflows: [...prevState.workflows, newWorkflow],
        currentWorkflow: newWorkflow,
        isCreatingWorkflow: false,
        statusMessage: 'Workflow created successfully',
        selectedTemplate: templateId,
      }));

      await this.saveWorkflows();
    } catch (error) {
      this.setState({
        statusMessage: 'Failed to create workflow',
        isCreatingWorkflow: false,
      });
      console.error('Workflow creation failed:', error);
    }
  };

  saveWorkflow = () => {
    if (!this.state.currentWorkflow) return;

    invoke('save_workflow', { workflow: JSON.stringify(this.state.currentWorkflow) });
    this.setState({ statusMessage: 'Workflow saved successfully' });
  };

  loadWorkflow = async (workflowId: string) => {
    const workflow = this.state.workflows.find((w) => w.id === workflowId);
    if (workflow) {
      this.setState({ currentWorkflow: workflow, showDetails: true });
    }
  };

  deleteWorkflow = async (workflowId: string) => {
    this.setState((prevState) => ({
      workflows: prevState.workflows.filter((w) => w.id !== workflowId),
      currentWorkflow:
        prevState.currentWorkflow?.id === workflowId ? null : prevState.currentWorkflow,
    }));
    await this.saveWorkflows();
  };

  private async saveWorkflows() {
    invoke('save_workflows', { workflows: JSON.stringify(this.state.workflows) });
  }

  analyzeConflicts = async () => {
    if (!this.state.currentWorkflow) return;

    try {
      const analysis = await this.orchestrator.analyzeConflicts(this.state.currentWorkflow);
      this.setState({ conflictAnalysis: analysis });
    } catch (error) {
      console.error('Conflict analysis failed:', error);
    }
  };

  startDryRun = async () => {
    if (!this.state.currentWorkflow) return;

    this.setState({
      statusMessage: 'Starting dry run...',
      isExecuting: true,
      progress: 0,
    });

    try {
      const results = await this.orchestrator.executeWorkflow(this.state.currentWorkflow, {
        dryRun: true,
        onProgress: (progress, operation) => {
          this.setState({ progress });
          this.props.onProgressUpdate?.(progress, operation);
        },
      });

      this.setState({
        dryRunResults: results,
        isExecuting: false,
        statusMessage: 'Dry run completed - review results before execution',
      });
    } catch (error) {
      this.setState({
        statusMessage: 'Dry run failed',
        isExecuting: false,
      });
    }
  };

  executeWorkflow = async () => {
    if (!this.state.currentWorkflow) return;

    this.setState({
      statusMessage: 'Executing workflow...',
      isExecuting: true,
      progress: 0,
    });

    try {
      const results = await this.orchestrator.executeWorkflow(this.state.currentWorkflow, {
        onProgress: (progress, operation) => {
          this.setState({ progress });
          this.props.onProgressUpdate?.(progress, operation);
        },
        pauseOnErrors: false, // Configurable
      });

      this.setState({
        executionState: results,
        isExecuting: false,
        statusMessage: `Workflow completed: ${results.completed.length} succeeded, ${results.failed.length} failed`,
      });

      this.props.onWorkflowCompleted?.(results);
      await this.saveWorkflows(); // Save updated workflow state
    } catch (error) {
      this.setState({
        statusMessage: 'Workflow execution failed',
        isExecuting: false,
      });
    }
  };

  pauseWorkflow = () => {
    // Implementation for pausing workflow execution
    this.setState({ statusMessage: 'Workflow paused', isExecuting: false });
  };

  resumeWorkflow = () => {
    // Implementation for resuming workflow execution
    this.executeWorkflow();
  };

  cancelWorkflow = () => {
    this.setState({
      statusMessage: 'Workflow cancelled',
      isExecuting: false,
      progress: 0,
    });
  };

  render() {
    const {
      workflows,
      currentWorkflow,
      templateWorkflows,
      executionState,
      conflictAnalysis,
      optimisationSuggestions,
      isCreatingWorkflow,
      isExecuting,
      progress,
      statusMessage,
      dryRunResults,
      showDetails,
    } = this.state;

    return (
      <div className="advanced-batch-manager">
        <div className="batch-manager-header">
          <h3>Advanced Batch Operations</h3>
          <div className="status-bar">
            {statusMessage && <span className="status-message">{statusMessage}</span>}
            {isExecuting && <span>Executing: {progress.toFixed(1)}%</span>}
          </div>
        </div>

        <div className="batch-manager-content">
          {this.renderTemplateSelector()}
          {this.renderWorkflowBuilder()}
          {this.renderExecutionControls()}
          {this.renderResultsViewer()}
        </div>

        {isExecuting && (
          <div className="execution-progress">
            <div className="progress-bar">
              <div className="progress-fill" style={{ width: `${progress}%` }}></div>
            </div>
            <div className="progress-details">Progress: {progress.toFixed(1)}%</div>
          </div>
        )}
      </div>
    );
  }

  renderTemplateSelector() {
    const { templateWorkflows, isCreatingWorkflow, selectedTemplate } = this.state;

    return (
      <div className="template-selector">
        <h4>Workflow Templates</h4>
        <div className="templates-grid">
          {templateWorkflows.map((template) => (
            <div
              key={template.id}
              className={`template-card ${selectedTemplate === template.id ? 'selected' : ''}`}
              onClick={() => this.setState({ selectedTemplate: template.id })}
            >
              <h5>{template.name}</h5>
              <p>{template.description}</p>
              <div className="template-operations">{template.operations.length} operations</div>
            </div>
          ))}
        </div>

        {selectedTemplate && (
          <div className="template-actions">
            <button
              className="btn btn-primary"
              onClick={() => this.createWorkflowFromTemplate(selectedTemplate)}
              disabled={isCreatingWorkflow}
            >
              {isCreatingWorkflow ? 'Creating...' : 'Create Workflow'}
            </button>
          </div>
        )}
      </div>
    );
  }

  renderWorkflowBuilder() {
    const { currentWorkflow, showDetails, conflictAnalysis, optimisationSuggestions } = this.state;

    if (!currentWorkflow) return null;

    return (
      <div className="workflow-builder">
        <div className="workflow-header">
          <h4>{currentWorkflow.name}</h4>
          <button
            className="btn btn-outline btn-sm"
            onClick={() => this.setState({ showDetails: !showDetails })}
          >
            {showDetails ? 'Hide Details' : 'Show Details'}
          </button>
        </div>

        <div className="workflow-meta">
          <span>Strategy: {currentWorkflow.executionStrategy}</span>
          <span>Max Concurrency: {currentWorkflow.maxConcurrency}</span>
          <span>Operations: {currentWorkflow.operations.length}</span>
          <span>Timeout: {currentWorkflow.timeoutMinutes}min</span>
        </div>

        {showDetails && (
          <div className="workflow-details">
            <div className="operations-list">
              <h5>Operations</h5>
              {currentWorkflow.operations.map((op, index) => (
                <div key={op.id} className="operation-item">
                  <span className="operation-order">#{index + 1}</span>
                  <span className="operation-name">{op.name}</span>
                  <span className="operation-priority">Priority: {op.priority}</span>
                  <span className="operation-deps">
                    {op.dependencies.length > 0 && `Depends on: ${op.dependencies.join(', ')}`}
                  </span>
                </div>
              ))}
            </div>

            {conflictAnalysis && (
              <div className="conflict-analysis">
                <h5>Conflict Analysis</h5>
                {conflictAnalysis.conflicts.map((conflict, index) => (
                  <div key={index} className="conflict-item">
                    <span className={`severity ${conflict.severity}`}>
                      {conflict.severity.toUpperCase()}
                    </span>
                    <span>{conflict.description}</span>
                  </div>
                ))}
                <div className="recommendations">
                  <h6>Recommendations</h6>
                  {conflictAnalysis.recommendations.map((rec, index) => (
                    <div key={index} className="recommendation">
                      💡 {rec}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {optimisationSuggestions.length > 0 && (
              <div className="optimisation-suggestions">
                <h5>Optimization Suggestions</h5>
                {optimisationSuggestions.map((suggestion, index) => (
                  <div key={index} className="suggestion-item">
                    <div className="suggestion-header">
                      <span className="suggestion-type">{suggestion.suggestionType}</span>
                      <span className={`benefit ${suggestion.benefit > 30 ? 'high' : 'medium'}`}>
                        +{suggestion.benefit}% improvement
                      </span>
                    </div>
                    <p>{suggestion.description}</p>
                    <div className="suggestion-effort">
                      Implementation effort: {suggestion.effort}/10
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    );
  }

  renderExecutionControls() {
    const { currentWorkflow, isExecuting, dryRunResults } = this.state;
    if (!currentWorkflow) return null;

    return (
      <div className="execution-controls">
        <div className="control-buttons">
          <button
            className="btn btn-outline"
            onClick={this.analyzeConflicts}
            disabled={isExecuting}
          >
            Analyze Conflicts
          </button>

          <button className="btn btn-outline" onClick={this.startDryRun} disabled={isExecuting}>
            Dry Run
          </button>

          {!isExecuting ? (
            <button className="btn btn-primary" onClick={this.executeWorkflow}>
              Execute Workflow
            </button>
          ) : (
            <>
              <button className="btn btn-warning" onClick={this.pauseWorkflow}>
                Pause
              </button>

              <button className="btn btn-danger" onClick={this.cancelWorkflow}>
                Cancel
              </button>
            </>
          )}
        </div>

        <div className="workflow-actions">
          <button className="btn btn-outline btn-sm" onClick={this.saveWorkflow}>
            Save Workflow
          </button>
        </div>
      </div>
    );
  }

  renderResultsViewer() {
    const { executionState, dryRunResults } = this.state;

    if (!executionState && !dryRunResults) return null;

    const results = executionState || dryRunResults;

    if (!results) return null;

    return (
      <div className="results-viewer">
        <h4>Execution Results</h4>

        <div className="results-summary">
          <div className="summary-stat">
            <span className="stat-label">Duration</span>
            <span className="stat-value">{(results.metrics.totalDuration / 1000).toFixed(1)}s</span>
          </div>

          <div className="summary-stat">
            <span className="stat-label">Completed</span>
            <span className="stat-value">{results.completed.length}</span>
          </div>

          <div className="summary-stat">
            <span className="stat-label">Failed</span>
            <span className="stat-value">{results.failed.length}</span>
          </div>

          <div className="summary-stat">
            <span className="stat-label">Peak Memory</span>
            <span className="stat-value">
              {(results.metrics.peakMemoryUsage / 1024 / 1024).toFixed(1)} MB
            </span>
          </div>

          <div className="summary-stat">
            <span className="stat-label">Throughput</span>
            <span className="stat-value">
              {results.metrics.throughputOpsPerSecond.toFixed(1)} ops/s
            </span>
          </div>
        </div>

        <div className="results-details">
          <div className="results-section">
            <h5>Completed Operations ({results.completed.length})</h5>
            <ul>
              {results.completed.map((opId, index) => (
                <li key={index} className="completed-operation">
                  <span className="operation-id">{opId}</span>
                  <span className="status success">✓</span>
                </li>
              ))}
            </ul>
          </div>

          <div className="results-section">
            <h5>Failed Operations ({results.failed.length})</h5>
            <ul>
              {results.failed.map((opId, index) => (
                <li key={index} className="failed-operation">
                  <span className="operation-id">{opId}</span>
                  <span className="status error">✗</span>
                </li>
              ))}
            </ul>
          </div>

          <div className="results-section">
            <h5>Rolled Back Operations ({results.rollbacks.length})</h5>
            <ul>
              {results.rollbacks.map((opId, index) => (
                <li key={index} className="rolled-back-operation">
                  <span className="operation-id">{opId}</span>
                  <span className="status warning">↶</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    );
  }
}

export default AdvancedBatchManager;
