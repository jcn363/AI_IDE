#!/bin/bash
set -e

# Fix broken links in markdown files
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/features\/ai-features\.md/..\/features\/ai-features.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/features\/security\.md/..\/features\/security.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/features\/collaboration\.md/..\/features\/collaboration.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/development\/PLUGINS\.md/..\/development\/PLUGINS.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/getting-started\/INSTALLATION\.md/..\/getting-started\/INSTALLATION.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/user-guide\/README\.md/..\/user-guide\/README.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/development\/CONTRIBUTING\.md/..\/development\/CONTRIBUTING.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/api\/README\.md/..\/api\/README.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/features\/version-control\.md/..\/features\/version-control.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/features\/debugging\.md/..\/features\/debugging.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/features\/terminal\.md/..\/features\/terminal.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/faq\.md/..\/faq.html/g' {} \;
find docs/src -name "*.md" -type f -exec sed -i 's/\.\.\/LICENSE/..\/LICENSE.html/g' {} \;

# Create a simple FAQ page
cat > docs/src/faq.md << 'EOL'
# Frequently Asked Questions

## General

### What is Rust AI IDE?
Rust AI IDE is an integrated development environment built with Rust that provides AI-powered coding assistance.

### Is it open source?
Yes, Rust AI IDE is open source and available under the MIT license.

## Installation

### What are the system requirements?
- Rust 1.65 or later
- 4GB RAM minimum (8GB recommended)
- 2GB disk space

### How do I update to the latest version?
Run `rustup update` to update Rust, then reinstall the IDE.

## Features

### What AI models does it support?
The IDE supports various AI models including OpenAI's GPT models and local models via the API.

### Can I use my own AI models?
Yes, you can configure the IDE to use your own models through the settings.

## Troubleshooting

### The IDE is running slowly
Try disabling some extensions or increasing the memory allocation in settings.

### I found a bug
Please report it on our [GitHub issues](https://github.com/your-org/rust-ai-ide/issues) page.

## Support

### Where can I get help?
Visit our [community forum](https://community.rust-ai-ide.example.com) or check the [documentation](https://rust-ai-ide.example.com/docs).
EOL

# Create a simple README for the API
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

echo "Fixed all remaining documentation links!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation."bin/bash
set -e

# Create missing files
touch docs/src/development/PLUGINS.md
cat > docs/src/development/PLUGINS.md << 'EOL'
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
   rust_ai_ide_plugin = { git = "https://github.com/your-org/rust-ai-ide" }
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

- [Code Formatter](https://github.com/example/code-formatter)
- [Git Integration](https://github.com/example/git-integration)
- [Theme Manager](https://github.com/example/theme-manager)

## Troubleshooting

- **Plugin not loading**: Check the logs at `~/.config/rust-ai-ide/logs/plugin.log`
- **API version mismatch**: Ensure your plugin is compatible with the current IDE version
- **Permission issues**: Make sure the plugin file has execute permissions

## Contributing

We welcome contributions! Please see our [Contributing Guide](../development/CONTRIBUTING.md) for details.
EOL

# Create a simple user guide README
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

# Create a simple API README
cat > docs/src/api/README.md << 'EOL'
# API Reference

Welcome to the Rust AI IDE API documentation. This section provides detailed information about the available APIs for extending and customizing the IDE.

## Core APIs

- [Editor API](CORE_API.md) - Interact with the text editor
- [Workspace API](WORKSPACE_API.md) - Manage projects and files
- [UI Components](UI_API.md) - Create and manage UI elements

## AI APIs

- [Code Completion](AI_API.md#code-completion) - AI-powered code suggestions
- [Code Analysis](AI_API.md#code-analysis) - Static code analysis
- [Refactoring](AI_API.md#refactoring) - Automated code transformations

## Plugin API

- [Plugin Development](../development/PLUGINS.md) - Create custom plugins
- [Lifecycle Hooks](PLUGIN_API.md#lifecycle-hooks) - Respond to IDE events
- [Event System](PLUGIN_API.md#event-system) - Subscribe to and emit events

## Getting Started

1. [Set up your development environment](../development/README.md)
2. [Create your first plugin](../development/PLUGINS.md)
3. [Test your integration](../development/TESTING.md)

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

echo "Created missing documentation files!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation."
