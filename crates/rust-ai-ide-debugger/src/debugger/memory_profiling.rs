//! Advanced memory visualization and profiling tools
//!
//! This module provides comprehensive memory profiling capabilities including:
//! - Memory allocation tracking with temporal analysis
//! - Object lifetime visualization
//! - Memory fragmentation analysis
//! - Leak detection with categorization
//! - Heap visualization tools

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Represents a memory allocation in the heap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    /// Allocation address
    pub address:            usize,
    /// Allocation size in bytes
    pub size:               usize,
    /// Number of bytes actually used (for some allocators)
    pub used_size:          Option<usize>,
    /// Timestamp when allocation occurred
    pub allocated_at:       u64,
    /// Call stack at allocation site
    pub allocation_stack:   Vec<String>,
    /// Timestamp when deallocation occurred (None if still allocated)
    pub deallocated_at:     Option<u64>,
    /// Call stack at deallocation site
    pub deallocation_stack: Option<Vec<String>>,
    /// Thread ID that performed the allocation
    pub thread_id:          Option<u32>,
    /// Allocation type (malloc, new, etc.)
    pub allocation_type:    AllocationType,
    /// Optional metadata about the object being allocated
    pub metadata:           String,
}

/// Types of memory allocations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllocationType {
    /// Standard heap allocation
    Heap,
    /// Stack allocation (tracked for completeness)
    Stack,
    /// Memory mapped allocation
    MemoryMapped,
    /// Shared memory
    Shared,
    /// Custom allocator type
    Custom(String),
}

/// Memory leak classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakClassification {
    /// Leaked address
    pub address:          usize,
    /// Estimated type of leak
    pub leak_type:        LeakType,
    /// Size of the leak in bytes
    pub size:             usize,
    /// Time the memory has been leaked
    pub leak_duration:    Duration,
    /// Allocation call stack
    pub allocation_stack: Vec<String>,
    /// Severity score (0.0 - 1.0, higher is more severe)
    pub severity_score:   f64,
}

/// Types of memory leaks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeakType {
    /// Definitely lost (no pointers to the allocation)
    DefinitelyLost,
    /// Indirectly lost (reachable through pointer but itself dead)
    IndirectlyLost,
    /// Possibly lost (reachable through pointer but unclear)
    PossiblyLost,
    /// Still reachable but suspicious (e.g., growing containers)
    StillReachableButSuspicious,
}

/// Memory fragmentation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationAnalysis {
    /// Average fragmentation percentage
    pub average_fragmentation:  f64,
    /// Largest free block size
    pub largest_free_block:     usize,
    /// Total free memory in bytes
    pub total_free_memory:      usize,
    /// Total allocated memory in bytes
    pub total_allocated_memory: usize,
    /// Fragmentation hotspots (areas with high fragmentation)
    pub fragmentation_hotspots: Vec<FragmentationHotspot>,
}

/// A memory fragmentation hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationHotspot {
    /// Memory address range
    pub address_range:            (usize, usize),
    /// Fragmentation percentage in this area
    pub fragmentation_percentage: f64,
    /// Recommended size for reallocation
    pub recommended_alignment:    usize,
}

/// Heap visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapVisualization {
    /// Total heap size
    pub total_heap_size:      usize,
    /// Used heap size
    pub used_heap_size:       usize,
    /// Free heap size
    pub free_heap_size:       usize,
    /// Memory segments with classification
    pub memory_segments:      Vec<MemorySegment>,
    /// Allocation histogram data
    pub allocation_histogram: Vec<HistogramBin>,
    /// Top memory consumers
    pub top_consumers:        Vec<TopConsumer>,
}

/// A memory segment for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySegment {
    /// Segment address
    pub address:      usize,
    /// Segment size
    pub size:         usize,
    /// Segment type
    pub segment_type: MemorySegmentType,
    /// Allocation timestamps in this segment
    pub allocations:  Vec<usize>,
}

/// Types of memory segments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemorySegmentType {
    /// Reserved but not allocated
    Reserved,
    /// Free memory available for allocation
    Free,
    /// Currently allocated memory
    Allocated,
    /// Memory marked as garbage (not yet reclaimed)
    Garbage,
}

/// Histogram bin for allocation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBin {
    /// Bin range in bytes
    pub size_range: (usize, usize),
    /// Number of allocations in this range
    pub count:      usize,
    /// Total memory used in this range
    pub total_size: usize,
}

/// Top memory consumer for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopConsumer {
    /// Allocation site identifier
    pub allocation_site:  String,
    /// Total memory consumed
    pub total_memory:     usize,
    /// Number of allocations
    pub allocation_count: usize,
    /// Average allocation size
    pub average_size:     f64,
    /// Rank based on memory consumption
    pub rank:             usize,
}

/// Memory profiling event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryProfileEvent {
    /// Memory allocation event
    Allocation(Allocation),
    /// Memory deallocation event
    Deallocation {
        address:            usize,
        deallocated_at:     u64,
        deallocation_stack: Vec<String>,
    },
    /// Heap statistics updated
    HeapStatisticsUpdated(HeapStatistics),
    /// Leak detected
    LeakDetected(LeakClassification),
    /// Fragmentation analysis completed
    FragmentationAnalysis(FragmentationAnalysis),
}

/// Heap statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapStatistics {
    /// Total heap size in bytes
    pub total_heap:         usize,
    /// Used heap size in bytes
    pub used_heap:          usize,
    /// Peak heap usage in bytes
    pub peak_usage:         usize,
    /// Number of allocations
    pub allocation_count:   usize,
    /// Number of deallocations
    pub deallocation_count: usize,
    /// Current memory fragmentation
    pub fragmentation:      f64,
}

/// Advanced memory profiler
pub struct MemoryProfiler {
    /// All current allocations
    allocations:        HashMap<usize, Allocation>,
    /// Historical allocation data for temporal analysis
    allocation_history: Vec<Allocation>,
    /// Leak candidates
    potential_leaks:    HashMap<usize, LeakCandidate>,
    /// Current heap statistics
    heap_stats:         HeapStatistics,
    /// Fragmentation analysis data
    fragmentation_data: FragmentationAnalysis,
    /// Event sender for integration
    pub event_sender:       Option<mpsc::UnboundedSender<MemoryProfileEvent>>,
    /// Profiling start time
    start_time:         Instant,
    /// Leaked addresses
    leaked_addresses:   HashSet<usize>,
}

/// Internal leak candidate tracking
#[derive(Debug, Clone)]
struct LeakCandidate {
    allocation:     Allocation,
    detection_time: Instant,
    classification: LeakType,
}

impl MemoryProfiler {
    /// Create a new memory profiler instance
    pub fn new(event_sender: Option<mpsc::UnboundedSender<MemoryProfileEvent>>) -> Self {
        Self {
            allocations: HashMap::new(),
            allocation_history: Vec::new(),
            potential_leaks: HashMap::new(),
            heap_stats: HeapStatistics {
                total_heap:         0,
                used_heap:          0,
                peak_usage:         0,
                allocation_count:   0,
                deallocation_count: 0,
                fragmentation:      0.0,
            },
            fragmentation_data: FragmentationAnalysis {
                average_fragmentation:  0.0,
                largest_free_block:     0,
                total_free_memory:      0,
                total_allocated_memory: 0,
                fragmentation_hotspots: Vec::new(),
            },
            event_sender,
            start_time: Instant::now(),
            leaked_addresses: HashSet::new(),
        }
    }

    /// Send an event to the profiling system
    fn send_event(&self, event: MemoryProfileEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(sender) = &self.event_sender {
            sender.send(event)?;
        }
        Ok(())
    }

    /// Track a memory allocation
    pub fn track_allocation(&mut self, allocation: Allocation) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Update heap statistics
        self.heap_stats.used_heap += allocation.size;
        self.heap_stats.peak_usage = self.heap_stats.peak_usage.max(self.heap_stats.used_heap);
        self.heap_stats.allocation_count += 1;
        self.heap_stats.total_heap = self.heap_stats.total_heap.max(self.heap_stats.used_heap);

        // Store allocation
        self.allocations
            .insert(allocation.address, allocation.clone());
        self.allocation_history.push(allocation.clone());

        // Analyze potential leaks periodically
        if self.heap_stats.allocation_count % 1000 == 0 {
            self.analyze_potential_leaks();
        }

        self.send_event(MemoryProfileEvent::Allocation(allocation))?;
        Ok(())
    }

    /// Track a memory deallocation
    pub fn track_deallocation(
        &mut self,
        address: usize,
        deallocated_at: u64,
        deallocation_stack: Vec<String>,
        thread_id: Option<u32>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(mut allocation) = self.allocations.remove(&address) {
            // Update allocation with deallocation info
            allocation.deallocated_at = Some(deallocated_at);
            allocation.deallocation_stack = Some(deallocation_stack.clone());

            // Update heap statistics
            self.heap_stats.used_heap = self.heap_stats.used_heap.saturating_sub(allocation.size);
            self.heap_stats.deallocation_count += 1;

            // Remove from potential leaks if it was there
            self.potential_leaks.remove(&address);
            self.leaked_addresses.remove(&address);

            // Add back to history
            self.allocation_history.push(allocation.clone());

            self.send_event(MemoryProfileEvent::Deallocation {
                address,
                deallocated_at,
                deallocation_stack,
            })?;
        } else {
            log::warn!("Attempted to deallocate unknown address: {}", address);
        }

        Ok(())
    }

    /// Update heap statistics
    pub fn update_heap_statistics(
        &mut self,
        new_stats: HeapStatistics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.heap_stats = new_stats.clone();
        self.send_event(MemoryProfileEvent::HeapStatisticsUpdated(new_stats))?;
        Ok(())
    }

    /// Update fragmentation analysis
    pub fn update_fragmentation_analysis(
        &mut self,
        analysis: FragmentationAnalysis,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.fragmentation_data = analysis.clone();
        self.send_event(MemoryProfileEvent::FragmentationAnalysis(analysis))?;
        Ok(())
    }

    /// Detect memory leaks
    pub fn analyze_potential_leaks(
        &mut self,
    ) -> Result<Vec<LeakClassification>, Box<dyn std::error::Error + Send + Sync>> {
        let leaks = self.detect_leaks();
        let classifications = leaks.clone();

        // Send leak events
        for leak in &leaks {
            self.send_event(MemoryProfileEvent::LeakDetected(leak.clone()))?;
        }

        Ok(classifications)
    }

    /// Perform leak detection
    fn detect_leaks(&mut self) -> Vec<LeakClassification> {
        let mut leaks = Vec::new();
        let now = Instant::now();

        // Find allocations that have lived for too long without deallocation
        for allocation in &self.allocation_history {
            if allocation.deallocated_at.is_none() && !self.leaked_addresses.contains(&allocation.address) {
                let allocation_time =
                    Duration::from_millis((self.start_time.elapsed().as_millis() - allocation.allocated_at as u128).try_into().unwrap());
                let leak_duration =
                    Duration::from_millis((self.start_time.elapsed().as_millis() - now.elapsed().as_millis()).try_into().unwrap());

                // Simple heuristic: if allocation has lived longer than 5 minutes, consider it suspicious
                if leak_duration > Duration::from_secs(300) {
                    let leak_type = self.classify_leak(allocation, &allocation_time);
                    let severity = self.calculate_leak_severity(allocation, &leak_duration);

                    let classification = LeakClassification {
                        address: allocation.address,
                        leak_type: leak_type.clone(),
                        size: allocation.size,
                        leak_duration,
                        allocation_stack: allocation.allocation_stack.clone(),
                        severity_score: severity,
                    };

                    leaks.push(classification.clone());
                    self.leaked_addresses.insert(allocation.address);

                    // Update potential leaks
                    self.potential_leaks
                        .insert(allocation.address, LeakCandidate {
                            allocation:     allocation.clone(),
                            detection_time: now,
                            classification: leak_type.clone(),
                        });
                }
            }
        }

        leaks
    }

    /// Classify the type of memory leak
    fn classify_leak(&self, allocation: &Allocation, lifetime: &Duration) -> LeakType {
        // Simple classification logic - in a real implementation, this would use more sophisticated
        // analysis
        if *lifetime > Duration::from_secs(3600) {
            // Very long-lived allocation that's still reachable
            LeakType::PossiblyLost
        } else if allocation.thread_id.is_some() {
            // Thread-local allocation that wasn't cleaned up
            LeakType::DefinitelyLost
        } else {
            // General leak
            LeakType::PossiblyLost
        }
    }

    /// Calculate leak severity score
    fn calculate_leak_severity(&self, allocation: &Allocation, leak_duration: &Duration) -> f64 {
        let base_severity = (allocation.size as f64) / (1024.0 * 1024.0); // Size in MB
        let time_factor = (leak_duration.as_secs() as f64) / 3600.0; // Hours leaked

        // Size-based severity (0-0.5) + time-based severity (0-0.5)
        (base_severity * 0.5).min(0.5) + (time_factor * 0.5).min(0.5)
    }

    /// Generate heap visualization data
    pub fn generate_heap_visualization(&self) -> HeapVisualization {
        let memory_segments = self.create_memory_segments();
        let histogram = self.create_allocation_histogram();
        let top_consumers = self.identify_top_consumers();

        HeapVisualization {
            total_heap_size: self.heap_stats.total_heap,
            used_heap_size: self.heap_stats.used_heap,
            free_heap_size: self
                .heap_stats
                .total_heap
                .saturating_sub(self.heap_stats.used_heap),
            memory_segments,
            allocation_histogram: histogram,
            top_consumers,
        }
    }

    /// Create memory segment information
    fn create_memory_segments(&self) -> Vec<MemorySegment> {
        let memory_range = 0..(self.heap_stats.total_heap / 1024); // 1KB ranges
        let mut segments = Vec::new();

        for addr in (0..self.heap_stats.total_heap).step_by(1024) {
            let allocations_in_range = self
                .allocations
                .values()
                .filter(|alloc| alloc.address >= addr && alloc.address < addr + 1024)
                .map(|alloc| alloc.address)
                .collect::<Vec<_>>();

            let segment_type = if allocations_in_range.is_empty() {
                MemorySegmentType::Free
            } else {
                MemorySegmentType::Allocated
            };

            segments.push(MemorySegment {
                address: addr,
                size: 1024,
                segment_type,
                allocations: allocations_in_range,
            });
        }

        segments
    }

    /// Create allocation histogram
    fn create_allocation_histogram(&self) -> Vec<HistogramBin> {
        let size_ranges = [
            (0, 64),             // Small allocations
            (64, 256),           // Medium small
            (256, 1024),         // Medium
            (1024, 4096),        // Medium large
            (4096, 16384),       // Large
            (16384, 65536),      // Very large
            (65536, usize::MAX), // Huge
        ];

        let mut histogram = Vec::new();
        for (min_size, max_size) in &size_ranges {
            let allocations_in_range = self
                .allocation_history
                .iter()
                .filter(|alloc| alloc.deallocated_at.is_none() && alloc.size >= *min_size && alloc.size < *max_size)
                .collect::<Vec<_>>();

            let total_size = allocations_in_range.iter().map(|alloc| alloc.size).sum();

            histogram.push(HistogramBin {
                size_range: (*min_size, *max_size),
                count: allocations_in_range.len(),
                total_size,
            });
        }

        histogram
    }

    /// Identify top memory consumers
    fn identify_top_consumers(&self) -> Vec<TopConsumer> {
        let mut consumers = HashMap::new();

        // Group allocations by allocation site (first frame of stack)
        for allocation in &self.allocation_history {
            if allocation.deallocated_at.is_none() && !allocation.allocation_stack.is_empty() {
                let site = allocation.allocation_stack.first().unwrap().clone();
                let entry = consumers.entry(site).or_insert((0, 0, Vec::new()));
                entry.0 += 1; // count
                entry.1 += allocation.size; // total size
                entry.2.push(allocation.size); // sizes for average
            }
        }

        // Convert to TopConsumer structs
        let mut top_consumers: Vec<_> = consumers
            .into_iter()
            .map(|(site, (count, total, sizes))| {
                let avg_size = sizes.iter().sum::<usize>() as f64 / sizes.len() as f64;
                TopConsumer {
                    allocation_site:  site,
                    total_memory:     total,
                    allocation_count: count,
                    average_size:     avg_size,
                    rank:             0, // Will be set after sorting
                }
            })
            .collect();

        // Sort by total memory consumption
        top_consumers.sort_by(|a, b| b.total_memory.cmp(&a.total_memory));

        // Assign ranks
        for (rank, consumer) in top_consumers.iter_mut().enumerate() {
            consumer.rank = rank + 1;
        }

        // Take top 10
        top_consumers.into_iter().take(10).collect()
    }

    /// Get current heap statistics
    pub fn get_heap_statistics(&self) -> &HeapStatistics {
        &self.heap_stats
    }

    /// Get fragmentation analysis
    pub fn get_fragmentation_analysis(&self) -> &FragmentationAnalysis {
        &self.fragmentation_data
    }

    /// Get all current allocations
    pub fn get_current_allocations(&self) -> Vec<&Allocation> {
        self.allocations.values().collect()
    }

    /// Get allocation history
    pub fn get_allocation_history(&self) -> &[Allocation] {
        &self.allocation_history
    }
}
