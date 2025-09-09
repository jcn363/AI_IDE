import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useAppDispatch, useAppSelector } from '../../store';
import { 
  setCurrentFile, 
  updateFileContent,
  loadFileTree,
  selectEditor,
  FileNode
} from '../../store/slices/editorSlice';
import {
  Box,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Collapse,
  Typography,
  IconButton,
  Tooltip,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  CircularProgress,
  Menu,
  MenuItem,
  ListItemSecondaryAction,
  Snackbar,
  Alert,
} from '@mui/material';
import {
  Folder as FolderIcon,
  FolderOpen as FolderOpenIcon,
  InsertDriveFileOutlined as FileIcon,
  KeyboardArrowRight,
  KeyboardArrowDown,
  CreateNewFolderOutlined as NewFolderIcon,
  NoteAddOutlined as NewFileIcon,
  Refresh as RefreshIcon,
  MoreVert as MoreVertIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Save as SaveIcon,
} from '@mui/icons-material';

interface FileExplorerProps {
  rootPath?: string;
  onFileSelect?: (path: string) => void;
}

export function FileExplorer({ rootPath = '/', onFileSelect }: FileExplorerProps) {
  const dispatch = useAppDispatch();
  const { 
    currentFile, 
    fileTree, 
    isLoading, 
    error: fileError 
  } = useAppSelector(selectEditor);
  
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [contextMenu, setContextMenu] = useState<{
    node: FileNode;
    mouseX: number;
    mouseY: number;
  } | null>(null);
  const [dialogOpen, setDialogOpen] = useState<{
    type: 'file' | 'folder' | 'rename'; 
    parentPath: string; 
    node?: FileNode;
  } | null>(null);
  const [newName, setNewName] = useState('');
  const [snackbar, setSnackbar] = useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info' | 'warning';
  }>({ open: false, message: '', severity: 'info' });

  // Load file tree when component mounts
  useEffect(() => {
    dispatch(loadFileTree(rootPath));
  }, [dispatch, rootPath]);
  
  // Handle file tree loading errors
  useEffect(() => {
    if (fileError) {
      setSnackbar({
        open: true,
        message: fileError,
        severity: 'error',
      });
    }
  }, [fileError]);
  
  // Helper function to find a node by path
  const findNodeByPath = useCallback((node: FileNode, path: string): FileNode | null => {
    if (node.path === path) return node;
    if (node.children) {
      for (const child of node.children) {
        const found = findNodeByPath(child, path);
        if (found) return found;
      }
    }
    return null;
  }, []);

  const handleToggle = (path: string) => {
    setExpanded((prev) => ({
      ...prev,
      [path]: !prev[path],
    }));
  };

  const handleFileClick = (filePath: string) => {
    dispatch(setCurrentFile(filePath));
    if (onFileSelect) {
      onFileSelect(filePath);
    }
  };

  const handleRefresh = () => {
    dispatch(loadFileTree(rootPath));
  };

  const handleCreateNew = (type: 'file' | 'folder' | 'rename', parentPath: string, node?: FileNode) => {
    setDialogOpen({ type, parentPath, node });
    setNewName(node ? node.name : '');
  };

  const handleContextMenu = (event: React.MouseEvent, node: FileNode) => {
    event.preventDefault();
    event.stopPropagation();
    
    setContextMenu({
      node,
      mouseX: event.clientX - 2,
      mouseY: event.clientY - 4,
    });
  };

  const handleCloseContextMenu = () => {
    setContextMenu(null);
  };

  const handleRename = (node: FileNode) => {
    handleCreateNew('rename', node.path, node);
    handleCloseContextMenu();
  };

  const handleDelete = (node: FileNode) => {
    // In a real app, this would delete the file/directory
    console.log(`Deleting ${node.path}`);
    setSnackbar({
      open: true,
      message: `Deleted ${node.name}`,
      severity: 'info',
    });
    handleCloseContextMenu();
    // Refresh the file tree
    handleRefresh();
  };

  const handleDialogConfirm = () => {
    if (!dialogOpen || !newName.trim()) return;
    
    const { type, parentPath, node } = dialogOpen;
    
    if (type === 'rename' && node) {
      // Handle rename
      console.log(`Renaming ${node.path} to ${parentPath}/${newName}`);
      setSnackbar({
        open: true,
        message: `Renamed ${node.name} to ${newName}`,
        severity: 'success',
      });
    } else {
      // Handle create new file/folder
      console.log(`Creating new ${type} at ${parentPath}/${newName}`);
      setSnackbar({
        open: true,
        message: `Created new ${type}: ${newName}`,
        severity: 'success',
      });
    }
    
    // Close the dialog and refresh the file tree
    setDialogOpen(null);
    handleRefresh();
  };
  
  const handleCloseSnackbar = () => {
    setSnackbar(prev => ({ ...prev, open: false }));
  };

  const renderFileTree = (node: FileNode, level = 0) => {
    const isExpanded = expanded[node.path] ?? false;
    const isFile = node.type === 'file';
    const hasChildren = node.children && node.children.length > 0;
    const isSelected = currentFile === node.path;

    return (
      <React.Fragment key={node.path}>
        <ListItemButton
          sx={{
            pl: 2 + level * 2,
            backgroundColor: isSelected ? 'rgba(144, 202, 249, 0.16)' : 'transparent',
            '&:hover': {
              backgroundColor: isSelected ? 'rgba(144, 202, 249, 0.24)' : 'rgba(255, 255, 255, 0.04)',
            },
          }}
          onClick={() => (isFile ? handleFileClick(node.path) : handleToggle(node.path))}
          onContextMenu={(e) => handleContextMenu(e, node)}
          dense
        >
          {!isFile ? (
            <IconButton
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                handleToggle(node.path);
              }}
              sx={{ mr: 0.5 }}
            >
              {isExpanded ? (
                <KeyboardArrowDown fontSize="small" />
              ) : (
                <KeyboardArrowRight fontSize="small" />
              )}
            </IconButton>
          ) : (
            <Box sx={{ width: 32, display: 'flex', justifyContent: 'center' }} />
          )}
          
          <ListItemIcon sx={{ minWidth: 36 }}>
            {isFile ? (
              <FileIcon fontSize="small" />
            ) : isExpanded ? (
              <FolderOpenIcon fontSize="small" />
            ) : (
              <FolderIcon fontSize="small" />
            )}
          </ListItemIcon>
          
          <ListItemText
            primary={node.name}
            primaryTypographyProps={{
              noWrap: true,
              title: node.name,
              sx: {
                fontSize: '0.875rem',
                fontFamily: 'monospace',
              },
            }}
          />
          
          {node.lastModified && (
            <Typography 
              variant="caption" 
              color="text.secondary"
              sx={{
                ml: 1,
                fontSize: '0.7rem',
                whiteSpace: 'nowrap',
              }}
            >
              {new Date(node.lastModified).toLocaleTimeString()}
            </Typography>
          )}
          
          <ListItemSecondaryAction>
            <IconButton
              edge="end"
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                handleContextMenu(e, node);
              }}
            >
              <MoreVertIcon fontSize="small" />
            </IconButton>
          </ListItemSecondaryAction>
        </ListItemButton>
        
        {!isFile && (
          <Collapse in={isExpanded} timeout="auto" unmountOnExit>
            <List component="div" disablePadding>
              {node.children?.map((child) => renderFileTree(child, level + 1))}
            </List>
          </Collapse>
        )}
      </React.Fragment>
    );
  };

  if (isLoading && !fileTree) {
    return (
      <Box p={2} display="flex" justifyContent="center" alignItems="center" height="100%">
        <CircularProgress size={24} />
      </Box>
    );
  }
  
  if (!fileTree) {
    return (
      <Box p={2} textAlign="center">
        <Typography variant="body2" color="text.secondary">
          No files found
        </Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Box
        sx={{
          p: 3,
          mb: 4,
          maxWidth: 800,
          borderBottom: '1px solid',
          borderColor: 'divider',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <Typography variant="h6" gutterBottom>
          File Explorer
        </Typography>
        <Box>
          <Tooltip title="New File">
            <IconButton
              size="small"
              onClick={() => handleCreateNew('file', rootPath)}
              sx={{ p: 0.5 }}
            >
              <NewFileIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="New Folder">
            <IconButton
              size="small"
              onClick={() => handleCreateNew('folder', rootPath)}
              sx={{ p: 0.5 }}
            >
              <NewFolderIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="Refresh">
            <IconButton size="small" onClick={handleRefresh} sx={{ p: 0.5 }}>
              <RefreshIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>
      
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        <List dense disablePadding>
          {fileTree ? (
            renderFileTree(fileTree)
          ) : (
            <Typography variant="body2" color="textSecondary" sx={{ p: 2 }}>
              No files found
            </Typography>
          )}
        </List>
      </Box>

      {/* Context Menu */}
      {contextMenu && (
        <Menu
          open={!!contextMenu}
          onClose={handleCloseContextMenu}
          anchorReference="anchorPosition"
          anchorPosition={{
            top: contextMenu.mouseY,
            left: contextMenu.mouseX
          }}
        >
          <MenuItem onClick={() => handleRename(contextMenu.node)}>
            <ListItemIcon>
              <EditIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText>Rename</ListItemText>
          </MenuItem>
          <MenuItem 
            onClick={() => handleDelete(contextMenu.node)}
            sx={{ color: 'error.main' }}
          >
            <ListItemIcon sx={{ color: 'error.main' }}>
              <DeleteIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText>Delete</ListItemText>
          </MenuItem>
        </Menu>
      )}

      {/* Create/Rename Dialog */}
      <Dialog 
        open={!!dialogOpen} 
        onClose={() => setDialogOpen(null)} 
        maxWidth="sm" 
        fullWidth
      >
        <DialogTitle>
          {dialogOpen?.type === 'rename' 
            ? 'Rename Item' 
            : `New ${dialogOpen?.type === 'file' ? 'File' : 'Folder'}`}
        </DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label={
              dialogOpen?.type === 'rename' 
                ? 'New name' 
                : `${dialogOpen?.type === 'file' ? 'File' : 'Folder'} name`
            }
            type="text"
            fullWidth
            variant="outlined"
            value={newName}
            onChange={(e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
              const val = (e.target as any).value as string;
              setNewName(val);
            }}
            onKeyPress={(e) => {
              if (e.key === 'Enter') {
                handleDialogConfirm();
              }
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDialogOpen(null)}>Cancel</Button>
          <Button 
            onClick={handleDialogConfirm} 
            color="primary" 
            variant="contained"
            disabled={!newName.trim()}
          >
            {dialogOpen?.type === 'rename' ? 'Rename' : 'Create'}
          </Button>
        </DialogActions>
      </Dialog>
      
      {/* Snackbar for notifications */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={3000}
        onClose={handleCloseSnackbar}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
      >
        <Alert 
          onClose={handleCloseSnackbar} 
          severity={snackbar.severity}
          variant="filled"
        >
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Box>
  );
}

export default FileExplorer;
