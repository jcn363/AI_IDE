/**
 * Unified TypeScript Configuration Management System Index
 */

import type { ValidationResult } from './types';
export { SourcePriority } from './types';
export type { ConfigSource, ValidationResult } from './types';

// Simplified manager for initial implementation
export class ConfigManager<T> {
  private config: T;
  private appName: string;

  constructor(appName: string, config: T) {
    this.appName = appName;
    this.config = config;
  }

  get(): T {
    return this.config;
  }

  update(updater: (config: T) => T): void {
    this.config = updater(this.config);
  }

  async load(): Promise<T> {
    // Simplified implementation - just return current config
    return this.config;
  }

  validate(): ValidationResult {
    return {
      isValid: true,
      errors: [],
      warnings: []
    };
  }

  dispose(): void {
    // Cleanup logic here
  }
}

// Factory function
export function createConfigManager<T>(appName: string, config: T): ConfigManager<T> {
  return new ConfigManager(appName, config);
}

export default {
  ConfigManager,
  createConfigManager
};