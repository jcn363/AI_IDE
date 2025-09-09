use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::println;
use std::sync::Arc;

use rust_ai_ide_ai::model_loader::*;

/// Custom model loader for demonstration purposes
#[derive(Debug)]
struct CustomModelLoader {
    supported_types: Vec<ModelType>,
}

impl CustomModelLoader {
    fn new() -> Self {
        Self {
            supported_types: vec![ModelType::CodeLlama, ModelType::StarCoder],
        }
    }
}

#[async_trait]
impl ModelLoader for CustomModelLoader {
    async fn load_model(&self, model_path: &str) -> Result<ModelHandle> {
        println!("🎨 Custom loader: Loading model from {}", model_path);

        // Validate file exists
        if !std::path::Path::new(model_path).exists() {
            return Err(anyhow!("Model file not found: {}", model_path));
        }

        // Estimate memory usage based on file size
        let metadata = std::fs::metadata(model_path)?;
        let file_size = metadata.len();
        let estimated_memory = (file_size / 1024 * 1024 * 100).max(1024 * 1024 * 200); // At least 200MB

        // Determine model type from path (simple heuristic)
        let model_type = if model_path.to_lowercase().contains("code_llama")
            || model_path.to_lowercase().contains("codellama")
        {
            ModelType::CodeLlama
        } else if model_path.to_lowercase().contains("star_coder")
            || model_path.to_lowercase().contains("starcoder")
        {
            ModelType::StarCoder
        } else {
            return Err(anyhow!(
                "Unable to determine model type from path: {}",
                model_path
            ));
        };

        // Determine size based on file size
        let size = if file_size > 4 * 1024 * 1024 * 1024 {
            // > 4GB
            ModelSize::ExtraLarge
        } else if file_size > 2 * 1024 * 1024 * 1024 {
            // > 2GB
            ModelSize::Large
        } else if file_size > 1 * 1024 * 1024 * 1024 {
            // > 1GB
            ModelSize::Medium
        } else {
            ModelSize::Small
        };

        // Create model handle
        let handle = ModelHandle::new(
            format!(
                "custom_{}_{}",
                model_type as u8,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs()
            ),
            std::path::PathBuf::from(model_path),
            size,
            model_type,
            estimated_memory,
        );

        println!("   ✅ Custom model loaded:");
        println!("     - ID: {}", handle.id);
        println!("     - Type: {:?}", handle.model_type);
        println!("     - Size: {:?}", handle.size);
        println!(
            "     - Estimated Memory: {:.1} MB",
            handle.memory_usage_mb()
        );

        Ok(handle)
    }

    async fn unload_model(&self, model_id: &str) -> Result<()> {
        println!("🎨 Custom loader: Unloading model {}", model_id);

        // In a real implementation, this would unload the actual model
        // from memory, close connections, clean up resources, etc.
        println!("   📤 Model unloaded successfully (simulated)");
        Ok(())
    }

    fn get_supported_sizes(&self) -> &'static [ModelSize] {
        &[
            ModelSize::Small,
            ModelSize::Medium,
            ModelSize::Large,
            ModelSize::ExtraLarge,
        ]
    }

    fn get_model_type(&self) -> ModelType {
        // This method is called but we override behavior in load_model
        ModelType::CodeLlama
    }
}

/// Demonstrate custom model loader implementation
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Custom Model Loader Example\n");

    // Create custom loader
    println!("🔧 Creating custom model loader...");
    let custom_loader = CustomModelLoader::new();

    // Demonstrate loader capabilities
    println!("📊 Loader Capabilities:");
    println!(
        "   - Supported sizes: {:?}",
        custom_loader.get_supported_sizes()
    );

    // For demonstration, we'll create a fake model path
    println!("\n🧪 Demonstration (using fake model path):");

    let fake_model_path = "/tmp/custom_codellama_model.bin";
    println!("   Model path: {}", fake_model_path);

    // Try loading (will fail because file doesn't exist)
    match custom_loader.load_model(fake_model_path).await {
        Ok(handle) => {
            println!("   ✅ Unexpected success: {}", handle.id);
        }
        Err(e) => {
            println!("   ⚠️ Expected error: {}", e);
        }
    }

    // Demonstrate creating a temporary test file
    println!("\n📝 Creating temporary test file for demonstration:");
    let temp_path = "/tmp/custom_test_model.bin";
    println!("   Creating test file: {}", temp_path);

    // Create a small test file (1MB)
    std::fs::write(temp_path, vec![0u8; 1024 * 1024])?;
    println!("   ✅ Test file created (1MB)");

    // Now try loading with the actual file
    match custom_loader.load_model(temp_path).await {
        Ok(handle) => {
            println!("   ✅ Model loaded from test file!");
            println!("     ID: {}", handle.id);
            println!(
                "     Expected memory usage: {:.1} MB",
                handle.memory_usage_mb()
            );
        }
        Err(e) => {
            println!("   ❌ Failed to load test model: {}", e);
        }
    }

    // Clean up test file
    if let Err(e) = std::fs::remove_file(temp_path) {
        println!("   ⚠️ Failed to clean up test file: {}", e);
    } else {
        println!("   🧹 Test file cleaned up");
    }

    // Demonstrate integrating with registry
    println!("\n🔧 Integrating Custom Loader with Registry:");

    let registry = ModelRegistry::new();
    println!("   📋 Custom loader could be registered with:");
    println!("     - LoaderFactory::create_loader() for singleton usage");
    println!("     - AnalysisRegistry for batch processing");
    println!("     - Direct implementation for specific use cases");

    // Show how to create an Arc<dyn ModelLoader> for registry compatibility
    let arc_loader: Arc<dyn ModelLoader> = Arc::new(CustomModelLoader::new());
    println!(
        "   ✅ Compatible with registry: {:?}",
        arc_loader.get_supported_sizes()
    );

    println!("\n🎉 Custom loader example completed!");
    println!("\n💡 Key Points:");
    println!("   • Implement ModelLoader trait for custom loading logic");
    println!("   • Handle async loading/unloading operations");
    println!("   • Compatible with Registry system for resource management");
    println!("   • Can be extended with ResourceAwareLoader for advanced features");

    Ok(())
}
