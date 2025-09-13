/**
 * Unified Cache Manager for TypeScript
 * Provides a high-level interface for managing multiple caches
 */

import { InMemoryCache } from './cache';
import { CACHE_PRESETS } from './types';
import {
  CacheManager as ICacheManager,
  GlobalCacheStats,
  CompilerDiagnosticsResult,
  ErrorCodeExplanation,
  DiagnosticRequest,
  ExplanationRequest,
} from './types';

export class UnifiedCacheManager implements ICacheManager {
  private diagnosticCache: InMemoryCache;
  private explanationCache: InMemoryCache;
  private performanceCache: InMemoryCache;

  constructor() {
    this.diagnosticCache = new InMemoryCache(CACHE_PRESETS.diagnostic());
    this.explanationCache = new InMemoryCache(CACHE_PRESETS.explanation());
    this.performanceCache = new InMemoryCache(CACHE_PRESETS.performance());
  }

  async getDiagnostic(request: DiagnosticRequest): Promise<CompilerDiagnosticsResult | undefined> {
    const key = this.generateDiagnosticKey(request);
    return this.diagnosticCache.get(key);
  }

  async setDiagnostic(
    request: DiagnosticRequest,
    result: CompilerDiagnosticsResult
  ): Promise<void> {
    const key = this.generateDiagnosticKey(request);
    const ttl = request.cacheTtlSeconds;
    await this.diagnosticCache.set(key, result, ttl);
  }

  async getExplanation(errorCode: string): Promise<ErrorCodeExplanation | undefined> {
    const key = `explanation:${errorCode}`;
    return this.explanationCache.get(key);
  }

  async setExplanation(
    errorCode: string,
    explanation: ErrorCodeExplanation,
    ttl?: number
  ): Promise<void> {
    const key = `explanation:${errorCode}`;
    await this.explanationCache.set(key, explanation, ttl);
  }

  async getPerformance(key: string): Promise<any | undefined> {
    return this.performanceCache.get(key);
  }

  async setPerformance(key: string, data: any, ttl?: number): Promise<void> {
    await this.performanceCache.set(key, data, ttl);
  }

  async clearAll(): Promise<void> {
    await Promise.all([
      this.diagnosticCache.clear(),
      this.explanationCache.clear(),
      this.performanceCache.clear(),
    ]);
  }

  async globalStats(): Promise<GlobalCacheStats> {
    const [diagStats, expStats, perfStats] = await Promise.all([
      this.diagnosticCache.stats(),
      this.explanationCache.stats(),
      this.performanceCache.stats(),
    ]);

    const now = Date.now();
    const createdAt = Math.min(
      diagStats.createdAt.getTime(),
      expStats.createdAt.getTime(),
      perfStats.createdAt.getTime()
    );

    return {
      diagnostic: diagStats,
      explanation: expStats,
      performance: perfStats,
      totalEvictions: diagStats.totalEvictions + expStats.totalEvictions + perfStats.totalEvictions,
      totalHits: diagStats.totalHits + expStats.totalHits + perfStats.totalHits,
      totalMisses: diagStats.totalMisses + expStats.totalMisses + perfStats.totalMisses,
    };
  }

  async cleanupExpired(): Promise<number> {
    const [diagCleaned, expCleaned, perfCleaned] = await Promise.all([
      this.diagnosticCache.cleanup(),
      this.explanationCache.cleanup(),
      this.performanceCache.cleanup(),
    ]);

    return diagCleaned + expCleaned + perfCleaned;
  }

  private generateDiagnosticKey(request: DiagnosticRequest): string {
    const components = [
      request.workspacePath,
      request.includeExplanations.toString(),
      request.includeSuggestedFixes.toString(),
    ];

    if (request.cacheTtlSeconds) {
      components.push(request.cacheTtlSeconds.toString());
    }

    return `diagnostic:${this.simpleHash(components.join(';'))}`;
  }

  private simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    return hash.toString(16);
  }

  // Utility methods for cache management
  async getCacheSizes() {
    const [diagSize, expSize, perfSize] = await Promise.all([
      this.diagnosticCache.size(),
      this.explanationCache.size(),
      this.performanceCache.size(),
    ]);

    return {
      diagnostic: diagSize,
      explanation: expSize,
      performance: perfSize,
    };
  }

  // Destroy method to clean up resources
  destroy(): void {
    this.diagnosticCache.destroy?.();
    this.explanationCache.destroy?.();
    this.performanceCache.destroy?.();
  }
}

// Export singleton instance for global use
export const cacheManager = new UnifiedCacheManager();

// Cleanup on page unload (browser environment)
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    cacheManager.destroy();
  });
}

// Legacy compatibility alias
export const LegacyCacheManager = UnifiedCacheManager;
