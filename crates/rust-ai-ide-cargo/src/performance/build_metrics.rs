//! Build metrics collection and analysis

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::OptimizationSuggestion;

/// Metrics for a single crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateMetrics {
    /// Name of the crate
    pub name: String,
    /// Version of the crate
    pub version: String,
    /// Build time for this crate
    pub build_time: Duration,
    /// Whether this is a workspace member
    pub is_workspace_member: bool,
    /// Dependencies of this crate
    pub dependencies: Vec<String>,
    /// Time spent in code generation
    pub codegen_time: Duration,
    /// Number of codegen units used
    pub codegen_units: usize,
    /// Whether incremental compilation is enabled
    pub incremental: bool,
    /// Features enabled for this crate
    pub features: Vec<String>,
}

/// Performance metrics for a build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    /// Total build time
    pub total_time: Duration,
    /// Time spent in each compilation unit
    pub crates: HashMap<String, CrateMetrics>,
    /// Dependencies and their build times
    pub dependencies: HashMap<String, Duration>,
    /// Feature flags used during the build
    pub features: HashMap<String, Vec<String>>,
}

impl BuildMetrics {
    /// Get optimization suggestions based on build metrics
    pub fn get_optimization_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Check for crates with long build times
        for (name, metrics) in &self.crates {
            if metrics.build_time.as_secs() > 10 {
                suggestions.push(OptimizationSuggestion::HighBuildTime(format!(
                    "Crate '{}' has a long build time: {:.2?}",
                    name, metrics.build_time
                )));
            }

            // Check for crates with many codegen units
            if metrics.codegen_units > 16 {
                suggestions.push(OptimizationSuggestion::CodegenUnits(format!(
                    "Crate '{}' has a high number of codegen units: {}",
                    name, metrics.codegen_units
                )));
            }

            // Check for crates with many features
            if metrics.features.len() > 3 {
                suggestions.push(OptimizationSuggestion::FeatureOptimization(format!(
                    "Crate '{}' has many features enabled: {}",
                    name,
                    metrics.features.join(", ")
                )));
            }
        }

        suggestions
    }
}

/// Collects build metrics for a Cargo project
pub struct BuildMetricsCollector<'a> {
    /// Path to the Cargo project
    project_path: &'a Path,
    /// Whether to build in release mode
    release: bool,
    /// Whether to enable incremental compilation
    incremental: bool,
}

impl<'a> BuildMetricsCollector<'a> {
    /// Create a new BuildMetricsCollector
    pub fn new(project_path: &'a Path, release: bool, incremental: bool) -> Self {
        Self {
            project_path,
            release,
            incremental,
        }
    }

    /// Run the build and collect metrics
    pub async fn collect(&self) -> Result<BuildMetrics> {
        let start_time = Instant::now();
        println!(
            "Starting build in directory: {}",
            self.project_path.display()
        );
        println!(
            "Release mode: {}, Incremental: {}",
            self.release, self.incremental
        );

        // Run cargo build with the appropriate flags
        let mut command = Command::new("cargo");
        command
            .current_dir(&self.project_path)
            .arg("build")
            .arg("--verbose")
            .arg("--message-format=json");

        if self.release {
            command.arg("--release");
        }

        // Set incremental compilation via environment variable
        command.env(
            "CARGO_INCREMENTAL",
            if self.incremental { "1" } else { "0" },
        );
        command.env("RUST_LOG", "debug");
        command.env("CARGO_LOG", "cargo::core::compiler::fingerprint=info");

        println!("Executing: {:?}", command);
        let output = command.output().context("Failed to execute cargo build")?;
        let wall_time = start_time.elapsed();

        println!("Build completed with status: {}", output.status);
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Build stderr: {}", stderr);
            return Err(anyhow::anyhow!("Cargo build failed: {}", stderr));
        }

        // Print the first 1000 chars of stdout for debugging
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        println!(
            "Build output (first 1000 chars): {}",
            &stdout_str.chars().take(1000).collect::<String>()
        );

        // Parse the build output
        let metrics = self.parse_build_output(&output.stdout, wall_time)?;
        println!("Parsed metrics: {} crates", metrics.crates.len());

        Ok(metrics)
    }

    /// Parse the JSON output from cargo build
    fn parse_build_output(&self, output: &[u8], wall_time: Duration) -> Result<BuildMetrics> {
        let mut crates = HashMap::new();
        let dependencies: HashMap<String, Duration> = HashMap::new();
        let mut features: HashMap<String, Vec<String>> = HashMap::new();

        // Track build metrics
        let mut crate_build_times: HashMap<String, Duration> = HashMap::new();
        let mut crate_codegen_times: HashMap<String, Duration> = HashMap::new();
        let _crate_codegen_units: HashMap<String, usize> = HashMap::new();
        let mut build_script_times: HashMap<String, Duration> = HashMap::new();
        let mut fresh_crates: HashSet<String> = HashSet::new();
        println!("Parsing build output ({} bytes)...", output.len());

        for (mut parsed_count, line) in output.split(|&b| b == b'\n').enumerate() {
            if line.is_empty() {
                continue;
            }

            // Try to parse the line as JSON
            let value: serde_json::Value = match serde_json::from_slice(line) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to parse JSON line: {}", e);
                    eprintln!("Line content: {}", String::from_utf8_lossy(line));
                    continue;
                }
            };

            parsed_count += 1;

            // Debug: Print the first few messages to understand the structure
            if parsed_count <= 5 {
                println!("--- Message #{} ---", parsed_count);
                println!(
                    "Full message: {}",
                    serde_json::to_string_pretty(&value)
                        .unwrap_or_else(|_| "[Invalid JSON]".to_string())
                );
                if let Some(reason) = value.get("reason").and_then(|r| r.as_str()) {
                    println!("Reason: {}", reason);
                }
            }

            // Process the message based on its reason
            if let Some(reason) = value.get("reason").and_then(|r| r.as_str()) {
                match reason {
                    "compiler-artifact" => {
                        if let (Some(pkg_id), Some(target), Some(profile)) = (
                            value.get("package_id").and_then(|v| v.as_str()),
                            value
                                .get("target")
                                .and_then(|t| t.get("name"))
                                .and_then(|n| n.as_str()),
                            value.get("profile").and_then(|p| p.as_object()),
                        ) {
                            // Get package name without version
                            let name = pkg_id.split_whitespace().next().unwrap_or("unknown");
                            let crate_name = format!("{}:{}", name, target);

                            // Track freshness (skip weighting later if fresh)
                            let fresh = value
                                .get("fresh")
                                .and_then(|f| f.as_bool())
                                .unwrap_or(false);
                            if fresh {
                                fresh_crates.insert(crate_name.clone());
                            }

                            // Get profile information
                            let codegen_units = profile
                                .get("codegen_units")
                                .and_then(|u| u.as_u64())
                                .unwrap_or(16)
                                as usize;

                            let incremental = profile
                                .get("incremental")
                                .and_then(|i| i.as_bool())
                                .unwrap_or(false);

                            // Get features
                            let crate_features: Vec<String> = value
                                .get("features")
                                .and_then(|f| f.as_array())
                                .map(|f| {
                                    f.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();

                            // Get dependencies
                            let deps_list: Vec<String> = value
                                .get("dependencies")
                                .and_then(|d| d.as_array())
                                .map(|deps| {
                                    deps.iter()
                                        .filter_map(|d| d.as_str())
                                        .map(String::from)
                                        .collect()
                                })
                                .unwrap_or_default();

                            // Store features
                            features.insert(crate_name.clone(), crate_features.clone());

                            // Update or create crate metrics
                            let entry =
                                crates
                                    .entry(crate_name.clone())
                                    .or_insert_with(|| CrateMetrics {
                                        name: crate_name.clone(),
                                        version: String::new(),
                                        build_time: Duration::default(),
                                        is_workspace_member: false,
                                        dependencies: Vec::new(),
                                        codegen_time: Duration::default(),
                                        codegen_units,
                                        incremental,
                                        features: Vec::new(),
                                    });

                            // Update the entry with build info
                            entry.dependencies = deps_list.clone();
                            entry.features = crate_features;

                            println!("Processed crate: {}", crate_name);
                        }
                    }
                    "build-script-executed" => {
                        if let (Some(pkg_id), Some(duration_secs)) = (
                            value.get("package_id").and_then(|v| v.as_str()),
                            value.get("duration").and_then(|d| d.as_f64()),
                        ) {
                            let name = pkg_id
                                .split_whitespace()
                                .next()
                                .unwrap_or("unknown")
                                .to_string();
                            let duration = Duration::from_secs_f64(duration_secs);

                            // Store the build script execution time; attribution handled later
                            build_script_times.insert(name.clone(), duration);

                            println!("Build script executed - {}: {:.2?}", name, duration);
                        }
                    }
                    "build-finished" => {
                        if let Some(success) = value.get("success").and_then(|s| s.as_bool()) {
                            if !success {
                                return Err(anyhow::anyhow!(
                                    "Build completed but was not successful"
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Calculate total build time (use wall-clock time measured around cargo build)
        let total_build_duration = wall_time;

        // Distribute total time into per-crate times
        // 1) Assign build script times directly to their build-script targets
        let total_script_time: Duration = build_script_times.values().copied().sum();

        // 2) Compute weights for non-build-script crates
        let mut weights: HashMap<String, f64> = HashMap::new();
        let mut weight_sum = 0.0;

        for (crate_name, metrics) in &crates {
            if crate_name.ends_with(":build-script-build") {
                continue;
            }
            let base_name = crate_name.split(':').next().unwrap_or("");
            let is_path = base_name.starts_with("path+");
            let features_count = features.get(crate_name).map(|v| v.len()).unwrap_or(0) as f64;

            // If crate is fresh, assign near-zero weight
            let w = if fresh_crates.contains(crate_name) {
                0.000_001
            } else {
                1.0 + (1.0 + metrics.codegen_units as f64).ln() * 0.5
                    + features_count * 0.2
                    + if is_path { 0.5 } else { 0.0 }
            };
            weights.insert(crate_name.clone(), w);
            weight_sum += w;
        }

        // 3) Remaining time for compilation after build scripts
        let remaining_time = if total_build_duration > total_script_time {
            total_build_duration - total_script_time
        } else {
            Duration::from_secs(0)
        };

        // 4) Apply times to crates
        let non_build_script_count = crates
            .iter()
            .filter(|(n, _)| !n.ends_with(":build-script-build"))
            .count()
            .max(1) as u32;
        for (crate_name, metrics) in crates.iter_mut() {
            if crate_name.ends_with(":build-script-build") {
                // attribute build script time directly
                let base_name = crate_name.split(':').next().unwrap_or("");
                let script_time = build_script_times
                    .get(base_name)
                    .copied()
                    .unwrap_or_default();
                metrics.build_time = script_time;
                metrics.codegen_time = script_time / 2;
                crate_build_times.insert(crate_name.clone(), script_time);
                crate_codegen_times.insert(crate_name.clone(), script_time / 2);
            } else {
                let w = *weights.get(crate_name).unwrap_or(&0.0);
                let compilation_time = if weight_sum > 0.0 {
                    remaining_time.mul_f64(w / weight_sum)
                } else {
                    // All weights zero; split evenly as last resort
                    remaining_time / non_build_script_count
                };
                metrics.build_time = compilation_time;
                metrics.codegen_time = compilation_time / 2;
                crate_build_times.insert(crate_name.clone(), compilation_time);
                crate_codegen_times.insert(crate_name.clone(), compilation_time / 2);
            }
        }

        println!("\n=== Build Summary ===");
        println!("Total build time: {:.2?}", total_build_duration);
        println!(
            "Build script execution time: {:.2?}",
            build_script_times.values().sum::<Duration>()
        );
        println!("Number of crates: {}", crates.len());

        let total_time = if !crate_build_times.is_empty() {
            // Use the sum of all crate build times as it's more accurate
            let sum: Duration = crate_build_times.values().sum();
            println!("Using sum of crate build times: {:.2}s", sum.as_secs_f64());
            sum
        } else {
            // Fallback to measured wall time
            println!(
                "Using measured wall time: {:.2}s",
                total_build_duration.as_secs_f64()
            );
            total_build_duration
        };

        println!("Total build time: {:.2}s", total_time.as_secs_f64());
        println!("Crates processed: {}", crates.len());
        println!("Dependencies tracked: {}", dependencies.len());

        Ok(BuildMetrics {
            total_time,
            crates,
            dependencies: dependencies.into_iter().collect(),
            features,
        })
    }
}
