import React, { useState, useEffect, useCallback } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs';
import {
  Switch,
  Slider,
  Button,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
  Alert,
  AlertDescription,
} from '@/components/ui';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { invoke } from '@tauri-apps/api/tauri';
import {
  Settings,
  Brain,
  Shield,
  Zap,
  Code,
  Building,
  Clock,
  FileText,
  Eye,
  AlertTriangle,
  Info,
  CheckCircle,
  XCircle,
} from 'lucide-react';
import type {
  AnalysisConfiguration,
  AIProvider,
  AnalysisPreferences,
  LearningPreferences,
  SeverityLevel,
  AnalysisCategory,
} from '../types';

interface AIConfigurationPanelProps {
  isOpen: boolean;
  onClose: () => void;
  currentConfig?: AnalysisConfiguration;
  onConfigChange?: (config: AnalysisConfiguration) => void;
}

interface ValidationError {
  field: string;
  message: string;
}

const DEFAULT_CONFIG: AnalysisConfiguration = {
  enabledCategories: ['code-smell', 'performance', 'security'],
  severityThreshold: 'warning',
  realTimeAnalysis: true,
  analysisOnSave: true,
  maxSuggestions: 50,
  aiProvider: {
    type: 'mock',
  },
  analysisPreferences: {
    enableCodeSmells: true,
    enableSecurity: true,
    enablePerformance: true,
    enableCodeStyle: false,
    enableArchitecture: false,
    enableLearning: false,
    confidenceThreshold: 0.7,
    timeoutSeconds: 30,
    includeExplanations: true,
    includeExamples: true,
    privacyMode: 'opt-out',
  },
  learningPreferences: {
    enableLearning: false,
    privacyMode: 'opt-out',
    shareAnonymousData: false,
    retainPersonalData: false,
    dataRetentionDays: 30,
    allowModelTraining: false,
  },
  confidenceThreshold: 0.7,
  excludePatterns: ['target/', 'node_modules/', '*.test.rs'],
  maxFileSizeKb: 1024,
  timeoutSeconds: 30,
  customRules: [],
  performance: {
    enableBenchmarking: false,
    profileMemoryUsage: false,
    detectHotPaths: true,
    enablePerformanceHints: true,
    enableOptimizationSuggestions: true,
  },
  security: {
    enableVulnerabilityScanning: true,
    checkDependencies: true,
    scanForSecrets: true,
    enableSecurityIssueDetection: true,
    includeCweMapping: true,
  },
  architecture: {
    enablePatternDetection: false,
    checkCircularDependencies: true,
    analyzeCoupling: false,
    enableArchitectureSuggestions: false,
    detectAntiPatterns: true,
  },
  codeStyle: {
    enableStyleViolationDetection: false,
    enforceNamingConventions: true,
    checkFormattingConsistency: true,
    enforceRustIdioms: true,
    requireDocumentation: false,
  },
  learning: {
    enableLearningSystem: false,
    recordSuccessfulFixes: false,
    useLearnedPatterns: false,
    shareAnonymousData: false,
    confidenceThresholdForLearning: 0.8,
  },
  compiler: {
    enableCompilerIntegration: true,
    parseCargoCheckOutput: true,
    enableErrorExplanations: true,
    enableSuggestedFixes: true,
    cacheExplanations: true,
  },
};

export const AIConfigurationPanel: React.FC<AIConfigurationPanelProps> = ({
  isOpen,
  onClose,
  currentConfig,
  onConfigChange,
}) => {
  const [config, setConfig] = useState<AnalysisConfiguration>(
    currentConfig || DEFAULT_CONFIG
  );
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);
  const [activeTab, setActiveTab] = useState('analysis');
  const [testConnectionStatus, setTestConnectionStatus] = useState<
    'idle' | 'testing' | 'success' | 'error'
  >('idle');

  // Load configuration on mount
  useEffect(() => {
    if (isOpen && !currentConfig) {
      loadConfiguration();
    }
  }, [isOpen, currentConfig]);

  const loadConfiguration = async () => {
    setIsLoading(true);
    try {
      const savedConfig = await invoke<AnalysisConfiguration>('get_ai_configuration');
      setConfig(savedConfig);
    } catch (error) {
      console.error('Failed to load configuration:', error);
      // Use default config if loading fails
      setConfig(DEFAULT_CONFIG);
    } finally {
      setIsLoading(false);
    }
  };

  const validateConfiguration = useCallback((config: AnalysisConfiguration): ValidationError[] => {
    const errors: ValidationError[] = [];

    // Validate AI provider configuration
    if (config.aiProvider.type === 'openai' && !config.aiProvider.openai?.apiKey) {
      errors.push({ field: 'openai.apiKey', message: 'OpenAI API key is required' });
    }
    if (config.aiProvider.type === 'anthropic' && !config.aiProvider.anthropic?.apiKey) {
      errors.push({ field: 'anthropic.apiKey', message: 'Anthropic API key is required' });
    }
    if (config.aiProvider.type === 'local' && !config.aiProvider.local?.modelPath) {
      errors.push({ field: 'local.modelPath', message: 'Local model path is required' });
    }

    // Validate thresholds
    if (config.confidenceThreshold < 0 || config.confidenceThreshold > 1) {
      errors.push({ field: 'confidenceThreshold', message: 'Confidence threshold must be between 0 and 1' });
    }

    // Validate timeouts
    if (config.timeoutSeconds < 1 || config.timeoutSeconds > 300) {
      errors.push({ field: 'timeoutSeconds', message: 'Timeout must be between 1 and 300 seconds' });
    }

    // Validate file size limits
    if (config.maxFileSizeKb < 1 || config.maxFileSizeKb > 10240) {
      errors.push({ field: 'maxFileSizeKb', message: 'File size limit must be between 1KB and 10MB' });
    }

    return errors;
  }, []);

  const handleSave = async () => {
    const errors = validateConfiguration(config);
    setValidationErrors(errors);

    if (errors.length > 0) {
      return;
    }

    setIsSaving(true);
    try {
      await invoke('update_ai_configuration', { config });
      onConfigChange?.(config);
      onClose();
    } catch (error) {
      console.error('Failed to save configuration:', error);
      setValidationErrors([{ field: 'general', message: 'Failed to save configuration' }]);
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    setTestConnectionStatus('testing');
    try {
      await invoke('test_ai_provider_connection', { provider: config.aiProvider });
      setTestConnectionStatus('success');
      setTimeout(() => setTestConnectionStatus('idle'), 3000);
    } catch (error) {
      console.error('Connection test failed:', error);
      setTestConnectionStatus('error');
      setTimeout(() => setTestConnectionStatus('idle'), 3000);
    }
  };

  const updateConfig = (updates: Partial<AnalysisConfiguration>) => {
    setConfig(prev => ({ ...prev, ...updates }));
  };

  const updateNestedConfig = <T extends keyof AnalysisConfiguration>(
    section: T,
    updates: Partial<AnalysisConfiguration[T]>
  ) => {
    setConfig(prev => ({
      ...prev,
      [section]: { ...prev[section], ...updates }
    }));
  };

  const getValidationError = (field: string) => {
    return validationErrors.find(error => error.field === field)?.message;
  };

  const renderAnalysisToggles = () => (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Brain className="h-5 w-5" />
          Analysis Types
        </CardTitle>
        <CardDescription>
          Enable or disable specific types of code analysis
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">Code Smells</Label>
              <p className="text-xs text-muted-foreground">
                Detect code quality issues and anti-patterns
              </p>
            </div>
            <Switch
              checked={config.analysisPreferences.enableCodeSmells}
              onCheckedChange={(checked) =>
                updateNestedConfig('analysisPreferences', { enableCodeSmells: checked })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">Performance</Label>
              <p className="text-xs text-muted-foreground">
                Identify performance bottlenecks and optimization opportunities
              </p>
            </div>
            <Switch
              checked={config.analysisPreferences.enablePerformance}
              onCheckedChange={(checked) =>
                updateNestedConfig('analysisPreferences', { enablePerformance: checked })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">Security</Label>
              <p className="text-xs text-muted-foreground">
                Scan for security vulnerabilities and unsafe patterns
              </p>
            </div>
            <Switch
              checked={config.analysisPreferences.enableSecurity}
              onCheckedChange={(checked) =>
                updateNestedConfig('analysisPreferences', { enableSecurity: checked })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">Code Style</Label>
              <p className="text-xs text-muted-foreground">
                Check for style violations and formatting issues
              </p>
            </div>
            <Switch
              checked={config.analysisPreferences.enableCodeStyle}
              onCheckedChange={(checked) =>
                updateNestedConfig('analysisPreferences', { enableCodeStyle: checked })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">Architecture</Label>
              <p className="text-xs text-muted-foreground">
                Analyze architectural patterns and dependencies
              </p>
            </div>
            <Switch
              checked={config.analysisPreferences.enableArchitecture}
              onCheckedChange={(checked) =>
                updateNestedConfig('analysisPreferences', { enableArchitecture: checked })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">Real-time Analysis</Label>
              <p className="text-xs text-muted-foreground">
                Analyze code as you type
              </p>
            </div>
            <Switch
              checked={config.realTimeAnalysis}
              onCheckedChange={(checked) =>
                updateConfig({ realTimeAnalysis: checked })
              }
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );

  const renderConfidenceThresholds = () => (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Zap className="h-5 w-5" />
          Confidence Thresholds
        </CardTitle>
        <CardDescription>
          Set minimum confidence levels for different analysis types
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-2">
          <Label className="text-sm font-medium">
            Overall Confidence Threshold: {Math.round(config.confidenceThreshold * 100)}%
          </Label>
          <Slider
            value={[config.confidenceThreshold]}
            onValueChange={([value]) => updateConfig({ confidenceThreshold: value })}
            min={0}
            max={1}
            step={0.05}
            className="w-full"
          />
          <p className="text-xs text-muted-foreground">
            Only show suggestions with confidence above this threshold
          </p>
        </div>

        <div className="space-y-2">
          <Label className="text-sm font-medium">
            Learning System Threshold: {Math.round(config.learning.confidenceThresholdForLearning * 100)}%
          </Label>
          <Slider
            value={[config.learning.confidenceThresholdForLearning]}
            onValueChange={([value]) =>
              updateNestedConfig('learning', { confidenceThresholdForLearning: value })
            }
            min={0}
            max={1}
            step={0.05}
            className="w-full"
          />
          <p className="text-xs text-muted-foreground">
            Minimum confidence for patterns to be learned
          </p>
        </div>
      </CardContent>
    </Card>
  );

  const renderProviderConfiguration = () => (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Settings className="h-5 w-5" />
          AI Provider Configuration
        </CardTitle>
        <CardDescription>
          Configure your AI analysis provider
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="provider-type">Provider Type</Label>
          <Select
            value={config.aiProvider.type}
            onValueChange={(value: 'openai' | 'anthropic' | 'local' | 'mock') =>
              updateConfig({
                aiProvider: { type: value }
              })
            }
          >
            <SelectTrigger>
              <SelectValue placeholder="Select AI provider" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="mock">Mock (Testing)</SelectItem>
              <SelectItem value="openai">OpenAI</SelectItem>
              <SelectItem value="anthropic">Anthropic</SelectItem>
              <SelectItem value="local">Local Model</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {config.aiProvider.type === 'openai' && (
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="openai-api-key">API Key</Label>
              <Input
                id="openai-api-key"
                type="password"
                placeholder="sk-..."
                value={config.aiProvider.openai?.apiKey || ''}
                onChange={(e) =>
                  updateConfig({
                    aiProvider: {
                      ...config.aiProvider,
                      openai: { ...config.aiProvider.openai, apiKey: e.target.value }
                    }
                  })
                }
              />
              {getValidationError('openai.apiKey') && (
                <p className="text-sm text-destructive">{getValidationError('openai.apiKey')}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="openai-model">Model</Label>
              <Select
                value={config.aiProvider.openai?.model || 'gpt-4'}
                onValueChange={(value) =>
                  updateConfig({
                    aiProvider: {
                      ...config.aiProvider,
                      openai: { ...config.aiProvider.openai, model: value }
                    }
                  })
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="gpt-4">GPT-4</SelectItem>
                  <SelectItem value="gpt-4-turbo">GPT-4 Turbo</SelectItem>
                  <SelectItem value="gpt-3.5-turbo">GPT-3.5 Turbo</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        )}

        {config.aiProvider.type === 'anthropic' && (
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="anthropic-api-key">API Key</Label>
              <Input
                id="anthropic-api-key"
                type="password"
                placeholder="sk-ant-..."
                value={config.aiProvider.anthropic?.apiKey || ''}
                onChange={(e) =>
                  updateConfig({
                    aiProvider: {
                      ...config.aiProvider,
                      anthropic: { ...config.aiProvider.anthropic, apiKey: e.target.value }
                    }
                  })
                }
              />
              {getValidationError('anthropic.apiKey') && (
                <p className="text-sm text-destructive">{getValidationError('anthropic.apiKey')}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="anthropic-model">Model</Label>
              <Select
                value={config.aiProvider.anthropic?.model || 'claude-3-opus-20240229'}
                onValueChange={(value) =>
                  updateConfig({
                    aiProvider: {
                      ...config.aiProvider,
                      anthropic: { ...config.aiProvider.anthropic, model: value }
                    }
                  })
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="claude-3-opus-20240229">Claude 3 Opus</SelectItem>
                  <SelectItem value="claude-3-sonnet-20240229">Claude 3 Sonnet</SelectItem>
                  <SelectItem value="claude-3-haiku-20240307">Claude 3 Haiku</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        )}

        {config.aiProvider.type === 'local' && (
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="local-model-path">Model Path</Label>
              <Input
                id="local-model-path"
                placeholder="/path/to/model"
                value={config.aiProvider.local?.modelPath || ''}
                onChange={(e) =>
                  updateConfig({
                    aiProvider: {
                      ...config.aiProvider,
                      local: { ...config.aiProvider.local, modelPath: e.target.value }
                    }
                  })
                }
              />
              {getValidationError('local.modelPath') && (
                <p className="text-sm text-destructive">{getValidationError('local.modelPath')}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="local-endpoint">Endpoint (Optional)</Label>
              <Input
                id="local-endpoint"
                placeholder="http://localhost:8080"
                value={config.aiProvider.local?.endpoint || ''}
                onChange={(e) =>
                  updateConfig({
                    aiProvider: {
                      ...config.aiProvider,
                      local: { ...config.aiProvider.local, endpoint: e.target.value }
                    }
                  })
                }
              />
            </div>
          </div>
        )}

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleTestConnection}
            disabled={testConnectionStatus === 'testing' || config.aiProvider.type === 'mock'}
          >
            {testConnectionStatus === 'testing' ? 'Testing...' : 'Test Connection'}
          </Button>
          {testConnectionStatus === 'success' && (
            <Badge variant="default" className="bg-green-500">
              <CheckCircle className="h-3 w-3 mr-1" />
              Connected
            </Badge>
          )}
          {testConnectionStatus === 'error' && (
            <Badge variant="destructive">
              <XCircle className="h-3 w-3 mr-1" />
              Failed
            </Badge>
          )}
        </div>
      </CardContent>
    </Card>
  );

  const renderLearningPreferences = () => (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Brain className="h-5 w-5" />
          Learning System
        </CardTitle>
        <CardDescription>
          Configure AI learning and privacy preferences
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label className="text-sm font-medium">Enable Learning System</Label>
            <p className="text-xs text-muted-foreground">
              Allow the AI to learn from successful fixes
            </p>
          </div>
          <Switch
            checked={config.learningPreferences.enableLearning}
            onCheckedChange={(checked) =>
              updateNestedConfig('learningPreferences', { enableLearning: checked })
            }
          />
        </div>

        {config.learningPreferences.enableLearning && (
          <>
            <Separator />
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>Privacy Mode</Label>
                <Select
                  value={config.learningPreferences.privacyMode}
                  onValueChange={(value: 'opt-in' | 'opt-out' | 'anonymous') =>
                    updateNestedConfig('learningPreferences', { privacyMode: value })
                  }
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="opt-in">Opt-in (Ask before learning)</SelectItem>
                    <SelectItem value="opt-out">Opt-out (Learn by default)</SelectItem>
                    <SelectItem value="anonymous">Anonymous (No personal data)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label className="text-sm font-medium">Share Anonymous Data</Label>
                  <p className="text-xs text-muted-foreground">
                    Help improve the AI by sharing anonymized patterns
                  </p>
                </div>
                <Switch
                  checked={config.learningPreferences.shareAnonymousData}
                  onCheckedChange={(checked) =>
                    updateNestedConfig('learningPreferences', { shareAnonymousData: checked })
                  }
                />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label className="text-sm font-medium">Retain Personal Data</Label>
                  <p className="text-xs text-muted-foreground">
                    Store learning data locally for personalized suggestions
                  </p>
                </div>
                <Switch
                  checked={config.learningPreferences.retainPersonalData}
                  onCheckedChange={(checked) =>
                    updateNestedConfig('learningPreferences', { retainPersonalData: checked })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="data-retention">Data Retention (days)</Label>
                <Input
                  id="data-retention"
                  type="number"
                  min="1"
                  max="365"
                  value={config.learningPreferences.dataRetentionDays}
                  onChange={(e) =>
                    updateNestedConfig('learningPreferences', {
                      dataRetentionDays: parseInt(e.target.value) || 30
                    })
                  }
                />
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );

  const renderPerformanceSettings = () => (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Performance Settings
        </CardTitle>
        <CardDescription>
          Configure analysis performance and resource limits
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label htmlFor="timeout">Analysis Timeout (seconds)</Label>
            <Input
              id="timeout"
              type="number"
              min="1"
              max="300"
              value={config.timeoutSeconds}
              onChange={(e) =>
                updateConfig({ timeoutSeconds: parseInt(e.target.value) || 30 })
              }
            />
            {getValidationError('timeoutSeconds') && (
              <p className="text-sm text-destructive">{getValidationError('timeoutSeconds')}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="max-file-size">Max File Size (KB)</Label>
            <Input
              id="max-file-size"
              type="number"
              min="1"
              max="10240"
              value={config.maxFileSizeKb}
              onChange={(e) =>
                updateConfig({ maxFileSizeKb: parseInt(e.target.value) || 1024 })
              }
            />
            {getValidationError('maxFileSizeKb') && (
              <p className="text-sm text-destructive">{getValidationError('maxFileSizeKb')}</p>
            )}
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="max-suggestions">Maximum Suggestions</Label>
          <Input
            id="max-suggestions"
            type="number"
            min="1"
            max="200"
            value={config.maxSuggestions}
            onChange={(e) =>
              updateConfig({ maxSuggestions: parseInt(e.target.value) || 50 })
            }
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="exclude-patterns">Exclude Patterns</Label>
          <Textarea
            id="exclude-patterns"
            placeholder="target/&#10;node_modules/&#10;*.test.rs"
            value={config.excludePatterns.join('\n')}
            onChange={(e) =>
              updateConfig({
                excludePatterns: e.target.value.split('\n').filter(p => p.trim())
              })
            }
            rows={4}
          />
          <p className="text-xs text-muted-foreground">
            One pattern per line. Supports glob patterns.
          </p>
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label className="text-sm font-medium">Analysis on Save</Label>
            <p className="text-xs text-muted-foreground">
              Automatically analyze files when saved
            </p>
          </div>
          <Switch
            checked={config.analysisOnSave}
            onCheckedChange={(checked) =>
              updateConfig({ analysisOnSave: checked })
            }
          />
        </div>
      </CardContent>
    </Card>
  );

  if (isLoading) {
    return (
      <Dialog open={isOpen} onOpenChange={onClose}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
          <div className="flex items-center justify-center p-8">
            <div className="text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
              <p>Loading configuration...</p>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            AI Analysis Configuration
          </DialogTitle>
        </DialogHeader>

        {validationErrors.length > 0 && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Please fix the following errors:
              <ul className="mt-2 list-disc list-inside">
                {validationErrors.map((error, index) => (
                  <li key={index}>{error.message}</li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="analysis">Analysis</TabsTrigger>
            <TabsTrigger value="provider">Provider</TabsTrigger>
            <TabsTrigger value="learning">Learning</TabsTrigger>
            <TabsTrigger value="performance">Performance</TabsTrigger>
          </TabsList>

          <TabsContent value="analysis" className="space-y-4">
            {renderAnalysisToggles()}
            {renderConfidenceThresholds()}
          </TabsContent>

          <TabsContent value="provider" className="space-y-4">
            {renderProviderConfiguration()}
          </TabsContent>

          <TabsContent value="learning" className="space-y-4">
            {renderLearningPreferences()}
          </TabsContent>

          <TabsContent value="performance" className="space-y-4">
            {renderPerformanceSettings()}
          </TabsContent>
        </Tabs>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isSaving}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={isSaving}>
            {isSaving ? 'Saving...' : 'Save Configuration'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default AIConfigurationPanel;