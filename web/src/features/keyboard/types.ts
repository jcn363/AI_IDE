import type * as monaco from 'monaco-editor';

export type ShortcutContext =
  | 'global'
  | 'editor'
  | 'terminal'
  | 'file-explorer'
  | 'command-palette'
  | 'search'
  | 'git'
  | 'debugger'
  | 'cargo'
  | 'ai-assistant';

export interface KeyCombination {
  key: string;
  ctrlKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
  metaKey?: boolean; // Cmd on Mac, Windows key on Windows
}

export interface ShortcutAction {
  id: string;
  name: string;
  description: string;
  context: ShortcutContext;
  action: (...args: any[]) => void | Promise<void>;
  defaultKeys?: KeyCombination[];
}

export interface UserShortcutProfile {
  id: string;
  name: string;
  description: string;
  shortcuts: Record<string, KeyCombination[]>;
  isDefault?: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface ShortcutState {
  currentProfile: string;
  profiles: Record<string, UserShortcutProfile>;
  conflicts: Array<{
    keys: string;
    actions: string[];
  }>;
  recordingMode: boolean;
  recordingFor?: string;
  lastRecordedKey?: KeyCombination;
}

export interface KeyboardService {
  registerShortcut(action: ShortcutAction): void;
  unregisterShortcut(actionId: string): void;
  getShortcut(actionId: string): KeyCombination[] | undefined;
  setShortcut(actionId: string, keys: KeyCombination[]): void;
  clearShortcut(actionId: string): void;
  createProfile(
    profile: Omit<UserShortcutProfile, 'id' | 'createdAt' | 'updatedAt'>
  ): UserShortcutProfile;
  switchProfile(profileId: string): void;
  deleteProfile(profileId: string): void;
  detectConflicts(): Array<{ keys: string; actions: string[] }>;
  loadUserShortcuts(): Promise<void>;
  saveUserShortcuts(): Promise<void>;
}

export interface MonacoKeyboardIntegration {
  registerMonacoAction(
    editor: monaco.editor.IStandaloneCodeEditor,
    action: ShortcutAction,
    keys?: KeyCombination[]
  ): void;
  unregisterMonacoAction(editor: monaco.editor.IStandaloneCodeEditor, actionId: string): void;
  updateMonacoKeybindings(editor: monaco.editor.IStandaloneCodeEditor): void;
}

export type KeyboardEventHandler = (event: KeyboardEvent) => boolean; // Return true to prevent default
export type ContextChangeHandler = (context: ShortcutContext) => void;
