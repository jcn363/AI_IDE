#!/bin/bash
set -e

# Create a simple README for the development directory
cat > docs/src/development/README.md << 'EOL'
# Development Guide

Welcome to the Rust AI IDE development guide. This section provides information for developers who want to contribute to the project.

## Getting Started

1. **Prerequisites**
   - Rust 1.65 or later
   - Node.js 16+ (for web components)
   - Git

2. **Building the Project**
   ```bash
   git clone https://github.com/your-org/rust-ai-ide.git
   cd rust-ai-ide
   cargo build
   ```

3. **Running Tests**
   ```bash
   cargo test
   ```

## Project Structure

- `/crates`: Core Rust crates
- `/web`: Web-based frontend
- `/docs`: Documentation
- `/scripts`: Build and development scripts

## Contributing

Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to contribute to the project.

## Development Workflow

1. Create a new branch for your feature or bugfix
2. Make your changes
3. Run tests
4. Submit a pull request

## Code Style

We follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
EOL

# Create a simple README for the user guide
cat > docs/src/user-guide/README.md << 'EOL'
# User Guide

Welcome to the Rust AI IDE User Guide. This guide will help you get started with the IDE and its features.

## Getting Started

- [Installation](../getting-started/INSTALLATION.html)
- [Quick Start](../getting-started/QUICKSTART.html)
- [Configuration](../getting-started/CONFIGURATION.html)

## Features

- [AI Features](../features/ai-features.html)
- [Version Control](../features/version-control.html)
- [Debugging](../features/debugging.html)
- [Terminal](../features/terminal.html)

## Customization

- [Settings](SETTINGS.md)
- [Keybindings](KEYBINDINGS.md)
- [Themes](THEMES.md)

## Troubleshooting

See the [Troubleshooting Guide](TROUBLESHOOTING.md) for help with common issues.

## Support

For additional help, please visit our [FAQ](../faq.html) or [file an issue](https://github.com/your-org/rust-ai-ide/issues).
EOL

# Create a simple settings guide
cat > docs/src/user-guide/SETTINGS.md << 'EOL'
# Settings

Customize Rust AI IDE to suit your workflow with these settings.

## User vs Workspace Settings

- **User Settings**: Apply globally to all your projects
- **Workspace Settings**: Apply only to the current workspace

## Common Settings

### Editor

```json
{
    "editor.fontSize": 14,
    "editor.tabSize": 4,
    "editor.wordWrap": "on",
    "editor.minimap.enabled": true
}
```

### Terminal

```json
{
    "terminal.integrated.fontSize": 14,
    "terminal.integrated.fontFamily": "'Fira Code', 'Droid Sans Mono', 'Courier New'"
}
```

### AI Features

```json
{
    "ai.enabled": true,
    "ai.model": "gpt-4",
    "ai.maxTokens": 2048
}
```

## Custom Keybindings

See [Keybindings](KEYBINDINGS.md) for information on customizing keyboard shortcuts.

## Sync Settings

Your settings can be synced across devices using the built-in settings sync.

## Troubleshooting

If settings don't take effect:
1. Check for JSON syntax errors
2. Ensure you're editing the correct settings file
3. Restart the IDE if needed

## Learn More

- [Full Settings Reference](https://rust-ai-ide.example.com/docs/settings)
- [Configuration Tips](https://rust-ai-ide.example.com/docs/configuration-tips)
EOL

# Create a simple keybindings guide
cat > docs/src/user-guide/KEYBINDINGS.md << 'EOL'
# Keybindings

Customize keyboard shortcuts to match your workflow.

## Default Keybindings

### General

| Command | Windows/Linux | macOS |
|---------|---------------|-------|
| Command Palette | `Ctrl+Shift+P` | `Cmd+Shift+P` |
| Quick Open | `Ctrl+P` | `Cmd+P` |
| New Terminal | `Ctrl+`` | `Cmd+`` |
| Toggle Sidebar | `Ctrl+B` | `Cmd+B` |

### Editor

| Command | Windows/Linux | macOS |
|---------|---------------|-------|
| Go to Definition | `F12` | `F12` |
| Find | `Ctrl+F` | `Cmd+F` |
| Replace | `Ctrl+H` | `Cmd+H` |
| Toggle Line Comment | `Ctrl+/` | `Cmd+/` |
| Format Document | `Shift+Alt+F` | `Shift+Option+F` |

### Debugging

| Command | Windows/Linux | macOS |
|---------|---------------|-------|
| Start/Continue | `F5` | `F5` |
| Step Over | `F10` | `F10` |
| Step Into | `F11` | `F11` |
| Step Out | `Shift+F11` | `Shift+F11` |

## Customizing Keybindings

1. Open the Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`)
2. Type "Preferences: Open Keyboard Shortcuts (JSON)"
3. Add your custom keybindings

Example:

```json
[
    {
        "key": "ctrl+shift+t",
        "command": "workbench.action.terminal.new",
        "when": "terminal.active"
    }
]
```

## Keybinding Conflicts

If a keybinding doesn't work as expected:
1. Check for conflicts in the Keyboard Shortcuts editor
2. Look for extensions that might override default keybindings
3. Reset to default keybindings if needed

## Platform-Specific Notes

- On macOS, use `Cmd` instead of `Ctrl` for most commands
- Some keybindings may be different in Linux distributions

## Learn More

- [Keybinding Syntax](https://code.visualstudio.com/docs/getstarted/keybindings)
- [Common Keybindings](https://code.visualstudio.com/shortcuts/keyboard-shortcuts-windows.pdf)
EOL

# Create a simple themes guide
cat > docs/src/user-guide/THEMES.md << 'EOL'
# Themes

Customize the appearance of Rust AI IDE with themes.

## Built-in Themes

### Light Themes
- Light+ (default light)
- Light (Visual Studio)
- Solarized Light

### Dark Themes
- Dark+ (default dark)
- Dark (Visual Studio)
- Monokai
- Solarized Dark
- GitHub Dark

### High Contrast
- High Contrast Light
- High Contrast Dark

## Changing the Theme

1. Open the Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`)
2. Type "Preferences: Color Theme"
3. Select your preferred theme

## Custom Themes

1. Create a theme extension using the Extension API
2. Install a theme from the marketplace
3. Load a custom theme file (`.tmTheme` or `.json`)

## Customizing Theme Colors

1. Open Settings (`Ctrl+,` or `Cmd+,`)
2. Search for "workbench.colorCustomizations"
3. Add your custom colors

Example:

```json
{
    "workbench.colorCustomizations": {
        "titleBar.activeBackground": "#1a1a1a",
        "titleBar.activeForeground": "#ffffff"
    }
}
```

## Icon Themes

Change file and folder icons:
1. Open the Command Palette
2. Type "Preferences: File Icon Theme"
3. Select an icon theme

## Syntax Highlighting

Syntax colors are determined by the current theme. To customize:

1. Open Settings (JSON)
2. Add `editor.tokenColorCustomizations`

## Contributing Themes

To contribute a new theme:
1. Create a theme extension
2. Follow the [Theme Authoring Guide](https://code.visualstudio.com/api/extension-guides/color-theme)
3. Submit a pull request

## Troubleshooting

- If colors don't update, try reloading the window
- Check the developer console for errors
- Verify theme file syntax

## Learn More

- [Theme Color Reference](https://code.visualstudio.com/api/references/theme-color)
- [Creating a Theme](https://code.visualstudio.com/api/extension-guides/color-theme)
- [Theme Marketplace](https://marketplace.visualstudio.com/search?term=theme&target=VSCode)
EOL

# Fix remaining broken links in markdown files
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

# Create a simple README for the API directory
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

echo "Fixed all remaining documentation links and created missing files!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation."
