import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  Box,
  TextField,
  Button,
  Paper,
  List,
  ListItem,
  ListItemText,
  Typography,
  IconButton,
  Chip,
  Autocomplete,
  Divider,
  Drawer,
  ListItemIcon,
  Tooltip,
  CircularProgress,
  Fab,
} from '@mui/material';
import {
  PlayArrow as RunIcon,
  History as HistoryIcon,
  SmartToy as AIIcon,
  Tab as AutoCompleteIcon,
  BookmarkAdd as BookmarkIcon,
  Bookmark as BookmarkedIcon,
  Settings as SettingsIcon,
  ExpandLess as ExpandLessIcon,
  ExpandMore as ExpandMoreIcon,
  Clear as ClearIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

// Terminal-related interfaces matching Rust types
interface CommandHistoryEntry {
  id: string;
  command: string;
  working_directory: string;
  timestamp: number;
  success: boolean;
  output_length?: number;
}

interface CompletionSuggestion {
  value: string;
  category: string;
  score: number;
  description?: string;
}

interface AICommandSuggestion {
  command: string;
  explanation: string;
  confidence_score: number;
  category: string;
}

interface TerminalBookmark {
  id: string;
  name: string;
  command: string;
  description?: string;
  created_at: number;
}

interface TerminalEvent {
  id: string;
  stream_type: string;
  line: string;
}

interface EnhancedTerminalProps {
  id: string;
  initialWorkingDirectory?: string;
  onDirectoryChange?: (directory: string) => void;
  height?: string | number;
}

const EnhancedTerminal: React.FC<EnhancedTerminalProps> = ({
  id,
  initialWorkingDirectory = '.',
  onDirectoryChange,
  height = 400,
}) => {
  const [command, setCommand] = useState('');
  const [workingDirectory, setWorkingDirectory] = useState(initialWorkingDirectory);
  const [output, setOutput] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [historyDrawer, setHistoryDrawer] = useState(false);
  const [bookmarksDrawer, setBookmarksDrawer] = useState(false);
  const [history, setHistory] = useState<CommandHistoryEntry[]>([]);
  const [bookmarks, setBookmarks] = useState<TerminalBookmark[]>([]);
  const [suggestions, setSuggestions] = useState<AICommandSuggestion[]>([]);
  const [completions, setCompletions] = useState<CompletionSuggestion[]>([]);
  const [showAISuggestions, setShowAISuggestions] = useState(false);
  const [showAutoComplete, setShowAutoComplete] = useState(false);
  const [loading, setLoading] = useState(false);

  const outputEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Scroll to bottom of output when it changes
  useEffect(() => {
    if (outputEndRef.current) {
      outputEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [output]);

  // Load initial data
  useEffect(() => {
    loadHistory();
    loadBookmarks();
    // Listen for terminal events
    const unsubscribe = window.addEventListener('terminal-' + id, (event: any) => {
      handleTerminalEvent(event.detail);
    });

    return () => {
      unsubscribe?.remove();
    };
  }, [id]);

  const loadHistory = useCallback(async () => {
    try {
      const result = await invoke('get_command_history', { limit: 50 });
      if (result.status === 'success') {
        setHistory(result.history);
      }
    } catch (error) {
      console.error('Failed to load command history:', error);
    }
  }, []);

  const loadBookmarks = useCallback(async () => {
    try {
      const result = await invoke('get_terminal_bookmarks');
      if (result.status === 'success') {
        setBookmarks(result.bookmarks);
      }
    } catch (error) {
      console.error('Failed to load bookmarks:', error);
    }
  }, []);

  const handleTerminalEvent = (event: TerminalEvent) => {
    if (event.id === id) {
      if (event.stream_type === 'stdout' || event.stream_type === 'stderr') {
        setOutput((prev) => [...prev, event.line]);
      } else if (event.stream_type === 'completion') {
        setIsRunning(false);
      } else if (event.stream_type === 'error') {
        setOutput((prev) => [...prev, `Error: ${event.line}`]);
        setIsRunning(false);
      }
    }
  };

  const executeCommand = async (cmdToExecute: string = command) => {
    if (!cmdToExecute.trim() || isRunning) return;

    setIsRunning(true);
    setOutput((prev) => [...prev, `$ ${cmdToExecute}`]);

    // Add to history
    const historyEntry: CommandHistoryEntry = {
      id: `cmd_${Date.now()}`,
      command: cmdToExecute,
      working_directory: workingDirectory,
      timestamp: Date.now(),
      success: false,
      output_length: undefined,
    };

    await invoke('add_command_to_history', { entry: historyEntry });
    loadHistory();

    // Execute command
    try {
      await invoke('terminal_execute_stream', {
        program: cmdToExecute.split(' ')[0],
        args: cmdToExecute.split(' ').slice(1),
        directory: workingDirectory,
        id,
      });
    } catch (error) {
      console.error('Failed to execute command:', error);
      setOutput((prev) => [...prev, `Command execution failed: ${error}`]);
      setIsRunning(false);
    }
  };

  const getAISuggestions = async () => {
    if (command.trim().length < 2) return;

    setLoading(true);
    try {
      const result = await invoke('get_ai_command_suggestions', {
        partial_command: command,
        context: 'terminal',
      });
      if (result.status === 'success') {
        setSuggestions(result.suggestions);
        setShowAISuggestions(true);
      }
    } catch (error) {
      console.error('Failed to get AI suggestions:', error);
    } finally {
      setLoading(false);
    }
  };

  const getAutoCompletions = async () => {
    if (command.trim().length < 1) return;

    try {
      const result = await invoke('get_auto_completion', {
        partial: command,
        working_directory: workingDirectory,
      });
      if (result.status === 'success') {
        setCompletions(result.suggestions);
        setShowAutoComplete(true);
      }
    } catch (error) {
      console.error('Failed to get completions:', error);
    }
  };

  const addBookmark = async () => {
    if (!command.trim()) return;

    const bookmark: TerminalBookmark = {
      id: `bookmark_${Date.now()}`,
      name: `Bookmark ${bookmarks.length + 1}`,
      command,
      description: undefined,
      created_at: Date.now(),
    };

    try {
      await invoke('add_terminal_bookmark', { bookmark });
      loadBookmarks();
    } catch (error) {
      console.error('Failed to add bookmark:', error);
    }
  };

  const clearOutput = () => setOutput([]);

  return (
    <Paper
      sx={{
        height,
        display: 'flex',
        flexDirection: 'column',
        bgcolor: 'grey.900',
        color: 'grey.100',
      }}
    >
      {/* Terminal header */}
      <Box
        sx={{
          p: 1,
          borderBottom: '1px solid',
          borderColor: 'divider',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <Typography variant="subtitle2">Terminal - {workingDirectory}</Typography>
        <Box sx={{ display: 'flex', gap: 0.5 }}>
          <Tooltip title="Command History">
            <IconButton
              size="small"
              onClick={() => setHistoryDrawer(true)}
              sx={{ color: 'text.secondary' }}
            >
              <HistoryIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="Terminal Bookmarks">
            <IconButton
              size="small"
              onClick={() => setBookmarksDrawer(true)}
              sx={{ color: 'text.secondary' }}
            >
              <BookmarkIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="Clear Output">
            <IconButton size="small" onClick={clearOutput} sx={{ color: 'text.secondary' }}>
              <ClearIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Terminal output */}
      <Box
        sx={{
          flex: 1,
          p: 1,
          overflow: 'auto',
          fontFamily: 'monospace',
          fontSize: '0.875rem',
          bgcolor: 'black',
        }}
      >
        {output.map((line, index) => (
          <div key={index} style={{ marginBottom: 2 }}>
            {line}
          </div>
        ))}
        <div ref={outputEndRef} />
      </Box>

      {/* AI Suggestions */}
      {showAISuggestions && suggestions.length > 0 && (
        <Paper
          sx={{
            position: 'absolute',
            bottom: 80,
            left: 8,
            right: 8,
            maxHeight: 200,
            overflow: 'auto',
            zIndex: 1000,
          }}
        >
          <Box sx={{ p: 1 }}>
            <Typography variant="subtitle2" sx={{ mb: 1 }}>
              AI Suggestions
            </Typography>
            <List dense>
              {suggestions.map((suggestion, index) => (
                <ListItem
                  key={index}
                  button
                  onClick={() => {
                    setCommand(suggestion.command);
                    setShowAISuggestions(false);
                  }}
                >
                  <ListItemText
                    primary={suggestion.command}
                    secondary={
                      <>
                        {suggestion.explanation}
                        <Chip
                          size="small"
                          label={`${(suggestion.confidence_score * 100).toFixed(1)}%`}
                          sx={{ ml: 1 }}
                        />
                      </>
                    }
                  />
                </ListItem>
              ))}
            </List>
          </Box>
        </Paper>
      )}

      {/* Auto-completion */}
      {showAutoComplete && completions.length > 0 && (
        <Paper
          sx={{
            position: 'absolute',
            bottom: 80,
            left: 8,
            maxHeight: 150,
            overflow: 'auto',
            zIndex: 1000,
          }}
        >
          <List dense>
            {completions.map((completion, index) => (
              <ListItem
                key={index}
                button
                onClick={() => {
                  setCommand(completion.value);
                  setShowAutoComplete(false);
                  inputRef.current?.focus();
                }}
              >
                <ListItemText
                  primary={completion.value}
                  secondary={
                    <>
                      {completion.category}
                      {completion.description && (
                        <span style={{ marginLeft: '1em', color: 'text.secondary' }}>
                          {completion.description}
                        </span>
                      )}
                    </>
                  }
                />
              </ListItem>
            ))}
          </List>
        </Paper>
      )}

      {/* Terminal input */}
      <Box sx={{ p: 1, borderTop: '1px solid', borderColor: 'divider' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Typography variant="body2" sx={{ minWidth: 'fit-content', color: 'text.secondary' }}>
            $
          </Typography>
          <TextField
            inputRef={inputRef}
            fullWidth
            variant="standard"
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                executeCommand();
              } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                // Navigate history
                const recentCommand = history.find((h) => h.success)?.command;
                if (recentCommand) setCommand(recentCommand);
              } else if (e.key === 'Tab') {
                e.preventDefault();
                if (command.includes(' ')) {
                  getAutoCompletions();
                } else {
                  getAISuggestions();
                }
              }
            }}
            placeholder="Enter command..."
            InputProps={{
              disableUnderline: true,
              sx: { color: 'grey.100' },
            }}
            disabled={isRunning}
          />
          {isRunning && <CircularProgress size={16} />}
          <IconButton
            size="small"
            onClick={addBookmark}
            sx={{ color: 'text.secondary' }}
            disabled={!command.trim()}
          >
            <BookmarkAddIcon fontSize="small" />
          </IconButton>
          <Button
            variant="contained"
            size="small"
            onClick={() => executeCommand()}
            disabled={isRunning || !command.trim()}
            sx={{ minWidth: 'fit-content' }}
          >
            {isRunning ? <CircularProgress size={16} /> : <RunIcon />}
          </Button>
        </Box>
      </Box>

      {/* Command History Drawer */}
      <Drawer anchor="right" open={historyDrawer} onClose={() => setHistoryDrawer(false)}>
        <Box sx={{ width: 400, p: 2 }}>
          <Typography variant="h6" sx={{ mb: 2 }}>
            Command History
          </Typography>
          <List>
            {history.map((entry) => (
              <ListItem
                key={entry.id}
                button
                onClick={() => {
                  setCommand(entry.command);
                  setHistoryDrawer(false);
                }}
              >
                <ListItemIcon>
                  <HistoryIcon />
                </ListItemIcon>
                <ListItemText
                  primary={entry.command}
                  secondary={
                    <>
                      {new Date(entry.timestamp).toLocaleString()}
                      {entry.success === false && (
                        <Chip size="small" label="Failed" color="error" sx={{ ml: 1 }} />
                      )}
                    </>
                  }
                />
              </ListItem>
            ))}
          </List>
        </Box>
      </Drawer>

      {/* Bookmarks Drawer */}
      <Drawer anchor="right" open={bookmarksDrawer} onClose={() => setBookmarksDrawer(false)}>
        <Box sx={{ width: 400, p: 2 }}>
          <Typography variant="h6" sx={{ mb: 2 }}>
            Terminal Bookmarks
          </Typography>
          <List>
            {bookmarks.map((bookmark) => (
              <ListItem
                key={bookmark.id}
                button
                onClick={() => {
                  setCommand(bookmark.command);
                  setBookmarksDrawer(false);
                }}
              >
                <ListItemIcon>
                  <BookmarkedIcon />
                </ListItemIcon>
                <ListItemText primary={bookmark.name} secondary={bookmark.command} />
              </ListItem>
            ))}
          </List>
        </Box>
      </Drawer>
    </Paper>
  );
};

export default EnhancedTerminal;
