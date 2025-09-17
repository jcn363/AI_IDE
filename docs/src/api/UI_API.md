# UI Components API

This document describes the UI Components API for extending the Rust AI IDE's user interface.

## Overview

The UI Components API allows you to create, modify, and interact with the IDE's user interface elements. This includes:

- Status bar items
- Sidebar views
- Editor decorations
- Context menus
- Modal dialogs

## Core Concepts

### UI Components

- **StatusBarItem**: Shows information in the status bar
- **ViewContainer**: A container for custom views in the sidebar
- **Decoration**: Visual decorations for the editor
- **ContextMenu**: Custom context menu items
- **ModalDialog**: Custom dialog windows

## Basic Usage

### Creating a Status Bar Item

```rust
use rust_ai_ide_api::ui::{StatusBarItem, StatusBarAlignment};

let item = StatusBarItem::new("my.extension.status", StatusBarAlignment::Right, 100);
item.text = "Hello, World!".to_string();
item.show();
```

### Adding a Custom View

```rust
use rust_ai_ide_api::ui::{ViewContainer, ViewContainerLocation};

let container = ViewContainer::new("my.extension.view", "My Custom View");
container.location = ViewContainerLocation::Explorer;
container.render(|ui| {
    ui.label("This is my custom view");
});
container.register();
```

## API Reference

### StatusBarItem

```rust
impl StatusBarItem {
    pub fn new(id: &str, alignment: StatusBarAlignment, priority: i32) -> Self;
    pub fn show(&self);
    pub fn hide(&self);
    pub fn dispose(self);
}

pub enum StatusBarAlignment {
    Left,
    Right
}
```

### ViewContainer

```rust
impl ViewContainer {
    pub fn new(id: &str, title: &str) -> Self;
    pub fn location(self, location: ViewContainerLocation) -> Self;
    pub fn render<F: Fn(&mut Ui) + 'static>(self, render_fn: F) -> Self;
    pub fn register(self) -> RegisteredViewContainer;
}

pub enum ViewContainerLocation {
    Explorer,
    Debug,
    SourceControl,
    Custom(&'static str)
}
```

## Best Practices

1. **Clean Up**: Always dispose of UI components when they're no longer needed
2. **Performance**: Keep UI updates efficient, especially in render callbacks
3. **Consistency**: Follow the IDE's design patterns and styles
4. **Accessibility**: Ensure your UI is accessible to all users

## Examples

### Context Menu Item

```rust
use rust_ai_ide_api::ui::ContextMenu;

let menu = ContextMenu::new("my.extension.context");
menu.add_item("Say Hello", |_| {
    println!("Hello from context menu!");
});
menu.register();
```

### Modal Dialog

```rust
use rust_ai_ide_api::ui::ModalDialog;

let dialog = ModalDialog::new("My Dialog");
dialog.set_content(|ui| {
    ui.label("This is a custom dialog");
    if ui.button("Close").clicked() {
        dialog.close();
    }
});
dialog.show();
```

## Troubleshooting

- **UI Not Updating**: Ensure you're on the main thread when updating UI
- **Memory Leaks**: Always dispose of UI components when done
- **Performance Issues**: Minimize re-renders and heavy computations in render callbacks

## Learn More

- [Rust UI Guidelines](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)
- [egui Documentation](https://docs.rs/egui/latest/egui/)
- [Iced Framework](https://github.com/iced-rs/iced)
