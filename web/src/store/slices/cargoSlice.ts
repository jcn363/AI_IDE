import { createSlice, PayloadAction, createAsyncThunk } from '@reduxjs/toolkit';
import type { RootState } from '../types';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export type CargoCommandName = 
  | 'build' 
  | 'run' 
  | 'test' 
  | 'check' 
  | 'clippy' 
  | 'fmt' 
  | 'doc' 
  | 'clean' 
  | 'update' 
  | 'add' 
  | 'remove';

export interface CargoCommand {
  id: string;
  command: string;
  args: string[];
  cwd: string;
  status: 'idle' | 'running' | 'success' | 'error' | 'cancelled';
  output: string;
  error?: string;
  timestamp: number;
  diagnostics?: CargoDiagnostic[];
  // lightweight parsed stats for UI
  stats?: {
    compiling: number;
    fresh: number;
    finished: number;
  };
}

export interface CargoState {
  commands: Record<string, CargoCommand>;
  currentProjectPath: string | null;
  isCargoAvailable: boolean;
  isLoading: boolean;
  error: string | null;
}

export interface CargoDiagnosticSpan {
  file_name: string;
  line_start: number;
  line_end: number;
  column_start: number;
  column_end: number;
}

export interface CargoDiagnostic {
  level: string;
  message: string;
  spans: CargoDiagnosticSpan[];
  code?: { code: string; explanation?: string } | null;
}

const initialState: CargoState = {
  commands: {},
  currentProjectPath: null,
  isCargoAvailable: false,
  isLoading: false,
  error: null,
};

export const executeCargoCommand = createAsyncThunk(
  'cargo/executeCommand',
  async (
    { command, args = [], cwd }: { command: CargoCommandName; args?: string[]; cwd: string },
    { rejectWithValue }
  ) => {
    try {
      // Tauri command for non-streaming execution (kept for compatibility)
      const response = await invoke<any>('cargo_execute_command', {
        command,
        args,
        directory: cwd,
      });
      return response;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred';
      return rejectWithValue(errorMessage);
    }
  }
);

// Streaming thunk: fire-and-forget; events will populate the store
export const executeCargoStream = createAsyncThunk(
  'cargo/executeStream',
  async (
    { command, args = [], cwd, commandId }: { command: CargoCommandName; args?: string[]; cwd: string; commandId?: string },
  ) => {
    await invoke('cargo_execute_stream', {
      command,
      args,
      directory: cwd,
      command_id: commandId,
    });
    return { ok: true };
  }
);

// Cancel a running cargo command by its id
export const cancelCargoCommand = createAsyncThunk(
  'cargo/cancelCommand',
  async ({ id }: { id: string }) => {
    const ok = await invoke<boolean>('cargo_cancel_command', { command_id: id });
    return { id, ok } as { id: string; ok: boolean };
  }
);

const cargoSlice = createSlice({
  name: 'cargo',
  initialState,
  reducers: {
    setCurrentProjectPath: (state, action: PayloadAction<string | null>) => {
      state.currentProjectPath = action.payload;
    },
    clearCommandOutput: (state, action: PayloadAction<{ commandId: string }>) => {
      const { commandId } = action.payload;
      if (commandId === 'all') {
        // Clear all commands
        state.commands = {};
      } else if (state.commands[commandId]) {
        // Clear specific command
        state.commands[commandId].output = '';
        state.commands[commandId].error = undefined;
      }
    },
    clearError: (state) => {
      state.error = null;
    },
    streamStarted: (
      state,
      action: PayloadAction<{ id: string; command: string; args: string[]; cwd: string; ts: number }>
    ) => {
      const { id, command, args, cwd, ts } = action.payload;
      state.commands[id] = {
        id,
        command,
        args,
        cwd,
        status: 'running',
        output: '',
        timestamp: ts,
        diagnostics: [],
        stats: { compiling: 0, fresh: 0, finished: 0 },
      };
      state.isLoading = true;
    },
    streamAppended: (
      state,
      action: PayloadAction<{ id: string; stream: 'stdout' | 'stderr'; line: string }>
    ) => {
      const { id, stream, line } = action.payload;
      if (!state.commands[id]) return;
      const prefix = stream === 'stderr' ? '[stderr] ' : '';
      state.commands[id].output += (state.commands[id].output ? '\n' : '') + prefix + line;
      // parse simple progress hints from plaintext output for UI chips
      const stats = state.commands[id].stats || { compiling: 0, fresh: 0, finished: 0 };
      if (line.includes('Compiling ')) stats.compiling += 1;
      if (line.includes('Fresh ')) stats.fresh += 1;
      if (line.startsWith('Finished ')) stats.finished += 1;
      state.commands[id].stats = stats;
    },
    diagnosticAppended: (
      state,
      action: PayloadAction<{ id: string; diagnostic: CargoDiagnostic }>
    ) => {
      const { id, diagnostic } = action.payload;
      if (!state.commands[id]) return;
      if (!state.commands[id].diagnostics) state.commands[id].diagnostics = [];
      state.commands[id].diagnostics!.push(diagnostic);
    },
    streamFinished: (state, action: PayloadAction<{ id: string; code: number }>) => {
      const { id, code } = action.payload;
      if (!state.commands[id]) return;
      // Do not override cancelled status if we've already marked it
      if (state.commands[id].status !== 'cancelled') {
        state.commands[id].status = code === 0 ? 'success' : 'error';
      }
      state.isLoading = false;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(executeCargoCommand.pending, (state, action) => {
        const { meta } = action;
        const commandId = meta.requestId;
        state.commands[commandId] = {
          id: commandId,
          command: meta.arg.command,
          args: meta.arg.args || [],
          cwd: meta.arg.cwd,
          status: 'running',
          output: '',
          timestamp: Date.now(),
        };
        state.isLoading = true;
      })
      .addCase(executeCargoCommand.fulfilled, (state, action) => {
        const commandId = action.meta.requestId;
        if (state.commands[commandId]) {
          state.commands[commandId].status = 'success';
          // payload may be tuple or object depending on backend; normalize
          const p: any = action.payload;
          state.commands[commandId].output = typeof p === 'object' && 'output' in p ? p.output : String(p ?? '');
          state.isLoading = false;
        }
      })
      .addCase(executeCargoCommand.rejected, (state, action) => {
        const commandId = action.meta.requestId;
        if (state.commands[commandId]) {
          state.commands[commandId].status = 'error';
          state.commands[commandId].error = action.error.message;
          state.isLoading = false;
        }
      })
      .addCase(cancelCargoCommand.fulfilled, (state, action) => {
        const { id, ok } = action.payload as { id: string; ok: boolean };
        if (state.commands[id] && ok) {
          state.commands[id].status = 'cancelled';
        }
      });
  },
});

export const { 
  setCurrentProjectPath, 
  clearCommandOutput,
  clearError,
  streamStarted,
  streamAppended,
  diagnosticAppended,
  streamFinished,
} = cargoSlice.actions;

export const selectCargoState = (state: RootState) => state.cargo;
export const selectCurrentProjectPath = (state: RootState) => state.cargo.currentProjectPath;
export const selectIsCargoAvailable = (state: RootState) => state.cargo.isCargoAvailable;
export const selectCargoCommands = (state: RootState) => state.cargo.commands;

export default cargoSlice.reducer;

// Listener registration (call once during app bootstrap)
let unlistenFns: UnlistenFn[] = [];
export async function initCargoStreamingListeners(dispatch: (a: any) => void) {
  // Prevent duplicate listeners
  if (unlistenFns.length > 0) return;

  const un1 = await listen('cargo:command-start', (event: any) => {
    const p: any = event.payload;
    const id: string = p.command_id ?? p.commandId ?? p.commandid ?? p.id;
    dispatch(
      streamStarted({
        id,
        command: p.command,
        args: p.args || [],
        cwd: p.cwd || '',
        ts: p.ts || Date.now(),
      })
    );
  });

  const un2 = await listen('cargo:command-output', (event: any) => {
    const p: any = event.payload;
    const id: string = p.commandId ?? p.command_id ?? p.id;
    dispatch(
      streamAppended({
        id,
        stream: (p.stream as 'stdout' | 'stderr') || 'stdout',
        line: p.line || '',
      })
    );
  });

  const un3 = await listen('cargo:command-finish', (event: any) => {
    const p: any = event.payload;
    const id: string = p.commandId ?? p.command_id ?? p.id;
    dispatch(
      streamFinished({
        id,
        code: typeof p.code === 'number' ? p.code : -1,
      })
    );
  });

  const un4 = await listen('cargo:command-diagnostic', (event: any) => {
    const p: any = event.payload;
    const id: string = p.commandId ?? p.command_id ?? p.id;
    const payload = p.payload || p;
    const diag: CargoDiagnostic = {
      level: payload.level || payload.message?.level || 'unknown',
      message: payload.message || payload.rendered || '',
      spans: (payload.spans || []).map((s: any) => ({
        file_name: s.file_name,
        line_start: s.line_start,
        line_end: s.line_end,
        column_start: s.column_start,
        column_end: s.column_end,
      })),
      code: payload.code || null,
    };
    dispatch(diagnosticAppended({ id, diagnostic: diag }));
  });

  unlistenFns = [un1, un2, un3, un4];
}
