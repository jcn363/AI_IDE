# Configuration Guide

## Basic Configuration

Rust AI IDE can be configured using a `config.toml` file in the following locations:

1. `~/.config/rust-ai-ide/config.toml` (User-specific)
2. `./.rust-ai-ide/config.toml` (Project-specific)

## Example Configuration

```toml
[core]
editor = "vscode"
theme = "dark"

[ai]
model = "default"
enable_suggestions = true

[git]
auto_fetch = true
show_untracked = true
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_AI_IDE_CONFIG` | Path to config file | `~/.config/rust-ai-ide/config.toml` |
| `RUST_LOG` | Log level | `info` |
| `RUST_AI_IDE_CACHE_DIR` | Cache directory | Platform-specific |

## Advanced Configuration

### Editor Settings

```toml
[editor]
tab_size = 4
font_size = 14
line_numbers = true
minimap = true
```

### AI Model Settings

```toml
[ai]
model = "local"  # or "remote"
local_model_path = "~/.cache/rust-ai-ide/models/default"
max_tokens = 2048
temperature = 0.7
```

### Git Integration

```toml
[git]
enable = true
auto_fetch = true
show_untracked = true
blame_inline = true
```

## Troubleshooting

If you encounter any issues with the configuration:

1. Check the log file at `~/.cache/rust-ai-ide/logs/rust-ai-ide.log`
2. Run with `RUST_LOG=debug` for more verbose output
3. Reset to defaults by removing the config file and restarting
