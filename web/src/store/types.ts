// Import types directly to avoid circular dependencies
import type { EditorState } from './slices/editorSlice';
import type { TabManagementState } from './slices/tabManagementSlice';
import type { CargoState } from './slices/cargoSlice';
import type { DebuggerSliceState } from './slices/debuggerSlice';
import type { ProjectsState } from './slices/projectsSlice';
import type { VersionAlignmentState } from '../features/dependency/types';
import type { AsyncThunk, UnknownAction } from '@reduxjs/toolkit';
import type { ThunkAction } from 'redux-thunk';

// Base type for async thunks
type AppThunk<ReturnType = void, Arg = unknown> = ThunkAction<
  Promise<ReturnType>,
  RootState,
  unknown,
  UnknownAction
> & {
  unwrap: () => Promise<ReturnType>;
};

// Helper type to extract the return type of an async thunk
type AsyncThunkReturnType<T> = T extends AsyncThunk<infer Returned, any, any> 
  ? Returned 
  : never;

type AsyncThunkArg<T> = T extends AsyncThunk<any, infer Arg, any> 
  ? Arg 
  : never;

export type { AppThunk };

export interface RootState {
  editor: EditorState;
  tabManagement: TabManagementState;
  cargo: CargoState;
  debugger: DebuggerSliceState;
  projects: ProjectsState;
  dependency: {
    versionAlignment: VersionAlignmentState;
  };
}
