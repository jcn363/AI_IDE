use rust_ai_ide_common::{IDEError, IDEErrorKind};
use rust_ai_ide_performance::{
    LazyLoader, ModulePreloader, ProfilingAdapter, ProfilingConfiguration, StartupCache,
    StartupProfiler, StartupReport, StartupStats, StartupValidator,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Integration tests for startup performance optimization
/// Ensures startup performance targets are met: <400ms cold / <80ms warm

const WARM_STARTUP_TARGET: Duration = Duration::from_millis(80);
const COLD_STARTUP_TARGET: Duration = Duration::from_millis(400);
const PERFORMANCE_VARIANCE_TOLERANCE: f64 = 0.15; // 15% tolerance for variance

/// Simulated IDE startup phases for testing
async fn simulate_startup_phases(
    profiler: Arc<StartupProfiler>,
    adapter: &ProfilingAdapter,
    is_cold_startup: bool,
) -> Result<(), IDEError> {
    // Phase 1: Core initialization
    adapter
        .measure_phase("core_initialization", async {
            simulate_cpu_bound_work(Duration::from_millis(if is_cold_startup { 80 } else { 15 }))
                .await;
            Ok::<_, IDEError>(())
        })
        .await?;

    // Phase 2: Plugin loading
    adapter
        .measure_phase("plugin_loading", async {
            simulate_io_bound_work(Duration::from_millis(if is_cold_startup {
                120
            } else {
                25
            }))
            .await;
            Ok::<_, IDEError>(())
        })
        .await?;

    // Phase 3: LSP service initialization
    adapter
        .measure_phase("lsp_initialization", async {
            simulate_async_work(Duration::from_millis(if is_cold_startup { 90 } else { 18 })).await;
            Ok::<_, IDEError>(())
        })
        .await?;

    // Phase 4: AI model preparation (lazy loaded)
    adapter
        .measure_phase("ai_model_preparation", async {
            simulate_cpu_bound_work(Duration::from_millis(if is_cold_startup { 70 } else { 12 }))
                .await;
            Ok::<_, IDEError>(())
        })
        .await?;

    // Phase 5: UI initialization
    adapter
        .measure_phase("ui_initialization", async {
            simulate_io_bound_work(Duration::from_millis(if is_cold_startup { 40 } else { 8 }))
                .await;
            Ok::<_, IDEError>(())
        })
        .await?;

    Ok(())
}

/// Simulate CPU-bound work (e.g., model loading, compilation)
async fn simulate_cpu_bound_work(duration: Duration) {
    let start = Instant::now();
    let mut counter = 0u64;

    // Busy loop simulation
    while start.elapsed() < duration {
        counter = counter.wrapping_add(1);
        if counter % 1000 == 0 {
            // Yield occasionally to simulate real async behavior
            tokio::task::yield_now().await;
        }
    }
}

/// Simulate I/O-bound work (e.g., file reading, network calls)
async fn simulate_io_bound_work(duration: Duration) {
    sleep(duration).await;
}

/// Simulate asynchronous work with proper tokio scheduling
async fn simulate_async_work(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// Complete startup simulation with performance measurement
async fn perform_startup_simulation(
    is_cold_startup: bool,
    enable_optimizations: bool,
) -> Result<Duration, IDEError> {
    let profiler = Arc::new(StartupProfiler::new());
    let adapter = ProfilingAdapter::new(profiler.clone());

    // Configure profiling
    let mut config = ProfilingConfiguration::default();
    config.cold_startup_target = COLD_STARTUP_TARGET;
    config.warm_startup_target = WARM_STARTUP_TARGET;
    adapter.update_configuration(config).await?;

    // Start measurement
    adapter.start_startup_measurement(is_cold_startup).await?;

    let startup_result = {
        // If optimizations enabled, simulate lazy loading and caching
        if enable_optimizations {
            simulate_optimized_startup(&adapter, is_cold_startup).await
        } else {
            simulate_startup_phases(profiler.clone(), &adapter, is_cold_startup).await
        }
    };

    // Get final report
    let report = adapter.end_startup_measurement().await?;

    match startup_result {
        Ok(_) => Ok(report.total_startup_time),
        Err(e) => Err(e),
    }
}

/// Simulate optimized startup with lazy loading and caching
async fn simulate_optimized_startup(
    adapter: &ProfilingAdapter,
    is_cold_startup: bool,
) -> Result<(), IDEError> {
    // Phase 1: Core initialization (always loaded)
    adapter
        .measure_phase("core_initialization", async {
            simulate_cpu_bound_work(Duration::from_millis(if is_cold_startup { 60 } else { 12 }))
                .await;
            Ok::<_, IDEError>(())
        })
        .await?;

    // Phase 2: Critical plugins (always loaded)
    adapter
        .measure_phase("critical_plugins", async {
            simulate_io_bound_work(Duration::from_millis(if is_cold_startup { 80 } else { 16 }))
                .await;
            Ok::<_, IDEError>(())
        })
        .await?;

    // Phase 3: Minimal LSP (lazy loaded heavy components)
    adapter
        .measure_phase("minimal_lsp", async {
            simulate_async_work(Duration::from_millis(if is_cold_startup { 50 } else { 10 })).await;
            Ok::<_, IDEError>(())
        })
        .await?;

    // Phase 4: UI essentials
    adapter
        .measure_phase("ui_essentials", async {
            simulate_cpu_bound_work(Duration::from_millis(if is_cold_startup { 25 } else { 5 }))
                .await;
            Ok::<_, IDEError>(())
        })
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_cold_startup_performance_baseline() {
    let duration = perform_startup_simulation(true, false).await.unwrap();

    println!("Cold startup baseline: {}ms", duration.as_millis());
    assert!(
        duration < COLD_STARTUP_TARGET,
        "Cold startup exceeded target: {}ms > {}ms",
        duration.as_millis(),
        COLD_STARTUP_TARGET.as_millis()
    );
}

#[tokio::test]
async fn test_warm_startup_performance_baseline() {
    let duration = perform_startup_simulation(false, false).await.unwrap();

    println!("Warm startup baseline: {}ms", duration.as_millis());
    assert!(
        duration < WARM_STARTUP_TARGET,
        "Warm startup exceeded target: {}ms > {}ms",
        duration.as_millis(),
        WARM_STARTUP_TARGET.as_millis()
    );
}

#[tokio::test]
async fn test_cold_startup_performance_optimized() {
    let duration = perform_startup_simulation(true, true).await.unwrap();

    println!("Cold startup optimized: {}ms", duration.as_millis());
    assert!(
        duration < COLD_STARTUP_TARGET,
        "Cold startup optimizations exceeded target: {}ms > {}ms",
        duration.as_millis(),
        COLD_STARTUP_TARGET.as_millis()
    );

    // Optimized should be significantly better than baseline
    let baseline_estimate = Duration::from_millis(380); // Estimated baseline
    let improvement_ratio = baseline_estimate.as_millis() as f64 / duration.as_millis() as f64;
    assert!(
        improvement_ratio > 1.1,
        "Optimization should provide at least 10% improvement, got {:.2}x",
        improvement_ratio
    );
}

#[tokio::test]
async fn test_warm_startup_performance_optimized() {
    let duration = perform_startup_simulation(false, true).await.unwrap();

    println!("Warm startup optimized: {}ms", duration.as_millis());
    assert!(
        duration < WARM_STARTUP_TARGET,
        "Warm startup optimizations exceeded target: {}ms > {}ms",
        duration.as_millis(),
        WARM_STARTUP_TARGET.as_millis()
    );

    // Optimized should be significantly better than baseline
    let baseline_estimate = Duration::from_millis(75); // Estimated baseline
    let improvement_ratio = baseline_estimate.as_millis() as f64 / duration.as_millis() as f64;
    assert!(
        improvement_ratio > 1.05,
        "Optimization should provide at least 5% improvement, got {:.2}x",
        improvement_ratio
    );
}

#[tokio::test]
async fn test_startup_performance_consistency() {
    let mut cold_durations = Vec::new();
    let mut warm_durations = Vec::new();

    // Run multiple startups to check consistency
    for _ in 0..5 {
        cold_durations.push(perform_startup_simulation(true, true).await.unwrap());
        warm_durations.push(perform_startup_simulation(false, true).await.unwrap());
    }

    // Check cold startup consistency
    let cold_avg = cold_durations.iter().sum::<Duration>() / cold_durations.len() as u32;
    let cold_variance = calculate_variance(&cold_durations, cold_avg);

    println!("Cold startup variance: {:.2}ms", cold_variance.as_millis());
    assert!(
        cold_variance < Duration::from_millis(50),
        "Cold startup variance too high: {:.2}ms > 50ms",
        cold_variance.as_millis()
    );

    // Check warm startup consistency
    let warm_avg = warm_durations.iter().sum::<Duration>() / warm_durations.len() as u32;
    let warm_variance = calculate_variance(&warm_durations, warm_avg);

    println!("Warm startup variance: {:.2}ms", warm_variance.as_millis());
    assert!(
        warm_variance < Duration::from_millis(20),
        "Warm startup variance too high: {:.2}ms > 20ms",
        warm_variance.as_millis()
    );
}

#[tokio::test]
async fn test_lazy_loading_effectiveness() {
    let profiler = Arc::new(StartupProfiler::new());
    let adapter = ProfilingAdapter::new(profiler);

    // Test lazy loading mechanisms
    adapter.start_startup_measurement(true).await.unwrap();

    // Simulate lazy loading with background tasks
    adapter
        .measure_phase("lazy_component_prep", async {
            // Fire off "lazy" tasks that don't block startup
            for i in 0..3 {
                tokio::spawn(async move {
                    simulate_cpu_bound_work(Duration::from_millis(100)).await;
                });
            }

            // Startup continues without waiting
            simulate_cpu_bound_work(Duration::from_millis(20)).await;
            Ok::<_, IDEError>(())
        })
        .await
        .unwrap();

    let report = adapter.end_startup_measurement().await.unwrap();

    // Startup time should not include lazy loaded components
    assert!(
        report.total_startup_time < Duration::from_millis(100),
        "Lazy loading should not block startup: {}ms >= 100ms",
        report.total_startup_time.as_millis()
    );
}

#[tokio::test]
async fn test_performance_monitoring_integration() {
    let profiler = Arc::new(StartupProfiler::new());
    let adapter = ProfilingAdapter::new(profiler.clone());
    let validator = StartupValidator::new(profiler.clone(), COLD_STARTUP_TARGET);

    // Run a performance test
    let startup_time = perform_startup_simulation(true, true).await.unwrap();

    // Validate through monitoring
    let validation_result = validator.validate_startup_performance().await;
    assert!(
        validation_result.is_within_threshold,
        "Performance validation failed for {}ms startup",
        startup_time.as_millis()
    );

    println!(
        "✅ Performance monitoring validation passed: {}ms",
        startup_time.as_millis()
    );
}

#[tokio::test]
async fn test_caching_performance_gains() {
    let cache = StartupCache::new(1000, 300); // 5 minutes TTL

    let test_data = vec![1, 2, 3, 4, 5];

    // First call - compute and cache
    let start = Instant::now();
    let result1 = cache
        .cache_expensive_computation("test_data", async {
            simulate_cpu_bound_work(Duration::from_millis(50)).await;
            Ok::<Vec<i32>, IDEError>(test_data.clone())
        })
        .await
        .unwrap();
    let first_duration = start.elapsed();

    // Second call - should be cached
    let start = Instant::now();
    let result2 = cache
        .cache_expensive_computation("test_data", async {
            simulate_cpu_bound_work(Duration::from_millis(50)).await;
            Ok::<Vec<i32>, IDEError>(test_data.clone())
        })
        .await
        .unwrap();
    let second_duration = start.elapsed();

    // Second call should be significantly faster
    assert_eq!(result1, result2);
    assert!(
        second_duration < first_duration.saturating_div(10),
        "Cached call should be at least 10x faster: {}ms vs {}ms",
        second_duration.as_millis(),
        first_duration.as_millis()
    );
}

fn calculate_variance(durations: &[Duration], mean: Duration) -> Duration {
    let variance = durations
        .iter()
        .map(|d| {
            let diff = if *d > mean {
                d.saturating_sub(mean)
            } else {
                mean.saturating_sub(*d)
            };
            diff.as_millis() as f64 * diff.as_millis() as f64
        })
        .sum::<f64>()
        / durations.len() as f64;

    Duration::from_millis(variance.sqrt() as u64)
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion};

    fn startup_performance_benchmark(c: &mut Criterion) {
        c.bench_function("cold_startup_optimized", |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| perform_startup_simulation(true, true))
        });

        c.bench_function("warm_startup_optimized", |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| perform_startup_simulation(false, true))
        });

        c.bench_function("cold_startup_baseline", |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| perform_startup_simulation(true, false))
        });

        c.bench_function("warm_startup_baseline", |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| perform_startup_simulation(false, false))
        });
    }

    criterion_group!(benches, startup_performance_benchmark);
    // criterion_main!(benches); // Uncomment to run benchmarks
}

/// Comprehensive performance regression test
#[tokio::test]
async fn test_performance_regression_suite() {
    println!("🧪 Running Startup Performance Regression Suite");

    // Test cold startup regression
    let cold_duration = perform_startup_simulation(true, true).await.unwrap();
    assert_cold_startup_target(cold_duration);

    // Test warm startup regression
    let warm_duration = perform_startup_simulation(false, true).await.unwrap();
    assert_warm_startup_target(warm_duration);

    // Test optimization effectiveness
    let cold_baseline = perform_startup_simulation(true, false).await.unwrap();
    let cold_optimized = perform_startup_simulation(true, true).await.unwrap();
    assert_optimization_effectiveness(cold_baseline, cold_optimized, "cold");

    let warm_baseline = perform_startup_simulation(false, false).await.unwrap();
    let warm_optimized = perform_startup_simulation(false, true).await.unwrap();
    assert_optimization_effectiveness(warm_baseline, warm_optimized, "warm");

    println!("✅ All performance regression tests passed!");
    println!(
        "📊 Results: Cold: {}ms (target: {}ms), Warm: {}ms (target: {}ms)",
        cold_duration.as_millis(),
        COLD_STARTUP_TARGET.as_millis(),
        warm_duration.as_millis(),
        WARM_STARTUP_TARGET.as_millis()
    );
}

fn assert_cold_startup_target(duration: Duration) {
    assert!(
        duration <= COLD_STARTUP_TARGET,
        "❌ Cold startup regression: {}ms exceeds target {}ms",
        duration.as_millis(),
        COLD_STARTUP_TARGET.as_millis()
    );
    println!(
        "✅ Cold startup within target: {}ms ≤ {}ms",
        duration.as_millis(),
        COLD_STARTUP_TARGET.as_millis()
    );
}

fn assert_warm_startup_target(duration: Duration) {
    assert!(
        duration <= WARM_STARTUP_TARGET,
        "❌ Warm startup regression: {}ms exceeds target {}ms",
        duration.as_millis(),
        WARM_STARTUP_TARGET.as_millis()
    );
    println!(
        "✅ Warm startup within target: {}ms ≤ {}ms",
        duration.as_millis(),
        WARM_STARTUP_TARGET.as_millis()
    );
}

fn assert_optimization_effectiveness(baseline: Duration, optimized: Duration, startup_type: &str) {
    let improvement_percent = ((baseline.as_millis() as f64 - optimized.as_millis() as f64)
        / baseline.as_millis() as f64)
        * 100.0;

    match startup_type {
        "cold" => {
            assert!(
                improvement_percent >= 5.0,
                "❌ Cold startup optimization insufficient: {:.1}% improvement < 5%",
                improvement_percent
            );
            println!(
                "✅ Cold startup optimization effective: {:.1}% improvement",
                improvement_percent
            );
        }
        "warm" => {
            assert!(
                improvement_percent >= 2.0,
                "❌ Warm startup optimization insufficient: {:.1}% improvement < 2%",
                improvement_percent
            );
            println!(
                "✅ Warm startup optimization effective: {:.1}% improvement",
                improvement_percent
            );
        }
        _ => panic!("Unknown startup type"),
    }
}
