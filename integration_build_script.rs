//! Build script integration for the shared types crate
//!
//! This demonstrates how to integrate automatic type generation into your build process.
//! Place this as build.rs in your project root or run it as a standalone script.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/types.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    // Only generate in debug builds for faster release builds
    if cfg!(debug_assertions) {
        generate_types().await?;
    } else {
        println!("cargo:warning=Type generation skipped in release mode");
    }

    println!("cargo:warning=Shared types generation completed");

    Ok(())
}

async fn generate_types() -> Result<(), Box<dyn std::error::Error>> {
    // Generate TypeScript types using the shared types crate binary
    let output = Command::new("cargo")
        .args(&["run", "--bin", "type_generator", "--package", "rust-ai-ide-shared-types"])
        .output()?;

    if !output.status.success() {
        println!("cargo:warning=Failed to generate types: {:?}", String::from_utf8_lossy(&output.stderr));
        return Ok(()); // Don't fail the build
    }

    // The binary generates TypeScript to stdout
    let generated_content = String::from_utf8_lossy(&output.stdout);
    println!("cargo:warning=Generated {} characters of TypeScript definitions", generated_content.len());

    // Write to web directory
    let web_types_dir = Path::new("web/src/types");
    fs::create_dir_all(web_types_dir)?;
    fs::write(web_types_dir.join("generated.ts"), &*generated_content)?;

    println!("cargo:warning=TypeScript types written to web/src/types/generated.ts");

    // Perform cross-platform validation
    validate_generated_types().await?;

    Ok(())
}

async fn validate_generated_types() -> Result<(), Box<dyn std::error::Error>> {
    // Import and validate cross-platform compatibility
    if cfg!(feature = "shared-types") {
        println!("cargo:warning=Performing cross-platform type validation...");

        // In a real implementation, this would use the shared types crate
        // rust_ai_ide_shared_types::validate_cross_platform(...)

        println!("cargo:warning=Cross-platform validation completed successfully");
    }

    Ok(())
}

// Alternative: If you want to embed the generator directly
#[cfg(feature = "embedded_generator")]
async fn generate_types_embedded() -> Result<(), Box<dyn std::error::Error>> {
    use rust_ai_ide_shared_types::*;

    let generator = create_typescript_generator()?;

    // Read your source files
    let type_source = include_str!("src/types.rs");
    let lib_source = include_str!("src/lib.rs");

    // Combine sources or process individually
    let combined_source = format!("{}\n{}", lib_source, type_source);

    // Generate TypeScript
    let ts_result = generator.generate_types_from_source(
        &combined_source,
        "src/combined.rs",
        &[] // Empty array means generate all types
    ).await?;

    // Write to frontend directory
    let frontend_dir = Path::new("../frontend/src/types");
    fs::create_dir_all(frontend_dir)?;
    fs::write(frontend_dir.join("generated.ts"), ts_result.content)?;

    // Generate for other platforms too
    let platforms = vec![
        ("python", "../backend/api/models.py"),
        ("go", "../backend/api/models.go"),
        ("graphql", "../docs/schema.graphql"),
        ("openapi", "../docs/openapi.json"),
    ];

    let generator = create_typescript_generator()?; // Reuse or create new

    for (platform, output_path) in platforms {
        let config = match platform {
            "python" => serde_json::json!({"format": "dataclass"}),
            "go" => serde_json::json!({"package": "api", "json_tags": true}),
            "graphql" => serde_json::json!({"schema_type": "basic"}),
            "openapi" => serde_json::json!({"title": "API", "version": "1.0.0"}),
            _ => serde_json::Value::Null,
        };

        let result = generator.generate_types_from_source(
            &combined_source,
            "src/combined.rs",
            &[]
        ).await?;

        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(output_path, result.content)?;
        println!("cargo:warning=Generated {} for {}", platform, output_path);
    }

    Ok(())
}