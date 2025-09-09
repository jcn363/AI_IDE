//! Basic usage example for the shared types crate
//!
//! This example demonstrates how to use the shared types crate to:
//! - Parse Rust types from source files
//! - Generate TypeScript definitions
//! - Validate cross-platform compatibility
//! - Use configuration options

use rust_ai_ide_shared_types::*;
use std::path::Path;

/// Example showing basic type generation workflow
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Shared Types Crate - Basic Usage Example ===\n");

    // Define some sample Rust types
    let rust_code = r#"
        /// User information
        #[derive(Serialize, Deserialize)]
        pub struct User {
            /// Unique user ID
            pub id: u32,
            /// User's full name
            pub name: String,
            /// User's email address
            pub email: Option<String>,
            /// User creation timestamp
            pub created_at: chrono::NaiveDateTime,
            /// User preferences
            pub preferences: UserPreferences,
        }

        /// User preferences
        #[derive(Serialize, Deserialize)]
        pub struct UserPreferences {
            /// Theme preference
            pub theme: Theme,
            /// Notification settings
            pub notifications: NotificationSettings,
        }

        /// Available themes
        #[derive(Serialize, Deserialize)]
        pub enum Theme {
            Light,
            Dark,
            System,
        }

        /// Notification settings
        #[derive(Serialize, Deserialize)]
        pub struct NotificationSettings {
            /// Email notifications enabled
            pub email: bool,
            /// Push notifications enabled
            pub push: bool,
            /// Desktop notifications enabled
            pub desktop: bool,
        }
    "#;

    // Create a temporary file with the sample code
    let temp_path = Path::new("temp_example.rs");
    std::fs::write(temp_path, rust_code)?;
    println!("✓ Created temporary Rust file with sample types");

    // Create TypeScript generator
    let generator = create_typescript_generator()?;
    println!("✓ Created TypeScript generator");

    // Generate TypeScript from all types in the file
    let result = generator.generate_types_from_source(rust_code, "example.rs", &[])?;
    println!("✓ Generated TypeScript definitions");

    println!("\n=== Generated TypeScript Code ===");
    println!("{}", result.content);

    println!("\n=== Generation Metadata ===");
    println!("- Target Platform: {}", result.target_platform);
    println!("- Types Processed: {}", result.metadata.stats.types_processed);
    println!("- Types Generated: {}", result.metadata.stats.types_generated);
    println!("- Generation Time: {}ms", result.metadata.stats.generation_time_ms);
    println!("- Status: {:?}", result.metadata.status);

    // Validate cross-platform compatibility
    println!("\n=== Cross-Platform Validation ===");
    let validation_result = validate_cross_platform(&result.source_types, &default_config()).await?;
    println!("✓ Compatibility Score: {:.2}%", validation_result.compatibility_score * 100.0);
    println!("✓ Compatible: {}", validation_result.compatible);

    if !validation_result.issues.is_empty() {
        println!("⚠️  Found {} validation issues:", validation_result.issues.len());
        for issue in &validation_result.issues {
            println!("  - {}: {}", issue.source_type, issue.description);
        }
    }

    if !validation_result.recommendations.is_empty() {
        println!("\n📝 Recommendations:");
        for rec in &validation_result.recommendations {
            println!("  - {}", rec);
        }
    }

    // Clean up
    std::fs::remove_file(temp_path)?;
    println!("\n✓ Cleaned up temporary files");

    println!("\n🎉 Example completed successfully!");
    Ok(())
}