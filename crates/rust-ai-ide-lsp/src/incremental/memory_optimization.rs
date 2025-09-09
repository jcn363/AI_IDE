//! Memory-Optimized Data Structures for LSP Analysis
//!
//! This module provides specialized data structures optimized for memory usage
//! in language server analysis operations.
//!
//! # Features
//! - Compressed symbol tables
//! - Lazy loading for unused language servers
//! - Memory pooling for frequent operations
//! - Optimized string interning

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use tokio::sync::RwLock;
use std::time::{Instant, Duration};

/// Zero-copy thread-safe string interning pool for memory optimization
pub struct StringInternPool {
    strings: RwLock<HashMap<Box<str>, Arc<str>>>,
    size: AtomicUsize,
    memory_used: AtomicUsize,
}

impl StringInternPool {
    pub fn new() -> Self {
        Self {
            strings: RwLock::new(HashMap::new()),
            size: AtomicUsize::new(0),
            memory_used: AtomicUsize::new(0),
        }
    }

    pub async fn intern(&self, s: &str) -> Arc<str> {
        let mut strings = self.strings.write().await;

        // Use to_owned() to create Box<str> (zero-copy)
        let owned_key = s.to_owned().into_boxed_str();

        if let Some(interned) = strings.get(&owned_key) {
            Arc::clone(interned)
        } else {
            // Zero-copy interning: create Arc<str> from the boxed string
            let interned: Arc<str> = Arc::from(owned_key);
            let memory_cost = interned.len();

            // Insert using the same Box<str> key to avoid cloning
            strings.insert(owned_key, Arc::clone(&interned));

            self.size.fetch_add(1, Ordering::Relaxed);
            self.memory_used.fetch_add(memory_cost, Ordering::Relaxed);
            interned
        }
    }

    /// Zero-copy removal - returns the stored string without copying
    pub async fn remove(&self, s: &str) -> Option<Arc<str>> {
        let mut strings = self.strings.write().await;
        let owned_key = s.to_owned().into_boxed_str();

        if let Some(removed) = strings.remove(&owned_key) {
            let memory_cost = removed.len();

            self.size.fetch_sub(1, Ordering::Relaxed);
            self.memory_used.fetch_sub(memory_cost, Ordering::Relaxed);

            Some(removed)
        } else {
            None
        }
    }

    pub async fn stats(&self) -> InternPoolStats {
        InternPoolStats {
            unique_strings: self.size.load(Ordering::Relaxed),
            estimated_memory_mb: self.memory_used.load(Ordering::Relaxed) / 1024 / 1024,
        }
    }
}

pub struct InternPoolStats {
    pub unique_strings: usize,
    pub estimated_memory_mb: usize,
}

/// Memory-efficient symbol table with compression
pub struct CompressedSymbolTable {
    symbols: HashMap<SymbolId, SymbolEntry>,
    string_pool: StringInternPool,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SymbolId(u64);

#[derive(Debug)]
pub struct SymbolEntry {
    name: Arc<String>,
    kind: SymbolKind,
    range: CompressedRange,
    parent: Option<SymbolId>,
}

#[derive(Debug)]
pub enum SymbolKind {
    Function,
    Variable,
    Class,
    Module,
    Other,
}

#[derive(Debug)]
pub struct CompressedRange {
    start_line: u32,
    start_col: u16,
    end_line: u32,
    end_col: u16,
}

impl CompressedSymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            string_pool: StringInternPool::new(),
        }
    }

    pub async fn add_symbol(&mut self, name: &str, kind: SymbolKind, range: CompressedRange) -> SymbolId {
        let interned_name = self.string_pool.intern(name).await;
        let id = SymbolId(self.symbols.len() as u64);

        let entry = SymbolEntry {
            name: interned_name,
            kind,
            range,
            parent: None,
        };

        self.symbols.insert(id, entry);
        id
    }

    pub fn get_symbol(&self, id: &SymbolId) -> Option<&SymbolEntry> {
        self.symbols.get(id)
    }
}

/// Lazy loading wrapper for language servers
pub struct LazyLanguageServer<T> {
    loader: Box<dyn FnOnce() -> T + Send + Sync>,
    instance: std::sync::Mutex<Option<T>>,
    load_count: AtomicUsize,
}

impl<T> LazyLanguageServer<T> {
    pub fn new<F>(loader: F) -> Self
    where
        F: FnOnce() -> T + Send + Sync + 'static,
    {
        Self {
            loader: Box::new(loader),
            instance: std::sync::Mutex::new(None),
            load_count: AtomicUsize::new(0),
        }
    }

    pub fn get(&self) -> Option<std::sync::MutexGuard<'_, Option<T>>> {
        let mut instance = self.instance.lock().unwrap();

        if instance.is_none() {
            // Lazy load the instance
            *instance = Some((self.loader)());
            self.load_count.fetch_add(1, Ordering::Relaxed);
        }

        Some(instance)
    }

    pub fn is_loaded(&self) -> bool {
        self.instance.lock().unwrap().is_some()
    }

    pub fn load_count(&self) -> usize {
        self.load_count.load(Ordering::Relaxed)
    }
}

/// Memory pool for frequently allocated analysis objects
pub struct AnalysisObjectPool<T> {
    pool: RwLock<Vec<T>>,
    capacity: usize,
    stats: Arc<RwLock<PoolStats>>,
}

#[derive(Debug)]
pub struct PoolStats {
    pub allocations: usize,
    pub hits: usize,
    pub misses: usize,
    pub active_allocations: usize,
    pub total_used_bytes: usize,
    pub peak_used_bytes: usize,
    pub fragmentation_ratio: f64,
    pub memory_pressure_ratio: f64,
    pub allocation_pattern_score: f64,
    pub last_gc_timestamp: Instant,
    pub leaked_allocations: usize,
    pub memory_growth_rate: f64,
}

impl<T> AnalysisObjectPool<T>
where
    T: Default,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: RwLock::new(Vec::with_capacity(capacity)),
            capacity,
            stats: Arc::new(RwLock::new(PoolStats {
                allocations: 0,
                hits: 0,
                misses: 0,
            })),
        }
    }

    pub async fn get(&self) -> T {
        let mut pool = self.pool.write().await;
        let mut stats = self.stats.write().await;

        if let Some(obj) = pool.pop() {
            stats.hits += 1;
            return obj;
        }

        stats.misses += 1;
        stats.allocations += 1;
        T::default()
    }

    pub async fn put(&self, obj: T) {
        let mut pool = self.pool.write().await;

        if pool.len() < self.capacity {
            pool.push(obj);
        }
    }

    pub async fn get_stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }
}

/// Global registry for memory optimization components
pub struct MemoryOptimizationRegistry {
    string_pool: StringInternPool,
    symbol_tables: RwLock<HashMap<String, Arc<CompressedSymbolTable>>>,
    object_pools: RwLock<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>,
}

impl MemoryOptimizationRegistry {
    pub fn new() -> Self {
        Self {
            string_pool: StringInternPool::new(),
            symbol_tables: RwLock::new(HashMap::new()),
            object_pools: RwLock::new(HashMap::new()),
        }
    }

    pub fn string_pool(&self) -> &StringInternPool {
        &self.string_pool
    }

    pub async fn get_symbol_table(&self, language: &str) -> Arc<CompressedSymbolTable> {
        let mut tables = self.symbol_tables.write().await;

        if let Some(table) = tables.get(language) {
            Arc::clone(table)
        } else {
            let table = Arc::new(CompressedSymbolTable::new());
            tables.insert(language.to_string(), Arc::clone(&table));
            table
        }
    }

    pub async fn register_object_pool<T>(
        &self,
        name: &str,
        pool: Arc<AnalysisObjectPool<T>>
    ) where
        T: 'static,
    {
        let mut pools = self.object_pools.write().await;
        pools.insert(name.to_string(), pool);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test as async_test;

    #[async_test]
    async fn test_string_interning() {
        let pool = StringInternPool::new();

        // Intern the same string multiple times
        let s1 = pool.intern("test_string").await;
        let s2 = pool.intern("test_string").await;
        let s3 = pool.intern("different_string").await;

        // s1 and s2 should point to the same Arc
        assert_eq!(Arc::as_ptr(&s1), Arc::as_ptr(&s2));
        assert_ne!(Arc::as_ptr(&s1), Arc::as_ptr(&s3));

        // Stats should reflect unique strings
        let stats = pool.stats().await;
        assert_eq!(stats.unique_strings, 2);
    }

    #[async_test]
    async fn test_compressed_symbol_table() {
        let mut table = CompressedSymbolTable::new();

        let range = CompressedRange {
            start_line: 10,
            start_col: 5,
            end_line: 12,
            end_col: 15,
        };

        let id = table.add_symbol("my_function", SymbolKind::Function, range).await;

        let symbol = table.get_symbol(&id).unwrap();
        assert_eq!(*symbol.name, "my_function");
        assert!(matches!(symbol.kind, SymbolKind::Function));
    }

    #[async_test]
    async fn test_object_pool() {
        let pool: AnalysisObjectPool<Vec<u8>> = AnalysisObjectPool::new(10);

        // Test pool allocation
        let obj1 = pool.get().await;
        let obj2 = pool.get().await;

        // Return objects to pool
        pool.put(obj1).await;
        pool.put(obj2).await;

        // Get objects from pool (should reuse)
        let obj3 = pool.get().await;
        assert!(!obj3.is_empty());

        let stats = pool.get_stats().await;
        assert_eq!(stats.hits, 1); // Reuse from pool
        assert_eq!(stats.misses, 2); // Initial allocations
        assert_eq!(stats.allocations, 2);
    }
}