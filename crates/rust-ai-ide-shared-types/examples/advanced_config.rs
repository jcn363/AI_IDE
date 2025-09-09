//! Advanced configuration and plugin example
//!
//! This example demonstrates advanced usage including:
//! - Custom configuration
//! - Plugin system integration
//! - Type transformation customization
//! - Multi-platform generation

use rust_ai_ide_shared_types::*;
use std::collections::HashMap;

/// Example showing advanced configuration and customization options
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Shared Types Crate - Advanced Configuration Example ===\n");

    // Define complex Rust types with various edge cases
    let rust_code = r#"
        /// API Response wrapper
        pub struct ApiResponse<T> {
            /// Success status
            pub success: bool,
            /// Response data
            pub data: Option<T>,
            /// Error message
            pub error: Option<String>,
            /// Response metadata
            pub meta: ResponseMeta,
        }

        /// Response metadata
        pub struct ResponseMeta {
            /// Request ID
            pub request_id: String,
            /// Response timestamp
            pub timestamp: chrono::DateTime<chrono::Utc>,
            /// API version
            pub version: semver::Version,
        }

        /// User with complex nested types
        pub struct UserProfile {
            /// Basic info
            pub basic_info: BasicUserInfo,
            /// Account settings
            pub settings: AccountSettings,
            /// Activity history
            pub activities: Vec<UserActivity>,
        }

        /// Basic user information
        pub struct BasicUserInfo {
            pub id: u64,
            pub username: String,
            pub display_name: String,
            pub avatar_url: Option<String>,
        }

        /// Account settings
        #[derive(Serialize, Deserialize)]
        pub struct AccountSettings {
            pub theme: Theme,
            pub language: Language,
            pub timezone: String,
            pub notifications: NotificationPrefs,
        }

        /// Available themes
        #[derive(Serialize, Deserialize)]
        pub enum Theme {
            Light,
            Dark,
            Auto,
            Custom(String),
        }

        /// Languages
        #[derive(Serialize, Deserialize)]
        pub enum Language {
            En,
            Es,
            Fr,
            De,
            Custom(String),
        }

        /// Notification preferences
        pub struct NotificationPrefs {
            pub email: bool,
            pub push: bool,
            pub sms: bool,
            pub marketing: bool,
        }

        /// User activity
        pub struct UserActivity {
            pub id: String,
            pub activity_type: ActivityType,
            pub description: String,
            pub timestamp: chrono::NaiveDateTime,
        }

        /// Activity types
        #[derive(Serialize, Deserialize)]
        pub enum ActivityType {
            Login,
            Logout,
            UpdateProfile,
            CreatePost,
            DeletePost,
            FollowUser,
            UnfollowUser,
        }
    "#;

    // Create custom configuration
    let mut config = GenerationConfig::preset_development();

    // Customize TypeScript settings
    config.typescript.naming_convention = crate::config::NamingConvention::CamelCase;
    config.typescript.generate_type_guards = true;
    config.typescript.target_version = crate::config::TypeScriptVersion::ES2020;
    config.typescript.module_system = crate::config::ModuleSystem::ESModules;

    // Add custom type mappings
    config.typescript.type_mappings.insert("chrono::DateTime<chrono::Utc>".to_string()".to_string(), "string".to_string());
    config.typescript.type_mappings.insert("chrono::NaiveDateTime".to_string(), "string".to_string());
    config.typescript.type_mappings.insert("semver::Version".to_string(), "string".to_string());

    println!("✓ Created custom configuration:");
    println!("  - Naming Convention: CamelCase");
    println!("  - Type Guards: Enabled");
    println!("  - Target Version: ES2020");
    println!("  - Module System: ESModules");

    // Create generator with custom config
    let generator = TypeGenerator::with_full_config(config)?;
    println!("✓ Created generator with custom configuration");

    // Generate TypeScript with all types
    let result = generator.generate_types_from_source(rust_code, "advanced_example.rs", &[])?;
    println!("✓ Generated TypeScript with advanced configuration");

    println!("\n=== Generated TypeScript Code (Excerpt) ===");
    // Show first 30 lines
    let lines: Vec<&str> = result.content.lines().collect();
    for (i, line) in lines.iter().take(30).enumerate() {
        println!("{:3}| {}", i + 1, line);
    }

    if lines.len() > 30 {
        println!("... (showing first 30 lines of {} total)", lines.len());
    }

    println!("\n=== Advanced Features Demonstrated ===");
    println!("✓ Complex generic types (ApiResponse<T>)");
    println!("✓ Nested structures (UserProfile)");
    println!("✓ Enum variants with data (Theme::Custom)");
    println!("✓ Custom type mappings (chrono -> string)");
    println!("✓ Type guards generation");

    // Validate the complex types
    println!("\n=== Complex Type Validation ===");
    let validation_result = validate_cross_platform(&result.source_types, &default_config()).await?;

    println!("✓ Types validated: {}", result.source_types.len());
    println!("✓ Compatibility Score: {:.1}%", validation_result.compatibility_score * 100.0);
    println!("✓ Critical Issues: {}", validation_result.issues.iter()
        .filter(|i| i.severity == crate::bridge::ValidationSeverity::Error)
        .count());
    println!("✓ Warnings: {}", validation_result.issues.iter()
        .filter(|i| i.severity == crate::bridge::ValidationSeverity::Warning)
        .count());

    // Show recommendations if any
    if !validation_result.recommendations.is_empty() {
        println!("\n📝 Validation Recommendations:");
        for rec in &validation_result.recommendations {
            println!("  • {}", rec);
        }
    }

    // Demonstrate caching (conceptual)
    println!("\n=== Caching Integration Example ===");
    println!("✓ Cache enabled: {}", default_config().cache.enabled);
    println!("✓ Cache TTL: {} seconds", default_config().cache.ttl_seconds);
    println!("✓ Max cache size: {} MB", default_config().cache.max_size_mb);

    println!("\n=== Performance Statistics ===");
    println!("- Types processed: {}", result.metadata.stats.types_processed);
    println!("- Types generated: {}", result.metadata.stats.types_generated);
    println!("- Output size: {:.1} KB", result.metadata.stats.bytes_generated as f64 / 1024.0);
    println!("- Generation completed successfully: {}", matches!(result.metadata.status, crate::generation::GenerationStatus::Success));

    println!("\n🎉 Advanced configuration example completed successfully!");
    Ok(())
}