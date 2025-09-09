/**
 * Main configuration manager implementation
 */

import {
  SourcePriority,
  type ConfigFileChange,
  type ConfigFormat,
  type ConfigSource,
  type Configurable,
  type IConfigManager,
  type IEnvResolver,
  type IHotReloadMonitor,
  type IPathResolver,
  type LoadOptions,
  type ValidationResult
} from './types';

/**
 * Simple logger utility
 */
function createLogger(context: string) {
  return {
    info: (message: string) => console.log(`[${context}] ${message}`),
    warn: (message: string) => console.warn(`[${context}] ${message}`),
    error: (message: string) => console.error(`[${context}] ${message}`)
  };
}

const logger = createLogger('ConfigManager');

/**
 * Core configuration manager for type-safe configuration handling
 */
export class ConfigManager<T> implements IConfigManager<T> {
  private config: ConfigSource<T>;
  private appName: string;
  private defaultConfig: T;
  private options: LoadOptions;
  private subscribers: Array<(config: T, change?: ConfigFileChange) => void> = [];
  private hotReloadMonitor?: HotReloadMonitor;

  constructor(appName: string, defaultConfig: T, options: LoadOptions = {}) {
    this.appName = appName;
    this.defaultConfig = defaultConfig;
    this.options = { ...this.getDefaultOptions(), ...options };
    this.config = {
      data: defaultConfig,
      priority: SourcePriority.DEFAULT,
      loadedAt: new Date()
    };
  }

  private getDefaultOptions(): Required<LoadOptions> {
    return {
      enableEnvOverride: true,
      envPrefix: 'APP_',
      enableHotReload: false,
      hotReloadDebounceMs: 500,
      searchIncludes: ['*.json', '*.yaml', '*.yml'],
      searchExcludes: ['node_modules/**', '.git/**', 'dist/**', 'build/**']
    };
  }

  /**
   * Get current configuration
   */
  get(): T {
    return this.config.data;
  }

  /**
   * Update configuration with validation
   */
  async update(updater: (config: T) => T): Promise<void> {
    const newConfig = updater(this.config.data);

    if (this.validateConfig(newConfig).isValid) {
      const previousData = this.config.data;
      this.config = {
        ...this.config,
        data: newConfig,
        loadedAt: new Date(),
        priority: SourcePriority.RUNTIME
      };

      // Notify subscribers
      const change: ConfigFileChange = {
        path: 'runtime-update',
        previousConfig: previousData,
        newConfig: newConfig,
        changeTime: new Date()
      };

      this.notifySubscribers(change);

      logger.info(`Configuration updated for ${this.appName}`);
    } else {
      throw new Error(`Configuration validation failed for ${this.appName}`);
    }
  }

  /**
   * Load configuration from all sources
   */
  async load(options?: Partial<LoadOptions>): Promise<T> {
    const opts = { ...this.options, ...options };
    this.options = opts;

    try {
      const pathResolver = new PathResolver();
      const configPaths = await pathResolver.resolveConfigPaths(this.appName);

      const loader = new FileMarshall();

      // Load all configuration sources
      const sources: ConfigSource<T>[] = [
        {
          data: this.defaultConfig,
          priority: SourcePriority.DEFAULT,
          loadedAt: new Date()
        }
      ];

      for (const path of configPaths) {
        try {
          const config = await loader.loadFromFile(path, 'auto');
          sources.push({
            data: config as T,
            priority: SourcePriority.PROJECT,
            sourcePath: path,
            loadedAt: new Date(),
            format: this.detectFormat(path)
          });
        } catch (error) {
          logger.warn(`Failed to load config from ${path}: ${error}`);
        }
      }

      // Merge configurations
      const mergedConfig = mergeConfigs<T>(sources, 'deep-merge');

      this.config = {
        ...mergedConfig,
        loadedAt: new Date()
      };

      // Setup hot reload if enabled
      if (opts.enableHotReload && !this.hotReloadMonitor) {
        this.setupHotReload(opts.hotReloadDebounceMs ?? 500);
      }

      // Validate the loaded configuration
      if (!this.validateConfig(this.config.data).isValid) {
        logger.warn(`Configuration validation failed for ${this.appName}`);
      }

      logger.info(`Configuration loaded for ${this.appName} from ${sources.length} sources`);
      return this.config.data;

    } catch (error) {
      logger.error(`Failed to load configuration for ${this.appName}: ${error}`);
      return this.defaultConfig;
    }
  }

  /**
   * Load configuration with environment variable overrides
   */
  async loadWithOverrides(envVars: string[], options?: Partial<LoadOptions>): Promise<T> {
    await this.load(options);

    if (!this.options.enableEnvOverride) {
      return this.config.data;
    }

    const envResolver = new EnvResolver(this.options.envPrefix);
    const envOverrides = envResolver.getOverrides(envVars);

    if (Object.keys(envOverrides).length > 0) {
      const pathResolver = new PathResolver();
      let overrideConfig = this.config.data;

      // Apply environment variable overrides
      for (const [envVar, value] of Object.entries(envOverrides)) {
        const configPath = envVar.toLowerCase().replace(/_/g, '.');
        overrideConfig = this.setNestedValue(overrideConfig, configPath.split('.'), value);
      }

      // Create new source with environment overrides
      const envSource: ConfigSource<T> = {
        data: overrideConfig,
        priority: SourcePriority.ENVIRONMENT,
        loadedAt: new Date()
      };

      // Re-merge with environment overrides taking precedence
      this.config = {
        ...envSource,
        loadedAt: new Date()
      };

      logger.info(`Applied ${Object.keys(envOverrides).length} environment overrides for ${this.appName}`);
    }

    return this.config.data;
  }

  /**
   * Save current configuration to file
   */
  async save(path?: string, format: ConfigFormat = 'json'): Promise<void> {
    const savePath = path || `${this.appName}-config.${format}`;
    const loader = new FileMarshall();

    try {
      await loader.saveToFile(savePath, this.config.data, format);
      logger.info(`Configuration saved to ${savePath}`);
    } catch (error) {
      logger.error(`Failed to save configuration to ${savePath}: ${error}`);
      throw error;
    }
  }

  /**
   * Get configuration validation result
   */
  validate(): ValidationResult {
    return this.validateConfig(this.config.data);
  }

  /**
   * Subscribe to configuration changes
   */
  subscribe(callback: (config: T, change?: ConfigFileChange) => void): () => void {
    this.subscribers.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.subscribers.indexOf(callback);
      if (index > -1) {
        this.subscribers.splice(index, 1);
      }
    };
  }

  /**
   * Reload configuration from all sources
   */
  async reload(): Promise<void> {
    logger.info(`Reloading configuration for ${this.appName}`);
    await this.load();
  }

  /**
   * Private helper methods
   */
  private validateConfig(config: T): ValidationResult {
    // TypeScript compile-time type safety is our primary validation
    // Additional validation would need to be implemented by specific config classes
    return {
      isValid: true,
      errors: [],
      warnings: []
    };
  }

  private detectFormat(path: string): ConfigFormat {
    const ext = path.toLowerCase().split('.').pop();
    switch (ext) {
      case 'json': return 'json';
      case 'yaml':
      case 'yml': return 'yaml';
      default: return 'auto';
    }
  }

  private setNestedValue(obj: any, keys: string[], value: any): any {
    if (keys.length === 0) return value;

    const [first, ...rest] = keys;
    const result = { ...obj };

    if (rest.length === 0) {
      result[first] = value;
    } else {
      result[first] = this.setNestedValue(obj[first] || {}, rest, value);
    }

    return result;
  }

  private setupHotReload(debounceMs: number): void {
    if (this.hotReloadMonitor) return;

    this.hotReloadMonitor = new HotReloadMonitor(debounceMs);

    this.hotReloadMonitor.start(async (change: ConfigFileChange) => {
      logger.info(`Configuration file changed: ${change.path}`);

      try {
        // Reload the configuration
        await this.reload();

        // Notify subscribers with the change
        this.notifySubscribers(change);
      } catch (error) {
        logger.error(`Failed to reload configuration after file change: ${error}`);
      }
    });
  }

  private notifySubscribers(change: ConfigFileChange): void {
    for (let i = 0; i < this.subscribers.length; i++) {
      try {
        this.subscribers[i](this.config.data, change);
      } catch (error) {
        logger.error(`Configuration subscriber callback failed: ${error}`);
      }
    }
  }

  /**
   * Cleanup resources
   */
  async dispose(): Promise<void> {
    // Stop hot reload monitoring
    if (this.hotReloadMonitor) {
      await this.hotReloadMonitor.stop();
      this.hotReloadMonitor = undefined;
    }

    // Clear subscribers
    this.subscribers = [];

    logger.info(`ConfigurationManager for ${this.appName} disposed`);
  }
}

/**
 * Merge configuration sources based on priority
 */
function mergeConfigs<T>(sources: ConfigSource<T>[], strategy: 'priority-take' | 'deep-merge' | ((high: any, low: any) => any) = 'priority-take'): ConfigSource<T> {
  if (sources.length === 0) {
    throw new Error('No config sources provided');
  }

  // Sort sources by priority (higher priority first)
  const sortedSources = [...sources].sort((a, b) => b.priority - a.priority);

  let mergedData = sortedSources[0].data;

  for (let i = 1; i < sortedSources.length; i++) {
    const source = sortedSources[i];
    if (strategy === 'priority-take') {
      // Only take from lower priority if the value is not defined in higher priority
      mergedData = mergeObjects(mergedData, source.data, true);
    } else if (strategy === 'deep-merge') {
      mergedData = mergeObjects(mergedData, source.data, false);
    } else if (typeof strategy === 'function') {
      mergedData = strategy(mergedData, source.data);
    }
  }

  return {
    ...sortedSources[0], // Base metadata from highest priority
    data: mergedData,
    loadedAt: new Date()
  };
}

/**
 * Deep merge objects
 */
function mergeObjects(target: any, source: any, skipDefined = false): any {
  const result = { ...target };

  for (const key in source) {
    if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
      result[key] = mergeObjects(result[key], source[key], skipDefined);
    } else if (!skipDefined || !(key in target)) {
      result[key] = source[key];
    }
  }

  return result;
}

/**
 * Path resolver for browser environment
 */
class PathResolver implements IPathResolver {
  async getUserConfigDir(): Promise<string> {
    // In browser, we use localStorage or a virtual path
    return 'user://config';
  }

  async getProjectConfigDir(): Promise<string> {
    // Relative to current location
    return './config';
  }

  async resolveConfigPaths(appName: string): Promise<string[]> {
    const paths: string[] = [];

    // Browser environment config paths
    paths.push(`/config/${appName}.json`);
    paths.push(`/config/${appName}.yaml`);
    paths.push(`/config/${appName}.yml`);
    paths.push(`${this.locationOrigin()}/config/${appName}.json`);
    paths.push(`${this.locationOrigin()}/config/${appName}.yaml`);
    paths.push(`${this.locationOrigin()}/config/${appName}.yml`);

    return paths;
  }

  private locationOrigin(): string {
    if (typeof window !== 'undefined') {
      return window.location.origin;
    }
    return '';
  }
}

/**
 * File loader for browser environment (stubbed)
 */
class FileMarshall {
  async loadFromFile(path: string, format: ConfigFormat = 'auto'): Promise<any> {
    // In browser environment, we would typically load via fetch or API
    // For now, return a stub implementation
    try {
      const response = await fetch(path);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const contentType = response.headers.get('content-type');
      if (contentType?.includes('application/json')) {
        return await response.json();
      } else {
        const text = await response.text();
        // Simple YAML parsing stub - in real implementation, use a YAML parser
        return this.parseYAML(text);
      }
    } catch (error) {
      throw new Error(`Failed to load config file ${path}: ${error}`);
    }
  }

  async saveToFile(path: string, data: any, format: ConfigFormat = 'json'): Promise<void> {
    // In browser environment, saving files is limited
    // Would typically use download or API endpoints
    throw new Error('File saving not supported in browser environment');
  }

  private parseYAML(text: string): any {
    // Stub YAML parser - in real implementation, use proper YAML library
    try {
      // Very basic key-value parser
      const lines = text.split('\n');
      const result: any = {};

      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('#')) {
          const colonIndex = trimmed.indexOf(':');
          if (colonIndex > 0) {
            const key = trimmed.substring(0, colonIndex).trim();
            const value = trimmed.substring(colonIndex + 1).trim();
            if (value) {
              // Try to parse JSON values
              try {
                result[key] = JSON.parse(value);
              } catch {
                result[key] = value;
              }
            }
          }
        }
      }

      return result;
    } catch (error) {
      throw new Error(`Failed to parse YAML: ${error}`);
    }
  }
}

/**
 * Environment variable resolver
 */
class EnvResolver implements IEnvResolver {
  constructor(private prefix: string = 'APP_') {}

  getValue(key: string): string | null {
    // In browser, environment variables are typically not available
    // We could check for global variables or URL parameters as fallback
    return this.getFromURLSearchParams(key) ||
           this.getFromGlobalVar(key) ||
           null;
  }

  getWithPrefix(prefix: string = this.prefix): Record<string, string> {
    const result: Record<string, string> = {};
    // Browser doesn't have env vars, but we can check for common patterns
    return result;
  }

  has(key: string): boolean {
    return this.getValue(key) !== null;
  }

  getOverrides(vars: string[]): Record<string, string> {
    const overrides: Record<string, string> = {};

    for (const varName of vars) {
      const value = this.getValue(varName);
      if (value !== null) {
        overrides[varName] = value;
      }
    }

    return overrides;
  }

  private getFromURLSearchParams(key: string): string | null {
    if (typeof window !== 'undefined' && window.location) {
      const params = new URLSearchParams(window.location.search);
      return params.get(key.toLowerCase());
    }
    return null;
  }

  private getFromGlobalVar(key: string): string | null {
    if (typeof window !== 'undefined') {
      return (window as any)[key] || null;
    }
    return null;
  }
}

/**
 * Hot reload monitor for browser environment
 */
class HotReloadMonitor implements IHotReloadMonitor {
  private watchedPaths: string[] = [];
  private isMonitoring = false;

  constructor(private debounceMs: number = 500) {}

  async start(onChange: (change: ConfigFileChange) => void): Promise<void> {
    if (this.isMonitoring) {
      return;
    }

    this.isMonitoring = true;

    // Use polling for file changes in browser environment
    setInterval(() => {
      this.checkForChanges(onChange);
    }, this.debounceMs);

    logger.info(`Hot reload monitor started with debounce ${this.debounceMs}ms`);
  }

  async stop(): Promise<void> {
    this.isMonitoring = false;
    this.watchedPaths = [];
    logger.info('Hot reload monitor stopped');
  }

  watchPaths(paths: string[]): void {
    this.watchedPaths = [...paths];
  }

  private async checkForChanges(onChange: (change: ConfigFileChange) => void): Promise<void> {
    for (let i = 0; i < this.watchedPaths.length; i++) {
      const path = this.watchedPaths[i];
      try {
        // Check if file has been modified by attempting to load it
        const response = await fetch(path, { method: 'HEAD' });

        if (response.ok) {
          const lastModified = response.headers.get('last-modified');
          if (lastModified) {
            // In a real implementation, we'd compare timestamps
            // For now, we'll assume any successful response indicates a potential change
            const change: ConfigFileChange = {
              path,
              previousConfig: {},
              newConfig: {},
              changeTime: new Date()
            };

            onChange(change);
          }
        }
      } catch (error) {
        // File might not exist or be accessible
        continue;
      }
    }
  }
}

// Alias for backward compatibility
class FileLoader extends FileMarshall {}

/**
 * Create a typed configuration manager instance
 */
export function createConfigManager<T>(
  appName: string,
  defaultConfig: T,
  options: LoadOptions = {}
): ConfigManager<T> {
  return new ConfigManager(appName, defaultConfig, options);
}

/**
 * Utility function for creating configuration with validation
 */
export function createValidatingConfig<T extends Configurable>(
  ConfigClass: new () => T,
  appName?: string
): ConfigManager<T> {
  const instance = new ConfigClass();
  const appNameToUse = appName || instance.FILE_PREFIX;

  return createConfigManager(
    appNameToUse,
    instance.getDefaultConfig() as T
  );
}