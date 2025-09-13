import { configureStore } from '@reduxjs/toolkit';
import { useDispatch, TypedUseSelectorHook, useSelector } from 'react-redux';
import { info } from '@/utils/logging/config';

// Reducers
import editorReducer from './slices/editorSlice';
import tabManagementReducer from './slices/tabManagementSlice';
import cargoReducer from './slices/cargoSlice';
import debuggerReducer, { debuggerActions, type DebuggerEvent } from './slices/debuggerSlice';
import projectsReducer from './slices/projectsSlice';
import codegenReducer from './slices/codegenSlice';

// Types and utilities
import type { RootState } from './types';
import { initCargoStreamingListeners } from './slices/cargoSlice';
import { listen } from '@tauri-apps/api/event';
import { middleware as loggerMiddleware } from './middleware/loggerMiddleware';

export const store = configureStore({
  reducer: {
    editor: editorReducer,
    tabManagement: tabManagementReducer,
    cargo: cargoReducer,
    debugger: debuggerReducer,
    projects: projectsReducer,
    codegen: codegenReducer,
  },
  middleware: (getDefaultMiddleware) => {
    const middlewares = getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: [
          'cargo/executeCommand/pending',
          'cargo/executeCommand/fulfilled',
          'cargo/executeCommand/rejected',
        ],
      },
    });

    // Add logger middleware in non-production environments
    if (process.env.NODE_ENV !== 'production') {
      return middlewares.concat(loggerMiddleware);
    }
    return middlewares;
  },
});

export type AppDispatch = typeof store.dispatch;

// Export typed hooks for use throughout the app
export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;

export default store;

// Initialize Tauri event listeners for streaming once
initCargoStreamingListeners(store.dispatch).catch((err) => {
  // Log the error but continue execution
  info('Failed to initialize cargo streaming listeners', {
    error: err?.message || 'Unknown error',
  });
});

// Initialize debugger event listener
(async () => {
  try {
    await listen('debugger-event', (event) => {
      const payload = event.payload as DebuggerEvent;
      store.dispatch(debuggerActions.applyDebuggerEvent(payload));
    });
    info('Debugger event listener initialized');
  } catch (error) {
    info('Debugger event listener initialization failed', {
      error: error instanceof Error ? error.message : 'Unknown error',
    });
  }
})();
