use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use rust_ai_ide_common::{IDEError, IDEErrorKind};
use rust_ai_ide_parallel_processing::{WorkStealingScheduler, MmapManager, ZeroCopyResourcePool, ZeroCopyOperation, ZeroCopyOperationType};
use rust_ai_ide_ai_inference::{MmapModel, ZeroCopyInferenceEngine, ZeroCopyModelManager};

/// Integration test suite for zero-copy operations across the pipeline
#[cfg(test)]
mod zero_copy_integration_tests {

    use super::*;

    /// Test complete zero-copy pipeline: memory mapping -> parallel processing -> AI inference
    #[tokio::test]
    async fn test_zero_copy_pipeline_integration() {
        // Setup temporary directory for testing
        let temp_dir = std::env::temp_dir().join("zero_copy_test");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        let model_path = temp_dir.join("test_model.bin");

        // Create test model data
        let model_size = 1024 * 1024; // 1MB test model
        create_test_model_data(&model_path, model_size).await.unwrap();

        // 1. Initialize zero-copy memory mapping components
        let mmap_manager = Arc::new(MmapManager::new(10));
        let zero_copy_pool = Arc::new(ZeroCopyResourcePool::new(mmap_manager.clone()));

        // 2. Setup work-stealing scheduler with zero-copy support
        let resource_manager = Arc::new(rust_ai_ide_parallel_processing::ResourcePoolManager::new(4, 1024, 10));
        let config = rust_ai_ide_parallel_processing::SchedulerConfig::default();
        let scheduler = WorkStealingScheduler::new_with_zero_copy(
            config,
            resource_manager,
            Some(mmap_manager.clone()),
            Some(zero_copy_pool.clone()),
        );

        // 3. Initialize AI model manager with zero-copy support
        let model_manager = Arc::new(ZeroCopyModelManager::new(512)); // 512MB max memory

        // 4. Load model with memory mapping
        let model_key = "test_model".to_string();
        model_manager.load_model(model_key.clone(), model_path.clone()).await.unwrap();

        // 5. Execute zero-copy inference pipeline
        let inference_engine = model_manager.get_inference_engine(&model_key).await.unwrap();

        // Create test input data
        let input_data = b"Hello World! This is test input for zero-copy inference.";

        // Measure performance baseline (regular execution)
        let baseline_start = Instant::now();
        let baseline_result = inference_engine.infer_zero_copy(input_data).await.unwrap();
        let baseline_duration = baseline_start.elapsed();

        // Verify results
        assert!(!baseline_result.is_empty());
        assert!(baseline_result.len() > input_data.len()); // Processed output should be larger

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
        println!("✓ Zero-copy pipeline integration test passed in {:?}", baseline_duration);
    }

    /// Test parallel processing with memory-mapped files under load
    #[tokio::test]
    async fn test_parallel_processing_load_test() {
        let temp_dir = std::env::temp_dir().join("parallel_load_test");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Create multiple test files
        let file_count = 5;
        let file_size = 512 * 1024; // 512KB each
        let mut test_files = Vec::new();

        for i in 0..file_count {
            let file_path = temp_dir.join(format!("test_file_{}.bin", i));
            create_test_data(&file_path, file_size, i as u8).await.unwrap();
            test_files.push(file_path);
        }

        // Initialize components
        let mmap_manager = Arc::new(MmapManager::new(20));
        let zero_copy_pool = Arc::new(ZeroCopyResourcePool::new(mmap_manager.clone()));
        let resource_manager = Arc::new(rust_ai_ide_parallel_processing::ResourcePoolManager::new(8, 2048, 20));

        let config = rust_ai_ide_parallel_processing::SchedulerConfig::default();
        let scheduler = WorkStealingScheduler::new_with_zero_copy(
            config,
            resource_manager,
            Some(mmap_manager.clone()),
            Some(zero_copy_pool.clone()),
        );

        scheduler.start().unwrap();

        // Submit parallel zero-copy operations
        let mut handles = Vec::new();

        for file_path in test_files {
            let zero_copy_pool = zero_copy_pool.clone();

            let handle = tokio::spawn(async move {
                // Perform batch zero-copy operations
                let operations = vec![
                    ZeroCopyOperation {
                        id: "read_op".to_string(),
                        path: file_path.clone(),
                        offset: 0,
                        size: 256 * 1024,
                        op_type: ZeroCopyOperationType::Read,
                    },
                    ZeroCopyOperation {
                        id: "transform_op".to_string(),
                        path: file_path.clone(),
                        offset: 256 * 1024,
                        size: 256 * 1024,
                        op_type: ZeroCopyOperationType::Transform(Box::new(|data: &[u8]| {
                            data.iter().map(|&b| b.wrapping_add(1)).collect()
                        })),
                    },
                ];

                let results = zero_copy_pool.advanced_ops.batch_process_files(operations).await.unwrap();
                assert_eq!(results.len(), 2);
                assert!(results.iter().all(|r| r.success));
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify memory usage tracking
        let active_buffers = zero_copy_pool.get_active_buffers().await;
        assert!(active_buffers.is_empty()); // All buffers should be cleaned up

        let memory_usage = zero_copy_pool.get_total_memory_usage().await;
        assert_eq!(memory_usage, 0); // All memory should be released

        scheduler.stop().unwrap();

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
        println!("✓ Parallel processing load test completed successfully");
    }

    /// Test memory-mapped AI model loading and concurrent inference
    #[tokio::test]
    async fn test_ai_model_zero_copy_concurrent_inference() {
        let temp_dir = std::env::temp_dir().join("ai_model_test");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        let model_path = temp_dir.join("concurrent_model.bin");

        // Create test model
        let model_size = 2 * 1024 * 1024; // 2MB model
        create_test_model_data(&model_path, model_size).await.unwrap();

        // Initialize model manager
        let model_manager = Arc::new(ZeroCopyModelManager::new(1024)); // 1GB max memory

        // Load model
        let model_key = "concurrent_model".to_string();
        model_manager.load_model(model_key.clone(), model_path.clone()).await.unwrap();

        // Test concurrent inference
        let inference_engine = model_manager.get_inference_engine(&model_key).await.unwrap().clone();

        // Create multiple concurrent requests
        let request_count = 10;
        let mut handles = Vec::new();
        let semaphore = Arc::new(Semaphore::new(4)); // Limit concurrency to 4

        for i in 0..request_count {
            let inference_engine = inference_engine.clone();
            let semaphore = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                let input_data = format!("Test input data batch {}", i).into_bytes();
                let start = Instant::now();

                let result = inference_engine.infer_zero_copy(&input_data).await.unwrap();
                let duration = start.elapsed();

                assert!(!result.is_empty());
                assert!(duration < Duration::from_secs(1)); // Reasonable performance bound

                result
            });

            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Verify all results
        assert_eq!(results.len(), request_count);
        assert!(results.iter().all(|r| !r.is_empty()));

        // Verify model memory state
        let current_memory = model_manager.get_current_memory_usage().await;
        assert!(current_memory <= 4 * 1024 * 1024); // Should not exceed reasonable bounds

        let loaded_models = model_manager.get_loaded_models().await;
        assert_eq!(loaded_models.len(), 1);
        assert_eq!(loaded_models[0], model_key);

        // Cleanup
        model_manager.unload_model(&model_key).await.unwrap();
        let final_memory = model_manager.get_current_memory_usage().await;
        assert_eq!(final_memory, 0);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
        println!("✓ AI model zero-copy concurrent inference test passed");
    }

    /// Test memory pressure and cleanup mechanisms
    #[tokio::test]
    async fn test_memory_pressure_and_cleanup() {
        let temp_dir = std::env::temp_dir().join("memory_pressure_test");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Initialize components with limited memory
        let mmap_manager = Arc::new(MmapManager::new(5));
        let zero_copy_pool = Arc::new(ZeroCopyResourcePool::new(mmap_manager.clone()));

        let manager = zero_copy_pool.manager.clone();

        // Simulate memory pressure by creating multiple large allocations
        let mut allocated_files = Vec::new();
        let allocation_size = 10 * 1024 * 1024; // 10MB each - will cause pressure with 5 concurrent limit

        for i in 0..6 { // Try to allocate 6 files (more than limit of 5)
            let file_path = temp_dir.path().join(format!("large_file_{}.bin", i));
            let result = manager.create_mmap_file(file_path, allocation_size).await;

            if i < 5 {
                // First 5 should succeed
                assert!(result.is_ok());
                allocated_files.push(result.unwrap());
            } else {
                // 6th should fail due to concurrency limit
                assert!(result.is_err());
                match result.err().unwrap() {
                    IDEError::WrappedError(_) => {
                        // Expected error due to semaphore limit
                    }
                    _ => panic!("Expected semaphore acquire error"),
                }
            }
        }

        // Test cleanup performance under memory pressure
        let cleanup_start = Instant::now();
        let mut cleanup_count = 0;

        for file_id in allocated_files {
            manager.remove_mmap(&file_id).await.unwrap();
            cleanup_count += 1;
        }

        let cleanup_duration = cleanup_start.elapsed();
        assert!(cleanup_duration < Duration::from_millis(500)); // Cleanup should be fast
        assert_eq!(cleanup_count, 5);

        // Verify all memory cleaned up
        let active_buffers = zero_copy_pool.get_active_buffers().await;
        assert!(active_buffers.is_empty());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
        println!("✓ Memory pressure and cleanup test completed in {:?}", cleanup_duration);
    }

    /// Test error handling and recovery in zero-copy operations
    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let temp_dir = std::env::temp_dir().join("error_handling_test");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Setup components
        let mmap_manager = Arc::new(MmapManager::new(10));
        let zero_copy_pool = Arc::new(ZeroCopyResourcePool::new(mmap_manager.clone()));

        // Test 1: Non-existent file handling
        let nonexistent_path = temp_dir.path().join("nonexistent_file.bin");
        let result = mmap_manager.create_mmap_file(nonexistent_path, 1024).await;
        assert!(result.is_err());

        match result.err().unwrap() {
            IDEError::WrappedError(_) => {
                // Expected file not found error
            }
            _ => panic!("Expected file operation error"),
        }

        // Test 2: Invalid operations on nonexistent buffers
        let invalid_access = zero_copy_pool.advanced_ops.access_segment("invalid_id").await;
        assert!(invalid_access.is_err());

        // Test 3: Recovery and continued operation
        let valid_file_path = temp_dir.path().join("recovery_test.bin");
        create_test_data(&valid_file_path, 1024, 0).await.unwrap();

        let file_id = mmap_manager.create_mmap_file(valid_file_path, 1024).await.unwrap();

        // Verify normal operation continues after errors
        let segment_data = zero_copy_pool.advanced_ops.access_segment(&file_id).await.unwrap();
        assert_eq!(segment_data.len(), 1024);

        // Cleanup
        mmap_manager.remove_mmap(&file_id).await.unwrap();

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
        println!("✓ Error handling and recovery test passed");
    }

    // Helper functions

    async fn create_test_model_data(path: &std::path::Path, size: usize) -> std::io::Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(path).await?;
        let header = b"MODL"; // Simple magic header
        let version = [1u8, 0u8, 0u8, 0u8]; // Version info
        let data_size = (size as u32).to_le_bytes();

        file.write_all(header).await?;
        file.write_all(&version).await?;
        file.write_all(&data_size).await?;

        // Fill with test data
        let remaining = size - 12; // Header size
        let pattern = b"RUST_ZERO_COPY_MODEL_TEST_DATA";
        while file.metadata().await?.len() < size as u64 {
            let to_write = std::cmp::min(pattern.len(), remaining);
            file.write_all(&pattern[..to_write]).await?;
        }

        file.flush().await?;
        Ok(())
    }

    async fn create_test_data(path: &std::path::Path, size: usize, fill_byte: u8) -> std::io::Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(path).await?;
        let data = vec![fill_byte; size];
        file.write_all(&data).await?;
        file.flush().await?;
        Ok(())
    }
}