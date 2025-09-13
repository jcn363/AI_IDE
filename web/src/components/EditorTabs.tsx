import React, { useEffect, useRef, useState } from 'react';
import { Box, IconButton, Tab, Tabs, styled, useTheme } from '@mui/material';
import { Close as CloseIcon, DragIndicator as DragHandleIcon } from '@mui/icons-material';
import { useAppDispatch, useAppSelector } from '../store';
import { tabManagementActions, tabManagementSelectors } from '../store/slices/tabManagementSlice';
import type { RootState } from '../store/types';

// Extend the global Node interface
declare global {
  interface Node {
    contains(otherNode: Node | null): boolean;
  }
}

// Type for tab data that will be transferred during drag and drop
type TabData = {
  paneId: string;
  tabIndex: number;
  filePath: string;
  isPinned?: boolean; // Make isPinned optional since it might not always be needed
};

// Extend the DataTransfer interface to include the methods we need
interface CustomDataTransfer extends DataTransfer {
  effectAllowed:
    | 'none'
    | 'copy'
    | 'copyLink'
    | 'copyMove'
    | 'link'
    | 'linkMove'
    | 'move'
    | 'all'
    | 'uninitialized';
  dropEffect: 'none' | 'copy' | 'link' | 'move';
  setData(format: string, data: string): void;
  getData(format: string): string;
  setDragImage(image: Element, x: number, y: number): void;
}

// Helper function to safely parse tab data from DataTransfer
function getTabDataFromDataTransfer(dataTransfer: DataTransfer): TabData | null {
  try {
    const dt = dataTransfer as unknown as CustomDataTransfer;
    const data = dt.getData('text/plain');
    if (!data) return null;
    const parsed = JSON.parse(data);
    if (
      typeof parsed === 'object' &&
      parsed !== null &&
      'paneId' in parsed &&
      'tabIndex' in parsed
    ) {
      return parsed as TabData;
    }
    return null;
  } catch (error) {
    console.error('Failed to parse tab data from DataTransfer', error);
    return null;
  }
}

interface EditorTab {
  path: string;
  isPinned?: boolean;
}

interface EditorTabsProps {
  paneId: string;
  files: EditorTab[];
  activeFile: string | null;
  onTabChange: (filePath: string) => void;
  onTabClose?: (filePath: string) => void;
}

const DraggableTab = styled(Tab)(({ theme }) => ({
  textTransform: 'none',
  minHeight: '48px',
  transition: 'background-color 0.2s',
  '&.Mui-selected': {
    backgroundColor: theme.palette.action.selected,
  },
  '&:hover': {
    backgroundColor: theme.palette.action.hover,
  },
}));

const EditorTabs: React.FC<EditorTabsProps> = ({
  paneId,
  files,
  activeFile,
  onTabChange,
  onTabClose,
}) => {
  const theme = useTheme();
  const dispatch = useAppDispatch();
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const [isDraggingOver, setIsDraggingOver] = useState(false);
  const [draggedTabIndex, setDraggedTabIndex] = useState<number | null>(null);
  const tabsContainerRef = useRef<HTMLDivElement>(null);

  // Get drag state from Redux store with proper typing
  const dragState = useAppSelector(tabManagementSelectors.selectDragState);
  const isSourcePane = dragState.sourcePaneId === paneId;
  const isTargetPane = dragState.targetPaneId === paneId;

  const handleTabClick = (_: React.SyntheticEvent, filePath: string) => {
    onTabChange(filePath);
  };

  const handleDragStart = (e: React.DragEvent<HTMLDivElement>, index: number) => {
    const dt = e.dataTransfer as unknown as CustomDataTransfer;
    dt.effectAllowed = 'move';

    const tabData: TabData = {
      paneId,
      tabIndex: index,
      filePath: files[index].path,
      isPinned: files[index].isPinned,
    };
    dt.setData('text/plain', JSON.stringify(tabData));

    // Start the drag operation in the Redux store
    dispatch(tabManagementActions.startDragTab({ paneId, index }));
    setDraggedTabIndex(index);

    // Set a custom drag image for better visual feedback
    const tabElement = e.currentTarget as unknown as HTMLElement;
    const rect = (
      tabElement as unknown as { getBoundingClientRect: () => DOMRect }
    ).getBoundingClientRect();

    // Create a simple drag image element
    const dragImage = document.createElement('div');
    dragImage.textContent = files[index].path.split('/').pop() || '';
    Object.assign(dragImage.style, {
      position: 'fixed',
      top: '-1000px',
      left: '0',
      width: `${rect.width}px`,
      height: `${rect.height}px`,
      backgroundColor: theme.palette.background.paper,
      border: `1px solid ${theme.palette.divider}`,
      borderRadius: '4px',
      padding: '8px',
      pointerEvents: 'none',
      opacity: '0.8',
      zIndex: '9999',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      boxShadow: theme.shadows[2],
    });

    // Add drag image to the DOM
    document.body.appendChild(dragImage);

    try {
      // Set the drag image
      dt.setDragImage(dragImage, rect.width / 2, rect.height / 2);

      // Clean up the drag image after a short delay
      setTimeout(() => {
        if (document.body.contains(dragImage)) {
          document.body.removeChild(dragImage);
        }
      }, 0);
    } catch (error) {
      console.error('Error setting drag image:', error);
      // Clean up if there was an error
      if (document.body.contains(dragImage)) {
        document.body.removeChild(dragImage);
      }
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>, index: number) => {
    e.preventDefault();
    e.stopPropagation();

    // Only update if the index has changed to prevent unnecessary re-renders
    if (dragOverIndex !== index) {
      setDragOverIndex(index);
    }

    // Update the drag target in the Redux store
    dispatch(
      tabManagementActions.updateDragTarget({
        paneId,
        index: index >= 0 ? index : 0,
      })
    );
  };

  const handleDrop = (e: React.DragEvent<HTMLDivElement>, targetIndex: number) => {
    e.preventDefault();
    e.stopPropagation();

    const tabData = getTabDataFromDataTransfer(e.dataTransfer);
    if (!tabData) return;

    const { paneId: sourcePaneId, tabIndex: sourceIndex, filePath: sourceFilePath } = tabData;

    // Handle the drop by dispatching the appropriate action
    if (e.ctrlKey || e.metaKey) {
      // If Ctrl/Cmd is pressed, create a copy of the tab
      const filePathToCopy =
        sourceFilePath ?? (sourcePaneId === paneId ? files[sourceIndex]?.path : undefined);

      if (filePathToCopy) {
        dispatch(
          tabManagementActions.openFileInPane({
            paneId,
            filePath: filePathToCopy,
          })
        );
      }
    } else if (e.shiftKey) {
      // If Shift is pressed, move the tab to a new split pane
      const direction = window.innerWidth / 2 > e.clientX ? 'horizontal' : 'vertical';
      dispatch(
        tabManagementActions.endDragTab({
          createNewPane: true,
          direction,
        })
      );
    } else {
      // Regular drag and drop move
      dispatch(tabManagementActions.endDragTab({}));
    }

    // Reset local state
    setDragOverIndex(null);
    setDraggedTabIndex(null);
    setIsDraggingOver(false);
  };

  const handleDragEnd = () => {
    // Only reset if this is the source of the drag
    if (draggedTabIndex !== null) {
      dispatch(tabManagementActions.endDragTab({}));
      setDragOverIndex(null);
      setDraggedTabIndex(null);
      setIsDraggingOver(false);
    }
  };

  const handleDragEnter = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDraggingOver(true);
  };

  const handleDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

    // Only set dragging over to false if we're leaving the container
    const relatedTarget = e.relatedTarget as Node | null;
    const { currentTarget } = e;

    if (!relatedTarget || !currentTarget.contains(relatedTarget)) {
      setIsDraggingOver(false);
      setDragOverIndex(null);
    }
  };

  const handleCloseTab = (e: React.MouseEvent, filePath: string) => {
    e.stopPropagation();
    if (onTabClose) {
      onTabClose(filePath);
    } else {
      dispatch(tabManagementActions.closeFileInPane({ paneId, filePath }));
    }
  };

  if (files.length === 0) return null;

  // Calculate the position for the drop indicator
  const getDropIndicatorPosition = () => {
    if (dragOverIndex === null || !tabsContainerRef.current) return null;

    const container = tabsContainerRef.current as unknown as HTMLElement;
    const tabElements = Array.from(container.querySelectorAll<HTMLElement>('[role="tab"]'));

    if (tabElements.length === 0) return null;

    // Position after the tab at dragOverIndex, or before the first tab if dragOverIndex is 0
    const targetIndex = Math.min(dragOverIndex, tabElements.length - 1);
    const targetTab = tabElements[targetIndex] as unknown as HTMLElement;

    if (!targetTab) return null;

    try {
      const rect = targetTab.getBoundingClientRect();
      const containerRect = (container as unknown as HTMLElement).getBoundingClientRect();

      return {
        left:
          dragOverIndex === 0
            ? rect.left - containerRect.left - 2
            : rect.right - containerRect.left,
        isEnd: dragOverIndex >= tabElements.length,
      };
    } catch (error) {
      console.error('Error calculating drop indicator position:', error);
      return null;
    }
  };

  const dropIndicator = getDropIndicatorPosition();

  return (
    <Box
      ref={tabsContainerRef}
      sx={{
        position: 'relative',
        borderBottom: 1,
        borderColor: 'divider',
        backgroundColor: isDraggingOver ? theme.palette.action.hover : 'inherit',
        transition: 'background-color 0.2s',
        '&:hover': {
          backgroundColor: isDraggingOver
            ? theme.palette.action.hover
            : theme.palette.action.hoverOpacity,
        },
      }}
      onDragEnter={handleDragEnter}
      onDragOver={(e) => {
        e.preventDefault();
        e.stopPropagation();
        if (!isDraggingOver) setIsDraggingOver(true);
      }}
      onDragLeave={handleDragLeave}
      onDrop={(e) => {
        e.preventDefault();
        e.stopPropagation();
        const targetIndex = files.length; // Drop at the end
        handleDrop(e, targetIndex);
      }}
    >
      {dropIndicator && (
        <Box
          sx={{
            position: 'absolute',
            top: 0,
            bottom: 0,
            left: dropIndicator.left,
            width: '2px',
            backgroundColor: theme.palette.primary.main,
            zIndex: 1,
            '&::after': {
              content: '""',
              position: 'absolute',
              top: '50%',
              left: '-3px',
              width: '8px',
              height: '8px',
              backgroundColor: theme.palette.primary.main,
              borderRadius: '50%',
              transform: 'translateY(-50%)',
            },
          }}
        />
      )}
      <Tabs
        value={activeFile || false}
        onChange={handleTabClick}
        variant="scrollable"
        scrollButtons="auto"
        aria-label="editor tabs"
        sx={{
          minHeight: '48px',
          '& .MuiTabs-scroller': {
            overflow: 'visible !important',
          },
        }}
      >
        {files.map((tab, index) => {
          const fileName = tab.path.split('/').pop() || tab.path;
          const isDragged = draggedTabIndex === index;
          const isDragOver = dragOverIndex === index;

          return (
            <DraggableTab
              key={tab.path}
              value={tab.path}
              draggable
              onDragStart={(e) => handleDragStart(e, index)}
              onDragOver={(e) => handleDragOver(e, index)}
              onDrop={(e) => handleDrop(e, index)}
              onDragEnd={handleDragEnd}
              onDragLeave={() => setDragOverIndex(null)}
              sx={{
                minHeight: '48px',
                pr: 0.5,
                opacity: isDragged || isDragOver ? 0.5 : 1,
                borderBottom: isDragOver ? '2px solid' : 'none',
                borderColor: 'primary.main',
                '&.Mui-selected': {
                  color: 'text.primary',
                  fontWeight: 'medium',
                  backgroundColor: 'action.selected',
                },
              }}
              label={
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    minWidth: 0,
                  }}
                >
                  <Box
                    component="span"
                    sx={{
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                      maxWidth: '150px',
                    }}
                  >
                    {fileName}
                  </Box>
                  <IconButton
                    size="small"
                    onClick={(e) => handleCloseTab(e, tab.path)}
                    sx={{ ml: 0.5, p: 0.5 }}
                  >
                    <CloseIcon fontSize="small" />
                  </IconButton>
                </Box>
              }
            />
          );
        })}
      </Tabs>
    </Box>
  );
};

export default EditorTabs;
