import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import RefactoringAnalytics, { RefactoringEvent, AnalyticsData } from '../../utils/analytics/RefactoringAnalytics';

interface SuggestionRule {
    id: string;
    name: string;
    description: string;
    triggerCondition: (context: ContextualInformation) => boolean;
    signature: {
        priority: number;
        confidence: number;
        actionable: boolean;
        automationLevel: 'manual' | 'semi' | 'auto';
    };
    operation: string;
    parameters?: { [key: string]: any };
}

interface ContextualInformation {
    currentFile: {
        path: string;
        content: string;
        language: string;
        lines: string[];
        lineCount: number;
        complexity: number;
    };
    cursorPosition: {
        line: number;
        character: number;
        selectedText?: string;
        selectionRange?: { start: number; end: number };
    };
    environment: {
        projectSize: number;
        activeFiles: string[];
        recentChanges: Array<{ file: string; timestamp: number; type: string }>;
        gitStatus?: { branch: string; dirty: boolean };
    };
    analytics: AnalyticsData | null;
    userProfile: {
        experience: 'novice' | 'intermediate' | 'expert';
        preferredOperations: string[];
        style: 'pragmatic' | 'clean' | 'performant' | 'readable';
    };
}

interface ContextAwareSuggestionsState {
    suggestions: Array<{
        rule: SuggestionRule;
        context: ContextualInformation;
        confidence: number;
        reasoning: string[];
        impact: {
            timeSaved: number; // minutes
            complexityReduced: number;
            maintainability: number; // -10 to +10 scale
            performance: number; // -10 to +10 scale
        };
    }>;
    isAnalyzing: boolean;
    lastAnalysisTime: number;
    suggestionHistory: Array<{
        suggestionId: string;
        accepted: boolean;
        timestamp: number;
        outcomes: string[];
    }>;
}

class ContextAwareSuggestions extends React.Component<{
    onSuggestionClick?: (suggestion: any) => void;
    onSuggestionApplied?: (suggestion: any) => void;
}, ContextAwareSuggestionsState> {
    private analytics: RefactoringAnalytics;
    private analysisDebounceTimer: NodeJS.Timeout | null = null;

    constructor(props: any) {
        super(props);

        this.analytics = new RefactoringAnalytics();

        this.state = {
            suggestions: [],
            isAnalyzing: false,
            lastAnalysisTime: 0,
            suggestionHistory: [],
        };
    }

    async componentDidMount() {
        await this.initializeRulesEngine();
    }

    private async initializeRulesEngine() {
        // Load and compile suggestion rules
        try {
            const loadedRules = [
                this.createExtractFunctionRule(),
                this.createRenameRule(),
                this.createInterfaceRule(),
                this.createAsyncConversionRule(),
                this.createDeadCodeRule(),
                this.createComplexityRule(),
                this.createDuplicationRule(),
                this.createErrorHandlingRule(),
            ];

            // Register rules (in a real implementation, this would be more sophisticated)
            this.rules = loadedRules;
        } catch (error) {
            console.error('Failed to initialize rules engine:', error);
        }
    }

    private rules: SuggestionRule[] = [];

    private createExtractFunctionRule(): SuggestionRule {
        return {
            id: 'extract-function-long-method',
            name: 'Extract Complex Function',
            description: 'Long functions (30+ lines) can be broken down into smaller, focused functions',
            triggerCondition: (context) => {
                const currentMethod = this.findCurrentMethod(context);
                return currentMethod && currentMethod.lineCount > 30;
            },
            signature: {
                priority: 8,
                confidence: 0.85,
                actionable: true,
                automationLevel: 'manual',
            },
            operation: 'extractFunction',
        };
    }

    private createRenameRule(): SuggestionRule {
        return {
            id: 'rename-poor-naming',
            name: 'Improve Variable Naming',
            description: 'Variables with generic names like temp/var/data can be more descriptive',
            triggerCondition: (context) => {
                const line = context.currentFile.lines[context.cursorPosition.line];
                return /let\s+\w{1,3}\s*=|var\s*\w{1,3}\s*=|const\s+\w{1,3}\s*=/.test(line);
            },
            signature: {
                priority: 6,
                confidence: 0.70,
                actionable: true,
                automationLevel: 'semi',
            },
            operation: 'rename',
        };
    }

    private createInterfaceRule(): SuggestionRule {
        return {
            id: 'extract-interface-duplication',
            name: 'Extract Common Interface',
            description: 'Similar struct implementations suggest an interface can be extracted',
            triggerCondition: (context) => {
                return this.detectImplementationPatterns(context) > 2;
            },
            signature: {
                priority: 7,
                confidence: 0.75,
                actionable: true,
                automationLevel: 'semi',
            },
            operation: 'extractInterface',
        };
    }

    private createAsyncConversionRule(): SuggestionRule {
        return {
            id: 'convert-sync-to-async',
            name: 'Convert to Async',
            description: 'Long-running synchronous operations should be made asynchronous',
            triggerCondition: (context) => {
                const line = context.currentFile.lines[context.cursorPosition.line];
                return /std::thread::sleep|\.write\(\)|File::read/.test(line);
            },
            signature: {
                priority: 9,
                confidence: 0.80,
                actionable: true,
                automationLevel: 'manual',
            },
            operation: 'convertToAsync',
        };
    }

    private createDeadCodeRule(): SuggestionRule {
        return {
            id: 'remove-unused-imports',
            name: 'Remove Unused Imports',
            description: 'Unused imports should be cleaned up',
            triggerCondition: (context) => {
                return context.analytics?.userBehavior.workflowPatterns.includes('cleanup-focused') || false;
            },
            signature: {
                priority: 4,
                confidence: 0.65,
                actionable: true,
                automationLevel: 'auto',
            },
            operation: 'removeDeadCode',
        };
    }

    private createComplexityRule(): SuggestionRule {
        return {
            id: 'reduce-complexity',
            name: 'Reduce Code Complexity',
            description: 'High complexity methods should be simplified',
            triggerCondition: (context) => context.currentFile.complexity > 10,
            signature: {
                priority: 8,
                confidence: 0.77,
                actionable: true,
                automationLevel: 'manual',
            },
            operation: 'splitClass',
        };
    }

    private createDuplicationRule(): SuggestionRule {
        return {
            id: 'extract-duplication',
            name: 'Extract Code Duplication',
            description: 'Similar code blocks should be extracted into reusable functions',
            triggerCondition: (context) => {
                return this.detectCodeDuplication(context) > 3;
            },
            signature: {
                priority: 7,
                confidence: 0.82,
                actionable: true,
                automationLevel: 'semi',
            },
            operation: 'extractFunction',
        };
    }

    private createErrorHandlingRule(): SuggestionRule {
        return {
            id: 'improve-error-handling',
            name: 'Improve Error Handling',
            description: 'Replace unwrap/expect with proper error handling',
            triggerCondition: (context) => {
                const line = context.currentFile.lines[context.cursorPosition.line];
                return /(\.unwrap\(\)|\.expect\(|panic!)/.test(line);
            },
            signature: {
                priority: 8,
                confidence: 0.85,
                actionable: true,
                automationLevel: 'manual',
            },
            operation: 'patternConversion',
        };
    }

    async analyzeContext(context: ContextualInformation): Promise<void> {
        this.setState({ isAnalyzing: true });

        try {
            const suggestions = [];
            const analytics = await this.analytics.analyzeUsagePatterns();

            for (const rule of this.rules) {
                if (rule.triggerCondition(context)) {
                    const confidence = this.calculateContextConfidence(rule, context, analytics);
                    const reasoning = this.generateReasoning(rule, context);

                    suggestions.push({
                        rule,
                        context,
                        confidence,
                        reasoning,
                        impact: this.estimateImpact(rule, context),
                    });
                }
            }

            // Sort by confidence and priority, then filter top 5
            suggestions.sort((a, b) => {
                const scoreA = (a.confidence * 0.7) + (a.rule.signature.priority * 0.3);
                const scoreB = (b.confidence * 0.7) + (b.rule.signature.priority * 0.3);
                return scoreB - scoreA;
            });

            this.setState({
                suggestions: suggestions.slice(0, 5),
                lastAnalysisTime: Date.now(),
                isAnalyzing: false,
            });

        } catch (error) {
            console.error('Context analysis failed:', error);
            this.setState({ isAnalyzing: false });
        }
    }

    private calculateContextConfidence(rule: SuggestionRule, context: ContextualInformation, analytics: AnalyticsData | null): number {
        let confidence = rule.signature.confidence;

        // Boost confidence based on user history
        if (analytics && analytics.userBehavior.mostUsedOperations.includes(rule.operation)) {
            confidence += 0.1;
        }

        // Adjust based on file complexity
        if (context.currentFile.complexity > 15) {
            if (rule.id.includes('complexity')) confidence += 0.2;
        }

        // Consider recent changes
        if (context.environment.recentChanges.length > 3) {
            if (rule.id.includes('cleanup')) confidence += 0.15;
        }

        // User experience adjustment
        if (context.userProfile.experience === 'novice' && rule.signature.automationLevel === 'auto') {
            confidence += 0.1;
        }
        if (context.userProfile.experience === 'expert' && rule.signature.automationLevel === 'manual') {
            confidence += 0.1;
        }

        return Math.min(confidence, 0.98); // Cap at 98%
    }

    private generateReasoning(rule: SuggestionRule, context: ContextualInformation): string[] {
        const reasoning = [rule.description];

        if (rule.id === 'extract-function-long-method') {
            const currentMethod = this.findCurrentMethod(context);
            if (currentMethod) {
                reasoning.push(`Function "${currentMethod.name}" spans ${currentMethod.lineCount} lines`);
            }
        }

        if (rule.id === 'reduce-complexity') {
            reasoning.push(`Cyclomatic complexity: ${context.currentFile.complexity}`);
        }

        if (rule.id === 'improve-error-handling') {
            reasoning.push('This will make code more robust and debugging-friendly');
        }

        return reasoning;
    }

    private estimateImpact(rule: SuggestionRule, context: ContextualInformation): any {
        const baseTimeSavings = {
            'extractFunction': 5,
            'rename': 2,
            'extractInterface': 15,
            'convertToAsync': 20,
            'removeDeadCode': 3,
            'splitClass': 25,
            'patternConversion': 8,
        };

        return {
            timeSaved: baseTimeSavings[rule.operation as keyof typeof baseTimeSavings] || 5,
            complexityReduced: rule.id.includes('complexity') ? 3 : 1,
            maintainability: rule.id.includes('cleanup') || rule.id.includes('naming') ? 2 : 0,
            performance: rule.id.includes('async') || rule.id.includes('optimize') ? 2 : 0,
        };
    }

    private findCurrentMethod(context: ContextualInformation): { name: string; lineCount: number } | null {
        // Simple method detection - in a real implementation this would use AST parsing
        for (let i = context.cursorPosition.line; i >= 0; i--) {
            const line = context.currentFile.lines[i];
            const methodMatch = line.match(/fn\s+(\w+)\s*\(/);
            if (methodMatch) {
                let endLine = i;
                for (let j = i + 1; j < context.currentFile.lines.length; j++) {
                    if (context.currentFile.lines[j].includes('}')) {
                        endLine = j;
                        break;
                    }
                }
                return {
                    name: methodMatch[1],
                    lineCount: endLine - i + 1,
                };
            }
        }
        return null;
    }

    private detectImplementationPatterns(context: ContextualInformation): number {
        // Simple pattern detection - count similar impl blocks
        const implementations: { [key: string]: number } = {};

        context.currentFile.lines.forEach(line => {
            const implMatch = line.match(/impl\s+(\w+)/);
            if (implMatch) {
                const structName = implMatch[1];
                implementations[structName] = (implementations[structName] || 0) + 1;
            }
        });

        return Object.keys(implementations).filter(key => implementations[key] > 1).length;
    }

    private detectCodeDuplication(context: ContextualInformation): number {
        // Simple duplication detection - count similar lines
        const lineHashes: { [key: string]: number } = {};

        context.currentFile.lines.forEach(line => {
            const trimmed = line.trim();
            if (trimmed.length > 20) { // Only consider substantial lines
                const hash = this.simpleHash(trimmed);
                lineHashes[hash] = (lineHashes[hash] || 0) + 1;
            }
        });

        return Object.values(lineHashes).filter(count => count > 1).length;
    }

    private simpleHash(str: string): string {
        let hash = 0;
        for (let i = 0; i < str.length; i++) {
            const char = str.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32-bit integer
        }
        return hash.toString();
    }

    handleSuggestionClick(suggestion: any) {
        // Track the suggestion click
        this.setState(prevState => ({
            ...prevState,
            suggestions: prevState.suggestions.map(s =>
                s.rule.id === suggestion.rule.id ? { ...s, confidence: Math.min(s.confidence + 0.1, 1) } : s
            ),
        }));

        if (this.props.onSuggestionClick) {
            this.props.onSuggestionClick(suggestion);
        }
    }

    handleSuggestionApplied(suggestion: any) {
        // Track successful application for learning
        const historyEntry = {
            suggestionId: suggestion.rule.id,
            accepted: true,
            timestamp: Date.now(),
            outcomes: [
                `Applied ${suggestion.rule.name}`,
                `Confidence: ${suggestion.confidence.toFixed(2)}`,
                `Impact: ${suggestion.impact.timeSaved} min saved`,
            ],
        };

        this.setState(prevState => ({
            suggestionHistory: [historyEntry, ...prevState.suggestionHistory].slice(0, 100),
        }));

        // Notify parent component
        if (this.props.onSuggestionApplied) {
            this.props.onSuggestionApplied(suggestion);
        }
    }

    render() {
        const { suggestions, isAnalyzing, suggestionHistory } = this.state;

        return (
            <div className="context-aware-suggestions">
                <div className="suggestions-header">
                    <h4>Smart Suggestions</h4>
                    {isAnalyzing && <span className="analyzing-indicator">Analyzing context...</span>}
                    <button
                        className="btn btn-small"
                        onClick={() => this.analyzeContext({} as ContextualInformation)}
                        disabled={isAnalyzing}
                    >
                        🔍 Analyze Again
                    </button>
                </div>

                {suggestions.length > 0 ? (
                    <div className="suggestions-list">
                        {suggestions.map((suggestion, index) => (
                            <div
                                key={suggestion.rule.id}
                                className="suggestion-item"
                                onClick={() => this.handleSuggestionClick(suggestion)}
                            >
                                <div className="suggestion-priority">
                                    #{index + 1}
                                </div>

                                <div className="suggestion-content">
                                    <h5>{suggestion.rule.name}</h5>
                                    <div className="suggestion-confidence">
                                        Confidence: {Math.round(suggestion.confidence * 100)}%
                                    </div>
                                    <p className="suggestion-description">{suggestion.rule.description}</p>

                                    <div className="suggestion-reasoning">
                                        <strong>Why this matters:</strong>
                                        <ul>
                                            {suggestion.reasoning.map((reason, idx) => (
                                                <li key={idx}>{reason}</li>
                                            ))}
                                        </ul>
                                    </div>

                                    <div className="suggestion-impact">
                                        <strong>Impact:</strong>
                                        <span>⚡ {suggestion.impact.timeSaved} min saved</span>
                                        <span>📈 Maintainability +{suggestion.impact.maintainability}</span>
                                        {suggestion.impact.performance > 0 && (
                                            <span>🚀 Performance +{suggestion.impact.performance}</span>
                                        )}
                                    </div>

                                    <div className="suggestion-automation">
                                        <span className={`automation-level ${suggestion.rule.signature.automationLevel}`}>
                                            {suggestion.rule.signature.automationLevel.toUpperCase()}
                                        </span>
                                    </div>
                                </div>

                                <div className="suggestion-actions">
                                    <button
                                        className="btn btn-primary btn-small"
                                        onClick={() => this.handleSuggestionApplied(suggestion)}
                                    >
                                        Apply
                                    </button>
                                    <button className="btn btn-outline btn-small">
                                        View Code
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="no-suggestions">
                        <div className="no-suggestions-icon">💡</div>
                        <h5>No suggestions available</h5>
                        <p>The system is analyzing your code for potential improvements.</p>
                        {suggestionHistory.length > 0 && (
                            <div className="suggestion-history-preview">
                                <h6>Recently applied:</h6>
                                {suggestionHistory.slice(0, 3).map((entry, index) => (
                                    <div key={index} className="history-item">
                                        ✓ {entry.outcomes[0]}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                )}

                <div className="suggestions-footer">
                    <div className="suggestions-stats">
                        <span>Last analysis: {new Date(this.state.lastAnalysisTime).toLocaleTimeString()}</span>
                        <span>History: {suggestionHistory.length} applied</span>
                    </div>
                </div>
            </div>
        );
    }
}

export default ContextAwareSuggestions;
export type { ContextualInformation, SuggestionRule };