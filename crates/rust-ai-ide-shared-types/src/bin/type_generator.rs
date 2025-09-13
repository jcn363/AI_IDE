use std::{env, fs};

use rust_ai_ide_shared_types::{create_typescript_generator, default_config};

/// Type generator binary for automated TypeScript generation from Rust types
///
/// Usage:
///   cargo run --bin type_generator [--file PATH | --types TYPE1 TYPE2 ...] [--output FILE]
///   cargo run --bin type_generator < crates/rust-ai-ide-shared-types/examples/types.rs >
/// web/src/types/generated.ts
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    // If no arguments provided, read from stdin
    if args.is_empty() {
        eprintln!("Reading type definitions from stdin...");
        eprintln!("Usage: cargo run --bin type_generator [--file PATH | --types TYPE1 TYPE2 ...]");
        eprintln!("Or: cat types.rs | cargo run --bin type_generator > output.ts");
        return Ok(());
    }

    // Parse command line arguments
    let mut file_path = None;
    let mut type_names = Vec::new();
    let mut output_path = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--file" =>
                if i + 1 < args.len() {
                    file_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --file requires a file path");
                    std::process::exit(1);
                },
            "--types" => {
                i += 1;
                while i < args.len() && !args[i].starts_with("--") {
                    type_names.push(args[i].clone());
                    i += 1;
                }
            }
            "--output" =>
                if i + 1 < args.len() {
                    output_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a file path");
                    std::process::exit(1);
                },
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    // Get type definitions source
    let (source, file_path) = match file_path {
        Some(path) => {
            let content = fs::read_to_string(&path)?;
            (content, path)
        }
        None => {
            // Read from stdin
            use std::io::Read;
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            (buffer, "stdin".to_string())
        }
    };

    // Create the TypeScript generator
    let generator = create_typescript_generator()?;

    // Generate types
    let type_names_refs: Vec<&str> = type_names.iter().map(|s| s.as_str()).collect();
    let result = generator
        .generate_types_from_source(&source, &file_path, &type_names_refs)
        .await?;

    // Validate cross-platform compatibility
    let rust_types = generator
        .generate_types_from_source(&source, &file_path, &type_names_refs)
        .await?
        .source_types;

    if !rust_types.is_empty() {
        let validation = rust_ai_ide_shared_types::validate_cross_platform(&rust_types, &default_config()).await?;

        if !validation.compatible {
            eprintln!("Warning: Cross-platform compatibility issues detected:");
            for issue in &validation.issues {
                eprintln!("  - {}", issue.description);
            }
        } else {
            eprintln!("✓ Cross-platform validation passed");
        }
    }

    // Output the generated TypeScript
    let output = if let Some(out_path) = output_path {
        // Write to file
        fs::write(&out_path, &result.content)?;
        eprintln!("Generated TypeScript written to: {}", out_path);
        result.content
    } else {
        // Write to stdout
        print!("{}", result.content);
        result.content
    };

    // Print statistics
    eprintln!(
        "Generated {} types with {} bytes",
        result.source_types.len(),
        output.len()
    );

    Ok(())
}

fn print_help() {
    eprintln!("TypeScript Generator for Rust AI IDE Shared Types");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  cargo run --bin type_generator [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --file PATH       Path to Rust source file containing type definitions");
    eprintln!("  --types TYPE...   Specific type names to generate (if not provided, generates all)");
    eprintln!("  --output FILE     Output file path (if not provided, writes to stdout)");
    eprintln!("  --help, -h        Show this help message");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  cargo run --bin type_generator --file src/types.rs > web/src/types/generated.ts");
    eprintln!("  cargo run --bin type_generator --file src/types.rs --types User Settings");
    eprintln!("  cat types.rs | cargo run --bin type_generator > generated.ts");
    eprintln!("  cargo run --bin type_generator --file src/main.rs --output web/src/generated.ts");
}
