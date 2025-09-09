import React, { useCallback } from 'react';
import { Box, IconButton, Tooltip } from '@mui/material';
import {
  CallSplit as SplitHorizontalIcon,
  Remove as RemoveIcon,
  Save as SaveIcon,
  RestoreFromTrash as LoadIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/tauri';

// Split view related interfaces matching Rust types
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

interface WordBoundary {
  start: CursorPosition;
  end: CursorPosition;
}

interface PanelTab {
  id: string;
  title: string;
  content_id: string;
  is_modified: boolean;
  is_pinned: boolean;
}

interface PanelInfo {
  id: string;
  content_type: 'Editor' | 'Terminal' | 'FileExplorer' | 'Output' | 'Debug' | 'Git' | 'Cargo' | 'Documentation';
  title?: string;
  is_active: boolean;
  tabs: PanelTab[];
}

interface SplitConfig {
  id: string;
  orientation: 'Horizontal' | 'Vertical';
  size: number;
  children: PanelNode[];
}

interface PanelNode {
  Split?: SplitConfig;
  Leaf?: PanelInfo;
}

interface LayoutConfig {
  root_panel: PanelNode;
  focused_panel: string;
  layout_version: number;
  last_updated: number;
}

interface SplitViewLayoutProps {
  layoutConfig: LayoutConfig;
  onLayoutChange?: (layout: LayoutConfig) => void;
  renderPanel: (panelInfo: PanelInfo, isFocused: boolean) => React.ReactNode;
}

const SplitViewLayout: React.FC<SplitViewLayoutProps> = ({
  layoutConfig,
  onLayoutChange,
  renderPanel,
}) => {
  const handleSplitPanel = async (
    panelId: string,
    orientation: 'Horizontal' | 'Vertical',
    size: number
  ) => {
    try {
      const rustOrientation = orientation === 'Horizontal' ? 'Vertical' : 'Horizontal'; // Monaco layout flip
      await invoke('split_panel', {
        panelId,
        orientation: rustOrientation,
        sizeRatio: size,
      });

      // Refresh layout from backend
      loadLayout();
    } catch (error) {
      console.error('Failed to split panel:', error);
    }
  };

  const handleClosePanel = async (panelId: string) => {
    try {
      await invoke('close_panel', { panelId });
      loadLayout();
    } catch (error) {
      console.error('Failed to close panel:', error);
    }
  };

  const loadLayout = useCallback(async () => {
    try {
      const result = await invoke('get_layout');
      if (result.status === 'success' && onLayoutChange) {
        onLayoutChange(result.layout);
      }
    } catch (error) {
      console.error('Failed to load layout:', error);
    }
  }, [onLayoutChange]);

  const saveLayout = async (name: string) => {
    try {
      await invoke('save_layout', { name });
      console.log(`Layout saved as: ${name}`);
    } catch (error) {
      console.error('Failed to save layout:', error);
    }
  };

  const loadSavedLayout = async (name: string) => {
    try {
      await invoke('load_layout', { name });
      loadLayout();
    } catch (error) {
      console.error('Failed to load layout:', error);
    }
  };

  // Recursive rendering function
  const renderPanelNode = useCallback(
    (node: PanelNode): React.ReactNode => {
      if (node.Leaf) {
        const isFocused = layoutConfig.focused_panel === node.Leaf.id;
        return (
          <Box
            key={node.Leaf.id}
            sx={{
              display: 'flex',
              flexDirection: 'column',
              height: '100%',
              border: isFocused ? '2px solid' : '1px solid',
              borderColor: isFocused ? 'primary.main' : 'divider',
              position: 'relative',
              borderRadius: 1,
            }}
          >
            {/* Panel header */}
            <Box
              sx={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                p: 1,
                borderBottom: '1px solid',
                borderColor: 'divider',
                backgroundColor: 'background.paper',
              }}
            >
              <span style={{ fontWeight: 'bold' }}>{node.Leaf.title || node.Leaf.content_type}</span>
              <Box sx={{ display: 'flex', gap: 0.5 }}>
                <Tooltip title={`Split ${node.Leaf.id} Horizontally`}>
                  <IconButton
                    size="small"
                    onClick={() => handleSplitPanel(node.Leaf!.id, 'Horizontal', 0.5)}
                  >
                    <SplitHorizontalIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
                <Tooltip title={`Split ${node.Leaf.id} Vertically`}>
                  <IconButton
                    size="small"
                    onClick={() => handleSplitPanel(node.Leaf!.id, 'Vertical', 0.5)}
                  >
                    <SplitHorizontalIcon sx={{ transform: 'rotate(90deg)' }} fontSize="small" />
                  </IconButton>
                </Tooltip>
                <Tooltip title={`Close ${node.Leaf.id}`}>
                  <IconButton
                    size="small"
                    onClick={() => handleClosePanel(node.Leaf!.id)}
                  >
                    <RemoveIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
              </Box>
            </Box>

            {/* Panel content */}
            <Box sx={{ flex: 1, overflow: 'auto' }}>
              {renderPanel(node.Leaf, isFocused)}
            </Box>
          </Box>
        );
      }

      if (node.Split) {
        const orientation = node.Split.orientation;
        const flexDirection = orientation === 'Horizontal' ? 'row' : 'column';
        const child1Size = node.Split.size * 100;
        const child2Size = (1 - node.Split.size) * 100;

        return (
          <Box
            key={node.Split.id}
            sx={{
              display: 'flex',
              flexDirection,
              height: '100%',
              width: '100%',
              gap: 1,
            }}
          >
            {/* First child */}
            <Box
              sx={{
                flex: `${child1Size} 1 ${child1Size}%`,
                minHeight: 0,
              }}
            >
              {renderPanelNode(node.Split.children[0])}
            </Box>

            {/* Resize handle */}
            <Box
              sx={{
                width: orientation === 'Horizontal' ? '2px' : '2px',
                height: orientation === 'Vertical' ? '2px' : '2px',
                backgroundColor: 'primary.main',
                cursor: orientation === 'Horizontal' ? 'ew-resize' : 'ns-resize',
                '&:hover': {
                  backgroundColor: 'primary.light',
                },
              }}
            />

            {/* Second child */}
            <Box
              sx={{
                flex: `${child2Size} 1 ${child2Size}%`,
                minHeight: 0,
              }}
            >
              {renderPanelNode(node.Split.children[1])}
            </Box>
          </Box>
        );
      }

      return null;
    },
    [layoutConfig.focused_panel, renderPanel]
  );

  return (
    <Box
      sx={{
        height: '100vh',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {/* Top toolbar */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          p: 1,
          borderBottom: '1px solid',
          borderColor: 'divider',
          backgroundColor: 'background.paper',
        }}
      >
        <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
          <Tooltip title="Save Layout">
            <IconButton
              size="small"
              onClick={() => saveLayout('current')}
            >
              <SaveIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="Load Default Layout">
            <IconButton
              size="small"
              onClick={() => loadSavedLayout('default')}
            >
              <LoadIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </Box>
        <Box>
          <span style={{ color: 'text.secondary', fontSize: '0.875rem' }}>
            Version: {layoutConfig.layout_version}
          </span>
        </Box>
      </Box>

      {/* Main layout area */}
      <Box sx={{ flex: 1, p: 1, overflow: 'hidden' }}>
        {renderPanelNode(layoutConfig.root_panel)}
      </Box>
    </Box>
  );
};

export default SplitViewLayout;