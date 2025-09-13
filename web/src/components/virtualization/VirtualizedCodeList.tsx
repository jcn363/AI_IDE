import React, { useState, useMemo, useCallback } from 'react';
import {
  Box,
  ListItem,
  ListItemText,
  Typography,
  Paper,
  IconButton,
  useTheme,
  Skeleton,
  Chip,
  Tooltip,
} from '@mui/material';
import { Search, Code, Description, Folder, Star, StarBorder } from '@mui/icons-material';
import { FixedSizeList as List, ListChildComponentProps } from 'react-window';
import { memo } from 'react';
import { useDeferredValue, useTransition } from 'react';

// Types for search results
export interface SearchResult {
  id: string;
  path: string;
  fileName: string;
  lineNumber: number;
  columnNumber: number;
  content: string;
  context?: string;
  score: number;
  language?: string;
  highlightedMatches?: Array<{
    start: number;
    end: number;
    text: string;
  }>;
  gitStatus?: 'modified' | 'added' | 'deleted' | 'untracked';
}

export interface SymbolItem {
  id: string;
  name: string;
  type: 'function' | 'class' | 'variable' | 'method' | 'property' | 'constant';
  path: string;
  lineNumber: number;
  language: string;
  scope?: string;
  documentation?: string;
}

interface VirtualizedCodeListProps {
  items: (SearchResult | SymbolItem)[];
  onItemClick?: (item: SearchResult | SymbolItem) => void;
  onItemHover?: (item: SearchResult | SymbolItem) => void;
  listHeight: number;
  itemHeight?: number;
  isLoading?: boolean;
  showRelevanceScore?: boolean;
  showGitStatus?: boolean;
  showLineNumbers?: boolean;
}

// Determine if item is a search result
const isSearchResult = (item: SearchResult | SymbolItem): item is SearchResult => {
  return 'content' in item && 'score' in item;
};

// Render individual list item
const CodeListItem: React.FC<{
  item: SearchResult | SymbolItem;
  onClick?: (item: SearchResult | SymbolItem) => void;
  onHover?: (item: SearchResult | SymbolItem) => void;
  showRelevanceScore?: boolean;
  showGitStatus?: boolean;
  showLineNumbers?: boolean;
}> = memo(({ item, onClick, onHover, showRelevanceScore, showGitStatus, showLineNumbers }) => {
  const theme = useTheme();
  const isResult = isSearchResult(item);

  const getItemIcon = () => {
    if (isResult) {
      return <Search fontSize="small" />;
    }
    switch (item.type) {
      case 'function':
        return <Code fontSize="small" />;
      case 'class':
        return <Code fontSize="small" />;
      case 'method':
        return <Code fontSize="small" />;
      case 'property':
        return <Code fontSize="small" />;
      default:
        return <Description fontSize="small" />;
    }
  };

  const getGitStatusColor = (status?: string) => {
    switch (status) {
      case 'modified':
        return theme.palette.warning.main;
      case 'added':
        return theme.palette.success.main;
      case 'deleted':
        return theme.palette.error.main;
      case 'untracked':
        return theme.palette.info.main;
      default:
        return 'transparent';
    }
  };

  const getTypeColor = (type?: string) => {
    switch (type) {
      case 'function':
        return theme.palette.primary.main;
      case 'class':
        return theme.palette.secondary.main;
      case 'method':
        return theme.palette.success.main;
      case 'property':
        return theme.palette.warning.main;
      case 'variable':
        return theme.palette.info.main;
      default:
        return theme.palette.text.primary;
    }
  };

  return (
    <ListItem
      component={Paper}
      elevation={0}
      sx={{
        borderRadius: 1,
        mb: 0.5,
        cursor: 'pointer',
        border: `1px solid ${theme.palette.divider}`,
        '&:hover': {
          backgroundColor: theme.palette.action.hover,
          borderColor: theme.palette.action.focus,
        },
        overflow: 'hidden',
      }}
      onClick={() => onClick?.(item)}
      onMouseEnter={() => onHover?.(item)}
    >
      <Box sx={{ display: 'flex', width: '100%', alignItems: 'flex-start', p: 1 }}>
        {/* Icon */}
        <Box sx={{ mr: 1, mt: 0.5 }}>{getItemIcon()}</Box>

        {/* Content */}
        <Box sx={{ flex: 1, minWidth: 0 }}>
          {/* Header */}
          <Box sx={{ display: 'flex', alignItems: 'center', mb: 0.5 }}>
            <Typography variant="subtitle2" sx={{ flex: 1, minWidth: 0 }}>
              {item.path.split('/').pop()}
            </Typography>
            {showLineNumbers && 'lineNumber' in item && (
              <Chip size="small" label={`Line ${item.lineNumber}`} sx={{ ml: 1, height: 18 }} />
            )}
            {!isResult && item.type && (
              <Chip
                size="small"
                label={item.type}
                sx={{
                  ml: 1,
                  height: 18,
                  backgroundColor: getTypeColor(item.type),
                  color: theme.palette.getContrastText(getTypeColor(item.type)),
                }}
              />
            )}
            {showRelevanceScore && isResult && (
              <Tooltip title={`Relevance score: ${item.score.toFixed(2)}`}>
                <IconButton size="small" sx={{ ml: 0.5 }}>
                  {item.score > 0.8 ? <Star /> : <StarBorder />}
                </IconButton>
              </Tooltip>
            )}
          </Box>

          {/* Path */}
          <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 0.5 }}>
            {item.path}
          </Typography>

          {/* Content/Description */}
          {isResult ? (
            <Box>
              {item.context && (
                <Typography variant="body2" sx={{ mb: 0.5, opacity: 0.7 }}>
                  {item.context}
                </Typography>
              )}
              <Box
                sx={{
                  p: 1,
                  backgroundColor: theme.palette.grey[50],
                  borderRadius: 1,
                  fontFamily: 'monospace',
                  fontSize: '0.875rem',
                  whiteSpace: 'pre-wrap',
                  maxHeight: 120,
                  overflow: 'hidden',
                  wordBreak: 'break-word',
                }}
              >
                {item.content}
              </Box>
            </Box>
          ) : (
            <Typography variant="body2" sx={{ opacity: 0.8 }}>
              {item.name}
              {item.scope && ` (${item.scope})`}
            </Typography>
          )}
        </Box>

        {/* Git status indicator */}
        {showGitStatus && item.gitStatus && (
          <Box
            sx={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              backgroundColor: getGitStatusColor(item.gitStatus),
              ml: 1,
              flexShrink: 0,
              mt: 2,
            }}
          />
        )}
      </Box>
    </ListItem>
  );
});

CodeListItem.displayName = 'CodeListItem';

// Loading skeleton component
const CodeListSkeleton: React.FC<{ height?: number }> = ({ height = 60 }) => (
  <Paper sx={{ p: 2, mb: 0.5, borderRadius: 1 }}>
    <Skeleton variant="text" width="40%" height={20} sx={{ mb: 1 }} />
    <Skeleton variant="text" width="100%" height={16} sx={{ mb: 0.5 }} />
    <Skeleton variant="rectangular" width="100%" height={height - 40} />
  </Paper>
);

// Main component
export const VirtualizedCodeList: React.FC<VirtualizedCodeListProps> = ({
  items,
  onItemClick,
  onItemHover,
  listHeight,
  itemHeight = 80,
  isLoading = false,
  showRelevanceScore = false,
  showGitStatus = false,
  showLineNumbers = true,
}) => {
  const [isPending, startTransition] = useTransition();
  const [visibleItems, setVisibleItems] = useState(items);

  // Use deferred value for items to avoid blocking UI updates
  const deferredItems = useDeferredValue(items);

  // Update visible items without blocking
  const updateVisibleItems = useCallback(() => {
    startTransition(() => {
      setVisibleItems(deferredItems);
    });
  }, [deferredItems, startTransition]);

  React.useEffect(() => {
    updateVisibleItems();
  }, [updateVisibleItems]);

  // Render virtualized list item
  const renderItem = useCallback(
    (props: ListChildComponentProps) => {
      const { index, style } = props;
      const item = visibleItems[index];

      return (
        <div style={style}>
          <CodeListItem
            item={item}
            onClick={onItemClick}
            onHover={onItemHover}
            showRelevanceScore={showRelevanceScore}
            showGitStatus={showGitStatus}
            showLineNumbers={showLineNumbers}
          />
        </div>
      );
    },
    [visibleItems, onItemClick, onItemHover, showRelevanceScore, showGitStatus, showLineNumbers]
  );

  // Loading state
  if (isLoading && visibleItems.length === 0) {
    return (
      <Box sx={{ p: 1 }}>
        {Array.from({ length: Math.min(10, Math.ceil(listHeight / itemHeight)) }, (_, i) => (
          <CodeListSkeleton key={i} height={itemHeight} />
        ))}
      </Box>
    );
  }

  // Empty state
  if (!isLoading && visibleItems.length === 0) {
    return (
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: listHeight,
          p: 3,
          color: 'text.secondary',
        }}
      >
        <Search sx={{ fontSize: 48, mb: 2, opacity: 0.5 }} />
        <Typography variant="h6" gutterBottom>
          No results found
        </Typography>
        <Typography variant="body2">Try adjusting your search query</Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ height: listHeight }}>
      <List
        height={listHeight}
        itemCount={visibleItems.length}
        itemSize={itemHeight}
        overscanCount={5}
      >
        {renderItem}
      </List>
    </Box>
  );
};

export default VirtualizedCodeList;
