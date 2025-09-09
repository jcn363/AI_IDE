/**
 * Component responsible for managing and executing Cargo commands
 * Handles command selection, execution, and command history display
 */

import React from 'react';
import { useSelector } from 'react-redux';
import { CargoCommandName, executeCargoStream, selectCargoCommands } from '../../store/slices/cargoSlice';
import { useAppDispatch } from '../../store/store';

interface CargoCommandManagerProps {
  projectPath: string;
  selectedCommand: CargoCommandName;
  commandArgs: string;
  onCommandChange: (command: CargoCommandName) => void;
  onArgsChange: (args: string) => void;
  onJsonDiagnosticsChange: (enabled: boolean) => void;
  jsonDiagnostics: boolean;
  isRunning: boolean;
  isCargoAvailable: boolean;
  error: string;
  onClearError: () => void;
}

/**
 * Manages the execution of Cargo commands and provides UI for command configuration
 */
export const CargoCommandManager: React.FC<CargoCommandManagerProps> = ({
  projectPath,
  selectedCommand,
  commandArgs,
  onCommandChange,
  onArgsChange,
  onJsonDiagnosticsChange,
  jsonDiagnostics,
  isRunning,
  isCargoAvailable,
  error,
  onClearError,
}) => {
  const dispatch = useAppDispatch();
  const commands = useSelector(selectCargoCommands);

  const handleCommandExecute = async () => {
    if (!projectPath) {
      // Handle error
      return;
    }

    const args = commandArgs ? commandArgs.split(' ').filter(arg => arg.trim() !== '') : [];
    const jsonCapable = ['build', 'check', 'test', 'clippy', 'run'];

    if (jsonDiagnostics && jsonCapable.includes(selectedCommand)) {
      if (!args.includes('--message-format=json')) {
        args.unshift('--message-format=json');
      }
    }

    try {
      onClearError();
      await dispatch(
        executeCargoStream({
          command: selectedCommand,
          args,
          cwd: projectPath,
        }),
      );
    } catch (err) {
      // Error is handled by the store/reducers
      console.error('Command execution failed:', err);
    }
  };

  return (
    <div className="cargo-command-manager">
      {/* Command selector and execution controls would go here */}
      {/* Implementation details would follow the existing command section from CargoPanel */}
    </div>
  );
};

export default CargoCommandManager;