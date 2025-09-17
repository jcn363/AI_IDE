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
