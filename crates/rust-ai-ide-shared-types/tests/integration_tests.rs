//! Comprehensive integration tests for the shared types crate
//!
//! This module contains end-to-end tests that validate the entire
//! pipeline from Rust type parsing through generation and validation.

use rust_ai_ide_shared_types::*;
use std::collections::HashMap;

/// Test the complete type generation pipeline
#[tokio::test]
async fn test_complete_generation_pipeline() {
    // Define comprehensive Rust types
    let rust_code = r#"
        /// A comprehensive user model
        #[derive(Serialize, Deserialize)]
        pub struct User {
            /// Unique user identifier
            pub id: u64,

            /// User's full name
            pub name: String,

            /// Email address (optional)
            pub email: Option<String>,

            /// Age in years
            pub age: i32,

            /// Account settings
            pub settings: UserSettings,

            /// List of permissions
            pub permissions: Vec<String>,

            /// Profile metadata
            pub profile: Profile,
        }

        /// User account settings
        #[derive(Serialize, Deserialize)]
        pub struct UserSettings {
            /// Theme preference
            pub theme: Theme,

            /// Notification preferences
            pub notifications: NotificationSettings,

            /// UI language
            pub language: Language,

            /// Account status
            pub status: AccountStatus,
        }

        /// Available themes
        #[derive(Serialize, Deserialize)]
        pub enum Theme {
            Light,
            Dark,
            Auto,
            Custom(String),
        }

        /// Notification settings
        #[derive(Serialize, Deserialize)]
        pub struct NotificationSettings {
            /// Email notifications
            pub email: bool,

            /// Push notifications
            pub push: bool,

            /// SMS notifications
            pub sms: bool,

            /// Desktop notifications
            pub desktop: bool,
        }

        /// Supported languages
        #[derive(Serialize, Deserialize)]
        pub enum Language {
            En,
            Es,
            Fr,
            De,
        }

        /// Account status
        #[derive(Serialize, Deserialize)]
        pub enum AccountStatus {
            Active,
            Suspended,
            Deleted,
        }

        /// User profile information
        #[derive(Serialize, Deserialize)]
        pub struct Profile {
            /// Biography
            pub bio: Option<String>,

            /// Website URL
            pub website: Option<String>,

            /// Location
            pub location: String,

            /// Registration date
            pub joined_at: chrono::NaiveDateTime,

            /// Last login
            pub last_login: Option<chrono::NaiveDateTime>,

            /// Avatar URLs
            pub avatars: HashMap<String, String>,
        }
    "#;

    // Create parser and parse types
    let parser = TypeParser::new();
    let types = parser.parse_file(rust_code, "test_types.rs").unwrap();

    assert_eq!(types.len(), 8, "Should parse 8 types");
    assert!(types.iter().any(|t| t.name == "User"));
    assert!(types.iter().any(|t| t.name == "UserSettings"));
    assert!(types.iter().any(|t| t.name == "Profile"));

    // Test TypeScript generation
    let generator = create_typescript_generator().unwrap();
    let result = generator
        .generate_types_from_source(rust_code, "test_types.rs", &[])
        .await
        .unwrap();

    // Validate generated TypeScript
    assert!(result.content.contains("export interface User"));
    assert!(result.content.contains("export interface UserSettings"));
    assert!(result.content.contains("export interface Profile"));
    assert!(result.content.contains("export type Theme"));
    assert!(result.content.contains("export type Language"));
    assert!(result.content.contains("export type AccountStatus"));

    // Check type mappings
    assert!(result.content.contains("id: number"));
    assert!(result.content.contains("name: string"));
    assert!(result.content.contains("email?: string"));
    assert!(result.content.contains("age: number"));
    assert!(result.content.contains("permissions: Array<string>"));

    // Test cross-platform validation
    let validation = validate_cross_platform(&types, &default_config())
        .await
        .unwrap();

    assert!(validation.compatible);
    assert!(validation.compatibility_score >= 0.8);
    assert!(validation.issues.len() <= 2); // Allow some minor issues

    println!("✅ Complete generation pipeline test passed");
    println!(
        "📊 Compatibility Score: {:.1}%",
        validation.compatibility_score * 100.0
    );
    println!("⚠️  Issues found: {}", validation.issues.len());

    for issue in &validation.issues {
        println!("   - {}: {}", issue.source_type, issue.description);
    }
}

/// Test advanced configuration options
#[tokio::test]
async fn test_advanced_configuration() {
    let rust_code = r#"
        pub struct TestType {
            pub name: String,
            pub value: i32,
        }
    "#;

    // Test different naming conventions
    let mut config = GenerationConfig::preset_development();
    config.typescript.naming_convention = NamingConvention::CamelCase;
    config.typescript.generate_type_guards = true;

    let generator = TypeGenerator::with_full_config(config).unwrap();
    let result = generator
        .generate_types_from_source(rust_code, "test.rs", &[])
        .await
        .unwrap();

    // Should use camelCase for generated interface
    assert!(result.content.contains("export interface testType"));

    // Test PascalCase
    let mut config = GenerationConfig::default();
    config.typescript.naming_convention = NamingConvention::PascalCase;

    let generator = TypeGenerator::with_full_config(config).unwrap();
    let result = generator
        .generate_types_from_source(rust_code, "test.rs", &[])
        .await
        .unwrap();

    assert!(result.content.contains("export interface TestType"));
}

/// Test complex nested types and generics
#[tokio::test]
async fn test_complex_nested_types() {
    let rust_code = r#"
        pub struct ApiResponse<T> {
            pub success: bool,
            pub data: Option<T>,
            pub error: Option<String>,
            pub metadata: ResponseMetadata,
        }

        pub struct ResponseMetadata {
            pub request_id: String,
            pub processing_time: f64,
            pub server_version: String,
        }

        pub struct UserList {
            pub users: Vec<User>,
            pub total_count: i64,
            pub page: i32,
            pub per_page: i32,
        }

        pub struct User {
            pub profile: UserProfile,
        }

        pub struct UserProfile {
            pub id: String,
            pub name: String,
            pub preferences: HashMap<String, String>,
        }
    "#;

    let generator = create_typescript_generator().unwrap();
    let result = generator
        .generate_types_from_source(rust_code, "complex_types.rs", &[])
        .await
        .unwrap();

    // Check generic type handling
    assert!(result.content.contains("export interface ApiResponse<T>"));

    // Check array handling
    assert!(result.content.contains("users: Array<User>"));

    // Check nested object handling
    assert!(result.content.contains("profile: UserProfile"));
    assert!(result
        .content
        .contains("preferences: Record<string, string>"));
}

/// Test error handling and validation
#[tokio::test]
async fn test_error_handling_and_validation() {
    // Test with invalid Rust code
    let invalid_code = r#"
        pub struct InvalidType {
            pub field: NonExistentType,
        }
    "#;

    let parser = TypeParser::new();
    let types = parser.parse_file(invalid_code, "invalid.rs").unwrap();

    let generator = create_typescript_generator().unwrap();
    let result = generator
        .generate_types_from_source(invalid_code, "invalid.rs", &[])
        .await
        .unwrap();

    // Should still generate, but with unknown types
    assert!(result.content.contains("field: any"));

    // Test validation on complex inheritance scenario
    let complex_code = r#"
        pub union ComplexUnion {
            pub field1: String,
            pub field2: i32,
        }
    "#;

    let parser = TypeParser::new();
    let types = parser.parse_file(complex_code, "complex.rs").unwrap();

    let validation = validate_cross_platform(&types, &default_config())
        .await
        .unwrap();

    // Union types might have compatibility issues
    assert!(!validation.issues.is_empty());
}

/// Test caching integration
#[tokio::test]
async fn test_caching_integration() {
    let config = GenerationConfig::default();
    assert!(config.cache.enabled);

    let rust_code = r#"
        pub struct CacheTest {
            pub value: String,
        }
    "#;

    let generator = create_typescript_generator().unwrap();

    // First generation
    let result1 = generator
        .generate_types_from_source(rust_code, "cache_test.rs", &[])
        .await
        .unwrap();

    // Second generation (should potentially use cache)
    let result2 = generator
        .generate_types_from_source(rust_code, "cache_test.rs", &[])
        .await
        .unwrap();

    assert_eq!(result1.content, result2.content);
}

/// Test plugin system integration
#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_system_integration() {
    use crate::plugins::PluginSystem;

    let mut plugin_system = PluginSystem::new();
    let plugins = plugin_system.load_plugins().await.unwrap();

    // Should have built-in plugins
    assert!(!plugins.is_empty());

    // Test JSON schema plugin
    let json_transformer = plugin_system.get_transformer("json-schema-transformer");
    assert!(json_transformer.is_some());

    // Test Python generator plugin
    let python_generator = plugin_system.get_generator("python-generator");
    assert!(python_generator.is_some());
}

/// Test type transformation rules
#[tokio::test]
async fn test_type_transformation_rules() {
    use crate::transformers::TypeTransformer;

    let mut transformer = TypeTransformer::new();

    // Add custom transformation rule
    transformer.add_custom_rule(crate::transformers::TransformationRule {
        source_platform: "rust".to_string(),
        target_platform: "typescript".to_string(),
        type_pattern: "CustomType".to_string(),
        transformation: "string".to_string(),
        priority: 10,
    });

    let context = TransformationContext {
        source_platform: "rust".to_string(),
        target_platform: "typescript".to_string(),
        type_mappings: HashMap::new(),
        rules: HashMap::new(),
        options: HashMap::new(),
    };

    let result = transformer
        .transform_type("CustomType", &context)
        .await
        .unwrap();
    assert_eq!(result.transformed_type, "string");
    assert!(result.success);
}

/// Test utility functions
#[test]
fn test_utility_functions() {
    use crate::utils::*;

    // Test string utilities
    assert_eq!(string_utils::to_pascal_case("hello_world"), "HelloWorld");
    assert_eq!(string_utils::to_camel_case("hello_world"), "helloWorld");
    assert_eq!(string_utils::to_snake_case("HelloWorld"), "hello_world");

    // Test file utilities
    assert!(file_utils::is_rust_file(std::path::Path::new("test.rs")));
    assert!(!file_utils::is_rust_file(std::path::Path::new("test.ts")));

    // Test validation utilities
    let rust_code = r#"
        pub struct Type1 {
            pub field: String,
        }

        pub struct Type1 {  // Duplicate
            pub field: String,
        }
    "#;

    let parser = TypeParser::new();
    let types = parser.parse_file(rust_code, "duplicate.rs").unwrap();

    let result = validation_utils::check_unique_names(&types);
    assert!(result.is_err());
}

/// Test comprehensive error scenarios
#[tokio::test]
async fn test_error_scenarios() {
    // Test with empty source
    let generator = create_typescript_generator().unwrap();
    let result = generator
        .generate_types_from_source("", "empty.rs", &[])
        .await
        .unwrap();
    assert!(result.content.is_empty());

    // Test with only comments
    let comment_only = r#"
        /// This is just a comment
        // Another comment
    "#;

    let result = generator
        .generate_types_from_source(comment_only, "comments.rs", &[])
        .await
        .unwrap();
    assert!(result.content.is_empty());
}

/// Performance test for large type sets
#[tokio::test]
async fn test_large_type_set_performance() {
    let mut large_code = String::new();

    // Generate many similar types
    for i in 1..=50 {
        large_code.push_str(&format!(
            r#"
            pub struct Type{} {{
                pub id: u32,
                pub name: String,
                pub data: Vec<SubType{}>,
            }}

            pub struct SubType{} {{
                pub value: String,
                pub count: i32,
            }}
        "#,
            i, i, i
        ));
    }

    let start = std::time::Instant::now();
    let generator = create_typescript_generator().unwrap();
    let result = generator
        .generate_types_from_source(&large_code, "large.rs", &[])
        .await
        .unwrap();
    let duration = start.elapsed();

    // Should complete in reasonable time (< 1 second for 50 types)
    assert!(duration < std::time::Duration::from_secs(1));
    assert!(result.source_types.len() > 50); // Should have parsed all types

    println!("✅ Large type set performance test passed");
    println!(
        "📊 Generated {} types in {:?}",
        result.source_types.len(),
        duration
    );
}

/// Stress test for concurrent operations
#[tokio::test]
async fn test_concurrent_operations() {
    let generator = create_typescript_generator().unwrap();

    let tasks = (1..=10).map(|i| {
        let gen = generator.clone();
        tokio::spawn(async move {
            let code = format!(
                r#"
                pub struct ConcurrentType{} {{
                    pub id: u32,
                    pub name: String,
                }}
            "#,
                i
            );

            gen.generate_types_from_source(&code, &format!("concurrent_{}.rs", i), &[])
                .await
        })
    });

    let results = futures::future::join_all(tasks).await;

    for result in results {
        let result = result.unwrap().unwrap();
        assert!(result.content.contains("export interface"));
        assert!(result.content.contains("id: number"));
        assert!(result.content.contains("name: string"));
    }
}

/// Test configuration file operations
#[tokio::test]
async fn test_configuration_file_operations() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = GenerationConfig::preset_production();
    let temp_file = NamedTempFile::new().unwrap();

    // Save configuration
    config.to_file(temp_file.path()).unwrap();

    // Load configuration
    let loaded_config = GenerationConfig::from_file(temp_file.path()).unwrap();

    assert_eq!(
        config.typescript.target_version,
        loaded_config.typescript.target_version
    );
    assert_eq!(config.cache.enabled, loaded_config.cache.enabled);

    // Test validation
    assert!(config.validate().is_ok());
}

/// Test edge cases in type parsing
#[test]
fn test_edge_cases_in_parsing() {
    let parser = TypeParser::new();

    // Test with derive attributes
    let with_derives = r#"
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct WithDerives {
            pub user_name: String,
            pub is_active: bool,
        }
    "#;

    let types = parser.parse_file(with_derives, "derives.rs").unwrap();
    assert_eq!(types.len(), 1);
    assert!(types[0].attributes.contains(&"derive(Debug)".to_string()));

    // Test empty struct
    let empty_struct = r#"
        pub struct EmptyStruct;
    "#;

    let types = parser.parse_file(empty_struct, "empty.rs").unwrap();
    assert_eq!(types[0].fields.len(), 0);

    // Test tuple struct
    let tuple_struct = r#"
        pub struct TupleStruct(String, i32, bool);
    "#;

    let types = parser.parse_file(tuple_struct, "tuple.rs").unwrap();
    assert_eq!(types[0].fields.len(), 3);
    assert_eq!(types[0].fields[0].name, "field_0");
    assert_eq!(types[0].fields[1].name, "field_1");
}

/// Integration test with external dependencies
#[tokio::test]
async fn test_external_dependency_integration() {
    // This test would validate integration with other workspace crates
    // For now, just test that imports work correctly

    use rust_ai_ide_common::types::*; // Should work without conflicts
    use rust_ai_ide_errors::*; // Should work without conflicts

    let rust_code = r#"
        use rust_ai_ide_common::types::ProgrammingLanguage;
        pub struct IntegrationTest {
            pub language: ProgrammingLanguage,
        }
    "#;

    let generator = create_typescript_generator().unwrap();
    let result = generator
        .generate_types_from_source(rust_code, "integration.rs", &[])
        .await
        .unwrap();

    // Should handle external types gracefully
    assert!(result.content.contains("language: any"));
}

/// Benchmark-style test for continuous monitoring
#[tokio::test]
async fn test_generation_consistency() {
    let rust_code = r#"
        pub struct ConsistencyTest {
            pub field1: String,
            pub field2: i32,
            pub field3: bool,
        }
    "#;

    let generator = create_typescript_generator().unwrap();

    // Generate multiple times to ensure consistency
    let mut results = Vec::new();
    for _ in 0..5 {
        let result = generator
            .generate_types_from_source(rust_code, "consistency.rs", &[])
            .await
            .unwrap();
        results.push(result.content);
    }

    // All results should be identical
    let first = &results[0];
    for result in &results[1..] {
        assert_eq!(first, result, "Generation results should be consistent");
    }
}
