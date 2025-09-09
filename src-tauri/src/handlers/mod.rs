//! Command handlers directory
//!
//! This module contains all Tauri command handlers organized by domain.

pub mod ai;
pub mod cargo;
pub mod documentation;
pub mod fs;
pub mod git;
pub mod lsp;
pub mod project;
pub mod terminal;
pub mod testing;

// Re-export all handlers for Tauri command generation
//
// Use with: tauri::generate_handler![
//     handlers::fs::*,
//     handlers::git::*,
//     handlers::*::*,
//     // ... etc
// ]