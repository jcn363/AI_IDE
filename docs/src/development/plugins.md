# Plugin Development Guide

## Overview

Rust AI IDE supports a powerful plugin system that allows extending the IDE's functionality.

## Getting Started

1. Create a new plugin project:
   ```bash
   cargo new --lib my-plugin
   cd my-plugin
   ```

2. Add the plugin API dependency to `Cargo.toml`:
   ```toml
   [dependencies]
   rust_ai_ide_plugin = { git = "https://github.com/rust-ai-ide/rust-ai-ide" }
   ```

## Plugin Structure

A basic plugin looks like this:

```rust
use rust_ai_ide_plugin::*;

#[no_mangle]
pub extern "C" fn register() -> Plugin {
    Plugin {
        name: "My Plugin".into(),
        version: "0.1.0".into(),
        hooks: vec![
            Hook::new("on_editor_open", on_editor_open),
        ],
    }
}

fn on_editor_open(context: PluginContext) -> PluginResult {
    println!("Editor opened: {}", context.file_path);
    Ok(())
}
```

## Building and Installing

```bash
# Build the plugin
cargo build --release

# Install the plugin
mkdir -p ~/.config/rust-ai-ide/plugins
cp target/release/libmy_plugin.so ~/.config/rust-ai-ide/plugins/
```

## Available Hooks

- `on_editor_open`: When a file is opened in the editor
- `on_save`: When a file is saved
- `on_command`: When a command is executed

## Best Practices

- Keep plugins focused on a single task
- Handle errors gracefully
- Document your plugin's functionality
- Follow semantic versioning

## Example Plugins

- [Code Formatter](https://github.com/rust-ai-ide/examples/tree/main/plugins/code-formatter)
- [Git Integration](https://github.com/rust-ai-ide/examples/tree/main/plugins/git-integration)
- [Theme Manager](https://github.com/rust-ai-ide/examples/tree/main/plugins/theme-manager)

## Troubleshooting

- **Plugin not loading**: Check the logs at `~/.config/rust-ai-ide/logs/plugin.log`
- **API version mismatch**: Ensure your plugin is compatible with the current IDE version
- **Permission issues**: Make sure the plugin file has execute permissions

## Contributing

We welcome contributions! Please see our [Contributing Guide](../development/CONTRIBUTING.html) for details.
