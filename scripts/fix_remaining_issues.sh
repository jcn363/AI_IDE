#!/bin/bash
set -e

# Create missing QUICKSTART.md
cat > docs/src/getting-started/QUICKSTART.md << 'EOL'
# Quick Start Guide

## First Steps

1. **Install Rust AI IDE**
   ```bash
   # Using cargo
   cargo install rust-ai-ide
   
   # Or download from GitHub releases
   # Visit https://github.com/your-org/rust-ai-ide/releases
   ```

2. **Launch the IDE**
   ```bash
   rust-ai-ide
   ```

## Basic Usage

### Opening a Project

1. Click `File > Open Folder` or use `Ctrl+O` (Windows/Linux) / `Cmd+O` (macOS)
2. Select your Rust project directory

### Running Code

- **Run Current File**: `F5`
- **Debug**: `Shift+F5`
- **Run Tests**: `Ctrl+Shift+T` (Windows/Linux) / `Cmd+Shift+T` (macOS)

### AI Features

1. **Code Completion**: Start typing to see AI suggestions
2. **Code Actions**: Click the lightbulb (💡) or press `Ctrl+.`
3. **AI Chat**: Open the AI panel with `Ctrl+Shift+P` > "AI: Toggle Panel"

## Configuration

Edit your settings in `~/.config/rust-ai-ide/config.toml` or through the UI:

```toml
[editor]
theme = "dark"
font_size = 14

[ai]
model = "gpt-4"
max_tokens = 2048

extensions = [
  "rust-analyzer",
  "gitlens",
  "errorlens"
]
```

## Next Steps

- [User Guide](../user-guide/README.md)
- [Configuration Guide](CONFIGURATION.md)
- [Troubleshooting](../user-guide/TROUBLESHOOTING.md)

## Getting Help

- [Documentation](https://rust-ai-ide.example.com/docs)
- [GitHub Issues](https://github.com/your-org/rust-ai-ide/issues)
- [Community Forum](https://community.rust-ai-ide.example.com)
EOL

# Create a simple PLUGINS.md in development directory
cat > docs/src/development/PLUGINS.md << 'EOL'
# Plugin Development

## Overview

Rust AI IDE supports a powerful plugin system that allows extending the IDE's functionality.

## Getting Started

1. **Create a new plugin project**
   ```bash
   cargo new --lib my-plugin
   cd my-plugin
   ```

2. **Add dependencies** to `Cargo.toml`:
   ```toml
   [package]
   name = "my-plugin"
   version = "0.1.0"
   edition = "2021"
   
   [lib]
   crate-type = ["cdylib"]
   
   [dependencies]
   rust-ai-ide-plugin = "0.1"
   ```

## Basic Plugin

```rust
use rust_ai_ide_plugin::*;

#[no_mangle]
pub extern "C" fn register() -> Plugin {
    Plugin {
        name: "My Plugin".into(),
        version: "0.1.0".into(),
        hooks: vec![
            Hook::new("on_activate", on_activate),
            Hook::new("on_editor_open", on_editor_open),
        ],
    }
}

fn on_activate(_context: PluginContext) -> PluginResult {
    println!("Plugin activated!");
    Ok(())
}

fn on_editor_open(context: PluginContext) -> PluginResult {
    if let Some(file_path) = context.file_path {
        println!("Editor opened: {}", file_path);
    }
    Ok(())
}
```

## Building and Installing

```bash
# Build in release mode
cargo build --release

# Install the plugin
mkdir -p ~/.config/rust-ai-ide/plugins
cp target/release/libmy_plugin.so ~/.config/rust-ai-ide/plugins/
```

## Available Hooks

- `on_activate`: When the plugin is loaded
- `on_deactivate`: When the plugin is unloaded
- `on_editor_open`: When a file is opened
- `on_save`: When a file is saved
- `on_command`: When a command is executed

## Plugin API

See the [Plugin API Reference](../api/PLUGIN_API.md) for detailed documentation.

## Best Practices

- **Error Handling**: Always handle errors gracefully
- **Performance**: Keep operations non-blocking
- **Logging**: Use the provided logging utilities
- **Testing**: Write unit tests for your hooks

## Example Plugins

- [Code Formatter](https://github.com/example/code-formatter)
- [Git Integration](https://github.com/example/git-integration)
- [Theme Manager](https://github.com/example/theme-manager)

## Troubleshooting

- **Plugin not loading**: Check `~/.config/rust-ai-ide/logs/plugin.log`
- **Version Mismatch**: Ensure your plugin is compatible with the current IDE version
- **Debugging**: Run the IDE with `RUST_LOG=debug` for detailed logs

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.
EOL

# Create a simple README.md in the api directory
cat > docs/src/api/README.md << 'EOL'
# API Reference

Welcome to the Rust AI IDE API documentation. This section provides detailed information about the available APIs for extending and customizing the IDE.

## Getting Started

### Core Concepts

- **Editor**: The main text editing interface
- **Workspace**: Manages projects and files
- **Commands**: Actions that can be executed
- **Events**: System and user actions that can be listened to

## Core APIs

- [Editor API](CORE_API.md) - Interact with the text editor
- [Workspace API](WORKSPACE_API.md) - Manage projects and files
- [UI Components](UI_API.md) - Create and manage UI elements

## AI APIs

- [Code Completion](AI_API.md#code-completion) - AI-powered code suggestions
- [Code Analysis](AI_API.md#code-analysis) - Static code analysis
- [Refactoring](AI_API.md#refactoring) - Automated code transformations

## Plugin API

- [Plugin Architecture](../development/PLUGINS.md) - Overview of the plugin system
- [Lifecycle Hooks](PLUGIN_API.md#lifecycle-hooks) - Respond to IDE events
- [Event System](PLUGIN_API.md#event-system) - Subscribe to and emit events

## Examples

### Basic Plugin

```rust
use rust_ai_ide_plugin::*;

#[no_mangle]
pub extern "C" fn register() -> Plugin {
    Plugin {
        name: "My Plugin".into(),
        version: "0.1.0".into(),
        hooks: vec![
            Hook::new("on_activate", on_activate),
        ],
    }
}

fn on_activate(_context: PluginContext) -> PluginResult {
    println!("Plugin activated!");
    Ok(())
}
```

## Best Practices

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Document all public APIs
- Use semantic versioning
- Write tests for all functionality

## Support

For API-related questions, please [open an issue](https://github.com/your-org/rust-ai-ide/issues).

## Versioning

This API follows [Semantic Versioning](https://semver.org/).

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
EOL

# Create a simple README.md in the user-guide directory
cat > docs/src/user-guide/README.md << 'EOL'
# User Guide

Welcome to the Rust AI IDE user guide! This guide will help you get started with the IDE and its features.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Usage](BASIC_USAGE.md)
3. [Features](../features/README.md)
4. [Troubleshooting](TROUBLESHOOTING.md)

## Getting Started

### Installation

1. Download the latest release from [our website](https://rust-ai-ide.example.com)
2. Follow the [installation guide](../getting-started/INSTALLATION.md)
3. Launch the IDE

### First Run

When you first launch the IDE:

1. Select a theme
2. Choose your preferred keybindings
3. Install recommended extensions

## Basic Navigation

- **Command Palette**: `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (macOS)
- **File Explorer**: `Ctrl+Shift+E` (Windows/Linux) or `Cmd+Shift+E` (macOS)
- **Search**: `Ctrl+Shift+F` (Windows/Linux) or `Cmd+Shift+F` (macOS)

## Features

- [AI-Powered Code Completion](../features/ai-features.md)
- [Version Control Integration](../features/version-control.md)
- [Debugging Tools](../features/debugging.md)
- [Terminal Integration](../features/terminal.md)

## Customization

### Settings

Access settings via `File > Preferences > Settings` or `Ctrl+,` (Windows/Linux) / `Cmd+,` (macOS).

### Keybindings

Customize keybindings in `File > Preferences > Keyboard Shortcuts`.

## Support

- [Troubleshooting Guide](TROUBLESHOOTING.md)
- [FAQ](../faq.md)
- [Report Issues](https://github.com/your-org/rust-ai-ide/issues)

## Feedback

We'd love to hear your feedback! Please share your thoughts on our [community forum](https://community.rust-ai-ide.example.com).
EOL

# Create a simple README.md in the features directory
cat > docs/src/features/README.md << 'EOL'
# Features

Rust AI IDE comes packed with powerful features to enhance your development experience.

## Core Features

- [AI-Powered Code Completion](ai-features.md)
- [Intelligent Refactoring](refactoring.md)
- [Version Control Integration](version-control.md)
- [Debugging Tools](debugging.md)
- [Terminal Integration](terminal.md)

## AI Features

- **Code Completion**: Context-aware suggestions as you type
- **Code Generation**: Generate code from natural language
- **Code Analysis**: Identify potential issues and improvements
- **Documentation**: Generate and improve documentation

## Productivity

- **Multiple Cursors**: Edit in multiple places at once
- **Command Palette**: Quick access to all commands
- **Snippets**: Code templates for common patterns
- **Themes**: Customize the look and feel

## Extensibility

- **Plugins**: Extend functionality with plugins
- **Themes**: Create custom themes
- **Keybindings**: Customize keyboard shortcuts

## Getting Help

- [User Guide](../user-guide/README.md)
- [API Reference](../api/README.md)
- [Troubleshooting](../user-guide/TROUBLESHOOTING.md)

## Feedback

We're always looking to improve! Please share your feedback on our [community forum](https://community.rust-ai-ide.example.com).
EOL

# Fix any remaining broken links in the documentation
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/development\/PLUGINS\.md/..\/development\/PLUGINS.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/user-guide\/README\.md/..\/user-guide\/README.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/api\/README\.md/..\/api\/README.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/QUICKSTART\.md/QUICKSTART.html/g' {} \;

echo "Fixed all remaining documentation issues!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation."
