import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import * as tomlParser from '@iarna/toml';
import { useDispatch, useSelector } from 'react-redux';
import Editor, { Monaco, useMonaco } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';
import { editor, Position, Range } from 'monaco-editor';
import { languages } from 'monaco-editor';
import { crateService } from '../../services/crateService';
import {
  Box,
  Typography,
  Button,
  Paper,
  CircularProgress,
  Alert,
  Tabs,
  Tab,
  useTheme,
  IconButton,
  Tooltip,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
} from '@mui/material';
import {
  Code as CodeIcon,
  List as ListIcon,
  Save as SaveIcon,
  Refresh as RefreshIcon,
  Edit as EditIcon,
  Info as InfoIcon,
} from '@mui/icons-material';
import { selectCurrentProjectPath } from '../../store/slices/cargoSlice';
import Markdown from 'react-markdown';

interface Dependency {
  name: string;
  version: string;
  features: string[];
  optional?: boolean;
  'default-features'?: boolean;
}

interface CargoTomlData {
  package?: {
    name?: string;
    version?: string;
    [key: string]: unknown;
  };
  dependencies?: Record<string, string | { version?: string; features?: string[]; [key: string]: unknown }>;
  [key: string]: unknown;
}

// Register TOML language configuration
const setupMonaco = (monaco: Monaco) => {
  monaco.languages.register({ id: 'toml' });
  monaco.languages.setMonarchTokensProvider('toml', {
    defaultToken: '',
    tokenPostfix: '.toml',
    escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
    tokenizer: {
      root: [
        [/(\[\s*)([^\[\s]+)(\s*\])/, ['delimiter.bracket.toml', 'type.toml', 'delimiter.bracket.toml']],
        [/([^\[\s]\s*)(=)(\s*)/, ['key.toml', 'delimiter.toml', '']],
        [/("[^\n"]*$)|("[^\n"]*\")/, 'string.invalid.toml'],
        [/("[^\n"]*\")/, 'string.toml'],
        [/\d+/, 'number.toml'],
        [/true|false/, 'constant.boolean.toml'],
        [/[a-zA-Z0-9_\-]+/, 'identifier.toml'],
        [/[\[\]{}()\[\].]/, 'delimiter.bracket.toml'],
        [/[=,;:]/, 'delimiter.toml'],
        [/\s+/, 'white'],
        [/[^\s\[\]{}()=,\n]+/, 'variable.source'],
      ],
    },
  });

  monaco.editor.defineTheme('toml-theme', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'delimiter.bracket.toml', foreground: '0000FF' },
      { token: 'type.toml', foreground: '267F99', fontStyle: 'bold' },
      { token: 'key.toml', foreground: '001080' },
      { token: 'string.toml', foreground: 'A31515' },
      { token: 'number.toml', foreground: '098658' },
      { token: 'constant.boolean.toml', foreground: '0000FF', fontStyle: 'bold' },
      { token: 'delimiter.toml', foreground: '000000' },
    ],
    colors: {
      'editor.foreground': '#000000',
    },
  });
};

const CargoTomlEditor: React.FC = () => {
  const theme = useTheme();
  const currentProjectPath = useSelector(selectCurrentProjectPath);
  const [tomlContent, setTomlContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'editor' | 'dependencies'>('editor');
  const [dependencies, setDependencies] = useState<Dependency[]>([]);
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const [isDirty, setIsDirty] = useState<boolean>(false);

  // State for dependency documentation
  const [dependencyDocs, setDependencyDocs] = useState<Record<string, any>>({});
  const [loadingDocs, setLoadingDocs] = useState<Record<string, boolean>>({});

  // Fetch documentation for a dependency
  const fetchDependencyDocs = useCallback(async (name: string) => {
    if (dependencyDocs[name] || loadingDocs[name]) return;
    
    setLoadingDocs(prev => ({ ...prev, [name]: true }));
    
    try {
      const response = await fetch(`https://crates.io/api/v1/crates/${name}`);
      if (response.ok) {
        const data = await response.json();
        setDependencyDocs(prev => ({
          ...prev,
          [name]: data.crate
        }));
      }
    } catch (error) {
      console.error(`Failed to fetch docs for ${name}:`, error);
    } finally {
      setLoadingDocs(prev => ({ ...prev, [name]: false }));
    }
  }, [dependencyDocs, loadingDocs]);

  // Parse dependencies from TOML content
  const parseDependencies = useCallback((content: string) => {
    try {
      const parsed = tomlParser.parse(content) as unknown as CargoTomlData;
      const deps: Dependency[] = [];
      
      // Extract dependencies from different sections
      const sections = ['dependencies', 'dev-dependencies', 'build-dependencies'];
      sections.forEach(section => {
        if (parsed[section]) {
          Object.entries(parsed[section]).forEach(([name, info]) => {
            if (typeof info === 'string') {
              deps.push({ name, version: info, features: [] });
              fetchDependencyDocs(name);
            } else if (info && typeof info === 'object') {
              deps.push({
                name,
                version: info.version || '*',
                features: info.features || [],
                optional: info.optional,
                'default-features': info['default-features']
              });
              fetchDependencyDocs(name);
            }
          });
        }
      });
      
      setDependencies(deps);
    } catch (error) {
      console.error('Error parsing dependencies:', error);
      setError('Failed to parse Cargo.toml dependencies');
    }
  }, [fetchDependencyDocs]);

  // Load Cargo.toml content
  const loadCargoToml = useCallback(async () => {
    if (!currentProjectPath) return;

    setIsLoading(true);
    setError(null);

    try {
      const response = await window.electron.invoke('fs:readFile', {
        path: `${currentProjectPath}/Cargo.toml`,
        encoding: 'utf-8',
      });

      setTomlContent(response);
      parseDependencies(response);
      setIsDirty(false);
      
      if (editorRef.current) {
        editorRef.current.setValue(response);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred';
      setError(`Failed to load Cargo.toml: ${errorMessage}`);
    } finally {
      setIsLoading(false);
    }
  }, [currentProjectPath, parseDependencies]);

  // Save Cargo.toml content
  const saveCargoToml = async () => {
    if (!currentProjectPath || !editorRef.current) return;

    setIsLoading(true);
    setError(null);

    try {
      const content = editorRef.current.getValue();
      await window.electron.invoke('fs:writeFile', {
        path: `${currentProjectPath}/Cargo.toml`,
        content,
        encoding: 'utf-8',
      });
      
      setTomlContent(content);
      parseDependencies(content);
      setIsDirty(false);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to save Cargo.toml';
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  // Register quick fixes for common TOML issues
  const registerQuickFixes = useCallback((monaco: Monaco) => {
    return monaco.languages.registerCodeActionProvider('toml', {
      provideCodeActions: (model, range, context) => {
        const codeActions: any[] = [];
        const lineContent = model.getLineContent(range.startLineNumber);
        
        // Check for missing quotes around version numbers
        const versionMatch = lineContent.match(/^(\s*[a-zA-Z0-9_-]+\s*=\s*)([^\s"'].*)$/);
        if (versionMatch && !lineContent.includes('{') && !lineContent.includes('}')) {
          const version = versionMatch[2].split(/[\s,#]/)[0];
          if (version && !version.startsWith('"') && !version.startsWith('\'')) {
            codeActions.push({
              title: 'Add quotes around version',
              kind: 'quickfix',
              edit: {
                edits: [{
                  resource: model.uri,
                  edit: {
                    range: new monaco.Range(
                      range.startLineNumber,
                      versionMatch[1].length + 1,
                      range.startLineNumber,
                      versionMatch[1].length + version.length + 1
                    ),
                    text: `"${version}"`
                  }
                }]
              },
              isPreferred: true
            });
          }
        }

        // Check for missing version specifier
        const missingVersionMatch = lineContent.match(/^(\s*[a-zA-Z0-9_-]+\s*=\s*)$/);
        if (missingVersionMatch) {
          codeActions.push({
            title: 'Add version specifier',
            kind: 'quickfix',
            edit: {
              edits: [{
                resource: model.uri,
                edit: {
                  range: new monaco.Range(
                    range.startLineNumber,
                    missingVersionMatch[1].length + 1,
                    range.startLineNumber,
                    missingVersionMatch[1].length + 1
                  ),
                  text: '"*"  # Latest version'
                }
              }]
            }
          });
        }

        // Check for missing comma in array
        if (lineContent.includes('[') && lineContent.includes(']') && !lineContent.includes(',')) {
          const arrayMatch = lineContent.match(/\[([^\]]+)\]/);
          if (arrayMatch && arrayMatch[1].trim()) {
            const items = arrayMatch[1].split(/\s+/).filter(Boolean);
            if (items.length > 1) {
              const fixedContent = `[${items.join(', ')}]`;
              codeActions.push({
                title: 'Add missing commas to array',
                kind: 'quickfix',
                edit: {
                  edits: [{
                    resource: model.uri,
                    edit: {
                      range: model.getFullModelRange(),
                      text: model.getValue().replace(/\[([^\]]+)\]/g, (_, p1) => 
                        `[${p1.split(/\s+/).filter(Boolean).join(', ')}]`
                      )
                    }
                  }]
                },
                isPreferred: true
              });
            }
          }
        }

        return {
          actions: codeActions,
          dispose: () => {}
        };
      }
    });
  }, []);

  // Setup IntelliSense providers
  const setupIntelliSense = useCallback((monaco: Monaco) => {
    // Register completion item provider for TOML
    return monaco.languages.registerCompletionItemProvider('toml', {
      triggerCharacters: ['"', ' ', '='],
      provideCompletionItems: async (model, position) => {
        const lineContent = model.getLineContent(position.lineNumber);
        const textUntilPosition = model.getValueInRange({
          startLineNumber: 1,
          startColumn: 1,
          endLineNumber: position.lineNumber,
          endColumn: position.column
        });

        // Check if we're in the dependencies section
        const isInDependencies = /\[dependencies\]/.test(textUntilPosition);
        if (!isInDependencies) return { suggestions: [] };

        // Check if we're at a crate name position
        const lineUntilPosition = lineContent.substring(0, position.column - 1);
        const isAtCrateName = /^\s*[a-zA-Z0-9_-]*$/.test(lineUntilPosition.trim());

        if (isAtCrateName) {
          // Provide crate name suggestions
          const match = lineUntilPosition.match(/([a-zA-Z0-9_-]*)$/);
          const word = match ? match[1] : '';
          
          const suggestions = await crateService.searchCrates(word);
          
          return {
            suggestions: suggestions.map(suggestion => ({
              label: suggestion.name,
              kind: monaco.languages.CompletionItemKind.Module,
              documentation: suggestion.description,
              insertText: suggestion.name,
              range: new monaco.Range(
                position.lineNumber,
                position.column - word.length,
                position.lineNumber,
                position.column
              )
            }))
          };
        }

        // Check if we're at a version position
        const isAtVersion = /=\s*["']?[0-9\.\*\^~><= ]*$/.test(lineUntilPosition);
        if (isAtVersion) {
          const lineTillPosition = model.getValueInRange({
            startLineNumber: position.lineNumber,
            startColumn: 1,
            endLineNumber: position.lineNumber,
            endColumn: position.column
          });
          
          const crateNameMatch = lineTillPosition.match(/^\s*([a-zA-Z0-9_-]+)\s*=/);
          if (!crateNameMatch) return { suggestions: [] };
          
          const crateName = crateNameMatch[1];
          const versions = await crateService.getCrateVersions(crateName);
          
          return {
            suggestions: versions.slice(0, 10).map(version => ({
              label: version.num,
              kind: monaco.languages.CompletionItemKind.Value,
              insertText: `"${version.num}"`,
              range: new monaco.Range(
                position.lineNumber,
                position.column,
                position.lineNumber,
                position.column
              )
            }))
          };
        }

        // Check if we're at features position
        const isAtFeatures = /features\s*=\s*\[([^\]]*)$/.test(lineUntilPosition);
        if (isAtFeatures) {
          const lineTillPosition = model.getValueInRange({
            startLineNumber: 1,
            startColumn: 1,
            endLineNumber: position.lineNumber,
            endColumn: position.column
          });
          
          const crateMatch = lineTillPosition.match(/^\s*([a-zA-Z0-9_-]+)\s*=/m);
          if (!crateMatch) return { suggestions: [] };
          
          const crateName = crateMatch[1];
          const features = await crateService.getCrateFeatures(crateName);
          
          return {
            suggestions: Object.keys(features).map(feature => ({
              label: feature,
              kind: monaco.languages.CompletionItemKind.Enum,
              insertText: `"${feature}"`,
              range: new monaco.Range(
                position.lineNumber,
                position.column,
                position.lineNumber,
                position.column
              ),
              documentation: features[feature]?.join('\n') || 'No description available'
            }))
          };
        }

        return { suggestions: [] };
      }
    });
  }, []);

  // Handle editor mount
  const handleEditorDidMount = useCallback((editor: editor.IStandaloneCodeEditor, monaco: Monaco) => {
    editorRef.current = editor;
    setupMonaco(monaco);
    
    // Register providers
    const intellisenseDisposable = setupIntelliSense(monaco);
    const quickFixesDisposable = registerQuickFixes(monaco);
    
    monaco.editor.setTheme(theme.palette.mode === 'dark' ? 'vs-dark' : 'vs-light');
    
    // Add listener for content changes
    const contentChangeDisposable = editor.onDidChangeModelContent(() => {
      setIsDirty(true);
    });
    
    return () => {
      // Cleanup disposables
      intellisenseDisposable?.dispose();
      quickFixesDisposable?.dispose();
      contentChangeDisposable.dispose();
    };
  }, [theme.palette.mode, setupIntelliSense, registerQuickFixes]);

  // Format markdown for tooltip content
  const formatTooltipContent = (dep: Dependency, doc: any) => {
    const lines = [
      `# ${dep.name} ${dep.version}`,
      doc?.description ? `\n${doc.description}` : '',
      doc?.documentation ? `\n[Documentation](${doc.documentation})` : '',
      doc?.homepage ? `\n[Homepage](${doc.homepage})` : '',
      doc?.repository ? `\n[Repository](${doc.repository})` : '',
      `\n---\n**Features:** ${dep.features.length > 0 ? dep.features.join(', ') : 'None'}`,
      dep.optional ? '\n**Optional:** Yes' : '',
      dep['default-features'] !== undefined ? `\n**Default Features:** ${dep['default-features'] ? 'Enabled' : 'Disabled'}` : ''
    ];
    
    return lines.join('\n');
  };

  // Dependency list component
  const DependencyList = () => (
    <List dense>
      {dependencies.map((dep) => {
        const doc = dependencyDocs[dep.name];
        const isLoading = loadingDocs[dep.name];
        const hasDocs = !!doc;
        
        return (
          <Tooltip 
            key={dep.name} 
            title={
              <Box sx={{ maxWidth: 400 }}>
                {isLoading ? (
                  <Box p={1}><CircularProgress size={20} /></Box>
                ) : hasDocs ? (
                  <Markdown>{formatTooltipContent(dep, doc)}</Markdown>
                ) : (
                  <Box p={1}>
                    <Typography variant="subtitle2">{dep.name} {dep.version}</Typography>
                    <Typography variant="body2">No additional information available</Typography>
                    <Typography variant="caption">Click to edit</Typography>
                  </Box>
                )}
              </Box>
            }
            arrow
            placement="right"
            enterDelay={500}
          >
            <ListItem disablePadding>
              <ListItemButton>
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      {dep.name}
                      {isLoading && <CircularProgress size={12} />}
                      {hasDocs && <InfoIcon fontSize="small" color="action" />}
                    </Box>
                  }
                  secondary={`${dep.version} ${dep.optional ? '(optional)' : ''}`}
                />
                <IconButton 
                  edge="end" 
                  size="small" 
                  onClick={(e) => {
                    e.stopPropagation();
                    // Handle edit
                  }}
                >
                  <EditIcon fontSize="small" />
                </IconButton>
              </ListItemButton>
            </ListItem>
          </Tooltip>
        );
      })}
    </List>
  );

  if (!currentProjectPath) {
    return (
      <Box p={2}>
        <Typography color="textSecondary">
          No project is currently open. Please open a Rust project to manage its dependencies.
        </Typography>
      </Box>
    );
  }

  if (isLoading && !tomlContent) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Paper sx={{ p: 2, height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="h6">Cargo.toml</Typography>
        <Box>
          <Tooltip title="Reload Cargo.toml">
            <IconButton onClick={loadCargoToml} disabled={isLoading} sx={{ mr: 1 }}>
              <RefreshIcon />
            </IconButton>
          </Tooltip>
          <Button
            startIcon={<SaveIcon />}
            onClick={saveCargoToml}
            disabled={!isDirty || isLoading}
            variant="contained"
            color="primary"
          >
            Save
          </Button>
        </Box>
      </Box>

      <Tabs
        value={activeTab}
        onChange={(_, newValue) => setActiveTab(newValue)}
        sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}
      >
        <Tab 
          icon={<CodeIcon />} 
          label="Editor" 
          value="editor" 
          iconPosition="start"
        />
        <Tab 
          icon={<ListIcon />} 
          label="Dependencies" 
          value="dependencies" 
          iconPosition="start"
        />
      </Tabs>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <Box sx={{ flexGrow: 1, overflow: 'hidden' }}>
        {activeTab === 'editor' ? (
          <Editor
            height="100%"
            defaultLanguage="toml"
            value={tomlContent}
            theme={theme.palette.mode === 'dark' ? 'vs-dark' : 'vs-light'}
            onMount={handleEditorDidMount}
            options={{
              minimap: { enabled: true },
              scrollBeyondLastLine: false,
              fontSize: 14,
              wordWrap: 'on',
              automaticLayout: true,
              tabSize: 2,
              formatOnType: true,
              formatOnPaste: true,
            }}
          />
        ) : (
          <Box sx={{ p: 2, height: '100%', overflow: 'auto' }}>
            <Typography variant="subtitle1" gutterBottom>
              Dependencies
            </Typography>
            {dependencies.length === 0 ? (
              <Typography variant="body2" color="textSecondary">
                No dependencies found or failed to parse Cargo.toml
              </Typography>
            ) : (
              <DependencyList />
            )}
          </Box>
        )}
      </Box>
    </Paper>
  );
};

export default CargoTomlEditor;
