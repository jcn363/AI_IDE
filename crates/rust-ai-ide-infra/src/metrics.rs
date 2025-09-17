//! Fragmentation metrics collection and analysis

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time;
use crate::tracker::{MemoryBlockTracker, FragmentationStats};
use crate::InfraResult;

/// Collects and analyzes fragmentation metrics
#[derive(Debug)]
pub struct FragmentationMetricsCollector {
    /// Memory block tracker reference
    tracker: Arc<MemoryBlockTracker>,

    /// Historical metrics data
    historical_metrics: Arc<RwLock<Vec<MetricsSnapshot>>>,

    /// Current metrics snapshot
    current_metrics: Arc<RwLock<MetricsSnapshot>>,

    /// Collection interval
    collection_interval: Duration,

    /// Maximum historical data retention
    max_retention: Duration,
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: Instant,

    /// Fragmentation statistics
    pub stats: FragmentationStats,

    /// Memory pressure ratio (0.0-1.0)
    pub memory_pressure: f64,

    /// Defragmentation efficiency (0.0-1.0)
    pub defragmentation_efficiency: f64,

    /// Average fragmentation rate over time
    pub fragmentation_rate: f64,

    /// Memory utilization trend
    pub utilization_trend: UtilizationTrend,
}

#[derive(Debug, Clone, Copy)]
pub enum UtilizationTrend {
    /// Memory utilization is increasing
    Increasing,
    /// Memory utilization is decreasing
    Decreasing,
    /// Memory utilization is stable
    Stable,
    /// Insufficient data to determine trend
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DefragmentationMetrics {
    /// Total memory defragmented in last cycle
    pub memory_defragmented: usize,

    /// Number of blocks relocated
    pub blocks_relocated: usize,

    /// Defragmentation duration
    pub duration: Duration,

    /// Fragmentation ratio before defragmentation
    pub fragmentation_before: f64,

    /// Fragmentation ratio after defragmentation
    pub fragmentation_after: f64,

    /// Memory pressure before defragmentation
    pub memory_pressure_before: f64,

    /// Memory pressure after defragmentation
    pub memory_pressure_after: f64,
}

impl FragmentationMetricsCollector {
    /// Create a new metrics collector
    pub fn new(tracker: Arc<MemoryBlockTracker>) -> Self {
        Self {
            tracker,
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
            current_metrics: Arc::new(RwLock::new(MetricsSnapshot::default())),
            collection_interval: Duration::from_secs(30), // 30 seconds
            max_retention: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Start the metrics collection task
    pub async fn start_collection(self: Arc<Self>) -> InfraResult<()> {
        let this = Arc::clone(&self);

        tokio::spawn(async move {
            let mut interval = time::interval(this.collection_interval);

            loop {
                interval.tick().await;
                if let Err(e) = this.collect_metrics().await {
                    tracing::error!("Failed to collect fragmentation metrics: {:?}", e);
                }
            }
        });

        Ok(())
    }

    /// Collect current metrics
    pub async fn collect_metrics(&self) -> InfraResult<()> {
        let stats = self.tracker.get_fragmentation_stats().await;
        let memory_pressure = self.calculate_memory_pressure(&stats).await;
        let defragmentation_efficiency = self.calculate_defragmentation_efficiency().await;
        let fragmentation_rate = self.calculate_fragmentation_rate().await;
        let utilization_trend = self.calculate_utilization_trend().await;

        let snapshot = MetricsSnapshot {
            timestamp: Instant::now(),
            stats,
            memory_pressure,
            defragmentation_efficiency,
            fragmentation_rate,
            utilization_trend,
        };

        // Update current metrics
        *self.current_metrics.write().await = snapshot.clone();

        // Add to historical data
        self.add_historical_snapshot(snapshot).await;

        // Cleanup old data
        self.cleanup_old_data().await;

        Ok(())
    }

    /// Get current metrics snapshot
    pub async fn get_current_metrics(&self) -> MetricsSnapshot {
        self.current_metrics.read().await.clone()
    }

    /// Get historical metrics
    pub async fn get_historical_metrics(&self, since: Instant) -> Vec<MetricsSnapshot> {
        let historical = self.historical_metrics.read().await;
        historical
            .iter()
            .filter(|snapshot| snapshot.timestamp >= since)
            .cloned()
            .collect()
    }

    /// Record defragmentation operation metrics
    pub async fn record_defragmentation(&self, metrics: DefragmentationMetrics) {
        let mut current = self.current_metrics.write().await;

        // Update efficiency based on recent defragmentation
        let efficiency_improvement = metrics.fragmentation_before - metrics.fragmentation_after;
        current.defragmentation_efficiency = efficiency_improvement.max(0.0).min(1.0);

        tracing::info!(
            "Defragmentation completed: {:.2}MB defragmented, {} blocks relocated, {:.1}% fragmentation reduction in {:?}",
            metrics.memory_defragmented as f64 / (1024.0 * 1024.0),
            metrics.blocks_relocated,
            efficiency_improvement * 100.0,
            metrics.duration
        );
    }

    /// Check if defragmentation is recommended
    pub async fn should_defragment(&self, threshold: f64) -> bool {
        let current = self.current_metrics.read().await;
        current.stats.fragmentation_ratio >= threshold
    }

    /// Get fragmentation trend analysis
    pub async fn get_fragmentation_trend(&self) -> FragmentationTrend {
        let historical = self.historical_metrics.read().await;

        if historical.len() < 2 {
            return FragmentationTrend::Unknown;
        }

        let mut increasing_count = 0;
        let mut decreasing_count = 0;

        for window in historical.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            if curr.stats.fragmentation_ratio > prev.stats.fragmentation_ratio {
                increasing_count += 1;
            } else if curr.stats.fragmentation_ratio < prev.stats.fragmentation_ratio {
                decreasing_count += 1;
            }
        }

        let total_comparisons = historical.len() - 1;
        let increasing_ratio = increasing_count as f64 / total_comparisons as f64;
        let decreasing_ratio = decreasing_count as f64 / total_comparisons as f64;

        if increasing_ratio > 0.6 {
            FragmentationTrend::Increasing
        } else if decreasing_ratio > 0.6 {
            FragmentationTrend::Decreasing
        } else {
            FragmentationTrend::Stable
        }
    }

    /// Calculate memory pressure ratio
    async fn calculate_memory_pressure(&self, stats: &FragmentationStats) -> f64 {
        // Memory pressure is based on fragmentation and utilization
        let fragmentation_pressure = stats.fragmentation_ratio;
        let utilization_pressure = 1.0 - stats.utilization_ratio();

        // Combine pressures with weights
        0.7 * fragmentation_pressure + 0.3 * utilization_pressure
    }

    /// Calculate defragmentation efficiency
    async fn calculate_defragmentation_efficiency(&self) -> f64 {
        let historical = self.historical_metrics.read().await;

        if historical.len() < 2 {
            return 0.0;
        }

        let recent = &historical[historical.len() - 1];
        let previous = &historical[historical.len() - 2];

        let fragmentation_improvement = previous.stats.fragmentation_ratio - recent.stats.fragmentation_ratio;

        fragmentation_improvement.max(0.0).min(1.0)
    }

    /// Calculate fragmentation rate over time
    async fn calculate_fragmentation_rate(&self) -> f64 {
        let historical = self.historical_metrics.read().await;

        if historical.len() < 2 {
            return 0.0;
        }

        let first = &historical[0];
        let last = &historical[historical.len() - 1];

        let time_diff = last.timestamp.duration_since(first.timestamp).as_secs_f64();
        if time_diff == 0.0 {
            return 0.0;
        }

        let fragmentation_diff = last.stats.fragmentation_ratio - first.stats.fragmentation_ratio;
        fragmentation_diff / time_diff
    }

    /// Calculate utilization trend
    async fn calculate_utilization_trend(&self) -> UtilizationTrend {
        let historical = self.historical_metrics.read().await;

        if historical.len() < 3 {
            return UtilizationTrend::Unknown;
        }

        let recent = historical.iter().rev().take(3).collect::<Vec<_>>();
        let mut increasing = 0;
        let mut decreasing = 0;

        for window in recent.windows(2) {
            let prev = window[0].stats.utilization_ratio();
            let curr = window[1].stats.utilization_ratio();

            if curr > prev {
                increasing += 1;
            } else if curr < prev {
                decreasing += 1;
            }
        }

        if increasing > decreasing {
            UtilizationTrend::Increasing
        } else if decreasing > increasing {
            UtilizationTrend::Decreasing
        } else {
            UtilizationTrend::Stable
        }
    }

    /// Add snapshot to historical data
    async fn add_historical_snapshot(&self, snapshot: MetricsSnapshot) {
        let mut historical = self.historical_metrics.write().await;
        historical.push(snapshot);
    }

    /// Cleanup old historical data
    async fn cleanup_old_data(&self) {
        let mut historical = self.historical_metrics.write().await;
        let cutoff = Instant::now() - self.max_retention;

        historical.retain(|snapshot| snapshot.timestamp >= cutoff);
    }

    /// Export metrics for monitoring
    pub async fn export_metrics(&self) -> serde_json::Value {
        let current = self.get_current_metrics().await;
        let historical_count = self.historical_metrics.read().await.len();

        serde_json::json!({
            "current": {
                "timestamp": current.timestamp.elapsed().as_secs(),
                "fragmentation_ratio": current.stats.fragmentation_ratio,
                "memory_pressure": current.memory_pressure,
                "total_memory": current.stats.total_memory,
                "utilization_ratio": current.stats.utilization_ratio(),
                "trend": format!("{:?}", current.utilization_trend)
            },
            "historical_count": historical_count,
            "collection_interval_seconds": self.collection_interval.as_secs()
        })
    }
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            stats: FragmentationStats::default(),
            memory_pressure: 0.0,
            defragmentation_efficiency: 0.0,
            fragmentation_rate: 0.0,
            utilization_trend: UtilizationTrend::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FragmentationTrend {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}