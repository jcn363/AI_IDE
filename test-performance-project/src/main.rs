use std::time::Instant;

use test_performance_project::{do_async_work, do_some_work};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    println!("Starting performance test...");

    // Test sync work
    let sync_iterations = 100_000;
    let sync_start = Instant::now();
    let sync_result = do_some_work(sync_iterations);
    let sync_duration = sync_start.elapsed();

    println!(
        "Synchronous work ({} iterations) took: {:.2?}",
        sync_iterations, sync_duration
    );
    println!("Result: {}", sync_result);

    // Test async work
    let async_iterations = 10;
    let async_start = Instant::now();
    let async_result = do_async_work(async_iterations).await;
    let async_duration = async_start.elapsed();

    println!(
        "\nAsynchronous work ({} iterations) took: {:.2?}",
        async_iterations, async_duration
    );
    println!("Result: {}", async_result);

    // Calculate and display performance metrics
    let sync_ops_per_sec = sync_iterations as f64 / sync_duration.as_secs_f64();
    let async_ops_per_sec = async_iterations as f64 / async_duration.as_secs_f64();

    println!("\nPerformance Metrics:");
    println!("  Synchronous:   {:.2} operations/second", sync_ops_per_sec);
    println!(
        "  Asynchronous:  {:.2} operations/second",
        async_ops_per_sec
    );

    Ok(())
}
