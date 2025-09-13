import React, { useState, useCallback } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  TextField,
  Slider,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Switch,
  FormControlLabel,
  Tabs,
  Tab,
  Paper,
  Alert,
} from '@mui/material';
import {
  ExpandMore,
  ColorLens,
  FontDownload,
  BorderStyle,
  LightMode,
  DarkMode,
  Contrast,
} from '@mui/icons-material';
import type {
  ThemeDefinition,
  ThemeCustomization,
  ColorPalette,
  AccessibilityOptions,
} from '../types';
import { useTheme } from '../useTheme';

interface ThemeCustomizerProps {
  open: boolean;
  onClose: () => void;
  theme: ThemeDefinition;
  onSave: (theme: ThemeDefinition) => void;
  onApplyAsCustomization?: (customization: ThemeCustomization) => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel({ children, value, index, ...other }: TabPanelProps) {
  return (
    <div role="tabpanel" hidden={value !== index} {...other}>
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

const ThemeCustomizer: React.FC<ThemeCustomizerProps> = ({
  open,
  onClose,
  theme,
  onSave,
  onApplyAsCustomization,
}) => {
  const { applyThemeCustomization, generatePalette } = useTheme();
  const [activeTab, setActiveTab] = useState(0);
  const [workingTheme, setWorkingTheme] = useState<ThemeDefinition>(() => ({ ...theme }));
  const [customization, setCustomization] = useState<ThemeCustomization>({});

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const updateColor = useCallback(
    (path: string, value: string) => {
      const newTheme = { ...workingTheme };
      const pathParts = path.split('.');
      let current: any = newTheme.colors;

      for (let i = 0; i < pathParts.length - 1; i++) {
        if (!current[pathParts[i]]) {
          current[pathParts[i]] = {};
        }
        current = current[pathParts[i]];
      }
      current[pathParts[pathParts.length - 1]] = value;
      setWorkingTheme(newTheme);

      // Update customization
      const newCustomization = { ...customization };
      if (!newCustomization.colorOverrides) {
        newCustomization.colorOverrides = {};
      }
      current = newCustomization.colorOverrides;

      for (let i = 0; i < pathParts.length - 1; i++) {
        if (!current[pathParts[i]]) {
          current[pathParts[i]] = {};
        }
        current = current[pathParts[i]];
      }
      current[pathParts[pathParts.length - 1]] = value;
      setCustomization(newCustomization);
    },
    [workingTheme, customization]
  );

  const batchGeneratePalette = useCallback(() => {
    const baseColor = workingTheme.colors.primary;
    const generatedPalette = generatePalette(baseColor);

    const newTheme = { ...workingTheme, colors: generatedPalette };
    setWorkingTheme(newTheme);

    setCustomization({
      ...customization,
      colorOverrides: generatedPalette,
    });
  }, [workingTheme, generatePalette, customization]);

  const handleSave = () => {
    onSave(workingTheme);
    onClose();
  };

  const handleApplyCustomization = () => {
    if (onApplyAsCustomization) {
      onApplyAsCustomization(customization);
    } else {
      applyThemeCustomization(customization);
    }
    onClose();
  };

  const renderColorPicker = (label: string, path: string, currentValue: string) => (
    <Box sx={{ mb: 2 }}>
      <Typography variant="subtitle2" sx={{ mb: 1 }}>
        {label}
      </Typography>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
        <input
          type="color"
          value={currentValue}
          onChange={(e) => updateColor(path, e.target.value)}
          style={{
            width: 50,
            height: 40,
            border: '1px solid #ccc',
            borderRadius: 4,
            cursor: 'pointer',
          }}
        />
        <TextField
          size="small"
          value={currentValue}
          onChange={(e) => updateColor(path, e.target.value)}
          sx={{ flex: 1 }}
        />
      </Box>
    </Box>
  );

  const renderColorSection = (section: string, colors: Record<string, any>) => (
    <Accordion key={section}>
      <AccordionSummary expandIcon={<ExpandMore />}>
        <Typography variant="h6" sx={{ textTransform: 'capitalize' }}>
          {section.replace(/([A-Z])/g, ' $1')}
        </Typography>
      </AccordionSummary>
      <AccordionDetails>
        {Object.entries(colors).map(([key, value]) => {
          if (typeof value === 'string') {
            return renderColorPicker(
              key.replace(/([A-Z])/g, ' $1').toLowerCase(),
              `${section}.${key}`,
              value
            );
          } else if (typeof value === 'object') {
            return (
              <Box key={key} sx={{ ml: 2 }}>
                <Typography variant="subtitle1" sx={{ mb: 1, textTransform: 'capitalize' }}>
                  {key.replace(/([A-Z])/g, ' $1').toLowerCase()}
                </Typography>
                {Object.entries(value).map(([subKey, subValue]) =>
                  typeof subValue === 'string'
                    ? renderColorPicker(
                        subKey.replace(/([A-Z])/g, ' $1').toLowerCase(),
                        `${section}.${key}.${subKey}`,
                        subValue
                      )
                    : null
                )}
              </Box>
            );
          }
          return null;
        })}
      </AccordionDetails>
    </Accordion>
  );

  return (
    <Dialog open={open} onClose={onClose} maxWidth="lg" fullWidth>
      <DialogTitle>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <ColorLens />
          <Typography variant="h6">Customize Theme: {theme.name}</Typography>
        </Box>
      </DialogTitle>

      <DialogContent>
        <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
          <Tabs value={activeTab} onChange={handleTabChange}>
            <Tab icon={<ColorLens />} label="Colors" />
            <Tab icon={<FontDownload />} label="Typography" />
            <Tab icon={<BorderStyle />} label="Spacing & Borders" />
            <Tab icon={<Contrast />} label="Accessibility" />
          </Tabs>
        </Box>

        <TabPanel value={activeTab} index={0}>
          <Alert severity="info" sx={{ mb: 2 }}>
            Changes here affect the current theme's color palette. Use the "Generate Palette" button
            to auto-generate harmonious colors from your primary color.
          </Alert>

          <Button
            onClick={batchGeneratePalette}
            variant="outlined"
            startIcon={<LightMode />}
            sx={{ mb: 2 }}
          >
            Generate Palette from Primary Color
          </Button>

          {Object.entries(workingTheme.colors).map(([section, colors]) => {
            if (typeof colors === 'object' && !Array.isArray(colors)) {
              return renderColorSection(section, colors);
            }
            return null;
          })}
        </TabPanel>

        <TabPanel value={activeTab} index={1}>
          <Typography variant="body1" sx={{ mb: 2 }}>
            Typography customization coming soon...
          </Typography>
        </TabPanel>

        <TabPanel value={activeTab} index={2}>
          <Typography variant="body1" sx={{ mb: 2 }}>
            Spacing and border customization coming soon...
          </Typography>
        </TabPanel>

        <TabPanel value={activeTab} index={3}>
          <Typography variant="body1" sx={{ mb: 2 }}>
            Accessibility options customization coming soon...
          </Typography>
        </TabPanel>
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose} color="inherit">
          Cancel
        </Button>
        {onApplyAsCustomization && (
          <Button onClick={handleApplyCustomization} variant="outlined" color="primary">
            Apply as Customization
          </Button>
        )}
        <Button onClick={handleSave} variant="contained" color="primary">
          Save Theme
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default ThemeCustomizer;
