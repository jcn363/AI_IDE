import React, { useState, useMemo, useCallback, useEffect, useDeferredValue } from 'react';
import {
  Box,
  TextField,
  IconButton,
  TreeView,
  TreeItem,
  Typography,
  useTheme,
  CircularProgress,
} from '@mui/material';
import {
  ChevronRight,
  ExpandMore,
  Folder,
  FolderOpen,
  Description,
  Search,
} from '@mui/icons-material';
import { Tree, Virtuoso } from 'react-virtuoso';
import { useDebouncedCallback } from 'use-debounce';
import { invoke } from '@tauri-apps/api/core';

// Types for file tree structure
export interface FileNode {
  id: string;
  path: string;
  name: string;
  isDirectory: boolean;
  isExpanded?: boolean;
  children?: FileNode[];
  isLoading?: boolean;
  depth: number;
}

interface VirtualizedFileTreeProps {
  rootPath: string;
  onFileSelect?: (filePath: string) => void;
  expanded?: Set<string>;
  onExpandedChange?: (expanded: Set<string>) => void;
  searchQuery?: string;
}

// File tree component with virtualization for large codebases
export const VirtualizedFileTree: React.FC<VirtualizedFileTreeProps> = ({
  rootPath,
  onFileSelect,
  expanded = new Set(),
  onExpandedChange,
  searchQuery = '',
}) => {
  const theme = useTheme();
  const [nodes, setNodes] = useState<FileNode[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [nodeMap, setNodeMap] = useState<Map<string, FileNode>>(new Map());
  const [filteredNodes, setFilteredNodes] = useState<FileNode[]>([]);
  const deferredQuery = useDeferredValue(searchQuery);

  // Debounced search callback
  const debouncedUpdateFilter = useDebouncedCallback((query: string) => {
    if (!query.trim()) {
      setFilteredNodes([]);
      return;
    }

    const filtered: FileNode[] = [];
    for (const node of nodes) {
      if (matchesSearch(node, query)) {
        filtered.push(node);
      }
    }
    setFilteredNodes(filtered);
  }, 300);

  // Load root directory contents
  const loadRootDirectory = useCallback(async () => {
    try {
      setIsLoading(true);
      const result = await invoke<FileNode[]>('get_directory_contents', {
        path: rootPath,
        maxDepth: 1,
      });

      if (result) {
        const rootNode: FileNode = {
          id: rootPath,
          path: rootPath,
          name: rootPath.split('/').pop() || '',
          isDirectory: true,
          isExpanded: expanded.has(rootPath),
          children: result,
          depth: 0,
        };

        setNodes(result);
        const newNodeMap = new Map<string, FileNode>();
        populateNodeMap(result, newNodeMap);
        setNodeMap(newNodeMap);
      }
    } catch (error) {
      console.error('Failed to load root directory:', error);
    } finally {
      setIsLoading(false);
    }
  }, [rootPath, expanded]);

  // Populate node map for quick lookups
  const populateNodeMap = useCallback((nodes: FileNode[], map: Map<string, FileNode>) => {
    for (const node of nodes) {
      map.set(node.id, node);
      if (node.children) {
        populateNodeMap(node.children, map);
      }
    }
  }, []);

  // Load directory contents on expand
  const loadDirectoryContents = useCallback(
    async (path: string) => {
      try {
        const node = nodeMap.get(path);
        if (!node || !node.isDirectory) return;

        const result = await invoke<FileNode[]>('get_directory_contents', {
          path,
          maxDepth: 1,
        });

        if (result) {
          // Update node's children
          const updatedNode = { ...node, children: result };
          nodeMap.set(path, updatedNode);

          // Update root nodes array
          const updateNodeChildren = (nodes: FileNode[]): FileNode[] =>
            nodes.map((n) => (n.id === path ? updatedNode : n));

          setNodes((prev) => updateNodeChildren(prev));
        }
      } catch (error) {
        console.error('Failed to load directory contents:', error);
      }
    },
    [nodeMap]
  );

  // Handle node expand/collapse
  const handleToggle = useCallback(
    async (path: string) => {
      const newExpanded = new Set(expanded);
      const node = nodeMap.get(path);

      if (node?.isDirectory) {
        if (expanded.has(path)) {
          newExpanded.delete(path);
          node.isExpanded = false;
        } else {
          newExpanded.add(path);
          node.isExpanded = true;

          // Load children if not already loaded
          if (!node.children || node.children.length === 0) {
            await loadDirectoryContents(path);
          }
        }
      }

      onExpandedChange?.(newExpanded);
    },
    [expanded, nodeMap, onExpandedChange, loadDirectoryContents]
  );

  // Search matching utility
  const matchesSearch = useCallback((node: FileNode, query: string): boolean => {
    const lowerQuery = query.toLowerCase();
    return (
      node.name.toLowerCase().includes(lowerQuery) || node.path.toLowerCase().includes(lowerQuery)
    );
  }, []);

  // Tree item component
  const TreeItemComponent = useCallback(
    ({ node }: { node: FileNode }) => {
      const isExpanded = expanded.has(node.id);
      const hasChildren = node.children && node.children.length > 0;

      return (
        <TreeItem
          nodeId={node.id}
          label={
            <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
              {node.isDirectory ? isExpanded ? <FolderOpen /> : <Folder /> : <Description />}
              <Typography
                variant="body2"
                sx={{ ml: 1, overflow: 'hidden', textOverflow: 'ellipsis' }}
              >
                {node.name}
              </Typography>
            </Box>
          }
          sx={{
            pl: node.depth * theme.spacing(2),
            '& .MuiTreeItem-content': {
              paddingLeft: node.depth * 8,
            },
          }}
        >
          {node.isDirectory &&
            isExpanded &&
            node.children &&
            node.children.map((child) => <TreeItemComponent key={child.id} node={child} />)}
        </TreeItem>
      );
    },
    [expanded, theme]
  );

  // Initialize component
  useEffect(() => {
    loadRootDirectory();
  }, [loadRootDirectory]);

  // Handle search filtering
  useEffect(() => {
    debouncedUpdateFilter(deferredQuery);
  }, [deferredQuery, debouncedUpdateFilter]);

  const displayNodes = deferredQuery ? filteredNodes : nodes;

  if (isLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Search input */}
      <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <TextField
          fullWidth
          variant="outlined"
          size="small"
          placeholder="Search files..."
          value={searchQuery}
          onChange={(e) => onFileSelect?.(e.target.value)} // Note: This might need to be separate handler
          InputProps={{
            startAdornment: <Search sx={{ mr: 1, color: 'text.secondary' }} />,
          }}
        />
      </Box>

      {/* Virtualized tree */}
      <Box sx={{ flex: 1 }}>
        <Tree
          treeTable
          data={displayNodes}
          totalCount={displayNodes.length}
          itemContent={(index, node) => <TreeItemComponent key={node.id} node={node} />}
          onExpandedChange={onExpandedChange}
          components={{
            Table: ({ children }) => <Box sx={{ p: 1 }}>{children}</Box>,
            TableRow: ({ children }) => <Box sx={{ py: 0.5 }}>{children}</Box>,
            TableCell: ({ children, onClick }) => (
              <Box
                onClick={() => {
                  if (onClick) onClick();
                  if (!node.isDirectory && onFileSelect) {
                    onFileSelect(node.path);
                  }
                }}
                sx={{
                  cursor: node.isDirectory ? 'pointer' : 'default',
                  '&:hover': {
                    backgroundColor: 'action.hover',
                  },
                }}
              >
                {children}
              </Box>
            ),
          }}
        >
          <Virtuoso
            totalCount={displayNodes.length}
            itemContent={(index) => {
              const node = displayNodes[index];
              return <TreeItemComponent key={node.id} node={node} />;
            }}
          />
        </Tree>
      </Box>
    </Box>
  );
};

export default VirtualizedFileTree;
