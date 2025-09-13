import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type { ShortcutState, KeyCombination, UserShortcutProfile } from './types';
import { keyboardService } from './KeyboardService';

const initialState: ShortcutState = {
  currentProfile: 'default',
  profiles: {},
  conflicts: [],
  recordingMode: false,
  recordingFor: undefined,
  lastRecordedKey: undefined,
};

const keyboardSlice = createSlice({
  name: 'keyboard',
  initialState,
  reducers: {
    setCurrentProfile: (state, action: PayloadAction<string>) => {
      const profileId = action.payload;
      if (keyboardService.getAllProfiles().find((p) => p.id === profileId)) {
        state.currentProfile = profileId;
        keyboardService.switchProfile(profileId);
      }
    },

    createProfile: (
      state,
      action: PayloadAction<Omit<UserShortcutProfile, 'id' | 'createdAt' | 'updatedAt'>>
    ) => {
      const profile = keyboardService.createProfile(action.payload);
      state.profiles[profile.id] = profile;
    },

    deleteProfile: (state, action: PayloadAction<string>) => {
      const profileId = action.payload;
      if (profileId !== 'default' && state.profiles[profileId]) {
        delete state.profiles[profileId];
        keyboardService.deleteProfile(profileId);

        // If deleted current profile, switch to default
        if (state.currentProfile === profileId) {
          state.currentProfile = 'default';
          keyboardService.switchProfile('default');
        }
      }
    },

    setShortcut: (state, action: PayloadAction<{ actionId: string; keys: KeyCombination[] }>) => {
      const { actionId, keys } = action.payload;
      keyboardService.setShortcut(actionId, keys);

      // Update conflicts
      state.conflicts = keyboardService.detectConflicts();

      // Exit recording mode if active
      if (state.recordingMode && state.recordingFor === actionId) {
        state.recordingMode = false;
        state.recordingFor = undefined;
        state.lastRecordedKey = undefined;
      }
    },

    clearShortcut: (state, action: PayloadAction<string>) => {
      const actionId = action.payload;
      keyboardService.clearShortcut(actionId);

      // Update conflicts
      state.conflicts = keyboardService.detectConflicts();
    },

    startRecording: (state, action: PayloadAction<string>) => {
      state.recordingMode = true;
      state.recordingFor = action.payload;
      state.lastRecordedKey = undefined;
    },

    stopRecording: (state) => {
      state.recordingMode = false;
      state.recordingFor = undefined;
      state.lastRecordedKey = undefined;
    },

    recordKey: (state, action: PayloadAction<KeyCombination>) => {
      state.lastRecordedKey = action.payload;
    },

    loadProfiles: (state) => {
      const profiles = keyboardService.getAllProfiles();
      state.profiles = {};
      profiles.forEach((profile) => {
        if (!profile.isDefault) {
          state.profiles[profile.id] = profile;
        }
      });

      const currentProfile = keyboardService.getCurrentProfile();
      if (currentProfile) {
        state.currentProfile = currentProfile.id;
      }

      state.conflicts = keyboardService.detectConflicts();
    },

    setConflicts: (state, action: PayloadAction<Array<{ keys: string; actions: string[] }>>) => {
      state.conflicts = action.payload;
    },
  },
});

export const {
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
} = keyboardSlice.actions;

export default keyboardSlice.reducer;

// Selectors
export const selectKeyboard = (state: any) => state.keyboard;
export const selectCurrentProfile = (state: any) => {
  const profiles = keyboardService.getAllProfiles();
  return (
    profiles.find((p) => p.id === state.keyboard.currentProfile) ||
    profiles.find((p) => p.isDefault)
  );
};
export const selectProfiles = (state: any) => {
  const profiles = keyboardService.getAllProfiles();
  return {
    default: profiles.find((p) => p.isDefault),
    user: profiles.filter((p) => !p.isDefault),
  };
};
export const selectShortcuts = (state: any) => {
  const currentProfile = selectCurrentProfile(state);
  return currentProfile?.shortcuts || {};
};
export const selectConflicts = (state: any) => state.keyboard.conflicts;
export const selectRecordingMode = (state: any) => state.keyboard.recordingMode;
export const selectRecordingFor = (state: any) => state.keyboard.recordingFor;
export const selectShortcut = (state: any, actionId: string) => {
  const shortcuts = selectShortcuts(state);
  return shortcuts[actionId] || [];
};
