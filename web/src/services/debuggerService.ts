import { invoke } from '@tauri-apps/api/core';

export const DebuggerService = {
  async start(executable_path: string, working_directory: string, args: string[] = []) {
    return invoke('start_debug_session', { executable_path, working_directory, args });
  },
  async run() { return invoke('debug_run'); },
  async cont() { return invoke('debug_continue'); },
  async pause() { return invoke('debug_pause'); },
  async stop() { return invoke('debug_stop'); },
  async stepOver() { return invoke('debug_step_over'); },
  async stepInto() { return invoke('debug_step_into'); },
  async stepOut() { return invoke('debug_step_out'); },
  async setBreakpoint(file: string, line: number, condition?: string) {
    return invoke<number>('debugger_set_breakpoint', { file, line, condition: condition ?? null });
  },
  async removeBreakpoint(id: number) { return invoke('debugger_remove_breakpoint', { id }); },
  async toggleBreakpoint(id: number) { return invoke('debugger_toggle_breakpoint', { id }); },
  async evaluate(expression: string) { return invoke<string>('debugger_evaluate', { expression }); },
  async setVariable(name: string, value: string) { return invoke('debugger_set_variable', { name, value }); },
  async selectFrame(frameId: number) { return invoke('debugger_select_frame', { frameId }); },
  async getVariables(scope: string = 'local') { return invoke('debugger_get_variables', { scope }); },
  async getCallStack() { return invoke('debugger_get_call_stack'); },
  async getBreakpoints() { return invoke('debugger_get_breakpoints'); },
  async getState() { return invoke('debugger_get_state'); },
  // Variable object APIs for lazy expansion
  async varCreate(expression: string) { return invoke<string>('debugger_var_create', { expression }); },
  async varDelete(name: string) { return invoke('debugger_var_delete', { name }); },
  async varChildren(name: string, allValues: boolean = true) { return invoke<unknown>('debugger_var_children', { name, allValues }); },
};
