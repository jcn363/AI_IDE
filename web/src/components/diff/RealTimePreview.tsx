import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface DiffRegion {
    lineStart: number;
    lineEnd: number;
    charStart?: number;
    charEnd?: number;
    type: 'addition' | 'deletion' | 'modification' | 'conflict';
    oldContent?: string;
    newContent?: string;
}

interface PreviewSettings {
    showLineNumbers: boolean;
    wrapLines: boolean;
    highlightChanges: boolean;
    showWhitespace: boolean;
    theme: 'light' | 'dark';
    fontSize: number;
}

interface RealTimePreviewState {
    originalContent: string;
    modifiedContent: string;
    diffRegions: DiffRegion[];
    currentRegion: number | null;
    isLoading: boolean;
    hasChanges: boolean;
    zoomLevel: number;
    settings: PreviewSettings;
    statistics: {
        additions: number;
        deletions: number;
        modifications: number;
        conflicts: number;
    };
}

class RealTimePreview extends React.Component<{
    originalContent: string;
    modifiedContent: string;
    onRegionSelect?: (region: DiffRegion) => void;
    onApplyChanges?: (changes: DiffRegion[]) => void;
    onDiscardChanges?: () => void;
}, RealTimePreviewState> {
    private diffViewerRef: React.RefObject<HTMLDivElement>;

    constructor(props: any) {
        super(props);

        this.diffViewerRef = React.createRef();

        this.state = {
            originalContent: '',
            modifiedContent: '',
            diffRegions: [],
            currentRegion: null,
            isLoading: false,
            hasChanges: false,
            zoomLevel: 1,
            settings: {
                showLineNumbers: true,
                wrapLines: false,
                highlightChanges: true,
                showWhitespace: true,
                theme: 'dark',
                fontSize: 14,
            },
            statistics: {
                additions: 0,
                deletions: 0,
                modifications: 0,
                conflicts: 0,
            },
        };
    }

    componentDidMount() {
        this.computeDiff();
    }

    componentDidUpdate(prevProps: any) {
        if (prevProps.originalContent !== this.props.originalContent ||
            prevProps.modifiedContent !== this.props.modifiedContent) {
            this.computeDiff();
        }
    }

    private async computeDiff() {
        if (!this.props.originalContent || !this.props.modifiedContent) return;

        this.setState({ isLoading: true, hasChanges: false });

        try {
            // Use Tauri to compute the diff (in a real implementation, this would call a backend diff algorithm)
            const diffResult = await invoke<{
                regions: DiffRegion[];
                hasChanges: boolean;
                statistics: typeof this.state.statistics;
            }>('compute_content_diff', {
                old_content: this.props.originalContent,
                new_content: this.props.modifiedContent,
            });

            this.setState({
                originalContent: this.props.originalContent,
                modifiedContent: this.props.modifiedContent,
                diffRegions: diffResult.regions,
                hasChanges: diffResult.hasChanges,
                statistics: diffResult.statistics,
                isLoading: false,
            });

        } catch (error) {
            console.error('Failed to compute diff:', error);
            // Fallback to client-side diff computation
            const clientDiff = this.computeClientSideDiff();
            this.setState({
                ...clientDiff,
                isLoading: false,
            });
        }
    }

    private computeClientSideDiff(): Omit<RealTimePreviewState, 'isLoading' | 'settings' | 'zoomLevel'> {
        const oldLines = this.props.originalContent.split('\n');
        const newLines = this.props.modifiedContent.split('\n');

        // Simple line-based diff (in a real implementation you'd use a proper diff algorithm)
        const regions: DiffRegion[] = [];
        let additions = 0;
        let deletions = 0;
        let modifications = 0;

        // Find differences line by line
        const maxLines = Math.max(oldLines.length, newLines.length);
        for (let i = 0; i < maxLines; i++) {
            const oldLine = oldLines[i] || '';
            const newLine = newLines[i] || '';

            if (oldLine !== newLine) {
                if (!oldLine && newLine) {
                    regions.push({
                        lineStart: i,
                        lineEnd: i,
                        type: 'addition',
                        newContent: newLine,
                    });
                    additions++;
                } else if (oldLine && !newLine) {
                    regions.push({
                        lineStart: i,
                        lineEnd: i,
                        type: 'deletion',
                        oldContent: oldLine,
                    });
                    deletions++;
                } else {
                    regions.push({
                        lineStart: i,
                        lineEnd: i,
                        type: 'modification',
                        oldContent: oldLine,
                        newContent: newLine,
                    });
                    modifications++;
                }
            }
        }

        return {
            originalContent: this.props.originalContent,
            modifiedContent: this.props.modifiedContent,
            diffRegions: regions,
            currentRegion: null,
            hasChanges: regions.length > 0,
            statistics: {
                additions,
                deletions,
                modifications,
                conflicts: 0,
            },
        };
    }

    private selectRegion(region: DiffRegion, index: number) {
        this.setState({ currentRegion: index });
        if (this.props.onRegionSelect) {
            this.props.onRegionSelect(region);
        }
    }

    private toggleSetting(setting: keyof PreviewSettings) {
        this.setState(prevState => ({
            settings: {
                ...prevState.settings,
                [setting]: typeof prevState.settings[setting] === 'boolean'
                    ? !prevState.settings[setting]
                    : prevState.settings[setting] as any,
            },
        }));
    }

    private applyChanges() {
        if (this.props.onApplyChanges) {
            this.props.onApplyChanges(this.state.diffRegions);
        }
    }

    private discardChanges() {
        if (this.props.onDiscardChanges) {
            this.props.onDiscardChanges();
        }
    }

    render() {
        const { settings, diffRegions, currentRegion, hasChanges, statistics, isLoading } = this.state;

        return (
            <div className={`realtime-preview ${settings.theme}`}>
                <div className="preview-header">
                    <h4>Refactoring Preview</h4>
                    <div className="preview-controls">
                        <div className="settings-toggle">
                            <button
                                className={`toggle-btn ${settings.showLineNumbers ? 'active' : ''}`}
                                onClick={() => this.toggleSetting('showLineNumbers')}
                                title="Show Line Numbers"
                            >
                                📊
                            </button>
                            <button
                                className={`toggle-btn ${settings.wrapLines ? 'active' : ''}`}
                                onClick={() => this.toggleSetting('wrapLines')}
                                title="Wrap Lines"
                            >
                                ↩️
                            </button>
                            <button
                                className={`toggle-btn ${settings.highlightChanges ? 'active' : ''}`}
                                onClick={() => this.toggleSetting('highlightChanges')}
                                title="Highlight Changes"
                            >
                                🖍️
                            </button>
                        </div>
                        <div className="zoom-controls">
                            <button
                                onClick={() => this.setState(prev => ({ zoomLevel: Math.max(0.5, prev.zoomLevel - 0.2) }))}
                            >
                                🔍-
                            </button>
                            <span>{Math.round(this.state.zoomLevel * 100)}%</span>
                            <button
                                onClick={() => this.setState(prev => ({ zoomLevel: Math.min(3, prev.zoomLevel + 0.2) }))}
                            >
                                🔍+
                            </button>
                        </div>
                    </div>
                </div>

                {isLoading ? (
                    <div className="loading-preview">
                        <div className="spinner"></div>
                        <p>Computing diff...</p>
                    </div>
                ) : (
                    <>
                        {this.renderDiffStatistics()}

                        {hasChanges ? (
                            <div className="diff-viewer" ref={this.diffViewerRef}>
                                <div className="diff-header">
                                    <div className="original-panel">
                                        <h5>Original</h5>
                                    </div>
                                    <div className="modified-panel">
                                        <h5>Modified</h5>
                                    </div>
                                </div>

                                <div className="diff-content" style={{ fontSize: `${settings.fontSize * this.state.zoomLevel}px` }}>
                                    {settings.showLineNumbers && this.renderLineNumbers()}
                                    {settings.highlightChanges ? this.renderHighlightedDiff() : this.renderSideBySideDiff()}
                                </div>
                            </div>
                        ) : (
                            <div className="no-changes">
                                <div className="no-changes-icon">✅</div>
                                <h5>No Changes Detected</h5>
                                <p>The refactoring operation didn't produce any code changes.</p>
                            </div>
                        )}

                        {hasChanges && this.renderChangeActions()}
                    </>
                )}
            </div>
        );
    }

    private renderDiffStatistics() {
        const { statistics } = this.state;

        return (
            <div className="diff-statistics">
                <div className="stats-grid">
                    <div className="stat-item additions">
                        <span className="stat-icon">➕</span>
                        <span className="stat-label">Additions</span>
                        <span className="stat-value">{statistics.additions}</span>
                    </div>
                    <div className="stat-item deletions">
                        <span className="stat-icon">➖</span>
                        <span className="stat-label">Deletions</span>
                        <span className="stat-value">{statistics.deletions}</span>
                    </div>
                    <div className="stat-item modifications">
                        <span className="stat-icon">✏️</span>
                        <span className="stat-label">Modifications</span>
                        <span className="stat-value">{statistics.modifications}</span>
                    </div>
                    <div className="stat-item conflicts">
                        <span className="stat-icon">⚠️</span>
                        <span className="stat-label">Conflicts</span>
                        <span className="stat-value">{statistics.conflicts}</span>
                    </div>
                </div>

                <div className="region-navigation">
                    <button
                        onClick={() => this.setState(prev => ({
                            currentRegion: prev.currentRegion !== null ?
                                Math.max(0, prev.currentRegion - 1) :
                                this.state.diffRegions.length - 1
                        }))}
                        disabled={this.state.diffRegions.length === 0}
                    >
                        ▲ Prev
                    </button>
                    <span>
                        {this.state.currentRegion !== null ? this.state.currentRegion + 1 : 0} / {this.state.diffRegions.length}
                    </span>
                    <button
                        onClick={() => this.setState(prev => ({
                            currentRegion: prev.currentRegion !== null ?
                                Math.min(this.state.diffRegions.length - 1, prev.currentRegion + 1) :
                                0
                        }))}
                        disabled={this.state.diffRegions.length === 0}
                    >
                        ▼ Next
                    </button>
                </div>
            </div>
        );
    }

    private renderLineNumbers() {
        const originalLines = this.state.originalContent.split('\n');
        const modifiedLines = this.state.modifiedContent.split('\n');
        const maxLines = Math.max(originalLines.length, modifiedLines.length);

        const lineNumbers = [];
        for (let i = 1; i <= maxLines; i++) {
            lineNumbers.push(i);
        }

        return (
            <div className="line-numbers">
                <div className="original-line-numbers">
                    {lineNumbers.map(num => (
                        <div key={`orig-${num}`} className="line-number">
                            {num}
                        </div>
                    ))}
                </div>
                <div className="modified-line-numbers">
                    {lineNumbers.map(num => (
                        <div key={`mod-${num}`} className="line-number">
                            {num}
                        </div>
                    ))}
                </div>
            </div>
        );
    }

    private renderSideBySideDiff() {
        const originalLines = this.state.originalContent.split('\n');
        const modifiedLines = this.state.modifiedContent.split('\n');
        const maxLines = Math.max(originalLines.length, modifiedLines.length);

        const lines = [];
        for (let i = 0; i < maxLines; i++) {
            const originalLine = originalLines[i] || '';
            const modifiedLine = modifiedLines[i] || '';
            const isDifferent = originalLine !== modifiedLine;
            const lineClass = `diff-line ${isDifferent ? 'different' : ''}`;

            lines.push(
                <div key={i} className={lineClass}>
                    <div className="original-line">
                        {isDifferent ? originalLine : null}
                    </div>
                    <div className="modified-line">
                        {isDifferent ? modifiedLine : originalLine}
                    </div>
                </div>
            );
        }

        return <div className="diff-container">{lines}</div>;
    }

    private renderHighlightedDiff() {
        const { diffRegions } = this.state;
        const originalLines = this.state.originalContent.split('\n');
        const modifiedLines = this.state.modifiedContent.split('\n');
        const maxLines = Math.max(originalLines.length, modifiedLines.length);

        const processedLines: Array<{
            original: string;
            modified: string;
            className: string;
            highlighted?: boolean;
        }> = [];

        for (let i = 0; i < maxLines; i++) {
            const originalLine = originalLines[i] || '';
            const modifiedLine = modifiedLines[i] || '';

            // Check if this line is part of a diff region
            const regionIndex = diffRegions.findIndex(region =>
                i >= region.lineStart && i <= region.lineEnd
            );

            const region = regionIndex !== -1 ? diffRegions[regionIndex] : null;
            const isSelected = this.state.currentRegion === regionIndex;

            let originalProcessed = originalLine;
            let modifiedProcessed = modifiedLine;

            if (region && this.state.settings.showWhitespace) {
                originalProcessed = originalProcessed.replace(/ /g, '·').replace(/\t/g, '→');
                modifiedProcessed = modifiedProcessed.replace(/ /g, '·').replace(/\t/g, '→');
            }

            processedLines.push({
                original: originalProcessed,
                modified: modifiedProcessed,
                className: region ? `diff-region ${region.type} ${isSelected ? 'selected' : ''}` : '',
                highlighted: !!region,
            });
        }

        return (
            <div className="highlighted-diff">
                {processedLines.map((line, index) => (
                    <div
                        key={index}
                        className={`diff-line ${line.className}`}
                        onClick={() => {
                            const regionIndex = diffRegions.findIndex(region =>
                                index >= region.lineStart && index <= region.lineEnd
                            );
                            if (regionIndex !== -1) {
                                this.selectRegion(diffRegions[regionIndex], regionIndex);
                            }
                        }}
                    >
                        <div className="original-content">
                            <pre>{line.original}</pre>
                            {line.highlighted && <div className="diff-indicator original">-</div>}
                        </div>
                        <div className="modified-content">
                            <pre>{line.modified}</pre>
                            {line.highlighted && <div className="diff-indicator modified">+</div>}
                        </div>
                    </div>
                ))}
            </div>
        );
    }

    private renderChangeActions() {
        return (
            <div className="change-actions">
                <div className="action-buttons">
                    <button
                        className="btn btn-primary"
                        onClick={() => this.applyChanges()}
                    >
                        ✅ Apply Changes
                    </button>
                    <button
                        className="btn btn-secondary"
                        onClick={() => this.discardChanges()}
                    >
                        ❌ Discard Changes
                    </button>
                    <button
                        className="btn btn-outline"
                        onClick={() => this.computeDiff()}
                    >
                        🔄 Refresh Preview
                    </button>
                </div>

                <div className="action-info">
                    <p>
                        This preview shows how the refactoring will affect {this.state.diffRegions.length} regions of code.
                        Review the changes carefully before applying.
                    </p>
                </div>
            </div>
        );
    }
}

export default RealTimePreview;