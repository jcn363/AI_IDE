import React from 'react';

interface RefactoringSuggestion {
    operationType: RefactoringOperationType;
    name: string;
    description: string;
    confidenceScore: number;
    expectedImpact: RefactoringImpact;
    prerequisites: string[];
    quickFix: boolean;
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

interface RefactoringSuggestionsListProps {
    suggestions: RefactoringSuggestion[];
    onSuggestionClick: (operationType: RefactoringOperationType) => void;
    loading: boolean;
}

interface RefactoringSuggestionsListState {
    expandedSuggestion: string | null;
    sortBy: 'confidence' | 'impact' | 'alphabetical';
}

class RefactoringSuggestionsList extends React.Component<
    RefactoringSuggestionsListProps,
    RefactoringSuggestionsListState
> {
    constructor(props: RefactoringSuggestionsListProps) {
        super(props);

        this.state = {
            expandedSuggestion: null,
            sortBy: 'confidence',
        };
    }

    handleSuggestionClick = (operationType: RefactoringOperationType) => {
        this.props.onSuggestionClick(operationType);
    };

    handleExpandToggle = (operationType: string) => {
        this.setState(prevState => ({
            expandedSuggestion: prevState.expandedSuggestion === operationType
                ? null
                : operationType,
        }));
    };

    handleSortChange = (sortBy: 'confidence' | 'impact' | 'alphabetical') => {
        this.setState({ sortBy });
    };

    getImpactColor(impact: RefactoringImpact): string {
        switch (impact) {
            case 'low': return 'text-green-600 bg-green-50';
            case 'medium': return 'text-yellow-600 bg-yellow-50';
            case 'high': return 'text-red-600 bg-red-50';
            default: return 'text-gray-600 bg-gray-50';
        }
    }

    getImpactIcon(impact: RefactoringImpact): string {
        switch (impact) {
            case 'low': return '🟢';
            case 'medium': return '🟡';
            case 'high': return '🔴';
            default: return '⚪';
        }
    }

    getConfidenceBar(confidence: number): React.ReactElement {
        const percentage = Math.round(confidence * 100);
        const color = confidence > 0.8 ? 'bg-green-500'
                   : confidence > 0.6 ? 'bg-yellow-500'
                   : 'bg-red-500';

        return (
            <div className="confidence-bar">
                <div
                    className={`confidence-fill ${color}`}
                    style={{ width: `${percentage}%` }}
                ></div>
                <span className="confidence-text">{percentage}%</span>
            </div>
        );
    }

    sortSuggestions(suggestions: RefactoringSuggestion[]): RefactoringSuggestion[] {
        const { sortBy } = this.state;

        return [...suggestions].sort((a, b) => {
            switch (sortBy) {
                case 'confidence':
                    return b.confidenceScore - a.confidenceScore;
                case 'impact':
                    const impactOrder = { low: 1, medium: 2, high: 3 };
                    return impactOrder[a.expectedImpact] - impactOrder[b.expectedImpact];
                case 'alphabetical':
                    return a.name.localeCompare(b.name);
                default:
                    return 0;
            }
        });
    }

    render() {
        const { suggestions, loading } = this.props;
        const { expandedSuggestion, sortBy } = this.state;
        const sortedSuggestions = this.sortSuggestions(suggestions);

        if (loading) {
            return (
                <div className="suggestions-loading">
                    <div className="loading-spinner">⟳</div>
                    <span>Analyzing code for refactoring opportunities...</span>
                </div>
            );
        }

        if (suggestions.length === 0) {
            return (
                <div className="no-suggestions">
                    <div className="no-suggestions-icon">💡</div>
                    <h4>No refactoring suggestions found</h4>
                    <p>Try selecting code or changing your cursor position to get suggestions.</p>
                    <button
                        className="btn btn-primary"
                        onClick={() => window.location.reload()} // Simple refresh
                    >
                        Refresh Analysis
                    </button>
                </div>
            );
        }

        return (
            <div className="refactoring-suggestions">
                <div className="suggestions-header">
                    <h4>AI Refactoring Suggestions ({suggestions.length})</h4>
                    <div className="sort-controls">
                        <label>Sort by:</label>
                        <select
                            value={sortBy}
                            onChange={(e) => this.handleSortChange(e.target.value as any)}
                            className="sort-select"
                        >
                            <option value="confidence">Confidence</option>
                            <option value="impact">Impact</option>
                            <option value="alphabetical">Name</option>
                        </select>
                    </div>
                </div>

                <div className="suggestions-list">
                    {sortedSuggestions.map((suggestion, index) => (
                        <div
                            key={suggestion.operationType}
                            className={`suggestion-card ${suggestion.quickFix ? 'quick-fix' : ''}`}
                        >
                            <div className="suggestion-header">
                                <div className="suggestion-meta">
                                    <span className="suggestion-number">#{index + 1}</span>
                                    <span className={`impact-badge ${this.getImpactColor(suggestion.expectedImpact)}`}>
                                        {this.getImpactIcon(suggestion.expectedImpact)} {suggestion.expectedImpact}
                                    </span>
                                    {suggestion.quickFix && (
                                        <span className="quick-fix-badge">⚡ Quick Fix</span>
                                    )}
                                </div>

                                <div className="suggestion-actions">
                                    <button
                                        className="btn-icon expanded-toggle"
                                        onClick={() => this.handleExpandToggle(suggestion.operationType)}
                                        title={expandedSuggestion === suggestion.operationType ? 'Collapse' : 'Expand'}
                                    >
                                        {expandedSuggestion === suggestion.operationType ? '▼' : '▶'}
                                    </button>
                                    <button
                                        className="btn btn-primary suggestion-apply"
                                        onClick={() => this.handleSuggestionClick(suggestion.operationType)}
                                        title="Apply this refactoring"
                                    >
                                        Apply
                                    </button>
                                </div>
                            </div>

                            <div className="suggestion-content">
                                <h5 className="suggestion-name">{suggestion.name}</h5>
                                <p className="suggestion-description">{suggestion.description}</p>

                                <div className="confidence-section">
                                    <label>Confidence:</label>
                                    {this.getConfidenceBar(suggestion.confidenceScore)}
                                </div>
                            </div>

                            {expandedSuggestion === suggestion.operationType && (
                                <div className="suggestion-details">
                                    <div className="prerequisites-section">
                                        <h6>Prerequisites:</h6>
                                        {suggestion.prerequisites.length > 0 ? (
                                            <ul>
                                                {suggestion.prerequisites.map((prereq, idx) => (
                                                    <li key={idx}>{prereq}</li>
                                                ))}
                                            </ul>
                                        ) : (
                                            <p>None - ready to apply!</p>
                                        )}
                                    </div>

                                    <div className="benefits-section">
                                        <h6>Expected Benefits:</h6>
                                        <ul>
                                            <li>Improves code maintainability</li>
                                            <li>Reduces cognitive complexity</li>
                                            <li>Enhances testability</li>
                                        </ul>
                                    </div>
                                </div>
                            )}
                        </div>
                    ))}
                </div>
            </div>
        );
    }
}

export default RefactoringSuggestionsList;
export type { RefactoringSuggestion, RefactoringOperationType, RefactoringImpact };