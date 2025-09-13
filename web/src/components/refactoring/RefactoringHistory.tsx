import React from 'react';
import { invoke } from '@tauri-apps/api/core';

interface RefactoringHistoryProps {
  currentFile?: string;
  onHistoryEntryClicked?: (historyId: string) => void;
}

interface RefactoringHistoryState {
  history: RefactoringHistoryEntry[];
  currentIndex: number;
  loading: boolean;
  selectedEntry: string | null;
  showDetails: boolean;
}

interface RefactoringHistoryEntry {
  id: string;
  timestamp: number;
  operationType: string;
  operationName: string;
  filePath: string;
  status: 'success' | 'failed' | 'in_progress' | 'reverted';
  changesCount: number;
  confidence?: number;
  duration?: number;
  description: string;
  details?: RefactoringDetails;
  canUndo: boolean;
  canRedo: boolean;
  workspace?: string;
}

interface RefactoringDetails {
  beforeSnippet: string;
  afterSnippet: string;
  lineRange: {
    start: number;
    end: number;
  };
  affectedSymbols: string[];
  testGenerated: boolean;
  impact: 'low' | 'medium' | 'high';
  warnings: string[];
  suggestions: string[];
}

class RefactoringHistory extends React.Component<RefactoringHistoryProps, RefactoringHistoryState> {
  constructor(props: RefactoringHistoryProps) {
    super(props);

    this.state = {
      history: [],
      currentIndex: -1,
      loading: true,
      selectedEntry: null,
      showDetails: false,
    };
  }

  componentDidMount() {
    this.loadHistory();
  }

  componentDidUpdate(prevProps: RefactoringHistoryProps) {
    if (prevProps.currentFile !== this.props.currentFile) {
      this.loadHistory();
    }
  }

  async loadHistory() {
    try {
      this.setState({ loading: true });

      const request = this.props.currentFile
        ? { filter: { file_path: this.props.currentFile } }
        : {};

      const history = await invoke<RefactoringHistoryEntry[]>('get_refactoring_history', request);

      this.setState({
        history,
        loading: false,
        currentIndex: history.findIndex((h) => h.status === 'in_progress' || h.canRedo) - 1,
      });
    } catch (error) {
      console.error('Failed to load refactoring history:', error);
      this.setState({ loading: false });
    }
  }

  async undoRefactoring(historyId: string) {
    try {
      await invoke('undo_refactoring_operation', { historyId });
      await this.loadHistory(); // Refresh history
    } catch (error) {
      console.error('Failed to undo operation:', error);
    }
  }

  async redoRefactoring(historyId: string) {
    try {
      await invoke('redo_refactoring_operation', { historyId });
      await this.loadHistory(); // Refresh history
    } catch (error) {
      console.error('Failed to redo operation:', error);
    }
  }

  selectEntry = (entryId: string) => {
    if (this.props.onHistoryEntryClicked) {
      this.props.onHistoryEntryClicked(entryId);
    }

    this.setState((prevState) => ({
      selectedEntry: prevState.selectedEntry === entryId ? null : entryId,
      showDetails: prevState.selectedEntry === entryId ? false : true,
    }));
  };

  formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) {
      // < 1 minute
      return 'Just now';
    } else if (diff < 3600000) {
      // < 1 hour
      const minutes = Math.floor(diff / 60000);
      return `${minutes}m ago`;
    } else if (diff < 86400000) {
      // < 24 hours
      const hours = Math.floor(diff / 3600000);
      return `${hours}h ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  getStatusColor = (status: string): string => {
    switch (status) {
      case 'success':
        return 'status-success';
      case 'failed':
        return 'status-error';
      case 'reverted':
        return 'status-warning';
      case 'in_progress':
        return 'status-info';
      default:
        return 'status-neutral';
    }
  };

  clearSelected = () => {
    this.setState({ selectedEntry: null, showDetails: false });
  };

  render() {
    const { history, loading, selectedEntry, showDetails } = this.state;
    const selectedEntryData = selectedEntry ? history.find((h) => h.id === selectedEntry) : null;

    return (
      <div className="refactoring-history">
        <div className="history-header">
          <h4>Refactoring History</h4>
          <div className="history-actions">
            <button
              className="btn btn-outline btn-sm"
              onClick={() => this.loadHistory()}
              disabled={loading}
            >
              ↻ Refresh
            </button>
            <button
              className="btn btn-outline btn-sm"
              onClick={this.clearSelected}
              disabled={!selectedEntry}
            >
              Clear Selection
            </button>
          </div>
        </div>

        <div className="history-content">
          {loading ? (
            <div className="loading">Loading history...</div>
          ) : history.length === 0 ? (
            <div className="empty-history">No refactoring operations yet</div>
          ) : (
            <div className="history-list">{this.renderHistoryList()}</div>
          )}
        </div>

        {selectedEntryData && this.renderDetailsPanel(selectedEntryData)}

        <div className="history-stats">
          <div className="stat">
            <span className="stat-label">Total Operations:</span>
            <span className="stat-value">{history.length}</span>
          </div>
          <div className="stat">
            <span className="stat-label">Successful:</span>
            <span className="stat-value">
              {history.filter((h) => h.status === 'success').length}
            </span>
          </div>
          <div className="stat">
            <span className="stat-label">Failed:</span>
            <span className="stat-value">
              {history.filter((h) => h.status === 'failed').length}
            </span>
          </div>
        </div>
      </div>
    );
  }

  renderHistoryList() {
    const { history, selectedEntry } = this.state;

    return history.map((entry, index) => (
      <div
        key={entry.id}
        className={`history-entry ${selectedEntry === entry.id ? 'selected' : ''} ${entry.status}`}
        onClick={() => this.selectEntry(entry.id)}
      >
        <div className="entry-status">
          <span className={`status-indicator ${this.getStatusColor(entry.status)}`}></span>
          <span className="entry-index">#{history.length - index}</span>
        </div>

        <div className="entry-content">
          <div className="entry-header">
            <span className="operation-name">{entry.operationName}</span>
            <span className="timestamp">{this.formatTimestamp(entry.timestamp)}</span>
          </div>

          <div className="entry-meta">
            <span className="file-name">{entry.filePath.split('/').pop()}</span>
            <span className="changes-count">{entry.changesCount} changes</span>
            {entry.confidence && (
              <span className="confidence">
                {entry.confidence >= 0.8 ? '🟢' : entry.confidence >= 0.6 ? '🟡' : '🔴'}
                {Math.round(entry.confidence * 100)}%
              </span>
            )}
            {entry.duration && <span className="duration">{entry.duration}ms</span>}
          </div>

          <div className="entry-description">{entry.description}</div>
        </div>

        <div className="entry-actions">
          {entry.canUndo && (
            <button
              className="btn btn-sm btn-outline"
              onClick={(e) => {
                e.stopPropagation();
                this.undoRefactoring(entry.id);
              }}
              title="Undo this operation"
            >
              ↶ Undo
            </button>
          )}
          {!entry.canUndo && entry.canRedo && (
            <button
              className="btn btn-sm btn-outline"
              onClick={(e) => {
                e.stopPropagation();
                this.redoRefactoring(entry.id);
              }}
              title="Redo this operation"
            >
              ↷ Redo
            </button>
          )}
        </div>
      </div>
    ));
  }

  renderDetailsPanel(entry: RefactoringHistoryEntry) {
    const { showDetails } = this.state;

    return (
      <div className="history-details-panel">
        <div className="details-header">
          <h5>{entry.operationName} Details</h5>
          <button
            className="btn-close"
            onClick={() => this.setState({ showDetails: !showDetails })}
          >
            {showDetails ? '▼' : '▲'}
          </button>
        </div>

        {showDetails && entry.details && (
          <div className="details-content">
            <div className="details-section">
              <h6>Impact</h6>
              <div className="impact-indicator">{entry.details.impact.toUpperCase()} IMPACT</div>
            </div>

            <div className="details-section">
              <h6>Affected Symbols</h6>
              <ul className="symbols-list">
                {entry.details.affectedSymbols.map((symbol, index) => (
                  <li key={index}>{symbol}</li>
                ))}
              </ul>
            </div>

            {entry.details.beforeSnippet && entry.details.afterSnippet && (
              <div className="details-section">
                <h6>Code Changes</h6>
                <div className="code-diff">
                  <div className="before-code">
                    <h6>Before</h6>
                    <pre>{entry.details.beforeSnippet}</pre>
                  </div>
                  <div className="after-code">
                    <h6>After</h6>
                    <pre>{entry.details.afterSnippet}</pre>
                  </div>
                </div>
              </div>
            )}

            {entry.details.warnings.length > 0 && (
              <div className="details-section">
                <h6>Warnings</h6>
                <ul className="warnings-list">
                  {entry.details.warnings.map((warning, index) => (
                    <li key={index}>⚠️ {warning}</li>
                  ))}
                </ul>
              </div>
            )}

            {entry.details.suggestions.length > 0 && (
              <div className="details-section">
                <h6>Suggestions</h6>
                <ul className="suggestions-list">
                  {entry.details.suggestions.map((suggestion, index) => (
                    <li key={index}>💡 {suggestion}</li>
                  ))}
                </ul>
              </div>
            )}

            <div className="details-section">
              <div className="additional-info">
                {entry.details.testGenerated && (
                  <span className="info-item">✅ Tests generated</span>
                )}
                <span className="info-item">
                  Lines: {entry.details.lineRange.start} - {entry.details.lineRange.end}
                </span>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  }
}

export default RefactoringHistory;
export type { RefactoringHistoryEntry };
