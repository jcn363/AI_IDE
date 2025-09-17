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

See [Keybindings](KEYBINDINGS.html) for information on customizing keyboard shortcuts.

## Sync Settings

Your settings can be synced across devices using the built-in settings sync.

## Troubleshooting

If settings don't take effect:
1. Check for JSON syntax errors
2. Ensure you're editing the correct settings file
3. Restart the IDE if needed

## Learn More

- [Full Settings Reference](https://rust-ai-ide.github.io/docs/settings)
- [Configuration Tips](https://rust-ai-ide.github.io/docs/configuration-tips)
