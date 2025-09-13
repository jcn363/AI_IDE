import React, { useState, useRef } from 'react';
import {
  Box,
  Button,
  Menu,
  MenuItem,
  IconButton,
  Tooltip,
  Typography,
  Divider,
} from '@mui/material';
import {
  ExpandLess as ExpandLessIcon,
  ExpandMore as ExpandMoreIcon,
  SelectAll as SelectAllIcon,
  RemoveCircleOutline as RemoveCursorIcon,
  Add as AddIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

// Multi-cursor related interfaces matching Rust types
interface CursorPosition {
  line: number;
  column: number;
}

interface MultiCursorState {
  primary_cursor: CursorPosition;
  secondary_cursors: CursorPosition[];
  document_version?: string;
  last_updated: number;
}

interface FindMatchConfig {
  query: string;
  case_sensitive: boolean;
  whole_word: boolean;
  regex: boolean;
}

interface MultiCursorMenuProps {
  documentUri: string;
  editor?: any; // Monaco editor instance
}

const MultiCursorMenu: React.FC<MultiCursorMenuProps> = ({ documentUri, editor }) => {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [cursorState, setCursorState] = useState<MultiCursorState | null>(null);
  const [loading, setLoading] = useState(false);

  const buttonRef = useRef<HTMLButtonElement>(null);

  // Menu actions
  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
    loadCursorState();
  };

  const handleMenuClose = () => {
    setAnchorEl(null);
  };

  const loadCursorState = async () => {
    try {
      setLoading(true);
      const result: { status: string; state?: MultiCursorState } = await invoke(
        'get_cursor_state',
        { documentUri }
      );

      if (result.status === 'success') {
        setCursorState(result.state || null);
      }
    } catch (error) {
      console.error('Failed to load cursor state:', error);
    } finally {
      setLoading(false);
    }
  };

  const addCursorAtPosition = async (line: number, column: number, primary: boolean) => {
    try {
      await invoke('add_cursor_at_position', {
        documentUri,
        position: { line, column },
        primary,
      });

      // Update Monaco editor if available
      if (editor) {
        const model = editor.getModel();
        if (model) {
          const position = { lineNumber: line, column };
          if (primary) {
            editor.setPosition(position);
          } else {
            // Add secondary cursor
            editor.setSelections([
              editor.getSelection(),
              monaco.Range.fromPositions(position, position),
            ]);
          }
        }
      }

      loadCursorState(); // Refresh UI
    } catch (error) {
      console.error('Failed to add cursor:', error);
    }
  };

  const removeSecondaryCursors = async () => {
    try {
      await invoke('remove_all_secondary_cursors', { documentUri });

      if (editor) {
        // Reset to single cursor
        const selection = editor.getSelection();
        editor.setSelections([selection]);
      }

      loadCursorState(); // Refresh UI
    } catch (error) {
      console.error('Failed to remove cursors:', error);
    }
  };

  const findAllOccurrences = async (
    query: string = '',
    caseSensitive: boolean = false,
    wholeWord: boolean = false
  ) => {
    try {
      const config: FindMatchConfig = {
        query: query || editor?.getSelection()?.getText() || '',
        case_sensitive: caseSensitive,
        whole_word: wholeWord,
        regex: false,
      };

      const result: { status: string; positions?: CursorPosition[] } = await invoke(
        'find_all_occurrences',
        { documentUri, config }
      );

      if (result.status === 'success' && result.positions && editor) {
        // Add cursors at all positions
        const selections = result.positions.map((pos) => {
          const startPos = { lineNumber: pos.line, column: pos.column };
          const endPos = { lineNumber: pos.line, column: pos.column + query.length };
          return monaco.Range.fromPositions(startPos, endPos);
        });

        if (selections.length > 0) {
          editor.setSelections(selections);
        }
      }

      loadCursorState(); // Refresh UI
    } catch (error) {
      console.error('Failed to find occurrences:', error);
    }
  };

  const addCursorsOnLineEnds = async (startLine: number, endLine: number) => {
    try {
      const result: { status: string; positions?: CursorPosition[] } = await invoke(
        'add_cursors_on_line_ends',
        { documentUri, startLine, endLine }
      );

      if (result.status === 'success' && result.positions && editor) {
        const selections = result.positions.map((pos) => {
          const position = { lineNumber: pos.line, column: pos.column };
          return monaco.Range.fromPositions(position, position);
        });

        if (selections.length > 0) {
          editor.setSelections(selections);
        }
      }

      loadCursorState(); // Refresh UI
    } catch (error) {
      console.error('Failed to add cursors on line ends:', error);
    }
  };

  return (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
      <Tooltip title="Multi-cursor operations">
        <IconButton
          ref={buttonRef}
          onClick={handleMenuOpen}
          size="small"
          sx={{
            color: cursorState?.secondary_cursors?.length ? 'primary.main' : 'text.secondary',
          }}
        >
          <AddIcon fontSize="small" />
        </IconButton>
      </Tooltip>

      {cursorState?.secondary_cursors?.length && (
        <Typography variant="caption" sx={{ minWidth: 30 }}>
          {cursorState.secondary_cursors.length + 1}
        </Typography>
      )}

      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={handleMenuClose}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'right',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
      >
        <MenuItem onClick={() => addCursorAtPosition(1, 1, false)}>
          <AddIcon fontSize="small" sx={{ mr: 1 }} />
          Add Cursor at (1,1)
        </MenuItem>

        <MenuItem onClick={() => findAllOccurrences()}>
          <SelectAllIcon fontSize="small" sx={{ mr: 1 }} />
          Select All Occurrences
        </MenuItem>

        <MenuItem onClick={() => addCursorsOnLineEnds(1, 10)}>
          <ExpandLessIcon fontSize="small" sx={{ mr: 1 }} />
          Cursors on Line Ends (Lines 1-10)
        </MenuItem>

        {cursorState?.secondary_cursors?.length && <Divider sx={{ my: 1 }} />}

        {cursorState?.secondary_cursors?.length && (
          <MenuItem onClick={removeSecondaryCursors}>
            <RemoveCursorIcon fontSize="small" sx={{ mr: 1, color: 'error.main' }} />
            Remove All Secondary Cursors ({cursorState.secondary_cursors.length})
          </MenuItem>
        )}
      </Menu>
    </Box>
  );
};

export default MultiCursorMenu;
