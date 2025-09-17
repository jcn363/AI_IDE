# Workspace API

This document describes the Workspace API for interacting with the Rust AI IDE's workspace, files, and projects.

## Overview

The Workspace API provides functionality to:

- Manage files and directories
- Search and navigate code
- Access workspace settings
- Handle workspace events
- Manage projects and dependencies

## Core Concepts

### Workspace

Represents the root of the workspace, providing access to files, folders, and configuration.

### FileSystem

Interface for file system operations, including reading, writing, and watching files.

### WorkspaceFolders

Collection of workspace folders in a multi-root workspace.

### WorkspaceConfiguration

Access to workspace-specific settings and configurations.

## Basic Usage

### Accessing the Workspace

```rust
use rust_ai_ide_api::workspace::Workspace;

let workspace = Workspace::current();
let root_path = workspace.root_path();
println!("Workspace root: {}", root_path.display());
```

### Reading a File

```rust
use rust_ai_ide_api::workspace::FileSystem;

let fs = FileSystem::new();
let content = fs.read_to_string("src/main.rs")?;
println!("File content: {}", content);
```

### Watching for File Changes

```rust
use rust_ai_ide_api::workspace::{FileSystem, FileChangeType};

let fs = FileSystem::new();
let watcher = fs.watch("src/**/*.rs", |event| {
    match event.change_type {
        FileChangeType::Created => println!("File created: {}", event.path.display()),
        FileChangeType::Changed => println!("File changed: {}", event.path.display()),
        FileChangeType::Deleted => println!("File deleted: {}", event.path.display()),
    }
});
```

## API Reference

### Workspace

```rust
impl Workspace {
    pub fn current() -> Self;
    pub fn root_path(&self) -> &Path;
    pub fn find_files(&self, pattern: &str) -> Vec<PathBuf>;
    pub fn get_configuration(&self) -> WorkspaceConfiguration;
    pub fn save_all(&self) -> Result<(), Error>;
}
```

### FileSystem

```rust
impl FileSystem {
    pub fn new() -> Self;
    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String>;
    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()>;
    pub fn watch<F>(&self, pattern: &str, callback: F) -> FileSystemWatcher
    where F: Fn(FileChangeEvent) + 'static;
}

pub struct FileChangeEvent {
    pub path: PathBuf,
    pub change_type: FileChangeType,
}

pub enum FileChangeType {
    Created,
    Changed,
    Deleted,
}
```

## Best Practices

1. **Error Handling**: Always handle potential errors from file system operations
2. **Resource Management**: Clean up watchers and other resources when done
3. **Performance**: Be mindful of file system operations in performance-critical code
4. **Concurrency**: Use appropriate synchronization when accessing shared resources

## Examples

### Finding All Rust Files

```rust
let workspace = Workspace::current();
let rust_files = workspace.find_files("**/*.rs");
for file in rust_files {
    println!("Found Rust file: {}", file.display());
}
```

### Reading Configuration

```rust
let config = Workspace::current().get_configuration();
let tab_size: usize = config.get("editor.tabSize").unwrap_or(4);
println!("Tab size: {}", tab_size);
```

## Troubleshooting

- **File Not Found**: Check if the path is relative to the workspace root
- **Permission Denied**: Ensure the IDE has proper file system permissions
- **Performance Issues**: Minimize file system operations in loops or frequent callbacks

## Learn More

- [Rust Path API](https://doc.rust-lang.org/std/path/)
- [File System Operations in Rust](https://doc.rust-lang.org/std/fs/)
- [The Rust I/O Guide](https://doc.rust-lang.org/stable/rust-by-example/std_misc/fs.html)
