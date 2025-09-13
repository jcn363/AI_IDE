//! Visualization of performance metrics

use crate::performance::{BuildMetrics, OptimizationSuggestion};
use anyhow::Result;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use std::collections::HashMap;
use std::path::Path;

/// Generate a flamegraph from build metrics
pub fn generate_flamegraph(_metrics: &BuildMetrics, _output_path: &Path) -> Result<()> {
    // TODO: Implement flamegraph generation
    Ok(())
}

/// Generate a dependency graph visualization
pub fn generate_dependency_graph(metrics: &BuildMetrics, output_path: &Path) -> Result<()> {
    let mut graph = DiGraph::<&str, &str>::new();

    // Add nodes for each crate
    let nodes: HashMap<_, _> = metrics
        .crates
        .keys()
        .map(|name| (name.as_str(), graph.add_node(name.as_str())))
        .collect();

    // Add edges for dependencies
    for (crate_name, metrics) in &metrics.crates {
        if let Some(&source) = nodes.get(crate_name.as_str()) {
            for dep in &metrics.dependencies {
                if let Some(&target) = nodes.get(dep.as_str()) {
                    graph.add_edge(source, target, "");
                }
            }
        }
    }

    // Generate DOT format
    let dot = format!(
        "{:?}",
        Dot::with_config(&graph, &[Config::EdgeNoLabel, Config::NodeIndexLabel])
    );

    // Write to file
    std::fs::write(output_path, dot)?;

    Ok(())
}

/// Generate an HTML report with performance metrics
pub fn generate_html_report(metrics: &BuildMetrics, output_path: &Path) -> Result<()> {
    let mut html = String::new();

    // HTML header
    let header = format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Build Performance Report</title>
        <style>
            body {{ font-family: Arial, sans-serif; margin: 20px; }}
            table {{ border-collapse: collapse; width: 100%; }}
            th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
            th {{ background-color: #f2f2f2; }}
            tr:nth-child(even) {{ background-color: #f9f9f9; }}
        </style>
    </head>
    <body>
        <h1>Build Performance Report</h1>
        <h2>Summary</h2>
        <p>Total build time: {:.2?}</p>
        <p>Number of crates: {}</p>

        <h2>Crate Metrics</h2>
        <table>
            <tr>
                <th>Crate</th>
                <th>Version</th>
                <th>Build Time</th>
                <th>Workspace Member</th>
                <th>Dependencies</th>
            </tr>
    "#,
        metrics.total_time,
        metrics.crates.len()
    );
    html.push_str(&header);

    // Add crate metrics
    for (_, metrics) in &metrics.crates {
        html.push_str(&format!(
            r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td>{:.2?}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
            "#,
            metrics.name,
            metrics.version,
            metrics.build_time,
            metrics.is_workspace_member,
            metrics.dependencies.join(", ")
        ));
    }

    html.push_str("</table></body>\n</html>");
    html.push_str("</body>\n</html>");

    // Write to file
    std::fs::write(output_path, html)?;

    Ok(())
}

/// Generate optimization suggestions
pub fn generate_optimization_suggestions(metrics: &BuildMetrics) -> Vec<OptimizationSuggestion> {
    use OptimizationSuggestion::*;
    let mut suggestions = Vec::new();

    // Check for crates with high build times
    for (_, metrics) in &metrics.crates {
        // Suggest incremental builds for crates with long build times
        if metrics.build_time.as_secs_f64() > 5.0 {
            suggestions.push(EnableIncrementalCompilation(metrics.name.clone()));
        }

        // Check for unused dependencies
        if !metrics.dependencies.is_empty() {
            // This is a simplified check - in a real implementation, you'd want to analyze actual usage
            suggestions.push(CheckDependencyUsage(metrics.name.clone()));
        }

        // Check for crates that might benefit from workspace optimization
        if metrics.is_workspace_member && metrics.build_time.as_secs_f64() > 2.0 {
            suggestions.push(OptimizeWorkspaceDeps(metrics.name.clone()));
        }
    }

    suggestions
}
