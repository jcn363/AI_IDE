import { store } from '../store';
import { executeCargoCommand } from '../store/slices/cargoSlice';

type AllowedCargoCommand =
  | 'build'
  | 'run'
  | 'test'
  | 'check'
  | 'clippy'
  | 'fmt'
  | 'update'
  | 'clean'
  | 'doc'
  | 'add'
  | 'remove';

export class CargoService {
  private static isAllowedCommand(cmd: string): cmd is AllowedCargoCommand {
    return [
      'build',
      'run',
      'test',
      'check',
      'clippy',
      'fmt',
      'update',
      'clean',
      'doc',
      'add',
      'remove',
    ].includes(cmd);
  }

  private static sanitizeArgs(args: unknown[]): string[] {
    return args
      .filter((a): a is string => typeof a === 'string')
      .map((a) => a.trim())
      .filter((a) => a.length > 0);
  }

  private static async dispatch(
    command: AllowedCargoCommand,
    projectPath: string,
    args: string[] = []
  ) {
    if (!this.isAllowedCommand(command)) {
      throw new Error('Program not allowed');
    }
    if (typeof projectPath !== 'string' || projectPath.trim().length === 0) {
      throw new Error('Invalid project path');
    }
    const safeArgs = this.sanitizeArgs(args);
    return store.dispatch(
      executeCargoCommand({
        command,
        args: safeArgs,
        cwd: projectPath,
      })
    );
  }

  static async checkCargoAvailable(): Promise<boolean> {
    try {
      const result = await window.electron.invoke('cargo:check-available');
      return result.available;
    } catch (error) {
      console.error('Failed to check Cargo availability:', error);
      return false;
    }
  }

  static async build(projectPath: string, args: string[] = []) {
    return this.dispatch('build', projectPath, args);
  }

  static async run(projectPath: string, args: string[] = []) {
    return this.dispatch('run', projectPath, args);
  }

  static async test(projectPath: string, args: string[] = []) {
    return this.dispatch('test', projectPath, args);
  }

  static async check(projectPath: string, args: string[] = []) {
    return this.dispatch('check', projectPath, args);
  }

  static async clippy(projectPath: string, args: string[] = []) {
    // Enforce warnings as errors, append user args after the clippy delimiter
    return this.dispatch('clippy', projectPath, ['--', '-D', 'warnings', ...args]);
  }

  static async fmt(projectPath: string, args: string[] = []) {
    // Check formatting without writing changes, append user args after delimiter
    return this.dispatch('fmt', projectPath, ['--', '--check', ...args]);
  }

  static async update(projectPath: string, args: string[] = []) {
    return this.dispatch('update', projectPath, args);
  }

  static async clean(projectPath: string, args: string[] = []) {
    return this.dispatch('clean', projectPath, args);
  }

  static async doc(projectPath: string, args: string[] = []) {
    return this.dispatch('doc', projectPath, ['--open', ...args]);
  }

static async addDependency(
    projectPath: string,
    crateName: string,
    version?: string,
    features: string[] = [],
    options: string[] = []
  ) {
    const baseArgs: string[] = [crateName];

    if (version) {
      baseArgs.push('--version', version);
    }

    if (features.length > 0) {
      baseArgs.push('--features', features.join(','));
    }

    const args = [...baseArgs, ...options];
    return this.dispatch('add', projectPath, args);
  }

  static async removeDependency(projectPath: string, crateName: string) {
    return this.dispatch('remove', projectPath, [crateName]);
  }
}

// Initialize Cargo availability check when the service is imported
CargoService.checkCargoAvailable().then((isAvailable) => {
  // You can dispatch an action to update the store if needed
  console.log('Cargo available:', isAvailable);
});

export default CargoService;