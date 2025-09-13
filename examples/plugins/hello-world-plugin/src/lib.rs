use async_trait::async_trait;
use rust_ai_ide_plugins::interfaces::{
    Plugin, PluginCapabilities, PluginContext, PluginError, PluginMetadata,
};
use rust_ai_ide_plugins::plugin::PluginResult;
use semver::Version;
use uuid::Uuid;

// Hello World plugin implementation
pub struct HelloWorldPlugin {
    metadata: PluginMetadata,
    capabilities: PluginCapabilities,
    is_loaded: bool,
}

impl HelloWorldPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            Uuid::new_v4(),
            "Hello World Plugin".to_string(),
            Version::parse("1.0.0").unwrap_or_else(|_| Version::new(1, 0, 0)),
            "Rust AI IDE Team".to_string(),
            "A simple hello world plugin that demonstrates the plugin API".to_string(),
        );

        let capabilities = PluginCapabilities::new()
            .with_command("hello-world")
            .with_command("hello-user")
            .with_feature("example");

        Self {
            metadata,
            capabilities,
            is_loaded: false,
        }
    }
}

#[async_trait]
impl Plugin for HelloWorldPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn capabilities(&self) -> &PluginCapabilities {
        &self.capabilities
    }

    async fn load(&mut self, _context: &PluginContext) -> PluginResult<()> {
        println!("Hello World Plugin loaded!");
        self.is_loaded = true;
        Ok(())
    }

    async fn unload(&mut self, _context: &PluginContext) -> PluginResult<()> {
        println!("Hello World Plugin unloaded!");
        self.is_loaded = false;
        Ok(())
    }

    async fn execute_command(
        &mut self,
        command: &str,
        args: Vec<String>,
        _context: &PluginContext,
    ) -> PluginResult<String> {
        if !self.is_loaded {
            return Err(PluginError::NotLoaded);
        }

        match command {
            "hello-world" => Ok("Hello, World!".to_string()),
            "hello-user" => {
                let user = args.get(0).map(|s| s.as_str()).unwrap_or("Anonymous");
                Ok(format!("Hello, {}!", user))
            }
            _ => Err(PluginError::CommandNotFound(command.to_string())),
        }
    }
}

/// Export function to create a new plugin instance
/// This is the entry point that the plugin loader will call
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    let plugin = Box::new(HelloWorldPlugin::new());
    Box::into_raw(plugin)
}

/// Export function to destroy a plugin instance
/// This should be called when the plugin is unloaded
#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn Plugin) {
    if !plugin.is_null() {
        unsafe {
            let _ = Box::from_raw(plugin);
        }
    }
}
