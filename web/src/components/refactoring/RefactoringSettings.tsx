import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface RefactoringSettingsProps {
    onSettingsChange?: (settings: RefactoringSettings) => void;
}

interface RefactoringSettingsState {
    settings: RefactoringSettings;
    loading: boolean;
    saving: boolean;
}

interface RefactoringSettings {
    ai: {
        enabled: boolean;
        model: string;
        confidenceThreshold: number;
        maxContextTokens: number;
        offlineMode: boolean;
    };
    safety: {
        safetyLevel: 'low' | 'medium' | 'high' | 'maximum';
        skipValidation: boolean;
        allowRiskyOperations: boolean;
        maxFileChanges: number;
    };
    operations: {
        generateTests: boolean;
        preserveComments: boolean;
        autoFormat: boolean;
        previewMode: boolean;
        batchOperations: boolean;
        backgroundProcessing: boolean;
    };
    ui: {
        showConfidence: boolean;
        showImpact: boolean;
        showDuration: boolean;
        autoApplyHighConfidence: boolean;
        darkMode: boolean;
    };
    operationsEnabled: {
        [key: string]: boolean;
    };
}

class RefactoringSettings extends React.Component<RefactoringSettingsProps, RefactoringSettingsState> {
    constructor(props: RefactoringSettingsProps) {
        super(props);

        // Default settings
        const defaultSettings: RefactoringSettings = {
            ai: {
                enabled: true,
                model: 'claude-3-sonnet',
                confidenceThreshold: 0.7,
                maxContextTokens: 8000,
                offlineMode: false,
            },
            safety: {
                safetyLevel: 'medium',
                skipValidation: false,
                allowRiskyOperations: false,
                maxFileChanges: 50,
            },
            operations: {
                generateTests: true,
                preserveComments: true,
                autoFormat: true,
                previewMode: true,
                batchOperations: true,
                backgroundProcessing: false,
            },
            ui: {
                showConfidence: true,
                showImpact: true,
                showDuration: false,
                autoApplyHighConfidence: false,
                darkMode: false,
            },
            operationsEnabled: {
                extractFunction: true,
                extractVariable: true,
                rename: true,
                inline: true,
                moveToModule: true,
                extractInterface: true,
                convertToAsync: true,
                splitClass: true,
                mergeClasses: false,
                patternConversion: true,
                moveMethod: true,
                changeSignature: true,
            },
        };

        this.state = {
            settings: defaultSettings,
            loading: true,
            saving: false,
        };
    }

    componentDidMount() {
        this.loadSettings();
    }

    async loadSettings() {
        try {
            const savedSettings = await invoke<Partial<RefactoringSettings>>('get_refactoring_settings', {});
            if (savedSettings) {
                this.setState(prevState => ({
                    settings: { ...prevState.settings, ...savedSettings },
                    loading: false,
                }));
            }
        } catch (error) {
            console.error('Failed to load settings:', error);
            // Continue with defaults
            this.setState({ loading: false });
        }
    }

    async saveSettings() {
        this.setState({ saving: true });

        try {
            await invoke('save_refactoring_settings', { settings: this.state.settings });

            if (this.props.onSettingsChange) {
                this.props.onSettingsChange(this.state.settings);
            }

            // Show success feedback would go here
        } catch (error) {
            console.error('Failed to save settings:', error);
            // Show error feedback would go here
        } finally {
            this.setState({ saving: false });
        }
    }

    handleAiSettingChange = (key: keyof RefactoringSettings['ai'], value: any) => {
        this.setState(prevState => ({
            settings: {
                ...prevState.settings,
                ai: {
                    ...prevState.settings.ai,
                    [key]: value,
                },
            },
        }));
    };

    handleSafetySettingChange = (key: keyof RefactoringSettings['safety'], value: any) => {
        this.setState(prevState => ({
            settings: {
                ...prevState.settings,
                safety: {
                    ...prevState.settings.safety,
                    [key]: value,
                },
            },
        }));
    };

    handleOperationSettingChange = (key: keyof RefactoringSettings['operations'], value: boolean) => {
        this.setState(prevState => ({
            settings: {
                ...prevState.settings,
                operations: {
                    ...prevState.settings.operations,
                    [key]: value,
                },
            },
        }));
    };

    handleUiSettingChange = (key: keyof RefactoringSettings['ui'], value: boolean) => {
        this.setState(prevState => ({
            settings: {
                ...prevState.settings,
                ui: {
                    ...prevState.settings.ui,
                    [key]: value,
                },
            },
        }));
    };

    handleEnabledOperationChange = (operation: string, enabled: boolean) => {
        this.setState(prevState => ({
            settings: {
                ...prevState.settings,
                operationsEnabled: {
                    ...prevState.settings.operationsEnabled,
                    [operation]: enabled,
                },
            },
        }));
    };

    resetToDefaults = () => {
        const defaultSettings: RefactoringSettings = {
            ai: {
                enabled: true,
                model: 'claude-3-sonnet',
                confidenceThreshold: 0.7,
                maxContextTokens: 8000,
                offlineMode: false,
            },
            safety: {
                safetyLevel: 'medium',
                skipValidation: false,
                allowRiskyOperations: false,
                maxFileChanges: 50,
            },
            operations: {
                generateTests: true,
                preserveComments: true,
                autoFormat: true,
                previewMode: true,
                batchOperations: true,
                backgroundProcessing: false,
            },
            ui: {
                showConfidence: true,
                showImpact: true,
                showDuration: false,
                autoApplyHighConfidence: false,
                darkMode: false,
            },
            operationsEnabled: {
                extractFunction: true,
                extractVariable: true,
                rename: true,
                inline: true,
                moveToModule: true,
                extractInterface: true,
                convertToAsync: true,
                splitClass: true,
                mergeClasses: false,
                patternConversion: true,
                moveMethod: true,
                changeSignature: true,
            },
        };

        this.setState({ settings: defaultSettings });
    };

    render() {
        const { settings, loading, saving } = this.state;

        if (loading) {
            return <div className="settings-loading">Loading settings...</div>;
        }

        return (
            <div className="refactoring-settings">
                <div className="settings-header">
                    <h3>Refactoring Settings</h3>
                    <div className="settings-actions">
                        <button
                            className="btn btn-outline"
                            onClick={this.resetToDefaults}
                            disabled={saving}
                        >
                            Reset to Defaults
                        </button>
                        <button
                            className="btn btn-primary"
                            onClick={() => this.saveSettings()}
                            disabled={saving}
                        >
                            {saving ? 'Saving...' : 'Save Settings'}
                        </button>
                    </div>
                </div>

                <div className="settings-content">
                    {this.renderAISettings()}
                    {this.renderSafetySettings()}
                    {this.renderOperationSettings()}
                    {this.renderUISettings()}
                    {this.renderEnabledOperations()}
                </div>
            </div>
        );
    }

    renderAISettings() {
        const { ai } = this.state.settings;

        return (
            <div className="settings-section">
                <h4>AI Configuration</h4>
                <div className="settings-group">
                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={ai.enabled}
                                onChange={(e) => this.handleAiSettingChange('enabled', e.target.checked)}
                            />
                            Enable AI-powered refactoring
                        </label>
                    </div>

                    {ai.enabled && (
                        <>
                            <div className="form-group">
                                <label htmlFor="ai-model">AI Model</label>
                                <select
                                    id="ai-model"
                                    value={ai.model}
                                    onChange={(e) => this.handleAiSettingChange('model', e.target.value)}
                                    className="form-control"
                                >
                                    <option value="claude-3-sonnet">Claude 3 Sonnet</option>
                                    <option value="claude-3-haiku">Claude 3 Haiku</option>
                                    <option value="gpt-4">GPT-4</option>
                                    <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
                                    <option value="local-model">Local Model (Offline)</option>
                                </select>
                            </div>

                            <div className="form-group">
                                <label htmlFor="confidence-threshold">
                                    Confidence Threshold: {ai.confidenceThreshold}
                                </label>
                                <input
                                    id="confidence-threshold"
                                    type="range"
                                    min="0.1"
                                    max="1.0"
                                    step="0.1"
                                    value={ai.confidenceThreshold}
                                    onChange={(e) => this.handleAiSettingChange('confidenceThreshold', parseFloat(e.target.value))}
                                    className="form-control"
                                />
                            </div>

                            <div className="form-group">
                                <label htmlFor="max-context">
                                    Max Context Tokens: {ai.maxContextTokens}
                                </label>
                                <select
                                    id="max-context"
                                    value={ai.maxContextTokens}
                                    onChange={(e) => this.handleAiSettingChange('maxContextTokens', parseInt(e.target.value))}
                                    className="form-control"
                                >
                                    <option value="2000">2K</option>
                                    <option value="4000">4K</option>
                                    <option value="8000">8K</option>
                                    <option value="16000">16K</option>
                                    <option value="32000">32K</option>
                                </select>
                            </div>

                            <div className="form-group">
                                <label className="checkbox-label">
                                    <input
                                        type="checkbox"
                                        checked={ai.offlineMode}
                                        onChange={(e) => this.handleAiSettingChange('offlineMode', e.target.checked)}
                                    />
                                    Offline mode (use local models only)
                                </label>
                            </div>
                        </>
                    )}
                </div>
            </div>
        );
    }

    renderSafetySettings() {
        const { safety } = this.state.settings;

        return (
            <div className="settings-section">
                <h4>Safety & Validation</h4>
                <div className="settings-group">
                    <div className="form-group">
                        <label htmlFor="safety-level">Safety Level</label>
                        <select
                            id="safety-level"
                            value={safety.safetyLevel}
                            onChange={(e) => this.handleSafetySettingChange('safetyLevel', e.target.value as any)}
                            className="form-control"
                        >
                            <option value="low">Low (Minimal validation)</option>
                            <option value="medium">Medium (Balanced)</option>
                            <option value="high">High (Thorough validation)</option>
                            <option value="maximum">Maximum (Extensive checks)</option>
                        </select>
                    </div>

                    <div className="form-group">
                        <label htmlFor="max-changes">
                            Maximum file changes: {safety.maxFileChanges}
                        </label>
                        <input
                            id="max-changes"
                            type="range"
                            min="10"
                            max="500"
                            step="10"
                            value={safety.maxFileChanges}
                            onChange={(e) => this.handleSafetySettingChange('maxFileChanges', parseInt(e.target.value))}
                            className="form-control"
                        />
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={safety.allowRiskyOperations}
                                onChange={(e) => this.handleSafetySettingChange('allowRiskyOperations', e.target.checked)}
                            />
                            Allow risky operations (requires safety level: Low)
                        </label>
                    </div>
                </div>
            </div>
        );
    }

    renderOperationSettings() {
        const { operations } = this.state.settings;

        return (
            <div className="settings-section">
                <h4>Operation Behavior</h4>
                <div className="settings-group">
                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={operations.generateTests}
                                onChange={(e) => this.handleOperationSettingChange('generateTests', e.target.checked)}
                            />
                            Generate tests for refactored code
                        </label>
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={operations.preserveComments}
                                onChange={(e) => this.handleOperationSettingChange('preserveComments', e.target.checked)}
                            />
                            Preserve comments during refactoring
                        </label>
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={operations.autoFormat}
                                onChange={(e) => this.handleOperationSettingChange('autoFormat', e.target.checked)}
                            />
                            Auto-format refactored code
                        </label>
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={operations.previewMode}
                                onChange={(e) => this.handleOperationSettingChange('previewMode', e.target.checked)}
                            />
                            Show refactoring preview before applying
                        </label>
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={operations.batchOperations}
                                onChange={(e) => this.handleOperationSettingChange('batchOperations', e.target.checked)}
                            />
                            Enable batch refactoring operations
                        </label>
                    </div>
                </div>
            </div>
        );
    }

    renderUISettings() {
        const { ui } = this.state.settings;

        return (
            <div className="settings-section">
                <h4>User Interface</h4>
                <div className="settings-group">
                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={ui.showConfidence}
                                onChange={(e) => this.handleUiSettingChange('showConfidence', e.target.checked)}
                            />
                            Show confidence scores
                        </label>
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={ui.showImpact}
                                onChange={(e) => this.handleUiSettingChange('showImpact', e.target.checked)}
                            />
                            Show refactoring impact estimates
                        </label>
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={ui.showDuration}
                                onChange={(e) => this.handleUiSettingChange('showDuration', e.target.checked)}
                            />
                            Show operation duration in UI
                        </label>
                    </div>

                    <div className="form-group">
                        <label className="checkbox-label">
                            <input
                                type="checkbox"
                                checked={ui.autoApplyHighConfidence}
                                onChange={(e) => this.handleUiSettingChange('autoApplyHighConfidence', e.target.checked)}
                            />
                            Auto-apply high confidence suggestions
                        </label>
                    </div>
                </div>
            </div>
        );
    }

    renderEnabledOperations() {
        const { operationsEnabled } = this.state.settings;

        const operationLabels = {
            extractFunction: 'Extract Function',
            extractVariable: 'Extract Variable',
            rename: 'Rename',
            inline: 'Inline',
            moveToModule: 'Move to Module',
            extractInterface: 'Extract Interface',
            convertToAsync: 'Convert to Async',
            splitClass: 'Split Class',
            mergeClasses: 'Merge Classes',
            patternConversion: 'Pattern Conversion',
            moveMethod: 'Move Method',
            changeSignature: 'Change Signature',
        };

        return (
            <div className="settings-section">
                <h4>Enabled Operations</h4>
                <div className="settings-group">
                    {Object.entries(operationsEnabled).map(([operation, enabled]) => (
                        <div key={operation} className="form-group">
                            <label className="checkbox-label">
                                <input
                                    type="checkbox"
                                    checked={enabled}
                                    onChange={(e) => this.handleEnabledOperationChange(operation, e.target.checked)}
                                />
                                {operationLabels[operation as keyof typeof operationLabels]}
                            </label>
                        </div>
                    ))}
                </div>
            </div>
        );
    }
}

export default RefactoringSettings;