/**
 * Cache Module Export
 *
 * Provides a unified caching interface for the Rust AI IDE frontend.
 * Mirrors the Rust cache architecture for consistency.
 */

// Export types
export * from './types';

// Export core cache implementation
export { InMemoryCache } from './cache';

// Export unified manager
export { UnifiedCacheManager, cacheManager, LegacyCacheManager } from './manager';

// Export presets for easy configuration
export { CACHE_PRESETS } from './types';

// Re-export commonly used items
export type {
  CompilerDiagnosticsResult,
  ErrorCodeExplanation,
  DiagnosticRequest,
  CacheStats,
  GlobalCacheStats,
} from './types';