import { useState, useCallback } from 'react';
import { Position } from 'vscode-languageserver-types';

export const useAIContextMenu = () => {
  const [contextMenu, setContextMenu] = useState<{
    mouseX: number;
    mouseY: number;
    selectedText?: string;
    position?: Position;
  } | null>(null);

  const handleContextMenu = useCallback(
    (event: React.MouseEvent) => {
      event.preventDefault();
      const selection = window.getSelection()?.toString().trim();

      setContextMenu(
        contextMenu === null
          ? {
              mouseX: event.clientX + 2,
              mouseY: event.clientY - 6,
              selectedText: selection,
            }
          : null
      );
    },
    [contextMenu]
  );

  const handleClose = useCallback(() => {
    setContextMenu(null);
  }, []);

  const handleEditorContextMenu = useCallback((event: MouseEvent, position: Position) => {
    event.preventDefault();
    const selection = window.getSelection()?.toString().trim();

    setContextMenu({
      mouseX: event.clientX + 2,
      mouseY: event.clientY - 6,
      selectedText: selection,
      position,
    });
  }, []);

  return {
    contextMenu,
    handleContextMenu,
    handleEditorContextMenu,
    handleClose,
  };
};

export default useAIContextMenu;
