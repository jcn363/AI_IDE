# Terminal Integration

Rust AI IDE includes a fully-featured integrated terminal that allows you to run command-line tools and scripts without leaving the editor.

## Features

- **Multiple Terminals**: Create and manage multiple terminal instances
- **Shell Integration**: Works with your default shell (Bash, Zsh, PowerShell, etc.)
- **Command History**: Access previous commands with up/down arrows
- **Customization**: Configure terminal appearance and behavior
- **Task Automation**: Define and run common tasks

## Getting Started

### Opening a Terminal

- **New Terminal**: `Ctrl+`` (backtick) or `Ctrl+Shift+``
- **From Command Palette**: `Ctrl+Shift+P` then type "Terminal: Create New Terminal"

### Terminal Navigation

- **Split Terminal**: `Ctrl+Shift+5` or right-click and select "Split"
- **Switch Between Terminals**: `Alt+Left/Right`
- **Close Terminal**: Click the trash can icon or type `exit`

## Configuration

### Settings

Customize terminal behavior in `settings.json`:

```json
{
    "terminal.integrated.fontFamily": "'Fira Code', 'Droid Sans Mono', 'Courier New'",
    "terminal.integrated.fontSize": 14,
    "terminal.integrated.cursorStyle": "line",
    "terminal.integrated.scrollback": 10000
}
```

### Shell Configuration

Set your default shell in settings:

```json
{
    "terminal.integrated.defaultProfile.linux": "bash",
    "terminal.integrated.defaultProfile.osx": "zsh",
    "terminal.integrated.defaultProfile.windows": "PowerShell"
}
```

## Advanced Usage

### Keybindings

| Command | Windows/Linux | macOS |
|---------|---------------|-------|
| New Terminal | `Ctrl+`` | `Cmd+`` |
| Copy | `Ctrl+C` | `Cmd+C` |
| Paste | `Ctrl+V` | `Cmd+V` |
| Clear | `Ctrl+K` | `Cmd+K` |
| Find | `Ctrl+F` | `Cmd+F` |

### Environment Variables

Set environment variables in `settings.json`:

```json
{
    "terminal.integrated.env.linux": {
        "RUST_BACKTRACE": "1",
        "PATH": "${env:PATH}:/custom/path"
    }
}
```

## Troubleshooting

- **Terminal Not Opening**: Check your shell path in settings
- **Slow Performance**: Try disabling GPU acceleration
- **Character Encoding Issues**: Set the correct encoding in settings

## Learn More

- [Bash Reference](https://www.gnu.org/software/bash/manual/)
- [PowerShell Documentation](https://docs.microsoft.com/en-us/powershell/)
- [Zsh Documentation](http://zsh.sourceforge.net/Doc/)
