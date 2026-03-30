// Cache Module
// wraps: phenotype-cache-adapter (Rust crate) — Two-tier (L1/L2) caching with invalidation

import type { CachePort } from '../../../pheno-core/src/ports/index';

// ============================================================================
// CORE INTERFACES
// ============================================================================

/**
 * Cache configuration
 */
export interface CacheConfig {
  l1MaxSize?: number; // Max entries in L1 cache
  l1TtlMs?: number; // L1 TTL in milliseconds
  l2MaxSize?: number; // Max entries in L2 cache (if enabled)
  l2TtlMs?: number; // L2 TTL in milliseconds
  enableL2?: boolean; // Enable two-tier caching (default: false)
}

/**
 * Two-tier cache implementation (L1 in-memory, L2 optional external)
 */
export interface TwoTierCache<T> extends CachePort<T> {
  /**
   * Get with tier info
   */
  getWithTier(key: string): Promise<{ value: T | null; tier?: 'l1' | 'l2' }>;

  /**
   * Invalidate across all tiers
   */
  invalidate(pattern: string): Promise<void>;

  /**
   * Get cache statistics
   */
  getStats(): CacheStats;

  /**
   * Warm cache (bulk load)
   */
  warmCache(entries: Record<string, T>): Promise<void>;
}

/**
 * Cache statistics
 */
export interface CacheStats {
  l1Hits: number;
  l1Misses: number;
  l2Hits?: number;
  l2Misses?: number;
  evictions: number;
  size: number;
}

/**
 * Cache invalidation strategy
 */
export type InvalidationStrategy = 'ttl' | 'lru' | 'lfu' | 'custom';

/**
 * Cache entry metadata
 */
export interface CacheEntry<T> {
  value: T;
  ttl?: number;
  createdAt: Date;
  accessedAt: Date;
  accessCount: number;
}

// ============================================================================
// IN-MEMORY IMPLEMENTATION
// ============================================================================

/**
 * In-memory L1 cache implementation
 */
export class InMemoryCache<T> implements CachePort<T> {
  private cache: Map<string, CacheEntry<T>> = new Map();
  private stats: CacheStats = {
    l1Hits: 0,
    l1Misses: 0,
    evictions: 0,
    size: 0,
  };

  constructor(private config: CacheConfig = {}) {
    this.config = {
      l1MaxSize: config.l1MaxSize ?? 1000,
      l1TtlMs: config.l1TtlMs ?? 60000, // 1 minute default
    };

    // Cleanup expired entries periodically
    if (config.l1TtlMs) {
      setInterval(() => this.cleanup(), config.l1TtlMs!);
    }
  }

  async get(key: string): Promise<T | null> {
    const entry = this.cache.get(key);

    if (!entry) {
      this.stats.l1Misses++;
      return null;
    }

    // Check if expired
    if (entry.ttl && Date.now() - entry.createdAt.getTime() > entry.ttl) {
      this.cache.delete(key);
      this.stats.l1Misses++;
      return null;
    }

    // Update access stats
    entry.accessedAt = new Date();
    entry.accessCount++;
    this.stats.l1Hits++;

    return entry.value;
  }

  async set(key: string, value: T, ttlMs?: number): Promise<void> {
    // Evict if at capacity
    if (
      this.cache.size >= (this.config.l1MaxSize ?? 1000) &&
      !this.cache.has(key)
    ) {
      this.evictLRU();
    }

    this.cache.set(key, {
      value,
      ttl: ttlMs ?? this.config.l1TtlMs,
      createdAt: new Date(),
      accessedAt: new Date(),
      accessCount: 0,
    });

    this.stats.size = this.cache.size;
  }

  async delete(key: string): Promise<void> {
    this.cache.delete(key);
    this.stats.size = this.cache.size;
  }

  async clear(): Promise<void> {
    this.cache.clear();
    this.stats = {
      l1Hits: 0,
      l1Misses: 0,
      evictions: 0,
      size: 0,
    };
  }

  private evictLRU(): void {
    let lruKey: string | null = null;
    let lruTime = Date.now();

    for (const [key, entry] of this.cache.entries()) {
      if (entry.accessedAt.getTime() < lruTime) {
        lruKey = key;
        lruTime = entry.accessedAt.getTime();
      }
    }

    if (lruKey) {
      this.cache.delete(lruKey);
      this.stats.evictions++;
    }
  }

  private cleanup(): void {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (entry.ttl && now - entry.createdAt.getTime() > entry.ttl) {
        this.cache.delete(key);
        this.stats.evictions++;
      }
    }
    this.stats.size = this.cache.size;
  }

  getStats(): CacheStats {
    return { ...this.stats };
  }
}

/**
 * Two-tier cache implementation
 */
export class TwoTierCacheImpl<T> implements TwoTierCache<T> {
  private l1: InMemoryCache<T>;
  private l2?: CachePort<T>;

  constructor(
    l1Config?: CacheConfig,
    l2?: CachePort<T>,
  ) {
    this.l1 = new InMemoryCache(l1Config);
    this.l2 = l2;
  }

  async get(key: string): Promise<T | null> {
    // Try L1 first
    let value = await this.l1.get(key);
    if (value) {
      return value;
    }

    // Try L2
    if (this.l2) {
      value = await this.l2.get(key);
      if (value) {
        // Promote to L1
        await this.l1.set(key, value);
      }
    }

    return value ?? null;
  }

  async getWithTier(
    key: string,
  ): Promise<{ value: T | null; tier?: 'l1' | 'l2' }> {
    const l1Value = await this.l1.get(key);
    if (l1Value) {
      return { value: l1Value, tier: 'l1' };
    }

    if (this.l2) {
      const l2Value = await this.l2.get(key);
      if (l2Value) {
        await this.l1.set(key, l2Value);
        return { value: l2Value, tier: 'l2' };
      }
    }

    return { value: null };
  }

  async set(key: string, value: T, ttlMs?: number): Promise<void> {
    await this.l1.set(key, value, ttlMs);
    if (this.l2) {
      await this.l2.set(key, value, ttlMs);
    }
  }

  async delete(key: string): Promise<void> {
    await this.l1.delete(key);
    if (this.l2) {
      await this.l2.delete(key);
    }
  }

  async clear(): Promise<void> {
    await this.l1.clear();
    if (this.l2) {
      await this.l2.clear();
    }
  }

  async invalidate(pattern: string): Promise<void> {
    // Simple pattern matching (prefix or exact)
    const isPrefix = pattern.endsWith('*');
    const prefix = isPrefix ? pattern.slice(0, -1) : pattern;

    // In real implementation, would have key registry
    // For now, would need custom implementation per L2 type
    await this.clear();
  }

  getStats(): CacheStats {
    return this.l1.getStats();
  }

  async warmCache(entries: Record<string, T>): Promise<void> {
    for (const [key, value] of Object.entries(entries)) {
      await this.set(key, value);
    }
  }
}
