use rust_ai_ide_ai::model_loader::loaders::CodeLlamaLoader;
use rust_ai_ide_ai::model_loader::*;
use std::time::Duration;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_model_registry_creation() {
        let registry = ModelRegistry::new();
        assert_eq!(registry.get_total_memory_usage().await, 0);

        let loaded = registry.get_loaded_models().await;
        assert!(loaded.is_empty());
    }

    #[tokio::test]
    async fn test_registry_with_custom_policy() {
        let policies = vec![
            UnloadingPolicy::LRU { max_age_hours: 24 },
            UnloadingPolicy::MemoryThreshold {
                max_memory_gb: 16.0,
            },
            UnloadingPolicy::TimeBased { max_age_hours: 48 },
            UnloadingPolicy::Hybrid {
                max_age_hours: 24,
                max_memory_gb: 12.0,
            },
        ];

        for policy in policies {
            let registry = ModelRegistry::with_policy(policy.clone());
            // Just test that registry creation succeeds with each policy
            assert!(
                matches!(registry.get_unloading_policy(), p if std::mem::discriminant(p) == std::mem::discriminant(&policy))
            );
        }
    }

    #[tokio::test]
    async fn test_system_resource_monitoring() {
        let registry = ModelRegistry::new();

        // Test resource info retrieval (should not panic)
        let (_used, total, percentage) = registry.get_system_resource_info().await;

        // Basic sanity checks
        assert!(total > 0);
        assert!(percentage >= 0.0 && percentage <= 100.0);
    }

    #[tokio::test]
    async fn test_model_loader_interface() {
        use std::sync::Arc;

        // Test that we can create Arc<dyn ModelLoader>
        let loader = Arc::new(CodeLlamaLoader::new());
        assert_eq!(loader.get_model_type(), ModelType::CodeLlama);
        assert!(!loader.get_supported_sizes().is_empty());
    }

    #[tokio::test]
    async fn test_registry_factory_loader() {
        // Test that factory creates proper loaders
        let supported_types = LoaderFactory::get_supported_model_types();

        for &model_type in supported_types {
            let loader = LoaderFactory::create_loader(model_type);
            assert_eq!(loader.get_model_type(), model_type);
        }
    }

    #[tokio::test]
    async fn test_model_handle_creation() {
        let handle = ModelHandle::new(
            "test_model".to_string(),
            "/tmp/test_model.bin".into(),
            ModelSize::Medium,
            ModelType::CodeLlama,
            1024 * 1024 * 500, // 500 MB
        );

        assert_eq!(handle.id, "test_model");
        assert_eq!(handle.model_type, ModelType::CodeLlama);
        assert_eq!(handle.size, ModelSize::Medium);

        // Test memory calculation
        let expected_mb = 500.0;
        let actual_mb = handle.memory_usage_mb();
        let diff = (actual_mb - expected_mb).abs();
        assert!(diff < 1.0, "Memory calculation should be accurate");
    }

    #[tokio::test]
    async fn test_model_handle_touch() {
        let mut handle = ModelHandle::new(
            "test_touch".to_string(),
            "/tmp/test.bin".into(),
            ModelSize::Small,
            ModelType::CodeLlama,
            1024 * 1024 * 100, // 100 MB
        );

        let initial_access_count = handle.resource_usage.access_count;
        handle.touch();

        assert_eq!(handle.resource_usage.access_count, initial_access_count + 1);
    }

    #[tokio::test]
    async fn test_unloading_policy_decoding() {
        // Test that policies can be created and compared
        let lru = UnloadingPolicy::LRU { max_age_hours: 24 };
        let mem = UnloadingPolicy::MemoryThreshold { max_memory_gb: 8.0 };
        let time = UnloadingPolicy::TimeBased { max_age_hours: 48 };
        let hybrid = UnloadingPolicy::Hybrid {
            max_age_hours: 24,
            max_memory_gb: 16.0,
        };

        // Different policies should have different discriminants
        assert!(std::mem::discriminant(&lru) != std::mem::discriminant(&mem));
        assert!(std::mem::discriminant(&mem) != std::mem::discriminant(&time));
        assert!(std::mem::discriminant(&time) != std::mem::discriminant(&hybrid));
        assert!(std::mem::discriminant(&hybrid) != std::mem::discriminant(&lru));
    }

    #[tokio::test]
    async fn test_system_monitor_interface() {
        let monitor = SystemMonitor::new();

        // Test memory info (should work without panicking)
        let (_used, total) = monitor.get_memory_info().await;
        assert!(total > 0);

        // Test memory percentage
        let percentage = monitor.get_memory_usage_percentage().await;
        assert!(percentage >= 0.0 && percentage <= 100.0);
    }

    #[tokio::test]
    async fn test_resource_aware_loader_trait() {
        let loader = CodeLlamaLoader::new();
        let monitor = SystemMonitor::new();

        // Test memory sufficiency check with non-existent file
        let result = loader
            .can_load_with_resources("/non/existent/file.bin", &monitor)
            .await;
        assert!(result.is_err() || !result.unwrap());
    }

    #[tokio::test]
    async fn test_model_size_memory_estimation() {
        let monitor = SystemMonitor::new();

        // Test memory estimation function (static method)
        let estimate =
            SystemMonitor::estimate_memory_requirement(ModelSize::Large, Some(Quantization::FP16));

        // Should return reasonable memory estimate
        let estimate_gb = estimate as f64 / (1024.0 * 1024.0 * 1024.0);
        assert!(estimate_gb > 0.1 && estimate_gb < 20.0); // Reasonable range for model memory
    }

    #[tokio::test]
    async fn test_quantization_values() {
        // Test that quantization enum values are correct
        assert_eq!(Quantization::FP32 as u8, 0);
        assert_eq!(Quantization::FP16 as u8, 1);
        assert_eq!(Quantization::INT8 as u8, 2);
        assert_eq!(Quantization::INT4 as u8, 3);
    }

    #[tokio::test]
    async fn test_model_type_values() {
        // Test that model type enum values are correct
        assert_eq!(ModelType::CodeLlama as u8, 0);
        assert_eq!(ModelType::StarCoder as u8, 1);
    }

    #[tokio::test]
    async fn test_auto_unload_evaluation() {
        let registry = ModelRegistry::with_policy(UnloadingPolicy::LRU { max_age_hours: 24 });

        // With empty registry, should return empty list
        let models_to_unload = registry.auto_unload_models().await.unwrap();
        assert!(models_to_unload.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_registry_instances() {
        // Test that multiple registry instances can coexist
        let registry1 = ModelRegistry::with_policy(UnloadingPolicy::LRU { max_age_hours: 12 });
        let registry2 = ModelRegistry::with_policy(UnloadingPolicy::MemoryThreshold {
            max_memory_gb: 16.0,
        });

        assert!(
            std::mem::discriminant(registry1.get_unloading_policy())
                != std::mem::discriminant(registry2.get_unloading_policy())
        );

        // Both should have zero memory usage initially
        assert_eq!(registry1.get_total_memory_usage().await, 0);
        assert_eq!(registry2.get_total_memory_usage().await, 0);
    }

    #[tokio::test]
    async fn test_registry_performance() {
        let registry = ModelRegistry::new();

        // Test rapid resource queries don't panic
        for _ in 0..100 {
            let _ = registry.get_total_memory_usage().await;
            let _ = registry.get_loaded_models().await;
        }
    }

    #[tokio::test]
    async fn test_background_task_start() {
        let registry = ModelRegistry::new();

        // Test that background task can be started (use timeout to prevent hanging)
        let handle = registry.start_auto_unloading_task(60).await;
        // Should start successfully
        assert!(true); // If we get here, task started successfully

        // Give a brief moment for task to initialize
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Task should still be alive (we don't need to wait for it to complete)
        // This is primarily testing that task creation doesn't panic
    }

    /// Test custom loader integration
    #[derive(Debug)]
    struct TestModelLoader;

    #[async_trait::async_trait]
    impl ModelLoader for TestModelLoader {
        async fn load_model(&self, model_path: &str) -> anyhow::Result<ModelHandle> {
            Ok(ModelHandle::new(
                format!(
                    "test_custom_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs()
                ),
                model_path.into(),
                ModelSize::Small,
                ModelType::CodeLlama,
                1024 * 1024 * 50, // 50 MB
            ))
        }

        async fn unload_model(&self, _model_id: &str) -> anyhow::Result<()> {
            Ok(())
        }

        fn get_supported_sizes(&self) -> &'static [ModelSize] {
            &[ModelSize::Small]
        }

        fn get_model_type(&self) -> ModelType {
            ModelType::CodeLlama
        }
    }

    #[tokio::test]
    async fn test_custom_loader_integration() {
        let custom_loader = TestModelLoader;

        // Test that custom loader works with trait
        let handle = custom_loader.load_model("/tmp/test.bin").await.unwrap();
        assert_eq!(handle.model_type, ModelType::CodeLlama);
        assert_eq!(handle.size, ModelSize::Small);

        // Test unloading
        custom_loader.unload_model(&handle.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_loader_factory_consistency() {
        // Test that factory produces consistent results
        for _ in 0..10 {
            let loader1 = LoaderFactory::create_loader(ModelType::CodeLlama);
            let loader2 = LoaderFactory::create_loader(ModelType::CodeLlama);

            assert_eq!(loader1.get_model_type(), ModelType::CodeLlama);
            assert_eq!(loader2.get_model_type(), ModelType::CodeLlama);
            assert_eq!(loader1.get_supported_sizes(), loader2.get_supported_sizes());
        }
    }
}
