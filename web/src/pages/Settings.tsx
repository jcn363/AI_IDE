import React, { useState, useCallback } from 'react';
import { useAppDispatch, useAppSelector } from '../store';
import {
  setTheme,
  setFontSize,
  setFontFamily,
  setWordWrap,
  setMinimap,
  setLineNumbers,
  setTabSize,
  EditorTheme,
} from '../store/slices/editorSlice';
import {
  Box,
  Typography,
  Slider,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  SelectChangeEvent,
  TextField,
  Switch,
  FormControlLabel,
  Divider,
  Paper,
  Button,
  Chip,
  Alert,
  IconButton,
  InputAdornment,
} from '@mui/material';
import { Visibility, VisibilityOff, Save, Refresh } from '@mui/icons-material';
// Removed Grid to avoid type issues; using responsive Box layout instead

const themes: { value: EditorTheme; label: string }[] = [
  { value: 'vs', label: 'Light' },
  { value: 'vs-dark', label: 'Dark' },
  { value: 'hc-black', label: 'High Contrast' },
];

const fontFamilies = [
  'Fira Code',
  'Consolas',
  'Monaco',
  'Source Code Pro',
  'Courier New',
  'monospace',
];

// AI Analysis Configuration Types
interface AIAnalysisConfig {
  enabled: boolean;
  provider: 'openai' | 'local' | 'ollama';
  apiKey: string;
  model: string;
  endpoint: string;
  analysisFrequency: 'realtime' | 'on-save' | 'manual';
  enabledCategories: {
    codeSmells: boolean;
    performance: boolean;
    security: boolean;
    style: boolean;
    architecture: boolean;
  };
  severityThresholds: {
    error: number;
    warning: number;
    info: number;
  };
  maxSuggestions: number;
  timeout: number;
}

const defaultAIConfig: AIAnalysisConfig = {
  enabled: false,
  provider: 'openai',
  apiKey: '',
  model: 'gpt-4',
  endpoint: 'https://api.openai.com/v1',
  analysisFrequency: 'on-save',
  enabledCategories: {
    codeSmells: true,
    performance: true,
    security: true,
    style: true,
    architecture: true,
  },
  severityThresholds: {
    error: 8,
    warning: 5,
    info: 2,
  },
  maxSuggestions: 50,
  timeout: 30000,
};

const aiProviders = [
  { value: 'openai', label: 'OpenAI' },
  { value: 'local', label: 'Local Model' },
  { value: 'ollama', label: 'Ollama' },
];

const openaiModels = ['gpt-4', 'gpt-4-turbo', 'gpt-3.5-turbo', 'gpt-4o', 'gpt-4o-mini'];

const analysisFrequencies = [
  { value: 'realtime', label: 'Real-time (as you type)' },
  { value: 'on-save', label: 'On file save' },
  { value: 'manual', label: 'Manual only' },
];

export function Settings() {
  const dispatch = useAppDispatch();
  const { theme, fontSize, fontFamily, wordWrap, minimap, lineNumbers, tabSize } = useAppSelector(
    (state) => state.editor
  );

  const [localFontSize, setLocalFontSize] = useState(fontSize);
  const [localTabSize, setLocalTabSize] = useState(tabSize);
  const [localFontFamily, setLocalFontFamily] = useState(fontFamily);

  // AI Analysis Configuration State
  const [aiConfig, setAiConfig] = useState<AIAnalysisConfig>(() => {
    const stored = localStorage.getItem('ai-analysis-config');
    return stored ? { ...defaultAIConfig, ...JSON.parse(stored) } : defaultAIConfig;
  });
  const [showApiKey, setShowApiKey] = useState(false);
  const [configSaved, setConfigSaved] = useState(false);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);

  const handleThemeChange = useCallback(
    (event: SelectChangeEvent) => {
      dispatch(setTheme(event.target.value as EditorTheme));
    },
    [dispatch]
  );

  const handleFontSizeChange = useCallback(
    (_: Event, value: number | number[]) => {
      const newSize = Array.isArray(value) ? value[0] : value;
      setLocalFontSize(newSize);
      dispatch(setFontSize(newSize));
    },
    [dispatch]
  );

  const handleFontFamilyChange = useCallback(
    (event: SelectChangeEvent<string>) => {
      const newFontFamily = event.target.value as string;
      setLocalFontFamily(newFontFamily);
      dispatch(setFontFamily(newFontFamily));
    },
    [dispatch]
  );

  const handleWordWrapToggle = useCallback(
    (_event: React.ChangeEvent<HTMLInputElement>, checked: boolean) => {
      dispatch(setWordWrap(checked));
    },
    [dispatch]
  );

  const handleMinimapToggle = useCallback(
    (_event: React.ChangeEvent<HTMLInputElement>, checked: boolean) => {
      dispatch(setMinimap(checked));
    },
    [dispatch]
  );

  const handleLineNumbersToggle = useCallback(
    (_event: React.ChangeEvent<HTMLInputElement>, checked: boolean) => {
      dispatch(setLineNumbers(checked));
    },
    [dispatch]
  );

  const handleTabSizeChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const newTabSize = parseInt((event.target as any).value, 10) || 2;
      setLocalTabSize(newTabSize);
      dispatch(setTabSize(newTabSize));
    },
    [dispatch]
  );

  // AI Configuration Handlers
  const validateAIConfig = useCallback((config: AIAnalysisConfig): string[] => {
    const errors: string[] = [];

    if (config.enabled) {
      if (config.provider === 'openai' && !config.apiKey.trim()) {
        errors.push('OpenAI API key is required when OpenAI provider is selected');
      }

      if (config.provider === 'openai' && config.apiKey.length < 32) {
        errors.push('OpenAI API key appears to be invalid (too short)');
      }

      if (!config.model.trim()) {
        errors.push('Model selection is required');
      }

      if (!config.endpoint.trim() || !config.endpoint.startsWith('http')) {
        errors.push('Valid endpoint URL is required');
      }

      if (config.timeout < 5000 || config.timeout > 300000) {
        errors.push('Timeout must be between 5 and 300 seconds');
      }

      if (config.maxSuggestions < 1 || config.maxSuggestions > 200) {
        errors.push('Max suggestions must be between 1 and 200');
      }
    }

    return errors;
  }, []);

  const saveAIConfig = useCallback(() => {
    const errors = validateAIConfig(aiConfig);
    setValidationErrors(errors);

    if (errors.length === 0) {
      // Store non-sensitive config in localStorage
      const configToStore = { ...aiConfig };
      if (aiConfig.provider === 'openai') {
        // Don't store API key in localStorage - this is a simplified example
        // In production, you'd want to use secure storage or send to backend
        configToStore.apiKey = '***STORED_SECURELY***';
      }

      localStorage.setItem('ai-analysis-config', JSON.stringify(configToStore));
      setConfigSaved(true);
      setTimeout(() => setConfigSaved(false), 3000);

      // Dispatch event to notify other components of config change
      (globalThis as any).dispatchEvent?.(
        new CustomEvent('ai-config-changed', { detail: aiConfig })
      );
    }
  }, [aiConfig, validateAIConfig]);

  const updateAIConfig = useCallback((updates: Partial<AIAnalysisConfig>) => {
    setAiConfig((prev) => ({ ...prev, ...updates }));
    setConfigSaved(false);
  }, []);

  const updateAICategory = useCallback(
    (category: keyof AIAnalysisConfig['enabledCategories'], enabled: boolean) => {
      updateAIConfig({
        enabledCategories: {
          ...aiConfig.enabledCategories,
          [category]: enabled,
        },
      });
    },
    [aiConfig.enabledCategories, updateAIConfig]
  );

  const updateSeverityThreshold = useCallback(
    (severity: keyof AIAnalysisConfig['severityThresholds'], value: number) => {
      updateAIConfig({
        severityThresholds: {
          ...aiConfig.severityThresholds,
          [severity]: value,
        },
      });
    },
    [aiConfig.severityThresholds, updateAIConfig]
  );

  const resetAIConfig = useCallback(() => {
    setAiConfig(defaultAIConfig);
    setValidationErrors([]);
    setConfigSaved(false);
  }, []);

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h5" gutterBottom>
        Editor Settings
      </Typography>

      <Paper sx={{ p: 3, mb: 4, maxWidth: 800 }}>
        <Typography variant="h6" gutterBottom>
          Appearance
        </Typography>

        <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', md: '1fr 1fr' }, gap: 3 }}>
          <Box>
            <FormControl fullWidth sx={{ mb: 3 }}>
              <InputLabel id="theme-select-label">Theme</InputLabel>
              <Select
                labelId="theme-select-label"
                id="theme-select"
                value={theme}
                label="Theme"
                onChange={handleThemeChange}
              >
                {themes.map((t) => (
                  <MenuItem key={t.value} value={t.value}>
                    {t.label}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <FormControl fullWidth sx={{ mb: 3 }}>
              <InputLabel id="font-family-select-label">Font Family</InputLabel>
              <Select
                labelId="font-family-select-label"
                id="font-family-select"
                value={localFontFamily}
                label="Font Family"
                onChange={handleFontFamilyChange}
              >
                {fontFamilies.map((font) => (
                  <MenuItem key={font} value={font} style={{ fontFamily: font }}>
                    {font}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Box>

          <Box>
            <Box sx={{ mb: 3 }}>
              <Typography id="font-size-slider" gutterBottom>
                Font Size: {localFontSize}px
              </Typography>
              <Slider
                value={localFontSize}
                onChange={handleFontSizeChange}
                aria-labelledby="font-size-slider"
                valueLabelDisplay="auto"
                step={1}
                marks
                min={8}
                max={32}
              />
            </Box>

            <TextField
              fullWidth
              label="Tab Size"
              type="number"
              value={localTabSize}
              onChange={handleTabSizeChange}
              inputProps={{ min: 1, max: 8, step: 1 }}
              sx={{ mb: 3 }}
            />
          </Box>
        </Box>
      </Paper>

      <Paper sx={{ p: 3, maxWidth: 800 }}>
        <Typography variant="h6" gutterBottom>
          Editor Features
        </Typography>

        <Box>
          <FormControlLabel
            control={<Switch checked={wordWrap} onChange={handleWordWrapToggle} color="primary" />}
            label="Word Wrap"
            sx={{ mb: 1, display: 'block' }}
          />

          <FormControlLabel
            control={<Switch checked={minimap} onChange={handleMinimapToggle} color="primary" />}
            label="Show Minimap"
            sx={{ mb: 1, display: 'block' }}
          />

          <FormControlLabel
            control={
              <Switch checked={lineNumbers} onChange={handleLineNumbersToggle} color="primary" />
            }
            label="Show Line Numbers"
            sx={{ mb: 1, display: 'block' }}
          />
        </Box>
      </Paper>

      {/* Notifications Settings */}
      <Paper sx={{ p: 3, maxWidth: 800, mt: 4 }}>
        <Typography variant="h6" gutterBottom>
          Notifications
        </Typography>

        <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', md: '1fr 1fr' }, gap: 3 }}>
          <Box>
            <FormControlLabel
              control={
                <Switch
                  checked={(localStorage.getItem('notifications.enabled') ?? 'true') === 'true'}
                  onChange={(_e, checked) => {
                    localStorage.setItem('notifications.enabled', String(checked));
                    (globalThis as any).dispatchEvent?.(
                      new Event('notifications:settings-changed')
                    );
                  }}
                  color="primary"
                />
              }
              label="Enable notifications"
              sx={{ mb: 1, display: 'block' }}
            />

            <FormControlLabel
              control={
                <Switch
                  checked={
                    (localStorage.getItem('notifications.showDiagCount') ?? 'true') === 'true'
                  }
                  onChange={(_e, checked) => {
                    localStorage.setItem('notifications.showDiagCount', String(checked));
                    (globalThis as any).dispatchEvent?.(
                      new Event('notifications:settings-changed')
                    );
                  }}
                  color="primary"
                />
              }
              label="Show diagnostics count in toast"
              sx={{ mb: 1, display: 'block' }}
            />
          </Box>

          <Box>
            <TextField
              fullWidth
              label="Notification duration (ms)"
              type="number"
              value={parseInt(localStorage.getItem('notifications.duration') || '4000', 10)}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                const v = Math.max(1000, parseInt((e.target as any).value, 10) || 4000);
                localStorage.setItem('notifications.duration', String(v));
                (globalThis as any).dispatchEvent?.(new Event('notifications:settings-changed'));
              }}
              inputProps={{ min: 1000, step: 500 }}
            />
          </Box>
        </Box>
      </Paper>
    </Box>
  );
}
