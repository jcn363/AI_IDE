/**
 * TypeScript Cache Types
 * Mirror of the Rust cache types for consistency
 */

// Cache configuration options
export interface CacheConfig {
  maxEntries: number;
  defaultTtl?: number; // seconds
  evictionPolicy: EvictionPolicy;
  enableMetrics: boolean;
  maxMemoryMb?: number;
  compressionThresholdKb?: number;
  backgroundCleanupIntervalSeconds: number;
}

// Supported eviction policies
export enum EvictionPolicy {
  Lru = 'lru',
  Lfu = 'lfu',
  Fifo = 'fifo',
  Random = 'random',
  SizeBased = 'size_based',
}

// Cache entry with metadata
export interface CacheEntry<T> {
  value: T;
  createdAt: Date;
  lastAccessed: Date;
  expiresAt?: Date;
  accessCount: number;
  ttlSeconds?: number;
  metadata: Record<string, string>;
}

// Cache performance statistics
export interface CacheStats {
  totalEntries: number;
  totalHits: number;
  totalMisses: number;
  totalEvictions: number;
  totalSets: number;
  hitRatio: number;
  memoryUsageBytes?: number;
  uptimeSeconds: number;
  createdAt: Date;
}

// Diagnostic cache key
export interface DiagnosticCacheKey {
  workspacePath: string;
  requestHash: number;
}

// Request types for type safety
export interface DiagnosticRequest {
  workspacePath: string;
  includeExplanations: boolean;
  includeSuggestedFixes: boolean;
  useCache: boolean;
  cacheTtlSeconds?: number;
  timeoutSeconds?: number;
}

export interface ExplanationRequest {
  errorCode: string;
  useCache?: boolean;
  cacheTtlSeconds?: number;
}

// Result types
export interface CompilerDiagnosticsResult {
  diagnostics: Diagnostic[];
  explanations: Record<string, string>;
  suggestedFixes: SuggestedFix[];
  metadata: DiagnosticMetadata;
}

export interface DiagnosticMetadata {
  workspacePath: string;
  timestamp: string;
  compilationTimeMs: number;
  totalErrors: number;
  totalWarnings: number;
  totalNotes: number;
  cached: boolean;
}

export interface Diagnostic {
  severity: 'error' | 'warning' | 'note' | 'help';
  message: string;
  file: string;
  line: number;
  column: number;
  code?: string;
  suggestions?: SuggestedFix[];
}

export interface SuggestedFix {
  title: string;
  edit: {
    file: string;
    start: { line: number; column: number };
    end: { line: number; column: number };
    new_text: string;
  };
}

export interface ErrorCodeExplanation {
  error_code: string;
  title: string;
  description: string;
  examples: string[];
  solutions: SuggestedFix[];
  related_errors: string[];
  severity: 'error' | 'warning' | 'note' | 'help';
  category: 'syntax' | 'type' | 'pattern' | 'other';
  rustc_code: string;
}

// Generic cache interface
export interface ICache<K = string, V = any> {
  get(key: K): Promise<V | undefined>;
  set(key: K, value: V, ttl?: number): Promise<void>;
  delete(key: K): Promise<boolean>;
  clear(): Promise<void>;
  size(): number;
  contains(key: K): boolean;
  stats(): Promise<CacheStats>;
  cleanup(): Promise<number>;
}

// Cache manager for coordinating multiple caches
export interface CacheManager {
  getDiagnostic(request: DiagnosticRequest): Promise<CompilerDiagnosticsResult | undefined>;
  setDiagnostic(request: DiagnosticRequest, result: CompilerDiagnosticsResult): Promise<void>;
  getExplanation(errorCode: string): Promise<ErrorCodeExplanation | undefined>;
  setExplanation(errorCode: string, explanation: ErrorCodeExplanation, ttl?: number): Promise<void>;
  getPerformance(key: string): Promise<any | undefined>;
  setPerformance(key: string, data: any, ttl?: number): Promise<void>;
  clearAll(): Promise<void>;
  globalStats(): Promise<GlobalCacheStats>;
}

// Combined global statistics
export interface GlobalCacheStats {
  diagnostic: CacheStats;
  explanation: CacheStats;
  performance: CacheStats;
  totalEvictions: number;
  totalHits: number;
  totalMisses: number;
}

// Cache presets for different use cases
export const CACHE_PRESETS = {
  diagnostic: (): CacheConfig => ({
    maxEntries: 1000,
    defaultTtl: 300, // 5 minutes
    evictionPolicy: EvictionPolicy.Lru,
    enableMetrics: true,
    maxMemoryMb: 50,
    compressionThresholdKb: 10,
    backgroundCleanupIntervalSeconds: 300,
  }),

  explanation: (): CacheConfig => ({
    maxEntries: 2000,
    defaultTtl: 86400, // 24 hours
    evictionPolicy: EvictionPolicy.Lfu,
    enableMetrics: true,
    maxMemoryMb: 30,
    compressionThresholdKb: 5,
    backgroundCleanupIntervalSeconds: 600,
  }),

  performance: (): CacheConfig => ({
    maxEntries: 5000,
    defaultTtl: 60, // 1 minute
    evictionPolicy: EvictionPolicy.Fifo,
    enableMetrics: true,
    maxMemoryMb: 20,
    compressionThresholdKb: undefined,
    backgroundCleanupIntervalSeconds: 60,
  }),
} as const;