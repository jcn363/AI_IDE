use rust_ai_ide_debugger::debugger::breakpoints::{BreakpointInfo, BreakpointManager};
use rust_ai_ide_debugger::debugger::error::DebuggerResult;
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

fn create_test_file() -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    File::create(&file_path).unwrap();
    (dir, file_path)
}

#[test]
fn test_add_and_remove_breakpoint() -> DebuggerResult<()> {
    let (_dir, file_path) = create_test_file();
    let mut manager = BreakpointManager::new();

    // Test adding a breakpoint
    let id = manager.add_breakpoint(&file_path, 10, None, None)?;
    assert_eq!(manager.get_breakpoints().len(), 1);

    // Test getting the breakpoint
    let bp = manager.get_breakpoint(id)?;
    assert_eq!(bp.line, 10);
    assert!(bp.enabled);

    // Test removing the breakpoint
    let removed = manager.remove_breakpoint(id)?;
    assert_eq!(removed.id, id);
    assert_eq!(manager.get_breakpoints().len(), 0);

    // Test removing non-existent breakpoint
    assert!(manager.remove_breakpoint(999).is_err());

    Ok(())
}

#[test]
fn test_toggle_breakpoint() -> DebuggerResult<()> {
    let (_dir, file_path) = create_test_file();
    let mut manager = BreakpointManager::new();
    let id = manager.add_breakpoint(&file_path, 10, None, None)?;

    // Initially should be enabled
    assert!(manager.get_breakpoint(id)?.enabled);

    // Toggle off
    let new_state = manager.toggle_breakpoint(id)?;
    assert!(!new_state);
    assert!(!manager.get_breakpoint(id)?.enabled);

    // Toggle back on
    let new_state = manager.toggle_breakpoint(id)?;
    assert!(new_state);
    assert!(manager.get_breakpoint(id)?.enabled);

    Ok(())
}

#[test]
fn test_breakpoint_conditions() -> DebuggerResult<()> {
    let (_dir, file_path) = create_test_file();
    let mut manager = BreakpointManager::new();
    let id = manager.add_breakpoint(&file_path, 10, None, None)?;

    // Set a condition
    let prev_condition = manager.update_breakpoint_condition(id, Some("x > 5".to_string()))?;
    assert!(prev_condition.is_none());

    // Update the condition
    let prev_condition = manager.update_breakpoint_condition(id, Some("x > 10".to_string()))?;
    assert_eq!(prev_condition, Some("x > 5".to_string()));

    // Remove the condition
    let prev_condition = manager.update_breakpoint_condition(id, None)?;
    assert_eq!(prev_condition, Some("x > 10".to_string()));

    Ok(())
}

#[test]
fn test_breakpoint_hit_count() -> DebuggerResult<()> {
    let (_dir, file_path) = create_test_file();
    let mut manager = BreakpointManager::new();
    let id = manager.add_breakpoint(&file_path, 10, None, None)?;

    // Initial hit count should be 0
    assert_eq!(manager.get_breakpoint(id)?.hit_count, 0);

    // Increment hit count
    assert_eq!(manager.increment_hit_count(id)?, 1);
    assert_eq!(manager.increment_hit_count(id)?, 2);

    // Verify hit count
    assert_eq!(manager.get_breakpoint(id)?.hit_count, 2);

    Ok(())
}

#[test]
fn test_breakpoints_for_file() -> DebuggerResult<()> {
    let (_dir1, file1_path) = create_test_file();
    let (_dir2, file2_path) = create_test_file();

    let mut manager = BreakpointManager::new();

    // Add breakpoints to both files
    manager.add_breakpoint(&file1_path, 10, None, None)?;
    manager.add_breakpoint(&file1_path, 20, None, None)?;
    manager.add_breakpoint(&file2_path, 5, None, None)?;

    // Test getting breakpoints for file1
    let file1_bps = manager.get_breakpoints_for_file(&file1_path);
    assert_eq!(file1_bps.len(), 2);

    // Test removing breakpoints for file1
    let removed = manager.remove_breakpoints_for_file(&file1_path);
    assert_eq!(removed, 2);

    // Verify file1 breakpoints are gone, but file2's remain
    assert!(manager.get_breakpoints_for_file(&file1_path).is_empty());
    assert_eq!(manager.get_breakpoints_for_file(&file2_path).len(), 1);

    Ok(())
}

#[test]
fn test_invalid_breakpoint() -> DebuggerResult<()> {
    let mut manager = BreakpointManager::new();

    // Test adding breakpoint to non-existent file
    let result = manager.add_breakpoint("nonexistent.rs", 10, None, None);
    assert!(result.is_err());

    // Test adding breakpoint with line 0
    let (_dir, file_path) = create_test_file();
    let result = manager.add_breakpoint(&file_path, 0, None, None);
    assert!(result.is_err());

    Ok(())
}
