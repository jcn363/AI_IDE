import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface RecoveryAction {
    id: string;
    title: string;
    description: string;
    actionType: 'retry' | 'alternative' | 'manual' | 'cancel';
    requiresConfirmation: boolean;
    estimatedResolution: number; // seconds
    confidence: number; // 0-1, how likely this will resolve the issue
}

interface ErrorContext {
    operationType: string;
    errorCode: string;
    timestamp: number;
    filePath?: string;
    lineNumber?: number;
    userAction?: string;
    environment: {
        workspaceSize: number;
        memoryUsage: number;
        networkConnectivity: boolean;
        aiModel: string;
    };
}

interface ErrorRecoverySuggestion {
    category: ErrorCategory;
    severity: ErrorSeverity;
    primaryAction: RecoveryAction;
    alternativeActions: RecoveryAction[];
    explanation: string;
    preventions: string[];
    relatedResources: string[];
}

enum ErrorCategory {
    NETWORK = 'network',
    VALIDATION = 'validation',
    AUTHENTICATION = 'authentication',
    RESOURCE = 'resource',
    COMPILATION = 'compilation',
    RUNTIME = 'runtime',
    CONFIGURATION = 'configuration',
    DEPENDENCY = 'dependency',
    PERMISSION = 'permission',
    SYSTEM = 'system',
}

enum ErrorSeverity {
    LOW = 'low',
    MEDIUM = 'medium',
    HIGH = 'high',
    CRITICAL = 'critical',
}

class ErrorRecovery extends React.Component {

    // Error patterns and their most effective recovery strategies
    private static ERROR_PATTERNS = {
        'network_timeout': {
            category: ErrorCategory.NETWORK,
            severity: ErrorSeverity.MEDIUM,
            primaryRecovery: 'retry_with_exponential_backoff',
            alternatives: ['check_network_connectivity', 'switch_to_offline_mode'],
        },
        'invalid_syntax': {
            category: ErrorCategory.VALIDATION,
            severity: ErrorSeverity.HIGH,
            primaryRecovery: 'syntax_suggestion',
            alternatives: ['manual_fix', 'revert_last_change'],
        },
        'ai_model_busy': {
            category: ErrorCategory.RESOURCE,
            severity: ErrorSeverity.MEDIUM,
            primaryRecovery: 'retry_later',
            alternatives: ['switch_ai_model', 'batch_operation_delay'],
        },
        'compilation_error': {
            category: ErrorCategory.COMPILATION,
            severity: ErrorSeverity.HIGH,
            primaryRecovery: 'fix_compilation_issues',
            alternatives: ['rollback_changes', 'syntax_assistance'],
        },
        'permission_denied': {
            category: ErrorCategory.PERMISSION,
            severity: ErrorSeverity.CRITICAL,
            primaryRecovery: 'grant_permissions',
            alternatives: ['run_as_administrator', 'change_target_directory'],
        },
    };

    async analyzeError(error: Error | string, context?: ErrorContext): Promise<ErrorRecoverySuggestion> {
        const errorMessage = error instanceof Error ? error.message : error;
        const errorCategory = this.categorizeError(errorMessage);
        const severity = this.assessSeverity(errorMessage, context);
        const recoveryActions = await this.generateRecoveryActions(errorMessage, context);

        return {
            category: errorCategory,
            severity: severity,
            primaryAction: recoveryActions.primary,
            alternativeActions: recoveryActions.alternatives,
            explanation: this.explainError(errorCategory, severity, errorMessage),
            preventions: this.suggestPreventions(errorCategory),
            relatedResources: this.getRelatedResources(errorCategory),
        };
    }

    private categorizeError(errorMessage: string): ErrorCategory {
        const message = errorMessage.toLowerCase();

        if (message.includes('network') || message.includes('timeout') || message.includes('connection')) {
            return ErrorCategory.NETWORK;
        }
        if (message.includes('syntax') || message.includes('invalid') || message.includes('parsing')) {
            return ErrorCategory.VALIDATION;
        }
        if (message.includes('authentication') || message.includes('unauthorized')) {
            return ErrorCategory.AUTHENTICATION;
        }
        if (message.includes('out of memory') || message.includes('resource') || message.includes('busy')) {
            return ErrorCategory.RESOURCE;
        }
        if (message.includes('compilation') || message.includes('compile') || message.includes('build')) {
            return ErrorCategory.COMPILATION;
        }
        if (message.includes('file not found') || message.includes('unable to read') || message.includes('i/o')) {
            return ErrorCategory.RESOURCE;
        }
        if (message.includes('dependency') || message.includes('import') || message.includes('module')) {
            return ErrorCategory.DEPENDENCY;
        }
        if (message.includes('permission') || message.includes('access denied')) {
            return ErrorCategory.PERMISSION;
        }

        return ErrorCategory.RUNTIME;
    }

    private assessSeverity(errorMessage: string, context?: ErrorContext): ErrorSeverity {
        // Critical errors that should block operations
        if (errorMessage.includes('security') || errorMessage.includes('corruption') ||
            errorMessage.includes('catastrophic') || errorMessage.includes('dangerous')) {
            return ErrorSeverity.CRITICAL;
        }

        // High severity for compilation and syntax errors
        if (errorMessage.includes('compilation') || errorMessage.includes('syntax') ||
            errorMessage.includes('parse') || errorMessage.includes('semantic')) {
            return ErrorSeverity.HIGH;
        }

        // Medium for recoverable errors
        if (errorMessage.includes('network') || errorMessage.includes('timeout') ||
            errorMessage.includes('busy') || errorMessage.includes('temporary')) {
            return ErrorSeverity.MEDIUM;
        }

        return ErrorSeverity.LOW;
    }

    private async generateRecoveryActions(errorMessage: string, context?: ErrorContext): Promise<{
        primary: RecoveryAction,
        alternatives: RecoveryAction[]
    }> {
        const pattern = this.matchErrorPattern(errorMessage);

        if (pattern) {
            return this.getActionFromPattern(pattern, context);
        }

        // Fallback: try context-aware recovery
        return this.contextAwareRecovery(errorMessage, context);
    }

    private matchErrorPattern(errorMessage: string): string | null {
        const message = errorMessage.toLowerCase();

        for (const [pattern, _] of Object.entries(ErrorRecovery.ERROR_PATTERNS)) {
            if (message.includes(pattern.split('_').join(' '))) {
                return pattern;
            }
        }

        return null;
    }

    private async getActionFromPattern(pattern: string, context?: ErrorContext): Promise<{
        primary: RecoveryAction,
        alternatives: RecoveryAction[]
    }> {
        const patternData = ErrorRecovery.ERROR_PATTERNS[pattern as keyof typeof ErrorRecovery.ERROR_PATTERNS];

        const primary = await this.createRecoveryAction(patternData.primaryRecovery);
        const alternatives = [];

        for (const alt of patternData.alternatives) {
            alternatives.push(await this.createRecoveryAction(alt));
        }

        return { primary, alternatives };
    }

    private async createRecoveryAction(actionType: string): Promise<RecoveryAction> {
        const actionBase = actionType.replace('_', ' ');

        switch (actionType) {
            case 'retry_with_exponential_backoff':
                return {
                    id: actionType,
                    title: 'Retry with backoff',
                    description: 'Automatically retry the operation with increasing delays',
                    actionType: 'retry',
                    requiresConfirmation: false,
                    estimatedResolution: 30,
                    confidence: 0.8,
                };

            case 'check_network_connectivity':
                return {
                    id: actionType,
                    title: 'Check network connection',
                    description: 'Verify your internet connection and proxy settings',
                    actionType: 'manual',
                    requiresConfirmation: false,
                    estimatedResolution: 60,
                    confidence: 0.6,
                };

            case 'switch_to_offline_mode':
                return {
                    id: actionType,
                    title: 'Switch to offline mode',
                    description: 'Continue with local AI processing without cloud connectivity',
                    actionType: 'alternative',
                    requiresConfirmation: true,
                    estimatedResolution: 5,
                    confidence: 0.9,
                };

            case 'syntax_suggestion':
                return {
                    id: actionType,
                    title: 'Get syntax suggestions',
                    description: 'Request AI-powered suggestions for syntax correction',
                    actionType: 'alternative',
                    requiresConfirmation: false,
                    estimatedResolution: 10,
                    confidence: 0.7,
                };

            case 'manual_fix':
                return {
                    id: actionType,
                    title: 'Manual correction',
                    description: 'Require manual code correction before retry',
                    actionType: 'manual',
                    requiresConfirmation: false,
                    estimatedResolution: 300,
                    confidence: 0.4,
                };

            case 'revert_last_change':
                return {
                    id: actionType,
                    title: 'Revert last change',
                    description: 'Undo the most recent modification causing the error',
                    actionType: 'alternative',
                    requiresConfirmation: true,
                    estimatedResolution: 5,
                    confidence: 0.95,
                };

            case 'retry_later':
                return {
                    id: actionType,
                    title: 'Retry in 1 minute',
                    description: 'Wait for system resources to become available',
                    actionType: 'retry',
                    requiresConfirmation: false,
                    estimatedResolution: 60,
                    confidence: 0.7,
                };

            case 'switch_ai_model':
                return {
                    id: actionType,
                    title: 'Switch AI model',
                    description: 'Use an alternative AI model for the operation',
                    actionType: 'alternative',
                    requiresConfirmation: false,
                    estimatedResolution: 10,
                    confidence: 0.8,
                };

            case 'batch_operation_delay':
                return {
                    id: actionType,
                    title: 'Delay batch operation',
                    description: 'Temporarily pause batch operations to reduce system load',
                    actionType: 'alternative',
                    requiresConfirmation: false,
                    estimatedResolution: 15,
                    confidence: 0.9,
                };

            case 'fix_compilation_issues':
                return {
                    id: actionType,
                    title: 'Fix compilation errors',
                    description: 'Resolve compilation errors and try again',
                    actionType: 'manual',
                    requiresConfirmation: false,
                    estimatedResolution: 600,
                    confidence: 0.5,
                };

            case 'rollback_changes':
                return {
                    id: actionType,
                    title: 'Rollback changes',
                    description: 'Undo recent changes that caused compilation failures',
                    actionType: 'alternative',
                    requiresConfirmation: true,
                    estimatedResolution: 30,
                    confidence: 0.85,
                };

            case 'syntax_assistance':
                return {
                    id: actionType,
                    title: 'Syntax assistance',
                    description: 'Get AI help with syntax corrections',
                    actionType: 'alternative',
                    requiresConfirmation: false,
                    estimatedResolution: 20,
                    confidence: 0.75,
                };

            default:
                return {
                    id: actionType,
                    title: 'General retry',
                    description: 'Attempt the operation once more',
                    actionType: 'retry',
                    requiresConfirmation: false,
                    estimatedResolution: 10,
                    confidence: 0.6,
                };
        }
    }

    private async contextAwareRecovery(errorMessage: string, context?: ErrorContext): Promise<{
        primary: RecoveryAction,
        alternatives: RecoveryAction[]
    }> {
        // Context-based recovery strategy
        let primaryAction: RecoveryAction;
        const alternatives: RecoveryAction[] = [];

        // If we have a file context, add file-specific recovery options
        if (context?.filePath) {
            primaryAction = await this.createRecoveryAction('revert_last_change');
            alternatives.push(await this.createRecoveryAction('syntax_assistance'));
            alternatives.push(await this.createRecoveryAction('manual_fix'));
        } else {
            primaryAction = await this.createRecoveryAction('retry_with_exponential_backoff');
        }

        // Add system-specific alternatives based on error category
        const category = this.categorizeError(errorMessage);
        switch (category) {
            case ErrorCategory.NETWORK:
                alternatives.push(await this.createRecoveryAction('check_network_connectivity'));
                alternatives.push(await this.createRecoveryAction('switch_to_offline_mode'));
                break;
            case ErrorCategory.RESOURCE:
                alternatives.push(await this.createRecoveryAction('retry_later'));
                alternatives.push(await this.createRecoveryAction('switch_ai_model'));
                break;
            case ErrorCategory.PERMISSION:
                // Secure alternatives for permission errors
                break;
        }

        return { primary: primaryAction, alternatives };
    }

    private explainError(category: ErrorCategory, severity: ErrorSeverity, errorMessage: string): string {
        const categoryDescriptions = {
            [ErrorCategory.NETWORK]: 'A network-related issue prevented the operation from completing successfully.',
            [ErrorCategory.VALIDATION]: 'The provided input or code structure contains validation errors.',
            [ErrorCategory.AUTHENTICATION]: 'Authentication or authorization failed.',
            [ErrorCategory.RESOURCE]: 'System resources are insufficient or unavailable.',
            [ErrorCategory.COMPILATION]: 'The code failed to compile due to syntax or semantic errors.',
            [ErrorCategory.RUNTIME]: 'An unexpected runtime error occurred during execution.',
            [ErrorCategory.CONFIGURATION]: 'Configuration settings are incorrect or missing.',
            [ErrorCategory.DEPENDENCY]: 'Required dependencies are unavailable or incompatible.',
            [ErrorCategory.PERMISSION]: 'The operation lacks necessary permissions.',
            [ErrorCategory.SYSTEM]: 'A system-level error occurred.',
        };

        const severityDescriptions = {
            [ErrorSeverity.LOW]: 'This is a minor issue that may have minimal impact.',
            [ErrorSeverity.MEDIUM]: 'This issue may affect operation performance or reliability.',
            [ErrorSeverity.HIGH]: 'This issue significantly impacts the operation and requires attention.',
            [ErrorSeverity.CRITICAL]: 'This is a critical error that may require immediate intervention.',
        };

        return `${categoryDescriptions[category]} ${severityDescriptions[severity]}

Original error: "${errorMessage}"

This issue has been automatically analyzed and recovery options have been generated.`;
    }

    private suggestPreventions(category: ErrorCategory): string[] {
        const preventions = {
            [ErrorCategory.NETWORK]: [
                'Ensure stable internet connection before operations',
                'Configure proxy settings correctly in the IDE',
                'Use offline mode for network-dependent operations',
                'Enable automatic retry mechanisms',
            ],
            [ErrorCategory.VALIDATION]: [
                'Validate input data before processing',
                'Use IDE syntax highlighting and error detection',
                'Implement input sanitization',
                'Test operations with sample data first',
            ],
            [ErrorCategory.AUTHENTICATION]: [
                'Verify API keys and credentials regularly',
                'Use secure credential storage',
                'Implement token refresh mechanisms',
                'Monitor authentication expiration dates',
            ],
            [ErrorCategory.RESOURCE]: [
                'Monitor system resource usage',
                'Implement resource pooling and connection reuse',
                'Cancel background operations when not needed',
                'Use streaming for large data operations',
            ],
            [ErrorCategory.COMPILATION]: [
                'Regularly update language tools and dependencies',
                'Use incremental compilation when possible',
                'Implement pre-compilation checks',
                'Keep dependencies up-to-date',
            ],
            [ErrorCategory.PERMISSION]: [
                'Review and update permission settings regularly',
                'Use least-privilege access principles',
                'Document required permissions for operations',
                'Implement permission request flows',
            ],
            [ErrorCategory.DEPENDENCY]: [
                'Audit dependencies regularly for security issues',
                'Keep dependency versions synchronized',
                'Implement dependency resolution caching',
                'Use dependency management tools effectively',
            ],
        };

        return preventions[category] || ['Review system configuration', 'Ensure dependencies are available', 'Monitor system health'];
    }

    private getRelatedResources(category: ErrorCategory): string[] {
        const resources = {
            [ErrorCategory.NETWORK]: [
                'Network troubleshooting guide',
                'Proxy configuration documentation',
                'Offline mode user manual',
                'Firewall configuration guide',
            ],
            [ErrorCategory.VALIDATION]: [
                'Data validation best practices',
                'Input sanitization guide',
                'Syntax reference manual',
                'Error handling patterns',
            ],
            [ErrorCategory.COMPILATION]: [
                'Language compiler documentation',
                'Build system configuration',
                'Dependency management guide',
                'Debugging compilation errors',
            ],
            [ErrorCategory.PERMISSION]: [
                'Security configuration guide',
                'Permission management document',
                'System administration manual',
                'Access control best practices',
            ],
        };

        return resources[category] || ['General troubleshooting guide', 'System documentation', 'User manual'];
    }

    async executeRecoveryAction(action: RecoveryAction, context?: ErrorContext): Promise<any> {
        try {
            await invoke('execute_recovery_action', {
                action: action.id,
                context: context,
            });
        } catch (recoveryError) {
            console.error('Recovery execution failed:', recoveryError);
            throw recoveryError;
        }
    }

    render() {
        return null; // This is a utility class
    }
}

export default ErrorRecovery;
export type { RecoveryAction, ErrorContext, ErrorRecoverySuggestion };
export { ErrorCategory, ErrorSeverity };