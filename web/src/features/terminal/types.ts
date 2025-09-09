export interface TerminalProps {
  terminalOpen: boolean;
  terminalProgram: string;
  terminalArgs: string;
  terminalDir: string;
  terminalId: string;
  terminalLines: string[];
  onTerminalProgramChange: (value: string) => void;
  onTerminalArgsChange: (value: string) => void;
  onTerminalDirChange: (value: string) => void;
  onStartTerminal: () => void;
  onCloseTerminal: () => void;
}
