import {
  Architecture,
  BugReport,
  CheckCircle,
  Error as ErrorIcon,
  Info,
  PlayArrow,
  Refresh,
  Security,
  Settings,
  Speed,
  Style,
  TrendingUp,
  Warning,
} from '@mui/icons-material';
import {
  Badge,
  Box,
  Chip,
  CircularProgress,
  Divider,
  FormControlLabel,
  IconButton,
  LinearProgress,
  Menu,
  MenuItem,
  Switch,
  Tooltip,
  Typography,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useMemo, useState } from 'react';
import { useAIAssistant } from '../../features/ai/hooks/useAIAssistant';
import type { AnalysisCategory, SeverityLevel } from '../../features/ai/types';
import { useAppSelector } from '../../store';
import { selectCargoState } from '../../store/slices/cargoSlice';

interface AIStatusSummary {
  totalIssues: number;
  criticalCount: number;
  errorCount: number;
  warningCount: number;
  infoCount: number;
  isAnalyzing: boolean;
  analysisProgress?: number;
  lastAnalysisTime?: number;
  aiProviderStatus: 'connected' | 'disconnected' | 'error';
  enabledCategories: AnalysisCategory[];
}

export function StatusBar() {
  const cargo = useAppSelector(selectCargoState);
  const {
    isAnalyzing,
    analysisResults,
    workspaceProgress,
    config,
    analyzeWorkspace,
    runCodeQualityCheck,
    updateConfiguration,
    getAnalysisStats,
  } = useAIAssistant();

  const [settingsMenuAnchor, setSettingsMenuAnchor] = useState<null | HTMLElement>(null);
  const [isRunningWorkspaceAnalysis, setIsRunningWorkspaceAnalysis] = useState(false);

  // Calculate AI analysis summary
  const aiSummary = useMemo((): AIStatusSummary => {
    let totalIssues = 0;
    let criticalCount = 0;
    let errorCount = 0;
    let warningCount = 0;
    let infoCount = 0;
    let lastAnalysisTime = 0;

    // Aggregate results from all analyzed files
    for (const result of analysisResults.values()) {
      totalIssues += result.summary?.totalIssues || 0;

      // Count by severity from all suggestion types
      const allSuggestions = [
        ...result.codeSmells,
        ...result.style,
        ...(result.security?.vulnerabilities || []),
        ...(result.architecture?.patterns || []),
      ];

      allSuggestions.forEach((suggestion) => {
        switch (suggestion.severity || suggestion.severityLevel) {
          case 'critical':
            criticalCount++;
            break;
          case 'error':
            errorCount++;
            break;
          case 'warning':
            warningCount++;
            break;
          case 'info':
            infoCount++;
            break;
        }
      });

      if (result.summary?.timestamp && result.summary.timestamp > lastAnalysisTime) {
        lastAnalysisTime = result.summary.timestamp;
      }
    }

    return {
      totalIssues,
      criticalCount,
      errorCount,
      warningCount,
      infoCount,
      isAnalyzing,
      analysisProgress: workspaceProgress?.progress,
      lastAnalysisTime: lastAnalysisTime || undefined,
      aiProviderStatus: config ? 'connected' : 'disconnected',
      enabledCategories: config?.enabledCategories || [],
    };
  }, [analysisResults, isAnalyzing, workspaceProgress, config]);

  // Calculate Cargo status (existing logic)
  const { overallStatus, summary } = useMemo(() => {
    const commands = Object.values(cargo.commands || {});
    const running = commands.filter((c) => c.status === 'running');
    const errored = commands.filter((c) => c.status === 'error');
    const success = commands.filter((c) => c.status === 'success');

    const overallStatus = running.length
      ? 'running'
      : errored.length
        ? 'error'
        : success.length
          ? 'success'
          : 'idle';

    const last = commands.sort((a, b) => b.timestamp - a.timestamp)[0];
    const lastLabel = last
      ? `${last.command}${last.args?.length ? ' ' + last.args.join(' ') : ''}`
      : 'no recent command';

    const summary = {
      running: running.length,
      errored: errored.length,
      success: success.length,
      lastLabel,
    };

    return { overallStatus, summary } as const;
  }, [cargo.commands]);

  const cargoColor: 'default' | 'success' | 'error' | 'warning' =
    overallStatus === 'running'
      ? 'warning'
      : overallStatus === 'success'
        ? 'success'
        : overallStatus === 'error'
          ? 'error'
          : 'default';

  const getAIStatusColor = () => {
    if (aiSummary.isAnalyzing) return 'warning';
    if (aiSummary.criticalCount > 0 || aiSummary.errorCount > 0) return 'error';
    if (aiSummary.warningCount > 0) return 'warning';
    if (aiSummary.totalIssues === 0 && aiSummary.lastAnalysisTime) return 'success';
    return 'default';
  };

  const handleRunWorkspaceAnalysis = useCallback(async () => {
    try {
      setIsRunningWorkspaceAnalysis(true);
      await analyzeWorkspace('/workspace', {
        includeDependencies: true,
        includeSecurityScan: true,
      });
    } catch (error) {
      console.error('Failed to run workspace analysis:', error);
    } finally {
      setIsRunningWorkspaceAnalysis(false);
    }
  }, [analyzeWorkspace]);

  const handleRunCodeQualityCheck = useCallback(async () => {
    try {
      await runCodeQualityCheck('/workspace', {
        runClippy: true,
        runRustfmt: true,
        runAIAnalysis: true,
      });
    } catch (error) {
      console.error('Failed to run code quality check:', error);
    }
  }, [runCodeQualityCheck]);

  const handleToggleAnalysisCategory = useCallback(
    async (category: AnalysisCategory, enabled: boolean) => {
      if (!config) return;

      const newCategories = enabled
        ? [...config.enabledCategories, category]
        : config.enabledCategories.filter((c) => c !== category);

      try {
        await updateConfiguration({
          enabledCategories: newCategories,
        });
      } catch (error) {
        console.error('Failed to update analysis configuration:', error);
      }
    },
    [config, updateConfiguration]
  );

  const handleOpenAISettings = useCallback(async () => {
    try {
      await invoke('open_ai_settings');
    } catch (error) {
      console.error('Failed to open AI settings:', error);
    }
    setSettingsMenuAnchor(null);
  }, []);

  const getCategoryIcon = (category: AnalysisCategory) => {
    switch (category) {
      case 'code-smell':
        return <BugReport fontSize="small" />;
      case 'performance':
        return <Speed fontSize="small" />;
      case 'security':
        return <Security fontSize="small" />;
      case 'style':
        return <Style fontSize="small" />;
      case 'architecture':
        return <Architecture fontSize="small" />;
      default:
        return <Info fontSize="small" />;
    }
  };

  const getSeverityIcon = (severity: SeverityLevel) => {
    switch (severity) {
      case 'critical':
        return <ErrorIcon fontSize="small" color="error" />;
      case 'error':
        return <ErrorIcon fontSize="small" color="error" />;
      case 'warning':
        return <Warning fontSize="small" color="warning" />;
      case 'info':
        return <Info fontSize="small" color="info" />;
      default:
        return <CheckCircle fontSize="small" color="success" />;
    }
  };

  return (
    <Box
      sx={{
        px: 1.5,
        py: 0.5,
        display: 'flex',
        alignItems: 'center',
        gap: 1,
        borderTop: '1px solid rgba(255,255,255,0.08)',
        bgcolor: 'background.paper',
        minHeight: 32,
      }}
    >
      {/* Cargo Status (existing) */}
      <Tooltip title={`Last: ${summary.lastLabel}`} arrow>
        <Chip size="small" color={cargoColor} label={`Cargo: ${overallStatus}`} />
      </Tooltip>
      <Chip size="small" variant="outlined" label={`running: ${summary.running}`} />
      <Chip size="small" variant="outlined" label={`success: ${summary.success}`} />
      <Chip size="small" variant="outlined" label={`error: ${summary.errored}`} />

      <Divider orientation="vertical" flexItem sx={{ mx: 1 }} />

      {/* AI Analysis Status */}
      <Tooltip
        title={
          <Box>
            <Typography variant="body2" sx={{ fontWeight: 'bold', mb: 1 }}>
              AI Analysis Status
            </Typography>
            <Typography variant="body2">Provider: {aiSummary.aiProviderStatus}</Typography>
            <Typography variant="body2">Total Issues: {aiSummary.totalIssues}</Typography>
            {aiSummary.lastAnalysisTime && (
              <Typography variant="body2">
                Last Analysis: {new Date(aiSummary.lastAnalysisTime).toLocaleTimeString()}
              </Typography>
            )}
          </Box>
        }
        arrow
      >
        <Chip
          size="small"
          color={getAIStatusColor()}
          label={
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
              {aiSummary.isAnalyzing && <CircularProgress size={12} color="inherit" />}
              AI: {aiSummary.isAnalyzing ? 'analyzing' : aiSummary.aiProviderStatus}
            </Box>
          }
        />
      </Tooltip>

      {/* Issue Counters */}
      {aiSummary.criticalCount > 0 && (
        <Badge badgeContent={aiSummary.criticalCount} color="error">
          <Chip
            size="small"
            variant="outlined"
            color="error"
            icon={<ErrorIcon fontSize="small" />}
            label="Critical"
          />
        </Badge>
      )}

      {aiSummary.errorCount > 0 && (
        <Badge badgeContent={aiSummary.errorCount} color="error">
          <Chip
            size="small"
            variant="outlined"
            color="error"
            icon={<ErrorIcon fontSize="small" />}
            label="Errors"
          />
        </Badge>
      )}

      {aiSummary.warningCount > 0 && (
        <Badge badgeContent={aiSummary.warningCount} color="warning">
          <Chip
            size="small"
            variant="outlined"
            color="warning"
            icon={<Warning fontSize="small" />}
            label="Warnings"
          />
        </Badge>
      )}

      {aiSummary.infoCount > 0 && (
        <Badge badgeContent={aiSummary.infoCount} color="info">
          <Chip
            size="small"
            variant="outlined"
            color="info"
            icon={<Info fontSize="small" />}
            label="Info"
          />
        </Badge>
      )}

      {/* Analysis Progress */}
      {aiSummary.isAnalyzing && aiSummary.analysisProgress !== undefined && (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, minWidth: 100 }}>
          <LinearProgress
            variant="determinate"
            value={aiSummary.analysisProgress}
            sx={{ flex: 1, height: 4 }}
          />
          <Typography variant="caption" color="text.secondary">
            {Math.round(aiSummary.analysisProgress)}%
          </Typography>
        </Box>
      )}

      <Box sx={{ flexGrow: 1 }} />

      {/* Quick Action Buttons */}
      <Tooltip title="Run Workspace Analysis">
        <IconButton
          size="small"
          onClick={handleRunWorkspaceAnalysis}
          disabled={isRunningWorkspaceAnalysis || aiSummary.isAnalyzing}
        >
          {isRunningWorkspaceAnalysis ? (
            <CircularProgress size={16} />
          ) : (
            <PlayArrow fontSize="small" />
          )}
        </IconButton>
      </Tooltip>

      <Tooltip title="Run Code Quality Check">
        <IconButton
          size="small"
          onClick={handleRunCodeQualityCheck}
          disabled={aiSummary.isAnalyzing}
        >
          <TrendingUp fontSize="small" />
        </IconButton>
      </Tooltip>

      <Tooltip title="Refresh Analysis">
        <IconButton
          size="small"
          onClick={handleRunWorkspaceAnalysis}
          disabled={aiSummary.isAnalyzing}
        >
          <Refresh fontSize="small" />
        </IconButton>
      </Tooltip>

      {/* Settings Menu */}
      <Tooltip title="AI Analysis Settings">
        <IconButton size="small" onClick={(e) => setSettingsMenuAnchor(e.currentTarget)}>
          <Settings fontSize="small" />
        </IconButton>
      </Tooltip>

      <Menu
        anchorEl={settingsMenuAnchor}
        open={Boolean(settingsMenuAnchor)}
        onClose={() => setSettingsMenuAnchor(null)}
        PaperProps={{
          sx: { minWidth: 250 },
        }}
      >
        <MenuItem onClick={handleOpenAISettings}>
          <Settings fontSize="small" sx={{ mr: 1 }} />
          AI Provider Settings
        </MenuItem>

        <Divider />

        <Box sx={{ px: 2, py: 1 }}>
          <Typography variant="subtitle2" color="text.secondary" sx={{ mb: 1 }}>
            Analysis Categories
          </Typography>

          {(
            ['code-smell', 'performance', 'security', 'style', 'architecture'] as AnalysisCategory[]
          ).map((category) => (
            <FormControlLabel
              key={category}
              control={
                <Switch
                  size="small"
                  checked={aiSummary.enabledCategories.includes(category)}
                  onChange={(e) => handleToggleAnalysisCategory(category, e.target.checked)}
                />
              }
              label={
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                  {getCategoryIcon(category)}
                  {category.replace('-', ' ').replace(/\b\w/g, (l) => l.toUpperCase())}
                </Box>
              }
              sx={{ display: 'flex', width: '100%', m: 0, py: 0.5 }}
            />
          ))}
        </Box>
      </Menu>
    </Box>
  );
}
