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
