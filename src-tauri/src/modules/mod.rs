//! Modular organization for the Rust AI IDE back-end
//!
//! This module provides a clear separation of concerns with organized sub-modules:
//! - core: Core IDE functionality
//! - cargo: Cargo integration and management
//! - ai: AI-powered features and services
//! - terminal: Terminal emulation and management
//! - filesystem: File system operations and utilities
//! - shared: Shared types and utilities

pub mod ai;
pub mod cargo;
pub mod core;
pub mod filesystem;
pub mod shared;
pub mod terminal;
