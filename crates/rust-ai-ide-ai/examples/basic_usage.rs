use anyhow::Result;
use rust_ai_ide_ai::model_loader::*;
use std::println;

/// Basic usage example demonstrating model registry functionality
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Rust AI IDE - Basic Usage Example\n");

    // Create a new model registry with default settings
    println!("📝 Creating model registry...");
    let registry = ModelRegistry::with_policy(UnloadingPolicy::LRU { max_age_hours: 24 });

    // Set up background automatic unloading task
    println!("🎯 Starting background unloading task...");
    let _handle = registry.start_auto_unloading_task(600).await; // Every 10 minutes

    // Get system resource information
    println!("💻 System Resource Status:");
    let (used_mb, total_mb, percentage) = registry.get_system_resource_info().await;
    println!(
        "   Memory: {:.0}MB / {:.0}MB ({:.1}%)",
        used_mb as f64 / (1024.0 * 1024.0),
        total_mb as f64 / (1024.0 * 1024.0),
        percentage
    );

    // Load a model (note: this will fail without actual model file, but demonstrates API)
    println!("\n🔄 Attempting to load a model...");
    println!("   Note: This demo shows API usage - model loading requires actual model files");

    let model_path = "/tmp/demo_model.bin"; // This doesn't exist, but shows usage
    match registry.load_model(ModelType::CodeLlama, model_path).await {
        Ok(model_id) => {
            println!("   ✅ Model loaded successfully with ID: {}", model_id);

            // Update access time for LRU policy
            if let Err(e) = registry.update_model_access(&model_id).await {
                println!("   ⚠️ Failed to update model access: {}", e);
            }

            // Get loaded models
            let loaded_models = registry.get_loaded_models().await;
            println!("   📊 Currently loaded models: {}", loaded_models.len());

            // Get resource statistics
            if let Some(total_memory) = registry
                .get_total_memory_usage()
                .await
                .checked_div(1024 * 1024)
            {
                println!("   📈 Total memory usage: {} MB", total_memory);
            }

            // Unload the model
            if let Err(e) = registry.unload_model(&model_id).await {
                println!("   ⚠️ Failed to unload model: {}", e);
            } else {
                println!("   📤 Model unloaded successfully");
            }
        }
        Err(e) => {
            println!("   ⚠️ Expected error (model file doesn't exist): {}", e);
        }
    }

    // Demonstrate auto-unloading trigger
    println!("\n🧹 Demonstrating automatic unloading evaluation...");
    match registry.auto_unload_models().await {
        Ok(models_to_unload) => {
            println!(
                "   📋 Models recommended for auto-unload: {}",
                models_to_unload.len()
            );
            for model_id in &models_to_unload {
                println!("     - {}", model_id);
            }

            if models_to_unload.is_empty() {
                println!("   ✨ No models need unloading at this time");
            }
        }
        Err(e) => {
            println!("   ⚠️ Auto-unload evaluation failed: {}", e);
        }
    }

    // Demonstrate different unloading policies
    println!("\n🔧 Demonstrating unloading policy configurations:");

    let policies = vec![
        ("LRU", UnloadingPolicy::LRU { max_age_hours: 24 }),
        (
            "Memory Threshold",
            UnloadingPolicy::MemoryThreshold {
                max_memory_gb: 16.0,
            },
        ),
        (
            "Time-based",
            UnloadingPolicy::TimeBased { max_age_hours: 48 },
        ),
        (
            "Hybrid",
            UnloadingPolicy::Hybrid {
                max_age_hours: 24,
                max_memory_gb: 12.0,
            },
        ),
    ];

    for (name, policy) in policies {
        let test_registry = ModelRegistry::with_policy(policy.clone());
        println!("   {}: {:?} configured", name, policy);

        // Get unloading policy
        println!(
            "     Current policy: {:?}",
            test_registry.get_unloading_policy()
        );
    }

    // Demonstrate resource refresh
    println!("\n🔄 Refreshing system resource information...");
    registry.refresh_system_resources().await;
    println!("   ✅ System resources refreshed");

    println!("\n🎉 Basic usage example completed successfully!");
    println!("\n💡 To use with real models:");
    println!("   1. Place model files in appropriate directory");
    println!("   2. Update model_path variable with actual file path");
    println!("   3. Ensure sufficient system memory for model loading");
    println!("   4. Consider setting appropriate unloading policies based on your use case");

    Ok(())
}
