import React, { RefObject, useCallback, useEffect, useRef } from 'react';
import { useAIAssistant } from '../hooks/useAIAssistant';
import './keyboardShortcuts.css';

type ShortcutHandler = (e: KeyboardEvent) => void;

interface ShortcutMap {
  [key: string]: {
    keys: string[];
    description: string;
    handler: ShortcutHandler;
  };
}

/**
 * Hook to manage keyboard shortcuts for AI features
 * @param editorRef Reference to the editor component
 */
export const useAIShortcuts = (editorRef: RefObject<any>) => {
  const {
    analyzeCurrentFile,
    generateTests,
    generateDocumentation,
    explainCode,
    refactorCode,
    togglePanel,
  } = useAIAssistant();

  // Get the current editor content and selection
  const getEditorContext = useCallback(() => {
    if (!editorRef.current) return { code: '', selection: null, path: 'current_file.rs' };

    const editor = editorRef.current;
    const model = editor.getModel();
    const selection = editor.getSelection();

    return {
      code: selection ? model?.getValueInRange(selection) || '' : editor.getValue(),
      selection,
      path: model?.uri?.path || 'current_file.rs',
    };
  }, [editorRef]);

  // Define keyboard shortcuts
  const shortcuts: ShortcutMap = {
    analyze: {
      keys: ['Ctrl+Shift+A', 'Cmd+Shift+A'],
      description: 'Analyze current file for issues and improvements',
      handler: (e: KeyboardEvent) => {
        e.preventDefault();
        const { code, path } = getEditorContext();
        analyzeCurrentFile(code, path);
      },
    },
    generateTests: {
      keys: ['Ctrl+Shift+T', 'Cmd+Shift+T'],
      description: 'Generate tests for the current selection or file',
      handler: async (e: KeyboardEvent) => {
        e.preventDefault();
        const { code, path } = getEditorContext();
        await generateTests(code, path);
      },
    },
    generateDocs: {
      keys: ['Ctrl+Shift+D', 'Cmd+Shift+D'],
      description: 'Generate documentation for the current selection',
      handler: async (e: KeyboardEvent) => {
        e.preventDefault();
        const { code, path } = getEditorContext();
        await generateDocumentation(code, path);
      },
    },
    explainCode: {
      keys: ['Ctrl+Shift+E', 'Cmd+Shift+E'],
      description: 'Explain the selected code',
      handler: async (e: KeyboardEvent) => {
        e.preventDefault();
        const { code } = getEditorContext();
        if (code) await explainCode(code);
      },
    },
    refactor: {
      keys: ['Ctrl+Shift+R', 'Cmd+Shift+R'],
      description: 'Refactor the selected code',
      handler: async (e: KeyboardEvent) => {
        e.preventDefault();
        const { code, path } = getEditorContext();
        if (code) await refactorCode(code, path);
      },
    },
    togglePanel: {
      keys: ['Ctrl+`', 'Cmd+`'],
      description: 'Toggle AI assistant panel',
      handler: (e: KeyboardEvent) => {
        e.preventDefault();
        togglePanel();
      },
    },
  };

  // Check if a keyboard event matches any of the defined shortcuts
  const matchesShortcut = useCallback((e: KeyboardEvent, keys: string[]): boolean => {
    return keys.some((key) => {
      const parts = key.split('+');
      const keyChar = parts.pop()?.toLowerCase() || '';

      const expectedModifiers = {
        ctrl: parts.some((p) => p.toLowerCase() === 'ctrl'),
        cmd: parts.some((p) => p.toLowerCase() === 'cmd'),
        alt: parts.some((p) => p.toLowerCase() === 'alt'),
        shift: parts.some((p) => p.toLowerCase() === 'shift'),
      };

      return (
        e.ctrlKey === expectedModifiers.ctrl &&
        e.metaKey === expectedModifiers.cmd &&
        e.altKey === expectedModifiers.alt &&
        e.shiftKey === expectedModifiers.shift &&
        e.key.toLowerCase() === keyChar
      );
    });
  }, []);

  // Set up event listeners for keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger if typing in an input or textarea
      const target = e.target as HTMLElement;
      if (['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)) {
        return;
      }

      // Check each shortcut
      Object.values(shortcuts).forEach(({ keys, handler }) => {
        if (matchesShortcut(e, keys)) {
          handler(e);
        }
      });
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [editorRef, matchesShortcut, shortcuts]);

  // Return the list of available shortcuts for display in the UI
  const getShortcutsList = useCallback(() => {
    return Object.entries(shortcuts).map(([id, { keys, description }]) => ({
      id,
      keys: keys[0], // Return the first key combination for display
      description,
    }));
  }, [shortcuts]);

  return {
    shortcuts: getShortcutsList(),
  };
};

interface ShortcutHelpItem {
  id: string;
  keys: string;
  description: string;
}

/**
 * Component to display available keyboard shortcuts
 */
export const AIShortcutsHelp: React.FC = () => {
  const shortcuts: ShortcutHelpItem[] = [
    { id: 'analyze', keys: 'Ctrl/Cmd + Alt + A', description: 'Analyze current file or selection' },
    { id: 'tests', keys: 'Ctrl/Cmd + Alt + T', description: 'Generate tests' },
    { id: 'docs', keys: 'Ctrl/Cmd + Alt + D', description: 'Generate documentation' },
    { id: 'explain', keys: 'Ctrl/Cmd + Alt + E', description: 'Explain selected code' },
    { id: 'refactor', keys: 'Ctrl/Cmd + Alt + R', description: 'Refactor selected code' },
  ];

  return (
    <div className="ai-shortcuts-help">
      <h3>AI Assistant Keyboard Shortcuts</h3>
      <table>
        <thead>
          <tr>
            <th>Shortcut</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          {shortcuts.map(({ id, keys, description }) => (
            <tr key={id}>
              <td>
                <code>{keys}</code>
              </td>
              <td>{description}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default useAIShortcuts;
