//! Type Generator Binary
//!
//! This binary demonstrates how to use the shared-types crate to generate
//! type definitions for multiple platforms from your Rust types.
//!
//! Usage:
//!   cargo run --bin type_generator
//!   cargo run --bin type_generator -- --platform typescript
//!   cargo run --bin type_generator -- --all-platforms

use std::fs;
use std::path::Path;

use clap::{Parser, ValueEnum};
use rust_ai_ide_shared_types::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target platform to generate for
    #[arg(value_enum, default_value_t = Platform::Typescript)]
    platform: Platform,

    /// Generate for all supported platforms
    #[arg(long)]
    all_platforms: bool,

    /// Output directory (default: generated/)
    #[arg(short, long, default_value = "generated")]
    output: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// TypeScript: Generate type guards
    #[arg(long)]
    type_guards: bool,

    /// TypeScript: Use strict null checks
    #[arg(long)]
    strict_null: bool,

    /// Python: Format (dataclass or pydantic)
    #[arg(long, default_value = "dataclass")]
    python_format: String,

    /// Go: Package name
    #[arg(long, default_value = "models")]
    go_package: String,

    /// GraphQL: Schema type (basic or federation)
    #[arg(long, default_value = "basic")]
    graphql_schema: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Platform {
    Typescript,
    Python,
    Go,
    Graphql,
    Openapi,
    All,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 Shared Types Generator");
    println!("========================\n");

    // Create output directory
    let output_dir = Path::new(&args.output);
    fs::create_dir_all(output_dir)?;
    println!("📁 Output directory: {}", output_dir.display());

    // Read the source types
    let type_source = include_str!("../lib.rs");

    println!("📖 Reading type definitions from src/lib.rs");
    println!(
        "🔍 Found {} lines of Rust code\n",
        type_source.lines().count()
    );

    let generator = create_typescript_generator()?;

    if args.all_platforms {
        println!("🌐 Generating for ALL platforms...\n");
        generate_all_platforms(generator, type_source, output_dir, &args).await?;
    } else {
        let platform = match args.platform {
            Platform::Typescript => "typescript",
            Platform::Python => "python",
            Platform::Go => "go",
            Platform::Graphql => "graphql",
            Platform::Openapi => "openapi",
            Platform::All => unreachable!(),
        };

        println!("🎯 Generating for {}...\n", platform);
        generate_for_platform(generator, type_source, platform, output_dir, &args).await?;
    }

    println!("\n🎉 Type generation completed successfully!");
    println!(
        "📁 Check the '{}' directory for generated files",
        args.output
    );

    Ok(())
}

async fn generate_all_platforms(
    generator: TypeGenerator,
    source: &str,
    output_dir: &Path,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let platforms = vec![
        ("typescript", "TypeScript"),
        ("python", "Python"),
        ("go", "Go"),
        ("graphql", "GraphQL"),
        ("openapi", "OpenAPI"),
    ];

    for (platform, display_name) in platforms {
        println!("📦 Generating {} types...", display_name);
        generate_for_platform(generator.clone(), source, platform, output_dir, args).await?;
        println!("✅ {} generation completed\n", display_name);
    }

    Ok(())
}

async fn generate_for_platform(
    generator: TypeGenerator,
    source: &str,
    platform: &str,
    output_dir: &Path,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create platform-specific configuration
    let config = create_platform_config(platform, args);

    // Generate the types
    let start_time = std::time::Instant::now();
    let result = generator
        .generate_types_from_source(source, "lib.rs", &[])
        .await?;
    let generation_time = start_time.elapsed();

    // Determine output file path
    let file_name = format!("types.{}", get_file_extension(platform));
    let output_path = output_dir.join(file_name);
    let platform_dir = output_dir.join(platform);
    fs::create_dir_all(&platform_dir)?;

    let final_output_path = platform_dir
        .join("types")
        .with_extension(get_file_extension(platform));

    // Write the generated code
    fs::write(&final_output_path, &result.content)?;

    // Display results
    println!("  📄 Platform: {}", platform);
    println!("  📁 Output: {}", final_output_path.display());
    println!("  ⏱️  Generation time: {:?}", generation_time);
    println!("  📏 Code size: {} characters", result.content.len());
    println!("  🔢 Types processed: {}", result.source_types.len());

    // Show a preview of the generated code
    if std::env::var("CI").is_err() {
        // Don't show preview in CI
        let preview_lines: Vec<&str> = result.content.lines().take(5).collect();
        println!("  📋 Preview:");
        for (i, line) in preview_lines.iter().enumerate() {
            println!("    {}| {}", i + 1, line);
        }
        if result.content.lines().count() > 5 {
            println!(
                "    ... ({}, {} total)",
                "truncated",
                result.content.lines().count()
            );
        }
    }

    println!("  ✨ Generation completed for {}", platform);

    // Validate the generated code if possible
    validate_generated_code(&result.content, platform)?;

    Ok(())
}

fn create_platform_config(platform: &str, args: &Args) -> serde_json::Value {
    match platform {
        "typescript" => serde_json::json!({
            "generate_type_guards": args.type_guards,
            "strict_null_checks": args.strict_null,
            "generate_docs": true
        }),
        "python" => serde_json::json!({
            "format": args.python_format
        }),
        "go" => serde_json::json!({
            "package": args.go_package,
            "json_tags": true,
            "generate_getters": true
        }),
        "graphql" => serde_json::json!({
            "schema_type": args.graphql_schema,
            "mutations": true
        }),
        "openapi" => serde_json::json!({
            "title": "Integration Example API",
            "version": "1.0.0",
            "example_paths": true
        }),
        _ => serde_json::Value::Null,
    }
}

fn get_file_extension(platform: &str) -> &'static str {
    match platform {
        "typescript" => "ts",
        "python" => "py",
        "go" => "go",
        "graphql" => "graphql",
        "openapi" => "json",
        _ => "txt",
    }
}

fn validate_generated_code(content: &str, platform: &str) -> Result<(), Box<dyn std::error::Error>> {
    match platform {
        "typescript" => {
            // Basic TypeScript syntax validation
            if !content.contains("export interface") && !content.contains("export type") {
                eprintln!("⚠️  Warning: Generated TypeScript may not contain expected exports");
            }
            Ok(())
        }
        "python" => {
            // Basic Python syntax validation
            if !content.contains("class ") && !content.contains("from typing import") {
                eprintln!("⚠️  Warning: Generated Python may not contain expected class definitions");
            }
            Ok(())
        }
        "go" => {
            // Basic Go syntax validation
            if !content.contains("type ") && !content.contains("package ") {
                eprintln!("⚠️  Warning: Generated Go may not contain expected type definitions");
            }
            Ok(())
        }
        "graphql" => {
            // Basic GraphQL syntax validation
            if !content.contains("type ") && !content.contains("schema") {
                eprintln!("⚠️  Warning: Generated GraphQL may not contain expected schema definitions");
            }
            Ok(())
        }
        "openapi" => {
            // Validate JSON
            let _: serde_json::Value = serde_json::from_str(content)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_extensions() {
        assert_eq!(get_file_extension("typescript"), "ts");
        assert_eq!(get_file_extension("python"), "py");
        assert_eq!(get_file_extension("go"), "go");
        assert_eq!(get_file_extension("graphql"), "graphql");
        assert_eq!(get_file_extension("openapi"), "json");
    }

    #[test]
    fn test_platform_config() {
        let args = Args {
            platform:       Platform::Typescript,
            all_platforms:  false,
            output:         "generated".to_string(),
            verbose:        false,
            type_guards:    true,
            strict_null:    true,
            python_format:  "dataclass".to_string(),
            go_package:     "models".to_string(),
            graphql_schema: "basic".to_string(),
        };

        let config = create_platform_config("typescript", &args);
        assert_eq!(config["generate_type_guards"], true);
        assert_eq!(config["strict_null_checks"], true);
    }
}
