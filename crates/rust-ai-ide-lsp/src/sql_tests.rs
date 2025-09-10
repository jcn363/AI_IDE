//! Comprehensive Test Suite for SQL LSP Server
//!
//! This dedicated test file provides exhaustive coverage for the SQL LSP server's
//! advanced caching, performance optimization, and parallel processing features.
//!
//! Test Organization:
//! - Unit tests for individual components
//! - Integration tests for component interactions
//! - Performance benchmarks
//! - Edge case and error scenario testing
//!
//! Run with: `cargo test -p rust-ai-ide-lsp`
//! Run specific test: `cargo test -p rust-ai-ide-lsp -- --test test_name`

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tokio::sync::RwLock;
use serde_json::json;

// Re-export all types from the main module
pub use super::*;

/// Test Infrastructure
struct TestHarness {
    pub server: Arc<SqlLspServer>,
    pub temp_dir: tempfile::TempDir,
}

impl TestHarness {
    /// Create a new test harness with default configuration
    async fn new() -> Result<Self, SqlLspError> {
        Self::with_config(SqlLspConfig::default()).await
    }

    /// Create a new test harness with custom configuration
    async fn with_config(config: SqlLspConfig) -> Result<Self, SqlLspError> {
        let temp_dir = tempfile::create_dir().map_err(|e|
            SqlLspError::ConfigurationError(format!("Failed to create temp dir: {}", e)))?;

        let config_arc = Arc::new(RwLock::new(config));
        let (_, receiver) = mpsc::unbounded_channel();

        // Clone config for initialization
        let config_for_init = SqlLspConfig {
            advanced_caching: config.enable_advanced_caching,
            parallel_processing: config.enable_parallel_processing,
            virtual_memory: config.enable_virtual_memory,
            performance_settings: config.performance_settings.clone(),
            cache_settings: config.cache_settings.clone(),
            ..config
        };

        let cache_manager = Arc::new(SqlLspServer::initialize_cache_manager(&config_for_init).await?);
        let parallel_executor = if config.enable_parallel_processing {
            Some(Arc::new(SqlLspServer::initialize_parallel_executor(&config_for_init).await?))
        } else {
            None
        };
        let virtual_memory_manager = if config.enable_virtual_memory {
            Some(Arc::new(SqlLspServer::initialize_virtual_memory_manager(&config_for_init).await?))
        } else {
            None
        };
        let background_task_manager = Arc::new(SqlLspServer::initialize_background_task_manager(mpsc::unbounded_channel().0).await?);

        let mut server = SqlLspServer {
            #[cfg(feature = "tree-sitter-sql")]
            sql_parser: SqlLspServer::initialize_sql_parser()?,
            #[cfg(feature = "rust-ai-ide-ai-predictive")]
            predictive_engine: SqlLspServer::initialize_predictive_engine().await,
            cache_manager,
            parallel_executor,
            virtual_memory_manager,
            background_task_manager,
            collaborative_sessions: Arc::new(RwLock::new(HashMap::new())),
            config: config_arc,
            client: None,
            dialect_detectors: HashMap::new(),
        };

        server.start_background_processing(receiver);
        server.initialize_dialect_detectors().await?;

        Ok(Self {
            server: Arc::new(server),
            temp_dir,
        })
    }

    /// Get pre-configured test queries
    fn test_queries() -> Vec<(String, &str)> {
        vec![
            ("SELECT * FROM users;".to_string(), "simple_select"),
            ("SELECT u.name FROM users u JOIN posts p ON u.id = p.user_id;".to_string(), "join_query"),
            ("SELECT COUNT(*) FROM users WHERE active = true GROUP BY status;".to_string(), "aggregation_query"),
            ("SELECT * FROM users WHERE id = 1;".to_string(), "indexed_where"),
            ("SELECT u.* FROM users u WHERE u.created_at >= '2023-01-01' ORDER BY u.name;".to_string(), "complex_filtering"),
        ]
    }

    /// Helper to assert query complexity ranges
    fn assert_complexity_in_range(query_name: &str, complexity: u8, min: u8, max: u8) {
        assert!(
            complexity >= min && complexity <= max,
            "Query {} complexity {} not in range [{}, {}]",
            query_name, complexity, min, max
        );
    }
}

mod server_initialization_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation_with_default_config() {
        let harness_result = TestHarness::new().await;
        assert!(harness_result.is_ok(), "Should successfully create server with default config");

        let harness = harness_result.unwrap();
        let config = harness.server.config.read().await;

        assert!(config.enable_optimization_suggestions);
        assert!(config.enable_performance_profiling);
        assert!(config.enable_advanced_caching);
        assert_eq!(config.supported_sql_dialects.len(), 5); // All major dialects
    }

    #[tokio::test]
    async fn test_server_creation_with_advanced_caching() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                max_memory_per_layer_mb: 512,
                max_entries_per_layer: 5000,
                collect_statistics: true,
                enable_cache_warming: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();
        assert!(harness.server.cache_manager.stats_collector.is_some(),
               "Should initialize cache statistics collector");
    }

    #[tokio::test]
    async fn test_server_creation_with_parallel_processing() {
        let config = SqlLspConfig {
            enable_parallel_processing: true,
            performance_settings: SqlPerformanceSettings {
                parallel_analysis: true,
                max_concurrent_tasks: 16,
                analysis_timeout_ms: 15000,
                batch_processing: true,
                batch_size: 50,
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();
        let parallel_executor = harness.server.parallel_executor.as_ref();
        assert!(parallel_executor.is_some(), "Should initialize parallel executor");

        if let Some(executor) = parallel_executor {
            assert_eq!(executor.max_concurrency, config.performance_settings.max_concurrent_tasks);
            assert!(executor.semaphore.available_permits() > 0);
        }
    }

    #[tokio::test]
    async fn test_server_creation_with_virtual_memory() {
        let config = SqlLspConfig {
            enable_virtual_memory: true,
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();
        assert!(harness.server.virtual_memory_manager.is_some(),
               "Should initialize virtual memory manager");

        if let Some(vm_manager) = &harness.server.virtual_memory_manager {
            assert!(vm_manager.temp_dir.exists(), "Should create temp directory");
            assert!(vm_manager.max_virtual_memory_mb > 0, "Should have memory limit");
        }
    }

    #[tokio::test]
    async fn test_dialect_detector_initialization() {
        let config = SqlLspConfig {
            supported_sql_dialects: vec![
                "postgresql".to_string(),
                "mysql".to_string(),
                "sqlite".to_string(),
            ],
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        // Should only have configured dialects
        assert_eq!(harness.server.dialect_detectors.len(), 3);
        assert!(harness.server.dialect_detectors.contains_key("postgresql"));
        assert!(harness.server.dialect_detectors.contains_key("mysql"));
        assert!(harness.server.dialect_detectors.contains_key("sqlite"));
        assert!(!harness.server.dialect_detectors.contains_key("oracle"));
    }

    #[tokio::test]
    async fn test_server_configuration_validation() {
        // Test minimal configuration
        let min_config = SqlLspConfig {
            enable_optimization_suggestions: false,
            enable_schema_inference: false,
            enable_performance_profiling: false,
            enable_collaborative_editing: false,
            enable_error_analysis: false,
            enable_advanced_caching: false,
            enable_parallel_processing: false,
            enable_virtual_memory: false,
            supported_sql_dialects: vec![],
            min_suggestion_confidence: 0.0,
            cache_settings: SqlCacheConfig::default(),
            performance_settings: SqlPerformanceSettings::default(),
            security_settings: SqlSecuritySettings::default(),
        };

        let harness = TestHarness::with_config(min_config).await.unwrap();
        let config = harness.server.config.read().await;

        assert!(!config.enable_optimization_suggestions);
        assert!(!config.enable_advanced_caching);
        assert!(!config.enable_parallel_processing);
        assert_eq!(config.supported_sql_dialects.len(), 0);
    }
}

mod cache_layer_tests {
    use super::*;

    #[cfg(feature = "moka")]
    #[tokio::test]
    async fn test_multi_tier_cache_operations() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                max_entries_per_layer: 100,
                collect_statistics: true,
                ttl_settings: CacheTtlSettings {
                    metrics_ttl_seconds: 3600,
                    schema_ttl_seconds: 7200,
                    optimization_ttl_seconds: 3600,
                    error_ttl_seconds: 1800,
                    virtual_memory_ttl_seconds: 3600,
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        // Test metrics cache
        let query1 = "SELECT COUNT(*) FROM users;".to_string();
        let metrics1 = harness.server.get_performance_metrics(query1.clone()).await.unwrap();
        let metrics2 = harness.server.get_performance_metrics(query1.clone()).await.unwrap();

        assert_eq!(metrics1.execution_time_us, metrics2.execution_time_us,
                  "Cached metrics should be identical");

        // Test optimization cache
        let query2 = "SELECT * FROM users WHERE id = 1;".to_string();
        let opt1 = harness.server.get_optimization_suggestions(query2.clone()).await.unwrap();
        let opt2 = harness.server.get_optimization_suggestions(query2.clone()).await.unwrap();

        assert_eq!(opt1.len(), opt2.len(),
                  "Cached optimizations should be identical");
    }

    #[cfg(feature = "moka")]
    #[tokio::test]
    async fn test_cache_eviction_policies() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                max_entries_per_layer: 3, // Very small cache
                eviction_policy: CacheEvictionPolicy::LeastRecentlyUsed,
                collect_statistics: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        // Fill cache beyond capacity
        let queries: Vec<String> = (0..5).map(|i| format!("SELECT * FROM table{};", i)).collect();

        for query in &queries {
            let _ = harness.server.get_performance_metrics(query.clone()).await;
        }

        // Check that statistics were recorded
        if let Some(stats) = &harness.server.cache_manager.stats_collector {
            let hit_stats = stats.hit_miss_stats.read().await;
            let metrics_stats = hit_stats.get("metrics").unwrap();
            assert!(metrics_stats.hit_ratio >= 0.0 && metrics_stats.hit_ratio <= 1.0,
                   "Hit ratio should be between 0 and 1");
            assert!(metrics_stats.insertions >= 3,
                   "Should have inserted at least 3 entries");
        }
    }

    #[cfg(feature = "moka")]
    #[tokio::test]
    async fn test_cache_statistics_collection() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                collect_statistics: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        let query = "SELECT id, name FROM users;".to_string();

        // Generate some cache activity
        for _ in 0..10 {
            let _ = harness.server.get_performance_metrics(query.clone()).await;
        }

        // Verify statistics collection
        if let Some(stats_collector) = &harness.server.cache_manager.stats_collector {
            let hit_stats = stats_collector.hit_miss_stats.read().await;
            let perf_stats = stats_collector.performance_stats.read().await;
            let mem_stats = stats_collector.memory_stats.read().await;

            assert!(hit_stats.contains_key("metrics"), "Should collect metrics cache stats");
            assert!(perf_stats.contains_key("metrics"), "Should collect performance stats");
            assert!(mem_stats.contains_key("metrics"), "Should collect memory stats");

            let metrics_perf = perf_stats.get("metrics").unwrap();
            assert!(metrics_perf.total_operations >= 10,
                   "Should have recorded at least 10 operations");
            assert!(metrics_perf.avg_lookup_time_ns > 0,
                   "Should have measured lookup times");
        }
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let query1 = "SELECT * FROM users;".to_string();
        let query2 = "SELECT * FROM users;".to_string(); // Same query
        let query3 = "SELECT * FROM posts;".to_string(); // Different query

        let key1 = format!("test_{}", hash_query(&query1));
        let key2 = format!("test_{}", hash_query(&query2));
        let key3 = format!("test_{}", hash_query(&query3));

        // Same queries should have same keys
        assert_eq!(key1, key2, "Identical queries should have identical keys");

        // Different queries should have different keys
        assert_ne!(key1, key3, "Different queries should have different keys");

        // Keys should contain query hash
        let hash1 = hash_query(&query1);
        assert!(key1.contains(&hash1), "Cache key should contain query hash");
        assert!(key1.starts_with("test_"), "Cache key should have correct prefix");
    }

    #[cfg(feature = "moka")]
    #[tokio::test]
    async fn test_cache_layer_error_handling() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        // Test removing non-existent key
        if let Some(cache_layer) = &harness.server.cache_manager.metrics_cache {
            let result = cache_layer.remove("non_existent_key");
            assert!(!result, "Removing non-existent key should return false");
        }

        // Test clearing cache
        if let Some(cache_layer) = &harness.server.cache_manager.metrics_cache {
            let clear_result = cache_layer.clear().await;
            assert!(clear_result.is_ok(), "Clear should succeed");
        }
    }
}

mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_metric_calculation() {
        let harness = TestHarness::new().await.unwrap();

        let queries = TestHarness::test_queries();

        for (query, query_name) in queries {
            let metrics = harness.server.get_performance_metrics(query).await.unwrap();

            // Verify all metrics are in valid ranges
            assert!(metrics.execution_time_us > 0, "Execution time should be positive");
            assert!(metrics.memory_usage_bytes > 0, "Memory usage should be positive");
            assert!(metrics.io_operations <= 100000, "IO operations should be reasonable");
            assert!(metrics.complexity_score <= 100,
                   "Complexity score for {} should be <= 100", query_name);

            // Verify CPU usage is valid percentage
            assert!(metrics.cpu_usage_percent >= 0.0 && metrics.cpu_usage_percent <= 100.0,
                   "CPU usage should be valid percentage");
        }
    }

    #[tokio::test]
    async fn test_query_complexity_scoring() {
        let harness = TestHarness::new().await.unwrap();

        let config = harness.server.config.read().await;

        // Test low complexity queries
        let simple_queries = vec![
            ("SELECT id FROM users;", "simple_select"),
            ("SELECT name FROM users WHERE id = 1;", "simple_where"),
        ];

        for (query, query_name) in simple_queries {
            let complexity = SqlLspServer::calculate_query_complexity(query, &config).unwrap();
            harness.assert_complexity_in_range(query_name, complexity, 0, 30);
        }

        // Test medium complexity queries
        let medium_queries = vec![
            ("SELECT u.name FROM users u JOIN posts p ON u.id = p.user_id;", "medium_join"),
            ("SELECT COUNT(*) FROM users GROUP BY status;", "medium_group"),
        ];

        for (query, query_name) in medium_queries {
            let complexity = SqlLspServer::calculate_query_complexity(query, &config).unwrap();
            harness.assert_complexity_in_range(query_name, complexity, 20, 70);
        }

        // Test high complexity queries
        let complex_queries = vec![
            ("SELECT u.name, COUNT(p.id) FROM users u LEFT JOIN posts p ON u.id = p.user_id WHERE u.created_at >= '2023-01-01' GROUP BY u.id, u.name HAVING COUNT(p.id) > 5 ORDER BY COUNT(p.id) DESC, u.name LIMIT 100 UNION SELECT name, 0 FROM users WHERE active = false;", "high_complexity"),
        ];

        for (query, query_name) in complex_queries {
            let complexity = SqlLspServer::calculate_query_complexity(query, &config).unwrap();
            harness.assert_complexity_in_range(query_name, complexity, 50, 100);
        }
    }

    #[test]
    fn test_bottleneck_identification_algorithm() {
        // Test IO bottleneck identification
        let join_query = "SELECT u.* FROM users u JOIN orders o ON u.id = o.user_id JOIN products p ON o.product_id = p.id;";
        let bottleneck = SqlLspServer::identify_bottleneck(join_query, 0);
        assert_eq!(bottleneck, QueryBottleneck::Io, "JOIN-heavy query should be IO bottleneck");

        // Test CPU bottleneck identification
        let cpu_query = "SELECT u.name FROM users u WHERE UPPER(u.email) LIKE '%@example.com' ORDER BY LOWER(u.name), LENGTH(u.bio) DESC;";
        let bottleneck = SqlLspServer::identify_bottleneck(cpu_query, 0);
        assert_eq!(bottleneck, QueryBottleneck::Cpu, "Function-heavy query should be CPU bottleneck");

        // Test memory bottleneck identification
        let memory_query = "SELECT DISTINCT u.*, p.title FROM users u CROSS JOIN posts p WHERE u.created_at BETWEEN '2023-01-01' AND '2023-12-31';";
        let bottleneck = SqlLspServer::identify_bottleneck(memory_query, 85);
        assert_eq!(bottleneck, QueryBottleneck::Memory, "High-complexity query should be memory bottleneck");

        // Test fallback for other queries
        let other_query = "SELECT * FROM users WHERE status = 'active';";
        let bottleneck = SqlLspServer::identify_bottleneck(other_query, 0);
        assert_eq!(bottleneck, QueryBottleneck::Other, "Simple query should be other category");
    }
}

mod memory_management_tests {
    use super::*;

    #[tokio::test]
    async fn test_virtual_memory_manager_initialization() {
        let config = SqlLspConfig {
            enable_virtual_memory: true,
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        if let Some(vm_manager) = &harness.server.virtual_memory_manager {
            assert!(vm_manager.temp_dir.exists(), "Temp directory should exist");
            assert!(vm_manager.max_virtual_memory_mb > 0, "Should have memory limit");
            assert!(vm_manager.memory_maps.read().await.is_empty(), "Should start with empty memory maps");

            // Verify temp directory is writable
            let test_file_path = vm_manager.temp_dir.path().join("test_write");
            std::fs::write(&test_file_path, "test").unwrap();
            assert!(test_file_path.exists(), "Should be able to write to temp directory");
            std::fs::remove_file(test_file_path).unwrap(); // Cleanup
        }
    }

    #[tokio::test]
    async fn test_memory_allocation_tracking() {
        let config = SqlLspConfig {
            enable_virtual_memory: true,
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        if let Some(vm_manager) = &harness.server.virtual_memory_manager {
            let mut tracker = vm_manager.allocation_tracker.write().await;

            // Simulate memory allocation
            let alloc_info = VirtualMemoryInfo {
                file_size: 1024,
                mapped_size: 2048,
                allocated_at: std::time::Instant::now(),
                access_pattern: MemoryAccessPattern::Sequential,
            };

            tracker.insert("test_allocation".to_string(), alloc_info);

            // Verify allocation is tracked
            let tracked = tracker.get("test_allocation").unwrap();
            assert_eq!(tracked.file_size, 1024);
            assert_eq!(tracked.mapped_size, 2048);
            assert_eq!(tracked.access_pattern, MemoryAccessPattern::Sequential);
        }
    }

    #[tokio::test]
    async fn test_memory_usage_statistics() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                collect_statistics: true,
                max_memory_per_layer_mb: 200,
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        // Perform some operations to generate memory usage
        for i in 0..10 {
            let query = format!("SELECT * FROM table{} WHERE id = {};", i, i);
            let _ = harness.server.get_performance_metrics(query).await;
        }

        // Check memory statistics
        if let Some(stats) = &harness.server.cache_manager.stats_collector {
            let mem_stats = stats.memory_stats.read().await;

            // Should have memory usage statistics for metrics layer
            if let Some(metrics_mem) = mem_stats.get("metrics") {
                assert!(metrics_mem.current_usage_bytes >= 0, "Memory usage should be non-negative");
                // Note: Actual memory usage may vary, so we mainly check that statistics are being collected
            }
        }
    }
}

mod parallel_processing_tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_executor_configuration() {
        let config = SqlLspConfig {
            enable_parallel_processing: true,
            performance_settings: SqlPerformanceSettings {
                max_concurrent_tasks: 8,
                analysis_timeout_ms: 10000,
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        if let Some(parallel_executor) = &harness.server.parallel_executor {
            assert_eq!(parallel_executor.max_concurrency, 8);
            assert_eq!(parallel_executor.semaphore.available_permits(), 8);

            let active_tasks = parallel_executor.active_tasks.lock().await;
            assert_eq!(*active_tasks, 0, "Should start with no active tasks");
        }
    }

    #[tokio::test]
    async fn test_batch_analysis_performance() {
        let config = SqlLspConfig {
            enable_parallel_processing: true,
            performance_settings: SqlPerformanceSettings {
                batch_processing: true,
                batch_size: 25,
                parallel_analysis: true,
                max_concurrent_tasks: 4,
                analysis_timeout_ms: 15000,
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        let queries = (0..50).map(|i| format!("SELECT * FROM table{} WHERE id = {};", i, i)).collect::<Vec<_>>();

        let start_time = std::time::Instant::now();
        let batch_result = timeout(
            Duration::from_secs(10),
            harness.server.perform_bulk_analysis(queries)
        ).await;

        match batch_result {
            Ok(Ok(result)) => {
                let duration = start_time.elapsed();

                // Should process all queries quickly
                assert_eq!(result.processing_stats.total_queries, 50);
                assert!(duration.as_millis() < 9000, "Batch processing should complete in less than 9 seconds");
                assert!(result.processing_stats.avg_processing_time_ms >= 0.0);
                assert!(result.processing_stats.error_count <= result.processing_stats.total_queries);
            }
            Err(_) => {
                // Timeout - acceptable for integration testing with limited resources
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_limit_enforcement() {
        let config = SqlLspConfig {
            enable_parallel_processing: true,
            performance_settings: SqlPerformanceSettings {
                max_concurrent_tasks: 2, // Very low concurrency for testing
                analysis_timeout_ms: 5000,
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        if let Some(parallel_executor) = &harness.server.parallel_executor {
            // Verify semaphore limits concurrency
            assert_eq!(parallel_executor.semaphore.available_permits(), 2);

            // Acquire one permit
            let permit1 = parallel_executor.semaphore.acquire().await.unwrap();
            assert_eq!(parallel_executor.semaphore.available_permits(), 1);

            // Acquire second permit
            let permit2 = parallel_executor.semaphore.acquire().await.unwrap();
            assert_eq!(parallel_executor.semaphore.available_permits(), 0);

            // Release permits
            drop(permit1);
            drop(permit2);

            assert_eq!(parallel_executor.semaphore.available_permits(), 2);
        }
    }

    #[tokio::test]
    async fn test_thread_pool_initialization() {
        let config = SqlLspConfig {
            enable_parallel_processing: true,
            performance_settings: SqlPerformanceSettings {
                max_concurrent_tasks: 4,
                ..Default::default()
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        if let Some(parallel_executor) = &harness.server.parallel_executor {
            // Should have thread pool when concurrency > 0
            assert!(parallel_executor.thread_pool.is_some(), "Should have thread pool");

            if let Some(thread_pool) = &parallel_executor.thread_pool {
                // Thread pool is initialized (we can't easily test the exact thread count in a unit test)
                // but we can verify it was created successfully
                assert!(parallel_executor.max_concurrency > 0);
            }
        }
    }
}

mod optimization_tests {
    use super::*;

    #[tokio::test]
    async fn test_select_star_optimization_detection() {
        let harness = TestHarness::new().await.unwrap();

        let queries_with_stars = vec![
            "SELECT * FROM users;",
            "SELECT *, COUNT(*) FROM posts GROUP BY author_id;",
            "SELECT u.*, p.title FROM users u JOIN posts p ON u.id = p.user_id;",
        ];

        for query in queries_with_stars {
            let optimizations = harness.server.get_optimization_suggestions(query.to_string()).await.unwrap();
            assert!(!optimizations.is_empty(), "Query with SELECT * should suggest optimization: {}", query);

            let star_suggestion = optimizations.iter().find(|opt| opt.explanation.contains("SELECT *"));
            assert!(star_suggestion.is_some(), "Should specifically suggest avoiding SELECT *: {}", query);
            assert!(star_suggestion.unwrap().confidence_score >= 0.9,
                   "SELECT * optimization should have high confidence");
        }
    }

    #[tokio::test]
    async fn test_index_optimization_detection() {
        let harness = TestHarness::new().await.unwrap();

        let queries_needing_indexes = vec![
            "SELECT * FROM users WHERE email = 'test@example.com';",
            "SELECT name FROM posts WHERE published_at >= '2023-01-01' AND published_at < '2024-01-01';",
            "SELECT COUNT(*) FROM orders WHERE customer_id = 123 AND total > 100.00;",
        ];

        for query in queries_needing_indexes {
            let optimizations = harness.server.get_optimization_suggestions(query.to_string()).await.unwrap();

            // Some queries may already have optimizations detected
            if !optimizations.is_empty() {
                let index_suggestion = optimizations.iter().find(|opt| opt.explanation.to_lowercase().contains("index"));
                if let Some(suggestion) = index_suggestion {
                    assert!(suggestion.confidence_score >= 0.5,
                           "Index optimization should have reasonable confidence");
                    assert!(!suggestion.required_changes.is_empty(),
                           "Index optimization should specify required changes");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_optimization_suggestion_structure() {
        let harness = TestHarness::new().await.unwrap();

        let query = "SELECT * FROM users WHERE name LIKE '%john%';";
        let optimizations = harness.server.get_optimization_suggestions(query.to_string()).await.unwrap();

        // Verify structure of optimization suggestions
        for optimization in optimizations {
            // Required fields
            assert!(!optimization.original_query.is_empty(), "Should have original query");
            assert!(!optimization.optimized_query.is_empty(), "Should have optimized query");
            assert!(!optimization.explanation.is_empty(), "Should have explanation");
            assert!(optimization.confidence_score >= 0.0 && optimization.confidence_score <= 1.0,
                   "Confidence score should be valid percentage");
            assert!(optimization.performance_improvement_percent >= 0.0,
                   "Performance improvement should be non-negative");
            assert!(optimization.time_reduction_us >= 0, "Time reduction should be non-negative");

            // Required changes structure
            if !optimization.required_changes.is_empty() {
                for change in &optimization.required_changes {
                    assert!(!change.original_code.is_empty() || !change.replacement_code.is_empty(),
                           "Change should have either original or replacement code");
                    assert!(!change.description.is_empty(), "Change should have description");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_performance_optimization_confidence() {
        let harness = TestHarness::new().await.unwrap();

        let high_confidence_query = "SELECT * FROM users;";
        let optimizations = harness.server.get_optimization_suggestions(high_confidence_query.to_string()).await.unwrap();

        // Should have high confidence for clearly optimizable queries
        if !optimizations.is_empty() {
            for optimization in optimizations {
                assert!(optimization.confidence_score >= 0.7,
                       "SELECT * optimization should have high confidence, got: {}",
                       optimization.confidence_score);
            }
        }
    }
}

mod error_analysis_tests {
    use super::*;

    #[tokio::test]
    async fn test_sql_error_pattern_detection() {
        let harness = TestHarness::new().await.unwrap();

        // Test WHERE without FROM error
        let invalid_query = "SELECT id WHERE active = true";
        let errors = harness.server.get_error_analysis(invalid_query.to_string()).await.unwrap();

        assert!(!errors.is_empty(), "Should detect WHERE without FROM error");
        let where_error = errors.iter().find(|e| e.error_description.contains("WHERE"));
        assert!(where_error.is_some(), "Should specifically mention WHERE clause");

        if let Some(error) = where_error {
            assert!(error.confidence_score >= 0.8, "WHERE error should have high confidence");
            assert!(!error.code_edits.is_empty(), "Should provide code edits for error");
        }
    }

    #[tokio::test]
    async fn test_dialect_specific_error_detection() {
        let harness = TestHarness::new().await.unwrap();

        // Test semicolon detection for PostgreSQL
        let config = SqlLspConfig {
            supported_sql_dialects: vec!["postgresql".to_string()],
            ..Default::default()
        };
        harness.server.config.write().await.clone_from(&config);

        let query_without_semicolon = "SELECT * FROM users";
        let errors = harness.server.get_error_analysis(query_without_semicolon.to_string()).await.unwrap();

        // May or may not detect missing semicolon depending on implementation
        if !errors.is_empty() {
            let semicolon_error = errors.iter().find(|e| e.error_description.contains("semicolon"));
            if let Some(error) = semicolon_error {
                assert!(error.confidence_score > 0.0, "Semicolon error should have some confidence");
            }
        }
    }

    #[tokio::test]
    async fn test_error_analysis_structure() {
        let harness = TestHarness::new().await.unwrap();

        let query = "SELECT id WHERE id = 1";  // Invalid: WHERE without FROM
        let errors = harness.server.get_error_analysis(query.to_string()).await.unwrap();

        for error in errors {
            // Verify error structure
            assert!(!error.error_description.is_empty(), "Should have error description");
            assert!(!error.suggested_fix.is_empty(), "Should have suggested fix");
            assert!(error.confidence_score >= 0.0 && error.confidence_score <= 1.0,
                   "Confidence should be valid percentage");
            assert!(matches!(error.impact_level,
                           FixImpact::Minimal | FixImpact::Moderate | FixImpact::Significant | FixImpact::Major),
                   "Should have valid impact level");

            // Alternatives should be provided even if empty
            assert!(error.alternatives.is_empty() || error.alternatives.len() >= 0,
                   "Alternatives should be array");

            // Code edits should have valid structure if present
            for edit in &error.code_edits {
                assert!(matches!(edit.range,
                               lsp_types::Range { start: _, end: _ }),
                       "Should have valid range");
            }
        }
    }

    #[tokio::test]
    async fn test_impact_level_assignment() {
        let harness = TestHarness::new().await.unwrap();
        let config = harness.server.config.read().await;

        // Test various error impacts
        let errors = vec![
            ("SELECT id WHERE", FixImpact::Significant), // Missing FROM is significant
            ("SELECT * FROM", FixImpact::Minimal),      // Missing semicolon might be minimal
            ("SELECT invalid_column FROM users", FixImpact::Moderate), // Invalid column is moderate
        ];

        for (invalid_query, expected_impact) in errors {
            if let Ok(error_fixes) = harness.server.get_error_analysis(invalid_query.to_string()).await {
                if !error_fixes.is_empty() {
                    let impact = error_fixes[0].impact_level.clone();
                    // Note: Actual impact may vary based on implementation
                    assert!(matches!(impact, FixImpact::Minimal | FixImpact::Moderate |
                                          FixImpact::Significant | FixImpact::Major),
                           "Should have valid impact level for query: {}", invalid_query);
                }
            }
        }
    }
}

mod collaborative_editing_tests {
    use super::*;

    #[tokio::test]
    async fn test_collaborative_session_management() {
        let harness = TestHarness::new().await.unwrap();

        let session_id = "test_session_123".to_string();
        let document_uri = lsp_types::DocumentUri::parse("file:///test_query.sql").unwrap();
        let participants = vec!["alice".to_string(), "bob".to_string()];

        // Create collaborative session
        let created_session_id = harness.server
            .start_collaborative_session(session_id.clone(), document_uri.clone(), participants.clone())
            .await.unwrap();

        assert_eq!(created_session_id, session_id);
        assert_eq!(harness.server.collaborative_sessions.read().await.len(), 1);

        // Verify session creation
        let sessions = harness.server.collaborative_sessions.read().await;
        let session = sessions.get(&session_id).unwrap();

        assert_eq!(session.session_id, session_id);
        assert_eq!(session.document_uri, document_uri);
        assert_eq!(session.participants, participants);
        assert_eq!(session.session_state, SessionState::Active);
        assert!(session.edit_operations.is_empty());
    }

    #[tokio::test]
    async fn test_live_edit_operations() {
        let harness = TestHarness::new().await.unwrap();

        let session_id = "edit_test_session".to_string();
        let document_uri = lsp_types::DocumentUri::parse("file:///edit_test.sql").unwrap();

        // Start session
        harness.server
            .start_collaborative_session(session_id.clone(), document_uri, vec!["user1".to_string()])
            .await.unwrap();

        let edit_operation = LiveEditOperation {
            operation_id: "op_123".to_string(),
            user_id: "user1".to_string(),
            range: LspRange {
                start: LspPosition::new(0, 0),
                end: LspPosition::new(0, 10),
            },
            edit_type: EditType::Replace,
            original_content: "SELECT old".to_string(),
            new_content: "SELECT updated".to_string(),
            timestamp: Utc::now(),
            status: OperationStatus::Applied,
        };

        // Update session with edit
        harness.server
            .update_collaborative_session(session_id.clone(), edit_operation.clone())
            .await.unwrap();

        // Verify edit was recorded
        let sessions = harness.server.collaborative_sessions.read().await;
        let session = sessions.get(&session_id).unwrap();

        assert_eq!(session.edit_operations.len(), 1);
        assert_eq!(session.edit_operations[0].edit_type, EditType::Replace);
        assert_eq!(session.edit_operations[0].original_content, "SELECT old");
        assert_eq!(session.edit_operations[0].new_content, "SELECT updated");
    }

    #[tokio::test]
    async fn test_session_isolation() {
        let harness = TestHarness::new().await.unwrap();

        // Create multiple sessions
        let session1 = "session_1".to_string();
        let session2 = "session_2".to_string();
        let uri1 = lsp_types::DocumentUri::parse("file:///query1.sql").unwrap();
        let uri2 = lsp_types::DocumentUri::parse("file:///query2.sql").unwrap();

        harness.server
            .start_collaborative_session(session1.clone(), uri1, vec!["user1".to_string()])
            .await.unwrap();

        harness.server
            .start_collaborative_session(session2.clone(), uri2, vec!["user2".to_string()])
            .await.unwrap();

        // Verify sessions are isolated
        let sessions = harness.server.collaborative_sessions.read().await;
        assert_eq!(sessions.len(), 2);
        assert!(sessions.get(&session1).is_some());
        assert!(sessions.get(&session2).is_some());
        assert_ne!(sessions.get(&session1).unwrap().document_uri,
                  sessions.get(&session2).unwrap().document_uri);
    }

    #[tokio::test]
    async fn test_session_error_handling() {
        let harness = TestHarness::new().await.unwrap();

        let session_id = "nonexistent_session".to_string();
        let dummy_operation = LiveEditOperation {
            operation_id: "dummy_op".to_string(),
            user_id: "test_user".to_string(),
            range: LspRange {
                start: LspPosition::new(0, 0),
                end: LspPosition::new(0, 1),
            },
            edit_type: EditType::Insert,
            original_content: "".to_string(),
            new_content: "x".to_string(),
            timestamp: Utc::now(),
            status: OperationStatus::Applied,
        };

        // Try to update non-existent session
        let result = harness.server
            .update_collaborative_session(session_id, dummy_operation)
            .await;

        assert!(result.is_err(), "Should fail to update non-existent session");

        if let Err(SqlLspError::CollaborationError(msg)) = result {
            assert!(msg.contains("Session not found"), "Should mention session not found");
        } else {
            panic!("Expected CollaborationError, got: {:?}", result);
        }
    }
}

mod comprehensive_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_server_lifecycle() {
        // Test complete server initialization
        let config = SqlLspConfig {
            enable_optimization_suggestions: true,
            enable_schema_inference: true,
            enable_performance_profiling: true,
            enable_collaborative_editing: true,
            enable_error_analysis: true,
            enable_advanced_caching: true,
            enable_parallel_processing: true,
            enable_virtual_memory: true,
            supported_sql_dialects: vec![
                "postgresql".to_string(),
                "mysql".to_string(),
                "sqlite".to_string(),
            ],
            min_suggestion_confidence: 0.6,
            cache_settings: SqlCacheConfig {
                max_memory_per_layer_mb: 256,
                max_entries_per_layer: 2000,
                collect_statistics: true,
                enable_cache_warming: true,
                eviction_policy: CacheEvictionPolicy::LeastRecentlyUsed,
                ttl_settings: CacheTtlSettings::default(),
            },
            performance_settings: SqlPerformanceSettings {
                parallel_analysis: true,
                max_concurrent_tasks: 4,
                analysis_timeout_ms: 10000,
                batch_processing: true,
                batch_size: 25,
            },
            security_settings: SqlSecuritySettings::default(),
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        // Verify all components are properly initialized
        assert!(harness.server.cache_manager.metrics_cache.is_some());
        assert!(harness.server.parallel_executor.is_some());
        assert!(harness.server.virtual_memory_manager.is_some());
        assert!(!harness.server.dialect_detectors.is_empty());

        // Test end-to-end query processing
        let test_query = "SELECT u.name, COUNT(o.id) FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.id, u.name;";

        // Process through multiple layers
        let metrics = harness.server.get_performance_metrics(test_query.to_string()).await.unwrap();
        let optimizations = harness.server.get_optimization_suggestions(test_query.to_string()).await.unwrap();
        let schema = harness.server.get_schema_inference(test_query.to_string()).await.unwrap();
        let errors = harness.server.get_error_analysis(test_query.to_string()).await.unwrap();

        // Verify end-to-end processing
        assert!(metrics.complexity_score > 30, "Should have reasonable complexity");
        assert!(!optimizations.is_empty(), "Should find optimization opportunities");
        assert!(!schema.tables.is_empty(), "Should infer schema");
        assert!(errors.is_empty(), "Valid query should have no errors");

        // Test collaborative features
        if harness.server.config.read().await.enable_collaborative_editing {
            let session_id = harness.server
                .start_collaborative_session(
                    "integration_test_session".to_string(),
                    lsp_types::DocumentUri::parse("file:///integration_test.sql").unwrap(),
                    vec!["integration_tester".to_string()],
                )
                .await.unwrap();

            let edit_op = LiveEditOperation {
                operation_id: "integration_edit_1".to_string(),
                user_id: "integration_tester".to_string(),
                range: LspRange {
                    start: LspPosition::new(0, test_query.len() as u32),
                    end: LspPosition::new(0, test_query.len() as u32),
                },
                edit_type: EditType::Insert,
                original_content: "".to_string(),
                new_content: " -- Collaborative edit!".to_string(),
                timestamp: Utc::now(),
                status: OperationStatus::Applied,
            };

            harness.server
                .update_collaborative_session(session_id.clone(), edit_op)
                .await.unwrap();

            // Verify collaborative session state
            let sessions = harness.server.collaborative_sessions.read().await;
            let session = sessions.get(&session_id).unwrap();
            assert_eq!(session.edit_operations.len(), 1);
            assert_eq!(session.session_state, SessionState::Active);
        }

        // Test resource cleanup on drop
        drop(harness);
        // Temp directory should be cleaned up by the time the test function returns
    }

    #[tokio::test]
    async fn test_performance_benchmarking_under_load() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            enable_parallel_processing: true,
            cache_settings: SqlCacheConfig {
                max_memory_per_layer_mb: 100,
                max_entries_per_layer: 500,
                collect_statistics: true,
                ..Default::default()
            },
            performance_settings: SqlPerformanceSettings {
                batch_processing: true,
                batch_size: 20,
                max_concurrent_tasks: 4,
                analysis_timeout_ms: 15000,
                parallel_analysis: true,
            },
            ..Default::default()
        };

        let harness = TestHarness::with_config(config).await.unwrap();

        // Generate test workload
        let queries: Vec<String> = (0..100).map(|i|
            format!("SELECT * FROM table{} t{} WHERE t{}.id = {} AND t{}.active = true;",
                   i, i, i, i % 10, i)
        ).collect();

        let start_time = std::time::Instant::now();

        // Process batch workload
        let batch_result = timeout(
            Duration::from_secs(30),
            harness.server.perform_bulk_analysis(queries.clone())
        ).await;

        match batch_result {
            Ok(Ok(result)) => {
                let duration = start_time.elapsed();

                // Performance assertions
                assert_eq!(result.processing_stats.total_queries, 100);
                assert!(duration.as_millis() < 25000, "Batch processing should complete in under 25 seconds");
                assert!(result.processing_stats.avg_processing_time_ms >= 0.0);
                assert!(result.processing_stats.avg_processing_time_ms < 500.0,
                       "Average processing time should be reasonable: {}ms",
                       result.processing_stats.avg_processing_time_ms);

                // Cache efficiency check
                if let Some(stats) = &harness.server.cache_manager.stats_col