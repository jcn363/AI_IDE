import { useEffect, useCallback } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import type { KeyCombination, ShortcutAction } from './types';
import { keyboardService } from './KeyboardService';
import { monacoKeyboardIntegration } from './MonacoKeyboardIntegration';
import {
  setShortcut,
  clearShortcut,
  startRecording,
  stopRecording,
  recordKey,
  loadProfiles,
} from './keyboardSlice';
import {
  selectKeyboard,
  selectCurrentProfile,
  selectShortcuts,
  selectConflicts,
  selectRecordingMode,
  selectRecordingFor,
  selectShortcut,
} from './keyboardSlice';

export function useKeyboard() {
  const dispatch = useDispatch();
  const keyboardState = useSelector(selectKeyboard);
  const currentProfile = useSelector(selectCurrentProfile);
  const shortcuts = useSelector(selectShortcuts);
  const conflicts = useSelector(selectConflicts);
  const recordingMode = useSelector(selectRecordingMode);
  const recordingFor = useSelector(selectRecordingFor);

  // Initialize keyboard service
  useEffect(() => {
    keyboardService.loadUserShortcuts().then(() => {
      dispatch(loadProfiles());
    });
  }, [dispatch]);

  // Global keyboard event handler
  useEffect(() => {
    const handleGlobalKeyDown = (event: KeyboardEvent) => {
      if (recordingMode) {
        const keyCombo: KeyCombination = {
          key: event.key,
          ctrlKey: event.ctrlKey,
          altKey: event.altKey,
          shiftKey: event.shiftKey,
          metaKey: event.metaKey,
        };

        dispatch(recordKey(keyCombo));
        dispatch(stopRecording());
        if (recordingFor) {
          dispatch(setShortcut({ actionId: recordingFor, keys: [keyCombo] }));
        }
        return;
      }

      // Handle shortcuts based on context
      const handled = keyboardService.handleGlobalKeyDown(event);
      if (handled) {
        event.preventDefault();
        event.stopPropagation();
      }
    };

    document.addEventListener('keydown', handleGlobalKeyDown);
    return () => {
      document.removeEventListener('keydown', handleGlobalKeyDown);
    };
  }, [recordingMode, recordingFor, dispatch]);

  const registerShortcut = useCallback((action: ShortcutAction) => {
    keyboardService.registerShortcut(action);
  }, []);

  const unregisterShortcut = useCallback((actionId: string) => {
    keyboardService.unregisterShortcut(actionId);
  }, []);

  const setShortcutKeys = useCallback(
    (actionId: string, keys: KeyCombination[]) => {
      dispatch(setShortcut({ actionId, keys }));
    },
    [dispatch]
  );

  const clearShortcutKeys = useCallback(
    (actionId: string) => {
      dispatch(clearShortcut(actionId));
    },
    [dispatch]
  );

  const startShortcutRecording = useCallback(
    (actionId: string) => {
      dispatch(startRecording(actionId));
    },
    [dispatch]
  );

  const stopShortcutRecording = useCallback(() => {
    dispatch(stopRecording());
  }, [dispatch]);

  const switchToProfile = useCallback(
    (profileId: string) => {
      keyboardService.switchProfile(profileId);
      // Reload profiles to get updated state
      dispatch(loadProfiles());
    },
    [dispatch]
  );

  const detectShortcutConflicts = useCallback(() => {
    return keyboardService.detectConflicts();
  }, []);

  return {
    // State
    keyboardState,
    currentProfile,
    shortcuts,
    conflicts,
    recordingMode,
    recordingFor,

    // Actions
    registerShortcut,
    unregisterShortcut,
    setShortcutKeys,
    clearShortcutKeys,
    startShortcutRecording,
    stopShortcutRecording,
    switchToProfile,
    detectShortcutConflicts,
    getShortcut: (actionId: string) => keyboardService.getShortcut(actionId),

    // Utilities
    keyCombinationToString: (combo: KeyCombination) => {
      const parts: string[] = [];
      if (combo.ctrlKey) parts.push('Ctrl');
      if (combo.altKey) parts.push('Alt');
      if (combo.shiftKey) parts.push('Shift');
      if (combo.metaKey) parts.push('Cmd');
      parts.push(combo.key.toUpperCase());
      return parts.join('+');
    },
  };
}

export function useKeyboardForEditor(editor: any) {
  const { registerShortcut } = useKeyboard();

  useEffect(() => {
    if (!editor) return;

    // Register Monaco-specific shortcuts
    const saveAction: ShortcutAction = {
      id: 'editor.save',
      name: 'Save File',
      description: 'Save the current file',
      context: 'editor',
      action: () => {
        // Implement save logic
        console.log('Save file');
      },
    };

    registerShortcut(saveAction);

    return () => {
      // Cleanup would normally happen here, but we'll let the service handle it
    };
  }, [editor, registerShortcut]);

  return {
    registerMonacoAction: (action: ShortcutAction, keys?: KeyCombination[]) => {
      if (editor) {
        monacoKeyboardIntegration.registerMonacoAction(editor, action, keys);
      }
    },
    unregisterMonacoAction: (actionId: string) => {
      if (editor) {
        monacoKeyboardIntegration.unregisterMonacoAction(editor, actionId);
      }
    },
  };
}
