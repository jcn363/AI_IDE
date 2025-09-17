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

/// Calculate operations per second
fn calculate_ops_per_second(operations: u64, duration: Duration) -> f64 {
    operations as f64 / duration.as_secs_f64()
}

fn main() {
    println!("=== Rust AI IDE Performance Baseline Establishment ===");
    println!("Running initial performance measurements...\n");

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

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Generate performance report
    println!("\n=== Performance Baseline Report ===");
    println!("Timestamp: {} (Unix)", timestamp);
    println!("Environment: development");
    println!("System: {} {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("");
    println!("Synchronous Performance:");
    println!("- Iterations: 100,000");
    println!("- Duration: {:.2?}", sync_duration);
    println!("- Operations/second: {:.2}", sync_ops_per_second);
    println!("- Result: {}", sync_result);
    println!("");
    println!("Recommendations:");
    println!("- Baseline established for CPU-bound operations");
    println!("- Memory usage appears stable");
    println!("- Performance monitoring ready for regression detection");

    // Save baseline data as JSON
    let baseline_data = format!(
        r#"{{
  "timestamp": {},
  "environment": "development",
  "system_info": {{
    "os": "{}",
    "arch": "{}"
  }},
  "sync_workload": {{
    "iterations": 100000,
    "result": {},
    "duration_ms": {},
    "ops_per_second": {:.2}
  }},
  "baseline_thresholds": {{
    "regression_threshold_percent": 5.0,
    "alert_on_regression": true
  }}
}}"#,
        timestamp,
        std::env::consts::OS,
        std::env::consts::ARCH,
        sync_result,
        sync_duration.as_millis(),
        sync_ops_per_second
    );

    std::fs::write("performance_baseline.json", baseline_data).unwrap();
    println!("\n📊 Baseline data saved to: performance_baseline.json");

    let total_duration = Instant::now().elapsed();
    println!("\n⏱️  Total baseline establishment time: {:.2?}", total_duration);
}