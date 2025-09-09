import type {
  ShortcutAction,
  KeyCombination,
  UserShortcutProfile,
  ShortcutContext,
  KeyboardService as IKeyboardService,
  KeyboardEventHandler,
  ContextChangeHandler,
} from './types';
import { DEFAULT_KEY_COMBINATIONS } from './defaultShortcuts';

export class KeyboardServiceImpl implements IKeyboardService {
  private shortcuts: Map<string, ShortcutAction> = new Map();
  private keyHandlers: Map<string, KeyboardEventHandler[]> = new Map();
  private contextHandlers: ContextChangeHandler[] = [];
  private currentContext: ShortcutContext = 'global';
  private profiles: Map<string, UserShortcutProfile> = new Map();
  private defaultProfile: UserShortcutProfile;

  constructor() {
    // Create default profile
    this.defaultProfile = {
      id: 'default',
      name: 'Default Profile',
      description: 'Standard keyboard shortcuts',
      shortcuts: { ...DEFAULT_KEY_COMBINATIONS },
      isDefault: true,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    this.profiles.set('default', this.defaultProfile);
    this.loadUserShortcuts();
  }

  registerShortcut(action: ShortcutAction): void {
    this.shortcuts.set(action.id, action);
    this.bindShortcutToHandlers(action);
  }

  unregisterShortcut(actionId: string): void {
    const action = this.shortcuts.get(actionId);
    if (action) {
      this.unbindShortcutFromHandlers(action);
      this.shortcuts.delete(actionId);
    }
  }

  getShortcut(actionId: string): KeyCombination[] | undefined {
    const currentProfile = this.profiles.get(this.currentProfile);
    return currentProfile?.shortcuts[actionId];
  }

  setShortcut(actionId: string, keys: KeyCombination[]): void {
    const currentProfile = this.profiles.get(this.currentProfile);
    if (currentProfile && !currentProfile.isDefault) {
      currentProfile.shortcuts[actionId] = keys;
      currentProfile.updatedAt = Date.now();
      this.saveUserShortcuts();

      const action = this.shortcuts.get(actionId);
      if (action && !currentProfile.isDefault) {
        this.rebindShortcutAction(action);
      }
    }
  }

  clearShortcut(actionId: string): void {
    const currentProfile = this.profiles.get(this.currentProfile);
    if (currentProfile && !currentProfile.isDefault) {
      delete currentProfile.shortcuts[actionId];
      currentProfile.updatedAt = Date.now();
      this.saveUserShortcuts();

      const action = this.shortcuts.get(actionId);
      if (action) {
        this.unbindShortcutFromHandlers(action);
      }
    }
  }

  createProfile(profileData: Omit<UserShortcutProfile, 'id' | 'createdAt' | 'updatedAt'>): UserShortcutProfile {
    const profile: UserShortcutProfile = {
      ...profileData,
      id: `profile_${Date.now()}`,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    this.profiles.set(profile.id, profile);
    this.saveUserShortcuts();
    return profile;
  }

  switchProfile(profileId: string): void {
    const profile = this.profiles.get(profileId);
    if (profile) {
      this.currentProfile = profileId;
      // Rebind all shortcuts with new profile
      this.shortcuts.forEach(action => {
        this.rebindShortcutAction(action);
      });
      this.saveUserShortcuts();
    }
  }

  deleteProfile(profileId: string): void {
    const profile = this.profiles.get(profileId);
    if (profile && !profile.isDefault) {
      this.profiles.delete(profileId);

      // Switch to default if deleting current profile
      if (this.currentProfile === profileId) {
        this.switchProfile('default');
      }

      this.saveUserShortcuts();
    }
  }

  detectConflicts(): Array<{ keys: string; actions: string[] }> {
    const keyMap: Map<string, string[]> = new Map();
    const conflicts: Array<{ keys: string; actions: string[] }> = [];

    this.shortcuts.forEach(action => {
      const keys = this.getShortcut(action.id);
      if (keys) {
        keys.forEach(keyCombo => {
          const keyString = this.keyCombinationToString(keyCombo);
          const actions = keyMap.get(keyString) || [];
          actions.push(action.id);
          keyMap.set(keyString, actions);

          if (actions.length > 1) {
            conflicts.push({ keys: keyString, actions: [...actions] });
          }
        });
      }
    });

    return conflicts;
  }

  async loadUserShortcuts(): Promise<void> {
    try {
      const storedProfiles = localStorage.getItem('keyboardProfiles');
      const storedCurrent = localStorage.getItem('currentKeyboardProfile');

      if (storedProfiles) {
        const parsedProfiles = JSON.parse(storedProfiles);
        Object.entries(parsedProfiles).forEach(([id, profile]: [string, any]) => {
          if (id !== 'default') { // Don't overwrite default profile
            this.profiles.set(id, profile);
          }
        });
      }

      if (storedCurrent && this.profiles.has(storedCurrent)) {
        this.currentProfile = storedCurrent;
      }
    } catch (error) {
      console.error('Failed to load keyboard shortcuts:', error);
    }
  }

  async saveUserShortcuts(): Promise<void> {
    try {
      const profilesToSave: Record<string, any> = {};
      this.profiles.forEach((profile, id) => {
        if (!profile.isDefault) {
          profilesToSave[id] = profile;
        }
      });

      localStorage.setItem('keyboardProfiles', JSON.stringify(profilesToSave));
      localStorage.setItem('currentKeyboardProfile', this.currentProfile);
    } catch (error) {
      console.error('Failed to save keyboard shortcuts:', error);
    }
  }

  // Helper methods
  private keyCombinationToString(combo: KeyCombination): string {
    const parts: string[] = [];
    if (combo.ctrlKey) parts.push('Ctrl');
    if (combo.altKey) parts.push('Alt');
    if (combo.shiftKey) parts.push('Shift');
    if (combo.metaKey) parts.push('Cmd');
    parts.push(combo.key.toUpperCase());
    return parts.join('+');
  }

  private bindShortcutToHandlers(action: ShortcutAction): void {
    const keys = this.getShortcut(action.id);
    if (!keys) return;

    keys.forEach(keyCombo => {
      const contextKey = `${action.context}_${this.keyCombinationToString(keyCombo)}`;
      const handlers = this.keyHandlers.get(contextKey) || [];

      const handler: KeyboardEventHandler = (event: KeyboardEvent) => {
        if (this.matchesKeyCombination(event, keyCombo) &&
            this.currentContext === action.context) {
          event.preventDefault();
          event.stopPropagation();
          action.action();
          return true;
        }
        return false;
      };

      handlers.push(handler);
      this.keyHandlers.set(contextKey, handlers);
    });
  }

  private unbindShortcutFromHandlers(action: ShortcutAction): void {
    const keys = this.getShortcut(action.id);
    if (!keys) return;

    keys.forEach(keyCombo => {
      const contextKey = `${action.context}_${this.keyCombinationToString(keyCombo)}`;
      this.keyHandlers.delete(contextKey);
    });
  }

  private rebindShortcutAction(action: ShortcutAction): void {
    this.unbindShortcutFromHandlers(action);
    this.bindShortcutToHandlers(action);
  }

  private matchesKeyCombination(event: KeyboardEvent, combo: KeyCombination): boolean {
    return event.key.toLowerCase() === combo.key.toLowerCase() &&
           !!event.ctrlKey === !!combo.ctrlKey &&
           !!event.altKey === !!combo.altKey &&
           !!event.shiftKey === !!combo.shiftKey &&
           !!event.metaKey === !!combo.metaKey;
  }

  // Public methods for external integration
  setContext(context: ShortcutContext): void {
    this.currentContext = context;
    this.contextHandlers.forEach(handler => handler(context));
  }

  addContextChangeHandler(handler: ContextChangeHandler): () => void {
    this.contextHandlers.push(handler);
    return () => {
      const index = this.contextHandlers.indexOf(handler);
      if (index > -1) {
        this.contextHandlers.splice(index, 1);
      }
    };
  }

  handleGlobalKeyDown(event: KeyboardEvent): boolean {
    const contextKey = `${this.currentContext}_${this.keyCombinationToString({
      key: event.key,
      ctrlKey: event.ctrlKey,
      altKey: event.altKey,
      shiftKey: event.shiftKey,
      metaKey: event.metaKey,
    })}`;

    const handlers = this.keyHandlers.get(contextKey);
    if (handlers) {
      for (const handler of handlers) {
        if (handler(event)) {
          return true;
        }
      }
    }

    return false;
  }

  getCurrentProfile(): UserShortcutProfile | undefined {
    return this.profiles.get(this.currentProfile) || this.defaultProfile;
  }

  getAllProfiles(): UserShortcutProfile[] {
    return Array.from(this.profiles.values());
  }
}

// Create singleton instance
export const keyboardService = new KeyboardServiceImpl();