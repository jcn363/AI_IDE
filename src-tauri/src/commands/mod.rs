//! Module containing all Tauri command definitions.
//!
//! This module exports all command modules and provides the central
//! command registration point.

pub mod analysis;
pub mod cargo;
pub mod cargo_lock_commands;
pub mod collaboration;
pub mod compiler_integration;
pub mod debugger;
pub mod dependency_commands;
pub mod integrations;
pub mod io;
pub mod model_commands;
pub mod performance;
pub mod plugins;
pub mod project;
pub mod refactoring_commands;
pub mod search;
pub mod security;
pub mod terminal;
pub mod types;
pub mod warmup_predictor;

pub mod keyboard;
pub mod multicursor;
pub mod split_view;

/// Re-export commonly used items for easier access
pub use keyboard::{
    apply_keybindings_profile, create_keybindings_profile, delete_keybindings_profile, export_keybindings,
    get_available_actions, get_conflicts, get_keybindings_profile, import_keybindings, reset_to_defaults,
    update_keybinding_profile, validate_keybinding_conflicts,
};
pub use multicursor::{
    add_cursor_at_position, add_cursors_on_line_ends, find_all_occurrences, get_cursor_state,
    remove_all_secondary_cursors, remove_cursor_at_position, select_word_boundaries, update_document_version,
};
pub use split_view::{
    add_tab_to_panel, close_panel, get_layout, load_layout, remove_tab_from_panel, save_layout, set_focused_panel,
    split_panel,
};
