/*!!

Integration tests for rust-ai-ide-common using shared-test-utils

This module demonstrates shared functionality testing patterns using the comprehensive
test utilities from shared-test-utils, including:

- Common utility functions and helpers
- Configuration management testing
- Performance and caching utilities
- Concurrent operations and synchronization
- Serialization and data management
- Error handling and validation patterns
- Workspace-wide utility testing

*/

use shared_test_utils::async_utils::AsyncContext;
use shared_test_utils::error::TestResult;
use shared_test_utils::fixtures::FixturePresets;
use shared_test_utils::*;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

// Test that demonstrates common functionality integration
#[cfg(test)]
mod common_integration_tests {
    use super::*;
    use std::fs;

    /// Integration test demonstrating common workspace setup patterns
    #[test]
    fn test_common_workspace_setup_with_shared_utils() {
        println!("🔧 Setting up common integration test with shared utilities...");

        // Create a temp workspace for common functionality testing
        let workspace = TempWorkspace::new().unwrap();

        // Set up common workspace structure
        workspace.setup_basic_project().unwrap();
        workspace.create_dir(Path::new("config")).unwrap();
        workspace.create_dir(Path::new("cache")).unwrap();
        workspace.create_dir(Path::new("logs")).unwrap();

        // Create common configuration files
        workspace
            .create_file(
                Path::new("config/app.json"),
                r#"{
  "application": {
    "name": "Rust AI IDE Common",
    "version": "0.1.0",
    "features": ["caching", "logging", "serialization"]
  },
  "performance": {
    "cache_enabled": true,
    "async_operations": true,
    "concurrent_workers": 4,
    "memory_limit_mb": 512
  },
  "logging": {
    "level": "info",
    "format": "json",
    "max_file_size_mb": 10,
    "retention_days": 7
  }
}"#,
            )
            .unwrap();

        workspace
            .create_file(
                Path::new("cache/manifest.json"),
                r#"{
  "cache_version": "1.0.0",
  "entries": {
    "config": "2024-01-15T10:30:00Z",
    "modules": "2024-01-15T09:45:00Z",
    "metadata": "2024-01-14T16:20:00Z"
  },
  "stats": {
    "total_entries": 3,
    "total_size_mb": 45.2,
    "hit_rate_percent": 87.5,
    "miss_rate_percent": 12.5
  }
}"#,
            )
            .unwrap();

        workspace
            .create_file(
                Path::new("logs/application.log"),
                r#"2024-01-15 10:00:00 INFO Application started successfully
2024-01-15 10:05:15 INFO Cache initialized with 45.2 MB
2024-01-15 10:15:30 INFO Concurrent workers pool created (4 workers)
2024-01-15 10:30:22 INFO Configuration loaded successfully
2024-01-15 11:00:00 INFO Periodic cleanup completed
"#,
            )
            .unwrap();

        // Test common file operations
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("config/app.json"));
        assert_test_file_exists!(workspace, Path::new("cache/manifest.json"));
        assert_test_file_exists!(workspace, Path::new("logs/application.log"));

        // Test content validation for common patterns
        assert_file_contains!(workspace, Path::new("config/app.json"), "caching");
        assert_file_contains!(workspace, Path::new("cache/manifest.json"), "cache_version");
        assert_file_contains!(workspace, Path::new("logs/application.log"), "INFO");

        // Validate comprehensive structure
        let total_files = fs::read_dir(workspace.path()).unwrap().count();
        assert!(
            total_files >= 6,
            "Should have at least 6 files (Cargo.toml + project files + common structure)"
        );

        println!("✅ Common workspace setup completed successfully");
    }

    /// Integration test using fixtures for common functionality scenarios
    #[test]
    fn test_common_functionality_with_fixtures() {
        println!("🔧 Testing common functionality with fixture scenarios...");

        // Use fixture for consistent common functionality environment
        let (workspace, fixture) = with_test_fixture!(FixturePresets::rust_library());

        // Extend with common-specific configuration files
        workspace
            .create_file(
                Path::new("workspace.json"),
                r#"{
  "workspace": {
    "name": "rust-ai-ide-workspace",
    "version": "0.1.0",
    "description": "Integration workspace for common functionality testing"
  },
  "dependencies": {
    "rust-ai-ide-core": "^0.1.0",
    "rust-ai-ide-common": "^0.1.0",
    "serde": "^1.0",
    "tokio": "^1.4"
  },
  "features": {
    "performance": ["async", "caching", "logging"],
    "stability": ["robust_errors", "validation"],
    "development": ["debug_mode", "verbose_logging"]
  },
  "build": {
    "target": "release",
    "optimization_level": "3",
    "link_time_optimization": true
  }
}"#,
            )
            .unwrap();

        workspace
            .create_file(
                Path::new("build_cache.json"),
                r#"{
  "build_cache": {
    "version": "cache-v2",
    "last_clean": "2024-01-15T06:00:00Z",
    "entries": [
      {
        "module": "core",
        "timestamp": "2024-01-15T10:00:00Z",
        "size_mb": 12.5,
        "dependencies": ["serde", "tokio"]
      },
      {
        "module": "common",
        "timestamp": "2024-01-15T10:45:00Z",
        "size_mb": 8.9,
        "dependencies": ["dashmap", "log"]
      }
    ],
    "statistics": {
      "cache_hit_rate": 0.78,
      "average_build_time_ms": 3200,
      "total_cached_mb": 45.2,
      "validity_period_hours": 24
    }
  }
}"#,
            )
            .unwrap();

        // Verify fixture integration
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("workspace.json"));
        assert_test_file_exists!(workspace, Path::new("build_cache.json"));

        // Test fixture content access
        let workspace_config_content = fixture
            .get_file_content(&Path::new("workspace.json").to_path_buf())
            .unwrap();
        assert!(workspace_config_content.contains("workspace"));
        assert!(workspace_config_content.contains("dependencies"));
        assert!(workspace_config_content.contains("tokio"));

        println!("✅ Common functionality fixtures integration test passed");
    }

    /// Performance-critical common operations with timeout handling
    #[tokio::test]
    async fn test_common_operations_with_timeout() {
        println!("🔧 Testing common operations with timeout handling...");

        let context = AsyncContext::with_timeout(Duration::from_secs(10));

        // Test common async operation patterns with timeout
        let result = with_timeout(
            async {
                // Simulate common operation (like caching, logging, serialization)
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                serde_json::json!({
                    "operation": "common_task",
                    "timestamp": "2024-01-15T10:30:00Z",
                    "duration_ms": 50,
                    "status": "success",
                    "resources_used": {
                        "memory_mb": 2.5,
                        "cpu_percent": 15.7,
                        "threads": 1
                    }
                })
            },
            Duration::from_millis(200),
        )
        .await;

        assert!(result.is_ok());
        let task_result: serde_json::Value = result.unwrap();
        assert_eq!(task_result["operation"], "common_task");
        assert_eq!(task_result["status"], "success");
        assert!(task_result["resources_used"]["memory_mb"].as_f64().unwrap() > 0.0);

        println!("✅ Common operations timeout test passed");
    }

    /// Complex common functionality workflows with concurrency
    #[tokio::test]
    async fn test_complex_common_workflows() {
        println!("🔧 Testing complex common workflows with multiple operations...");

        // Create workspace and set up complex common functionality scenario
        let context = AsyncContext::with_timeout(Duration::from_secs(30));
        let workspace = TempWorkspace::new().unwrap();

        // Set up comprehensive common infrastructure
        workspace.create_dir(Path::new("performance")).unwrap();
        workspace.create_dir(Path::new("serialization")).unwrap();
        workspace.create_dir(Path::new("caching")).unwrap();
        workspace.create_dir(Path::new("logs")).unwrap();

        // Simulate concurrent common functionality operations
        async fn simulate_common_operation(operation_type: &str) -> Result<String, TestError> {
            Ok(format!("common_{}_operation_completed", operation_type))
        }

        // Test multiple concurrent common operations (caching, logging, serialization, etc.)
        let common_operations = vec![
            simulate_common_operation("caching"),
            simulate_common_operation("serialization"),
            simulate_common_operation("logging"),
            simulate_common_operation("validation"),
            simulate_common_operation("performance"),
            simulate_common_operation("configuration"),
        ];

        let results = context
            .execute_concurrent(common_operations, Some(Duration::from_millis(300)))
            .await;

        assert!(results.is_ok());
        let result_strings = results.unwrap();
        assert_eq!(result_strings.len(), 6);

        // Verify each common operation completed successfully
        let mut operation_types = std::collections::HashMap::new();
        for result in &result_strings {
            if let Ok(content) = result {
                assert!(content.starts_with("common_"));
                assert!(content.contains("_operation_completed"));

                // Count operations by type
                if content.contains("caching")
                    || content.contains("serialization")
                    || content.contains("logging")
                    || content.contains("validation")
                    || content.contains("performance")
                    || content.contains("configuration")
                {
                    let op_type = content.split('_').nth(1).unwrap_or("unknown");
                    *operation_types.entry(op_type).or_insert(0) += 1;
                }
            } else {
                assert!(false, "Common operation failed with error: {:?}", result);
            }
        }

        // Verify we have tests for different types of common operations
        assert!(
            operation_types.len() >= 4,
            "Should have results for at least 4 operation types"
        );
        assert!(
            operation_types.values().map(|&v| v as i32).sum::<i32>() <= 8,
            "Should have reasonable operation counts"
        ); // Allow for duplicate types

        println!(
            "✅ Complex common workflows test completed successfully - {} operations verified",
            operation_types.values().sum::<i32>()
        );
    }

    /// Integration test for common error handling and validation
    #[test]
    fn test_common_error_handling_and_validation() {
        println!("🔧 Testing common error handling and validation patterns...");

        let workspace = TempWorkspace::new().unwrap();

        // Test common file operation error handling
        let invalid_config_result = std::fs::write(
            workspace.path().join("invalid_config.json"),
            r#"{
  "invalid": json syntax here ### missing closing }
}"#,
        );

        if let Err(e) = invalid_config_result {
            let test_error = TestError::Io(e);
            assert!(matches!(test_error, TestError::Io(_)));
        }

        // Test common path validation patterns
        let non_existent_path = Path::new("/invalid/common/path/does/not/exist");
        let path_validation_result = ValidationUtils::validate_path_security(non_existent_path);
        assert!(path_validation_result.is_err());

        // Test common component validation patterns
        let common_components = vec![
            Some("caching"),
            Some("logging"),
            None, // Missing optional component
        ];
        let component_names = vec!["Caching", "Logging", "Metrics"];

        assert!(
            ValidationUtils::validate_test_setup(&common_components, &component_names).is_err()
        );

        // Valid common component setup
        let valid_common_components = vec![Some("caching"), Some("logging"), Some("metrics")];
        assert!(
            ValidationUtils::validate_test_setup(&valid_common_components, &component_names)
                .is_ok()
        );

        println!("✅ Common error handling and validation test completed");
    }

    /// Command integration testing for common functionality patterns
    #[test]
    fn test_common_command_integration_patterns() {
        println!("🔧 Testing common command integration patterns...");

        use shared_test_utils::command_tests::{CommandTestBuilder, MockCommand};

        // Create mock commands for common functionality operations
        let commands = vec![
            MockCommand::new(
                "initialize_workspace",
                serde_json::json!({
                    "workspace_path": "/project",
                    "config_file": "workspace.json",
                    "features": ["caching", "logging", "async"]
                }),
            )
            .with_result(serde_json::json!({
                "workspace_initialized": true,
                "config_loaded": true,
                "features_enabled": ["caching", "logging", "async"],
                "initialization_time_ms": 75,
                "status": "ready"
            })),
            MockCommand::new(
                "cache_operations",
                serde_json::json!({
                    "operation": "refresh",
                    "cache_type": "all",
                    "force_invalidation": false
                }),
            )
            .with_result(serde_json::json!({
                "cache_operation": "refresh",
                "entries_refreshed": 15,
                "entries_invalidated": 0,
                "cache_size_mb": 42.7,
                "operation_time_ms": 120
            })),
            MockCommand::new(
                "performance_monitoring",
                serde_json::json!({
                    "metrics": ["cpu_usage", "memory_usage", "io_operations"],
                    "duration_seconds": 60,
                    "interval_ms": 1000
                }),
            )
            .with_result(serde_json::json!({
                "monitoring_started": true,
                "metrics_collected": ["cpu_usage", "memory_usage", "io_operations"],
                "avg_cpu_percent": 23.4,
                "avg_memory_mb": 89.2,
                "total_io_operations": 1456,
                "avg_response_time_ms": 12.3
            })),
            MockCommand::new(
                "serialize_configuration",
                serde_json::json!({
                    "target_file": "config/config.json",
                    "format": "json",
                    "version_compatibility": "stable"
                }),
            )
            .with_result(serde_json::json!({
                "serialization_complete": true,
                "file_written": "config/config.json",
                "bytes_written": 2048,
                "compression_enabled": false,
                "validation_passed": true
            })),
        ];

        // Test common command setup
        let runner = CommandTestBuilder::new()
            .success_command(
                "validate_workspace",
                serde_json::json!({}),
                serde_json::json!({"workspace_valid": true, "issues_found": 0}),
            )
            .error_command(
                "invalid_common_operation",
                serde_json::json!({}),
                "Common operation requires valid configuration",
            )
            .build_runner();

        // Verify common commands were registered correctly
        assert_eq!(commands[0].name, "initialize_workspace");
        assert!(commands[0].result.is_ok());

        assert_eq!(commands[1].name, "cache_operations");
        assert!(commands[1].result.is_ok());

        assert_eq!(commands[2].name, "performance_monitoring");
        assert!(commands[2].result.is_ok());

        assert_eq!(commands[3].name, "serialize_configuration");
        assert!(commands[3].result.is_ok());

        // Verify common command tester is set up
        assert_eq!(runner.called_commands().len(), 0);

        println!("✅ Common command integration patterns test completed");
    }

    /// Concurrent common operations with resource management
    #[tokio::test]
    async fn test_concurrent_common_operations() {
        println!("🔧 Testing concurrent common operations with resource management...");

        // Create a mechanism for testing multiple common operations concurrently
        async fn simulate_common_operation(
            operation_type: &str,
            resource_id: usize,
        ) -> Result<String, TestError> {
            // Simulate varying execution times for different common operations
            let base_time = match operation_type {
                "cache" => 30,
                "serialize" => 40,
                "validate" => 25,
                "log" => 20,
                _ => 35,
            };

            let total_time = base_time + (resource_id as u64 * 5);
            let result = with_timeout(
                async {
                    tokio::time::sleep(Duration::from_millis(total_time)).await;
                    format!(
                        "common_{}_resource_{}_{}ms",
                        operation_type, resource_id, total_time
                    )
                },
                Duration::from_millis(total_time + 10),
            )
            .await;

            match result {
                Ok(value) => Ok(value),
                Err(_) => Ok(format!(
                    "common_{}_resource_{}_timeout",
                    operation_type, resource_id
                )),
            }
        }

        // Test multiple concurrent common operations running simultaneously
        let context = AsyncContext::with_timeout(Duration::from_secs(10));

        // Create batch of common operations
        let common_batch = vec![
            (1, "cache"),
            (2, "cache"),
            (1, "serialize"),
            (2, "serialize"),
            (1, "validate"),
            (2, "validate"),
            (1, "log"),
            (2, "log"),
        ];

        // Convert to futures
        let mut common_operations = vec![];
        for (resource_id, operation_type) in common_batch {
            common_operations.push(simulate_common_operation(operation_type, resource_id));
        }

        let results = context
            .execute_concurrent(
                common_operations,
                Some(Duration::from_millis(150)), // Appropriate timeout for common operations
            )
            .await;

        assert!(results.is_ok());
        let result_values = results.unwrap();
        assert_eq!(result_values.len(), 8);

        // Verify common operations by type and resource
        let mut operation_counts = std::collections::HashMap::new();
        let mut resource_ids = Vec::new();

        for result in &result_values {
            if let Ok(content) = result {
                assert!(content.starts_with("common_"));
                assert!(content.contains("_resource_") || content.contains("_timeout"));

                // Extract operation type
                if let Some(op_type) = content.split('_').nth(1) {
                    *operation_counts.entry(op_type.to_string()).or_insert(0) += 1;

                    // Extract resource ID
                    if let Some(resource_str) = content.split("_resource_").nth(1) {
                        if let Some(id_str) = resource_str.split('_').next() {
                            if let Ok(resource_id) = id_str.parse::<usize>() {
                                resource_ids.push(resource_id);
                            }
                        }
                    }
                }
            } else {
                assert!(false, "Common operation failed: {:?}", result);
            }
        }

        // Verify operation distribution
        assert!(operation_counts.contains_key("cache"));
        assert!(operation_counts.contains_key("serialize"));
        assert!(operation_counts.contains_key("validate"));
        assert!(operation_counts.contains_key("log"));

        // Verify resource distribution (should have resources 1 and 2)
        resource_ids.sort();
        resource_ids.dedup();
        assert!(!resource_ids.is_empty());
        assert!(resource_ids.contains(&1) || resource_ids.contains(&2));

        println!(
            "✅ Concurrent common operations test completed - {} operations across {} types",
            operation_counts.values().sum::<i32>(),
            operation_counts.len()
        );
    }

    /// Resource management and lifecycle testing for common operations
    #[test]
    fn test_flow_control_and_resource_management() {
        println!("🔧 Testing flow control and resource management in common operations...");

        let workspace = TempWorkspace::new().unwrap();

        // Create resource tracking and management scenario
        workspace.create_dir(Path::new("resources")).unwrap();
        workspace.create_dir(Path::new("locks")).unwrap();
        workspace.create_dir(Path::new("state")).unwrap();

        // Create resource allocation configuration
        workspace
            .create_file(
                Path::new("resources/allocation_config.json"),
                r#"{
  "resource_allocation": {
    "memory_pools": [
      {
        "name": "cache_pool",
        "size_mb": 64,
        "priority": "high",
        "allocation_strategy": "lru"
      },
      {
        "name": "log_pool",
        "size_mb": 32,
        "priority": "medium",
        "allocation_strategy": "fifo"
      }
    ],
    "worker_threads": {
      "cache_workers": 2,
      "io_workers": 4,
      "validation_workers": 1
    },
    "rate_limits": {
      "requests_per_second": 100,
      "burst_allowance": 50,
      "backoff_multiplier": 2.0
    }
  }
}"#,
            )
            .unwrap();

        // Create flow control state
        workspace
            .create_file(
                Path::new("state/flow_control.json"),
                r#"{
  "flow_states": {
    "idle": { "operations_pending": 0, "resources_available": true },
    "normal": { "operations_pending": 3, "resources_available": true, "load_percent": 45 },
    "high_load": { "operations_pending": 12, "resources_available": false, "load_percent": 78 },
    "overload": { "operations_pending": 25, "resources_available": false, "load_percent": 95 }
  },
  "current_state": "normal",
  "flow_control": {
    "congestion_window": 10,
    "slow_start_threshold": 16,
    "retransmission_timeout_ms": 200
  },
  "resource_management": {
    "active_connections": 8,
    "connection_pool_size": 20,
    "idle_timeout_seconds": 300,
    "max_retries": 3
  }
}"#,
            )
            .unwrap();

        // Create lock and synchronization tracking
        workspace
            .create_file(
                Path::new("locks/resource_locks.json"),
                r#"{
  "active_locks": [
    {
      "resource_id": "cache_pool",
      "holder": "worker_1",
      "timestamp": "2024-01-15T10:30:22Z",
      "timeout_ms": 5000,
      "exclusive": true
    },
    {
      "resource_id": "file_access",
      "holder": "worker_3",
      "timestamp": "2024-01-15T10:30:20Z",
      "timeout_ms": 3000,
      "exclusive": false
    }
  ],
  "lock_statistics": {
    "total_locks_issued": 147,
    "average_hold_time_ms": 1240,
    "deadlock_preventions": 3,
    "wait_queue_length": 2
  }
}"#,
            )
            .unwrap();

        // Test resource management file operations
        assert_test_file_exists!(workspace, Path::new("resources/allocation_config.json"));
        assert_test_file_exists!(workspace, Path::new("state/flow_control.json"));
        assert_test_file_exists!(workspace, Path::new("locks/resource_locks.json"));

        // Verify resource management configuration
        let allocation_config =
            fs::read_to_string(workspace.path().join("resources/allocation_config.json")).unwrap();
        assert!(allocation_config.contains("memory_pools"));
        assert!(allocation_config.contains("cache_pool"));
        assert!(allocation_config.contains("log_pool"));

        let flow_control_state =
            fs::read_to_string(workspace.path().join("state/flow_control.json")).unwrap();
        assert!(flow_control_state.contains("flow_control"));
        assert!(flow_control_state.contains("congestion_window"));

        let lock_state =
            fs::read_to_string(workspace.path().join("locks/resource_locks.json")).unwrap();
        assert!(lock_state.contains("active_locks"));
        assert!(lock_state.contains("worker_1"));

        println!("✅ Resource management and flow control test completed");
    }
}
