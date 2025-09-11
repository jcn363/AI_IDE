//! File system watching module
//!
//! This module provides file system watching capabilities.
//! The main implementation is in the filesystem module, this is for backward compatibility.

pub use crate::filesystem::DefaultFileEventProcessor;
pub use crate::filesystem::FileEventProcessor;
pub use crate::filesystem::FileSystemWatcher;
