//! Module containing all Tauri command definitions.
//!
//! This module exports all command modules and provides the central
//! command registration point.

pub mod ai_analysis_commands;
pub mod analysis;
pub mod collaboration;
pub mod cargo_lock_commands;
pub mod cargo;
pub mod commands;
pub mod compiler_integration;
pub mod core_diagnostics;
pub mod dependency_commands;
pub mod debugger;
pub mod integrations;
pub mod io;
pub mod lsp;
pub mod model_commands;
pub mod performance;
pub mod plugins;
pub mod processing_helpers;
pub mod project;
pub mod refactoring_commands;
pub mod search;
pub mod security;
pub mod streaming_cache;
pub mod terminal;
pub mod types;
pub mod utils;

pub mod keyboard;
pub mod multicursor;
pub mod split_view;

//! Re-export commonly used items for easier access
pub use keyboard::{
    get_keybindings_profile,
    create_keybindings_profile,
    update_keybinding_profile,
    delete_keybindings_profile,
    get_available_actions,
    validate_keybinding_conflicts,
    apply_keybindings_profile,
    export_keybindings,
    import_keybindings,
    reset_to_defaults,
    get_conflicts,
};

pub use multicursor::{
    add_cursor_at_position,
    remove_cursor_at_position,
    remove_all_secondary_cursors,
    find_all_occurrences,
    select_word_boundaries,
    add_cursors_on_line_ends,
    get_cursor_state,
    update_document_version,
};

pub use split_view::{
    split_panel,
    close_panel,
    add_tab_to_panel,
    remove_tab_from_panel,
    set_focused_panel,
    get_layout,
    save_layout,
    load_layout,
};