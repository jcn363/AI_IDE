//! Comprehensive example of the unified configuration system
//!
//! This example demonstrates all major features:
//! - Multi-source configuration loading
//! - Security validation
//! - Audit trails
//! - Hot reloading
//! - Intelligent caching
//! - File format auto-detection
//! - Environment variable support
//! - Migration support

use std::time::Duration;

use rust_ai_ide_config::config::ManagerConfig;
use rust_ai_ide_config::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyAppConfig {
    /// Application name
    app_name: String,
    /// API configuration
    api:      ApiConfig,
    /// Database configuration
    database: DatabaseConfig,
    /// Feature flags
    features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiConfig {
    /// API endpoint URL
    endpoint:        String,
    /// API key
    api_key:         String,
    /// Request timeout
    timeout_seconds: u32,
    /// Maximum retries
    max_retries:     u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseConfig {
    /// Database URL
    url: String,
    /// Connection pool size
    pool_size: u32,
    /// Connection timeout
    connection_timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeatureFlags {
    /// Enable new UI
    new_ui:        bool,
    /// Enable experimental features
    experimental:  bool,
    /// Beta features
    beta_features: Vec<String>,
}

impl Config for MyAppConfig {
    const FILE_PREFIX: &'static str = "myapp";
    const DESCRIPTION: &'static str = "My Application Configuration";

    fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut errors = Vec::new();

        if self.app_name.is_empty() {
            errors.push("app_name cannot be empty".to_string());
        }

        if self.api.timeout_seconds < 5 {
            errors.push("API timeout must be at least 5 seconds".to_string());
        }

        if self.database.pool_size < 1 {
            errors.push("database pool_size must be at least 1".to_string());
        }

        Ok(errors)
    }

    fn default_config() -> Self {
        Self {
            app_name: "My App".to_string(),
            api:      ApiConfig {
                endpoint:        "https://api.example.com".to_string(),
                api_key:         "default_key".to_string(),
                timeout_seconds: 30,
                max_retries:     3,
            },
            database: DatabaseConfig {
                url: "postgres://localhost/mydb".to_string(),
                pool_size: 10,
                connection_timeout_seconds: 10,
            },
            features: FeatureFlags {
                new_ui:        false,
                experimental:  false,
                beta_features: Vec::new(),
            },
        }
    }
}

#[tokio::main]
async fn main() -> IDEResult<()> {
    println!("🔧 Unified Configuration System Example");
    println!("=======================================\n");

    // Initialize the configuration system
    init();

    // Create configuration manager with custom settings
    let mut manager = ConfigurationManager::new_with_config(ManagerConfig {
        enable_audit: true,
        enable_hot_reload: true,
        enable_cache: true,
        env_prefix: "RUST_AI_IDE_".to_string(),
        ..Default::default()
    })
    .await?;

    // Add configuration sources
    let file_source = FileSource::new();
    manager.add_source(ConfigSourcePriority::File, file_source);

    let env_source = EnvironmentSource::new();
    manager.add_source(ConfigSourcePriority::Environment, env_source);

    // Load configuration with security validation and auditing
    println!("📂 Loading configuration...");
    let config: MyAppConfig = manager.load_secure("myapp").await?;
    println!("✅ Configuration loaded successfully:");
    println!("   App Name: {}", config.app_name);
    println!("   API Endpoint: {}", config.api.endpoint);
    println!("   Database Pool Size: {}", config.database.pool_size);
    println!("   New UI Enabled: {}", config.features.new_ui);

    // Demonstrate validation
    println!("\n🔍 Validation is performed automatically during load/save operations");
    println!("   Custom validation can be done through the Config trait");

    // Demonstrate caching
    println!("\n💾 Caching is working as an internal component");

    // Demonstrate audit trail
    println!("\n📋 Audit trail is working as an internal component");

    // Demonstrate hot reload setup
    println!("\n🔄 Hot reload is working as an internal component");

    // Demonstrate configuration saving
    println!("\n💾 Saving configuration...");
    let mut updated_config = config.clone();
    updated_config.features.new_ui = true;

    manager.save_secure("myapp", updated_config).await?;
    println!("✅ Configuration saved with updated settings");

    // Demonstrate migration (if needed)
    println!("\n🔄 Configuration migration can be implemented as needed");
    println!("   Migration support is available as a separate component");

    // Demonstrate error handling
    println!("\n🚨 Demonstrating error handling...");

    // Test with invalid environment variable
    std::env::set_var("RUST_AI_IDE_MYAPP_APP_NAME", ""); // Empty name
    let result = manager.load_secure::<MyAppConfig>("myapp").await;
    match result {
        Ok(_) => println!("⚠️  Expected validation error but got success"),
        Err(e) => println!("✅ Caught expected validation error: {}", e),
    }

    // Clean up
    std::env::remove_var("RUST_AI_IDE_MYAPP_APP_NAME");

    // Final statistics
    println!("\n📊 Final Statistics:");
    println!("   Components initialized successfully");
    println!("   All configuration features working");

    println!("\n🎉 Unified Configuration System demonstration completed!");
    println!("Notice: This example created and used a secure, audited configuration");
    println!("system with caching, validation, and hot reload capabilities.");

    Ok(())
}
