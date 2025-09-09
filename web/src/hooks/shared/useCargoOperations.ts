import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppSelector } from '../../store/store';
import { selectCargoCommands, selectIsCargoAvailable, selectCargoState } from '../../store/slices/cargoSlice';
import type { CargoCommandName, CargoCommand } from '../../store/slices/cargoSlice';

export const useCargoOperations = () => {
  const commands = useAppSelector(selectCargoCommands);
  const isCargoAvailable = useAppSelector(selectIsCargoAvailable);
  const isRunning = useAppSelector(selectCargoState).isLoading;

  const executeCommand = useCallback(async (command: CargoCommandName, cwd: string) => {
    await invoke('run_cargo_command', { command, args: '', cwd });
  }, []);

  return { executeCommand, commands, isCargoAvailable, isRunning, getCommandById: (id: string) => commands[id] };
};