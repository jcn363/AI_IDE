# Quick Start Guide

## First Steps

1. **Install Rust AI IDE**
   ```bash
   # Using cargo
   cargo install rust-ai-ide
   
   # Or download from GitHub releases
   # Visit https://github.com/rust-ai-ide/rust-ai-ide/releases
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

- [User Guide](../user-guide/README.html)
- [Configuration Guide](configuration.html)
- [Troubleshooting](../user-guide/troubleshooting.html)

## Getting Help

- [Documentation](https://rust-ai-ide.github.io/docs)
- [GitHub Issues](https://github.com/rust-ai-ide/rust-ai-ide/issues)
- [Community Forum](https://github.com/rust-ai-ide/rust-ai-ide/discussions)
