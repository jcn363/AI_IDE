use rust_ai_ide_debugger::debugger::state::StateManager;
use rust_ai_ide_debugger::debugger::types::{DebuggerState, StackFrame, VariableInfo};

#[test]
fn test_state_management() {
    let mut manager = StateManager::new();

    // Initial state should be Disconnected
    assert!(matches!(*manager.get_state(), DebuggerState::Disconnected));

    // Update state
    manager.set_state(DebuggerState::Running);
    assert!(matches!(*manager.get_state(), DebuggerState::Running));

    // Test call stack updates
    let stack = vec![StackFrame {
        id: 1,
        function: "main".to_string(),
        file: "main.rs".to_string(),
        line: 10,
        column: Some(5),
        args: Vec::new(),
        locals: Vec::new(),
    }];

    manager.update_call_stack(stack.clone());
    assert_eq!(manager.get_current_frame().unwrap().function, "main");

    // Test variable updates
    let vars = vec![VariableInfo {
        id: Some(1),
        name: "x".to_string(),
        value: "42".to_string(),
        type_name: "i32".to_string(),
        in_scope: true,
        children: Vec::new(),
        expression: None,
    }];

    manager.update_variables(vars);
    assert_eq!(manager.get_variable("x").unwrap().value, "42");
}
