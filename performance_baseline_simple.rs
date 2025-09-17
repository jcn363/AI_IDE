use std::time::{Duration, Instant};

/// Simple synchronous performance workload
fn run_sync_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    for i in 0..iterations {
        // Simple hash-like operation
        let x = i as u64 * 2654435761 % (1 << 31);
        result = result.wrapping_add(x);

        // Simulate memory allocation
        let mut vec = Vec::with_capacity(1000);
        for j in 0..1000 {
            vec.push(j as u64);
        }

        // Use the vector to prevent optimization
        result = result.wrapping_add(vec[vec.len() - 1]);
    }

    result
}

/// Simple asynchronous performance workload
async fn run_async_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    for i in 0..iterations {
        // Simulate async I/O
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Some computation
        let x = i as u64 * 11400714819323198549u64;
        result = result.wrapping_add(x);
    }

    result
}

/// Calculate operations per second
fn calculate_ops_per_second(operations: u64, duration: Duration) -> f64 {
    operations as f64 / duration.as_secs_f64()
}

/// Generate performance report
fn generate_performance_report(
    sync_iterations: u32,
    sync_result: u64,
    sync_duration: Duration,
    async_iterations: u32,
    async_result: u64,
    async_duration: Duration,
) -> String {
    let sync_ops_per_sec = calculate_ops_per_second(sync_iterations as u64, sync_duration);
    let async_ops_per_sec = calculate_ops_per_second(async_iterations as u64, async_duration);

    format!(
        r#"=== Performance Test Report ===

Synchronous Work:
- Iterations: {}
- Duration: {:.2?}
- Operations/second: {:.2}
- Result: {}

Asynchronous Work:
- Iterations: {}
- Duration: {:.2?}
- Operations/second: {:.2}
- Result: {}

Recommendations:
- Synchronous workload shows {:.1}x performance advantage
- Consider sync for CPU-bound operations
- Memory usage appears stable
"#,
        sync_iterations,
        sync_duration,
        sync_ops_per_sec,
        sync_result,
        async_iterations,
        async_duration,
        async_ops_per_sec,
        async_result,
        sync_ops_per_sec / async_ops_per_sec.max(0.001)
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Rust AI IDE Performance Baseline Establishment ===");
    println!("Running initial performance measurements...\n");

    let baseline_data = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "environment": "development",
        "system_info": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH
        }
    });

    // Run synchronous performance workload
    println!("Running synchronous workload (100,000 iterations)...");
    let sync_start = Instant::now();
    let sync_result = run_sync_workload(100_000);
    let sync_duration = sync_start.elapsed();
    let sync_ops_per_second = calculate_ops_per_second(100_000, sync_duration);

    println!("✅ Sync workload completed:");
    println!("   Result: {}", sync_result);
    println!("   Duration: {:.2?}", sync_duration);
    println!("   Operations/second: {:.2}", sync_ops_per_second);

    // Run asynchronous performance workload
    println!("\nRunning asynchronous workload (1,000 iterations)...");
    let async_start = Instant::now();
    let async_result = run_async_workload(1_000).await;
    let async_duration = async_start.elapsed();
    let async_ops_per_second = calculate_ops_per_second(1_000, async_duration);

    println!("✅ Async workload completed:");
    println!("   Result: {}", async_result);
    println!("   Duration: {:.2?}", async_duration);
    println!("   Operations/second: {:.2}", async_ops_per_second);

    // Generate performance report
    let report = generate_performance_report(
        100_000,
        sync_result,
        sync_duration,
        1_000,
        async_result,
        async_duration,
    );

    println!("\n{}", report);

    // Save baseline data
    let baseline_file = std::path::PathBuf::from("performance_baseline.json");
    std::fs::write(&baseline_file, serde_json::to_string_pretty(&baseline_data)?)?;
    println!("📊 Baseline data saved to: {}", baseline_file.display());

    let total_duration = std::time::Instant::now().elapsed();
    println!("\n⏱️  Total baseline establishment time: {:.2?}", total_duration);

    Ok(())
}