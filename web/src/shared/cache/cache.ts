/**
 * Corrected Browser-compatible in-memory cache implementation
 * Mirrors the Rust cache implementation for consistency
 * Fixed to avoid naming conflicts
 */

import { CacheConfig, CacheEntry, CacheStats, EvictionPolicy, ICache } from './types';

export class InMemoryCache<K = string, V = any> implements ICache<K, V> {
  private data: Map<string, CacheEntry<V>> = new Map();
  private config: CacheConfig;
  private statsData: CacheStats;
  private cleanupInterval?: NodeJS.Timeout;

  constructor(config: CacheConfig) {
    this.config = config;
    this.statsData = {
      totalEntries: 0,
      totalHits: 0,
      totalMisses: 0,
      totalEvictions: 0,
      totalSets: 0,
      hitRatio: 0.0,
      uptimeSeconds: 0,
      createdAt: new Date(),
    };

    // Start background cleanup if configured
    if (config.backgroundCleanupIntervalSeconds > 0) {
      this.startCleanupTask();
    }
  }

  private keyToString(key: K): string {
    return typeof key === 'string' ? key : JSON.stringify(key);
  }

  private createEntry(value: V, ttlSeconds?: number): CacheEntry<V> {
    const now = new Date();
    const expiresAt = ttlSeconds ? new Date(now.getTime() + ttlSeconds * 1000) : undefined;

    return {
      value,
      createdAt: now,
      lastAccessed: now,
      expiresAt,
      accessCount: 0,
      ttlSeconds,
      metadata: {},
    };
  }

  private isExpired(entry: CacheEntry<V>): boolean {
    if (!entry.expiresAt) return false;
    return new Date() > entry.expiresAt;
  }

  private accessEntry(entry: CacheEntry<V>): void {
    entry.accessCount++;
    entry.lastAccessed = new Date();
  }

  private evict(): void {
    if (this.data.size <= this.config.maxEntries) return;

    const toEvict = this.data.size - this.config.maxEntries;

    switch (this.config.evictionPolicy) {
      case EvictionPolicy.Lru:
        this.evictLru(toEvict);
        break;
      case EvictionPolicy.Lfu:
        this.evictLfu(toEvict);
        break;
      case EvictionPolicy.Fifo:
        this.evictFifo(toEvict);
        break;
      case EvictionPolicy.Random:
        this.evictRandom(toEvict);
        break;
      default:
        this.evictLru(toEvict);
    }
  }

  private evictLru(count: number): void {
    const entries = Array.from(this.data.entries());
    entries.sort(([, a], [, b]) => a.lastAccessed.getTime() - b.lastAccessed.getTime());
    this.evictEntries(entries.slice(0, count).map(([key]) => key));
  }

  private evictLfu(count: number): void {
    const entries = Array.from(this.data.entries());
    entries.sort(([, a], [, b]) => a.accessCount - b.accessCount);
    this.evictEntries(entries.slice(0, count).map(([key]) => key));
  }

  private evictFifo(count: number): void {
    const entries = Array.from(this.data.entries());
    entries.sort(([, a], [, b]) => a.createdAt.getTime() - b.createdAt.getTime());
    this.evictEntries(entries.slice(0, count).map(([key]) => key));
  }

  private evictRandom(count: number): void {
    const keys = Array.from(this.data.keys());
    const selectedKeys = [];
    for (let i = 0; i < Math.min(count, keys.length); i++) {
      const randomIndex = Math.floor(Math.random() * keys.length);
      selectedKeys.push(keys.splice(randomIndex, 1)[0]);
    }
    this.evictEntries(selectedKeys);
  }

  private evictEntries(keys: string[]): void {
    keys.forEach((key) => {
      this.data.delete(key);
      this.statsData.totalEvictions++;
    });
  }

  private cleanupExpired(): number {
    let count = 0;
    for (const [key, entry] of this.data.entries()) {
      if (this.isExpired(entry)) {
        this.data.delete(key);
        count++;
      }
    }
    return count;
  }

  private startCleanupTask(): void {
    const cleanup = () => {
      try {
        this.cleanupExpired();
      } catch (error) {
        console.warn('Cache cleanup failed:', error);
      }
    };

    this.cleanupInterval = setInterval(
      cleanup,
      this.config.backgroundCleanupIntervalSeconds * 1000
    );
  }

  private updateHitRatio(): void {
    const total = this.statsData.totalHits + this.statsData.totalMisses;
    this.statsData.hitRatio = total > 0 ? this.statsData.totalHits / total : 0;
  }

  async get(key: K): Promise<V | undefined> {
    const keyStr = this.keyToString(key);

    await this.cleanupExpired();

    const entry = this.data.get(keyStr);
    if (!entry) {
      this.statsData.totalMisses++;
      this.updateHitRatio();
      return undefined;
    }

    if (this.isExpired(entry)) {
      this.data.delete(keyStr);
      this.statsData.totalMisses++;
      this.updateHitRatio();
      return undefined;
    }

    this.accessEntry(entry);
    this.statsData.totalHits++;
    this.updateHitRatio();
    return entry.value;
  }

  async set(key: K, value: V, ttlSeconds?: number): Promise<void> {
    const keyStr = this.keyToString(key);
    const ttl = ttlSeconds ?? this.config.defaultTtl;
    const entry = this.createEntry(value, ttl);

    this.evict();

    this.data.set(keyStr, entry);
    this.statsData.totalSets++;
    this.statsData.totalEntries = this.data.size;
  }

  async delete(key: K): Promise<boolean> {
    const keyStr = this.keyToString(key);
    const exists = this.data.delete(keyStr);
    if (exists) {
      this.statsData.totalEntries = this.data.size;
    }
    return exists;
  }

  async clear(): Promise<void> {
    this.data.clear();
    this.statsData.totalEntries = 0;
  }

  size(): number {
    return this.data.size;
  }

  contains(key: K): boolean {
    const keyStr = this.keyToString(key);
    const entry = this.data.get(keyStr);
    return entry !== undefined && !this.isExpired(entry);
  }

  async stats(): Promise<CacheStats> {
    this.statsData.uptimeSeconds =
      (new Date().getTime() - this.statsData.createdAt.getTime()) / 1000;
    this.statsData.totalEntries = this.data.size;
    return { ...this.statsData };
  }

  async cleanup(): Promise<number> {
    return this.cleanupExpired();
  }

  destroy(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = undefined;
    }
    this.clear();
  }
}
