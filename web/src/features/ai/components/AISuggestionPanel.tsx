import { DiagnosticSeverity } from 'vscode-languageserver-types';

import {
  AutoFixHigh,
  BugReport,
  CheckCircle,
  Close,
  Code, ExpandLess,
  ExpandMore,
  FilterList, Info,
  Psychology,
  Refresh, Security,
  Sort,
  Speed, Style,
} from '@mui/icons-material';

import type {
  AnalysisCategory,
  AnalysisProgress,
  AnalysisResult,
  CodeAction,
  CodeSmell,
  CodeSuggestion,
  EnhancedAnalysisResult,
  LearnedPattern,
  SeverityLevel,
} from '../types';

// Local type extensions for enhanced analysis results
interface LocalCodeSmell extends Omit<CodeSuggestion, 'id' | 'category'> {
  id?: string;
  smellType?: string;
  lineRange?: [number, number];
  columnRange?: [number, number];
}

interface LocalStyleViolation extends Omit<CodeSuggestion, 'id' | 'category'> {
  id?: string;
  violationType?: string;
  lineRange?: [number, number];
  columnRange?: [number, number];
}

interface LocalPerformanceHint extends Omit<CodeSuggestion, 'id' | 'category'> {
  id?: string;
  optimization?: string;
  hintType?: string;
  estimatedImpact?: string;
  lineRange?: [number, number];
  columnRange?: [number, number];
}

interface LocalSecurityIssue extends Omit<CodeSuggestion, 'id' | 'category'> {
  id?: string;
  recommendation?: string;
  lineRange: [number, number];
  columnRange: [number, number];
  confidence: number;
}

import {
  Accordion, AccordionDetails, AccordionSummary,
  Alert,
  Badge,
  Box,
  Button, Chip,
  CircularProgress,
  Collapse,
  FormControl,
  FormControlLabel,
  IconButton,
  InputLabel,
  LinearProgress, ListItem, MenuItem,
  Paper,
  Select, Switch, Tooltip,
  Typography,
} from '@mui/material';


import React, { FC, useCallback, useEffect, useMemo, useState } from 'react';


// Ensure JSX is available
declare global {
  namespace JSX {
    interface Element { }
    interface IntrinsicElements {
      [elemName: string]: any;
    }
  }
}

// Helper type for severity level conversion
type NumericSeverity = 1 | 2 | 3 | 4; // 1=Error, 2=Warning, 3=Info, 4=Hint

// Helper function to get severity level from number
const getSeverityLevelFromNumber = (severity: number): SeverityLevel => {
  switch (severity) {
    case 0: return 'critical';
    case 1: return 'error';
    case 2: return 'warning';
    case 3: return 'info';
    case 4: return 'hint';
    default:
      return 'info';
  }
};

// Helper function to get severity level label
const getSeverityLabel = (severity: number | SeverityLevel): string => {
  let level: SeverityLevel;
  if (typeof severity === 'number') {
    level = getSeverityLevelFromNumber(severity);
  } else {
    level = severity;
  }

  switch (level) {
    case 'critical': return 'Critical';
    case 'error': return 'Error';
    case 'warning': return 'Warning';
    case 'info': return 'Info';
    case 'hint': return 'Hint';
    default: return 'Info';
  }
};

// Helper function to convert string severity to number
const getSeverityNumber = (level: SeverityLevel): number => {
  switch (level) {
    case 'critical': return 0;
    case 'error': return 1;
    case 'warning': return 2;
    case 'info': return 3;
    case 'hint': return 4;
    default: return 3; // Default to info
  }
};

// Helper function to get severity level color
const getSeverityColor = (severity: number | SeverityLevel): string => {
  let level: SeverityLevel;
  if (typeof severity === 'number') {
    level = getSeverityLevelFromNumber(severity);
  } else {
    level = severity;
  }

  switch (level) {
    case 'critical': return 'error';
    case 'error': return 'error';
    case 'warning': return 'warning';
    case 'info': return 'info';
    case 'hint': return 'default';
    default: return 'info';
  }
};

// Default filter options
const defaultFilterOptions: FilterOptions = {
  categories: [],
  severities: [],
  showOnlyFixable: false,
  minConfidence: 0,
  searchText: ''
};

// Default sort options
const defaultSortOptions: SortOptions = {
  field: 'severity',
  direction: 'desc'
};

interface FilterOptions {
  categories: AnalysisCategory[];
  severities: SeverityLevel[];
  showOnlyFixable: boolean;
  minConfidence: number;
  searchText: string;
}

interface SortOptions {
  field: 'severity' | 'confidence' | 'category' | 'timestamp';
  direction: 'asc' | 'desc';
}

interface AISuggestionPanelProps {
  suggestions: CodeSuggestion[];
  analysisResult?: AnalysisResult;
  enhancedAnalysisResult?: EnhancedAnalysisResult;
  onApplyFix: (suggestion: CodeSuggestion, fix: CodeAction) => void | Promise<void>;
  onDismiss: (suggestion: CodeSuggestion) => void;
  onLearnMore: (suggestion: CodeSuggestion) => void;
  visible: boolean;
  onClose: () => void;
  onRefresh: () => void;
  isAnalyzing: boolean;
  analysisProgress?: AnalysisProgress;
  learnedPatterns: LearnedPattern[];
  onRecordFix: (suggestion: CodeSuggestion, fix: CodeAction) => void;
  filters?: FilterOptions;
  sortOptions?: SortOptions;
  searchText?: string;
}

export const AISuggestionPanel: FC<AISuggestionPanelProps> = ({
  suggestions = [],
  analysisResult,
  enhancedAnalysisResult,
  onApplyFix,
  onDismiss,
  onLearnMore,
  visible = true,
  onClose,
  onRefresh,
  isAnalyzing = false,
  analysisProgress,
  learnedPatterns = [],
  onRecordFix,
  filters = defaultFilterOptions,
  sortOptions = defaultSortOptions,
  searchText = '',
}) => {
  // Local state for filters and sort options
  const [localFilters, setLocalFilters] = useState<FilterOptions>({
    categories: filters?.categories || [],
    severities: filters?.severities || [],
    showOnlyFixable: filters?.showOnlyFixable || false,
    minConfidence: filters?.minConfidence || 0,
    searchText: filters?.searchText || '',
  });

  // Update local state when props change
  useEffect(() => {
    setLocalFilters({
      categories: filters?.categories || [],
      severities: filters?.severities || [],
      showOnlyFixable: filters?.showOnlyFixable || false,
      minConfidence: filters?.minConfidence || 0,
      searchText: filters?.searchText || '',
    });
  }, [filters]);

  // Local state for UI
  const [showFilters, setShowFilters] = useState(false);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  const [applyingFixes, setApplyingFixes] = useState<Set<string>>(new Set());
  const [localSortOptions, setLocalSortOptions] = useState<SortOptions>(sortOptions || defaultSortOptions);

  // Toggle category expansion
  const toggleCategory = useCallback((category: string) => {
    setExpandedCategories(prev => {
      const newSet = new Set(prev);
      if (newSet.has(category)) {
        newSet.delete(category);
      } else {
        newSet.add(category);
      }
      return newSet;
    });
  }, []);

  // Handle applying a fix
  const handleApplyFix = useCallback(async (suggestion: CodeSuggestion, fix: CodeAction) => {
    if (!onApplyFix) return;

    try {
      setApplyingFixes(prev => new Set(prev).add(suggestion.id));
      await onApplyFix(suggestion, fix);
    } catch (error) {
      console.error('Failed to apply fix:', error);
    } finally {
      setApplyingFixes(prev => {
        const newSet = new Set(prev);
        newSet.delete(suggestion.id);
        return newSet;
      });
    }
  }, [onApplyFix]);

  if (!visible) {
    return null;
  }

  // Transform CodeSmell to CodeSuggestion
  const transformToCodeSuggestion = useCallback((smell: CodeSmell): CodeSuggestion => ({
    id: smell.id,
    message: smell.message,
    severity: getSeverityNumber(smell.severity), // convert SeverityLevel to number
    severityLevel: smell.severity, // smell.severity is already SeverityLevel
    range: {
      start: { line: smell.lineRange[0], character: smell.columnRange[0] || 0 },
      end: { line: smell.lineRange[1], character: smell.columnRange[1] || 0 }
    },
    category: 'code-smell' as const,
    confidence: smell.confidence,
    explanation: smell.description || '',
    quickFixes: [],
    source: 'code-analyzer',
    timestamp: Date.now(),
    filePath: smell.filePath,
    suggestion: smell.suggestion || '',
  }), []);

  // Combine legacy and enhanced suggestions with proper type safety
  const allSuggestions = useMemo<CodeSuggestion[]>(() => {
    const legacy = Array.isArray(suggestions) ? suggestions : [];
    const enhanced = enhancedAnalysisResult?.codeSmells?.map(transformToCodeSuggestion) || [];
    return [...legacy, ...enhanced];
  }, [suggestions, enhancedAnalysisResult, transformToCodeSuggestion]);

  // Filter and sort suggestions based on current filters
  const filteredSuggestions = useMemo(() => {
    return allSuggestions.filter(suggestion => {
      // Filter by category
      if (localFilters.categories.length > 0 && !localFilters.categories.includes(suggestion.category)) {
        return false;
      }

      // Filter by severity - check both severityLevel and severity
      if (localFilters.severities.length > 0) {
        const severityLevel = suggestion.severityLevel || getSeverityLevelFromNumber(suggestion.severity);
        if (!localFilters.severities.includes(severityLevel)) {
          return false;
        }
      }

      // Filter by fixable
      if (localFilters.showOnlyFixable && (!suggestion.quickFixes || suggestion.quickFixes.length === 0)) {
        return false;
      }

      // Filter by confidence with null check
      if ((suggestion.confidence || 0) < localFilters.minConfidence) {
        return false;
      }

      // Filter by search text with null checks
      if (localFilters.searchText) {
        const searchLower = localFilters.searchText.toLowerCase();
        const matchesSearch =
          (suggestion.message || '').toLowerCase().includes(searchLower) ||
          (suggestion.explanation && suggestion.explanation.toLowerCase().includes(searchLower)) ||
          (suggestion.suggestion && suggestion.suggestion.toLowerCase().includes(searchLower)) ||
          (suggestion.filePath && suggestion.filePath.toLowerCase().includes(searchLower));

        if (!matchesSearch) return false;
      }

      return true;
    }).sort((a, b) => {
      // Apply sorting based on sortOptions
      let comparison = 0;

      switch (localSortOptions.field) {
        case 'severity':
          const severityOrder = { critical: 0, error: 1, warning: 2, info: 3, hint: 4 };
          // Handle both numeric and string severity values
          const aSeverityLevel = a.severityLevel || getSeverityLevelFromNumber(a.severity);
          const bSeverityLevel = b.severityLevel || getSeverityLevelFromNumber(b.severity);
          // Ensure we have valid severity levels
          const aOrder = aSeverityLevel in severityOrder ? severityOrder[aSeverityLevel] : 4;
          const bOrder = bSeverityLevel in severityOrder ? severityOrder[bSeverityLevel] : 4;
          comparison = aOrder - bOrder;
          break;

        case 'confidence':
          comparison = (b.confidence || 0) - (a.confidence || 0);
          break;

        case 'category':
          comparison = (a.category || '').localeCompare(b.category || '');
          break;

        case 'timestamp':
          const timeA = new Date(a.timestamp || 0).getTime();
          const timeB = new Date(b.timestamp || 0).getTime();
          comparison = timeB - timeA; // Newest first
          break;
      }

      // Apply sort direction
      return localSortOptions.direction === 'asc' ? comparison : -comparison;
    });
  }, [allSuggestions, localFilters]);

  // Group suggestions by category
  const suggestionsByCategory = useMemo<Partial<Record<AnalysisCategory, CodeSuggestion[]>>>(() => {
    // Initialize with all possible categories to avoid undefined errors
    const grouped: Partial<Record<AnalysisCategory, CodeSuggestion[]>> = {
      'code-smell': [],
      'performance': [],
      'security': [],
      'style': [],
      'architecture': [],
      'documentation': [],
    };

    // Initialize all categories to ensure they exist
    const categories: AnalysisCategory[] = ['code-smell', 'performance', 'security', 'style', 'architecture', 'documentation'];
    categories.forEach(cat => {
      if (!grouped[cat]) {
        grouped[cat] = [];
      }
    });

    filteredSuggestions.forEach(suggestion => {
      const category = (suggestion.category && categories.includes(suggestion.category as AnalysisCategory)) 
        ? suggestion.category as AnalysisCategory 
        : 'code-smell';
      
      const targetArray = grouped[category];
      if (targetArray) {
        targetArray.push(suggestion);
      }
    });

    // Filter out empty categories
    Object.keys(grouped).forEach(key => {
      if (grouped[key as AnalysisCategory]?.length === 0) {
        delete grouped[key as AnalysisCategory];
      }
    });

    return grouped;
  }, [filteredSuggestions]);

  // Prepare architecture suggestions from enhanced results
  const architectureSuggestions = useMemo(() => {
    return enhancedAnalysisResult?.architectureSuggestions || [];
  }, [enhancedAnalysisResult?.architectureSuggestions]);

  // Get learned pattern for a suggestion
  const getLearnedPattern = (suggestionId: string) => {
    return learnedPatterns.find(pattern => pattern.successfulFix.id === suggestionId || pattern.errorPattern.id === suggestionId);
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'success';
    if (confidence >= 0.6) return 'warning';
    return 'error';
  };

  // Calculate the score for the header with proper type safety
  const score = useMemo(() => {
    if (enhancedAnalysisResult && typeof enhancedAnalysisResult.qualityScore === 'number') {
      return enhancedAnalysisResult.qualityScore;
    }
    if (analysisResult?.summary?.overallScore) {
      return analysisResult.summary.overallScore;
    }
    return 0;
  }, [enhancedAnalysisResult, analysisResult]);

  return (
    <Paper elevation={3} sx={{ position: 'absolute', bottom: 16, right: 16, width: 500, maxHeight: '80vh', overflow: 'hidden', zIndex: 1300, display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', p: 2, borderBottom: '1px solid', borderColor: 'divider', bgcolor: 'background.paper' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="h6" component="h2">
            AI Code Analysis
          </Typography>
          <Chip label={`Score: ${Math.round(score)}%`} color={score >= 80 ? 'success' : score >= 60 ? 'warning' : 'error'} variant="outlined" size="small" />
        </Box>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Tooltip title="Refresh analysis">
            <IconButton onClick={onRefresh} disabled={isAnalyzing} size="small">
              <Refresh />
            </IconButton>
          </Tooltip>
          <Tooltip title="Filter suggestions">
            <IconButton onClick={() => setShowFilters(!showFilters)} color={showFilters ? 'primary' : 'default'} size="small">
              <FilterList />
            </IconButton>
          </Tooltip>
          <Tooltip title="Close panel">
            <IconButton onClick={onClose} size="small">
              <Close />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Analysis Progress */}
      {isAnalyzing && analysisProgress && (
        <Box sx={{ p: 1.5, borderBottom: '1px solid', borderColor: 'divider' }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
            <Typography variant="body2" color="text.secondary">
              {analysisProgress.stage.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase())}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {Math.round(analysisProgress.progress)}%
            </Typography>
          </Box>
          <LinearProgress variant="determinate" value={analysisProgress.progress} sx={{ mb: 1 }} />
          {analysisProgress.currentFile && (
            <Typography variant="caption" color="text.secondary" noWrap>
              {analysisProgress.currentFile}
            </Typography>
          )}
        </Box>
      )}

      {/* Filters */}
      <Collapse in={showFilters}>
        <Box sx={{ p: 1.5, borderBottom: '1px solid', borderColor: 'divider', bgcolor: 'grey.50' }}>
          <Box sx={{ display: 'flex', gap: 1, mb: 1, flexWrap: 'wrap' }}>
            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>Sort by</InputLabel>
              <Select value={localSortOptions.field} label="Sort by" onChange={(e) => setLocalSortOptions(prev => ({ ...prev, field: e.target.value as any }))}>
                <MenuItem value="severity">Severity</MenuItem>
                <MenuItem value="confidence">Confidence</MenuItem>
                <MenuItem value="category">Category</MenuItem>
                <MenuItem value="timestamp">Time</MenuItem>
              </Select>
            </FormControl>
            <IconButton size="small" onClick={() => setLocalSortOptions(prev => ({ ...prev, direction: prev.direction === 'asc' ? 'desc' : 'asc' }))}>
              <Sort fontSize="small" />
            </IconButton>
          </Box>
          <FormControlLabel control={<Switch checked={localFilters.showOnlyFixable} onChange={(e) => setLocalFilters(prev => ({ ...prev, showOnlyFixable: e.target.checked }))} size="small" />} label="Only fixable" sx={{ fontSize: '0.875rem' }} />
        </Box>
      </Collapse>

      {/* Errors */}
      {analysisResult?.errors && analysisResult.errors.length > 0 && (
        <Box sx={{ p: 1 }}>
          {analysisResult.errors.slice(0, 2).map((error, index) => (
            <Alert key={index} severity="error" sx={{ mb: 1 }}>
              {error.message}
            </Alert>
          ))}
        </Box>
      )}

      {/* Content */}
      <Box sx={{ flex: 1, overflowY: 'auto' }}>
        {filteredSuggestions.length === 0 ? (
          <Box sx={{ p: 3, textAlign: 'center' }}>
            <CheckCircle color="success" sx={{ fontSize: 48, mb: 1 }} />
            <Typography variant="h6" color="text.secondary">
              {isAnalyzing ? 'Analyzing...' : 'No Issues Found'}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {isAnalyzing ? 'Please wait while we analyze your code' : 'Your code looks great!'}
            </Typography>
          </Box>
        ) : (
          Object.entries(suggestionsByCategory).map(([category, categorySuggestions]) => {
            if (categorySuggestions.length === 0) return null;

            return (
              <Accordion key={category} expanded={expandedCategories.has(category)} onChange={() => toggleCategory(category)} sx={{ '&:before': { display: 'none' } }}>
                <AccordionSummary expandIcon={<ExpandMore />}>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, width: '100%' }}>
                    <Box sx={{ color: getCategoryColor(category as AnalysisCategory) }}>
                      {getCategoryIcon(category as AnalysisCategory)}
                    </Box>
                    <Typography variant="subtitle2" sx={{ flex: 1 }}>
                      {category.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                    </Typography>
                    <Badge badgeContent={categorySuggestions.length} color="primary" sx={{ mr: 1 }} />
                  </Box>
                </AccordionSummary>
                <AccordionDetails sx={{ p: 0 }}>
                  {categorySuggestions.map((suggestion, index) => (
                    <Box
                      key={suggestion.id}
                      data-testid="suggestion-item"
                      sx={{ p: 2, borderBottom: index < categorySuggestions.length - 1 ? '1px solid' : 'none', borderColor: 'divider', bgcolor: 'background.default' }}>
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Box sx={{ display: 'flex', gap: 1, alignItems: 'center', flex: 1, flexWrap: 'wrap' }}>
                          <Chip label={getSeverityLabel(suggestion.severity)} size="small" color={getSeverityColor(suggestion.severity) as any} variant="outlined" />
                          <Chip label={`${Math.round((suggestion.confidence || 0) * 100)}%`} size="small" color={getConfidenceColor((suggestion.confidence || 0)) as any} variant="outlined" />
                          {getLearnedPattern(suggestion.id) && (
                            <Chip icon={<Psychology />} label={`Learned (${Math.round((getLearnedPattern(suggestion.id)?.confidence || 0) * 100)}%)`} size="small" color="secondary" variant="outlined" />
                          )}
                          {suggestion.tags?.map(tag => (
                            <Chip key={tag} label={tag} size="small" variant="outlined" sx={{ fontSize: '0.7rem' }} />
                          ))}
                        </Box>
                        <Box>
                          {onDismiss && (
                            <Tooltip title="Dismiss">
                              <IconButton size="small" onClick={() => onDismiss(suggestion)}>
                                <Close fontSize="small" />
                              </IconButton>
                            </Tooltip>
                          )}
                        </Box>
                      </Box>

                      <Typography variant="body2" sx={{ mb: 1, fontWeight: 'medium' }}>
                        {suggestion.message}
                      </Typography>

                      {suggestion.explanation && suggestion.explanation !== suggestion.message && (
                        <Typography variant="body2" color="text.secondary" sx={{ mb: 1.5, fontSize: '0.8rem' }}>
                          {suggestion.explanation}
                        </Typography>
                      )}

                      <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end', flexWrap: 'wrap' }}>
                        {onLearnMore && (
                          <Button size="small" startIcon={<Info fontSize="small" />} onClick={() => onLearnMore(suggestion)}>
                            Learn More
                          </Button>
                        )}

                        {suggestion.quickFixes?.map((quickFix, fixIndex) => (
                          <Button
                            key={fixIndex}
                            size="small"
                            variant="contained"
                            color="primary"
                            startIcon={
                              applyingFixes.has(suggestion.id) ? <CircularProgress size={16} /> : <AutoFixHigh fontSize="small" />
                            }
                            onClick={() => handleApplyFix(suggestion, quickFix)}
                            disabled={applyingFixes.has(suggestion.id)}
                            sx={{ ml: 1 }}
                          >
                            {quickFix.title}
                          </Button>
                        ))}
                      </Box>
                    </Box>
                  ))}
                </AccordionDetails>
              </Accordion>
            );
          })
        )}
      </Box>
    </Paper>
  );
};

// Helper component for rendering individual suggestion items
const SuggestionItem = React.memo(({
  suggestion,
  onApplyFix,
  onDismiss,
  onLearnMore,
  isApplying,
}: {
  suggestion: CodeSuggestion;
  onApplyFix: (suggestion: CodeSuggestion, fix: CodeAction) => void | Promise<void>;
  onDismiss: (suggestion: CodeSuggestion) => void;
  onLearnMore: (suggestion: CodeSuggestion) => void;
  isApplying: boolean;
}) => {
  const [expanded, setExpanded] = useState(false);

  const severityColor = (() => {
    switch (suggestion.severity) {
      case DiagnosticSeverity.Error: return 'error.main';
      case DiagnosticSeverity.Warning: return 'warning.main';
      case DiagnosticSeverity.Information: return 'info.main';
      case DiagnosticSeverity.Hint: return 'text.secondary';
      default: return 'text.secondary';
    }
  })();

  return (
    <ListItem
      sx={{
        borderBottom: '1px solid',
        borderColor: 'divider',
        '&:last-child': { borderBottom: 'none' },
      }}
    >
      <Box sx={{ width: '100%' }} data-testid="suggestion-item">
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            cursor: 'pointer',
            '&:hover': { bgcolor: 'action.hover' },
            p: 1,
            borderRadius: 1,
          }}
          onClick={() => setExpanded(!expanded)}
        >
          <Box sx={{ minWidth: 120 }}>
            <Chip
              label={suggestion.severity}
              size="small"
              sx={{
                bgcolor: severityColor,
                color: 'common.white',
                fontWeight: 'medium',
              }}
            />
          </Box>
          <Typography variant="body2" sx={{ flex: 1 }}>
            {suggestion.message}
          </Typography>
          <IconButton size="small" edge="end">
            {expanded ? <ExpandLess /> : <ExpandMore />}
          </IconButton>
        </Box>

        <Collapse in={expanded} timeout="auto" unmountOnExit>
          <Box sx={{ pl: 2, pr: 1, pb: 1 }}>
            {suggestion.explanation && (
              <Typography variant="body2" color="text.secondary" paragraph>
                {suggestion.explanation}
              </Typography>
            )}

            {suggestion.suggestion && (
              <Box sx={{
                bgcolor: 'background.paper',
                p: 1.5,
                borderRadius: 1,
                borderLeft: '3px solid',
                borderColor: 'primary.main',
                mb: 2,
              }}>
                <Typography variant="subtitle2" gutterBottom>
                  Suggestion:
                </Typography>
                <pre style={{
                  margin: 0,
                  whiteSpace: 'pre-wrap',
                  fontFamily: 'monospace',
                  fontSize: '0.875rem',
                }}>
                  {suggestion.suggestion}
                </pre>
              </Box>
            )}

            <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
              <Button
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  onLearnMore(suggestion);
                }}
              >
                Learn More
              </Button>

              {suggestion.quickFixes?.map((fix, index) => (
                <Button
                  key={index}
                  size="small"
                  variant="contained"
                  onClick={(e) => {
                    e.stopPropagation();
                    onApplyFix(suggestion, fix);
                  }}
                  disabled={isApplying}
                  startIcon={
                    isApplying ? <CircularProgress size={16} /> : <AutoFixHigh />
                  }
                >
                  {fix.title}
                </Button>
              ))}

              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  onDismiss(suggestion);
                }}
              >
                <Close />
              </IconButton>
            </Box>
          </Box>
        </Collapse>
      </Box>
    </ListItem>
  );
});

// Helper function to get icon for category
const getCategoryIcon = (category: AnalysisCategory) => {
  switch (category) {
    case 'performance':
      return <Speed />;
    case 'security':
      return <Security />;
    case 'style':
      return <Style />;
    case 'code-smell':
      return <Code />;
    case 'bug':
      return <BugReport />;
    default:
      return <Info />;
  }
};

// Helper function to get color for category
const getCategoryColor = (category: AnalysisCategory): string => {
  const colorMap: Record<AnalysisCategory, string> = {
    'performance': '#4caf50', // Green
    'security': '#f44336',    // Red
    'style': '#2196f3',       // Blue
    'code-smell': '#ff9800',  // Orange
    'bug': '#9c27b0',        // Purple
    'documentation': '#607d8b', // Blue Gray
    'error': '#f44336',       // Red
    'suggestion': '#00bcd4',   // Cyan
    'architecture': '#00bcd4',   // Cyan
  };

  return colorMap[category] || '#9e9e9e'; // Default to gray if category not found
};

export default AISuggestionPanel;
