import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import RefactoringSuggestionsList from './RefactoringSuggestionsList';

interface BatchRefactoringPanelProps {
    currentFile: string;
    availableOperations: Array<{
        operationType: string;
        name: string;
        description: string;
        requiresSelection: boolean;
        isExperimental: boolean;
        typicalConfidenceScore: number;
    }>;
    onBack: () => void;
}

interface BatchOperation {
    id: string;
    operationType: string;
    name: string;
    description: string;
    status: 'pending' | 'running' | 'completed' | 'failed';
    result?: string;
}

interface BatchRefactoringPanelState {
    batchOperations: BatchOperation[];
    selectedOperations: Set<string>;
    operationOptions: { [key: string]: any };
    isExecuting: boolean;
    progress: number;
    error: string | null;
    currentOperationIndex: number;
    batchSummary: {
        total: number;
        completed: number;
        failed: number;
        skipped: number;
    };
}

class BatchRefactoringPanel extends React.Component<BatchRefactoringPanelProps, BatchRefactoringPanelState> {
    constructor(props: BatchRefactoringPanelProps) {
        super(props);

        this.state = {
            batchOperations: [],
            selectedOperations: new Set(),
            operationOptions: {},
            isExecuting: false,
            progress: 0,
            error: null,
            currentOperationIndex: 0,
            batchSummary: {
                total: 0,
                completed: 0,
                failed: 0,
                skipped: 0,
            },
        };
    }

    componentDidMount() {
        this.loadBatchOperations();
    }

    async loadBatchOperations() {
        try {
            // Get batch operations from backend or filter available operations
            const batchOperations = this.props.availableOperations.map(op => ({
                id: `${op.operationType}_${Date.now()}_${Math.random()}`,
                operationType: op.operationType,
                name: op.name,
                description: op.description,
                status: 'pending' as const,
            }));

            this.setState({
                batchOperations,
                batchSummary: { ...this.state.batchSummary, total: batchOperations.length },
            });
        } catch (error) {
            console.error('Error loading batch operations:', error);
            this.setState({ error: 'Failed to load batch operations' });
        }
    }

    handleOperationSelect = (operationId: string, operationType: string) => {
        this.setState(prevState => {
            const newSelected = new Set(prevState.selectedOperations);

            if (newSelected.has(operationId)) {
                newSelected.delete(operationId);
            } else {
                newSelected.add(operationId);
            }

            return { selectedOperations: newSelected };
        });
    };

    handleSelectAll = () => {
        const allIds = this.state.batchOperations.map(op => op.id);
        this.setState(prevState => ({
            selectedOperations: new Set(prevState.selectedOperations.size === allIds.length ? [] : allIds),
        }));
    };

    handleClearSelection = () => {
        this.setState({ selectedOperations: new Set() });
    };

    executeBatchOperations = async () => {
        if (this.state.selectedOperations.size === 0) {
            this.setState({ error: 'Please select at least one operation to execute' });
            return;
        }

        const selectedOps = this.state.batchOperations.filter(op =>
            this.state.selectedOperations.has(op.id)
        );

        this.setState({
            isExecuting: true,
            progress: 0,
            error: null,
            currentOperationIndex: 0,
            batchSummary: { ...this.state.batchSummary, completed: 0, failed: 0, skipped: 0 },
        });

        const updatedOperations = [...this.state.batchOperations];
        let completed = 0;
        let failed = 0;
        let skipped = 0;

        for (let i = 0; i < selectedOps.length; i++) {
            const operation = selectedOps[i];
            const operationIndex = updatedOperations.findIndex(op => op.id === operation.id);

            try {
                // Update status to running
                updatedOperations[operationIndex] = {
                    ...updatedOperations[operationIndex],
                    status: 'running',
                };
                this.setState({
                    batchOperations: [...updatedOperations],
                    currentOperationIndex: i,
                    progress: Math.round(((i) / selectedOps.length) * 100),
                });

                // Execute the operation
                const context = {
                    filePath: this.props.currentFile,
                    symbolName: null,
                    symbolKind: null,
                    cursorLine: 0, // Would come from current position
                    cursorCharacter: 0,
                    selection: null,
                    projectRoot: '/workspace',
                };

                const request = {
                    filePath: this.props.currentFile,
                    operationType: operation.operationType,
                    context,
                    options: this.state.operationOptions[operation.id] || {},
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
                    updatedOperations[operationIndex] = {
                        ...updatedOperations[operationIndex],
                        status: 'completed',
                        result: `Applied successfully (${result.changes.length} changes)`,
                    };
                    completed++;
                } else {
                    updatedOperations[operationIndex] = {
                        ...updatedOperations[operationIndex],
                        status: 'failed',
                        result: result.errorMessage || 'Operation failed',
                    };
                    failed++;
                }

            } catch (error) {
                updatedOperations[operationIndex] = {
                    ...updatedOperations[operationIndex],
                    status: 'failed',
                    result: error instanceof Error ? error.message : 'Unknown error',
                };
                failed++;
            }
        }

        this.setState({
            batchOperations: updatedOperations,
            isExecuting: false,
            progress: 100,
            batchSummary: { ...this.state.batchSummary, total: selectedOps.length, completed, failed, skipped },
        });
    };

    render() {
        const {
            batchOperations,
            selectedOperations,
            isExecuting,
            progress,
            error,
            currentOperationIndex,
            batchSummary,
        } = this.state;

        return (
            <div className="batch-refactoring-panel">
                <div className="batch-header">
                    <button className="btn btn-secondary back-btn" onClick={this.props.onBack}>
                        ← Back to Main
                    </button>
                    <h3>Batch Refactoring Operations</h3>
                </div>

                {error && (
                    <div className="alert alert-error">
                        <span>{error}</span>
                        <button onClick={() => this.setState({ error: null })} className="btn-close">
                            ×
                        </button>
                    </div>
                )}

                <div className="batch-controls">
                    <div className="selection-buttons">
                        <button
                            className="btn btn-outline"
                            onClick={this.handleSelectAll}
                            disabled={batchSummary.total === 0}
                        >
                            {selectedOperations.size === batchSummary.total ? 'Deselect All' : 'Select All'}
                        </button>
                        <button
                            className="btn btn-outline"
                            onClick={this.handleClearSelection}
                            disabled={selectedOperations.size === 0}
                        >
                            Clear Selection ({selectedOperations.size})
                        </button>
                    </div>
                    <div className="execute-controls">
                        <button
                            className="btn btn-primary"
                            onClick={this.executeBatchOperations}
                            disabled={selectedOperations.size === 0 || isExecuting}
                        >
                            {isExecuting ? 'Executing...' : `Execute (${selectedOperations.size})`}
                        </button>
                    </div>
                </div>

                {isExecuting && (
                    <div className="progress-container">
                        <div className="progress-bar">
                            <div className="progress-fill" style={{ width: `${progress}%` }}></div>
                        </div>
                        <div className="progress-text">
                            {progress}% Complete - Processing {currentOperationIndex + 1} of {selectedOperations.size}
                        </div>
                    </div>
                )}

                <div className="batch-summary">
                    <div className="summary-stats">
                        <div className="stat">
                            <div className="stat-label">Selected</div>
                            <div className="stat-value">{selectedOperations.size}</div>
                        </div>
                        <div className="stat">
                            <div className="stat-label">Completed</div>
                            <div className="stat-value">{batchSummary.completed}</div>
                        </div>
                        <div className="stat">
                            <div className="stat-label">Failed</div>
                            <div className="stat-value">{batchSummary.failed}</div>
                        </div>
                        <div className="stat">
                            <div className="stat-label">Skipped</div>
                            <div className="stat-value">{batchSummary.skipped}</div>
                        </div>
                    </div>
                </div>

                <div className="batch-operations-list">
                    {batchOperations.map(operation => (
                        <div
                            key={operation.id}
                            className={`batch-operation-item ${operation.status} ${
                                selectedOperations.has(operation.id) ? 'selected' : ''
                            }`}
                        >
                            <div className="operation-checkbox">
                                <input
                                    type="checkbox"
                                    checked={selectedOperations.has(operation.id)}
                                    onChange={() => this.handleOperationSelect(operation.id, operation.operationType)}
                                    disabled={isExecuting}
                                />
                            </div>
                            <div className="operation-info">
                                <div className="operation-name">
                                    {operation.name}
                                    <span className={`status-badge ${operation.status}`}>
                                        {operation.status}
                                    </span>
                                </div>
                                <div className="operation-description">
                                    {operation.description}
                                </div>
                                {operation.result && (
                                    <div className="operation-result">
                                        {operation.result}
                                    </div>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        );
    }
}

export default BatchRefactoringPanel;