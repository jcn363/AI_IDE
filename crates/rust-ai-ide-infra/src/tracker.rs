//! Memory block tracker for monitoring memory fragmentation

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use dashmap::DashMap;

/// Tracks memory blocks and fragmentation patterns
#[derive(Debug)]
pub struct MemoryBlockTracker {
    /// Active memory blocks
    blocks: Arc<RwLock<DashMap<Uuid, MemoryBlock>>>,

    /// Fragmentation statistics
    stats: Arc<RwLock<FragmentationStats>>,

    /// Pool-specific trackers
    pool_trackers: Arc<RwLock<DashMap<String, PoolTracker>>>,
}

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    /// Unique block identifier
    pub id: Uuid,

    /// Pool identifier
    pub pool_id: String,

    /// Block size in bytes
    pub size: usize,

    /// Allocation timestamp
    pub allocated_at: Instant,

    /// Last access timestamp
    pub last_accessed: Instant,

    /// Number of access operations
    pub access_count: u64,

    /// Block address for fragmentation analysis
    pub address: Option<usize>,

    /// Is block free
    pub is_free: bool,
}

#[derive(Debug, Clone)]
pub struct FragmentationStats {
    /// Total allocated memory
    pub total_allocated: usize,

    /// Total free memory
    pub total_free: usize,

    /// Total memory (allocated + free)
    pub total_memory: usize,

    /// Number of allocated blocks
    pub allocated_blocks: usize,

    /// Number of free blocks
    pub free_blocks: usize,

    /// Fragmentation ratio (0.0-1.0)
    pub fragmentation_ratio: f64,

    /// Average block size
    pub average_block_size: f64,

    /// Largest contiguous free block
    pub largest_free_block: usize,
}

#[derive(Debug)]
struct PoolTracker {
    pool_id: String,
    blocks: DashMap<Uuid, MemoryBlock>,
    stats: FragmentationStats,
}

impl Default for FragmentationStats {
    fn default() -> Self {
        Self {
            total_allocated: 0,
            total_free: 0,
            total_memory: 0,
            allocated_blocks: 0,
            free_blocks: 0,
            fragmentation_ratio: 0.0,
            average_block_size: 0.0,
            largest_free_block: 0,
        }
    }
}

impl MemoryBlockTracker {
    /// Create a new memory block tracker
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(RwLock::new(DashMap::new())),
            stats: Arc::new(RwLock::new(FragmentationStats::default())),
            pool_trackers: Arc::new(RwLock::new(DashMap::new())),
        }
    }

    /// Register a new memory block
    pub async fn register_block(&self, pool_id: String, size: usize, address: Option<usize>) -> Uuid {
        let id = Uuid::new_v4();
        let now = Instant::now();

        let block = MemoryBlock {
            id,
            pool_id: pool_id.clone(),
            size,
            allocated_at: now,
            last_accessed: now,
            access_count: 0,
            address,
            is_free: false,
        };

        // Add to global blocks
        self.blocks.read().await.insert(id, block.clone());

        // Add to pool-specific tracker
        self.add_to_pool_tracker(&pool_id, block).await;

        // Update statistics
        self.update_stats().await;

        id
    }

    /// Mark a block as accessed
    pub async fn record_access(&self, block_id: Uuid) {
        if let Some(mut block) = self.blocks.read().await.get_mut(&block_id) {
            block.last_accessed = Instant::now();
            block.access_count += 1;
        }
    }

    /// Free a memory block
    pub async fn free_block(&self, block_id: Uuid) -> bool {
        let mut freed = false;

        if let Some(mut block) = self.blocks.read().await.get_mut(&block_id) {
            block.is_free = true;
            freed = true;
        }

        if freed {
            self.update_stats().await;
        }

        freed
    }

    /// Get fragmentation statistics
    pub async fn get_fragmentation_stats(&self) -> FragmentationStats {
        self.stats.read().await.clone()
    }

    /// Get pool-specific fragmentation statistics
    pub async fn get_pool_stats(&self, pool_id: &str) -> Option<FragmentationStats> {
        let pool_trackers = self.pool_trackers.read().await;
        pool_trackers.get(pool_id).map(|tracker| tracker.stats.clone())
    }

    /// Get blocks that should be defragmented
    pub async fn get_fragmented_blocks(&self, threshold: f64) -> Vec<MemoryBlock> {
        let stats = self.stats.read().await;
        if stats.fragmentation_ratio < threshold {
            return Vec::new();
        }

        let blocks = self.blocks.read().await;
        blocks
            .iter()
            .filter_map(|entry| {
                let block = entry.value();
                if block.is_free {
                    Some(block.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Cleanup old free blocks
    pub async fn cleanup_old_blocks(&self, max_age: Duration) -> usize {
        let mut removed = 0;
        let now = Instant::now();

        // Remove from global blocks
        let blocks = self.blocks.read().await;
        let to_remove: Vec<Uuid> = blocks
            .iter()
            .filter_map(|entry| {
                let block = entry.value();
                if block.is_free && now.duration_since(block.allocated_at) > max_age {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();

        for block_id in to_remove {
            blocks.remove(&block_id);
            removed += 1;
        }

        // Update statistics
        if removed > 0 {
            self.update_stats().await;
        }

        removed
    }

    /// Add block to pool-specific tracker
    async fn add_to_pool_tracker(&self, pool_id: &str, block: MemoryBlock) {
        let mut pool_trackers = self.pool_trackers.write().await;

        pool_trackers
            .entry(pool_id.to_string())
            .or_insert_with(|| PoolTracker {
                pool_id: pool_id.to_string(),
                blocks: DashMap::new(),
                stats: FragmentationStats::default(),
            })
            .blocks
            .insert(block.id, block);
    }

    /// Update fragmentation statistics
    async fn update_stats(&self) {
        let blocks = self.blocks.read().await;
        let mut stats = FragmentationStats::default();

        let mut free_blocks = Vec::new();

        for entry in blocks.iter() {
            let block = entry.value();

            stats.total_memory += block.size;

            if block.is_free {
                stats.total_free += block.size;
                stats.free_blocks += 1;
                free_blocks.push(block.size);
            } else {
                stats.total_allocated += block.size;
                stats.allocated_blocks += 1;
            }
        }

        // Calculate fragmentation ratio (1 - largest_free / total_free)
        stats.largest_free_block = free_blocks.iter().cloned().max().unwrap_or(0);

        if stats.total_free > 0 {
            stats.fragmentation_ratio = 1.0 - (stats.largest_free_block as f64 / stats.total_free as f64);
        }

        // Calculate average block size
        let total_blocks = stats.allocated_blocks + stats.free_blocks;
        if total_blocks > 0 {
            stats.average_block_size = stats.total_memory as f64 / total_blocks as f64;
        }

        *self.stats.write().await = stats;
    }
}

impl MemoryBlock {
    /// Get age of the block
    pub fn age(&self) -> Duration {
        self.allocated_at.elapsed()
    }

    /// Get time since last access
    pub fn time_since_access(&self) -> Duration {
        self.last_accessed.elapsed()
    }

    /// Check if block should be considered for defragmentation
    pub fn should_defragment(&self, max_age: Duration, max_idle_time: Duration) -> bool {
        self.is_free &&
        (self.age() > max_age || self.time_since_access() > max_idle_time)
    }
}

impl FragmentationStats {
    /// Check if defragmentation is needed
    pub fn needs_defragmentation(&self, threshold: f64) -> bool {
        self.fragmentation_ratio >= threshold
    }

    /// Get memory utilization ratio
    pub fn utilization_ratio(&self) -> f64 {
        if self.total_memory == 0 {
            0.0
        } else {
            self.total_allocated as f64 / self.total_memory as f64
        }
    }
}