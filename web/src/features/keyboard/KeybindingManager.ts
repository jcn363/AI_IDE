//! Enhanced Keybinding Manager with persistence and conflict resolution
//!
//! This service provides a comprehensive keybinding management system with
//! support for multiple profiles, conflict detection, and persistent storage.

import { invoke } from '@tauri-apps/api/core';
import {
  ShortcutContext,
  KeyCombination,
  ShortcutAction,
  UserShortcutProfile,
  KeybindingConflict,
} from './types';
import { defaultShortcuts } from './defaultShortcuts';

export class KeybindingManager {
  private currentProfile: string = 'default';
  private profiles: Map<string, UserShortcutProfile> = new Map();
  private actions: ShortcutAction[] = [];
  private eventListeners: Map<string, (profile: UserShortcutProfile) => void> = new Map();

  constructor() {
    this.initializeDefaultActions();
  }

  private initializeDefaultActions() {
    this.actions = [
      {
        id: 'editor.save',
        name: 'Save File',
        description: 'Save the current file',
        context: 'editor',
        action: () => {},
        defaultKeys: [{ key: 's', ctrlKey: true }],
      },
      {
        id: 'editor.undo',
        name: 'Undo',
        description: 'Undo last action',
        context: 'editor',
        action: () => {},
        defaultKeys: [{ key: 'z', ctrlKey: true }],
      },
      {
        id: 'editor.redo',
        name: 'Redo',
        description: 'Redo last undone action',
        context: 'editor',
        action: () => {},
        defaultKeys: [
          { key: 'z', ctrlKey: true, shiftKey: true },
          { key: 'y', ctrlKey: true },
        ],
      },
      {
        id: 'terminal.new',
        name: 'New Terminal',
        description: 'Open new terminal instance',
        context: 'terminal',
        action: () => {},
        defaultKeys: [{ key: '`', ctrlKey: true, shiftKey: true }],
      },
      {
        id: 'search.find',
        name: 'Find',
        description: 'Open find dialog',
        context: 'search',
        action: () => {},
        defaultKeys: [{ key: 'f', ctrlKey: true }],
      },
      {
        id: 'search.replace',
        name: 'Replace',
        description: 'Open find and replace dialog',
        context: 'search',
        action: () => {},
        defaultKeys: [{ key: 'h', ctrlKey: true }],
      },
      {
        id: 'command.palette',
        name: 'Command Palette',
        description: 'Open command palette',
        context: 'command-palette',
        action: () => {},
        defaultKeys: [{ key: 'p', ctrlKey: true, shiftKey: true }],
      },
      {
        id: 'git.status',
        name: 'Git Status',
        description: 'Show git status',
        context: 'git',
        action: () => {},
        defaultKeys: [{ key: 'g', ctrlKey: true, shiftKey: true }],
      },
      {
        id: 'explorer.toggle',
        name: 'Toggle Explorer',
        description: 'Toggle file explorer visibility',
        context: 'file-explorer',
        action: () => {},
        defaultKeys: [{ key: 'b', ctrlKey: true, shiftKey: true }],
      },
      // Add more actions as needed
      ...this.actions,
    ];
  }

  // Profile Management
  async loadProfiles(): Promise<void> {
    try {
      const profiles = Object.values(this.profiles);
      if (profiles.length === 0) {
        await this.createDefaultProfile();
      }
    } catch (error) {
      console.error('Failed to load keybinding profiles:', error);
      await this.createDefaultProfile();
    }
  }

  private async createDefaultProfile(): Promise<void> {
    const shortcuts: Record<string, KeyCombination[]> = {};

    for (const action of this.actions) {
      if (action.defaultKeys && action.defaultKeys.length > 0) {
        shortcuts[action.id] = action.defaultKeys;
      }
    }

    const defaultProfile: UserShortcutProfile = {
      id: 'default',
      name: 'Default Profile',
      description: 'Default keybindings profile',
      shortcuts,
      isDefault: true,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    this.profiles.set('default', defaultProfile);
  }

  async getProfile(profileId: string): Promise<UserShortcutProfile | null> {
    try {
      const result = await invoke('get_keybindings_profile', { profileId });
      return typeof result === 'string' ? JSON.parse(result) : result;
    } catch (error) {
      console.error('Failed to get keybinding profile:', error);
      return this.profiles.get(profileId) || null;
    }
  }

  async createProfile(name: string, description: string): Promise<UserShortcutProfile> {
    try {
      const result = await invoke('create_keybindings_profile', {
        profileData: {
          name,
          description,
          shortcuts: this.createDefaultShortcuts(),
        },
      });

      const profile = typeof result === 'string' ? JSON.parse(result) : result;
      this.profiles.set(profile.id, profile);
      return profile;
    } catch (error) {
      console.error('Failed to create keybinding profile:', error);
      throw error;
    }
  }

  async updateProfile(profileId: string, updates: Partial<UserShortcutProfile>): Promise<void> {
    try {
      await invoke('update_keybinding_profile', { profileId, updates });

      const currentProfile = this.profiles.get(profileId);
      if (currentProfile) {
        this.profiles.set(profileId, { ...currentProfile, ...updates });
      }

      this.notifyListeners('profile_updated', this.profiles.get(profileId)!);
    } catch (error) {
      console.error('Failed to update keybinding profile:', error);
      throw error;
    }
  }

  async deleteProfile(profileId: string): Promise<void> {
    try {
      await invoke('delete_keybindings_profile', { profileId });
      this.profiles.delete(profileId);
      this.notifyListeners('profile_deleted', undefined);
    } catch (error) {
      console.error('Failed to delete keybinding profile:', error);
      throw error;
    }
  }

  async switchProfile(profileId: string): Promise<void> {
    try {
      await invoke('apply_keybindings_profile', { profileId });
      this.currentProfile = profileId;
      this.notifyListeners('profile_switched', this.profiles.get(profileId)!);
    } catch (error) {
      console.error('Failed to switch keybinding profile:', error);
      throw error;
    }
  }

  // Key Combination Formatting
  static formatKeyCombination(combination: KeyCombination): string {
    const parts: string[] = [];

    if (combination.ctrlKey) parts.push('Ctrl');
    if (combination.altKey) parts.push('Alt');
    if (combination.shiftKey) parts.push('Shift');
    if (combination.metaKey) parts.push('Cmd');

    if (combination.key) {
      const key = combination.key.toUpperCase();
      if (key === ' ') {
        parts.push('Space');
      } else if (key.length === 1) {
        parts.push(key);
      } else {
        parts.push(key);
      }
    }

    return parts.join('+');
  }

  static parseKeyCombination(formatted: string): KeyCombination {
    const parts = formatted.split('+').map((p) => p.trim().toLowerCase());
    const combination: KeyCombination = {
      key: '',
      ctrlKey: parts.includes('ctrl'),
      altKey: parts.includes('alt'),
      shiftKey: parts.includes('shift'),
      metaKey: parts.includes('cmd') || parts.includes('meta'),
    };

    // Find the key part
    for (const part of parts) {
      if (!['ctrl', 'alt', 'shift', 'cmd', 'meta'].includes(part)) {
        combination.key = part === 'space' ? ' ' : part;
        break;
      }
    }

    return combination;
  }

  // Conflict Detection
  async detectConflicts(profileId: string): Promise<KeybindingConflict[]> {
    try {
      const result = await invoke('validate_keybinding_conflicts', { profileId });
      return typeof result === 'string' ? JSON.parse(result) : result;
    } catch (error) {
      console.error('Failed to validate keybinding conflicts:', error);
      return [];
    }
  }

  // Shortcuts Management
  getShortcut(actionId: string): KeyCombination[] {
    const profile = this.profiles.get(this.currentProfile);
    if (!profile) return [];

    return profile.shortcuts[actionId] || [];
  }

  setShortcut(actionId: string, keys: KeyCombination[]): void {
    const profile = this.profiles.get(this.currentProfile);
    if (!profile) return;

    profile.shortcuts[actionId] = keys;
    profile.updatedAt = Date.now();

    this.notifyListeners('shortcuts_updated', profile);
  }

  clearShortcut(actionId: string): void {
    const profile = this.profiles.get(this.currentProfile);
    if (!profile) return;

    delete profile.shortcuts[actionId];
    profile.updatedAt = Date.now();

    this.notifyListeners('shortcuts_updated', profile);
  }

  // Event System
  addEventListener(event: string, listener: (profile: UserShortcutProfile) => void): void {
    this.eventListeners.set(event, listener);
  }

  removeEventListener(event: string): void {
    this.eventListeners.delete(event);
  }

  private notifyListeners(event: string, profile?: UserShortcutProfile): void {
    const listener = this.eventListeners.get(event);
    if (listener && profile) {
      listener(profile);
    }
  }

  // Utility methods
  getAvailableActions(): ShortcutAction[] {
    return this.actions;
  }

  getCurrentProfile(): UserShortcutProfile | null {
    return this.profiles.get(this.currentProfile) || null;
  }

  getAllProfiles(): UserShortcutProfile[] {
    return Array.from(this.profiles.values());
  }

  private createDefaultShortcuts(): Record<string, KeyCombination[]> {
    const shortcuts: Record<string, KeyCombination[]> = {};

    for (const action of this.actions) {
      if (action.defaultKeys && action.defaultKeys.length > 0) {
        shortcuts[action.id] = action.defaultKeys;
      }
    }

    return shortcuts;
  }

  // Export/Import
  async exportProfile(profileId: string): Promise<string> {
    try {
      const result = await invoke('export_keybindings', { profileId });
      return typeof result === 'string' ? result : JSON.stringify(result);
    } catch (error) {
      console.error('Failed to export keybindings:', error);
      throw error;
    }
  }

  async importProfile(profileData: string): Promise<string> {
    try {
      const profile = JSON.parse(profileData);
      const result = await invoke('import_keybindings', { importData: profile });
      return typeof result === 'string' ? result : JSON.stringify(result);
    } catch (error) {
      console.error('Failed to import keybindings:', error);
      throw error;
    }
  }

  async resetToDefaults(): Promise<void> {
    try {
      await invoke('reset_to_defaults');
      await this.loadProfiles();
      this.notifyListeners('profile_reset', this.getCurrentProfile()!);
    } catch (error) {
      console.error('Failed to reset to defaults:', error);
      throw error;
    }
  }
}

// Create singleton instance
export const keybindingManager = new KeybindingManager();
