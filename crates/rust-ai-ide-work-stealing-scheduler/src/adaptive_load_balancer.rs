/*!
 * Adaptive Load Balancer for optimal CPU utilization
 *
 * This module provides intelligent load balancing with real-time CPU monitoring,
 * predictive scheduling, and adaptive work distribution for maximum throughput.
 */

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// CPU utilization metrics per worker
#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub worker_id: usize,
    pub utilization_percent: f64,
    pub tasks_per_second: f64,
    pub queue_depth: usize,
    pub last_updated: std::time::Instant,
}

/// Adaptive load balancer with predictive scheduling
pub struct AdaptiveLoadBalancer {
    /// Worker CPU metrics
    worker_metrics: Arc<RwLock<HashMap<usize, CpuMetrics>>>,
    /// Target CPU utilization (0.0-1.0)
    target_utilization: f64,
    /// Load balancing interval
    balance_interval: Duration,
    /// Performance history for prediction
    perf_history: Arc<RwLock<Vec<PerformanceSnapshot>>>,
    /// Current balancing strategy
    strategy: Arc<RwLock<BalancingStrategy>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: std::time::Instant,
    pub total_throughput: f64,
    pub avg_utilization: f64,
    pub load_imbalance: f64,
}

#[derive(Debug, Clone)]
pub enum BalancingStrategy {
    RoundRobin,
    LeastLoaded,
    Predictive,
    Adaptive,
}

/// Work distribution recommendation
#[derive(Debug)]
pub struct WorkDistribution {
    pub target_worker: usize,
    pub priority_boost: i32,
    pub predicted_completion: Duration,
    pub confidence_score: f64,
}

impl AdaptiveLoadBalancer {
    pub fn new(num_workers: usize, target_utilization: f64) -> Self {
        let mut worker_metrics = HashMap::new();

        // Initialize metrics for all workers
        for i in 0..num_workers {
            worker_metrics.insert(i, CpuMetrics {
                worker_id: i,
                utilization_percent: 0.0,
                tasks_per_second: 0.0,
                queue_depth: 0,
                last_updated: std::time::Instant::now(),
            });
        }

        Self {
            worker_metrics: Arc::new(RwLock::new(worker_metrics)),
            target_utilization,
            balance_interval: Duration::from_millis(50), // 20Hz balancing
            perf_history: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            strategy: Arc::new(RwLock::new(BalancingStrategy::Adaptive)),
        }
    }

    /// Update CPU metrics for a worker
    pub async fn update_metrics(&self, worker_id: usize, metrics: CpuMetrics) {
        let mut worker_metrics = self.worker_metrics.write().await;
        worker_metrics.insert(worker_id, metrics);
    }

    /// Get optimal worker for new task based on current load
    pub async fn get_optimal_worker(&self, task_complexity: f64) -> WorkDistribution {
        let strategy = self.strategy.read().await.clone();
        let worker_metrics = self.worker_metrics.read().await;

        match strategy {
            BalancingStrategy::RoundRobin => {
                // Simple round-robin for baseline
                static NEXT_WORKER: AtomicUsize = AtomicUsize::new(0);
                let worker_id = NEXT_WORKER.fetch_add(1, Ordering::Relaxed) % worker_metrics.len();

                WorkDistribution {
                    target_worker: worker_id,
                    priority_boost: 0,
                    predicted_completion: Duration::from_millis(100),
                    confidence_score: 0.5,
                }
            }
            BalancingStrategy::LeastLoaded => {
                self.select_least_loaded_worker(&worker_metrics, task_complexity).await
            }
            BalancingStrategy::Predictive => {
                self.predictive_worker_selection(&worker_metrics, task_complexity).await
            }
            BalancingStrategy::Adaptive => {
                self.adaptive_worker_selection(&worker_metrics, task_complexity).await
            }
        }
    }

    /// Select least loaded worker
    async fn select_least_loaded_worker(
        &self,
        metrics: &HashMap<usize, CpuMetrics>,
        _complexity: f64,
    ) -> WorkDistribution {
        let mut best_worker = 0;
        let mut best_score = f64::INFINITY;

        for (worker_id, metric) in metrics {
            // Score based on utilization and queue depth
            let score = metric.utilization_percent + (metric.queue_depth as f64 * 0.1);
            if score < best_score {
                best_score = score;
                best_worker = *worker_id;
            }
        }

        WorkDistribution {
            target_worker: best_worker,
            priority_boost: 0,
            predicted_completion: Duration::from_millis((best_score * 10.0) as u64),
            confidence_score: 0.7,
        }
    }

    /// Predictive worker selection using performance history
    async fn predictive_worker_selection(
        &self,
        metrics: &HashMap<usize, CpuMetrics>,
        complexity: f64,
    ) -> WorkDistribution {
        let history = self.perf_history.read().await;

        if history.len() < 10 {
            // Not enough history, fall back to least loaded
            return self.select_least_loaded_worker(metrics, complexity).await;
        }

        // Simple linear regression for throughput prediction
        let mut best_worker = 0;
        let mut best_predicted_throughput = 0.0;

        for (worker_id, metric) in metrics {
            let predicted_throughput = self.predict_worker_throughput(*worker_id, complexity, &history).await;
            if predicted_throughput > best_predicted_throughput {
                best_predicted_throughput = predicted_throughput;
                best_worker = *worker_id;
            }
        }

        WorkDistribution {
            target_worker: best_worker,
            priority_boost: 0,
            predicted_completion: Duration::from_millis((1000.0 / best_predicted_throughput.max(0.1)) as u64),
            confidence_score: 0.8,
        }
    }

    /// Adaptive worker selection with real-time load balancing
    async fn adaptive_worker_selection(
        &self,
        metrics: &HashMap<usize, CpuMetrics>,
        complexity: f64,
    ) -> WorkDistribution {
        let current_avg_utilization = metrics.values()
            .map(|m| m.utilization_percent)
            .sum::<f64>() / metrics.len() as f64;

        // Adjust strategy based on current utilization
        let mut strategy = self.strategy.write().await;
        if current_avg_utilization < self.target_utilization * 0.7 {
            // Underutilized - use predictive for better distribution
            *strategy = BalancingStrategy::Predictive;
            return self.predictive_worker_selection(metrics, complexity).await;
        } else if current_avg_utilization > self.target_utilization * 1.3 {
            // Overutilized - use least loaded for immediate relief
            *strategy = BalancingStrategy::LeastLoaded;
            return self.select_least_loaded_worker(metrics, complexity).await;
        } else {
            // Balanced - use predictive
            *strategy = BalancingStrategy::Predictive;
            return self.predictive_worker_selection(metrics, complexity).await;
        }
    }

    /// Predict throughput for a worker based on complexity and history
    async fn predict_worker_throughput(
        &self,
        worker_id: usize,
        complexity: f64,
        history: &[PerformanceSnapshot],
    ) -> f64 {
        // Simple prediction based on recent performance
        let recent_snapshots: Vec<_> = history.iter()
            .rev()
            .take(5)
            .collect();

        if recent_snapshots.is_empty() {
            return 10.0; // Default throughput
        }

        let avg_throughput = recent_snapshots.iter()
            .map(|s| s.total_throughput)
            .sum::<f64>() / recent_snapshots.len() as f64;

        // Adjust for complexity (simplified model)
        avg_throughput / (1.0 + complexity.log10().max(0.0))
    }

    /// Record performance snapshot
    pub async fn record_performance_snapshot(&self, snapshot: PerformanceSnapshot) {
        let mut history = self.perf_history.write().await;
        history.push(snapshot);

        // Keep only recent history
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get current load imbalance metric
    pub async fn calculate_load_imbalance(&self) -> f64 {
        let metrics = self.worker_metrics.read().await;
        let utilizations: Vec<f64> = metrics.values()
            .map(|m| m.utilization_percent)
            .collect();

        if utilizations.is_empty() {
            return 0.0;
        }

        let avg = utilizations.iter().sum::<f64>() / utilizations.len() as f64;
        let variance = utilizations.iter()
            .map(|u| (u - avg).powi(2))
            .sum::<f64>() / utilizations.len() as f64;

        variance.sqrt() / avg.max(0.01) // Coefficient of variation
    }

    /// Start background load balancing task
    pub async fn start_load_balancing_task(self: Arc<Self>) {
        let balance_interval = self.balance_interval;
        tokio::spawn(async move {
            let mut interval = interval(balance_interval);
            loop {
                interval.tick().await;

                // Calculate current imbalance
                let imbalance = self.calculate_load_imbalance().await;

                // Record performance snapshot
                let metrics = self.worker_metrics.read().await;
                let total_throughput = metrics.values()
                    .map(|m| m.tasks_per_second)
                    .sum::<f64>();
                let avg_utilization = metrics.values()
                    .map(|m| m.utilization_percent)
                    .sum::<f64>() / metrics.len() as f64;

                let snapshot = PerformanceSnapshot {
                    timestamp: std::time::Instant::now(),
                    total_throughput,
                    avg_utilization,
                    load_imbalance: imbalance,
                };

                self.record_performance_snapshot(snapshot).await;
            }
        });
    }
}

impl Default for AdaptiveLoadBalancer {
    fn default() -> Self {
        Self::new(num_cpus::get(), 0.8) // 80% target utilization
    }
}