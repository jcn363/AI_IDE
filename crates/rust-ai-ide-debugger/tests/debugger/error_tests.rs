use std::io;

use rust_ai_ide_debugger::debugger::error::{DebuggerError, OptionExt};

#[test]
fn test_debugger_error_display() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let debugger_err = DebuggerError::from(io_err);
    assert!(format!("{}", debugger_err).contains("I/O error: file not found"));

    let proc_err = DebuggerError::process_error("process failed");
    assert_eq!(
        format!("{}", proc_err),
        "Debugger process error: process failed"
    );
}

#[test]
fn test_option_ext() {
    let some: Option<i32> = Some(42);
    assert_eq!(some.ok_or_err(|| "error").unwrap(), 42);

    let none: Option<i32> = None;
    assert_eq!(none.ok_or_err(|| "custom error"), Err("custom error"));
}
