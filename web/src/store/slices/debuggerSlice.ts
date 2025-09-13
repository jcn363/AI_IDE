import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type { RootState } from '../types';

export type DebuggerStateType = 'stopped' | 'running' | 'paused' | 'terminated' | 'error';

export interface BreakpointInfo {
  id: number;
  file: string;
  line: number;
  condition?: string | null;
  enabled: boolean;
  hit_count: number;
}

export interface VariableInfo {
  name: string;
  value: string;
  type_name: string;
  in_scope: boolean;
  children: VariableInfo[];
}

export interface StackFrame {
  id: number;
  function: string;
  file: string;
  line: number;
  column?: number | null;
  args: VariableInfo[];
  locals: VariableInfo[];
}

export interface DebuggerSliceState {
  state: DebuggerStateType;
  reason?: string;
  location?: { file: string; line: number } | null;
  breakpoints: BreakpointInfo[];
  variables: VariableInfo[];
  callStack: StackFrame[];
  output: string[];
  lastEvalResult?: string;
  isOpen: boolean;
}

const initialState: DebuggerSliceState = {
  state: 'stopped',
  reason: undefined,
  location: null,
  breakpoints: [],
  variables: [],
  callStack: [],
  output: [],
  lastEvalResult: undefined,
  isOpen: false,
};

// Event payloads from backend
export type DebuggerEvent =
  | {
      StateChanged:
        | { Stopped: {} }
        | { Running: {} }
        | { Paused: { reason: string; location?: [string, number] } }
        | { Terminated: { exit_code?: number | null } }
        | { Error: { message: string } };
    }
  | { BreakpointHit: { breakpoint: BreakpointInfo; stack_frame: StackFrame } }
  | { OutputReceived: string }
  | { ErrorReceived: string }
  | { VariablesUpdated: VariableInfo[] }
  | { CallStackUpdated: StackFrame[] }
  | { BreakpointChanged: BreakpointInfo };

const debuggerSlice = createSlice({
  name: 'debugger',
  initialState,
  reducers: {
    openDebugger(state) {
      state.isOpen = true;
    },
    closeDebugger(state) {
      state.isOpen = false;
    },
    setState(state, action: PayloadAction<DebuggerSliceState['state']>) {
      state.state = action.payload;
    },
    setBreakpoints(state, action: PayloadAction<BreakpointInfo[]>) {
      state.breakpoints = action.payload;
    },
    setVariables(state, action: PayloadAction<VariableInfo[]>) {
      state.variables = action.payload;
    },
    setCallStack(state, action: PayloadAction<StackFrame[]>) {
      state.callStack = action.payload;
    },
    appendOutput(state, action: PayloadAction<string>) {
      state.output.push(action.payload);
      if (state.output.length > 500) state.output.shift();
    },
    setEvalResult(state, action: PayloadAction<string>) {
      state.lastEvalResult = action.payload;
    },
    applyDebuggerEvent(state, action: PayloadAction<DebuggerEvent>) {
      const ev = action.payload as any;
      if (ev.StateChanged) {
        if ('Running' in ev.StateChanged) state.state = 'running';
        else if ('Stopped' in ev.StateChanged) state.state = 'stopped';
        else if ('Paused' in ev.StateChanged) state.state = 'paused';
        else if ('Terminated' in ev.StateChanged) state.state = 'terminated';
        else if ('Error' in ev.StateChanged) state.state = 'error';
      } else if (ev.BreakpointHit) {
        state.state = 'paused';
        // Optionally update current frame
      } else if (ev.OutputReceived) {
        state.output.push(ev.OutputReceived);
      } else if (ev.ErrorReceived) {
        state.output.push(`[error] ${ev.ErrorReceived}`);
      } else if (ev.VariablesUpdated) {
        state.variables = ev.VariablesUpdated;
      } else if (ev.CallStackUpdated) {
        state.callStack = ev.CallStackUpdated;
      } else if (ev.BreakpointChanged) {
        const idx = state.breakpoints.findIndex((b) => b.id === ev.BreakpointChanged.id);
        if (idx >= 0) state.breakpoints[idx] = ev.BreakpointChanged;
        else state.breakpoints.push(ev.BreakpointChanged);
      }
    },
  },
});

export const debuggerActions = debuggerSlice.actions;

export const selectDebugger = (state: RootState) => state.debugger;

export default debuggerSlice.reducer;
