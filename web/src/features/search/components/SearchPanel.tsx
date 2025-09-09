import React, { useState, useRef, useEffect, useCallback } from 'react';
import { SearchService } from '../services/SearchService';
import { SearchOptions, SearchResult, SearchHistoryItem } from '../types';
import { useSearch } from '../hooks/useSearch';

interface SearchPanelProps {
  workspacePath: string;
  onResultClick?: (result: SearchResult) => void;
  onSearchHistoryUpdate?: () => void;
}

export const SearchPanel: React.FC<SearchPanelProps> = ({
  workspacePath,
  onResultClick,
  onSearchHistoryUpdate,
}) => {
  const {
    searchState,
    performSearch,
    clearSearch,
    selectResult,
    goToPreviousResult,
    goToNextResult,
  } = useSearch(workspacePath);

  const [searchOptions, setSearchOptions] = useState<SearchOptions>({
    query: '',
    case_sensitive: false,
    whole_word: false,
    regex: false,
    include_hidden: false,
    include_binary: false,
    file_patterns: ['*.rs', '*.toml', '*.md', '*.js', '*.ts', '*.json'],
    exclude_patterns: ['target/*', 'node_modules/*', '.git/*'],
    max_results: 100,
    context_lines: 2,
  });

  const [showAdvanced, setShowAdvanced] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const resultsContainerRef = useRef<HTMLDivElement>(null);

  const handleSearch = useCallback(async () => {
    if (!searchOptions.query.trim()) return;

    const validation = SearchService.getInstance().validateSearchOptions(searchOptions);
    if (!validation.valid) {
      console.error('Search validation failed:', validation.errors);
      return;
    }

    await performSearch(searchOptions);
    onSearchHistoryUpdate?.();
  }, [searchOptions, performSearch, onSearchHistoryUpdate]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    } else if (e.key === 'Escape') {
      clearSearch();
    } else if (searchState.results.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        const nextIndex = Math.min(selectedIndex + 1, searchState.results.length - 1);
        setSelectedIndex(nextIndex);
        selectResult(nextIndex);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        const prevIndex = Math.max(selectedIndex - 1, 0);
        setSelectedIndex(prevIndex);
        selectResult(prevIndex);
      }
    }
  };

  const handleResultClick = (result: SearchResult, index: number) => {
    setSelectedIndex(index);
    selectResult(index);
    onResultClick?.(result);
  };

  const toggleOption = (option: keyof SearchOptions) => {
    setSearchOptions(prev => ({
      ...prev,
      [option]: !prev[option],
    }));
  };

  return (
    <div className="search-panel">
      {/* Search Input Area */}
      <div className="search-input-container">
        <div className="search-input-wrapper">
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search files and symbols... (Ctrl+Shift+F)"
            value={searchOptions.query}
            onChange={(e) => setSearchOptions(prev => ({ ...prev, query: e.target.value }))}
            onKeyDown={handleKeyDown}
            className="search-input"
            disabled={searchState.loading}
          />
          <button
            onClick={handleSearch}
            disabled={!searchOptions.query.trim() || searchState.loading}
            className="search-button"
          >
            {searchState.loading ? '🔄' : '🔍'}
          </button>
        </div>

        {/* Quick Options */}
        <div className="search-quick-options">
          <label className="option-toggle">
            <input
              type="checkbox"
              checked={searchOptions.case_sensitive}
              onChange={() => toggleOption('case_sensitive')}
            />
            Case
          </label>
          <label className="option-toggle">
            <input
              type="checkbox"
              checked={searchOptions.regex}
              onChange={() => toggleOption('regex')}
            />
            Regex
          </label>
          <label className="option-toggle">
            <input
              type="checkbox"
              checked={searchOptions.whole_word}
              onChange={() => toggleOption('whole_word')}
            />
            Word
          </label>
          <div
            className="advanced-toggle"
            onClick={() => setShowAdvanced(!showAdvanced)}
          >
            Advanced ▼
          </div>
        </div>

        {/* Advanced Options */}
        {showAdvanced && (
          <div className="search-advanced-options">
            <div className="option-group">
              <label>File Patterns:</label>
              <input
                type="text"
                value={searchOptions.file_patterns.join(', ')}
                onChange={(e) => setSearchOptions(prev => ({
                  ...prev,
                  file_patterns: e.target.value.split(',').map(s => s.trim())
                }))}
                placeholder="*.rs, *.toml, *.md"
              />
            </div>
            <div className="option-group">
              <label>Exclude Patterns:</label>
              <input
                type="text"
                value={searchOptions.exclude_patterns.join(', ')}
                onChange={(e) => setSearchOptions(prev => ({
                  ...prev,
                  exclude_patterns: e.target.value.split(',').map(s => s.trim())
                }))}
                placeholder="target/*, node_modules/*"
              />
            </div>
            <div className="option-group">
              <label>Max Results:</label>
              <input
                type="number"
                value={searchOptions.max_results || 100}
                onChange={(e) => setSearchOptions(prev => ({
                  ...prev,
                  max_results: parseInt(e.target.value) || 100
                }))}
                min="1"
                max="1000"
              />
            </div>
            <div className="option-group">
              <label>Context Lines:</label>
              <input
                type="number"
                value={searchOptions.context_lines}
                onChange={(e) => setSearchOptions(prev => ({
                  ...prev,
                  context_lines: parseInt(e.target.value) || 2
                }))}
                min="0"
                max="5"
              />
            </div>
          </div>
        )}
      </div>

      {/* Results Summary */}
      {searchState.results.length > 0 && (
        <div className="search-summary">
          <span>
            {searchState.results.length} results found
            {searchState.results.length > (searchOptions.max_results || 100) && ' (truncated)'}
          </span>
          {searchState.results.length > 1 && (
            <div className="navigation-controls">
              <button onClick={goToPreviousResult} disabled={selectedIndex === 0}>
                ↑ Previous
              </button>
              <span>{selectedIndex + 1} / {searchState.results.length}</span>
              <button
                onClick={goToNextResult}
                disabled={selectedIndex === searchState.results.length - 1}
              >
                Next ↓
              </button>
            </div>
          )}
        </div>
      )}

      {/* Error Display */}
      {searchState.error && (
        <div className="search-error">
          <span>⚠️ {searchState.error}</span>
        </div>
      )}

      {/* Results */}
      <div
        ref={resultsContainerRef}
        className="search-results"
        onScroll={(e) => {
          // Handle infinite scrolling if needed in future
        }}
      >
        {searchState.results.map((result, index) => (
          <SearchResultItem
            key={result.id}
            result={result}
            isSelected={index === selectedIndex}
            workspacePath={workspacePath}
            onClick={() => handleResultClick(result, index)}
          />
        ))}

        {searchState.results.length === 0 && !searchState.loading && searchState.current_query && (
          <div className="no-results">
            <span>No results found for "{searchState.current_query}"</span>
          </div>
        )}
      </div>

      {/* Loading Indicator */}
      {searchState.loading && (
        <div className="search-loading">
          <span>🔍 Searching...</span>
          <div className="loading-spinner"></div>
        </div>
      )}

      <style jsx>{`
        .search-panel {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: var(--bg-color);
          color: var(--text-color);
        }

        .search-input-container {
          padding: 16px;
          border-bottom: 1px solid var(--border-color);
          background: var(--bg-color-secondary);
        }

        .search-input-wrapper {
          display: flex;
          gap: 8px;
        }

        .search-input {
          flex: 1;
          padding: 8px 12px;
          font-size: 14px;
          border: 1px solid var(--border-color);
          border-radius: 4px;
          background: var(--bg-color);
          color: var(--text-color);
        }

        .search-input:focus {
          outline: none;
          border-color: var(--accent-color);
          box-shadow: 0 0 0 2px rgba(33, 150, 243, 0.1);
        }

        .search-input:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .search-button {
          padding: 8px 16px;
          font-size: 14px;
          border: 1px solid var(--border-color);
          background: var(--bg-color);
          color: var(--text-color);
          border-radius: 4px;
          cursor: pointer;
        }

        .search-button:hover:not(:disabled) {
          background: var(--bg-color-hover);
        }

        .search-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .search-quick-options {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-top: 8px;
        }

        .option-toggle {
          display: flex;
          align-items: center;
          gap: 4px;
          font-size: 12px;
          cursor: pointer;
        }

        .option-toggle input {
          margin: 0;
        }

        .advanced-toggle {
          font-size: 12px;
          cursor: pointer;
          color: var(--accent-color);
        }

        .advanced-toggle:hover {
          text-decoration: underline;
        }

        .search-advanced-options {
          margin-top: 12px;
          padding-top: 12px;
          border-top: 1px solid var(--border-color);
        }

        .option-group {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
        }

        .option-group label {
          font-size: 12px;
          font-weight: 500;
          min-width: 100px;
        }

        .option-group input {
          flex: 1;
          padding: 4px 8px;
          font-size: 12px;
          border: 1px solid var(--border-color);
          border-radius: 3px;
          background: var(--bg-color);
          color: var(--text-color);
        }

        .search-summary {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 16px;
          border-bottom: 1px solid var(--border-color);
          font-size: 12px;
          background: var(--bg-color-secondary);
        }

        .navigation-controls {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .navigation-controls button {
          padding: 4px 8px;
          font-size: 11px;
          border: 1px solid var(--border-color);
          background: var(--bg-color);
          color: var(--text-color);
          border-radius: 3px;
          cursor: pointer;
        }

        .search-error {
          padding: 8px 16px;
          color: var(--error-color);
          background: var(--error-bg);
          border-bottom: 1px solid var(--border-color);
          font-size: 13px;
        }

        .search-results {
          flex: 1;
          overflow-y: auto;
        }

        .no-results {
          padding: 32px 16px;
          text-align: center;
          color: var(--text-color-secondary);
          font-style: italic;
        }

        .search-loading {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 12px;
          padding: 16px;
          font-size: 14px;
        }

        .loading-spinner {
          width: 16px;
          height: 16px;
          border: 2px solid var(--border-color);
          border-top: 2px solid var(--accent-color);
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
};

// Individual search result component
interface SearchResultItemProps {
  result: SearchResult;
  isSelected: boolean;
  workspacePath: string;
  onClick: () => void;
}

const SearchResultItem: React.FC<SearchResultItemProps> = ({
  result,
  isSelected,
  workspacePath,
  onClick,
}) => {
  const relativePath = result.file_path.replace(workspacePath, '').replace(/^[\/\\]/, '');
  const { text: highlightedText, highlights } = SearchService.highlightMatches(
    result.content,
    result.content.includes('println!') ? 'println' : result.content.split(' ')[0], // For demo
    false,
    false
  );

  return (
    <div
      className={`search-result-item ${isSelected ? 'selected' : ''}`}
      onClick={onClick}
    >
      <div className="result-header">
        <span className="file-path">{relativePath}</span>
        <span className="line-info">:{result.line_number}</span>
      </div>

      <div className="result-content">
        <div className="context-pre">
          {result.context_before.map((line, idx) => (
            <div key={idx} className="context-line">{line}</div>
          ))}
        </div>
        <div className="match-line">
          <span className="line-number">{result.line_number}:</span>
          <span className="content">{result.content}</span>
        </div>
        <div className="context-post">
          {result.context_after.map((line, idx) => (
            <div key={idx} className="context-line">{line}</div>
          ))}
        </div>
      </div>

      <style jsx>{`
        .search-result-item {
          padding: 8px 16px;
          border-bottom: 1px solid var(--border-color);
          cursor: pointer;
          background: var(--bg-color);
        }

        .search-result-item:hover {
          background: var(--bg-color-hover);
        }

        .search-result-item.selected {
          background: var(--bg-color-selected);
          border-left: 3px solid var(--accent-color);
        }

        .result-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 4px;
        }

        .file-path {
          font-weight: 500;
          font-family: var(--monospace-font);
          font-size: 13px;
          color: var(--text-color);
        }

        .line-info {
          font-size: 11px;
          color: var(--text-color-secondary);
          font-family: var(--monospace-font);
        }

        .result-content {
          font-family: var(--monospace-font);
          font-size: 12px;
        }

        .context-line {
          color: var(--text-color-secondary);
          padding-left: 20px;
        }

        .match-line {
          background: rgba(255, 200, 87, 0.1);
          padding: 2px 4px;
          margin: 4px 0;
        }

        .line-number {
          color: var(--text-color-secondary);
          margin-right: 8px;
          user-select: none;
        }

        .content {
          white-space: pre;
        }
      `}</style>
    </div>
  );
};