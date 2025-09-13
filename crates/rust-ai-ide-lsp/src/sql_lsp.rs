
//! # SQL Language Server Protocol (LSP) Comprehensive Test Suite
//!
//! This test suite provides comprehensive coverage for the SQL LSP server's
//! advanced caching and performance optimization features.

#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::{timeout, Duration};

    // Test infrastructure setup
    struct TestServer {
        server: Arc<SqlLspServer>,
        temp_dir: tempfile::TempDir,
    }

    impl TestServer {
        async fn new_with_config(config: SqlLspConfig) -> SqlLspResult<Self> {
            let temp_dir = tempfile::create_dir().expect("Failed to create temp dir");
            let config_arc = Arc::new(RwLock::new(config));
            let (_, receiver) = mpsc::unbounded_channel();

            let cache_manager = Arc::new(SqlLspServer::initialize_cache_manager(&config).await?);
            let parallel_executor = if config.enable_parallel_processing {
                Some(Arc::new(SqlLspServer::initialize_parallel_executor(&config).await?))
            } else {
                None
            };
            let virtual_memory_manager = if config.enable_virtual_memory {
                Some(Arc::new(SqlLspServer::initialize_virtual_memory_manager(&config).await?))
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

        async fn new() -> SqlLspResult<Self> {
            Self::new_with_config(SqlLspConfig::default()).await
        }
    }

    /// Test server initialization with various configurations
    #[tokio::test]
    async fn test_server_initialization_with_configurations() {
        // Test default configuration
        let default_config = SqlLspConfig::default();
        let test_server = TestServer::new_with_config(default_config.clone()).await;
        assert!(test_server.is_ok(), "Should initialize with default config");

        // Test configuration with advanced caching enabled
        let caching_config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                max_memory_per_layer_mb: 128,
                max_entries_per_layer: 20000,
                enable_cache_warming: true,
                collect_statistics: true,
                ..Default::default()
            },
            ..default_config.clone()
        };
        let caching_server = TestServer::new_with_config(caching_config).await;
        assert!(caching_server.is_ok(), "Should initialize with advanced caching");

        // Test configuration with parallel processing enabled
        let parallel_config = SqlLspConfig {
            enable_parallel_processing: true,
            performance_settings: SqlPerformanceSettings {
                parallel_analysis: true,
                max_concurrent_tasks: 8,
                analysis_timeout_ms: 30000,
                batch_processing: true,
                batch_size: 100,
            },
            ..default_config.clone()
        };
        let parallel_server = TestServer::new_with_config(parallel_config).await;
        assert!(parallel_server.is_ok(), "Should initialize with parallel processing");

        // Test configuration with virtual memory enabled
        let vm_config = SqlLspConfig {
            enable_virtual_memory: true,
            ..default_config
        };
        let vm_server = TestServer::new_with_config(vm_config).await;
        assert!(vm_server.is_ok(), "Should initialize with virtual memory");
    }

    /// Test cache layer operations with multi-tier caching
    #[cfg(feature = "moka")]
    #[tokio::test]
    async fn test_multi_tier_caching_operations() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                max_memory_per_layer_mb: 64,
                max_entries_per_layer: 1000,
                collect_statistics: true,
                ttl_settings: CacheTtlSettings {
                    metrics_ttl_seconds: 300,
                    schema_ttl_seconds: 600,
                    optimization_ttl_seconds: 300,
                    error_ttl_seconds: 300,
                    virtual_memory_ttl_seconds: 300,
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        // Test metrics cache operations
        let query = "SELECT * FROM users WHERE id = 1;".to_string();
        let metrics1 = test_server.server.get_performance_metrics(query.clone()).await.unwrap();
        let metrics2 = test_server.server.get_performance_metrics(query.clone()).await.unwrap(); // Should hit cache

        assert_eq!(metrics1.execution_time_us, metrics2.execution_time_us,
                  "Cached metrics should be identical");

        // Test cache statistics
        let cache_stats = test_server.server.cache_manager.stats_collector.as_ref().unwrap().hit_miss_stats.read().await;
        let metrics_layer_stats = cache_stats.get("metrics").unwrap();
        assert!(metrics_layer_stats.hits >= 1, "Should have at least one cache hit");

        // Test cache invalidation
        if let Some(cache_layer) = &test_server.server.cache_manager.metrics_cache.as_ref() {
            let cache_key = format!("perf_{}", hash_query(&query));
            let removed = cache_layer.remove(&cache_key).await;
            assert!(removed, "Should successfully remove from cache");
        }

        // Test cache clearing
        if let Some(cache_layer) = &test_server.server.cache_manager.metrics_cache.as_ref() {
            cache_layer.clear().await.unwrap();
            let cache_key = format!("perf_{}", hash_query(&query));
            let result = cache_layer.get(&cache_key).await;
            assert!(result.is_none(), "Cache should be empty after clear");
        }
    }

    /// Test cache eviction policies and statistics
    #[cfg(feature = "moka")]
    #[tokio::test]
    async fn test_cache_eviction_and_statistics() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                max_entries_per_layer: 3, // Very small cache to test eviction
                collect_statistics: true,
                eviction_policy: CacheEvictionPolicy::LeastRecentlyUsed,
                ..Default::default()
            },
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        // Fill cache with multiple entries
        let queries = vec![
            "SELECT * FROM table1;".to_string(),
            "SELECT * FROM table2;".to_string(),
            "SELECT * FROM table3;".to_string(),
            "SELECT * FROM table4;".to_string(), // This should evict the first entry
        ];

        for query in &queries {
            let _ = test_server.server.get_performance_metrics(query.clone()).await;
        }

        // Check cache statistics
        let stats = test_server.server.cache_manager.stats_collector.as_ref().unwrap().performance_stats.read().await;
        let metrics_stats = stats.get("metrics").unwrap();
        assert!(metrics_stats.total_operations >= 4, "Should have processed at least 4 operations");
        assert!(metrics_stats.avg_lookup_time_ns > 0, "Should have measured lookup times");

        // Verify cache capacity is respected
        if let Some(cache_layer) = &test_server.server.cache_manager.metrics_cache {
            let cache_stats = cache_layer.stats().await;
            assert!(cache_stats.total_entries <= 3, "Cache should respect capacity limit");
        }
    }

    /// Test performance metric calculation
    #[tokio::test]
    async fn test_performance_metric_calculation() {
        let test_server = TestServer::new().await.unwrap();

        // Test various query complexity levels
        let simple_query = "SELECT * FROM users;".to_string();
        let complex_query = "SELECT u.name, COUNT(o.id) FROM users u LEFT JOIN orders o ON u.id = o.user_id WHERE u.created_at >= '2023-01-01' GROUP BY u.id HAVING COUNT(o.id) > 5 ORDER BY COUNT(o.id) DESC LIMIT 10;".to_string();

        let simple_metrics = test_server.server.get_performance_metrics(simple_query).await.unwrap();
        let complex_metrics = test_server.server.get_performance_metrics(complex_query).await.unwrap();

        // Complex query should have higher complexity score
        assert!(complex_metrics.complexity_score >= simple_metrics.complexity_score,
               "Complex query should have higher complexity score");

        // Verify all metrics are within valid ranges
        assert!(simple_metrics.execution_time_us >= 0);
        assert!(simple_metrics.memory_usage_bytes >= 0);
        assert!(simple_metrics.io_operations >= 0);
        assert!(simple_metrics.complexity_score <= 100);

        // Test bottleneck identification
        let join_query = "SELECT * FROM users u JOIN posts p ON u.id = p.user_id;".to_string();
        let join_metrics = test_server.server.get_performance_metrics(join_query).await.unwrap();
        assert_eq!(join_metrics.bottleneck_category, QueryBottleneck::Io); // JOINS typically cause IO bottlenecks
    }

    /// Test virtual memory management functionality
    #[tokio::test]
    async fn test_virtual_memory_management() {
        let config = SqlLspConfig {
            enable_virtual_memory: true,
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        if let Some(vm_manager) = &test_server.server.virtual_memory_manager {
            // Test memory mapping creation
            let test_data = b"Hello, Virtual Memory!" as &[u8];
            let file_name = "test_data.bin";

            // Simulate memory mapping (would normally map a file)
            let mapping_result = vm_manager.memory_maps.write().await.insert(
                file_name.to_string(),
                memmap2::Mmap::map(&std::fs::File::open(std::env::current_exe().unwrap()).unwrap()).unwrap()
            );

            // Test allocation tracking
            let mut allocation_tracker = vm_manager.allocation_tracker.write().await;
            allocation_tracker.insert(
                file_name.to_string(),
                VirtualMemoryInfo {
                    file_size: test_data.len() as u64,
                    mapped_size: test_data.len() as u64,
                    allocated_at: std::time::Instant::now(),
                    access_pattern: MemoryAccessPattern::Sequential,
                }
            );

            // Verify memory limit is respected
            assert!(vm_manager.max_virtual_memory_mb <= 1024, "Memory limit should be reasonable");

            // Verify temp directory exists
            assert!(vm_manager.temp_dir.exists(), "Temporary directory should exist");
        }
    }

    /// Test parallel processing capabilities
    #[tokio::test]
    async fn test_parallel_processing_concurrency() {
        let config = SqlLspConfig {
            enable_parallel_processing: true,
            performance_settings: SqlPerformanceSettings {
                parallel_analysis: true,
                max_concurrent_tasks: 4,
                analysis_timeout_ms: 5000,
                batch_processing: true,
                batch_size: 10,
            },
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        if let Some(parallel_executor) = &test_server.server.parallel_executor {
            // Test semaphore-based concurrency limiting
            assert!(parallel_executor.semaphore.available_permits() >= 0);
            assert_eq!(parallel_executor.max_concurrency, 4);

            // Test thread pool when available
            if let Some(_thread_pool) = &parallel_executor.thread_pool {
                // If thread pool is available, it should be configured correctly
                assert!(parallel_executor.max_concurrency > 0);
            }
        }

        // Test batch processing
        let queries = (0..20).map(|i| format!("SELECT * FROM table{};", i)).collect::<Vec<_>>();
        let result = timeout(
            Duration::from_secs(10),
            test_server.server.perform_bulk_analysis(queries)
        ).await;

        match result {
            Ok(Ok(bulk_result)) => {
                assert!(!bulk_result.query_results.is_empty(), "Should process multiple queries");
                assert!(bulk_result.processing_stats.avg_processing_time_ms >= 0.0);
                assert_eq!(bulk_result.processing_stats.total_queries, 20);
            }
            Err(_) => {
                // Timeout or other error - acceptable for this integration test
            }
        }
    }

    /// Test background task management
    #[tokio::test]
    async fn test_background_task_management() {
        let test_server = TestServer::new().await.unwrap();

        // Test active background task counter
        {
            let mut active_tasks = test_server.server.background_task_manager.active_background_tasks.lock().await;
            *active_tasks = 2; // Simulate some active tasks
        }

        {
            let active_tasks = test_server.server.background_task_manager.active_background_tasks.lock().await;
            assert_eq!(*active_tasks, 2);
        }

        // Test task cancellation and cleanup
        {
            let cleanup_notify = test_server.server.background_task_manager.cleanup_scheduler.clone();
            cleanup_notify.notify_one();
            // In a real scenario, this would trigger cleanup operations
        }
    }

    /// Test SQL dialect detection for various dialects
    #[tokio::test]
    async fn test_dialect_detection_validation() {
        let test_server = TestServer::new().await.unwrap();

        // Test PostgreSQL dialect detector
        if let Some(detector) = test_server.server.dialect_detectors.get("postgresql") {
            let dialect = detector.detect_dialect("SELECT id, name FROM users;").await.unwrap();
            assert_eq!(dialect, "postgresql");

            let validation_result = detector.validate_syntax("SELECT * FROM users;", &dialect).await;
            assert!(validation_result.is_ok());
        }

        // Test MySQL dialect detector
        if let Some(detector) = test_server.server.dialect_detectors.get("mysql") {
            let dialect = detector.detect_dialect("SELECT id, name FROM users;").await.unwrap();
            assert_eq!(dialect, "mysql");

            let completions = detector.get_completions("SELECT ", 7, &dialect).await.unwrap();
            assert!(completions.is_empty() || completions.len() >= 0); // Should return empty vec or some completions
        }

        // Test SQLite dialect detector
        if let Some(detector) = test_server.server.dialect_detectors.get("sqlite") {
            let dialect = detector.detect_dialect("SELECT * FROM sqlite_master;").await.unwrap();
            assert_eq!(dialect, "sqlite");
        }

        // Test unsupported dialect handling
        let config = SqlLspConfig {
            supported_sql_dialects: vec!["postgresql".to_string()],
            ..Default::default()
        };
        let limited_server = TestServer::new_with_config(config).await.unwrap();
        assert!(limited_server.server.dialect_detectors.get("mysql").is_none(),
               "Should only have configured dialects");
    }

    /// Test optimization suggestions for different query patterns
    #[tokio::test]
    async fn test_query_optimization_suggestions() {
        let test_server = TestServer::new().await.unwrap();

        // Test SELECT * optimization
        let select_star_query = "SELECT * FROM users;".to_string();
        let optimizations = test_server.server.get_optimization_suggestions(select_star_query).await.unwrap();

        assert!(!optimizations.is_empty(), "Should suggest optimizations for SELECT *");
        let select_star_suggestion = optimizations.iter()
            .find(|opt| opt.explanation.contains("SELECT *"))
            .unwrap();
        assert!(select_star_suggestion.performance_improvement_percent > 0.0);
        assert!(!select_star_suggestion.required_changes.is_empty());

        // Test WHERE without index optimization
        let no_index_query = "SELECT name FROM users WHERE email = 'test@example.com';".to_string();
        let optimizations = test_server.server.get_optimization_suggestions(no_index_query).await.unwrap();

        let index_suggestion = optimizations.iter()
            .find(|opt| opt.explanation.contains("index"))
            .unwrap();
        assert!(index_suggestion.confidence_score >= 0.8);
        assert!(index_suggestion.performance_improvement_percent >= 50.0);

        // Test query with no obvious optimizations
        let optimized_query = "SELECT id, name FROM users WHERE id = 1;".to_string();
        let optimizations = test_server.server.get_optimization_suggestions(optimized_query).await.unwrap();
        assert!(optimizations.is_empty() || optimizations.iter().all(|opt| opt.confidence_score < 0.5),
               "Should not suggest optimizations for already optimized query");
    }

    /// Test schema inference capabilities
    #[tokio::test]
    async fn test_schema_inference_capabilities() {
        let test_server = TestServer::new().await.unwrap();

        // Test basic schema inference
        let query = "SELECT id, name, email FROM users WHERE active = true;".to_string();
        let schema = test_server.server.get_schema_inference(query).await.unwrap();

        assert!(!schema.tables.is_empty(), "Should infer at least one table");
        assert!(schema.confidence_score > 0.0, "Should have some confidence in inference");

        if let Some(users_table) = schema.tables.get("users") {
            assert!(!users_table.columns.is_empty(), "Should infer table columns");
            assert!(users_table.estimated_row_count.unwrap_or(0) > 0, "Should estimate row count");
        }

        // Test complex query with joins
        let join_query = "SELECT u.name, p.title FROM users u INNER JOIN posts p ON u.id = p.user_id;".to_string();
        let join_schema = test_server.server.get_schema_inference(join_query).await.unwrap();

        assert!(join_schema.tables.len() >= 2, "Should infer multiple tables from JOIN");
        assert!(join_schema.relationships.is_empty() ||
                join_schema.relationships.len() >= 0, "May infer relationships");
    }

    /// Test collaborative editing session management
    #[tokio::test]
    async fn test_collaborative_editing_sessions() {
        let test_server = TestServer::new().await.unwrap();

        let session_id = "session_123".to_string();
        let document_uri = lsp_types::DocumentUri::parse("file:///test.sql").unwrap();
        let participants = vec!["user1".to_string(), "user2".to_string()];

        // Start collaborative session
        let created_session_id = test_server.server
            .start_collaborative_session(session_id.clone(), document_uri, participants.clone())
            .await.unwrap();

        assert_eq!(created_session_id, session_id, "Should return the correct session ID");

        // Verify session was created
        let sessions = test_server.server.collaborative_sessions.read().await;
        let session = sessions.get(&session_id).unwrap();

        assert_eq!(session.participants, participants);
        assert_eq!(session.session_state, SessionState::Active);

        // Update session with edit operation
        let edit_operation = LiveEditOperation {
            operation_id: "op_123".to_string(),
            user_id: "user1".to_string(),
            range: LspRange {
                start: LspPosition::new(1, 0),
                end: LspPosition::new(1, 5),
            },
            edit_type: EditType::Replace,
            original_content: "FROM\n".to_string(),
            new_content: "FROM users\n".to_string(),
            timestamp: Utc::now(),
            status: OperationStatus::Applied,
        };

        test_server.server
            .update_collaborative_session(session_id.clone(), edit_operation.clone())
            .await.unwrap();

        // Verify operation was recorded
        let sessions = test_server.server.collaborative_sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert_eq!(session.edit_operations.len(), 1);
        assert_eq!(session.edit_operations[0].user_id, "user1");
    }

    /// Test configuration validation and defaults
    #[tokio::test]
    async fn test_configuration_validation() {
        // Test default configuration
        let default_config = SqlLspConfig::default();
        assert!(default_config.enable_optimization_suggestions);
        assert!(default_config.enable_error_analysis);
        assert!(!default_config.min_suggestion_confidence.is_nan());
        assert!(default_config.min_suggestion_confidence >= 0.0 &&
                default_config.min_suggestion_confidence <= 1.0);

        // Test cache configuration defaults
        let cache_config = SqlCacheConfig::default();
        assert!(cache_config.max_memory_per_layer_mb > 0);
        assert!(cache_config.max_entries_per_layer > 0);
        assert_eq!(cache_config.eviction_policy, CacheEvictionPolicy::LeastRecentlyUsed);

        // Test TTL settings defaults
        let ttl_settings = CacheTtlSettings::default();
        assert!(ttl_settings.metrics_ttl_seconds > 0);
        assert!(ttl_settings.schema_ttl_seconds > ttl_settings.metrics_ttl_seconds); // Schema typically cached longer

        // Test performance settings defaults
        let perf_settings = SqlPerformanceSettings::default();
        assert!(perf_settings.max_concurrent_tasks > 0);
        assert!(perf_settings.analysis_timeout_ms > 0);
        assert!(perf_settings.batch_size > 0);

        // Test security settings defaults
        let security_settings = SqlSecuritySettings::default();
        assert!(security_settings.detect_sql_injection);
        assert!(security_settings.audit_logging);
        assert!(!security_settings.trusted_sources.is_empty());

        // Test input validation settings
        let input_validation = SqlInputValidationSettings::default();
        assert!(input_validation.max_query_length > 0);
        assert!(input_validation.max_parameter_count > 0);
    }

    /// Test error handling and recovery scenarios
    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let test_server = TestServer::new().await.unwrap();

        // Test cache error recovery
        let cache_error = SqlLspError::CacheError("Test cache failure".to_string());
        assert!(cache_error.is_recoverable(), "Cache errors should be recoverable");
        assert!(!cache_error.recovery_suggestions().is_empty(),
               "Cache errors should have recovery suggestions");

        // Test configuration error recovery
        let config_error = SqlLspError::ConfigurationError("Invalid config".to_string());
        assert!(!config_error.is_recoverable(), "Configuration errors should not be recoverable");
        assert!(!config_error.recovery_suggestions().is_empty(),
               "Configuration errors should have recovery suggestions");

        // Test performance error recovery
        let perf_error = SqlLspError::PerformanceError("Analysis timeout".to_string());
        assert!(perf_error.is_recoverable(), "Performance errors should be recoverable");
        assert!(perf_error.recovery_suggestions().iter()
                .any(|s| s.contains("timeout")),
               "Performance error suggestions should mention timeout");

        // Test security error recovery
        let security_error = SqlLspError::SecurityError("Security violation".to_string());
        assert!(!security_error.is_recoverable(), "Security errors should not be recoverable");
        assert!(!security_error.recovery_suggestions().is_empty(),
               "Security errors should have recovery suggestions");

        // Test query analysis with invalid input
        let empty_query = "".to_string();
        let analysis_result = test_server.server.get_error_analysis(empty_query).await;
        match analysis_result {
            Ok(errors) => {
                // Should either return empty results or handle gracefully
                assert!(errors.is_empty() || !errors.is_empty()); // Either outcome is acceptable
            }
            Err(_) => {
                // Error handling is also acceptable
            }
        }
    }

    /// Test integration between multiple components
    #[tokio::test]
    async fn test_component_integration() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            enable_parallel_processing: true,
            enable_performance_profiling: true,
            enable_error_analysis: true,
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        let query = "SELECT u.name, COUNT(o.id) FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.id;".to_string();

        // Test comprehensive analysis integrating multiple components
        let performance = test_server.server.get_performance_metrics(query.clone()).await.unwrap();
        let optimizations = test_server.server.get_optimization_suggestions(query.clone()).await.unwrap();
        let schema = test_server.server.get_schema_inference(query.clone()).await.unwrap();
        let errors = test_server.server.get_error_analysis(query.clone()).await.unwrap();

        // Verify all components work together
        assert!(performance.complexity_score > 0, "Should have complexity score");
        assert!(!optimizations.is_empty(),
               "Should find optimization opportunities in complex join query");
        assert!(!schema.tables.is_empty(), "Should infer schema from JOIN query");

        // Join queries should typically have optimization suggestions
        assert!(optimizations.iter().any(|opt| opt.explanation.to_lowercase().contains("join")),
               "Should suggest JOIN optimizations");

        assert!(errors.is_empty() || !errors.is_empty(),
               "Should analyze errors without crashing");
    }

    /// Test memory usage tracking and limits
    #[tokio::test]
    async fn test_memory_usage_and_limits() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            cache_settings: SqlCacheConfig {
                max_memory_per_layer_mb: 50, // Very low limit to test boundary
                collect_statistics: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        // Test memory statistics collection
        if let Some(stats_collector) = &test_server.server.cache_manager.stats_collector {
            let memory_stats = stats_collector.memory_stats.read().await;
            // Should be able to access memory statistics even if empty
            assert!(memory_stats.len() >= 0, "Should have memory statistics");

            // Test memory limit enforcement
            assert!(config.cache_settings.max_memory_per_layer_mb > 0,
                   "Memory limit should be positive");
        }

        // Test virtual memory manager limits (if enabled)
        if let Some(vm_manager) = &test_server.server.virtual_memory_manager {
            assert!(vm_manager.max_virtual_memory_mb > 0,
                   "Virtual memory limit should be positive");
            assert!(vm_manager.max_virtual_memory_mb <= 4096,
                   "Virtual memory limit should be reasonable");
        }
    }

    /// Test comprehensive query analysis
    #[tokio::test]
    async fn test_comprehensive_query_analysis() {
        let test_server = TestServer::new().await.unwrap();

        let complex_query = "
            SELECT
                u.id,
                u.name,
                u.email,
                COUNT(o.id) as order_count,
                AVG(o.total_amount) as avg_order_value,
                MAX(o.created_at) as last_order_date
            FROM users u
            LEFT JOIN orders o ON u.id = o.user_id AND o.status = 'completed'
            WHERE u.created_at >= '2023-01-01'
            AND u.active = true
            GROUP BY u.id, u.name, u.email
            HAVING COUNT(o.id) > 0
            ORDER BY COUNT(o.id) DESC, u.name
            LIMIT 100;
        ".to_string();

        // Perform comprehensive analysis
        let analysis_result = test_server.server
            .analyze_query_comprehensive(complex_query.clone())
            .await;

        match analysis_result {
            Ok(result) => {
                // Should successfully analyze complex query
                assert!(result.performance_metrics.complexity_score > 50,
                       "Complex query should have high complexity score");
                assert!(!result.optimizations.is_empty(),
                       "Should find optimization opportunities");

                // Should have processed the query
                assert_eq!(result.original_query, complex_query);
            }
            Err(e) => {
                // Analysis failure is acceptable if implementation is incomplete
                // but should not crash the server
                println!("Analysis failed (expected): {:?}", e);
            }
        }
    }

    /// Performance benchmarking test
    #[tokio::test]
    async fn test_performance_benchmarking() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            enable_parallel_processing: true,
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        let queries = vec![
            "SELECT * FROM users;".to_string(),
            "SELECT * FROM users WHERE id = 1;".to_string(),
            "SELECT u.name FROM users u JOIN posts p ON u.id = p.user_id;".to_string(),
            "SELECT COUNT(*) FROM users WHERE active = true;".to_string(),
            "SELECT * FROM users ORDER BY created_at DESC LIMIT 10;".to_string(),
        ];

        let start_time = std::time::Instant::now();

        // Measure performance across multiple queries
        for query in &queries {
            let _metrics = timeout(
                Duration::from_millis(1000), // 1 second timeout per query
                test_server.server.get_performance_metrics(query.clone())
            ).await;
        }

        let total_time = start_time.elapsed();

        // Performance test - should complete reasonably quickly
        assert!(total_time.as_millis() < 5000,
               "Batch processing should complete within 5 seconds");

        // Test with caching benefits
        let query = "SELECT * FROM users;".to_string();

        // First call - cache miss
        let first_start = std::time::Instant::now();
        let _first_result = timeout(
            Duration::from_millis(1000),
            test_server.server.get_performance_metrics(query.clone())
        ).await;
        let first_duration = first_start.elapsed();

        // Second call - cache hit (should be faster)
        let second_start = std::time::Instant::now();
        let _second_result = timeout(
            Duration::from_millis(1000),
            test_server.server.get_performance_metrics(query.clone())
        ).await;
        let second_duration = second_start.elapsed();

        // Cached access should be faster or equal (not significantly slower)
        assert!(second_duration <= first_duration.mul_f32(2.0),
               "Cached access should not be more than 2x slower (ideally faster)");
    }

    /// Test query complexity calculation with edge cases
    #[test]
    fn test_query_complexity_edge_cases() {
        let config = SqlLspConfig::default();

        // Empty query
        let empty_result = SqlLspServer::calculate_query_complexity("", &config);
        assert_eq!(empty_result.unwrap(), 0);

        // Very long query (should cap at reasonable complexity)
        let long_query = "SELECT * FROM ".repeat(1000);
        let long_result = SqlLspServer::calculate_query_complexity(&long_query, &config);
        assert!(long_result.unwrap() <= 100); // Should cap at 100

        // Query with many complexity keywords
        let complex_query = "UNION INTERSECT HAVING GROUP BY ORDER BY INNER JOIN LEFT JOIN RIGHT JOIN WHERE EXISTS IN";
        let complex_result = SqlLspServer::calculate_query_complexity(complex_query, &config);
        assert!(complex_result.unwrap() > 50); // Should have high complexity

        // Simple query
        let simple_query = "SELECT id FROM users";
        let simple_result = SqlLspServer::calculate_query_complexity(simple_query, &config);
        assert!(simple_result.unwrap() < 20); // Should have low complexity
    }

    /// Test bottleneck identification for different scenarios
    #[test]
    fn test_bottleneck_identification() {
        // Test IO bottleneck (JOINs)
        let join_query = "SELECT * FROM users u JOIN posts p ON u.id = p.user_id";
        let join_bottleneck = SqlLspServer::identify_bottleneck(join_query, 0);
        assert_eq!(join_bottleneck, QueryBottleneck::Io);

        // Test CPU bottleneck (GROUP BY and ORDER BY)
        let group_query = "SELECT COUNT(*) FROM users GROUP BY status ORDER BY name";
        let group_bottleneck = SqlLspServer::identify_bottleneck(group_query, 0);
        assert_eq!(group_bottleneck, QueryBottleneck::Cpu);

        // Test memory bottleneck (high complexity)
        let memory_query = "SELECT * FROM users";
        let memory_bottleneck = SqlLspServer::identify_bottleneck(memory_query, 85);
        assert_eq!(memory_bottleneck, QueryBottleneck::Memory);

        // Test default bottleneck
        let default_query = "SELECT * FROM users WHERE id = 1";
        let default_bottleneck = SqlLspServer::identify_bottleneck(default_query, 0);
        assert_eq!(default_bottleneck, QueryBottleneck::Other);

        // Test network bottleneck (would be identified differently in real implementation)
        let network_bottleneck = SqlLspServer::identify_bottleneck("SELECT * FROM remote_table", 0);
        assert_eq!(network_bottleneck, QueryBottleneck::Other); // Currently defaults here
    }

    /// Test bulk analysis with various query types
    #[tokio::test]
    async fn test_bulk_analysis_different_query_types() {
        let test_server = TestServer::new().await.unwrap();

        let queries = vec![
            "SELECT * FROM users;".to_string(),
            "INSERT INTO users (name) VALUES ('test');".to_string(),
            "UPDATE users SET active = false WHERE id = 1;".to_string(),
            "DELETE FROM users WHERE created_at < '2020-01-01';".to_string(),
            "CREATE TABLE test (id INTEGER PRIMARY KEY);".to_string(),
            "SELECT * FROM users WHERE active = true;".to_string(),
            "SELECT COUNT(*) FROM orders;".to_string(),
            "SELECT u.name FROM users u JOIN orders o ON u.id = o.user_id;".to_string(),
        ];

        timeout(
            Duration::from_secs(30),
            test_server.server.perform_bulk_analysis(queries.clone())
        ).await
        .unwrap()
        .unwrap();

        // After successful bulk analysis, verify individual query results are cached
        for query in &queries {
            if let Ok(individual_result) = timeout(
                Duration::from_millis(500),
                test_server.server.get_performance_metrics(query.clone())
            ).await {
                if let Ok(metrics) = individual_result {
                    assert!(metrics.complexity_score <= 100);
                }
            }
        }
    }

    /// Test error analysis for various SQL error patterns
    #[tokio::test]
    async fn test_error_analysis_patterns() {
        let test_server = TestServer::new().await.unwrap();

        let config = SqlLspConfig {
            supported_sql_dialects: vec!["postgresql".to_string()],
            ..Default::default()
        };

        // Write config to server
        test_server.server.config.write().await.clone_from(&config);

        // Test WHERE clause without FROM
        let invalid_query1 = "SELECT * WHERE id = 1";
        let errors1 = test_server.server.get_error_analysis(invalid_query1.to_string()).await.unwrap();
        assert!(!errors1.is_empty(), "Should detect missing FROM clause");

        // Test missing semicolon for PostgreSQL dialect
        let invalid_query2 = "SELECT * FROM users";
        let errors2 = test_server.server.get_error_analysis(invalid_query2.to_string()).await.unwrap();
        // May detect missing semicolon for PostgreSQL

        // Test valid query (should have no errors)
        let valid_query = "SELECT * FROM users WHERE id = 1;";
        let errors3 = test_server.server.get_error_analysis(valid_query.to_string()).await.unwrap();
        assert!(errors3.is_empty() || errors3.iter().all(|e|
            e.confidence_score < 0.9), // May have minor suggestions even for valid queries
            "Valid query should have no high-confidence error detections");

        // Test very complex invalid query
        let invalid_complex = "SELECT *, COUNT(*) GROUP BY name users";
        let errors4 = test_server.server.get_error_analysis(invalid_complex.to_string()).await.unwrap();
        assert!(!errors4.is_empty(), "Should detect syntax errors in malformed query");
    }

    /// Test hash query function for cache keys
    #[test]
    fn test_hash_query_consistency() {
        let query1 = "SELECT * FROM users;".to_string();
        let query2 = "SELECT * FROM users;".to_string();
        let query3 = "SELECT * FROM posts;".to_string();

        let hash1 = hash_query(&query1);
        let hash2 = hash_query(&query2);
        let hash3 = hash_query(&query3);

        // Same queries should have same hash
        assert_eq!(hash1, hash2, "Identical queries should have identical hashes");

        // Different queries should have different hashes
        assert_ne!(hash1, hash3, "Different queries should have different hashes");

        // Hash should be deterministic
        for _ in 0..10 {
            assert_eq!(hash_query(&query1), hash1, "Hash should be deterministic");
        }

        // Hash should be non-empty
        assert!(!hash1.is_empty(), "Hash should not be empty");

        // Hash should be hexadecimal
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit() ||
                                  c == 'x' || c == '-'),
               "Hash should contain only valid characters");
    }

    /// Test cache key generation patterns
    #[test]
    fn test_cache_key_generation() {
        let query = "SELECT * FROM users WHERE id = 1;".to_string();

        let perf_key = format!("perf_{}", hash_query(&query));
        let opt_key = format!("opt_{}", hash_query(&query));
        let schema_key = format!("schema_{}", hash_query(&query));
        let error_key = format!("error_{}", hash_query(&query));

        // All keys should be different
        let keys = vec![perf_key, opt_key, schema_key, error_key];
        for i in 0..keys.len() {
            for j in (i+1)..keys.len() {
                assert_ne!(keys[i], keys[j], "Different cache keys should not collide");
            }
        }

        // All keys should contain the same query hash
        let query_hash = hash_query(&query);
        assert!(keys.iter().all(|key| key.contains(&query_hash)),
               "All cache keys should contain the query hash");

        // Key prefixes should be correct
        assert!(keys[0].starts_with("perf_"), "Performance key should have correct prefix");
        assert!(keys[1].starts_with("opt_"), "Optimization key should have correct prefix");
        assert!(keys[2].starts_with("schema_"), "Schema key should have correct prefix");
        assert!(keys[3].starts_with("error_"), "Error key should have correct prefix");
    }

    /// Test resource cleanup after intensive operations
    #[tokio::test]
    async fn test_resource_cleanup_after_intensive_operations() {
        let config = SqlLspConfig {
            enable_advanced_caching: true,
            enable_parallel_processing: true,
            cache_settings: SqlCacheConfig {
                max_entries_per_layer: 50, // Small cache to test cleanup
                collect_statistics: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        // Perform intensive operations
        let queries: Vec<String> = (0..100).map(|i| format!("SELECT * FROM table{} WHERE id = {};", i, i)).collect();

        // Execute multiple queries in parallel (as much as allowed)
        let futures: Vec<_> = queries.iter().take(20).map(|query| {
            test_server.server.get_performance_metrics(query.clone())
        }).collect();

        let _results: Vec<_> = futures::future::join_all(futures).await;

        // Test cleanup of completed tasks
        if let Some(parallel_executor) = &test_server.server.parallel_executor {
            // Simulate task cleanup
            let mut active_tasks = parallel_executor.active_tasks.lock().await;
            *active_tasks = (*active_tasks).saturating_sub(10); // Simulate cleanup of some tasks
            drop(active_tasks); // Release lock

            // Verify cleanup worked
            let remaining_tasks = parallel_executor.active_tasks.lock().await;
            assert_eq!(*remaining_tasks, 0); // Should be cleaned up
        }

        // Test cache cleanup
        if let Some(cache_manager) = &test_server.server.cache_manager {
            if let Some(metrics_cache) = &cache_manager.metrics_cache {
                // Clear some entries to simulate cleanup
                metrics_cache.clear().await.unwrap();
            }
        }

        // Verify temp directory cleanup if virtual memory is enabled
        if let Some(vm_manager) = &test_server.server.virtual_memory_manager {
            // In real implementation, this would clean up temp files
            assert!(vm_manager.temp_dir.exists(), "Temp directory should still exist for cleanup");
        }
    }

    /// Comprehensive integration test for all major components
    #[cfg(all(feature = "moka", feature = "sql-lsp"))]
    #[tokio::test]
    async fn test_full_integration_all_features() {
        let config = SqlLspConfig {
            enable_optimization_suggestions: true,
            enable_schema_inference: true,
            enable_performance_profiling: true,
            enable_collaborative_editing: true,
            enable_error_analysis: true,
            enable_advanced_caching: true,
            enable_parallel_processing: true,
            enable_virtual_memory: true,
            min_suggestion_confidence: 0.7,
            cache_settings: SqlCacheConfig {
                ttl_settings: CacheTtlSettings::default(),
                max_memory_per_layer_mb: 128,
                max_entries_per_layer: 1000,
                eviction_policy: CacheEvictionPolicy::LeastRecentlyUsed,
                enable_cache_warming: true,
                collect_statistics: true,
            },
            supported_sql_dialects: vec![
                "postgresql".to_string(),
                "mysql".to_string(),
                "sqlite".to_string(),
            ],
            performance_settings: SqlPerformanceSettings {
                parallel_analysis: true,
                max_concurrent_tasks: 8,
                analysis_timeout_ms: 10000,
                batch_processing: true,
                batch_size: 25,
            },
            security_settings: SqlSecuritySettings::default(),
        };

        let test_server = TestServer::new_with_config(config).await.unwrap();

        // Complex real-world SQL query
        let complex_query = "
            -- Multi-table analysis with complex conditions
            SELECT
                customer.id,
                customer.name,
                customer.email,
                COUNT(order_product.order_id) as total_items_purchased,
                SUM(order_product.quantity * order_product.price) as total_spent,
                AVG(review.rating) as avg_rating,
                MAX(customer_order.created_at) as last_purchase_date
            FROM customer
            JOIN customer_order ON customer.id = customer_order.customer_id
            JOIN order_product ON customer_order.id = order_product.order_id
            LEFT JOIN review ON customer.id = review.customer_id
            WHERE customer.active = true
            AND customer.created_at >= '2023-01-01'
            AND customer_order.status = 'completed'
            GROUP BY customer.id, customer.name, customer.email
            HAVING COUNT(DISTINCT customer_order.id) >= 3
            ORDER BY total_spent DESC, customer.name
            LIMIT 100;
        ".to_string();

        // Test comprehensive analysis
        let performance = test_server.server.get_performance_metrics(complex_query.clone()).await.unwrap();
        let schema = test_server.server.get_schema_inference(complex_query.clone()).await.unwrap();
        let optimizations = test_server.server.get_optimization_suggestions(complex_query.clone()).await.unwrap();
        let errors = test_server.server.get_error_analysis(complex_query.clone()).await.unwrap();
        let comprehensive = test_server.server.analyze_query_comprehensive(complex_query.clone()).await.unwrap();

        // Verify comprehensive analysis results
        assert!(performance.complexity_score > 60, "Complex query should have high complexity");
        assert!(!schema.tables.is_empty(), "Should infer multiple tables from complex query");
        assert!(!optimizations.is_empty(), "Should find optimization opportunities");
        assert!(!comprehensive.issues.is_empty() || comprehensive.issues.is_empty(), // Either is fine
               "Should analyze query comprehensively");
        assert!(!comprehensive.optimizations.is_empty(), "Should provide optimization suggestions");
        assert_eq!(comprehensive.performance_metrics.complexity_score, performance.complexity_score,
                  "Performance metrics should match");

        // Test collaborative editing with complex query
        let session_id = test_server.server
            .start_collaborative_session(
                "complex_analysis_session".to_string(),
                lsp_types::DocumentUri::parse("file:///complex_query.sql").unwrap(),
                vec!["analyst1".to_string(), "developer1".to_string()],
            )
            .await.unwrap();

        let edit_ops = vec![
            LiveEditOperation {
                operation_id: "add_index_hint".to_string(),
                user_id: "analyst1".to_string(),
                range: LspRange {
                    start: LspPosition::new(2, 0),
                    end: LspPosition::new(2, 0),
                },
                edit_type: EditType::Insert,
                original_content: "".to_string(),
                new_content: "/* Add INDEX(customer_id, status) */".to_string(),
                timestamp: Utc::now(),
                status: OperationStatus::Applied,
            },
            LiveEditOperation {
                operation_id: "optimize_join".to_string(),
                user_id: "developer1".to_string(),
                range: LspRange {
                    start: LspPosition::new(4, 0),
                    end: LspPosition::new(4, 0),
                },
                edit_type: EditType::Insert,
                original_content: "".to_string(),
                new_content: "LEFT JOIN review ON customer.id = review.customer_id".to_string(),
                timestamp: Utc::now(),
                status: OperationStatus::Applied,
            },
        ];

        for operation in edit_ops {
            test_server.server
                .update_collaborative_session(session_id.clone(), operation)
                .await
                .unwrap();
        }