use std::path::Path;
use std::time::Duration;
use rust_ai_ide_cargo::performance::{PerformanceAnalyzer, BuildMetrics};
use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Ensure the test project exists
    let project_path = "./test-performance-project";
    if !Path::new(project_path).exists() {
        anyhow::bail!("Test project not found at {}", project_path);
    }
    // Test debug build
    println!("Testing debug build with incremental compilation...");
    let analyzer = PerformanceAnalyzer::new(project_path, false, true);
    let metrics = analyzer.analyze_build().await
        .context("Failed to analyze debug build")?;
    print_metrics(&metrics);
    
    // Test release build
    println!("\nTesting release build without incremental compilation...");
    let analyzer = PerformanceAnalyzer::new(project_path, true, false);
    let metrics = analyzer.analyze_build().await
        .context("Failed to analyze release build")?;
    print_metrics(&metrics);
    
    Ok(())
}

fn print_metrics(metrics: &BuildMetrics) {
    println!("Performance Metrics:");
    println!("Total build time: {:.2?}", metrics.total_time);
    println!("\nCrates ({}):", metrics.crates.len());
    
    for (name, crate_metrics) in &metrics.crates {
        println!("\n  {}", name);
        println!("    Build time: {:.2?}", crate_metrics.build_time);
        println!("    Codegen time: {:.2?}", crate_metrics.codegen_time);
        println!("    Codegen units: {}", crate_metrics.codegen_units);
        println!("    Incremental: {}", crate_metrics.incremental);
        
        if !crate_metrics.dependencies.is_empty() {
            println!("    Dependencies ({}):", crate_metrics.dependencies.len());
            for dep in &crate_metrics.dependencies[..crate_metrics.dependencies.len().min(5)] {
                println!("      - {}", dep);
            }
            if crate_metrics.dependencies.len() > 5 {
                println!("      ... and {} more", crate_metrics.dependencies.len() - 5);
            }
        }
        
        if !crate_metrics.features.is_empty() {
            println!("    Features ({}): {}", 
                crate_metrics.features.len(), 
                crate_metrics.features.join(", ")
            );
        }
    }
    
    println!("\nDependencies ({}):", metrics.dependencies.len());
    let mut deps: Vec<_> = metrics.dependencies.iter().collect();
    deps.sort_by(|a, b| b.1.cmp(a.1));
    
    for (name, duration) in deps.iter().take(10) {
        println!("  {}: {:.2?}", name, duration);
    }
    if deps.len() > 10 {
        println!("  ... and {} more", deps.len() - 10);
    }
}
