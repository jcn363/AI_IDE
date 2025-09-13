use rust_ai_ide_debugger::debugger::error::DebuggerResult;
use rust_ai_ide_debugger::debugger::expressions::ExpressionManager;
use rust_ai_ide_debugger::debugger::types::DebuggerState;

#[test]
fn test_add_watch_expression() -> DebuggerResult<()> {
    let mut manager = ExpressionManager::new();

    // Test adding a valid expression
    let id = manager.add_watch_expression("x")?;
    assert_eq!(id, 1);

    // Test adding a duplicate expression
    assert!(manager.add_watch_expression("x").is_err());

    // Test adding an empty expression
    assert!(manager.add_watch_expression("").is_err());

    // Test getting the expression by ID
    let expr = manager.get_watch_expression(id)?;
    assert_eq!(expr.expression.as_deref(), Some("x"));

    Ok(())
}

#[test]
fn test_remove_watch_expression() -> DebuggerResult<()> {
    let mut manager = ExpressionManager::new();

    // Add some expressions
    let id1 = manager.add_watch_expression("x")?;
    let id2 = manager.add_watch_expression("y")?;

    // Remove by ID
    manager.remove_watch_expression_by_id(id1)?;
    assert_eq!(manager.get_watch_expressions().len(), 1);

    // Remove by expression text
    assert!(manager.remove_watch_expression("y").is_some());
    assert!(manager.get_watch_expressions().is_empty());

    // Test removing non-existent expression
    assert!(manager.remove_watch_expression("nonexistent").is_none());
    assert!(manager.remove_watch_expression_by_id(999).is_err());

    Ok(())
}

#[test]
fn test_evaluate_all() -> DebuggerResult<()> {
    let mut manager = ExpressionManager::new();

    // Add some expressions
    manager.add_watch_expression("x")?;
    manager.add_watch_expression("y")?;

    // Evaluate in running state (should do nothing)
    let events = manager.evaluate_all(&DebuggerState::Running)?;
    assert!(events.is_empty());

    // Evaluate in paused state
    let state = DebuggerState::Paused {
        reason: "breakpoint".to_string(),
        location: Some(("test.rs".to_string(), 10)),
    };

    let events = manager.evaluate_all(&state)?;
    assert_eq!(events.len(), 2); // Should get update events for both expressions

    // Evaluate again with no changes (should produce no new events)
    let events = manager.evaluate_all(&state)?;
    assert!(events.is_empty());

    Ok(())
}

#[test]
fn test_auto_evaluate_setting() {
    let mut manager = ExpressionManager::new();
    assert!(manager.auto_evaluate);

    manager.set_auto_evaluate(false);
    assert!(!manager.auto_evaluate);

    manager.set_auto_evaluate(true);
    assert!(manager.auto_evaluate);
}
