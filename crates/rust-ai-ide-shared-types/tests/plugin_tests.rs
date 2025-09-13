//! Tests for the plugin system functionality
//!
//! This module validates the plugin loading, execution, and
//! transformation capabilities of the shared types crate.

use rust_ai_ide_shared_types::*;
use std::collections::HashMap;

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_loading_and_execution() {
    use crate::plugins::*;

    let mut plugin_system = PluginSystem::new();
    let plugins = plugin_system.load_plugins().await.unwrap();

    // Should load built-in plugins
    assert!(!plugins.is_empty());

    // Check for expected built-in plugins
    let transformer_names: Vec<String> = plugins
        .iter()
        .filter_map(|p| {
            if p.instance.as_ref()?.is_transformer() {
                Some(p.metadata.name.clone())
            } else {
                None
            }
        })
        .collect();

    let generator_names: Vec<String> = plugins
        .iter()
        .filter_map(|p| {
            if p.instance.as_ref()?.is_generator() {
                Some(p.metadata.name.clone())
            } else {
                None
            }
        })
        .collect();

    assert!(transformer_names.contains(&"json-schema-transformer".to_string()));
    assert!(generator_names.contains(&"python-generator".to_string()));

    // Test plugin execution
    let transformer = plugin_system
        .get_transformer("json-schema-transformer")
        .unwrap();

    let test_type = ParsedType {
        name: "TestPluginType".to_string(),
        kind: TypeKind::Struct,
        documentation: Some("A test type for plugin validation".to_string()),
        visibility: Visibility::Public,
        generics: vec![],
        fields: vec![Field {
            name: "id".to_string(),
            ty: "String".to_string(),
            documentation: None,
            visibility: Visibility::Public,
            is_mutable: false,
            attributes: vec![],
        }],
        variants: vec![],
        associated_items: vec![],
        attributes: vec![],
        source_location: SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
            module_path: vec![],
        },
        dependencies: vec![],
        metadata: TypeMetadata::default(),
    };

    let context = TransformationContext {
        source_platform: "rust".to_string(),
        target_platform: "json-schema".to_string(),
        type_mappings: HashMap::new(),
        rules: HashMap::new(),
        options: HashMap::new(),
    };

    let result = transformer
        .transform_type(&test_type, &context)
        .await
        .unwrap();
    assert!(result.is_some());

    let generated = result.unwrap();
    assert!(generated.content.contains("TestPluginType"));
    assert!(generated.target_platform == "json-schema");
}

#[tokio::test]
async fn test_built_in_transformer_plugins() {
    let json_plugin = crate::plugins::JsonTransformerPlugin::new();

    // Test metadata
    let metadata = json_plugin.metadata();
    assert_eq!(metadata.name, "json-schema-transformer");
    assert_eq!(metadata.version, "1.0.0");
    assert!(metadata.platforms.contains(&"json-schema".to_string()));

    // Test simple transformation
    let simple_type = ParsedType {
        name: "Simple".to_string(),
        kind: TypeKind::Struct,
        documentation: None,
        visibility: Visibility::Public,
        generics: vec![],
        fields: vec![Field {
            name: "name".to_string(),
            ty: "String".to_string(),
            documentation: None,
            visibility: Visibility::Public,
            is_mutable: false,
            attributes: vec![],
        }],
        variants: vec![],
        associated_items: vec![],
        attributes: vec![],
        source_location: SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
            module_path: vec![],
        },
        dependencies: vec![],
        metadata: TypeMetadata::default(),
    };

    let context = TransformationContext {
        source_platform: "rust".to_string(),
        target_platform: "json-schema".to_string(),
        type_mappings: HashMap::new(),
        rules: HashMap::new(),
        options: HashMap::new(),
    };

    let result = json_plugin
        .transform_type(&simple_type, &context)
        .await
        .unwrap();
    assert!(result.is_some());

    let generated = result.unwrap();
    let json: serde_json::Value = serde_json::from_str(&generated.content).unwrap();
    assert!(json["properties"].is_object());
}

#[tokio::test]
async fn test_built_in_generator_plugins() {
    let python_plugin = crate::plugins::PythonGeneratorPlugin::new();

    // Test metadata
    let metadata = python_plugin.metadata();
    assert_eq!(metadata.name, "python-generator");
    assert!(metadata.target_platforms.contains(&"python".to_string()));
    assert!(metadata
        .supported_formats
        .contains(&"dataclass".to_string()));

    // Test dataclass generation
    let test_type = ParsedType {
        name: "Person".to_string(),
        kind: TypeKind::Struct,
        documentation: None,
        visibility: Visibility::Public,
        generics: vec![],
        fields: vec![
            Field {
                name: "name".to_string(),
                ty: "String".to_string(),
                documentation: None,
                visibility: Visibility::Public,
                is_mutable: false,
                attributes: vec![],
            },
            Field {
                name: "age".to_string(),
                ty: "i32".to_string(),
                documentation: None,
                visibility: Visibility::Public,
                is_mutable: false,
                attributes: vec![],
            },
        ],
        variants: vec![],
        associated_items: vec![],
        attributes: vec![],
        source_location: SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
            module_path: vec![],
        },
        dependencies: vec![],
        metadata: TypeMetadata::default(),
    };

    let config = serde_json::json!({"format": "dataclass"});
    let result = python_plugin
        .generate(&vec![test_type], "python-dataclasses", &config)
        .await
        .unwrap();

    assert!(result.content.contains("@dataclass"));
    assert!(result.content.contains("class Person"));
    assert!(result.content.contains("name: str"));
    assert!(result.content.contains("age: int"));
}

#[tokio::test]
async fn test_plugin_transformation_pipeline() {
    // Test the full plugin transformation pipeline
    let rust_code = r#"
        pub struct TransformTest {
            pub id: String,
            pub count: i32,
        }
    "#;

    // Create a custom transformer for testing
    struct CustomTransformer;
    impl CustomTransformer {
        pub fn transform_field_name(name: &str) -> String {
            format!("custom_{}", name)
        }
    }

    // Test field transformation
    let original_field = "user_id";
    let transformed = CustomTransformer::transform_field_name(original_field);
    assert_eq!(transformed, "custom_user_id");

    // Test through the plugin system
    let generator = create_typescript_generator().unwrap();
    let result = generator
        .generate_types_from_source(rust_code, "transform.rs", &[])
        .await
        .unwrap();

    assert!(result.content.contains("id: string"));
    assert!(result.content.contains("count: number"));
}

#[tokio::test]
async fn test_plugin_error_handling() {
    let json_plugin = crate::plugins::JsonTransformerPlugin::new();

    // Test with empty type (should handle gracefully)
    let empty_type = ParsedType {
        name: "Empty".to_string(),
        kind: TypeKind::Struct,
        documentation: None,
        visibility: Visibility::Public,
        generics: vec![],
        fields: vec![],
        variants: vec![],
        associated_items: vec![],
        attributes: vec![],
        source_location: SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
            module_path: vec![],
        },
        dependencies: vec![],
        metadata: TypeMetadata::default(),
    };

    let context = TransformationContext {
        source_platform: "rust".to_string(),
        target_platform: "json-schema".to_string(),
        type_mappings: HashMap::new(),
        rules: HashMap::new(),
        options: HashMap::new(),
    };

    let result = json_plugin
        .transform_type(&empty_type, &context)
        .await
        .unwrap();
    assert!(result.is_some());

    let generated = result.unwrap();
    let json: serde_json::Value = serde_json::from_str(&generated.content).unwrap();
    assert_eq!(json["type"], "object");
    assert!(json["properties"].as_object().unwrap().is_empty());
}

#[test]
fn test_plugin_metadata_validation() {
    let json_metadata = crate::plugins::JsonTransformerPlugin::new().metadata();
    let python_metadata = crate::plugins::PythonGeneratorPlugin::new().metadata();

    // Validate JSON plugin metadata
    assert!(!json_metadata.name.is_empty());
    assert!(!json_metadata.version.is_empty());
    assert!(!json_metadata.author.is_empty());
    assert!(!json_metadata.platforms.is_empty());
    assert!(json_metadata.platforms.contains(&"json-schema".to_string()));

    // Validate Python plugin metadata
    assert!(!python_metadata.name.is_empty());
    assert!(!python_metadata.version.is_empty());
    assert!(python_metadata
        .target_platforms
        .contains(&"python".to_string()));
    assert!(python_metadata
        .supported_formats
        .contains(&"dataclass".to_string()));
}

#[tokio::test]
async fn test_plugin_platform_support() {
    let json_plugin = crate::plugins::JsonTransformerPlugin::new();
    let python_plugin = crate::plugins::PythonGeneratorPlugin::new();

    // Test platform support checking
    assert!(json_plugin.supports_platform("json-schema"));
    assert!(!json_plugin.supports_platform("typescript"));

    assert!(python_plugin
        .target_platforms()
        .contains(&"python".to_string()));
    assert!(python_plugin
        .target_platforms()
        .contains(&"python-dataclasses".to_string()));
    assert!(!python_plugin
        .target_platforms()
        .contains(&"invalid-platform".to_string()));

    // Test supported formats
    assert!(python_plugin
        .supported_formats()
        .contains(&"dataclass".to_string()));
    assert!(python_plugin
        .supported_formats()
        .contains(&"pydantic".to_string()));
    assert!(python_plugin
        .supported_formats()
        .contains(&"typeddict".to_string()));
}

#[tokio::test]
async fn test_multiple_plugin_execution() {
    // Test running multiple plugins sequentially
    let plugins = vec![
        ("json-schema-transformer", "json-schema"),
        ("python-generator", "python-dataclasses"),
    ];

    for (plugin_name, target_platform) in plugins {
        let plugin_system = crate::plugins::PluginSystem::new();
        let result = plugin_system.load_plugins().await.unwrap();

        let found_plugin = result.iter().any(|p| p.metadata.name == plugin_name);
        assert!(found_plugin, "Plugin {} should be loaded", plugin_name);
    }
}

#[tokio::test]
async fn test_plugin_configuration() {
    // Test plugin configuration handling
    let mut plugin_system = crate::plugins::PluginSystem::new();

    let config = serde_json::json!({
        "format": "pydantic",
        "include_imports": true,
        "generate_validators": false
    });

    plugin_system.configure_plugin("python-generator", config.clone());

    let stored_config = plugin_system.get_plugin_config("python-generator");
    assert!(stored_config.is_some());
    assert_eq!(stored_config.unwrap()["format"], "pydantic");
}

#[tokio::test]
async fn test_plugin_validation() {
    let json_plugin = crate::plugins::JsonTransformerPlugin::new();
    let python_plugin = crate::plugins::PythonGeneratorPlugin::new();

    // Should not error during validation
    json_plugin.validate().await.unwrap();
    python_plugin.validate().await.unwrap();
}

#[test]
fn test_plugin_instance_types() {
    use crate::plugins::{GeneratorPluginTrait, PluginInstance, TransformerPluginTrait};

    let transformer = crate::plugins::JsonTransformerPlugin::new();
    let generator = crate::plugins::PythonGeneratorPlugin::new();

    let transformer_instance = PluginInstance::Transformer(std::sync::Arc::new(transformer));
    let generator_instance = PluginInstance::Generator(std::sync::Arc::new(generator));

    // Test that instances work as expected
    match transformer_instance {
        PluginInstance::Transformer(_) => assert!(true),
        _ => panic!("Should be transformer"),
    }

    match generator_instance {
        PluginInstance::Generator(_) => assert!(true),
        _ => panic!("Should be generator"),
    }
}

#[tokio::test]
async fn test_plugin_extensions() {
    let json_plugin = crate::plugins::JsonTransformerPlugin::new();

    // Test file extension support
    assert!(json_plugin
        .supported_extensions()
        .contains(&"json".to_string()));

    // Test multiple types at once
    let types = vec![
        ParsedType {
            name: "Type1".to_string(),
            kind: TypeKind::Struct,
            documentation: None,
            visibility: Visibility::Public,
            generics: vec![],
            fields: vec![],
            variants: vec![],
            associated_items: vec![],
            attributes: vec![],
            source_location: SourceLocation {
                file: "test.rs".to_string(),
                line: 1,
                column: 1,
                module_path: vec![],
            },
            dependencies: vec![],
            metadata: TypeMetadata::default(),
        },
        ParsedType {
            name: "Type2".to_string(),
            kind: TypeKind::Struct,
            documentation: None,
            visibility: Visibility::Public,
            generics: vec![],
            fields: vec![],
            variants: vec![],
            associated_items: vec![],
            attributes: vec![],
            source_location: SourceLocation {
                file: "test.rs".to_string(),
                line: 10,
                column: 1,
                module_path: vec![],
            },
            dependencies: vec![],
            metadata: TypeMetadata::default(),
        },
    ];

    let context = TransformationContext {
        source_platform: "rust".to_string(),
        target_platform: "json-schema".to_string(),
        type_mappings: HashMap::new(),
        rules: HashMap::new(),
        options: HashMap::new(),
    };

    let result = json_plugin.transform_file(&types, &context).await.unwrap();
    assert!(result.is_some());
    let generated = result.unwrap();
    let json: serde_json::Value = serde_json::from_str(&generated.content).unwrap();
    assert!(json["properties"]
        .as_object()
        .unwrap()
        .contains_key("Type1"));
    assert!(json["properties"]
        .as_object()
        .unwrap()
        .contains_key("Type2"));
}
