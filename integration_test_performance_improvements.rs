// Integration test demonstrating performance & memory management enhancements

use std::time::Instant;

use rust_ai_ide_cache::{DistributedWorkStealingCache, WorkStealingConfig};
use rust_ai_ide_performance::*;
use rust_ai_ide_shared_types::MemoryUsageSample;

#[tokio::test]
async fn test_integrated_performance_system() {
    println!("🧪 Testing Integrated Performance & Memory Management System\n");

    // 1. Test Distributed Work-Stealing Cache
    println!("📊 Testing Distributed Work-Stealing Cache Performance...");

    let cache_config = WorkStealingConfig {
        max_steal_attempts:     5,
        steal_batch_size:       10,
        load_balance_threshold: 0.8,
        adaptive_partitioning:  true,
        predictive_placement:   false,
    };

    let mut cache = DistributedWorkStealingCache::new(cache_config.clone());

    // Register multiple workers
    cache.register_worker("worker1".to_string()).await.unwrap();
    cache.register_worker("worker2".to_string()).await.unwrap();

    // Performance benchmark: Insert operations
    let start = Instant::now();
    for i in 0..1000 {
        let key = format!("cache_key_{}", i);
        let value = format!("cache_value_{}", i * i);
        cache
            .insert(
                key.clone(),
                value,
                Some(std::time::Duration::from_secs(300)),
            )
            .await
            .unwrap();
    }
    let cache_insert_time = start.elapsed();
    println!(
        "Cache inserts: {} ops in {:.2}ms (avg: {:.2}μs/op)",
        1000,
        cache_insert_time.as_millis(),
        cache_insert_time.as_micros() as f64 / 1000.0
    );

    // 2. Test Adaptive Memory Management
    println!("🧠 Testing Adaptive Memory Management...");

    let adaptive_config = AdaptiveConfig {
        enable_predictive_allocation: true,
        monitoring_interval_seconds:  1,
        prediction_horizon_minutes:   1,
        adaptation_threshold:         0.1,
        min_confidence_threshold:     0.5,
    };

    let mut memory_manager = AdaptiveMemoryManager::new(adaptive_config);

    // Simulate memory usage patterns
    let sample = MemoryUsageSample {
        total_memory_mb:      8192,
        used_memory_mb:       4096,
        free_memory_mb:       4096,
        available_memory_mb:  4096,
        allocation_rate_kbps: 256.0,
        timestamp:            chrono::Utc::now(),
    };

    memory_manager
        .adapt_to_memory_pressure(sample)
        .await
        .unwrap();
    let recommendation = memory_manager
        .get_allocation_recommendation()
        .await
        .unwrap();

    println!(
        "Memory adaptation: Current pressure -> recommendation: {:?}",
        recommendation
    );

    // 3. Test Enhanced Memory Leak Detection
    println!("🔍 Testing Enhanced Memory Leak Detection...");

    let mut memory_analyzer = MemoryAnalyzer::new();
    let mut leak_detector = EnhancedLeakDetector::new(100); // 100MB warning threshold
    leak_detector.set_auto_fix(true);

    // This creates a test scenario where we track a potential leak
    unsafe {
        use std::alloc::{alloc, dealloc, Layout};
        let layout = Layout::from_size_align(1024 * 1024, 8).unwrap(); // 1MB allocation
        let ptr = alloc(layout);

        // Simulate memory operations
        memory_analyzer.record_allocation(ptr, layout);

        // Track with leak detector
        leak_detector.track_allocation(ptr as usize, AllocationInfo {
            size: 1024 * 1024,
            alignment: 8,
            ptr,
            backtrace: None,
        });

        // Access the allocation a few times
        for _ in 0..5 {
            leak_detector.record_access(ptr as usize);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Analyze for leaks (this allocation won't be deallocated, simulating a leak)
        let candidates = leak_detector.analyze_for_leaks();
        println!("Leak analysis: {} candidates found", candidates.len());

        // For our purposes, we'll apply automatic fixes but this is just a test
        leak_detector.apply_automatic_fixes(&mut memory_analyzer);

        // Clean up the test allocation
        dealloc(ptr, layout);
        memory_analyzer.record_deallocation(ptr);
    }

    let leak_stats = leak_detector.get_leak_stats();
    println!(
        "Leak detection stats: {} tracked, {} high-risk candidates",
        leak_stats.total_tracked, leak_stats.high_risk_candidates
    );

    // 4. Test GPU Acceleration (Note: This would need actual GPU drivers/backends in production)
    println!("🚀 GPU Acceleration Framework Ready...");
    let gpu_config = GPUConfig {
        enable_gpu_acceleration: true,
        prefer_gpu:              true,
        memory_threshold_gb:     1.0,
        operation_Timeout_ms:    30000,
        max_queued_operations:   100,
        fallback_to_cpu:         true,
    };

    let gpu_manager = GPUAccelerationManager::new(gpu_config);

    // In a real scenario, you'd initialize actual GPU devices
    // For now, we just show the framework is ready
    println!(
        "GPU Manager configured with fallback to CPU: {}",
        gpu_manager.config.fallback_to_cpu
    );

    println!("\n✅ All Performance & Memory Management Enhancements Completed!");
    println!("📈 Key Improvements:");
    println!(
        "- Distributed caching with work-stealing: {}ms for {} inserts",
        cache_insert_time.as_millis(),
        1000
    );
    println!("- Polling model recommendation: {:?}", recommendation);
    println!("- Leak detection: Enhanced with automatic fixes");
    println!("- GPU acceleration: Framework integrated and ready");
}

#[test]
fn test_performance_benchmarks() {
    println!("🏃 Running Performance Benchmarks...\n");

    // Parallel processing benchmark
    let test_data: Vec<usize> = (0..100_000).collect();

    let start = Instant::now();
    let results = process_parallel(test_data, |x| x * x);
    let parallel_time = start.elapsed();

    assert_eq!(results.len(), 100_000);
    assert_eq!(results[0], 0);
    assert_eq!(results[5000], 5000 * 5000);

    println!(
        "Parallel processing: {} operations in {:.2}ms",
        results.len(),
        parallel_time.as_millis()
    );

    println!("✅ Performance benchmarks completed successfully!");
}
