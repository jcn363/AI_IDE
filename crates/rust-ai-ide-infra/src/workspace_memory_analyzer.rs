//! Workspace memory analyzer
//!
//! This module provides specialized analysis for workspace memory patterns,
//! detecting large workspaces and optimizing compaction strategies accordingly.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::tracker::MemoryBlockTracker;
use crate::metrics::FragmentationMetricsCollector;
use crate::InfraResult;

/// Specialized analyzer for workspace memory patterns
#[derive(Debug)]
pub struct WorkspaceMemoryAnalyzer {
    /// Memory block tracker
    tracker: Arc<MemoryBlockTracker>,

    /// Metrics collector
    metrics: Arc<FragmentationMetricsCollector>,

    /// Analyzer configuration
    config: AnalyzerConfig,

    /// Analysis state
    state: Arc<RwLock<AnalyzerState>>,

    /// Analysis history
    history: Arc<RwLock<Vec<AnalysisRecord>>>,
}

/// Configuration for the workspace analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Analysis interval
    pub analysis_interval: Duration,

    /// Large workspace threshold (bytes)
    pub large_workspace_threshold: usize,

    /// Fragmentation analysis depth
    pub fragmentation_analysis_depth: usize,

    /// Memory pressure analysis window (seconds)
    pub memory_pressure_window: u64,

    /// Enable pattern detection
    pub pattern_detection_enabled: bool,

    /// Pattern history size
    pub pattern_history_size: usize,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            analysis_interval: Duration::from_secs(30),
            large_workspace_threshold: 1_073_741_824, // 1GB
            fragmentation_analysis_depth: 1000,
            memory_pressure_window: 300, // 5 minutes
            pattern_detection_enabled: true,
            pattern_history_size: 100,
        }
    }
}

/// Internal state of the analyzer
#[derive(Debug)]
struct AnalyzerState {
    /// Last analysis time
    last_analysis: Option<Instant>,

    /// Current workspace size estimate
    current_workspace_size: usize,

    /// Large workspace detected
    large_workspace_detected: bool,

    /// Current fragmentation pattern
    fragmentation_pattern: FragmentationPattern,

    /// Memory pressure trend
    memory_pressure_trend: MemoryPressureTrend,

    /// Analysis cycle count
    analysis_cycles: usize,
}

/// Fragmentation pattern analysis
#[derive(Debug, Clone)]
struct FragmentationPattern {
    /// Average block size
    avg_block_size: usize,

    /// Block size distribution
    size_distribution: Vec<(usize, usize)>, // (size_range, count)

    /// Fragmentation hotspots
    hotspots: Vec<FragmentationHotspot>,

    /// Pattern classification
    pattern_type: PatternType,
}

/// Memory pressure trend analysis
#[derive(Debug, Clone)]
struct MemoryPressureTrend {
    /// Current pressure level
    current_pressure: f64,

    /// Pressure trend (increasing/decreasing/stable)
    trend: TrendDirection,

    /// Pressure volatility
    volatility: f64,

    /// Peak pressure periods
    peak_periods: Vec<PressurePeak>,
}

/// Analysis record for historical tracking
#[derive(Debug, Clone)]
struct AnalysisRecord {
    /// Timestamp of analysis
    timestamp: Instant,

    /// Workspace size at analysis time
    workspace_size: usize,

    /// Fragmentation ratio
    fragmentation_ratio: f64,

    /// Memory pressure
    memory_pressure: f64,

    /// Block count
    block_count: usize,

    /// Large workspace detected
    large_workspace_detected: bool,

    /// Fragmentation pattern
    pattern: FragmentationPattern,
}

/// Fragmentation hotspot identification
#[derive(Debug, Clone)]
struct FragmentationHotspot {
    /// Memory region start address
    region_start: usize,

    /// Region size
    region_size: usize,

    /// Fragmentation level in this region
    fragmentation_level: f64,

    /// Block count in this region
    block_count: usize,
}

/// Pattern type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// Uniform block sizes
    Uniform,

    /// Variable block sizes with clustering
    Clustered,

    /// Highly fragmented with many small blocks
    Scattered,

    /// Large blocks with some fragmentation
    LargeBlocks,

    /// Mixed pattern with various sizes
    Mixed,
}

/// Trend direction for analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    /// Increasing trend
    Increasing,

    /// Decreasing trend
    Decreasing,

    /// Stable trend
    Stable,

    /// Volatile/unpredictable
    Volatile,
}

/// Memory pressure peak identification
#[derive(Debug, Clone)]
struct PressurePeak {
    /// Peak timestamp
    timestamp: Instant,

    /// Peak pressure value
    pressure_value: f64,

    /// Duration of peak
    duration: Duration,

    /// Peak intensity
    intensity: f64,
}

/// Status information for the analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerStatus {
    /// Last analysis timestamp
    pub last_analysis: Option<Instant>,

    /// Current workspace size
    pub workspace_size: usize,

    /// Large workspace detected
    pub large_workspace_detected: bool,

    /// Current fragmentation pattern
    pub fragmentation_pattern: PatternType,

    /// Memory pressure trend
    pub memory_pressure_trend: TrendDirection,

    /// Analysis cycle count
    pub analysis_cycles: usize,

    /// Fragmentation hotspots count
    pub hotspots_count: usize,

    /// Average block size
    pub avg_block_size: usize,
}

impl WorkspaceMemoryAnalyzer {
    /// Create a new workspace memory analyzer
    pub fn new(
        tracker: Arc<MemoryBlockTracker>,
        metrics: Arc<FragmentationMetricsCollector>,
    ) -> Self {
        Self {
            tracker,
            metrics,
            config: AnalyzerConfig::default(),
            state: Arc::new(RwLock::new(AnalyzerState {
                last_analysis: None,
                current_workspace_size: 0,
                large_workspace_detected: false,
                fragmentation_pattern: FragmentationPattern {
                    avg_block_size: 0,
                    size_distribution: Vec::new(),
                    hotspots: Vec::new(),
                    pattern_type: PatternType::Uniform,
                },
                memory_pressure_trend: MemoryPressureTrend {
                    current_pressure: 0.0,
                    trend: TrendDirection::Stable,
                    volatility: 0.0,
                    peak_periods: Vec::new(),
                },
                analysis_cycles: 0,
            })),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Analyze workspace memory patterns
    pub async fn analyze_workspace(&self) -> super::large_workspace_compactor::WorkspaceAnalysis {
        let start_time = Instant::now();

        // Get current memory statistics
        let stats = self.tracker.get_fragmentation_stats().await;
        let metrics = self.metrics.get_current_metrics().await;

        // Analyze fragmentation pattern
        let fragmentation_pattern = self.analyze_fragmentation_pattern().await;

        // Analyze memory pressure trend
        let memory_pressure_trend = self.analyze_memory_pressure_trend().await;

        // Detect large workspace
        let workspace_size = self.estimate_workspace_size().await;
        let large_workspace_detected = workspace_size > self.config.large_workspace_threshold;

        // Classify fragmentation pattern
        let pattern_type = self.classify_fragmentation_pattern(&fragmentation_pattern);

        // Update internal state
        let mut state = self.state.write().await;
        state.last_analysis = Some(start_time);
        state.current_workspace_size = workspace_size;
        state.large_workspace_detected = large_workspace_detected;
        state.fragmentation_pattern = fragmentation_pattern.clone();
        state.memory_pressure_trend = memory_pressure_trend.clone();
        state.analysis_cycles += 1;

        // Record analysis in history
        let record = AnalysisRecord {
            timestamp: start_time,
            workspace_size,
            fragmentation_ratio: metrics.stats.fragmentation_ratio,
            memory_pressure: metrics.memory_pressure,
            block_count: stats.allocated_blocks,
            large_workspace_detected,
            pattern: fragmentation_pattern,
        };

        let mut history = self.history.write().await;
        history.push(record);

        // Keep history size manageable
        if history.len() > self.config.pattern_history_size {
            history.remove(0);
        }

        drop(state);
        drop(history);

        super::large_workspace_compactor::WorkspaceAnalysis {
            fragmentation_ratio: metrics.stats.fragmentation_ratio,
            memory_pressure: metrics.memory_pressure,
            large_workspace_detected,
        }
    }

    /// Analyze fragmentation pattern in detail
    async fn analyze_fragmentation_pattern(&self) -> FragmentationPattern {
        let fragmented_blocks = self.tracker.get_fragmented_blocks(0.1).await;

        if fragmented_blocks.is_empty() {
            return FragmentationPattern {
                avg_block_size: 0,
                size_distribution: Vec::new(),
                hotspots: Vec::new(),
                pattern_type: PatternType::Uniform,
            };
        }

        // Calculate average block size
        let total_size: usize = fragmented_blocks.iter().map(|b| b.size).sum();
        let avg_block_size = total_size / fragmented_blocks.len();

        // Analyze size distribution
        let size_distribution = self.analyze_size_distribution(&fragmented_blocks);

        // Identify fragmentation hotspots
        let hotspots = self.identify_fragmentation_hotspots(&fragmented_blocks).await;

        FragmentationPattern {
            avg_block_size,
            size_distribution,
            hotspots,
            pattern_type: PatternType::Mixed, // Will be classified later
        }
    }

    /// Analyze memory pressure trend
    async fn analyze_memory_pressure_trend(&self) -> MemoryPressureTrend {
        let history = self.history.read().await;

        if history.is_empty() {
            return MemoryPressureTrend {
                current_pressure: 0.0,
                trend: TrendDirection::Stable,
                volatility: 0.0,
                peak_periods: Vec::new(),
            };
        }

        let current_pressure = history.last().unwrap().memory_pressure;

        // Calculate trend
        let trend = self.calculate_pressure_trend(&history);

        // Calculate volatility
        let volatility = self.calculate_pressure_volatility(&history);

        // Identify peak periods
        let peak_periods = self.identify_pressure_peaks(&history);

        MemoryPressureTrend {
            current_pressure,
            trend,
            volatility,
            peak_periods,
        }
    }

    /// Estimate current workspace size
    async fn estimate_workspace_size(&self) -> usize {
        let stats = self.tracker.get_fragmentation_stats().await;
        stats.total_allocated
    }

    /// Analyze size distribution of memory blocks
    fn analyze_size_distribution(&self, blocks: &[crate::tracker::MemoryBlock]) -> Vec<(usize, usize)> {
        let mut distribution = std::collections::HashMap::new();

        for block in blocks {
            let size_range = self.classify_block_size(block.size);
            *distribution.entry(size_range).or_insert(0) += 1;
        }

        let mut result: Vec<_> = distribution.into_iter().collect();
        result.sort_by_key(|(size_range, _)| *size_range);
        result
    }

    /// Classify block size into ranges
    fn classify_block_size(&self, size: usize) -> usize {
        match size {
            0..=1023 => 0,           // < 1KB
            1024..=10239 => 1024,    // 1KB - 10KB
            10240..=102399 => 10240, // 10KB - 100KB
            102400..=1048575 => 102400, // 100KB - 1MB
            1048576..=10485759 => 1048576, // 1MB - 10MB
            _ => 10485760,           // > 10MB
        }
    }

    /// Identify fragmentation hotspots
    async fn identify_fragmentation_hotspots(&self, blocks: &[crate::tracker::MemoryBlock]) -> Vec<FragmentationHotspot> {
        let mut hotspots = Vec::new();

        // Group blocks by memory regions
        let mut region_groups = std::collections::HashMap::new();

        for block in blocks {
            let region_start = (block.address / (1024 * 1024)) * (1024 * 1024); // 1MB regions
            region_groups.entry(region_start).or_insert_with(Vec::new).push(block);
        }

        // Analyze each region
        for (region_start, region_blocks) in region_groups {
            if region_blocks.len() < 10 {
                continue; // Skip regions with few blocks
            }

            let region_size = region_blocks.iter().map(|b| b.size).sum::<usize>();
            let fragmentation_level = self.calculate_region_fragmentation(&region_blocks);

            if fragmentation_level > 0.3 { // Significant fragmentation threshold
                hotspots.push(FragmentationHotspot {
                    region_start,
                    region_size,
                    fragmentation_level,
                    block_count: region_blocks.len(),
                });
            }
        }

        // Sort by fragmentation level (highest first)
        hotspots.sort_by(|a, b| b.fragmentation_level.partial_cmp(&a.fragmentation_level).unwrap());
        hotspots.truncate(10); // Keep top 10 hotspots

        hotspots
    }

    /// Calculate fragmentation level for a region
    fn calculate_region_fragmentation(&self, blocks: &[&crate::tracker::MemoryBlock]) -> f64 {
        if blocks.is_empty() {
            return 0.0;
        }

        let total_size: usize = blocks.iter().map(|b| b.size).sum();
        let max_block_size = blocks.iter().map(|b| b.size).max().unwrap_or(0);

        if total_size == 0 {
            return 0.0;
        }

        // Fragmentation is high when there are many small blocks relative to total size
        let avg_block_size = total_size as f64 / blocks.len() as f64;
        let max_to_avg_ratio = max_block_size as f64 / avg_block_size;

        // Higher ratio indicates more fragmentation
        (max_to_avg_ratio - 1.0).max(0.0) / 10.0 // Normalize to 0-1 range
    }

    /// Calculate memory pressure trend
    fn calculate_pressure_trend(&self, history: &[AnalysisRecord]) -> TrendDirection {
        if history.len() < 3 {
            return TrendDirection::Stable;
        }

        let recent = &history[history.len().saturating_sub(3)..];
        let pressures: Vec<f64> = recent.iter().map(|r| r.memory_pressure).collect();

        let first = pressures[0];
        let last = pressures[pressures.len() - 1];
        let diff = last - first;

        if diff.abs() < 0.05 {
            TrendDirection::Stable
        } else if diff > 0.1 {
            TrendDirection::Increasing
        } else if diff < -0.1 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Volatile
        }
    }

    /// Calculate pressure volatility
    fn calculate_pressure_volatility(&self, history: &[AnalysisRecord]) -> f64 {
        if history.len() < 2 {
            return 0.0;
        }

        let pressures: Vec<f64> = history.iter().map(|r| r.memory_pressure).collect();
        let mean = pressures.iter().sum::<f64>() / pressures.len() as f64;

        let variance = pressures.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / pressures.len() as f64;

        variance.sqrt()
    }

    /// Identify memory pressure peaks
    fn identify_pressure_peaks(&self, history: &[AnalysisRecord]) -> Vec<PressurePeak> {
        let mut peaks = Vec::new();
        let mut current_peak: Option<PressurePeak> = None;

        for record in history {
            if record.memory_pressure > 0.8 { // Peak threshold
                if let Some(ref mut peak) = current_peak {
                    // Extend current peak
                    peak.pressure_value = peak.pressure_value.max(record.memory_pressure);
                    peak.duration = record.timestamp - peak.timestamp;
                    peak.intensity = (peak.intensity + record.memory_pressure) / 2.0;
                } else {
                    // Start new peak
                    current_peak = Some(PressurePeak {
                        timestamp: record.timestamp,
                        pressure_value: record.memory_pressure,
                        duration: Duration::from_secs(0),
                        intensity: record.memory_pressure,
                    });
                }
            } else if let Some(peak) = current_peak.take() {
                // End current peak
                if peak.duration > Duration::from_secs(30) { // Minimum peak duration
                    peaks.push(peak);
                }
            }
        }

        // Add final peak if exists
        if let Some(peak) = current_peak {
            if peak.duration > Duration::from_secs(30) {
                peaks.push(peak);
            }
        }

        peaks
    }

    /// Classify fragmentation pattern
    fn classify_fragmentation_pattern(&self, pattern: &FragmentationPattern) -> PatternType {
        if pattern.size_distribution.is_empty() {
            return PatternType::Uniform;
        }

        let total_blocks: usize = pattern.size_distribution.iter().map(|(_, count)| count).sum();
        let small_blocks: usize = pattern.size_distribution.iter()
            .filter(|(size_range, _)| *size_range <= 10240) // <= 10KB
            .map(|(_, count)| count)
            .sum();

        let small_block_ratio = small_blocks as f64 / total_blocks as f64;

        if small_block_ratio > 0.7 {
            PatternType::Scattered
        } else if pattern.avg_block_size > 1024 * 1024 { // > 1MB average
            PatternType::LargeBlocks
        } else if pattern.size_distribution.len() > 5 {
            PatternType::Mixed
        } else {
            PatternType::Clustered
        }
    }

    /// Get current analyzer status
    pub async fn get_status(&self) -> AnalyzerStatus {
        let state = self.state.read().await;

        AnalyzerStatus {
            last_analysis: state.last_analysis,
            workspace_size: state.current_workspace_size,
            large_workspace_detected: state.large_workspace_detected,
            fragmentation_pattern: state.fragmentation_pattern.pattern_type,
            memory_pressure_trend: state.memory_pressure_trend.trend,
            analysis_cycles: state.analysis_cycles,
            hotspots_count: state.fragmentation_pattern.hotspots.len(),
            avg_block_size: state.fragmentation_pattern.avg_block_size,
        }
    }

    /// Get workspace size estimate
    pub async fn get_workspace_size(&self) -> usize {
        let state = self.state.read().await;
        state.current_workspace_size
    }

    /// Export analyzer status for monitoring
    pub async fn export_status(&self) -> serde_json::Value {
        let status = self.get_status().await;
        let state = self.state.read().await;
        let history = self.history.read().await;

        serde_json::json!({
            "analyzer": {
                "last_analysis_seconds_ago": status.last_analysis.map(|t| t.elapsed().as_secs()).unwrap_or(0),
                "workspace_size": status.workspace_size,
                "large_workspace_detected": status.large_workspace_detected,
                "fragmentation_pattern": format!("{:?}", status.fragmentation_pattern),
                "memory_pressure_trend": format!("{:?}", status.memory_pressure_trend),
                "analysis_cycles": status.analysis_cycles,
                "hotspots_count": status.hotspots_count,
                "avg_block_size": status.avg_block_size
            },
            "pattern_analysis": {
                "size_distribution": state.fragmentation_pattern.size_distribution,
                "hotspots": state.fragmentation_pattern.hotspots.iter().take(5).map(|h| {
                    serde_json::json!({
                        "region_start": h.region_start,
                        "region_size": h.region_size,
                        "fragmentation_level": h.fragmentation_level,
                        "block_count": h.block_count
                    })
                }).collect::<Vec<_>>()
            },
            "pressure_analysis": {
                "current_pressure": state.memory_pressure_trend.current_pressure,
                "volatility": state.memory_pressure_trend.volatility,
                "peak_periods_count": state.memory_pressure_trend.peak_periods.len()
            },
            "history": {
                "size": history.len(),
                "last_record": history.last().map(|r| {
                    serde_json::json!({
                        "timestamp_seconds_ago": r.timestamp.elapsed().as_secs(),
                        "workspace_size": r.workspace_size,
                        "fragmentation_ratio": r.fragmentation_ratio,
                        "block_count": r.block_count
                    })
                })
            },
            "config": {
                "analysis_interval_seconds": self.config.analysis_interval.as_secs(),
                "large_workspace_threshold": self.config.large_workspace_threshold,
                "fragmentation_analysis_depth": self.config.fragmentation_analysis_depth,
                "pattern_detection_enabled": self.config.pattern_detection_enabled
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let tracker = Arc::new(MemoryBlockTracker::new());
        let metrics = Arc::new(FragmentationMetricsCollector::new(Arc::clone(&tracker)));

        let analyzer = WorkspaceMemoryAnalyzer::new(tracker, metrics);
        let status = analyzer.get_status().await;

        assert_eq!(status.analysis_cycles, 0);
        assert!(!status.large_workspace_detected);
    }

    #[tokio::test]
    async fn test_workspace_analysis() {
        let tracker = Arc::new(MemoryBlockTracker::new());
        let metrics = Arc::new(FragmentationMetricsCollector::new(Arc::clone(&tracker)));

        let analyzer = WorkspaceMemoryAnalyzer::new(tracker, metrics);
        let analysis = analyzer.analyze_workspace().await;

        // Should return valid analysis even with empty data
        assert!(!analysis.large_workspace_detected); // Default state

        let status = analyzer.get_status().await;
        assert_eq!(status.analysis_cycles, 1);
    }
}