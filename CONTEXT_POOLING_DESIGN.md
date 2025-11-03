# BookCore Context Pooling Design Document

**Date**: 2025-01-04
**Version**: 1.0
**Author**: @zsakvo

---

## 1. Executive Summary

### Problem Statement
The current BookCore implementation creates a new Boa JavaScript Context for every request, causing:
- Repeated initialization overhead (~50-100ms per Context creation)
- Redundant script compilation for frequently used book sources
- Poor performance for high-frequency requests

### Proposed Solution
Implement a **tiered LRU caching system** that balances:
- ✅ Fast response times for popular sources
- ✅ Reasonable memory footprint (~150-400MB)
- ✅ Natural isolation between book sources
- ✅ Graceful degradation for infrequent sources

### Key Metrics
| Metric | Current | After Pooling |
|--------|---------|---------------|
| Hot source response | 100-150ms | 5-10ms |
| Memory overhead | ~5MB peak | 150-400MB steady |
| Cold source response | 100-150ms | 100-150ms (unchanged) |
| Context reuse rate | 0% | 60-80% (estimated) |

---

## 2. Memory Analysis

### Boa Context Memory Footprint

**Estimated per Context** (based on Rust JS engine characteristics):
```
Base Context (empty):     2-5 MB
  ├─ Global object        ~500 KB
  ├─ Built-in prototypes  ~1 MB
  ├─ VM structures        ~500 KB
  └─ GC metadata          ~1 MB

Loaded Book Source:       +1-3 MB
  ├─ Compiled bytecode    ~500 KB
  ├─ Function closures    ~500 KB
  └─ Script state         ~1 MB

Total per source:         3-8 MB (average ~5 MB)
```

**Note**: Actual memory usage may vary based on:
- Book source script complexity
- Number of registered APIs
- Runtime data (cached HTTP responses, etc.)

### Memory Overhead Comparison

| Approach | Contexts | Memory Usage | Notes |
|----------|----------|--------------|-------|
| **Current (One-shot)** | 1 active | ~5 MB transient | Freed after each request |
| **Small LRU Pool** | 10 cached | 30-80 MB | Good for <10 active sources |
| **Medium LRU Pool** | 20 cached | 60-160 MB | Balanced approach |
| **Tiered Hybrid** | 50 (20+30) | 150-400 MB | **Recommended** |
| **Full Pool** | 100+ | 300-800+ MB | Impractical for large source counts |

---

## 3. Tiered Caching Architecture

### 3.1 Cache Tiers

```
┌─────────────────────────────────────────────────────┐
│  Tier 1: HOT CACHE (High Performance)               │
│  ├─ Size: 10-20 contexts                            │
│  ├─ TTL: 5 minutes since last use                   │
│  ├─ Strategy: LRU eviction                          │
│  └─ Use case: Frequently accessed book sources      │
├─────────────────────────────────────────────────────┤
│  Tier 2: WARM CACHE (Medium Performance)            │
│  ├─ Size: 20-30 contexts                            │
│  ├─ TTL: 30 seconds since last use                  │
│  ├─ Strategy: LRU eviction → drop                   │
│  ├─ Promotion: Move to Hot on second hit            │
│  └─ Use case: Recently used sources                 │
├─────────────────────────────────────────────────────┤
│  Tier 3: COLD START (On-Demand)                     │
│  ├─ No persistent Context                           │
│  ├─ Create → Execute → Drop immediately             │
│  └─ Use case: Infrequent/first-time sources         │
└─────────────────────────────────────────────────────┘
```

### 3.2 Lifecycle Flow

```rust
Request for book source "example_source"
    │
    ├─→ Check HOT cache
    │   ├─ Hit? → Execute immediately (5-10ms)
    │   └─ Miss? → Continue
    │
    ├─→ Check WARM cache
    │   ├─ Hit? → Execute + Promote to HOT
    │   └─ Miss? → Continue
    │
    └─→ COLD START
        ├─ Create new Context
        ├─ Load book source script
        ├─ Execute request
        ├─ Add to WARM cache
        └─ Return result

Background cleanup thread (every 10s):
    ├─ HOT: Demote to WARM if unused >5min
    ├─ WARM: Drop if unused >30s
    └─ Collect dropped Contexts' memory
```

---

## 4. Implementation Plan

### 4.1 Data Structures

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use lru::LruCache;
use boa_engine::Context;

/// Tier classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheTier {
    Hot,   // Frequently used (5min TTL)
    Warm,  // Recently used (30s TTL)
    Cold,  // Not cached (create on demand)
}

/// Cached Context with metadata
pub struct CachedContext {
    context: Context,
    source_id: String,
    last_used: Instant,
    hit_count: usize,
    tier: CacheTier,
}

/// Pooling configuration
#[derive(Clone)]
pub struct PoolConfig {
    pub hot_cache_size: usize,         // Default: 20
    pub warm_cache_size: usize,        // Default: 30
    pub hot_ttl: Duration,             // Default: 5min
    pub warm_ttl: Duration,            // Default: 30s
    pub cleanup_interval: Duration,    // Default: 10s
    pub promotion_threshold: usize,    // Hits to promote Warm→Hot (default: 2)
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            hot_cache_size: 20,
            warm_cache_size: 30,
            hot_ttl: Duration::from_secs(300),     // 5 minutes
            warm_ttl: Duration::from_secs(30),     // 30 seconds
            cleanup_interval: Duration::from_secs(10),
            promotion_threshold: 2,
        }
    }
}

/// Main pool manager
pub struct BookCorePool {
    hot_cache: Arc<Mutex<LruCache<String, CachedContext>>>,
    warm_cache: Arc<Mutex<LruCache<String, CachedContext>>>,
    config: PoolConfig,

    // Statistics
    stats: Arc<Mutex<PoolStats>>,
}

#[derive(Default)]
pub struct PoolStats {
    pub hot_hits: usize,
    pub warm_hits: usize,
    pub cold_starts: usize,
    pub promotions: usize,
    pub evictions: usize,
}
```

### 4.2 Core API

```rust
impl BookCorePool {
    /// Create new pool with default config
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create pool with custom config
    pub fn with_config(config: PoolConfig) -> Self {
        let hot_cache = Arc::new(Mutex::new(
            LruCache::new(config.hot_cache_size.try_into().unwrap())
        ));
        let warm_cache = Arc::new(Mutex::new(
            LruCache::new(config.warm_cache_size.try_into().unwrap())
        ));

        let pool = Self {
            hot_cache,
            warm_cache,
            config: config.clone(),
            stats: Arc::new(Mutex::new(PoolStats::default())),
        };

        // Start background cleanup thread
        pool.start_cleanup_thread();

        pool
    }

    /// Execute script for a book source (main API)
    pub fn execute(
        &self,
        source_id: &str,
        script: &str,
        function_name: &str,
        args: &[String],
    ) -> BookResult<String> {
        // 1. Try HOT cache
        if let Some(mut cached) = self.try_hot_cache(source_id) {
            self.stats.lock().unwrap().hot_hits += 1;
            return self.execute_on_context(&mut cached.context, function_name, args);
        }

        // 2. Try WARM cache
        if let Some(mut cached) = self.try_warm_cache(source_id) {
            self.stats.lock().unwrap().warm_hits += 1;
            cached.hit_count += 1;

            // Promote to HOT if threshold reached
            if cached.hit_count >= self.config.promotion_threshold {
                self.promote_to_hot(source_id, cached);
                self.stats.lock().unwrap().promotions += 1;
            }

            return self.execute_on_context(&mut cached.context, function_name, args);
        }

        // 3. COLD START
        self.stats.lock().unwrap().cold_starts += 1;
        let mut new_context = self.create_context_for_source(source_id, script)?;
        let result = self.execute_on_context(&mut new_context, function_name, args);

        // Add to WARM cache for future use
        self.add_to_warm_cache(source_id, new_context);

        result
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clear all caches (useful for testing)
    pub fn clear(&self) {
        self.hot_cache.lock().unwrap().clear();
        self.warm_cache.lock().unwrap().clear();
    }
}
```

### 4.3 Internal Helper Methods

```rust
impl BookCorePool {
    /// Try to get Context from HOT cache
    fn try_hot_cache(&self, source_id: &str) -> Option<CachedContext> {
        let mut hot = self.hot_cache.lock().unwrap();
        hot.get_mut(source_id).map(|cached| {
            cached.last_used = Instant::now();
            cached.hit_count += 1;
            // Return a moved value (remove from cache temporarily)
            // Will be re-inserted after execution
            cached.clone() // Note: Need to implement Clone for CachedContext
        })
    }

    /// Try to get Context from WARM cache
    fn try_warm_cache(&self, source_id: &str) -> Option<CachedContext> {
        let mut warm = self.warm_cache.lock().unwrap();
        warm.pop(source_id).map(|mut cached| {
            cached.last_used = Instant::now();
            cached
        })
    }

    /// Promote Context to HOT tier
    fn promote_to_hot(&self, source_id: &str, mut cached: CachedContext) {
        cached.tier = CacheTier::Hot;

        let mut hot = self.hot_cache.lock().unwrap();

        // If HOT cache is full, evict LRU to WARM
        if hot.len() >= self.config.hot_cache_size {
            if let Some((evicted_id, evicted_ctx)) = hot.pop_lru() {
                self.demote_to_warm(&evicted_id, evicted_ctx);
            }
        }

        hot.put(source_id.to_string(), cached);
    }

    /// Demote Context to WARM tier
    fn demote_to_warm(&self, source_id: &str, mut cached: CachedContext) {
        cached.tier = CacheTier::Warm;
        cached.last_used = Instant::now();

        let mut warm = self.warm_cache.lock().unwrap();
        warm.put(source_id.to_string(), cached);
    }

    /// Add new Context to WARM cache
    fn add_to_warm_cache(&self, source_id: &str, context: Context) {
        let cached = CachedContext {
            context,
            source_id: source_id.to_string(),
            last_used: Instant::now(),
            hit_count: 1,
            tier: CacheTier::Warm,
        };

        let mut warm = self.warm_cache.lock().unwrap();
        warm.put(source_id.to_string(), cached);
    }

    /// Create and initialize Context for a book source
    fn create_context_for_source(
        &self,
        source_id: &str,
        script: &str,
    ) -> BookResult<Context> {
        // Reuse existing BookCore initialization logic
        let mut context = Context::default();

        // Register global APIs (from BookCore::new_with_context)
        crate::global::console::register_console(&mut context);
        crate::global::encoding::register_encoding_apis(&mut context);
        // ... (all other API registrations)

        // Compile and load the book source script
        context.eval_script(boa_engine::Source::from_bytes(script))
            .map_err(|e| BookError::ScriptError(format!("Failed to load source {}: {}", source_id, e)))?;

        Ok(context)
    }

    /// Execute a function on a Context
    fn execute_on_context(
        &self,
        context: &mut Context,
        function_name: &str,
        args: &[String],
    ) -> BookResult<String> {
        // Reuse existing BookCore::eval_script logic
        // Convert args to JsValue, call function, serialize result
        todo!("Implement execution logic (reuse from BookCore)")
    }

    /// Background cleanup thread
    fn start_cleanup_thread(&self) {
        let hot_cache = Arc::clone(&self.hot_cache);
        let warm_cache = Arc::clone(&self.warm_cache);
        let config = self.config.clone();
        let stats = Arc::clone(&self.stats);

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(config.cleanup_interval);

                let now = Instant::now();
                let mut eviction_count = 0;

                // Check HOT cache: demote stale entries to WARM
                {
                    let mut hot = hot_cache.lock().unwrap();
                    let stale_keys: Vec<String> = hot
                        .iter()
                        .filter(|(_, cached)| now.duration_since(cached.last_used) > config.hot_ttl)
                        .map(|(k, _)| k.clone())
                        .collect();

                    for key in stale_keys {
                        if let Some(cached) = hot.pop(&key) {
                            // Demote to WARM instead of dropping
                            let mut warm = warm_cache.lock().unwrap();
                            let mut demoted = cached;
                            demoted.tier = CacheTier::Warm;
                            demoted.last_used = now;
                            warm.put(key, demoted);
                        }
                    }
                }

                // Check WARM cache: drop stale entries
                {
                    let mut warm = warm_cache.lock().unwrap();
                    let stale_keys: Vec<String> = warm
                        .iter()
                        .filter(|(_, cached)| now.duration_since(cached.last_used) > config.warm_ttl)
                        .map(|(k, _)| k.clone())
                        .collect();

                    for key in stale_keys {
                        warm.pop(&key);
                        eviction_count += 1;
                    }
                }

                stats.lock().unwrap().evictions += eviction_count;
            }
        });
    }
}
```

---

## 5. Migration Strategy

### Phase 1: Parallel Implementation (Week 1-2)
- ✅ Create `pool` module in `src/`
- ✅ Implement `BookCorePool` alongside existing `BookCore`
- ✅ Add `lru` crate dependency
- ✅ Write unit tests for pooling logic

### Phase 2: Integration Testing (Week 3)
- ✅ Create integration tests comparing pooled vs non-pooled performance
- ✅ Memory profiling with `heaptrack` or `valgrind`
- ✅ Benchmark with realistic book source workloads

### Phase 3: API Compatibility (Week 4)
- ✅ Add `BookCore::with_pool()` constructor
- ✅ Maintain backward compatibility with existing API
- ✅ Update documentation and examples

### Phase 4: Production Rollout (Week 5+)
- ✅ Feature flag: `features = ["context-pooling"]`
- ✅ Gradual rollout with monitoring
- ✅ Performance metrics collection

---

## 6. Configuration Recommendations

### Small-Scale Deployment (<50 active sources)
```rust
PoolConfig {
    hot_cache_size: 10,
    warm_cache_size: 20,
    hot_ttl: Duration::from_secs(300),    // 5 min
    warm_ttl: Duration::from_secs(30),    // 30 sec
    // Memory: ~90-240 MB
}
```

### Medium-Scale Deployment (50-200 sources)
```rust
PoolConfig {
    hot_cache_size: 20,
    warm_cache_size: 30,
    hot_ttl: Duration::from_secs(300),
    warm_ttl: Duration::from_secs(30),
    // Memory: ~150-400 MB (recommended)
}
```

### Large-Scale Deployment (200+ sources)
```rust
PoolConfig {
    hot_cache_size: 30,
    warm_cache_size: 50,
    hot_ttl: Duration::from_secs(180),    // 3 min (shorter)
    warm_ttl: Duration::from_secs(20),    // 20 sec (shorter)
    // Memory: ~240-640 MB
    // Note: Consider horizontal scaling instead
}
```

---

## 7. Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cold_start() {
        let pool = BookCorePool::new();
        // Execute on new source → should create Context
        let result = pool.execute("source1", "script", "func", &[]);
        assert_eq!(pool.stats().cold_starts, 1);
    }

    #[test]
    fn test_warm_cache_hit() {
        let pool = BookCorePool::new();
        pool.execute("source1", "script", "func", &[]); // Cold
        pool.execute("source1", "script", "func", &[]); // Warm hit
        assert_eq!(pool.stats().warm_hits, 1);
    }

    #[test]
    fn test_promotion_to_hot() {
        let pool = BookCorePool::with_config(PoolConfig {
            promotion_threshold: 2,
            ..Default::default()
        });

        pool.execute("source1", "script", "func", &[]); // Cold
        pool.execute("source1", "script", "func", &[]); // Warm
        pool.execute("source1", "script", "func", &[]); // Promoted to Hot

        assert_eq!(pool.stats().promotions, 1);
        assert_eq!(pool.stats().hot_hits, 1);
    }

    #[test]
    fn test_ttl_eviction() {
        let pool = BookCorePool::with_config(PoolConfig {
            warm_ttl: Duration::from_millis(100),
            cleanup_interval: Duration::from_millis(50),
            ..Default::default()
        });

        pool.execute("source1", "script", "func", &[]);
        std::thread::sleep(Duration::from_millis(200));

        // Should be evicted, causing cold start
        pool.execute("source1", "script", "func", &[]);
        assert_eq!(pool.stats().cold_starts, 2);
    }
}
```

### Performance Benchmarks
```rust
#[bench]
fn bench_cold_start(b: &mut Bencher) {
    let pool = BookCorePool::new();
    b.iter(|| {
        pool.clear(); // Force cold start
        pool.execute("source", SCRIPT, "search", &["keyword"])
    });
}

#[bench]
fn bench_hot_cache_hit(b: &mut Bencher) {
    let pool = BookCorePool::new();
    pool.execute("source", SCRIPT, "search", &["keyword"]); // Prime cache

    b.iter(|| {
        pool.execute("source", SCRIPT, "search", &["keyword"])
    });
}
```

---

## 8. Monitoring and Observability

### Metrics to Track
```rust
pub struct DetailedPoolStats {
    // Cache performance
    pub hot_hits: usize,
    pub warm_hits: usize,
    pub cold_starts: usize,
    pub hit_rate: f64,              // (hot + warm) / total

    // Cache churn
    pub promotions: usize,          // Warm → Hot
    pub demotions: usize,           // Hot → Warm
    pub evictions: usize,           // Warm → Dropped

    // Memory usage
    pub hot_cache_size: usize,      // Current contexts in HOT
    pub warm_cache_size: usize,     // Current contexts in WARM
    pub estimated_memory_mb: f64,   // Total estimated usage

    // Performance
    pub avg_hot_latency_ms: f64,
    pub avg_warm_latency_ms: f64,
    pub avg_cold_latency_ms: f64,
}
```

### Logging Integration
```rust
use tracing::{info, debug, warn};

impl BookCorePool {
    pub fn execute(&self, ...) -> BookResult<String> {
        let start = Instant::now();

        if let Some(cached) = self.try_hot_cache(source_id) {
            let latency = start.elapsed();
            debug!(
                source_id = %source_id,
                tier = "hot",
                latency_ms = latency.as_millis(),
                "Cache hit"
            );
            // ...
        }

        // Periodic stats logging
        if self.stats().total_requests() % 1000 == 0 {
            info!(
                hot_hit_rate = %self.stats().hot_hit_rate(),
                warm_hit_rate = %self.stats().warm_hit_rate(),
                "Pool performance"
            );
        }
    }
}
```

---

## 9. Risk Assessment

### Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Memory leak in cached Contexts** | High | Strict TTL enforcement, monitoring, fallback to cold start |
| **Context state pollution** | High | Each source gets isolated Context (natural Boa isolation) |
| **Cache stampede on popular source** | Medium | LRU protects most popular sources in HOT tier |
| **Uneven cache distribution** | Low | LRU automatically adapts to access patterns |
| **Complexity in lifecycle management** | Medium | Comprehensive testing, clear state machine |

---

## 10. Success Criteria

### Performance Targets
- ✅ HOT cache hit latency: <10ms (vs 100ms cold start)
- ✅ Cache hit rate: >60% for typical workloads
- ✅ Memory overhead: <400MB for 50 cached sources

### Quality Targets
- ✅ Test coverage: >80%
- ✅ No memory leaks in 24h stress test
- ✅ Backward compatible API

---

## 11. Future Enhancements

### Short-term (1-2 months)
- [ ] Adaptive cache sizing based on memory pressure
- [ ] Per-source memory limits
- [ ] Async execution support

### Long-term (6+ months)
- [ ] Distributed caching (Redis/Memcached)
- [ ] Machine learning-based prefetching
- [ ] Context serialization for persistence

---

## 12. References

- [Boa Engine Documentation](https://docs.rs/boa_engine)
- [LRU Cache Algorithm](https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_recently_used_(LRU))
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

---

**Document Status**: Draft for Review
**Next Steps**: Obtain stakeholder approval before implementation
**Estimated Timeline**: 4-5 weeks for full implementation
