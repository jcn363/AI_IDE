import React from 'react';
import {
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Box,
  IconButton,
  CircularProgress,
  Typography,
} from '@mui/material';
import {
  Code as CodeIcon,
  Search as SearchIcon,
  Book as BookIcon,
  ChatBubble as ChatBubbleIcon,
  Build as BuildIcon,
  Lightbulb as LightbulbIcon,
  Close as CloseIcon,
} from '@mui/icons-material';
import { useAIAssistant } from '../hooks/useAIAssistant';

interface AIContextMenuProps {
  anchorEl: HTMLElement | null;
  onClose: () => void;
  position?: { x: number; y: number };
  selectedText?: string;
  filePath?: string;
  onGenerateCode?: (code: string) => void;
  children?: React.ReactNode;
}

const AIContextMenu: React.FC<AIContextMenuProps> = ({
  anchorEl,
  onClose,
  position,
  selectedText,
  filePath = 'current_file.rs',
  onGenerateCode,
  children,
}) => {
  const {
    analyzeCurrentFile,
    generateTests,
    generateDocumentation,
    explainCode,
    refactorCode,
    isGenerating,
  } = useAIAssistant();

  const handleAction = async (action: () => Promise<any>) => {
    try {
      const result = await action();
      if (result?.content && onGenerateCode) {
        onGenerateCode(result.content);
      }
    } catch (error) {
      console.error('AI action failed:', error);
    } finally {
      onClose();
    }
  };

  const menuItems = [
    {
      label: 'Analyze Code',
      icon: <SearchIcon fontSize="small" />,
      action: () => analyzeCurrentFile(selectedText || '', filePath),
      disabled: isGenerating,
    },
    {
      label: 'Generate Tests',
      icon: <CodeIcon fontSize="small" />,
      action: () => generateTests(selectedText || '', filePath),
      disabled: isGenerating || !selectedText,
    },
    {
      label: 'Generate Documentation',
      icon: <BookIcon fontSize="small" />,
      action: () => generateDocumentation(selectedText || '', filePath),
      disabled: isGenerating || !selectedText,
    },
    {
      label: 'Explain Code',
      icon: <ChatBubbleIcon fontSize="small" />,
      action: () => explainCode(selectedText || ''),
      disabled: isGenerating || !selectedText,
    },
    {
      label: 'Refactor Code',
      icon: <BuildIcon fontSize="small" />,
      action: () => refactorCode(selectedText || '', filePath),
      disabled: isGenerating || !selectedText,
    },
  ];

  // If children are provided, render them with the menu as a portal
  if (React.Children.count(children) > 0) {
    return (
      <>
        {children}
        <Menu
          anchorEl={anchorEl}
          open={Boolean(anchorEl)}
          onClose={onClose}
          anchorReference={position ? 'anchorPosition' : 'anchorEl'}
          anchorPosition={position ? { top: position.y, left: position.x } : undefined}
          transformOrigin={{
            vertical: 'top',
            horizontal: 'left',
          }}
          PaperProps={{
            sx: {
              width: 220,
              maxWidth: '100%',
              mt: 1,
            },
          }}
        >
          <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center' }}>
            <LightbulbIcon color="primary" fontSize="small" sx={{ mr: 1 }} />
            <Typography variant="subtitle2" color="primary">
              AI Assistant
            </Typography>
            <IconButton size="small" onClick={onClose} sx={{ ml: 'auto' }} disabled={isGenerating}>
              <CloseIcon fontSize="small" />
            </IconButton>
          </Box>
          <Divider />
          {menuItems.map((item, index) => (
            <MenuItem
              key={index}
              onClick={() => handleAction(item.action)}
              disabled={item.disabled}
              dense
            >
              <ListItemIcon>{item.icon}</ListItemIcon>
              <ListItemText>{item.label}</ListItemText>
              {isGenerating && item.disabled && <CircularProgress size={16} sx={{ ml: 1 }} />}
            </MenuItem>
          ))}
        </Menu>
      </>
    );
  }

  // Original behavior without children
  return (
    <Menu
      anchorEl={anchorEl}
      open={Boolean(anchorEl)}
      onClose={onClose}
      anchorReference={position ? 'anchorPosition' : 'anchorEl'}
      anchorPosition={position ? { top: position.y, left: position.x } : undefined}
      transformOrigin={{
        vertical: 'top',
        horizontal: 'left',
      }}
      PaperProps={{
        sx: {
          width: 220,
          maxWidth: '100%',
          mt: 1,
        },
      }}
    >
      <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center' }}>
        <LightbulbIcon color="primary" fontSize="small" sx={{ mr: 1 }} />
        <Typography variant="subtitle2" color="primary">
          AI Assistant
        </Typography>
        <IconButton size="small" onClick={onClose} sx={{ ml: 'auto' }} disabled={isGenerating}>
          <CloseIcon fontSize="small" />
        </IconButton>
      </Box>
      <Divider />
      {menuItems.map((item, index) => (
        <MenuItem
          key={index}
          onClick={() => handleAction(item.action)}
          disabled={item.disabled}
          dense
        >
          <ListItemIcon>{item.icon}</ListItemIcon>
          <ListItemText>{item.label}</ListItemText>
          {isGenerating && item.disabled && <CircularProgress size={16} sx={{ ml: 1 }} />}
        </MenuItem>
      ))}
    </Menu>
  );
};

export default AIContextMenu;
