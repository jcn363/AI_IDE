import React, { useState, useEffect, useCallback } from 'react';
import { keybindingManager, KeybindingManager } from '../KeybindingManager';
import {
  ShortcutContext,
  KeyCombination,
  ShortcutAction,
  UserShortcutProfile,
  KeybindingConflict
} from '../types';
import './KeybindingSettings.css';

interface KeybindingSettingsProps {
  onClose: () => void;
}

const KeybindingSettings: React.FC<KeybindingSettingsProps> = ({ onClose }) => {
  const [currentProfile, setCurrentProfile] = useState<UserShortcutProfile | null>(null);
  const [availableActions, setAvailableActions] = useState<ShortcutAction[]>([]);
  const [profiles, setProfiles] = useState<UserShortcutProfile[]>([]);
  const [conflicts, setConflicts] = useState<KeybindingConflict[]>([]);
  const [recordingFor, setRecordingFor] = useState<string | null>(null);
  const [selectedProfileId, setSelectedProfileId] = useState<string>('default');
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [showImportDialog, setShowImportDialog] = useState(false);
  const [newProfileName, setNewProfileName] = useState('');
  const [newProfileDescription, setNewProfileDescription] = useState('');

  useEffect(() => {
    loadKeybindingData();
    setupEventListeners();
  }, []);

  const loadKeybindingData = async () => {
    try {
      await keybindingManager.loadProfiles();
      const profile = keybindingManager.getCurrentProfile();
      const actions = keybindingManager.getAvailableActions();
      const allProfiles = keybindingManager.getAllProfiles();

      setCurrentProfile(profile);
      setAvailableActions(actions);
      setProfiles(allProfiles);

      if (profile) {
        setSelectedProfileId(profile.id);
        const profileConflicts = await keybindingManager.detectConflicts(profile.id);
        setConflicts(profileConflicts);
      }
    } catch (error) {
      console.error('Failed to load keybinding data:', error);
    }
  };

  const setupEventListeners = () => {
    keybindingManager.addEventListener('profile_switched', (profile) => {
      setCurrentProfile(profile);
      setSelectedProfileId(profile.id);
    });

    keybindingManager.addEventListener('shortcuts_updated', (profile) => {
      setCurrentProfile(profile);
    });
  };

  const handleKeyPress = useCallback((event: KeyboardEvent) => {
    if (!recordingFor) return;

    event.preventDefault();
    event.stopPropagation();

    const combination: KeyCombination = {
      key: event.key.toLowerCase(),
      ctrlKey: event.ctrlKey,
      altKey: event.altKey,
      shiftKey: event.shiftKey,
      metaKey: event.metaKey
    };

    // Update the keybinding for the recording action
    keybindingManager.setShortcut(recordingFor, [combination]);
    setRecordingFor(null);

    // Update conflicts
    if (currentProfile) {
      loadConficts(currentProfile.id);
    }
  }, [recordingFor, currentProfile]);

  useEffect(() => {
    if (recordingFor) {
      document.addEventListener('keydown', handleKeyPress);
      return () => {
        document.removeEventListener('keydown', handleKeyPress);
      };
    }
  }, [recordingFor, handleKeyPress]);

  const loadConficts = async (profileId: string) => {
    const profileConflicts = await keybindingManager.detectConflicts(profileId);
    setConflicts(profileConflicts);
  };

  const handleProfileSwitch = async (profileId: string) => {
    try {
      await keybindingManager.switchProfile(profileId);
      await loadKeybindingData();
    } catch (error) {
      console.error('Failed to switch profile:', error);
    }
  };

  const handleCreateProfile = async () => {
    if (!newProfileName.trim()) return;

    try {
      const profile = await keybindingManager.createProfile(
        newProfileName.trim(),
        newProfileDescription.trim()
      );
      setProfiles(prev => [...prev, profile]);
      setNewProfileName('');
      setNewProfileDescription('');
    } catch (error) {
      console.error('Failed to create profile:', error);
    }
  };

  const handleDeleteProfile = async (profileId: string) => {
    if (profileId === 'default') return;

    try {
      await keybindingManager.deleteProfile(profileId);
      await loadKeybindingData();
    } catch (error) {
      console.error('Failed to delete profile:', error);
    }
  };

  const startRecording = (actionId: string) => {
    setRecordingFor(actionId);
  };

  const clearShortcut = (actionId: string) => {
    keybindingManager.clearShortcut(actionId);
    if (currentProfile) {
      loadConficts(currentProfile.id);
    }
  };

  const handleExportProfile = async () => {
    if (!currentProfile) return;

    try {
      const exportedData = await keybindingManager.exportProfile(currentProfile.id);
      const blob = new Blob([exportedData], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${currentProfile.name.replace(/\s+/g, '_')}_keybindings.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export profile:', error);
    }
  };

  const handleImportFile = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string;
        await keybindingManager.importProfile(content);
        await loadKeybindingData();
        setShowImportDialog(false);
      } catch (error) {
        console.error('Failed to import profile:', error);
      }
    };
    reader.readAsText(file);
  };

  const resetToDefaults = async () => {
    try {
      await keybindingManager.resetToDefaults();
      await loadKeybindingData();
    } catch (error) {
      console.error('Failed to reset to defaults:', error);
    }
  };

  const currentKeys = (actionId: string): string[] => {
    if (!currentProfile) return [];
    const shortcuts = currentProfile.shortcuts[actionId] || [];
    return shortcuts.map(KeybindingManager.formatKeyCombination);
  };

  const isConflicted = (actionId: string): boolean => {
    return conflicts.some(conflict =>
      conflict.actions.some(action => action === actionId)
    );
  };

  return (
    <div className="keybinding-settings">
      <div className="keybinding-header">
        <h2>Keyboard Shortcuts</h2>
        <button onClick={onClose} className="close-button">×</button>
      </div>

      <div className="keybinding-content">
        {/* Profile Management */}
        <div className="profiles-section">
          <h3>Keybinding Profiles</h3>
          <div className="profile-controls">
            <select
              value={selectedProfileId}
              onChange={(e) => handleProfileSwitch(e.target.value)}
              className="profile-select"
            >
              {profiles.map(profile => (
                <option key={profile.id} value={profile.id}>
                  {profile.name} {profile.isDefault ? '(Default)' : ''}
                </option>
              ))}
            </select>
            <button onClick={() => setShowImportDialog(true)} className="action-button">
              Import
            </button>
          </div>

          {/* Create New Profile */}
          <div className="create-profile">
            <input
              type="text"
              placeholder="New profile name"
              value={newProfileName}
              onChange={(e) => setNewProfileName(e.target.value)}
              className="profile-name-input"
            />
            <input
              type="text"
              placeholder="Description (optional)"
              value={newProfileDescription}
              onChange={(e) => setNewProfileDescription(e.target.value)}
              className="profile-desc-input"
            />
            <button onClick={handleCreateProfile} className="action-button">
              Create Profile
            </button>
          </div>

          {/* Profile Actions */}
          <div className="profile-actions">
            <button onClick={handleExportProfile} className="action-button">
              Export Profile
            </button>
            <button onClick={resetToDefaults} className="reset-button">
              Reset to Defaults
            </button>
          </div>
        </div>

        {/* Conflicts Section */}
        {conflicts.length > 0 && (
          <div className="conflicts-section">
            <h3>⚠️ Shortcut Conflicts</h3>
            {conflicts.map((conflict, index) => (
              <div key={index} className="conflict-item">
                <strong>{conflict.keys}:</strong> {conflict.actions.join(', ')}
              </div>
            ))}
          </div>
        )}

        {/* Search */}
        <div className="search-section">
          <input
            type="text"
            placeholder="Search shortcuts..."
            className="search-input"
          />
        </div>

        {/* Shortcuts List */}
        <div className="shortcuts-section">
          <h3>Keyboard Shortcuts</h3>
          <div className="shortcuts-grid">
            {availableActions.map(action => (
              <div key={action.id} className={`shortcut-item ${isConflicted(action.id) ? 'conflicted' : ''}`}>
                <div className="shortcut-info">
                  <span className="action-name">{action.name}</span>
                  <span className="action-description">{action.description}</span>
                </div>
                <div className="shortcut-keys">
                  {currentKeys(action.id).map((formatted, index) => (
                    <kbd key={index}>{formatted}</kbd>
                  ))}
                </div>
                <div className="shortcut-actions">
                  {recordingFor === action.id ? (
                    <button className="recording-button">Press keys...</button>
                  ) : (
                    <button onClick={() => startRecording(action.id)} className="record-button">
                      Record
                    </button>
                  )}
                  <button onClick={() => clearShortcut(action.id)} className="clear-button">
                    Clear
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Export Dialog */}
      {showExportDialog && (
        <div className="modal-overlay">
          <div className="modal-content">
            <h3>Export Keybindings</h3>
            <p>Export will download a JSON file containing your current keybindings.</p>
            <div className="modal-buttons">
              <button onClick={() => setShowExportDialog(false)}>Cancel</button>
              <button onClick={handleExportProfile}>Export</button>
            </div>
          </div>
        </div>
      )}

      {/* Import Dialog */}
      {showImportDialog && (
        <div className="modal-overlay">
          <div className="modal-content">
            <h3>Import Keybindings</h3>
            <p>Select a JSON file containing your keybinding configuration.</p>
            <input
              type="file"
              accept=".json"
              onChange={handleImportFile}
              className="file-input"
            />
            <div className="modal-buttons">
              <button onClick={() => setShowImportDialog(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default KeybindingSettings;