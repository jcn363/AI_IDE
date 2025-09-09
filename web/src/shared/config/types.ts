/**
 * # TypeScript Configuration Management System
 *
 * This module provides a unified, type-safe configuration management system
 * that mirrors the Rust configuration system for the web frontend.
 *
 * ## Features
 *
 * - **Multi-format support**: Load from JSON and YAML files
 * - **Environment variable overrides**: Override any config value using environment variables
 * - **Browser-compatible paths**: Discover config files based on web standards
 * - **Schema validation**: Validate configuration using JSON Schema
 * - **Merge strategies**: Intelligent merging of multiple configuration sources
 * - **Hot-reload capability**: Watch config files for changes and auto-reload
 * - **Type-safe access**: Strongly-typed configuration access with fallbacks
 *
 * ## Usage
 *
 * ```typescript
 * import { ConfigManager } from '@/shared/config';
 * import type { AppConfig } from '@/shared/types/app';
 *
 * // Load configuration with default settings
 * const configManager = new ConfigManager<AppConfig>('app', {
 *   apiUrl: 'https://api.example.com',
 *   debug: false,
 *   features: ['basic']
 * });
 *
 * // Load with environment overrides
 * await configManager.loadWithOverrides(['API_URL', 'DEBUG_ENABLED', 'FEATURE_FLAGS']);
 * ```
 */

/**
 * Configuration format types
 */
export type ConfigFormat = 'json' | 'yaml' | 'auto';

/**
 * Configuration source priority levels
 */
export enum SourcePriority {
  DEFAULT = 0,
  GLOBAL_USER = 1,
  PROJECT = 2,
  ENVIRONMENT = 3,
  RUNTIME = 4
}

/**
 * Configuration source with associated priority and metadata
 */
export interface ConfigSource<T = any> {
  data: T;
  priority: SourcePriority;
  sourcePath?: string;
  loadedAt: Date;
  format?: ConfigFormat;
}

/**
 * Configuration loading options
 */
export interface LoadOptions {
  /** Enable environment variable overrides */
  enableEnvOverride?: boolean;
  /** Environment variable prefix (default: 'APP_') */
  envPrefix?: string;
  /** Enable hot-reloading of config files */
  enableHotReload?: boolean;
  /** Hot-reload debounce time in milliseconds */
  hotReloadDebounceMs?: number;
  /** Search includes patterns */
  searchIncludes?: string[];
  /** Search excludes patterns */
  searchExcludes?: string[];
}

/**
 * Merge strategy for combining configuration sources
 */
export type MergeStrategy = 'priority-take' | 'deep-merge' | ((high: any, low: any) => any);

/**
 * File change event for hot reloading
 */
export interface ConfigFileChange {
  path: string;
  previousConfig: any;
  newConfig: any;
  changeTime: Date;
}

/**
 * Configuration validation result
 */
export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Configuration trait for type-safe management
 */
export interface Configurable {
  readonly FILE_PREFIX: string;
  readonly DESCRIPTION: string;

  /** Validate the configuration */
  validate?(): ValidationResult;

  /** Default configuration */
  getDefaultConfig(): any;

  /** Transform raw configuration before merging */
  transform?(config: any): any;
}

/**
 * Configuration manager interface
 */
export interface IConfigManager<T = any> {
  /** Current configuration */
  get(): T;

  /** Update configuration with validation */
  update(updater: (config: T) => T): Promise<void>;

  /** Load configuration from all sources */
  load(options?: LoadOptions): Promise<T>;

  /** Load with environment overrides */
  loadWithOverrides(envVars: string[], options?: LoadOptions): Promise<T>;

  /** Save current configuration */
  save(path?: string, format?: ConfigFormat): Promise<void>;

  /** Get configuration validation result */
  validate(): ValidationResult;

  /** Subscribe to configuration changes */
  subscribe(callback: (config: T, change?: ConfigFileChange) => void): () => void;

  /** Reload configuration */
  reload(): Promise<void>;
}

/**
 * Configuration loading context for tracking sources
 */
export interface ConfigLoadingContext {
  sources: ConfigSource[];
  loadedPaths: string[];
  envOverrides: Record<string, string>;
  errors: string[];
  warnings: string[];
}

/**
 * Platform-specific path resolver interface
 */
export interface IPathResolver {
  /** Get user config directory */
  getUserConfigDir(): Promise<string>;

  /** Get project config directory */
  getProjectConfigDir(): Promise<string>;

  /** Resolve all possible config file paths */
  resolveConfigPaths(appName: string): Promise<string[]>;
}

/**
 * Hot reload monitor interface
 */
export interface IHotReloadMonitor {
  /** Start monitoring for config changes */
  start(onChange: (change: ConfigFileChange) => void): Promise<void>;

  /** Stop monitoring */
  stop(): Promise<void>;

  /** Watch specific paths */
  watchPaths(paths: string[]): void;
}

/**
 * Environment variable resolver interface
 */
export interface IEnvResolver {
  /** Get an environment variable value */
  getValue(key: string): string | null;

  /** Get all environment variables matching a prefix */
  getWithPrefix(prefix: string): Record<string, string>;

  /** Check if an environment variable exists */
  has(key: string): boolean;
}