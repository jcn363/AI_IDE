// Types
export * from './types';

// Services
export { keyboardService } from './KeyboardService';
export { monacoKeyboardIntegration } from './MonacoKeyboardIntegration';

// Redux slice and actions
export { default as keyboardReducer } from './keyboardSlice';
export {
  setCurrentProfile,
  createProfile,
  deleteProfile,
  setShortcut,
  clearShortcut,
  startRecording,
  stopRecording,
  recordKey,
  loadProfiles,
  setConflicts,
} from './keyboardSlice';

// Selectors
export {
  selectKeyboard,
  selectCurrentProfile,
  selectProfiles,
  selectShortcuts,
  selectConflicts,
  selectRecordingMode,
  selectRecordingFor,
  selectShortcut,
} from './keyboardSlice';

// Hooks
export { useKeyboard, useKeyboardForEditor } from './useKeyboard';

// Default shortcuts
export { DEFAULT_KEY_COMBINATIONS } from './defaultShortcuts';

// New Keybinding Manager
export { keybindingManager, KeybindingManager } from './KeybindingManager';

// Components
export { default as KeybindingSettings } from './components/KeybindingSettings';