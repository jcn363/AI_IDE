#!/bin/bash
set -e

# Create required directories
mkdir -p docs/src/{getting-started,user-guide,development,api,features,enterprise}

# 1. Create installation guide
cat > docs/src/getting-started/INSTALLATION.md << 'EOL'
# Installation Guide

## Prerequisites

- Rust (latest stable version)
- Cargo
- Node.js (for web interface)
- Git

## Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/rust-ai-ide.git
   cd rust-ai-ide
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Start the IDE:
   ```bash
   cargo run --release
   ```

## Building from Source

1. Install dependencies:
   ```bash
   sudo apt-get update
   sudo apt-get install -y build-essential cmake pkg-config
   ```

2. Build with all features:
   ```bash
   cargo build --release --all-features
   ```

## System Requirements

- Minimum: 4GB RAM, 2 CPU cores, 2GB disk space
- Recommended: 8GB+ RAM, 4+ CPU cores, SSD storage

## Configuration

See [Configuration Guide](CONFIGURATION.md) for detailed setup instructions.

## Troubleshooting

Common issues and solutions:

- **Build fails**: Ensure all dependencies are installed
- **Missing components**: Run `cargo update`
- **Permission errors**: Run with `sudo` if necessary

## Next Steps

- [Quick Start Guide](QUICKSTART.md)
- [Configuration Guide](CONFIGURATION.md)
- [User Guide](../user-guide/README.md)
EOL

# 2. Create plugins documentation
cat > docs/src/development/PLUGINS.md << 'EOL'
# Plugin Development

## Overview

Rust AI IDE supports a plugin system that allows extending the IDE's functionality.

## Creating a Plugin

1. Create a new library crate:
   ```bash
   cargo new --lib my-plugin
   cd my-plugin
   ```

2. Add to `Cargo.toml`:
   ```toml
   [package]
   name = "my-plugin"
   version = "0.1.0"
   edition = "2021"

   [lib]
   crate-type = ["cdylib"]

   [dependencies]
   rust_ai_ide_plugin = { git = "https://github.com/your-org/rust-ai-ide" }
   ```

## Plugin API

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
cargo build --release
cp target/release/libmy_plugin.so ~/.config/rust-ai-ide/plugins/
```

## Best Practices

- Keep plugins focused on a single responsibility
- Handle errors gracefully
- Document your plugin's functionality
- Follow semantic versioning

## Testing

Use the built-in test harness:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin() {
        let plugin = register();
        assert_eq!(plugin.name, "My Plugin");
    }
}
```
EOL

# 3. Create user guide
cat > docs/src/user-guide/README.md << 'EOL'
# User Guide

## Getting Started

1. [Installation](../getting-started/INSTALLATION.md)
2. [Configuration](../getting-started/CONFIGURATION.md)
3. [Basic Usage](BASIC_USAGE.md)

## Features

- [AI-Powered Code Completion](ai-features.md#code-completion)
- [Smart Refactoring](ai-features.md#refactoring)
- [Version Control Integration](version-control.md)
- [Debugging Tools](debugging.md)

## Customization

### Themes

1. Open Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`)
2. Type "Preferences: Color Theme"
3. Select your preferred theme

### Keybindings

Customize keybindings in `~/.config/rust-ai-ide/keybindings.json`:

```json
[
  {
    "key": "ctrl+shift+r",
    "command": "refactor.rename",
    "when": "editorTextFocus"
  }
]
```

## Productivity Tips

- Use `Ctrl+P` to quickly open files
- `Ctrl+Shift+F` to search across files
- `F12` to go to definition
- `Alt+Click` for multiple cursors

## Extensions

Enhance your experience with extensions:

1. Open Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`)
2. Search for extensions
3. Click Install

## Support

- [Troubleshooting](TROUBLESHOOTING.md)
- [FAQ](../faq.md)
- [Report Issues](https://github.com/your-org/rust-ai-ide/issues)
EOL

# 4. Create API documentation
cat > docs/src/api/README.md << 'EOL'
# API Reference

## Overview

Welcome to the Rust AI IDE API documentation. This section provides detailed information about the available APIs for extending and customizing the IDE.

## Core APIs

- [Editor API](CORE_API.md#editor-api)
- [Workspace API](CORE_API.md#workspace-api)
- [UI Components](CORE_API.md#ui-components)

## AI APIs

- [Code Completion](AI_API.md#code-completion)
- [Code Analysis](AI_API.md#code-analysis)
- [Refactoring](AI_API.md#refactoring)

## Plugin API

- [Plugin Development](../development/PLUGINS.md)
- [Lifecycle Hooks](PLUGIN_API.md#lifecycle-hooks)
- [Event System](PLUGIN_API.md#event-system)

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
EOL

echo "Missing documentation files have been generated successfully!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation."
