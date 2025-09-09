use anyhow::Context;
use std::time::Instant;

fn run_command(cmd: &str, args: &[&str], cwd: &std::path::Path) -> anyhow::Result<()> {
    println!("Running: {} {}", cmd, args.join(" "));
    let status = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .status()?;

    if !status.success() {
        anyhow::bail!("Command failed with status: {}", status);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Ensure the test project exists
    let project_path = "test-performance-project";
    let abs_path = std::env::current_dir()?.join(project_path);
    if !abs_path.exists() {
        anyhow::bail!("Test project not found at {}", abs_path.display());
    }

    // Clean the project first
    println!("Cleaning the project...");
    run_command("cargo", &["clean"], &abs_path)?;

    // Test debug build with incremental compilation
    println!("\n=== Testing debug build with incremental compilation ===");
    let start = Instant::now();
    let metrics = rust_ai_ide_cargo::commands::analyze_performance(&abs_path, false, true)
        .await
        .context("Failed to analyze debug build")?;
    let elapsed = start.elapsed();
    print_metrics(&metrics);
    println!("Analysis took: {:.2?}\n", elapsed);

    // Clean again before release build
    println!("Cleaning the project...");
    run_command("cargo", &["clean"], &abs_path)?;

    // Test release build without incremental compilation
    println!("\n=== Testing release build without incremental compilation ===");
    let start = Instant::now();
    let metrics = rust_ai_ide_cargo::commands::analyze_performance(&abs_path, true, false)
        .await
        .context("Failed to analyze release build")?;
    let elapsed = start.elapsed();
    print_metrics(&metrics);
    println!("Analysis took: {:.2?}", elapsed);

    Ok(())
}

fn print_metrics(metrics: &rust_ai_ide_cargo::commands::PerformanceMetrics) {
    println!("Performance Metrics:");
    println!("Total build time: {:.2}ms", metrics.total_time_ms);
    println!("\nCrates ({}):", metrics.crates.len());

    for (name, crate_metrics) in &metrics.crates {
        println!("\n  {}", name);
        println!("    Build time: {:.2}ms", crate_metrics.build_time_ms);
        println!("    Codegen time: {:.2}ms", crate_metrics.codegen_time_ms);
        println!("    Codegen units: {}", crate_metrics.codegen_units);
        println!("    Incremental: {}", crate_metrics.incremental);

        if !crate_metrics.dependencies.is_empty() {
            println!("    Dependencies ({}):", crate_metrics.dependencies.len());
            for dep in &crate_metrics.dependencies[..crate_metrics.dependencies.len().min(5)] {
                println!("      - {}", dep);
            }
            if crate_metrics.dependencies.len() > 5 {
                println!(
                    "      ... and {} more",
                    crate_metrics.dependencies.len() - 5
                );
            }
        }

        if !crate_metrics.features.is_empty() {
            println!(
                "    Features ({}): {}",
                crate_metrics.features.len(),
                crate_metrics.features.join(", ")
            );
        }
    }

    println!("\nDependencies ({}):", metrics.dependencies.len());
    let mut deps: Vec<_> = metrics.dependencies.iter().collect();
    deps.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (name, duration) in deps.iter().take(10) {
        println!("  {}: {:.2}ms", name, duration);
    }
    if deps.len() > 10 {
        println!("  ... and {} more", deps.len() - 10);
    }
}
