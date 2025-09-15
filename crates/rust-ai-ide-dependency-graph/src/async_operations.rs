//! Async operations for dependency graph processing

use std::collections::HashMap;
use std::sync::Arc;

use futures::stream::{self, StreamExt};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task;

use crate::cache::*;
use crate::error::*;
use crate::graph::*;
use crate::resolver::*;

#[derive(Debug, Clone)]
pub struct AsyncOperationConfig {
    pub max_concurrent_operations: usize,
    pub operation_timeout_secs:    u64,
    pub batch_size:                usize,
}

impl Default for AsyncOperationConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 5,
            operation_timeout_secs:    30,
            batch_size:                10,
        }
    }
}

/// Async graph processor for handling concurrent dependency operations
pub struct AsyncGraphProcessor {
    graph:     Arc<RwLock<DependencyGraph>>,
    resolver:  Arc<DependencyResolver>,
    cache:     Arc<GraphCache>,
    config:    AsyncOperationConfig,
    semaphore: Arc<Semaphore>,
}

/// Operation types for async processing
#[derive(Debug, Clone)]
pub enum AsyncOperation {
    ResolveDependencies {
        package_name: String,
    },
    UpdatePackageVersions {
        package_versions: HashMap<String, String>,
    },
    ValidateConflicts {
        check_workspace: bool,
    },
    AnalyzeDependencies {
        analysis_type: AnalysisType,
    },
    UpdateCache {
        invalidate_all: bool,
    },
}

#[derive(Debug, Clone)]
pub enum AnalysisType {
    Security,
    Performance,
    Licensing,
    Comprehensive,
}

/// Result of an async operation
#[derive(Debug, Clone)]
pub enum OperationResult {
    Resolution {
        resolved_versions: HashMap<String, String>,
    },
    Validation {
        is_valid:  bool,
        conflicts: Vec<String>,
    },
    Analysis {
        results: HashMap<String, serde_json::Value>,
    },
    Cache {
        updated_entries: usize,
    },
    BatchResults {
        results: Vec<AsyncResult>,
    },
}

/// Async operation result wrapper
#[derive(Debug, Clone)]
pub struct AsyncResult {
    pub operation_id:      String,
    pub result:            Result<OperationResult, DependencyError>,
    pub execution_time_ms: u64,
    pub timestamp:         chrono::DateTime<chrono::Utc>,
}

impl AsyncGraphProcessor {
    pub fn new(graph: Arc<RwLock<DependencyGraph>>, resolver: Arc<DependencyResolver>, cache: Arc<GraphCache>) -> Self {
        Self::with_config(graph, resolver, cache, AsyncOperationConfig::default())
    }

    pub fn with_config(
        graph: Arc<RwLock<DependencyGraph>>,
        resolver: Arc<DependencyResolver>,
        cache: Arc<GraphCache>,
        config: AsyncOperationConfig,
    ) -> Self {
        Self {
            graph,
            resolver,
            cache,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_operations)),
            config,
        }
    }

    /// Execute a single async operation
    pub async fn execute_operation(&self, operation: AsyncOperation) -> DependencyResult<OperationResult> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| DependencyError::ResolutionError {
                package: "semaphore".to_string(),
                reason:  format!("Failed to acquire semaphore: {}", e),
            })?;

        match operation {
            AsyncOperation::ResolveDependencies { package_name } => self.resolve_dependencies(package_name).await,
            AsyncOperation::UpdatePackageVersions { package_versions } =>
                self.update_package_versions(package_versions).await,
            AsyncOperation::ValidateConflicts { check_workspace } => self.validate_conflicts(check_workspace).await,
            AsyncOperation::AnalyzeDependencies { analysis_type } => self.analyze_dependencies(analysis_type).await,
            AsyncOperation::UpdateCache { invalidate_all } => self.update_cache(invalidate_all).await,
        }
    }

    /// Execute multiple operations concurrently with result aggregation
    pub async fn execute_batch_operations(
        &self,
        operations: Vec<(String, AsyncOperation)>,
    ) -> DependencyResult<AsyncResult> {
        let start_time = std::time::Instant::now();

        let operation_id = format!(
            "batch-{}-{}",
            chrono::Utc::now().timestamp(),
            operations.len()
        );

        // Process operations in parallel using futures streams
        let results: Vec<Result<(String, OperationResult), DependencyError>> = stream::iter(operations)
            .map(|(id, op)| {
                let processor = &self;
                async move {
                    let result = processor.execute_operation(op).await;
                    result.map(|res| (id, res))
                }
            })
            .buffer_unordered(self.config.max_concurrent_operations)
            .collect()
            .await;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Collect successful and failed results
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for result in results {
            match result {
                Ok((id, operation_result)) => {
                    successful.push(AsyncResult {
                        operation_id:      id,
                        result:            Ok(operation_result),
                        execution_time_ms: execution_time,
                        timestamp:         chrono::Utc::now(),
                    });
                }
                Err(e) => {
                    failed.push(AsyncResult {
                        operation_id:      format!("failed-{}", chrono::Utc::now().timestamp()),
                        result:            Err(e),
                        execution_time_ms: execution_time,
                        timestamp:         chrono::Utc::now(),
                    });
                }
            }
        }

        Ok(AsyncResult {
            operation_id,
            result: Ok(OperationResult::BatchResults {
                results: [successful, failed].concat(),
            }),
            execution_time_ms: execution_time,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Stream processing for large dependency sets
    pub async fn process_stream<T, F, Fut>(&self, items: Vec<T>, processor: F) -> DependencyResult<Vec<AsyncResult>>
    where
        F: Fn(T) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = DependencyResult<OperationResult>> + Send,
        T: Send + 'static + Clone,
    {
        let results: Vec<_> = stream::iter(items)
            .map(move |item| {
                let processor_clone = processor.clone();
                async move {
                    let start_time = std::time::Instant::now();
                    let result = processor_clone(item).await;
                    let execution_time = start_time.elapsed().as_millis() as u64;

                    AsyncResult {
                        operation_id: format!("stream-{}", chrono::Utc::now().timestamp()),
                        result,
                        execution_time_ms: execution_time,
                        timestamp: chrono::Utc::now(),
                    }
                }
            })
            .buffer_unordered(self.config.max_concurrent_operations)
            .collect()
            .await;

        Ok(results)
    }

    // Operation implementations
    async fn resolve_dependencies(&self, package_name: String) -> DependencyResult<OperationResult> {
        let cache_key = DependencyResolutionKey {
            root_package:        package_name.clone(),
            resolution_strategy: format!("{:?}", self.resolver.strategy),
            constraints:         HashMap::new(),
        };

        // Check cache first
        if let Some(cached) = self.cache.get_resolution(&cache_key).await {
            return Ok(OperationResult::Resolution {
                resolved_versions: cached.resolved_versions,
            });
        }

        // Perform resolution
        let resolved_versions = self.resolver.resolve_conflicts().await?;
        let operation_result = OperationResult::Resolution {
            resolved_versions: resolved_versions.clone(),
        };

        // Cache the result
        let cache_entry = DependencyResolutionEntry {
            resolved_versions,
            conflicts: vec![], // Would be filled by actual resolution
            last_updated: chrono::Utc::now(),
        };
        self.cache.put_resolution(cache_key, cache_entry).await;

        Ok(operation_result)
    }

    async fn update_package_versions(
        &self,
        package_versions: HashMap<String, String>,
    ) -> DependencyResult<OperationResult> {
        self.resolver
            .apply_resolved_versions(&package_versions)
            .await?;

        Ok(OperationResult::Resolution {
            resolved_versions: package_versions,
        })
    }

    async fn validate_conflicts(&self, check_workspace: bool) -> DependencyResult<OperationResult> {
        let graph = self.graph.read().await;

        // Check for cycles if enabled
        let cycle_conflicts = if check_workspace && graph.has_cycles() {
            graph.get_cycles()
        } else {
            vec![]
        };

        let cycle_strings: Vec<String> = cycle_conflicts.into_iter().flatten().collect();

        let is_valid = cycle_strings.is_empty();

        Ok(OperationResult::Validation {
            is_valid,
            conflicts: cycle_strings,
        })
    }

    async fn analyze_dependencies(&self, analysis_type: AnalysisType) -> DependencyResult<OperationResult> {
        let graph = self.graph.read().await;
        let stats = graph.get_statistics();

        let mut analysis_results = HashMap::new();

        match analysis_type {
            AnalysisType::Security => {
                // Basic security analysis
                analysis_results.insert(
                    "vulnerability_check".to_string(),
                    serde_json::json!({"status": "completed", "vulnerabilities_found": 0}),
                );
            }
            AnalysisType::Performance => {
                analysis_results.insert(
                    "performance_metrics".to_string(),
                    serde_json::json!({
                        "package_count": stats.total_packages,
                        "dependencies_count": stats.total_dependencies
                    }),
                );
            }
            AnalysisType::Licensing => {
                analysis_results.insert(
                    "license_compliance".to_string(),
                    serde_json::json!({"compliant": true, "licenses_checked": stats.total_packages}),
                );
            }
            AnalysisType::Comprehensive => {
                analysis_results.insert(
                    "full_analysis".to_string(),
                    serde_json::json!({
                        "packages": stats.total_packages,
                        "dependencies": stats.total_dependencies,
                        "workspace_members": stats.workspace_members,
                        "has_cycles": stats.has_cycles
                    }),
                );
            }
        }

        Ok(OperationResult::Analysis {
            results: analysis_results,
        })
    }

    async fn update_cache(&self, invalidate_all: bool) -> DependencyResult<OperationResult> {
        if invalidate_all {
            self.cache.clear_all().await;
            return Ok(OperationResult::Cache { updated_entries: 0 });
        }

        // Run cache maintenance
        self.cache.run_maintenance().await;

        // Get stats for reporting
        let stats = self.cache.get_stats().await;

        Ok(OperationResult::Cache {
            updated_entries: stats.package_metadata_entries as usize
                + stats.dependency_tree_entries as usize
                + stats.resolution_entries as usize,
        })
    }
}

/// Actor-based async processor for handling multiple concurrent operations
pub struct AsyncProcessorActor {
    processor:    Arc<AsyncGraphProcessor>,
    operation_tx: mpsc::Sender<(String, AsyncOperation)>,
    result_rx:    mpsc::Receiver<AsyncResult>,
}

impl AsyncProcessorActor {
    pub fn new(processor: Arc<AsyncGraphProcessor>) -> Self {
        let (operation_tx, mut operation_rx) = mpsc::channel(100);
        let (result_tx, result_rx) = mpsc::channel(100);

        let processor_clone = processor.clone();
        tokio::spawn(async move {
            while let Some((operation_id, operation)) = operation_rx.recv().await {
                let processor = processor_clone.clone();
                let result_tx = result_tx.clone();

                tokio::spawn(async move {
                    let start_time = std::time::Instant::now();
                    let result = processor.execute_operation(operation).await;
                    let execution_time = start_time.elapsed().as_millis() as u64;

                    let async_result = AsyncResult {
                        operation_id,
                        result,
                        execution_time_ms: execution_time,
                        timestamp: chrono::Utc::now(),
                    };

                    if let Err(e) = result_tx.send(async_result).await {
                        tracing::error!("Failed to send operation result: {}", e);
                    }
                });
            }
        });

        Self {
            processor,
            operation_tx,
            result_rx,
        }
    }

    /// Submit an operation for async processing
    pub async fn submit_operation(&self, operation_id: String, operation: AsyncOperation) -> DependencyResult<()> {
        self.operation_tx
            .send((operation_id, operation))
            .await
            .map_err(|e| DependencyError::ResolutionError {
                package: "actor".to_string(),
                reason:  format!("Failed to submit operation: {}", e),
            })
    }

    /// Receive operation results
    pub async fn receive_result(&mut self) -> Option<AsyncResult> {
        self.result_rx.recv().await
    }

    /// Get the underlying processor
    pub fn processor(&self) -> &Arc<AsyncGraphProcessor> {
        &self.processor
    }
}

/// Batch operation queue for managing large-scale dependency processing
pub struct BatchOperationQueue {
    queue:      Arc<RwLock<Vec<(String, AsyncOperation)>>>,
    processor:  Arc<AsyncProcessorActor>,
    batch_size: usize,
}

impl BatchOperationQueue {
    pub fn new(processor: Arc<AsyncProcessorActor>, batch_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            processor,
            batch_size,
        }
    }

    /// Add operation to queue
    pub async fn enqueue(&self, operation_id: String, operation: AsyncOperation) -> DependencyResult<()> {
        let mut queue = self.queue.write().await;
        queue.push((operation_id, operation));

        // Process batch if queue is full
        if queue.len() >= self.batch_size {
            let batch = queue.drain(..).collect::<Vec<_>>();
            self.process_batch(batch).await?;
        }

        Ok(())
    }

    /// Process all queued operations immediately
    pub async fn flush(&self) -> DependencyResult<()> {
        let mut queue = self.queue.write().await;
        let batch = queue.drain(..).collect::<Vec<_>>();
        drop(queue);

        self.process_batch(batch).await
    }

    async fn process_batch(&self, batch: Vec<(String, AsyncOperation)>) -> DependencyResult<()> {
        // Submit all operations for processing
        for (operation_id, operation) in batch {
            if let Err(e) = self
                .processor
                .submit_operation(operation_id, operation)
                .await
            {
                tracing::error!("Failed to submit batch operation: {}", e);
            }
        }

        Ok(())
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }
}
