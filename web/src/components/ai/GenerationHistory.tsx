import React, { useState, useCallback, useEffect } from 'react';
import { useAppSelector, useAppDispatch } from '../../store/hooks';
import {
  codegenSelectors,
  codegenActions,
  GenerationHistoryItem,
} from '../../store/slices/codegenSlice';

interface GenerationHistoryProps {
  onLoadGeneration?: (item: GenerationHistoryItem) => void;
}

const GenerationHistory: React.FC<GenerationHistoryProps> = ({ onLoadGeneration }) => {
  const dispatch = useAppDispatch();

  // State from Redux
  const history = useAppSelector(codegenSelectors.selectFilteredHistory);
  const favorites = useAppSelector(codegenSelectors.selectFilteredFavorites);
  const searchTerm = useAppSelector(codegenSelectors.selectSearchTerm);
  const filterLanguage = useAppSelector(codegenSelectors.selectFilterLanguage);
  const sortBy = useAppSelector(codegenSelectors.selectSortBy);
  const sortOrder = useAppSelector(codegenSelectors.selectSortOrder);
  const isLoading = useAppSelector(codegenSelectors.selectIsLoading);
  const error = useAppSelector(codegenSelectors.selectError);
  const historyCount = useAppSelector(codegenSelectors.selectHistoryCount);
  const favoritesCount = useAppSelector(codegenSelectors.selectFavoritesCount);

  // Local state
  const [activeTab, setActiveTab] = useState<'history' | 'favorites'>('history');
  const [selectedItem, setSelectedItem] = useState<GenerationHistoryItem | null>(null);

  // Load data on mount
  useEffect(() => {
    dispatch(codegenActions.loadGenerationHistory());
  }, [dispatch]);

  const handleSearchChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      dispatch(codegenActions.setSearchTerm(e.target.value));
    },
    [dispatch]
  );

  const handleLanguageFilterChange = useCallback(
    (language: string | null) => {
      dispatch(codegenActions.setFilterLanguage(language));
    },
    [dispatch]
  );

  const handleSortChange = useCallback(
    (sortBy: 'timestamp' | 'confidence' | 'complexity' | 'language') => {
      dispatch(codegenActions.setSortBy(sortBy));
    },
    [dispatch]
  );

  const handleSortOrderToggle = useCallback(() => {
    const newOrder = sortOrder === 'asc' ? 'desc' : 'asc';
    dispatch(codegenActions.setSortOrder(newOrder));
  }, [dispatch, sortOrder]);

  const handleToggleFavorite = useCallback(
    (itemId: string) => {
      dispatch(codegenActions.toggleFavorite(itemId));
    },
    [dispatch]
  );

  const handleRemoveFromHistory = useCallback(
    (itemId: string) => {
      if (window.confirm('Are you sure you want to remove this item from history?')) {
        dispatch(codegenActions.removeFromHistory(itemId));
      }
    },
    [dispatch]
  );

  const handleClearHistory = useCallback(() => {
    if (
      window.confirm('Are you sure you want to clear all history? This action cannot be undone.')
    ) {
      dispatch(codegenActions.clearHistory());
    }
  }, [dispatch]);

  const handleLoadGeneration = useCallback(
    (item: GenerationHistoryItem) => {
      setSelectedItem(item);
      onLoadGeneration?.(item);
    },
    [onLoadGeneration]
  );

  const formatTimestamp = useCallback((timestamp: number): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = diffMs / (1000 * 60 * 60);

    if (diffHours < 24) {
      return `${Math.floor(diffHours)}h ago`;
    } else if (diffHours < 168) {
      return `${Math.floor(diffHours / 24)}d ago`;
    } else {
      return date.toLocaleDateString();
    }
  }, []);

  const getLanguageColor = useCallback((language: string): string => {
    switch (language.toLowerCase()) {
      case 'rust':
        return '#000000';
      case 'python':
        return '#3776ab';
      case 'typescript':
        return '#3178c6';
      case 'javascript':
        return '#f7df1e';
      default:
        return '#6b7280';
    }
  }, []);

  const getScoreColor = useCallback((score: number): string => {
    if (score >= 0.8) return '#10b981';
    if (score >= 0.6) return '#f59e0b';
    return '#ef4444';
  }, []);

  const currentItems = activeTab === 'history' ? history : favorites;

  return (
    <div className="generation-history">
      {/* Header */}
      <div className="history-header">
        <div className="header-content">
          <h2>Generation History</h2>
          <div className="stats">
            <span className="stat-item">History: {historyCount}</span>
            <span className="stat-item">Favorites: {favoritesCount}</span>
          </div>
        </div>
        <div className="header-actions">
          {activeTab === 'history' && historyCount > 0 && (
            <button className="clear-btn" onClick={handleClearHistory} disabled={isLoading}>
              🗑️ Clear All
            </button>
          )}
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="tab-navigation">
        <button
          className={`tab-btn ${activeTab === 'history' ? 'active' : ''}`}
          onClick={() => setActiveTab('history')}
        >
          📚 History ({historyCount})
        </button>
        <button
          className={`tab-btn ${activeTab === 'favorites' ? 'active' : ''}`}
          onClick={() => setActiveTab('favorites')}
        >
          ⭐ Favorites ({favoritesCount})
        </button>
      </div>

      {/* Search and Filters */}
      <div className="filters-section">
        <div className="search-group">
          <input
            type="text"
            placeholder="Search generations..."
            value={searchTerm}
            onChange={handleSearchChange}
            className="search-input"
          />
        </div>

        <div className="filter-controls">
          <div className="filter-group">
            <label>Language:</label>
            <select
              value={filterLanguage || ''}
              onChange={(e) => handleLanguageFilterChange(e.target.value || null)}
              className="filter-select"
            >
              <option value="">All Languages</option>
              <option value="Rust">Rust</option>
              <option value="Python">Python</option>
              <option value="TypeScript">TypeScript</option>
              <option value="JavaScript">JavaScript</option>
            </select>
          </div>

          <div className="filter-group">
            <label>Sort by:</label>
            <select
              value={sortBy}
              onChange={(e) => handleSortChange(e.target.value as any)}
              className="filter-select"
            >
              <option value="timestamp">Date</option>
              <option value="confidence">Confidence</option>
              <option value="complexity">Complexity</option>
              <option value="language">Language</option>
            </select>
            <button
              className="sort-order-btn"
              onClick={handleSortOrderToggle}
              title={`Sort ${sortOrder === 'asc' ? 'descending' : 'ascending'}`}
            >
              {sortOrder === 'asc' ? '↑' : '↓'}
            </button>
          </div>
        </div>
      </div>

      {/* Loading State */}
      {isLoading && (
        <div className="loading-state">
          <div className="loading-spinner"></div>
          <p>Loading...</p>
        </div>
      )}

      {/* Error State */}
      {error && (
        <div className="error-state">
          <div className="error-icon">⚠️</div>
          <p>{error}</p>
          <button
            className="retry-btn"
            onClick={() => dispatch(codegenActions.loadGenerationHistory())}
          >
            Retry
          </button>
        </div>
      )}

      {/* History List */}
      {!isLoading && !error && (
        <div className="history-list">
          {currentItems.length === 0 ? (
            <div className="empty-state">
              <div className="empty-icon">{activeTab === 'history' ? '📚' : '⭐'}</div>
              <h3>No {activeTab} found</h3>
              <p>
                {activeTab === 'history'
                  ? 'Your generation history will appear here after creating functions.'
                  : 'Mark generations as favorites to see them here.'}
              </p>
            </div>
          ) : (
            currentItems.map((item) => (
              <div key={item.id} className="history-item">
                <div className="item-header">
                  <div className="item-title">
                    <h4>{item.result.generated_function?.name || 'Generated Function'}</h4>
                    <div className="item-meta">
                      <span
                        className="language-badge"
                        style={{ backgroundColor: getLanguageColor(item.request.target_language) }}
                      >
                        {item.request.target_language}
                      </span>
                      <span className="timestamp">{formatTimestamp(item.timestamp)}</span>
                    </div>
                  </div>
                  <div className="item-actions">
                    <button
                      className="action-btn"
                      onClick={() => handleLoadGeneration(item)}
                      title="Load this generation"
                    >
                      📥 Load
                    </button>
                    <button
                      className={`action-btn favorite-btn ${item.isFavorite ? 'active' : ''}`}
                      onClick={() => handleToggleFavorite(item.id)}
                      title={item.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
                    >
                      {item.isFavorite ? '⭐' : '☆'}
                    </button>
                    <button
                      className="action-btn delete-btn"
                      onClick={() => handleRemoveFromHistory(item.id)}
                      title="Remove from history"
                    >
                      🗑️
                    </button>
                  </div>
                </div>

                <div className="item-content">
                  <div className="purpose">
                    <strong>Purpose:</strong> {item.request.function_purpose}
                  </div>

                  {item.result.success ? (
                    <div className="success-details">
                      <div className="metrics">
                        {item.result.generated_function?.confidence_score && (
                          <div className="metric">
                            <span className="metric-label">Confidence:</span>
                            <span
                              className="metric-value"
                              style={{
                                color: getScoreColor(
                                  item.result.generated_function.confidence_score
                                ),
                              }}
                            >
                              {Math.round(item.result.generated_function.confidence_score * 100)}%
                            </span>
                          </div>
                        )}
                        {item.result.generated_function?.complexity && (
                          <div className="metric">
                            <span className="metric-label">Complexity:</span>
                            <span className="metric-value">
                              {item.result.generated_function.complexity}/10
                            </span>
                          </div>
                        )}
                      </div>

                      {item.validation && (
                        <div className="validation-summary">
                          <div className="validation-score">
                            <span className="score-label">Validation:</span>
                            <span
                              className="score-value"
                              style={{ color: getScoreColor(item.validation.overall_score) }}
                            >
                              {Math.round(item.validation.overall_score * 100)}%
                            </span>
                          </div>
                          {item.validation.issues.length > 0 && (
                            <div className="issues-count">
                              {item.validation.issues.length} issue
                              {item.validation.issues.length !== 1 ? 's' : ''}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  ) : (
                    <div className="error-details">
                      <div className="error-message">❌ {item.result.error}</div>
                    </div>
                  )}
                </div>

                {/* Expanded Details */}
                {selectedItem?.id === item.id && (
                  <div className="item-details">
                    <div className="details-section">
                      <h5>Request Details</h5>
                      <div className="details-grid">
                        <div className="detail-item">
                          <strong>Parameters:</strong>
                          <span>{item.request.parameters.join(', ') || 'None'}</span>
                        </div>
                        <div className="detail-item">
                          <strong>Return Type:</strong>
                          <span>{item.request.return_type || 'None'}</span>
                        </div>
                        <div className="detail-item">
                          <strong>Error Handling:</strong>
                          <span>{item.request.error_handling ? 'Yes' : 'No'}</span>
                        </div>
                        {item.request.performance_requirements && (
                          <div className="detail-item">
                            <strong>Performance:</strong>
                            <span>{item.request.performance_requirements}</span>
                          </div>
                        )}
                        {item.request.safety_requirements && (
                          <div className="detail-item">
                            <strong>Safety:</strong>
                            <span>{item.request.safety_requirements}</span>
                          </div>
                        )}
                      </div>
                    </div>

                    {item.result.generated_function && (
                      <div className="details-section">
                        <h5>Generated Code</h5>
                        <div className="code-preview">
                          <pre>{item.result.generated_function.code}</pre>
                        </div>
                      </div>
                    )}

                    {item.validation?.issues && item.validation.issues.length > 0 && (
                      <div className="details-section">
                        <h5>Validation Issues</h5>
                        <div className="issues-list">
                          {item.validation.issues.map((issue, index) => (
                            <div key={index} className={`issue-item ${issue.severity}`}>
                              <div className="issue-header">
                                <span className="issue-category">{issue.category}</span>
                                <span className="issue-severity">{issue.severity}</span>
                              </div>
                              <div className="issue-message">{issue.message}</div>
                              {issue.suggestion && (
                                <div className="issue-suggestion">💡 {issue.suggestion}</div>
                              )}
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      )}

      <style jsx>{`
        .generation-history {
          padding: 24px;
          border-radius: 12px;
          background: white;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          max-width: 1000px;
          margin: 0 auto;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .history-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 24px;
          padding-bottom: 16px;
          border-bottom: 1px solid #e5e7eb;
        }

        .header-content h2 {
          margin: 0 0 8px 0;
          color: #111827;
          font-size: 24px;
        }

        .stats {
          display: flex;
          gap: 16px;
          font-size: 14px;
          color: #6b7280;
        }

        .header-actions {
          display: flex;
          gap: 8px;
        }

        .clear-btn {
          padding: 8px 16px;
          background: #ef4444;
          color: white;
          border: none;
          border-radius: 6px;
          cursor: pointer;
          font-size: 14px;
        }

        .clear-btn:hover {
          background: #dc2626;
        }

        .tab-navigation {
          display: flex;
          margin-bottom: 20px;
          border-bottom: 1px solid #e5e7eb;
        }

        .tab-btn {
          padding: 12px 24px;
          border: none;
          background: none;
          cursor: pointer;
          font-size: 16px;
          font-weight: 500;
          color: #6b7280;
          border-bottom: 2px solid transparent;
        }

        .tab-btn.active {
          color: #3b82f6;
          border-bottom-color: #3b82f6;
        }

        .tab-btn:hover:not(.active) {
          color: #374151;
        }

        .filters-section {
          display: flex;
          flex-direction: column;
          gap: 16px;
          margin-bottom: 24px;
          padding: 20px;
          background: #f9fafb;
          border-radius: 8px;
        }

        .search-group {
          width: 100%;
        }

        .search-input {
          width: 100%;
          padding: 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 14px;
        }

        .filter-controls {
          display: flex;
          gap: 16px;
          flex-wrap: wrap;
        }

        .filter-group {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .filter-group label {
          font-weight: 500;
          color: #374151;
          font-size: 14px;
        }

        .filter-select {
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
        }

        .sort-order-btn {
          padding: 8px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          background: white;
          cursor: pointer;
          font-size: 14px;
        }

        .loading-state,
        .error-state {
          text-align: center;
          padding: 40px;
          color: #6b7280;
        }

        .loading-spinner {
          width: 32px;
          height: 32px;
          border: 3px solid #e5e7eb;
          border-top: 3px solid #3b82f6;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin: 0 auto 16px;
        }

        @keyframes spin {
          0% {
            transform: rotate(0deg);
          }
          100% {
            transform: rotate(360deg);
          }
        }

        .error-icon {
          font-size: 48px;
          margin-bottom: 16px;
        }

        .retry-btn {
          padding: 8px 16px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          margin-top: 16px;
        }

        .history-list {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .empty-state {
          text-align: center;
          padding: 60px 20px;
          color: #6b7280;
        }

        .empty-icon {
          font-size: 64px;
          margin-bottom: 16px;
        }

        .empty-state h3 {
          margin: 0 0 8px 0;
          color: #374151;
        }

        .history-item {
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 16px;
          background: white;
          transition: all 0.2s ease;
        }

        .history-item:hover {
          border-color: #d1d5db;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
        }

        .item-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 12px;
        }

        .item-title h4 {
          margin: 0 0 4px 0;
          color: #111827;
          font-size: 16px;
        }

        .item-meta {
          display: flex;
          gap: 12px;
          font-size: 12px;
          color: #6b7280;
        }

        .language-badge {
          color: white;
          padding: 2px 8px;
          border-radius: 12px;
          font-size: 11px;
          font-weight: 500;
        }

        .item-actions {
          display: flex;
          gap: 8px;
        }

        .action-btn {
          padding: 6px 8px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          background: white;
          cursor: pointer;
          font-size: 14px;
          transition: all 0.2s;
        }

        .action-btn:hover {
          border-color: #9ca3af;
        }

        .favorite-btn.active {
          background: #fef3c7;
          border-color: #f59e0b;
        }

        .delete-btn:hover {
          background: #fef2f2;
          border-color: #ef4444;
        }

        .item-content {
          margin-bottom: 12px;
        }

        .purpose {
          color: #374151;
          margin-bottom: 8px;
          line-height: 1.4;
        }

        .success-details,
        .error-details {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .metrics {
          display: flex;
          gap: 16px;
          flex-wrap: wrap;
        }

        .metric {
          display: flex;
          gap: 4px;
          align-items: center;
          font-size: 14px;
        }

        .metric-label {
          color: #6b7280;
        }

        .validation-summary {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          background: #f9fafb;
          border-radius: 4px;
        }

        .error-message {
          color: #dc2626;
          font-size: 14px;
        }

        .item-details {
          margin-top: 16px;
          padding-top: 16px;
          border-top: 1px solid #e5e7eb;
        }

        .details-section {
          margin-bottom: 16px;
        }

        .details-section h5 {
          margin: 0 0 8px 0;
          color: #374151;
          font-size: 14px;
        }

        .details-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 8px;
        }

        .detail-item {
          display: flex;
          justify-content: space-between;
          padding: 4px 0;
          font-size: 13px;
        }

        .detail-item strong {
          color: #374151;
        }

        .code-preview {
          background: #f3f4f6;
          border-radius: 4px;
          padding: 12px;
          overflow-x: auto;
        }

        .code-preview pre {
          margin: 0;
          font-size: 12px;
          line-height: 1.4;
          color: #111827;
        }

        .issues-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .issue-item {
          padding: 8px 12px;
          border-radius: 4px;
          font-size: 13px;
        }

        .issue-item.high {
          background: #fef2f2;
          border-left: 4px solid #ef4444;
        }

        .issue-item.medium {
          background: #fef3c7;
          border-left: 4px solid #f59e0b;
        }

        .issue-item.low {
          background: #f0fdf4;
          border-left: 4px solid #10b981;
        }

        .issue-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 4px;
        }

        .issue-category {
          font-weight: 600;
          color: #374151;
        }

        .issue-severity {
          font-size: 11px;
          padding: 2px 6px;
          border-radius: 10px;
          text-transform: uppercase;
        }

        .issue-item.high .issue-severity {
          background: #fecaca;
          color: #dc2626;
        }

        .issue-item.medium .issue-severity {
          background: #fde68a;
          color: #d97706;
        }

        .issue-item.low .issue-severity {
          background: #bbf7d0;
          color: #166534;
        }

        .issue-suggestion {
          margin-top: 4px;
          font-size: 12px;
          color: #059669;
          font-style: italic;
        }

        @media (max-width: 768px) {
          .generation-history {
            padding: 16px;
          }

          .history-header {
            flex-direction: column;
            align-items: stretch;
            gap: 16px;
          }

          .item-header {
            flex-direction: column;
            align-items: stretch;
            gap: 12px;
          }

          .item-actions {
            justify-content: flex-end;
          }

          .filter-controls {
            flex-direction: column;
            align-items: stretch;
          }

          .metrics {
            flex-direction: column;
            gap: 4px;
          }

          .validation-summary {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
          }

          .details-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};

export default GenerationHistory;
